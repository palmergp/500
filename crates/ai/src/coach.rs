//! The coach: analyzes a decision from the player's seat, grades the choice they made, and
//! explains the difference in plain language.
//!
//! Grades use only what the player could know at the time: the coach sees the same
//! [`Observation`] the player saw and scores every option on the same sampled worlds. The
//! hindsight view (every card visible) is kept separate, in [`dd_trace`].

use crate::belief::Belief;
use crate::cardplay::{is_boss, trumps};
use crate::evaluate::has_bower;
use crate::thumb;
use crate::rule_bot::{heuristic_discards, RuleBot, Style};
use crate::search_bot::{Goal, PlayStat, PlayStats, SearchBot, SearchConfig};
use crate::solver::{Position, Solver};
use crate::Bot;
use fh_core::rules::is_trump;
use fh_core::trick::trick_strength;
use fh_core::{Action, Call, Card, CardSet, Contract, HandRecord, Observation, Phase, Play, Seat, Strain, Trick};

/// The lab's rule of thumb: partner and the kitty are worth about 2.7 tricks (docs/lab/README.md).
pub const PARTNER_AND_KITTY: f32 = 2.7;

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Verdict {
    Best,
    Good,
    Inaccuracy,
    Mistake,
    Blunder,
}

impl Verdict {
    pub fn label(self) -> &'static str {
        match self {
            Verdict::Best => "best",
            Verdict::Good => "good",
            Verdict::Inaccuracy => "inaccuracy",
            Verdict::Mistake => "mistake",
            Verdict::Blunder => "blunder",
        }
    }

    /// From expected points lost. Bids are judged more loosely: their estimates are noisier.
    fn from_loss(loss: f64, bidding: bool) -> Verdict {
        let scale = if bidding { 1.6 } else { 1.0 };
        match loss / scale {
            l if l < 2.0 => Verdict::Best,
            l if l < 10.0 => Verdict::Good,
            l if l < 25.0 => Verdict::Inaccuracy,
            l if l < 60.0 => Verdict::Mistake,
            _ => Verdict::Blunder,
        }
    }
}

/// One option the coach considered.
#[derive(Clone, Debug)]
pub struct OptionEval {
    pub action: Action,
    /// Expected points for the player's side (relative: compare options, not absolutes).
    pub ev: f64,
    /// Chance the contract makes (play and kitty decisions, and bids).
    pub makes: Option<f64>,
}

/// What the coach's values measure. Near the end of a game (and in a short game) it plays for the
/// game, not the points (docs/lab/README.md, finding 20): values are then the chance of winning
/// in tenths of a percent, so 10 is 1% and the grading scale reads them like points.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Units {
    Points,
    WinChance,
}

impl Units {
    /// The units for a position.
    pub fn of(obs: &Observation) -> Units {
        if plays_for_the_game(obs) {
            Units::WinChance
        } else {
            Units::Points
        }
    }

    /// "about 12 points" or "about 1.2% of games", for a value difference.
    pub fn worth(self, diff: f64) -> String {
        match self {
            Units::Points => format!("about {diff:.0} points"),
            Units::WinChance => format!("about {:.1}% of games", diff / 10.0),
        }
    }

    /// "A is worth about 12 points more than B" or "A wins the game about 1.2% more often than B".
    pub fn compare(self, a: &str, b: &str, diff: f64) -> String {
        match self {
            Units::Points => format!("{a} is worth about {diff:.0} points more than {b}"),
            Units::WinChance => format!("{a} wins the game about {:.1}% more often than {b}", diff / 10.0),
        }
    }
}

/// Whether the coach plays for the game rather than for points: a short game, or either side
/// within 160 of the target.
pub fn plays_for_the_game(obs: &Observation) -> bool {
    obs.rules.hand_limit.is_some() || obs.scores.iter().any(|&s| s >= obs.rules.target_score - 160)
}

#[derive(Clone, Debug)]
pub struct Analysis {
    /// Options, best first.
    pub options: Vec<OptionEval>,
    pub best: Action,
    /// Why the best option is best (or why the player's choice falls short).
    pub reasons: Vec<String>,
    /// Sampled worlds behind the numbers.
    pub worlds: usize,
    /// Other options about as good as the best: the sampled deals can't separate them (card
    /// play only). Shown as a close call rather than presenting a near tie as a firm answer.
    pub close: Vec<Action>,
    /// The hint's note on those close calls ("Close call: K♠ does about as well…"). A grade
    /// says something about the player's own card instead.
    close_note: Option<String>,
    /// What `options[].ev` measures.
    pub units: Units,
    /// Card play only: the raw statistics, for paired comparisons.
    play: Option<PlayStats>,
}

#[derive(Clone, Debug)]
pub struct Grade {
    pub chosen: Action,
    pub best: Action,
    /// Expected points the choice gave up versus the best option (never negative).
    pub loss: f64,
    pub verdict: Verdict,
    pub analysis: Analysis,
}

/// The grade for a whole kitty discard (all three cards at once).
#[derive(Clone, Debug)]
pub struct DiscardGrade {
    pub chosen: Vec<Card>,
    /// The coach's plan.
    pub plan: Vec<Card>,
    /// Expected points the discards gave up versus the plan (never negative).
    pub loss: f64,
    pub verdict: Verdict,
    pub reasons: Vec<String>,
}

pub struct Coach {
    search: SearchBot,
    rules_bot: RuleBot,
    /// The units of the analysis under way (for the wording of its reasons).
    units: Units,
}

impl Coach {
    pub fn new(seed: u64) -> Coach {
        // Card play as The Counter (more samples); bidding as The Oracle, which beat rule bidding
        // in the lab (docs/lab/README.md, finding 9).
        let config = SearchConfig { play_samples: 32, min_play_samples: 20, node_budget: 8_000_000, bid_samples: 16, neural_bidding: true, ..SearchConfig::COUNTER };
        Coach {
            search: SearchBot::new("Coach", config, seed),
            rules_bot: RuleBot::new("Coach", Style { aggression: SearchConfig::COUNTER.rule_aggression, ..Style::STEADY }, seed),
            units: Units::Points,
        }
    }

    /// The best action here and why. `chosen`, if given, is also evaluated so it can be compared.
    pub fn analyze(&mut self, obs: &Observation, legal: &[Action], chosen: Option<Action>) -> Analysis {
        let mut analysis = self.analyze_options(obs, legal, chosen);
        analysis.reasons.extend(analysis.close_note.take());
        analysis
    }

    /// As [`Coach::analyze`], leaving the close-call note out of the reasons.
    fn analyze_options(&mut self, obs: &Observation, legal: &[Action], chosen: Option<Action>) -> Analysis {
        let units = if plays_for_the_game(obs) { Units::WinChance } else { Units::Points };
        self.search.set_goal(if units == Units::WinChance { Goal::WinChance } else { Goal::Points });
        self.units = units;
        let mut analysis = self.analyze_by_phase(obs, legal, chosen);
        analysis.units = units;
        analysis
    }

    fn analyze_by_phase(&mut self, obs: &Observation, legal: &[Action], chosen: Option<Action>) -> Analysis {
        match obs.phase {
            Phase::Playing => self.analyze_play(obs, legal, chosen),
            Phase::Bidding => self.analyze_call(obs, legal, chosen),
            Phase::Discarding => self.analyze_discard(obs, legal, chosen),
            Phase::GameOver => Analysis { options: vec![], best: legal[0], reasons: vec![], worlds: 0, close: vec![], close_note: None, units: Units::Points, play: None },
        }
    }

    /// Grades `chosen` against the best option.
    ///
    /// A choice within two points of the best counts as best (equivalent cards, or plays that make no
    /// difference), and the reasons then explain the player's own choice. When the sampled worlds
    /// can't separate the two plays (mean difference under two standard errors), the verdict is
    /// capped at an inaccuracy, since the gap may be noise.
    pub fn grade(&mut self, obs: &Observation, legal: &[Action], chosen: Action) -> Grade {
        let mut analysis = self.analyze_options(obs, legal, Some(chosen));
        let ev = |a: Action, analysis: &Analysis| analysis.options.iter().find(|o| o.action == a).map(|o| o.ev);
        let best_ev = ev(analysis.best, &analysis).unwrap_or(0.0);
        let loss = (best_ev - ev(chosen, &analysis).unwrap_or(best_ev)).max(0.0);
        let signal = |a: Action| matches!(a, Action::Call(Call::Six(_) | Call::Mention(_)));
        if chosen != analysis.best && loss < 2.0 {
            if signal(analysis.best) && !signal(chosen) && obs.phase == Phase::Bidding {
                // Passing instead of signalling costs no points directly, but it leaves partner
                // guessing: worth a tip, not a black mark.
                return Grade { chosen, best: analysis.best, loss: 0.0, verdict: Verdict::Good, analysis };
            }
            if let Some(play) = analysis.play.as_ref() {
                let stat = |a: Action| play.stats.iter().find(|s| s.action == a).map(|&s| PlayStat { makes: real_play_chance(s.makes, PLAY_CURVE), ..s });
                analysis.reasons = play_reasons(obs, chosen, None, stat(chosen), None, play.worlds);
            } else if obs.phase == Phase::Bidding {
                analysis.reasons = call_reasons(obs, chosen, None, &analysis.options, obs.auction.high_bid());
            }
            analysis.best = chosen;
            return Grade { chosen, best: chosen, loss: 0.0, verdict: Verdict::Best, analysis };
        }
        let mut verdict = if chosen == analysis.best { Verdict::Best } else { Verdict::from_loss(loss, obs.phase == Phase::Bidding) };
        if analysis.close.contains(&chosen) {
            // One of the plays the coach couldn't separate from its choice: not a mistake.
            verdict = verdict.min(Verdict::Good);
            analysis.reasons.push(format!(
                "Close call: {chosen} does about as well as {} (the {} likely deals can't separate them).",
                analysis.best, analysis.worlds
            ));
        } else if verdict > Verdict::Good && !significant(analysis.play.as_ref(), analysis.best, chosen) {
            verdict = verdict.min(Verdict::Inaccuracy);
            analysis.reasons.push("A close call: the likely deals don't clearly separate these two plays.".into());
        }
        Grade { chosen, best: analysis.best, loss, verdict, analysis }
    }

    // ---- Card play ---------------------------------------------------------------------------

    fn analyze_play(&mut self, obs: &Observation, legal: &[Action], chosen: Option<Action>) -> Analysis {
        let mut result = self.search.play_stats(obs, legal);
        // The first look is quick, and on a couple of dozen deals plays a few percent apart can
        // swap places. If the best play isn't clearly ahead of the next different play, look at
        // many more deals before choosing.
        let first = self.search.evaluate_plays_from(obs, &result);
        let runner_up = first.iter().skip(1).find(|&&(a, _)| !equivalent(&result, first[0].0, a));
        if let Some(&(second, _)) = runner_up.filter(|_| result.valid) {
            if !significant(Some(&result), first[0].0, second) {
                let more = self.search.play_stats_sampled(obs, legal, CLOSER_LOOK, CLOSER_LOOK_MIN, CLOSER_LOOK_BUDGET);
                result = result.merge(more);
            }
        }
        let ranked = self.search.evaluate_plays_from(obs, &result);
        let best = ranked[0].0;
        // What players see is the real-play chance: the sampled deals are solved as if everyone
        // played perfectly, which understates the declarer's chances (`fh-lab honesty`).
        let stat = |a: Action| result.stats.iter().find(|s| s.action == a).map(|&s| PlayStat { makes: real_play_chance(s.makes, PLAY_CURVE), ..s });
        let mut options: Vec<OptionEval> =
            ranked.iter().map(|&(action, ev)| OptionEval { action, ev, makes: stat(action).map(|s| s.makes) }).collect();
        options.dedup_by_key(|o| o.action);
        // Still too close to call after the closer look: say so, rather than pick one as if sure.
        let close: Vec<Action> = ranked
            .iter()
            .skip(1)
            .filter(|&&(a, ev)| ranked[0].1 - ev < CLOSE_POINTS && (equivalent(&result, best, a) || !significant(Some(&result), best, a)))
            .map(|&(a, _)| a)
            .take(3)
            .collect();
        let reasons = play_reasons(obs, best, chosen, stat(best), chosen.and_then(stat), result.worlds);
        let close_note = (!close.is_empty()).then(|| close_call_reason(&result, best, &close));
        Analysis { options, best, reasons, worlds: result.worlds, close, close_note, units: Units::Points, play: Some(result) }
    }

    // ---- Bidding -----------------------------------------------------------------------------

    fn analyze_call(&mut self, obs: &Observation, legal: &[Action], chosen: Option<Action>) -> Analysis {
        if let Some(model) = crate::models::bid_model() {
            return self.analyze_call_neural(obs, legal, chosen, model);
        }
        let recommended = self.rules_bot.choose(obs, legal);
        let belief = Belief::new(obs, true);
        let worlds = belief.sample_fitting(&mut self.search_rng(), obs.hand, self.search.config().bid_samples);
        let me = obs.seat;

        // Passing (or signalling) leaves the current high bid standing.
        let high = obs.auction.high_bid();
        let pass_ev = match high {
            Some((seat, contract)) => {
                let results = self.search.simulate(&worlds, seat, contract.strain);
                self.search.expected(obs, &results, contract, seat)
            }
            None => 0.0,
        };
        let evaluate = |action: Action, coach: &mut Coach| -> OptionEval {
            match action {
                Action::Call(Call::Bid(contract)) => {
                    let results = coach.search.simulate(&worlds, me, contract.strain);
                    let ev = coach.search.expected(obs, &results, contract, me);
                    let bias = coach.search.config().dd_bias;
                    let weight: f64 = results.iter().map(|r| r.1).sum();
                    let makes = results.iter().filter(|r| r.0 as f64 - bias >= contract.level as f64).map(|r| r.1).sum::<f64>() / weight.max(1e-9);
                    OptionEval { action, ev, makes: Some(real_play_chance(makes, BID_CURVE)) }
                }
                _ => OptionEval { action, ev: pass_ev, makes: None },
            }
        };
        let mut options = vec![evaluate(recommended, self)];
        if let Some(c) = chosen.filter(|&c| c != recommended) {
            options.push(evaluate(c, self));
        }
        let pass = Action::Call(Call::Pass);
        if legal.contains(&pass) && !options.iter().any(|o| o.action == pass) {
            options.push(OptionEval { action: pass, ev: pass_ev, makes: None });
        }
        // The expert's call is the recommendation; others are ranked by value behind it.
        let first = options.remove(0);
        options.sort_by(|a, b| b.ev.total_cmp(&a.ev));
        options.insert(0, first);
        let reasons = call_reasons(obs, recommended, chosen, &options, high);
        Analysis { best: recommended, options, reasons, worlds: worlds.len(), close: vec![], close_note: None, units: Units::Points, play: None }
    }

    /// Bidding analysis with the neural evaluator: the same numbers The Oracle bids by, for every
    /// legal call. Calls that don't bid (pass, mentions, 6-calls) are worth the same in points:
    /// letting the current high bid stand.
    fn analyze_call_neural(&mut self, obs: &Observation, legal: &[Action], chosen: Option<Action>, model: &crate::nn::Mlp) -> Analysis {
        let recommended = self.search.choose(obs, legal);
        let (bids, baseline) = self.search.neural_values(obs, legal, model);
        let eval = |action: Action| -> OptionEval {
            match action {
                Action::Call(Call::Bid(c)) => {
                    let (_, ev, makes) = bids.iter().copied().find(|b| b.0 == c).unwrap_or((c, baseline, 0.0));
                    OptionEval { action, ev, makes: Some(real_play_chance(makes, BID_CURVE)) }
                }
                _ => OptionEval { action, ev: baseline, makes: None },
            }
        };
        let mut options = vec![eval(recommended)];
        if let Some(c) = chosen.filter(|&c| c != recommended) {
            options.push(eval(c));
        }
        // Show the strongest alternative bids too.
        let mut others: Vec<OptionEval> = bids.iter().map(|&(c, _, _)| eval(Action::Call(Call::Bid(c)))).filter(|o| !options.iter().any(|x| x.action == o.action)).collect();
        others.sort_by(|a, b| b.ev.total_cmp(&a.ev));
        options.extend(others.into_iter().take(2));
        let reasons = call_reasons(obs, recommended, chosen, &options, obs.auction.high_bid());
        Analysis { best: recommended, options, reasons, worlds: 0, close: vec![], close_note: None, units: Units::Points, play: None }
    }

    fn search_rng(&mut self) -> fh_core::Rng {
        fh_core::Rng::new(self.search.next_seed())
    }

    // ---- Kitty -------------------------------------------------------------------------------

    fn analyze_discard(&mut self, obs: &Observation, legal: &[Action], chosen: Option<Action>) -> Analysis {
        let trump = obs.trump();
        let plan = heuristic_discards(obs.hand, trump, 3 - obs.discards.map_or(0, |d| d.len()));
        let best = Action::Discard(plan[0]);
        let mut options = vec![OptionEval { action: best, ev: 0.0, makes: None }];
        if let Some(c) = chosen.filter(|&c| c != best && legal.contains(&c)) {
            // Compare the kitty plans: the expert's, and the player's choice completed the same way.
            let Action::Discard(card) = c else { unreachable!() };
            let rest = heuristic_discards(obs.hand.minus([card].into_iter().collect()), trump, plan.len() - 1);
            let mine: Vec<Card> = std::iter::once(card).chain(rest).collect();
            let (expert_ev, mine_ev) = self.compare_discards(obs, &plan, &mine);
            options[0].ev = expert_ev;
            options.push(OptionEval { action: c, ev: mine_ev, makes: None });
        }
        let reasons = discard_reasons(obs, &plan, chosen);
        Analysis { options, best, reasons, worlds: 12, close: vec![], close_note: None, units: Units::Points, play: None }
    }

    /// Grades a whole discard: `chosen` against the coach's plan, from `obs`, the declarer's
    /// position before the first discard. Judging the set together counts how the cards work as a
    /// group (voids, guards) rather than three separate guesses.
    pub fn grade_discard_set(&mut self, obs: &Observation, chosen: &[Card]) -> DiscardGrade {
        let trump = obs.trump();
        let plan = heuristic_discards(obs.hand, trump, chosen.len());
        let set = |cards: &[Card]| cards.iter().copied().collect::<CardSet>();
        // The plan, then how yours compares and any cards you shouldn't have let go, then voids.
        let mut notes = discard_reasons(obs, &plan, None);
        let mut reasons = vec![notes.remove(0)];
        let loss = if set(&plan) == set(chosen) {
            0.0
        } else {
            let (plan_ev, chosen_ev) = self.compare_discards(obs, &plan, chosen);
            let loss = (plan_ev - chosen_ev).max(0.0);
            if loss >= 2.0 {
                reasons.push(format!("{} across 12 likely deals.", self.units.compare("The plan", "your discards", loss)));
            } else {
                reasons.push("Your discards do about as well as the plan.".into());
            }
            loss
        };
        for &card in chosen {
            if is_trump(card, trump) || card.is_joker() {
                reasons.push(format!("You put back {card}, a trump: each trump is a likely trick."));
            } else if card.rank() == fh_core::cards::ACE {
                reasons.push(format!("You put back {card}: aces are winners, so keep them."));
            }
        }
        reasons.extend(notes);
        reasons.truncate(4);
        DiscardGrade { chosen: chosen.to_vec(), plan, loss, verdict: Verdict::from_loss(loss, false), reasons }
    }

    /// Expected points for two sets of discards on the same worlds.
    fn compare_discards(&mut self, obs: &Observation, a: &[Card], b: &[Card]) -> (f64, f64) {
        let (declarer, contract) = obs.contract.expect("discarding without a contract");
        let trump = contract.trump();
        let belief = Belief::new_with(obs, self.search.config().inference, true);
        let worlds = belief.sample_fitting(&mut self.search_rng(), obs.hand, 12);
        let mut solver = Solver::with_limit(trump, 1_500_000);
        let mut totals = [0.0, 0.0];
        let mut weight = 0.0;
        for world in &worlds {
            let mut results = [0u8; 2];
            let mut ok = true;
            for (i, set) in [a, b].iter().enumerate() {
                let mut hands = world.hands;
                hands[declarer.index()] = obs.hand.minus(set.iter().copied().collect());
                results[i] = solver.solve(&Position { hands, trump, trick: Trick::new(declarer) }, declarer.team());
                ok &= !solver.aborted();
            }
            if ok {
                for i in 0..2 {
                    totals[i] += world.weight * self.search.utility(obs, contract, declarer, results[i]);
                }
                weight += world.weight;
            }
        }
        (totals[0] / weight.max(1e-9), totals[1] / weight.max(1e-9))
    }
}

/// How many more deals to look at when the first look can't separate the best plays: up to
/// `CLOSER_LOOK`, stopping after `CLOSER_LOOK_MIN` once the budget is spent (early in a hand,
/// where each deal takes longest to solve).
const CLOSER_LOOK: usize = 64;
const CLOSER_LOOK_MIN: usize = 32;
/// The node budget for the closer look (about twice the first look's).
const CLOSER_LOOK_BUDGET: u64 = 16_000_000;
/// Plays the deals can't separate from the best are reported as close calls, if within this many
/// points (beyond it, a play that only looks close because its deals vary a lot isn't offered).
const CLOSE_POINTS: f64 = 30.0;

/// Whether two plays win exactly the same in every sampled deal (touching honors, say, or the
/// Joker and the right bower when both are winners).
fn equivalent(play: &PlayStats, a: Action, b: Action) -> bool {
    let index = |x: Action| play.stats.iter().position(|s| s.action == x);
    let (Some(i), Some(j)) = (index(a), index(b)) else { return false };
    play.per_world.iter().all(|(_, u)| (u[i] - u[j]).abs() < 1e-9)
}

/// "Close call: …" for the hint and the grade.
fn close_call_reason(play: &PlayStats, best: Action, close: &[Action]) -> String {
    let (same, near): (Vec<Action>, Vec<Action>) = close.iter().partition(|&&a| equivalent(play, best, a));
    let list = |actions: &[Action]| {
        let names: Vec<String> = actions.iter().map(|a| a.to_string()).collect();
        match names.len() {
            0 => String::new(),
            1 => names[0].clone(),
            n => format!("{} or {}", names[..n - 1].join(", "), names[n - 1]),
        }
    };
    let same_text = format!("{} is the same as {best} here: it wins the same tricks.", list(&same));
    match (same.is_empty(), near.is_empty()) {
        (false, true) => same_text,
        (true, false) => format!("Close call: {} does about as well (the {} likely deals can't separate them).", list(&near), play.worlds),
        (false, false) => format!(
            "Close call: {} does about as well (the {} likely deals can't separate them). {same_text}",
            list(&near),
            play.worlds
        ),
        (true, true) => String::new(),
    }
}

/// Whether `best` beats `chosen` by more than two standard errors of the paired, per-world
/// difference. The deals carry weights (how well each fits the auction), as the averages do, so
/// the error uses the effective number of deals, (Σw)² / Σw²: a few heavy deals count as a few.
/// Decisions without per-world data (bids, discards) count as significant.
fn significant(play: Option<&PlayStats>, best: Action, chosen: Action) -> bool {
    let Some(play) = play else { return true };
    let index = |a: Action| play.stats.iter().position(|s| s.action == a);
    let (Some(b), Some(c)) = (index(best), index(chosen)) else { return true };
    let total: f64 = play.per_world.iter().map(|(w, _)| w).sum();
    let squares: f64 = play.per_world.iter().map(|(w, _)| w * w).sum();
    if total <= 0.0 {
        return false;
    }
    let n = total * total / squares;
    if n < 3.0 {
        return false;
    }
    let mean = play.per_world.iter().map(|(w, u)| w * (u[b] - u[c])).sum::<f64>() / total;
    let var = play.per_world.iter().map(|(w, u)| w * (u[b] - u[c] - mean).powi(2)).sum::<f64>() / total * n / (n - 1.0);
    mean > 2.0 * (var / n).sqrt()
}

// ---- Explanations ---------------------------------------------------------------------------

/// From the coach's raw chance that the bid is made to what actually happens with expert play,
/// measured by `fh-lab honesty` (2,000 hands, docs/lab/README.md finding 15): pairs of (raw,
/// real), increasing, interpolated linearly. The solver plays every deal perfectly for both sides;
/// real defenders aren't perfect, so a bid the solver calls 15% is really made about half the
/// time. Bid estimates also run low, by about 10 points.
const PLAY_CURVE: &[(f64, f64)] = &[
    // Refit after finding 21 (card play without auction weights): 6,200 predictions from
    // `fh-lab honesty` runs at seeds 950,000 and 1,200,000.
    (0.000, 0.03),
    (0.010, 0.08),
    (0.036, 0.43),
    (0.071, 0.445),
    (0.103, 0.554),
    (0.140, 0.617),
    (0.200, 0.634),
    (0.385, 0.732),
    (0.617, 0.836),
    (0.810, 0.903),
    (0.990, 0.986),
    (1.000, 0.99),
];
const BID_CURVE: &[(f64, f64)] = &[
    (0.00, 0.05),
    (0.27, 0.33),
    (0.36, 0.48),
    (0.45, 0.59),
    (0.55, 0.69),
    (0.65, 0.74),
    (0.75, 0.83),
    (0.84, 0.92),
    (0.91, 0.94),
    (1.00, 0.97),
];

/// The real-play chance for a raw chance, on `curve`.
pub fn real_play_chance(raw: f64, curve: &[(f64, f64)]) -> f64 {
    let raw = raw.clamp(0.0, 1.0);
    for w in curve.windows(2) {
        let ((x0, y0), (x1, y1)) = (w[0], w[1]);
        if raw <= x1 {
            return y0 + (y1 - y0) * (raw - x0) / (x1 - x0).max(1e-9);
        }
    }
    curve.last().map_or(raw, |&(_, y)| y)
}

fn pct(x: f64) -> String {
    format!("{:.0}%", 100.0 * x)
}

fn play_card(action: Action) -> Option<Play> {
    match action {
        Action::Play(p) => Some(p),
        _ => None,
    }
}

/// Plain-language reasons for a card-play decision.
fn play_reasons(obs: &Observation, best: Action, chosen: Option<Action>, best_stat: Option<PlayStat>, chosen_stat: Option<PlayStat>, worlds: usize) -> Vec<String> {
    let mut reasons = Vec::new();
    let (Some(best_play), Some((declarer, contract))) = (play_card(best), obs.contract) else { return reasons };
    let trump = contract.trump();
    let trick = obs.current_trick.clone().unwrap_or_else(|| Trick::new(obs.seat));
    let me = obs.seat;
    let declaring = declarer.team() == me.team();
    let seen = obs.played().union(obs.hand).union(obs.discards.unwrap_or_default());
    let chosen_play = chosen.and_then(play_card).filter(|&c| Some(c.card) != Some(best_play.card));

    // How the contract fares, from the player's side.
    if let Some(b) = best_stat {
        let side = |makes: f64| if declaring { makes } else { 1.0 - makes };
        let goal = if declaring { "is made" } else { "goes down" };
        match (chosen_play, chosen_stat) {
            (Some(c), Some(cs)) if (side(b.makes) - side(cs.makes)).abs() >= 0.05 => reasons.push(format!(
                "With {} the bid {goal} in {} of likely deals; with {}, {}.",
                best_play.card,
                pct(side(b.makes)),
                c.card,
                pct(side(cs.makes))
            )),
            (Some(c), Some(cs)) if b.ev - cs.ev >= 2.0 => reasons.push(format!(
                "{} across {worlds} likely deals.",
                Units::of(obs).compare(&best_play.card.to_string(), &c.card.to_string(), b.ev - cs.ev)
            )),
            (None, _) => reasons.push(format!("The bid {goal} in {} of likely deals after this play.", pct(side(b.makes)))),
            _ => {}
        }
    }

    // About the trick in progress.
    if let (Some(led), Some(winner), Some(winning)) = (trick.led_suit(), trick.winner(trump), trick.winning_card(trump)) {
        let partner_winning = winner == me.partner();
        let to_beat = trick_strength(winning, led, trump);
        let beats = |c: Card| trick_strength(c, led, trump) > to_beat;
        if partner_winning {
            let sure = trick.cards().len() == 3 || is_boss(winning, seen, trump);
            if let Some(c) = chosen_play.filter(|c| beats(c.card)) {
                if is_trump(c.card, trump) && Some(led) != trump {
                    reasons.push(format!("Don't trump your partner's winner: {winning} already takes this trick."));
                } else {
                    reasons.push(format!("Your partner's {winning} is already winning; {} wastes a winner.", c.card));
                }
            } else if !beats(best_play.card) && sure {
                reasons.push(format!("Your partner's {winning} has this trick, so play low."));
                if let Some(c) = chosen_play.filter(|c| is_boss(c.card, seen, trump) && !c.card.is_joker()) {
                    reasons.push(format!("Save the {}: it is now the highest card left in its suit, a winner later.", c.card));
                }
            }
        } else if beats(best_play.card) && !chosen_play.is_some_and(|c| beats(c.card)) {
            reasons.push(format!("{} wins this trick.", best_play.card));
        }
    } else if declaring && trump.is_some() && is_trump(best_play.card, trump) {
        // Leading: drawing trumps.
        let out = trumps(CardSet::pack(), trump).minus(seen).len();
        if out > 0 {
            reasons.push(format!("Draw trumps: the opponents may still hold {out}."));
        }
    } else if is_boss(best_play.card, seen, trump) && !best_play.card.is_joker() {
        reasons.push(format!("{} is the highest card left in its suit.", best_play.card));
    }
    let partner_winning = trick.winner(trump) == Some(me.partner());
    if chosen_play.is_some_and(|c| c.card.is_joker()) && !best_play.card.is_joker() && (trump.is_none() || partner_winning) {
        reasons.push("Keep the Joker: it can win a trick whenever you need one.".into());
    }
    reasons.truncate(3);
    reasons
}

fn call_label(action: Action) -> String {
    match action {
        Action::Call(call) => call.to_string(),
        other => other.to_string(),
    }
}

/// Plain-language reasons for a bidding decision.
fn call_reasons(obs: &Observation, best: Action, chosen: Option<Action>, options: &[OptionEval], high: Option<(Seat, Contract)>) -> Vec<String> {
    let mut reasons = Vec::new();
    let hand = obs.hand;
    // The rule-of-thumb count (docs/rules-of-thumb.md) for the strain that matters here.
    let focus = match best {
        Action::Call(Call::Bid(c)) => c.strain,
        _ => Strain::ALL.into_iter().max_by(|&a, &b| thumb::count(obs, a).0.total_cmp(&thumb::count(obs, b).0)).unwrap(),
    };
    let (total, parts) = thumb::count(obs, focus);
    let shown: Vec<String> = parts.iter().take(6).map(|(l, v)| format!("{l} {}", thumb::fraction(*v))).collect();
    reasons.push(format!("Rule-of-thumb count for {focus}: about {} ({}).", thumb::fraction(total), shown.join(", ")));
    reasons.push("A 7 bid pays from a count of about 7. Go up to 8 at about 8½, and to 9 only at about 10.".into());
    // Suit or no trumps (docs/rules-of-thumb.md, rule 5a).
    let joker = hand.contains(Card::JOKER);
    let aces = fh_core::Suit::ALL.iter().filter(|&&s| hand.contains(Card::new(s, fh_core::cards::ACE))).count();
    let longest = fh_core::Suit::ALL.iter().map(|&s| hand.of_printed_suit(s).len()).max().unwrap_or(0);
    let strain_of = |a: Option<Action>| match a {
        Some(Action::Call(Call::Bid(c))) => Some(c.strain),
        Some(Action::Call(Call::Mention(s) | Call::Six(s))) => Some(s),
        _ => None,
    };
    let considered = |strain: Strain| focus == strain || strain_of(chosen) == Some(strain);
    if joker && aces >= 2 && longest <= 4 && focus != Strain::NoTrumps {
        reasons.push("With the Joker, two aces and no long suit, no trumps scores better on about 79% of hands like this.".into());
    } else if !joker && considered(Strain::NoTrumps) {
        reasons.push("Without the Joker, no trumps beats your best suit only about 30% of the time.".into());
    }
    let partner = obs.seat.partner();
    for (seat, call) in obs.auction.calls() {
        if seat == partner {
            match call {
                Call::Six(s) => reasons.push(format!("Your partner showed the Joker (6{s}).")),
                Call::Mention(s) if s.trump().is_some() => reasons.push(format!("Your partner mentioned {s}, probably with a bower.")),
                Call::Mention(_) => reasons.push("Your partner mentioned NT, probably with aces or the Joker.".into()),
                Call::Bid(c) => reasons.push(format!("Your partner bid {c}.")),
                Call::Pass => {}
            }
        }
    }
    let ev = |a: Action| options.iter().find(|o| o.action == a);
    match best {
        Action::Call(Call::Bid(c)) => {
            if let Some(o) = ev(best) {
                if let Some(m) = o.makes {
                    reasons.push(format!("{c} makes in about {} of similar deals.", pct(m)));
                }
            }
        }
        Action::Call(Call::Six(s)) => reasons.push(format!("You hold the Joker but not quite a bid: 6{s} tells your partner.")),
        Action::Call(Call::Mention(s)) => reasons.push(format!("Not enough to bid, but mention {s} to show your partner your strength there.")),
        _ => {
            if let Some((seat, c)) = high.filter(|(s, _)| s.team() != obs.seat.team()) {
                reasons.push(format!("Bidding over {seat}'s {c} would likely cost more than letting it play."));
            }
        }
    }
    if obs.auction.must_bid() {
        reasons.push("As dealer on the third deal, you must bid.".into());
    }
    // Overbidding by a level: the asymmetry the count alone doesn't show.
    if let (Some(Action::Call(Call::Bid(mine))), Action::Call(Call::Bid(better))) = (chosen, best) {
        if mine.level > better.level {
            reasons.insert(
                1,
                format!(
                    "{mine} adds only {} points over {better} when made, but with exactly {} tricks it scores −{} instead of +{}.",
                    mine.value() - better.value(),
                    better.level,
                    mine.value(),
                    better.value()
                ),
            );
        }
    }
    if let (Some(c), Some(b)) = (chosen.filter(|&c| c != best), ev(best)) {
        if let Some(o) = ev(c) {
            if b.ev - o.ev >= 5.0 {
                reasons.push(format!("{} here.", Units::of(obs).compare(&call_label(best), &call_label(c), b.ev - o.ev)));
            }
        }
        if hand.contains(Card::JOKER) && !matches!(c, Action::Call(Call::Six(_) | Call::Bid(_))) && matches!(best, Action::Call(Call::Six(_))) {
            reasons.push("Calling 6 when you hold the Joker helps your partner bid.".into());
        }
    }
    let bowers: Vec<String> = Strain::ALL.iter().filter_map(|s| s.trump()).filter(|&s| has_bower(hand, s)).map(|s| s.to_string()).collect();
    if bowers.is_empty() && matches!(best, Action::Call(Call::Pass)) && !hand.contains(Card::JOKER) {
        reasons.push("No Joker and no bowers: nothing to show.".into());
    }
    reasons.truncate(4);
    reasons
}

/// Plain-language reasons for kitty discards.
fn discard_reasons(obs: &Observation, plan: &[Card], chosen: Option<Action>) -> Vec<String> {
    let trump = obs.trump();
    let mut reasons = Vec::new();
    let plan_text: Vec<String> = plan.iter().map(|c| c.to_string()).collect();
    reasons.push(format!("Plan: put back {}.", plan_text.join(", ")));
    let after = obs.hand.minus(plan.iter().copied().collect());
    for suit in fh_core::Suit::ALL {
        let before = obs.hand.iter().filter(|&c| !is_trump(c, trump) && fh_core::effective_suit(c, trump) == Some(suit)).count();
        let left = after.iter().filter(|&c| !is_trump(c, trump) && fh_core::effective_suit(c, trump) == Some(suit)).count();
        if before > 0 && left == 0 && Some(suit) != trump {
            reasons.push(format!("That leaves you void in {suit}, so you can trump it."));
        }
    }
    if let Some(Action::Discard(card)) = chosen {
        if is_trump(card, trump) || card.is_joker() {
            reasons.push("Never put back a trump: each one is a likely trick.".into());
        } else if card.rank() == fh_core::cards::ACE {
            reasons.push("Keep your aces: they are winners.".into());
        }
    }
    reasons.truncate(3);
    reasons
}

// ---- Patterns: what kind of slip a decision was -------------------------------------------------

/// What kind of decision `chosen` was, when it differs from the coach's `best`: a short id the
/// trainer groups by to find a player's recurring leaks ("overbid", "draw-trumps", ...). `None`
/// when the two are the same.
pub fn slip_pattern(obs: &Observation, chosen: Action, best: Action) -> Option<&'static str> {
    if chosen == best {
        return None;
    }
    match (chosen, best) {
        (Action::Call(mine), Action::Call(better)) => Some(match (mine, better) {
            (Call::Bid(m), Call::Bid(b)) if m.level > b.level => "overbid",
            (Call::Bid(m), Call::Bid(b)) if m.level < b.level => "underbid",
            (Call::Bid(_), Call::Bid(_)) => "strain",
            // A bid where the coach would pass or signal, or the other way round.
            (Call::Bid(_), _) => "overbid",
            (_, Call::Bid(_)) => "underbid",
            (Call::Pass, Call::Six(_) | Call::Mention(_)) => "missed-signal",
            _ => "signal",
        }),
        (Action::Play(mine), Action::Play(better)) => Some(play_pattern(obs, mine.card, better.card)),
        (Action::Discard(card), _) => Some(discard_pattern(&[card], obs.trump())),
        _ => None,
    }
}

/// The pattern for a set of kitty discards that differs from the plan.
pub fn discard_pattern(chosen: &[Card], trump: Option<fh_core::Suit>) -> &'static str {
    if chosen.iter().any(|&c| is_trump(c, trump) || c.is_joker()) {
        "kitty-trump"
    } else if chosen.iter().any(|c| c.rank() == fh_core::cards::ACE) {
        "kitty-ace"
    } else {
        "kitty"
    }
}

fn play_pattern(obs: &Observation, mine: Card, better: Card) -> &'static str {
    let trump = obs.trump();
    let declaring = obs.declarer().is_some_and(|d| d.team() == obs.seat.team());
    let trumpish = |c: Card| is_trump(c, trump) || c.is_joker();
    let trick = obs.current_trick.clone().unwrap_or_else(|| Trick::new(obs.seat));
    let Some(led) = trick.led_suit() else {
        // Leading.
        return match (declaring, trumpish(mine), trumpish(better)) {
            (true, false, true) => "draw-trumps",
            (true, true, false) => "trumps-early",
            (true, _, _) => "declarer-lead",
            (false, _, _) => "defense-lead",
        };
    };
    let strength = |c: Card| fh_core::trick::trick_strength(c, led, trump);
    let top = trick.winning_card(trump).map_or(0, strength);
    let partner_winning = trick.winner(trump) == Some(obs.seat.partner());
    let (wins_mine, wins_better) = (strength(mine) > top, strength(better) > top);
    let follows = |c: Card| fh_core::effective_suit(c, trump) == Some(led);
    if partner_winning && wins_mine && !wins_better {
        "overtake"
    } else if !wins_mine && wins_better {
        "missed-win"
    } else if wins_mine && !wins_better {
        "spent-winner"
    } else if !follows(mine) && !follows(better) && !trumpish(mine) && !trumpish(better) {
        "throw-off"
    } else {
        "follow"
    }
}

// ---- Hindsight: the double-dummy trace ------------------------------------------------------

/// One card played, judged with every card visible.
#[derive(Clone, Debug)]
pub struct TracedPlay {
    pub seat: Seat,
    pub card: Card,
    /// Declarer's tricks with perfect play from here, after this card.
    pub par_after: u8,
    /// Tricks this play cost the player's side compared with its best play (0 if it was best).
    pub cost: u8,
}

/// The hand replayed card by card with every card visible: the best result possible at each
/// point and where tricks slipped away. This is hindsight, not a grade: players can't see the
/// hidden cards.
pub fn dd_trace(record: &HandRecord) -> Option<(u8, Vec<TracedPlay>)> {
    let (declarer, contract) = record.contract?;
    let trump = contract.trump();
    let team = declarer.team();
    let mut hands = record.dealt;
    hands[declarer.index()] = hands[declarer.index()].union(record.kitty).minus(record.discards);
    let mut solver = Solver::new(trump);
    let start = solver.solve(&Position { hands, trump, trick: Trick::new(declarer) }, team);
    let mut traced = Vec::new();
    let mut won = 0u8;
    for (done, trick) in record.tricks.iter().enumerate() {
        let remaining = 10 - done as u8;
        let mut so_far: Vec<Play> = Vec::new();
        let first_named = trick.cards().first().filter(|c| c.is_joker() && trump.is_none()).and(trick.led_suit());
        for (i, (seat, card)) in trick.plays().enumerate() {
            let partial = Trick::from_plays(trick.leader(), &so_far, trump);
            let values = solver.analyze(&Position { hands, trump, trick: partial }, team);
            let named = if i == 0 { first_named } else { None };
            let play = Play { card, named_suit: named };
            let actual = values.iter().find(|(p, _)| *p == play).map_or(0, |&(_, v)| v);
            let ours = |v: u8| if seat.team() == team { v } else { remaining - v };
            let best = values.iter().map(|&(_, v)| ours(v)).max().unwrap_or(0);
            traced.push(TracedPlay { seat, card, par_after: won + actual, cost: best.saturating_sub(ours(actual)) });
            hands[seat.index()].remove(card);
            so_far.push(play);
        }
        if trick.winner(trump).map(|w| w.team()) == Some(team) {
            won += 1;
        }
    }
    Some((start, traced))
}

// ---- Luck vs skill --------------------------------------------------------------------------

/// Plays the hand that starts in `game` with the expert in `seat` (other seats keep their
/// players) and returns the resulting record, so it can be compared with what happened.
pub fn expert_replay(mut game: fh_core::Game, players: [&str; 4], seat: Seat, seed: u64) -> Option<HandRecord> {
    let start = game.history().len();
    let mut bots: [Box<dyn Bot>; 4] = std::array::from_fn(|i| {
        let id = if i == seat.index() { "counter" } else { players[i] };
        crate::make_bot(id, seed.wrapping_add(i as u64)).unwrap_or_else(|| crate::make_bot("counter", seed).unwrap())
    });
    while game.history().len() == start && game.phase() != Phase::GameOver {
        let to_act = game.to_act()?;
        let legal = game.legal_actions();
        let action = bots[to_act.index()].choose(&game.observe(to_act), &legal);
        game.apply(action).ok()?;
    }
    game.history().get(start).cloned()
}

#[cfg(test)]
mod tests {
    use super::*;
    use fh_core::{Game, Rules};

    /// Near the end the coach plays for the game: behind with the opponents near 500, it bids
    /// light hands, and high enough to reach 500, more often than at the start (finding 20).
    #[test]
    fn near_the_end_the_coach_plays_for_the_game() {
        let (mut early, mut late, mut cases) = ([0, 0], [0, 0], 0);
        for seed in 0..400u64 {
            let mut game = Game::with_dealer(Rules::default(), seed, Seat::EAST);
            let obs = game.observe(Seat::SOUTH);
            let Some((strain, count)) = [Strain::Spades, Strain::Clubs, Strain::Diamonds]
                .into_iter()
                .map(|s| (s, crate::thumb::count(&obs, s).0))
                .max_by(|a, b| a.1.total_cmp(&b.1))
            else {
                continue;
            };
            if !(6.5..7.5).contains(&count) {
                continue;
            }
            cases += 1;
            let legal = game.legal_actions();
            let tally = |a: &Analysis, t: &mut [usize; 2]| {
                if let Action::Call(Call::Bid(c)) = a.best {
                    t[0] += 1;
                    t[1] += usize::from(c.level >= 8 && c.strain == strain);
                }
            };
            let first = Coach::new(seed).analyze(&obs, &legal, None);
            assert_eq!(first.units, Units::Points);
            tally(&first, &mut early);
            game.set_scores([300, 460]); // South's side behind, the opponents near 500
            let obs = game.observe(Seat::SOUTH);
            let end = Coach::new(seed).analyze(&obs, &legal, None);
            assert_eq!(end.units, Units::WinChance);
            tally(&end, &mut late);
        }
        assert!(cases >= 20, "{cases} light hands");
        assert!(late[0] > early[0], "bids: {late:?} near the end vs {early:?} at the start, of {cases}");
        assert!(late[1] > early[1], "8-level bids: {late:?} vs {early:?}");
    }

    /// The real-play curves are increasing, and lift the solver's pessimistic low chances.
    #[test]
    fn real_play_chances_follow_the_measured_curves() {
        for curve in [PLAY_CURVE, BID_CURVE] {
            let mut last = -1.0;
            for i in 0..=100 {
                let v = real_play_chance(i as f64 / 100.0, curve);
                assert!(v >= last && (0.0..=1.0).contains(&v));
                last = v;
            }
        }
        assert!((0.55..0.7).contains(&real_play_chance(0.15, PLAY_CURVE)));
        assert!(real_play_chance(0.2, PLAY_CURVE) > 0.5, "a 20% solver chance is made more often than not");
        assert!((real_play_chance(0.65, BID_CURVE) - 0.74).abs() < 1e-9);
    }

    /// Slips are tagged by kind, so the trainer can find a player's recurring leaks.
    #[test]
    fn slips_are_tagged_by_kind() {
        let bid = |level, strain| Action::Call(Call::Bid(Contract::new(level, strain)));
        let start = Game::new(Rules::default(), 4);
        let obs = start.observe(start.to_act().unwrap());
        assert_eq!(slip_pattern(&obs, bid(9, Strain::Diamonds), bid(8, Strain::Diamonds)), Some("overbid"));
        assert_eq!(slip_pattern(&obs, Action::Call(Call::Pass), bid(7, Strain::Hearts)), Some("underbid"));
        assert_eq!(slip_pattern(&obs, bid(7, Strain::Spades), bid(7, Strain::NoTrumps)), Some("strain"));
        assert_eq!(slip_pattern(&obs, Action::Call(Call::Pass), Action::Call(Call::Six(Strain::Hearts))), Some("missed-signal"));
        assert_eq!(slip_pattern(&obs, bid(8, Strain::Diamonds), bid(8, Strain::Diamonds)), None);

        // The declarer's opening lead: a side suit instead of a trump, and the reverse.
        let south = CardSet::parse("Joker JD QD 9D 7D 4D KS QS TS 5S").unwrap();
        let kitty = CardSet::parse("4H 5H 5C").unwrap();
        let rest: Vec<Card> = CardSet::pack().minus(south).minus(kitty).iter().collect();
        let hand = |i: usize| rest[i * 10..i * 10 + 10].iter().copied().collect::<CardSet>();
        let mut game = Game::from_deal(Rules::default(), 1, Seat::SOUTH, [south, hand(0), hand(1), hand(2)], kitty);
        for _ in 0..3 {
            game.apply(Action::Call(Call::Pass)).unwrap();
        }
        game.apply(bid(8, Strain::Diamonds)).unwrap();
        for card in kitty.iter() {
            game.apply(Action::Discard(card)).unwrap();
        }
        let obs = game.observe(Seat::SOUTH);
        let play = |c: &str| Action::Play(Play::card(Card::parse(c).unwrap()));
        assert_eq!(slip_pattern(&obs, play("5S"), play("JD")), Some("draw-trumps"));
        assert_eq!(slip_pattern(&obs, play("JD"), play("KS")), Some("trumps-early"));
        assert_eq!(discard_pattern(&[Card::JOKER], obs.trump()), "kitty-trump");
        assert_eq!(discard_pattern(&[Card::parse("AS").unwrap()], obs.trump()), "kitty-ace");
    }

    /// Bidding a level too high is explained by the swing: little to gain, much to lose.
    #[test]
    fn an_overbid_explains_the_swing() {
        let game = Game::new(Rules::default(), 4);
        let obs = game.observe(game.to_act().unwrap());
        let bid = |level| Action::Call(Call::Bid(Contract::new(level, Strain::Diamonds)));
        let options = [OptionEval { action: bid(8), ev: 200.0, makes: Some(0.9) }, OptionEval { action: bid(9), ev: 100.0, makes: Some(0.7) }];
        let reasons = call_reasons(&obs, bid(8), Some(bid(9)), &options, None);
        assert!(reasons.iter().any(|r| r.contains("9♦ adds only 100 points over 8♦") && r.contains("−380 instead of +280")), "{reasons:?}");
    }

    /// A near tie is reported as one, not presented as a firm answer that flips from run to run.
    /// The position is a user's 8♦ opening lead: Joker J♦ Q♦ 9♦ 7♦ 4♦ K♠ Q♠ 10♠ 5♠. On 600 deals
    /// with every card visible, a top trump makes 81%, the K♠ 80% and the 5♠ only 74%.
    #[test]
    fn close_calls_are_reported_and_weak_leads_are_not_suggested() {
        let south = CardSet::parse("Joker JD QD 9D 7D 4D KS QS TS 5S").unwrap();
        let kitty = CardSet::parse("4H 5H 5C").unwrap();
        let mut rest: Vec<Card> = CardSet::pack().minus(south).minus(kitty).iter().collect();
        fh_core::Rng::new(3).shuffle(&mut rest);
        let hand = |i: usize| rest[i * 10..i * 10 + 10].iter().copied().collect::<CardSet>();
        let mut game = Game::from_deal(Rules::default(), 1, Seat::SOUTH, [south, hand(0), hand(1), hand(2)], kitty);
        for _ in 0..3 {
            game.apply(Action::Call(Call::Pass)).unwrap();
        }
        game.apply(Action::Call(Call::Bid(Contract::new(8, Strain::Diamonds)))).unwrap();
        for card in kitty.iter() {
            game.apply(Action::Discard(card)).unwrap();
        }
        let (obs, legal) = (game.observe(Seat::SOUTH), game.legal_actions());
        let strong = ["Joker", "J♦", "K♠", "Q♠"];
        for seed in [11, 18] {
            let hint = Coach::new(seed).analyze(&obs, &legal, None);
            assert!(strong.contains(&hint.best.to_string().as_str()), "seed {seed}: {}", hint.best);
            assert!(hint.worlds > 40, "a close call gets a closer look ({} deals)", hint.worlds);
            assert!(!hint.close.is_empty(), "the top trump and the K♠ are a near tie");
            assert!(hint.reasons.iter().any(|r| r.contains("the same as") || r.starts_with("Close call")), "{:?}", hint.reasons);
        }
    }

    /// The coach sees only what the player can: two deals that differ only in the hidden cards
    /// (West's and East's hands swapped, a different kitty) get exactly the same advice.
    #[test]
    fn advice_depends_only_on_what_you_can_see() {
        for seed in 0..6u64 {
            let mut cards: Vec<Card> = CardSet::pack().iter().collect();
            fh_core::Rng::new(seed).shuffle(&mut cards);
            let set = |range: std::ops::Range<usize>| cards[range].iter().copied().collect::<CardSet>();
            let (south, west, north, east) = (set(0..10), set(10..20), set(20..30), set(30..40));
            let kitty = set(40..43);
            // The second deal: West and East swap hands, and North trades three cards with the kitty.
            let north_b: CardSet = set(23..30).union(kitty);
            let kitty_b = set(20..23);
            let a = Game::from_deal(Rules::default(), 1, Seat::EAST, [south, west, north, east], kitty);
            let b = Game::from_deal(Rules::default(), 1, Seat::EAST, [south, east, north_b, west], kitty_b);
            assert_eq!(a.to_act(), Some(Seat::SOUTH));
            let (obs_a, obs_b) = (a.observe(Seat::SOUTH), b.observe(Seat::SOUTH));
            assert_eq!(obs_a, obs_b, "South's view is the same in both deals");
            let legal = a.legal_actions();
            let hint_a = Coach::new(seed).analyze(&obs_a, &legal, None);
            let hint_b = Coach::new(seed).analyze(&obs_b, &legal, None);
            assert_eq!(hint_a.best, hint_b.best);
            assert_eq!(format!("{:?}", hint_a.options), format!("{:?}", hint_b.options));
            assert_eq!(hint_a.reasons, hint_b.reasons);
        }
    }

    #[test]
    fn grades_overtaking_partner_as_a_mistake() {
        // East follows to partner's winning A♦ with the K♦ instead of the 6♦.
        let rules = Rules::default();
        let mut auction = fh_core::Auction::new(Seat::EAST, false);
        assert!(auction.apply(Call::Bid(Contract::new(7, Strain::Spades)), &rules));
        for _ in 0..3 {
            assert!(auction.apply(Call::Pass, &rules));
        }
        let plays: Vec<Play> = ["5D", "AD", "4D"].iter().map(|c| Play::card(Card::parse(c).unwrap())).collect();
        let trick = Trick::from_plays(Seat::SOUTH, &plays, Some(fh_core::Suit::Spades));
        let obs = Observation {
            seat: Seat::EAST,
            phase: Phase::Playing,
            to_act: Some(Seat::EAST),
            rules,
            scores: [0, 0],
            dealer: Seat::EAST,
            deal_number: 1,
            hand: CardSet::parse("KD 6D 9C 8C 7C 10H 9H 8H 6S 7S").unwrap(),
            auction,
            contract: Some((Seat::SOUTH, Contract::new(7, Strain::Spades))),
            kitty: None,
            discards: None,
            tricks: vec![],
            current_trick: Some(trick.clone()),
            tricks_won: [0, 0],
            hands_played: 0,
        };
        let legal: Vec<Action> = fh_core::legal_plays(obs.hand, &trick, obs.trump()).into_iter().map(Action::Play).collect();
        let king = Action::Play(Play::card(Card::parse("KD").unwrap()));
        let grade = Coach::new(3).grade(&obs, &legal, king);
        assert_eq!(grade.best, Action::Play(Play::card(Card::parse("6D").unwrap())));
        assert!(grade.verdict >= Verdict::Inaccuracy, "{:?} loss {}", grade.verdict, grade.loss);
        assert!(grade.analysis.reasons.iter().any(|r| r.contains("Save the K♦")), "{:?}", grade.analysis.reasons);
    }

    #[test]
    fn analyzes_every_decision_of_a_hand() {
        let mut game = Game::new(Rules::default(), 5);
        let mut bots: [Box<dyn Bot>; 4] = std::array::from_fn(|i| crate::make_bot("steady", i as u64).unwrap());
        let mut coach = Coach::new(1);
        let start = game.history().len();
        let mut graded = 0;
        while game.history().len() == start {
            let seat = game.to_act().unwrap();
            let legal = game.legal_actions();
            let obs = game.observe(seat);
            let action = bots[seat.index()].choose(&obs, &legal);
            if seat == Seat::SOUTH && legal.len() > 1 {
                let grade = coach.grade(&obs, &legal, action);
                assert!(legal.contains(&grade.best));
                assert!(grade.loss >= 0.0);
                assert!(!grade.analysis.reasons.is_empty() || obs.phase == Phase::Playing, "{:?}", obs.phase);
                graded += 1;
            }
            game.apply(action).unwrap();
        }
        assert!(graded > 0);
        // The hindsight trace covers all 40 cards.
        if let Some((_, plays)) = dd_trace(&game.history()[start]) {
            assert_eq!(plays.len(), 40);
        }
    }
}
