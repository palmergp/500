//! The coach: analyzes a decision from the player's seat, grades the choice they made, and
//! explains the difference in plain language.
//!
//! Grades use only what the player could know at the time: the coach sees the same
//! [`Observation`] the player saw and scores every option on the same sampled worlds. The
//! hindsight view (every card visible) is kept separate, in [`dd_trace`].

use crate::belief::Belief;
use crate::cardplay::{is_boss, suit_order, trumps};
use crate::evaluate::has_bower;
use crate::thumb;
use crate::rule_bot::{heuristic_discards, RuleBot, Style};
use crate::search_bot::{Goal, PlayStat, PlayStats, SearchBot, SearchConfig};
use crate::solver::{Position, Solver};
use crate::Bot;
use fh_core::rules::is_trump;
use fh_core::trick::trick_strength;
use fh_core::{effective_suit, Action, Call, Card, CardSet, Contract, HandRecord, Observation, Phase, Play, Seat, Strain, Suit, Trick};

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
impl Analysis {
    /// Card play: every legal play with its value, the chance players see that the bid makes,
    /// and the tricks the player's side takes on average, for the hand's tooltips after a hint.
    pub fn play_chances(&self, obs: &Observation) -> Vec<(Action, f64, f64, f64)> {
        let (Some(play), Some((declarer, _))) = (self.play.as_ref(), obs.contract) else { return vec![] };
        let ours = declarer.team() == obs.seat.team();
        play.stats.iter().map(|s| (s.action, s.ev, shown_play_chance(obs, s.makes), if ours { s.tricks } else { 10.0 - s.tricks })).collect()
    }
}

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

/// Whether the coach plays for the game rather than for points: in a game to 500, once either side
/// is within 160 of 500, or of −500 (where a set bid loses the game, H24). In a short game (H25) it
/// always plays for points (your score less theirs, hand by hand): there's no target to race to,
/// so sacrifices that throw away points don't pay.
pub fn plays_for_the_game(obs: &Observation) -> bool {
    let near = |s: i32| s >= obs.rules.target_score - 160 || obs.rules.losing_score.is_some_and(|floor| s <= floor + 160);
    obs.rules.hand_limit.is_none() && obs.scores.iter().any(|&s| near(s))
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

impl Analysis {
    /// Card play: the best option that plays differently from the best (not a card that wins the
    /// same tricks in every sampled deal), with its value.
    pub fn runner_up(&self) -> Option<(Action, f64)> {
        let play = self.play.as_ref()?;
        self.options.iter().find(|o| o.action != self.best && !equivalent(play, self.best, o.action)).map(|o| (o.action, o.ev))
    }

    /// An option's value, if it was evaluated.
    pub fn value(&self, action: Action) -> Option<f64> {
        self.options.iter().find(|o| o.action == action).map(|o| o.ev)
    }
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
    /// Kitty grades: the deals of the first look, and of the closer look (`discard_comparison`).
    kitty_deals: (usize, usize),
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
            kitty_deals: (DISCARD_DEALS, DISCARD_CLOSER_LOOK),
        }
    }

    /// The coach with kitty grades on other numbers of deals (a first look of 0 goes straight to
    /// the closer look), for the lab's comparisons (`fh-lab kittystab`).
    pub fn with_kitty_deals(mut self, first: usize, closer: usize) -> Coach {
        self.kitty_deals = (first, closer);
        self
    }

    /// The coach with card play read through the auction too (partner's bid, mentions and passes
    /// weigh the sampled deals), for the lab's comparisons (`fh-lab coachaudit`).
    pub fn with_inference(seed: u64, inference: bool) -> Coach {
        let mut coach = Coach::new(seed);
        let config = SearchConfig { inference, ..*coach.search.config() };
        coach.search = SearchBot::new("Coach", config, seed);
        coach
    }

    /// The best action here and why. `chosen`, if given, is also evaluated so it can be compared.
    pub fn analyze(&mut self, obs: &Observation, legal: &[Action], chosen: Option<Action>) -> Analysis {
        let mut analysis = self.analyze_options(obs, legal, chosen);
        analysis.reasons.extend(analysis.close_note.take());
        analysis
    }

    /// In the auction: how often each legal bid would make in real play (the bidding network,
    /// through the measured curve), for the bidding box's tooltips once you've asked for a hint.
    pub fn bid_chances(&self, obs: &Observation, legal: &[Action]) -> Vec<(Action, f64)> {
        let Some(model) = crate::models::bid_model() else { return vec![] };
        let (bids, _) = self.search.neural_values(obs, legal, model);
        bids.into_iter().map(|(c, _, makes)| (Action::Call(Call::Bid(c)), real_play_chance(makes, BID_CURVE))).collect()
    }

    /// What a call that isn't a bid offers, in place of a make chance: a mention or 6-call is worth
    /// tricks to your partner's count (rule 4 of docs/rules-of-thumb.md); passing over an
    /// opponent's bid leaves them a measured chance to make it.
    pub fn call_note(&self, obs: &Observation, action: Action) -> Option<String> {
        let Action::Call(call) = action else { return None };
        match call {
            Call::Mention(s) => Some(format!("+{} trick to your partner's {s} count", crate::thumb::fraction(0.4))),
            Call::Six(s) => Some(format!("shows the Joker: +{} trick to your partner's {s} count, less in other suits", crate::thumb::fraction(1.33))),
            Call::Pass => {
                let (_, contract) = obs.auction.high_bid()?;
                let model = crate::models::bid_model()?;
                if let Some(chance) = self.search.partner_chance(obs, model) {
                    return Some(format!("leaves your partner in {contract}: it makes about {:.0}%", 100.0 * real_play_chance(chance, BID_CURVE)));
                }
                let chance = self.search.opponents_chance(obs, model)?;
                Some(format!("they make {contract} about {:.0}% of the time", 100.0 * chance))
            }
            Call::Bid(_) => None,
        }
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
                let stat = |a: Action| play.stats.iter().find(|s| s.action == a).map(|&s| PlayStat { makes: shown_play_chance(obs, s.makes), ..s });
                analysis.reasons = play_reasons(obs, chosen, None, stat(chosen), None);
            } else if obs.phase == Phase::Bidding {
                analysis.reasons = call_reasons(obs, chosen, None, &analysis.options, obs.auction.high_bid());
            }
            analysis.best = chosen;
            return Grade { chosen, best: chosen, loss: 0.0, verdict: Verdict::Best, analysis };
        }
        // A card play's gap on the likely deals, seen double dummy, comes to less at the table (finding 65).
        let loss = match chosen {
            Action::Play(c) => at_the_table(obs, c.card, loss),
            _ => loss,
        };
        let mut verdict = if chosen == analysis.best { Verdict::Best } else { Verdict::from_loss(loss, obs.phase == Phase::Bidding).max(Verdict::Good) };
        if let (Action::Call(Call::Bid(c)), true) = (chosen, verdict > Verdict::Good) {
            if let Some(note) = count_disagreement(obs, c, &analysis) {
                analysis.reasons.insert(0, note);
            }
        }
        if crate::cardplay::ace_or_top_trump(obs, analysis.best, chosen) {
            // The top trump or a side ace early: even when played out (finding 55), whichever the
            // search happened to prefer.
            analysis.reasons = vec![ace_or_top_trump_note(chosen, analysis.best)];
            return Grade { chosen, best: analysis.best, loss: 0.0, verdict: Verdict::Good, analysis };
        }
        if let (Action::Play(c), Action::Play(b)) = (chosen, analysis.best) {
            if throws_away(obs, c.card) && throws_away(obs, b.card) {
                // One discard against another: played out, the coach's discard came out a few points
                // better whatever the deals said, its "blunders" no better than the rest (findings 60
                // and 65). With the cards hidden, the opponents rarely know which suit you let go.
                if verdict > Verdict::Good {
                    verdict = Verdict::Good;
                    analysis.reasons.push(format!(
                        "Discards are hard to judge: {} looks better on these deals, but played out, the discard the coach prefers gains only a few points on average. The opponents can't see which suit you let go.",
                        b.card
                    ));
                }
            } else if low_side_lead(obs, c.card) && low_side_lead(obs, b.card) && verdict > Verdict::Good {
                // Which low card to lead: played out, the coach's choice came out no better (finding 65).
                verdict = Verdict::Good;
                analysis.reasons.push(format!("A close call: which low card you lead made little difference when hands like this were played out, even where the likely deals favour {}.", b.card));
            }
        }
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
        // Anything short of best says what was: first, unless a reason already names it.
        if verdict != Verdict::Best && chosen != analysis.best {
            let best = call_label(analysis.best);
            if !analysis.reasons.iter().any(|r| r.contains(&best)) {
                let what = if obs.phase == Phase::Bidding { "call" } else { "play" };
                let mut note = format!("Best {what}: {best}.");
                if loss >= 1.0 {
                    note = format!("{note} {}.", Units::of(obs).compare(&best, &format!("your {}", call_label(chosen)), loss));
                }
                analysis.reasons.insert(0, note);
            }
        }
        Grade { chosen, best: analysis.best, loss, verdict, analysis }
    }

    // ---- Card play ---------------------------------------------------------------------------

    /// The sampled deals behind a card-play analysis: the expensive part, remembered for the last
    /// few positions so the grade that follows a hint (or the app's look-ahead) reuses them. Hints
    /// and grades of a position use the same seed, so the numbers are the same either way.
    fn play_result(&mut self, obs: &Observation, legal: &[Action]) -> PlayStats {
        let units = self.units;
        // Keyed by how the deals are read too: a coach reading the auction samples different deals.
        let units = (units, self.search.config().inference);
        if let Some(hit) = PLAY_CACHE.with(|c| c.borrow().iter().find(|(o, u, _)| o == obs && *u == units).map(|(_, _, r)| r.clone())) {
            return hit;
        }
        let result = if legal.len() == 1 {
            // No choice to make: a few deals are enough for the bid's chances.
            self.search.play_stats_sampled(obs, legal, 8, 4, self.search.config().node_budget)
        } else {
            let mut result = self.search.play_stats(obs, legal);
            // The first look is quick, and on a couple of dozen deals plays a few percent apart can
            // swap places. If the best play isn't clearly ahead of the next different play, look
            // at many more deals before choosing.
            let first = self.search.evaluate_plays_from(obs, &result);
            let runner_up = first.iter().skip(1).find(|&&(a, _)| !equivalent(&result, first[0].0, a));
            if let Some(&(second, _)) = runner_up.filter(|_| result.valid) {
                if !significant(Some(&result), first[0].0, second) {
                    let more = self.search.play_stats_sampled(obs, legal, CLOSER_LOOK, CLOSER_LOOK_MIN, CLOSER_LOOK_BUDGET);
                    result = result.merge(more);
                }
            }
            result
        };
        PLAY_CACHE.with(|c| {
            let mut cache = c.borrow_mut();
            cache.push((obs.clone(), units, result.clone()));
            let excess = cache.len().saturating_sub(PLAY_CACHE_SIZE);
            cache.drain(..excess);
        });
        result
    }

    fn analyze_play(&mut self, obs: &Observation, legal: &[Action], chosen: Option<Action>) -> Analysis {
        let result = self.play_result(obs, legal);
        let ranked = self.search.evaluate_plays_from(obs, &result);
        let best = ranked[0].0;
        // What players see is the real-play chance: the sampled deals are solved as if everyone
        // played perfectly, which understates the declarer's chances (`fh-lab honesty`).
        let stat = |a: Action| result.stats.iter().find(|s| s.action == a).map(|&s| PlayStat { makes: shown_play_chance(obs, s.makes), ..s });
        // Once the tricks taken settle the bid, every play has the same chance: what's left is tricks.
        let settled = settled(obs).is_some();
        let mut options: Vec<OptionEval> = ranked
            .iter()
            .map(|&(action, ev)| OptionEval { action, ev, makes: stat(action).filter(|_| !settled).map(|s| s.makes) })
            .collect();
        options.dedup_by_key(|o| o.action);
        // Still too close to call after the closer look: say so, rather than pick one as if sure.
        // A side ace early, holding the top trump: the search usually prefers the ace, but played
        // out the two came out even (finding 55), so the top trump is offered as just as good.
        let top = crate::cardplay::early_top_trump(obs).map(|c| Action::Play(Play::card(c))).filter(|&top| crate::cardplay::ace_or_top_trump(obs, best, top));
        let mut close: Vec<Action> = ranked
            .iter()
            .skip(1)
            .filter(|&&(a, ev)| Some(a) != top && ranked[0].1 - ev < CLOSE_POINTS && (equivalent(&result, best, a) || !significant(Some(&result), best, a)))
            .map(|&(a, _)| a)
            .take(3)
            .collect();
        let reasons = play_reasons(obs, best, chosen, stat(best), chosen.and_then(stat));
        let mut close_note = (!close.is_empty()).then(|| close_call_reason(obs, &result, best, &close));
        if let Some(top) = top {
            let note = ace_or_top_trump_note(top, best);
            close_note = Some(match close_note.filter(|n| !n.is_empty()) {
                Some(rest) => format!("{note} {rest}"),
                None => note,
            });
            close.insert(0, top);
            close.truncate(3);
        }
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
                    let bias = coach.search.play_gap(contract.strain);
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
        // The strongest alternative bids too, and passing, which every bid is weighed against (it
        // lets the current high bid stand), ranked by value behind the choices. The hint shows four.
        let mut others: Vec<OptionEval> = bids.iter().map(|&(c, _, _)| eval(Action::Call(Call::Bid(c)))).filter(|o| !options.iter().any(|x| x.action == o.action)).collect();
        others.sort_by(|a, b| b.ev.total_cmp(&a.ev));
        others.truncate(4 - options.len() - 1);
        let pass = Action::Call(Call::Pass);
        if legal.contains(&pass) && !options.iter().any(|o| o.action == pass) {
            others.push(eval(pass));
        }
        others.sort_by(|a, b| b.ev.total_cmp(&a.ev));
        options.extend(others);
        let reasons = call_reasons(obs, recommended, chosen, &options, obs.auction.high_bid());
        Analysis { best: recommended, options, reasons, worlds: 0, close: vec![], close_note: None, units: Units::Points, play: None }
    }

    fn search_rng(&mut self) -> fh_core::Rng {
        fh_core::Rng::new(self.search.next_seed())
    }

    // ---- Kitty -------------------------------------------------------------------------------

    fn analyze_discard(&mut self, obs: &Observation, legal: &[Action], chosen: Option<Action>) -> Analysis {
        let trump = obs.trump();
        let plan = self.kitty_plan(obs, 3 - obs.discards.map_or(0, |d| d.len()));
        let best = Action::Discard(plan[0]);
        let mut options = vec![OptionEval { action: best, ev: 0.0, makes: None }];
        let mut worlds = DISCARD_DEALS;
        let mut close = vec![];
        if let Some(c) = chosen.filter(|&c| c != best && legal.contains(&c)) {
            // Compare the kitty plans: the expert's, and the player's choice completed the same way.
            let Action::Discard(card) = c else { unreachable!() };
            let rest = heuristic_discards(obs.hand.minus([card].into_iter().collect()), trump, plan.len() - 1);
            let mine: Vec<Card> = std::iter::once(card).chain(rest).collect();
            let cmp = self.discard_comparison(obs, &plan, &mine);
            options[0].ev = cmp.a;
            options.push(OptionEval { action: c, ev: cmp.b, makes: None });
            worlds = cmp.deals;
            // The deals can't tell them apart: a close call, not a mistake.
            if !cmp.clear() && cmp.a - cmp.b < CLOSE_POINTS {
                close.push(c);
            }
        }
        let reasons = discard_reasons(obs, &plan, chosen);
        Analysis { options, best, reasons, worlds, close, close_note: None, units: Units::Points, play: None }
    }

    /// Grades a whole discard: `chosen` against the coach's plan, from `obs`, the declarer's
    /// position before the first discard. Judging the set together counts how the cards work as a
    /// group (voids, guards) rather than three separate guesses.
    pub fn grade_discard_set(&mut self, obs: &Observation, chosen: &[Card]) -> DiscardGrade {
        let trump = obs.trump();
        let plan = self.kitty_plan(obs, chosen.len());
        let set = |cards: &[Card]| cards.iter().copied().collect::<CardSet>();
        // The plan, then how yours compares and any cards you shouldn't have let go, then voids.
        let mut notes = discard_reasons(obs, &plan, None);
        let mut reasons = vec![notes.remove(0)];
        let mut clear = true;
        let loss = if set(&plan) == set(chosen) {
            0.0
        } else {
            let cmp = self.discard_comparison(obs, &plan, chosen);
            let loss = (cmp.a - cmp.b).max(0.0);
            clear = cmp.clear();
            if loss >= 2.0 && clear {
                reasons.push(format!("{} across {} likely deals.", self.units.compare("The plan", "your discards", loss), cmp.deals));
            } else if loss >= 2.0 {
                reasons.push(format!(
                    "A close call: {} on average, but the {} likely deals don't clearly separate your discards from the plan.",
                    self.units.compare("the plan", "yours", loss),
                    cmp.deals
                ));
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
        // As in card play, a gap the deals can't confirm is a close call (good) within
        // `CLOSE_POINTS`, and at most an inaccuracy beyond.
        let verdict = match (clear, loss < CLOSE_POINTS) {
            (true, _) => Verdict::from_loss(loss, false),
            (false, true) => Verdict::from_loss(loss, false).min(Verdict::Good),
            (false, false) => Verdict::from_loss(loss, false).min(Verdict::Inaccuracy),
        };
        DiscardGrade { chosen: chosen.to_vec(), plan, loss, verdict, reasons }
    }

    /// The coach's kitty plan: the `count` cards to put back. In a suit bid, the rule's set
    /// (`heuristic_discards`, finding 40): searching among the sets it ranks next gained nothing
    /// when played out. In no trumps, the search bots' plan (`SearchBot::nt_kitty_plan`), which
    /// took 0.34 tricks more played out where it differed from the rule (finding 62).
    pub fn kitty_plan(&mut self, obs: &Observation, count: usize) -> Vec<Card> {
        if obs.trump().is_some() {
            return heuristic_discards(obs.hand, obs.trump(), count);
        }
        self.search.nt_kitty_plan(obs, count)
    }

    /// Two sets of discards compared on the same likely deals: first `DISCARD_DEALS` of them. If
    /// those can't separate them (or find only a small gap), the verdict comes from `DISCARD_CLOSER_LOOK` fresh deals (not
    /// counting the first look's, which were looked past for being unclear). A kitty grade on a
    /// dozen deals once called putting back the 4♥ rather than the 4♦ a blunder; over 400 deals
    /// each did better about as often.
    fn discard_comparison(&mut self, obs: &Observation, a: &[Card], b: &[Card]) -> DiscardComparison {
        let (first_deals, closer_deals) = self.kitty_deals;
        if first_deals > 0 {
            let first = DiscardComparison::from(&self.compare_discards(obs, a, b, first_deals));
            // A clear gap of CLOSE_POINTS or more stands. A small one gets the closer look too: on a
            // dozen deals, the few where a kitty card decides the bid may not have come up.
            let gap = (first.a - first.b).abs();
            if gap < 2.0 || (first.clear() && gap >= CLOSE_POINTS) {
                return first;
            }
        }
        DiscardComparison::from(&self.compare_discards(obs, a, b, closer_deals))
    }

    /// Each likely deal's weight and its value with discards `a` and with `b` (in `utility`'s units).
    fn compare_discards(&mut self, obs: &Observation, a: &[Card], b: &[Card], deals: usize) -> Vec<(f64, f64, f64)> {
        let (declarer, contract) = obs.contract.expect("discarding without a contract");
        let trump = contract.trump();
        // The deals fit the auction (partner's bid, mentions and passes), unlike the search's card
        // play: with partner's call read, a kitty comparison's error fell by a tenth and its chance
        // for the bid went from 61% to 79% against a real 86% (finding 58).
        let belief = Belief::new_with(obs, true, true);
        let worlds = belief.sample_fitting(&mut self.search_rng(), obs.hand, deals);
        let mut solver = Solver::with_limit(trump, 1_500_000);
        let mut out = Vec::new();
        for world in &worlds {
            let mut results = [0u8; 2];
            let mut ok = true;
            for (i, set) in [a, b].iter().enumerate() {
                let mut hands = world.hands;
                hands[declarer.index()] = obs.hand.minus(set.iter().copied().collect());
                // The search starts from a guess: the bid's level, then the first set's tricks
                // (the same answers, about a quarter sooner than guessing half the tricks).
                let guess = if i == 0 { contract.level } else { results[0] };
                results[i] = solver.solve_near(&Position { hands, trump, trick: Trick::new(declarer) }, declarer.team(), guess);
                ok &= !solver.aborted();
            }
            if ok {
                let value = |r: u8| self.search.utility(obs, contract, declarer, r);
                out.push((world.weight, value(results[0]), value(results[1])));
            }
        }
        out
    }
}

/// The likely deals a kitty grade first compares discards on, and how many more it looks at when
/// those can't separate them. 24 then 120 agreed with grades on 400 deals 83% of the time, against
/// 76% for 12 then 60, and rarely called good what those called a mistake (finding 66).
const DISCARD_DEALS: usize = 24;
const DISCARD_CLOSER_LOOK: usize = 120;

/// Two sets of discards, compared: their average values (`a` the plan's), the deals compared on,
/// and each deal's weight and difference.
struct DiscardComparison {
    a: f64,
    b: f64,
    deals: usize,
    per_world: Vec<(f64, f64)>,
}

impl DiscardComparison {
    fn from(worlds: &[(f64, f64, f64)]) -> DiscardComparison {
        let weight: f64 = worlds.iter().map(|w| w.0).sum::<f64>().max(1e-9);
        DiscardComparison {
            a: worlds.iter().map(|w| w.0 * w.1).sum::<f64>() / weight,
            b: worlds.iter().map(|w| w.0 * w.2).sum::<f64>() / weight,
            deals: worlds.len(),
            per_world: worlds.iter().map(|w| (w.0, w.1 - w.2)).collect(),
        }
    }

    /// Whichever set is ahead, it's clearly ahead (more than two standard errors).
    fn clear(&self) -> bool {
        let diffs: Vec<(f64, f64)> = self.per_world.iter().map(|&(w, d)| (w, if self.a >= self.b { d } else { -d })).collect();
        clearly_ahead(&diffs)
    }
}

/// Whether weighted per-deal differences (weight, difference) are clearly above zero: their mean
/// beats two standard errors, counting the deals by their effective number, (Σw)² / Σw².
fn clearly_ahead(diffs: &[(f64, f64)]) -> bool {
    let total: f64 = diffs.iter().map(|(w, _)| w).sum();
    let squares: f64 = diffs.iter().map(|(w, _)| w * w).sum();
    if total <= 0.0 {
        return false;
    }
    let n = total * total / squares;
    if n < 3.0 {
        return false;
    }
    let mean = diffs.iter().map(|(w, d)| w * d).sum::<f64>() / total;
    let var = diffs.iter().map(|(w, d)| w * (d - mean).powi(2)).sum::<f64>() / total * n / (n - 1.0);
    mean > 2.0 * (var / n).sqrt()
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

/// Why the top trump and a side ace are as good as each other early in a suit bid (finding 55):
/// `yours` is the play being described, `other` the one it's as good as.
fn ace_or_top_trump_note(yours: Action, other: Action) -> String {
    let is_ace = |a: Action| matches!(a, Action::Play(p) if !p.card.is_joker() && p.card.rank() == fh_core::cards::ACE);
    let (ace, top) = if is_ace(yours) { (yours, other) } else { (other, yours) };
    format!(
        "{yours} does as well as {other} here. Early in the hand, leading your top trump ({top}) to draw theirs and cashing a side ace ({ace}) came out even when both were played out from 1,747 hands like this: a side ace is seldom trumped this early (6% of the time)."
    )
}

/// What a card play's gap on the likely deals comes to at the table. The deals are solved with
/// every card seen, where every slip is punished and the best of a few dozen solved deals flatters
/// the winner: played out (`fh-lab gradeaudit`, 7,000 deals, finding 65), a gap of 40 came to about
/// 18 points, one of 200 to about 115, and under 10 to nothing. Grades and their reasons use this.
pub fn played_out(gap: f64) -> f64 {
    (PLAYED_OUT_SHARE * gap - PLAYED_OUT_LESS).max(0.0)
}

const PLAYED_OUT_SHARE: f64 = 0.6;
const PLAYED_OUT_LESS: f64 = 6.0;

/// `played_out` for playing `card`, except an early Joker lead in no trumps: its cost was measured
/// at the table already (findings 54 and 56) and goes into the values as it is.
fn at_the_table(obs: &Observation, card: Card, gap: f64) -> f64 {
    if card.is_joker() && crate::cardplay::early_joker_lead(obs).is_some() {
        gap
    } else {
        played_out(gap)
    }
}

/// Whether `card` would be thrown away: it neither follows the suit led nor beats the trick.
fn throws_away(obs: &Observation, card: Card) -> bool {
    let trump = obs.trump();
    let Some(trick) = obs.current_trick.as_ref() else { return false };
    let (Some(led), Some(winning)) = (trick.led_suit(), trick.winning_card(trump)) else { return false };
    effective_suit(card, trump) != Some(led) && trick_strength(card, led, trump) <= trick_strength(winning, led, trump)
}

/// Whether `card` is a low side card to lead: no trump, not the Joker, below the king, and not the
/// highest card left in its suit.
fn low_side_lead(obs: &Observation, card: Card) -> bool {
    let trump = obs.trump();
    let leading = obs.current_trick.as_ref().is_none_or(|t| t.cards().is_empty());
    let seen = obs.played().union(obs.hand).union(obs.discards.unwrap_or_default());
    leading && !card.is_joker() && !is_trump(card, trump) && card.rank() < fh_core::cards::KING && !is_boss(card, seen, trump)
}

/// "Close call: …" for the hint and the grade.
fn close_call_reason(obs: &Observation, play: &PlayStats, best: Action, close: &[Action]) -> String {
    let (same, near): (Vec<Action>, Vec<Action>) = close.iter().partition(|&&a| equivalent(play, best, a));
    let list = |actions: &[Action]| {
        let names: Vec<String> = actions.iter().map(|a| a.to_string()).collect();
        match names.len() {
            0 => String::new(),
            1 => names[0].clone(),
            n => format!("{} or {}", names[..n - 1].join(", "), names[n - 1]),
        }
    };
    // The ace from A-K: the same tricks, but partner can see it's a winner (`show_winner`).
    let shown = same.iter().any(|&a| crate::cardplay::show_winner(obs, a) == best);
    let same_text = if shown {
        format!("{} wins the same tricks, but play {best}: your partner can see the trick is already won and won't trump it.", list(&same))
    } else {
        format!("{} is the same as {best} here: it wins the same tricks.", list(&same))
    };
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
    let diffs: Vec<(f64, f64)> = play.per_world.iter().map(|(w, u)| (*w, u[b] - u[c])).collect();
    clearly_ahead(&diffs)
}

// ---- Explanations ---------------------------------------------------------------------------

/// From the coach's raw chance that the bid is made to what actually happens with expert play,
/// measured by `fh-lab honesty` (2,000 hands, docs/lab/README.md finding 15): pairs of (raw,
/// real), increasing, interpolated linearly. The solver plays every deal perfectly for both sides;
/// real defenders aren't perfect, so a bid the solver calls 15% is really made about half the
/// time. Bid estimates also run low, by about 10 points.
const PLAY_CURVE: &[(f64, f64)] = &[
    // Refit after finding 47 (the set kitty plan, cheap signals, the retrained bidding network):
    // 3,954 predictions from `fh-lab honesty --seed 4200000`, each old point mapped through the
    // measured stated-to-actual table.
    (0.000, 0.11),
    (0.010, 0.131),
    (0.036, 0.281),
    (0.071, 0.288),
    (0.103, 0.526),
    (0.140, 0.572),
    (0.200, 0.581),
    (0.385, 0.656),
    (0.617, 0.782),
    (0.810, 0.871),
    (0.990, 0.976),
    (1.000, 0.98),
];
/// Card-play positions whose sampled deals are remembered (see `Coach::play_result`).
const PLAY_CACHE_SIZE: usize = 6;

thread_local! {
    static PLAY_CACHE: std::cell::RefCell<Vec<CachedPlay>> = const { std::cell::RefCell::new(Vec::new()) };
}

/// A remembered analysis: the position, its units and whether the deals read the auction, and the result.
type CachedPlay = (Observation, (Units, bool), PlayStats);

/// Forgets the remembered card-play analyses, so the next one samples afresh (the lab, grading the
/// same position with differently seeded coaches).
pub fn clear_play_cache() {
    PLAY_CACHE.with(|c| c.borrow_mut().clear());
}

pub const BID_CURVE: &[(f64, f64)] = &[
    // Refit after finding 47: 1,993 bid predictions, as for PLAY_CURVE.
    (0.00, 0.06),
    (0.27, 0.34),
    (0.36, 0.452),
    (0.45, 0.52),
    (0.55, 0.603),
    (0.65, 0.65),
    (0.75, 0.756),
    (0.84, 0.87),
    (0.91, 0.89),
    (1.00, 0.92),
];

/// Whether the tricks already taken settle the bid: `Some(true)` made, `Some(false)` beaten.
pub fn settled(obs: &Observation) -> Option<bool> {
    let (declarer, contract) = obs.contract?;
    let won = obs.tricks_won[declarer.team().index()];
    let lost = obs.tricks_won[declarer.team().other().index()];
    if won >= contract.level {
        Some(true)
    } else if lost > 10 - contract.level {
        Some(false)
    } else {
        None
    }
}

/// The make chance players see for a card play: exact once the tricks taken settle it (a bid
/// already beaten goes down every time), otherwise the measured real-play curve.
fn shown_play_chance(obs: &Observation, raw: f64) -> f64 {
    match settled(obs) {
        Some(made) => f64::from(u8::from(made)),
        None => real_play_chance(raw, PLAY_CURVE),
    }
}

/// "K♠ is the highest card left in its suit", or, when you also hold higher cards of the suit
/// (all the others above it gone), that it's as good as those.
fn top_card_note(card: Card, hand: CardSet, trump: Option<Suit>) -> String {
    let higher: Vec<String> = match effective_suit(card, trump) {
        Some(suit) => hand
            .iter()
            .filter(|&c| c != card && !c.is_joker() && effective_suit(c, trump) == Some(suit) && trick_strength(c, suit, trump) > trick_strength(card, suit, trump))
            .map(|c| c.to_string())
            .collect(),
        None => vec![],
    };
    if higher.is_empty() {
        format!("{card} is the highest card left in its suit.")
    } else {
        format!("{card} wins as surely as your {}: together they're the highest cards left in the suit.", higher.join(" and "))
    }
}

/// "K♠, Q♠ and J♠ are still out among others": the cards of `card`'s suit (trumps for a trump)
/// above it that the player hasn't seen, the top three named; `None` if there are none.
fn still_out_above(card: Card, seen: CardSet, trump: Option<Suit>) -> Option<String> {
    let above: Vec<Card> = suit_order(effective_suit(card, trump)?, trump).into_iter().take_while(|&x| x != card).filter(|&x| !seen.contains(x)).collect();
    let names: Vec<String> = above.iter().take(3).map(|&x| if x.is_joker() { "the Joker".into() } else { x.to_string() }).collect();
    let list = match names.len() {
        0 => return None,
        1 => format!("{} is", names[0]),
        n => format!("{} and {} are", names[..n - 1].join(", "), names[n - 1]),
    };
    let more = if above.len() > 3 { " among others" } else { "" };
    Some(format!("{list} still out{more}"))
}

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

/// The text with its first letter in capitals.
fn capitalize(text: &str) -> String {
    let mut chars = text.chars();
    chars.next().map_or_else(String::new, |first| first.to_uppercase().chain(chars).collect())
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
fn play_reasons(obs: &Observation, best: Action, chosen: Option<Action>, best_stat: Option<PlayStat>, chosen_stat: Option<PlayStat>) -> Vec<String> {
    let mut reasons = Vec::new();
    let (Some(best_play), Some((declarer, contract))) = (play_card(best), obs.contract) else { return reasons };
    let trump = contract.trump();
    let trick = obs.current_trick.clone().unwrap_or_else(|| Trick::new(obs.seat));
    let me = obs.seat;
    let declaring = declarer.team() == me.team();
    let seen = obs.played().union(obs.hand).union(obs.discards.unwrap_or_default());
    let chosen_play = chosen.and_then(play_card).filter(|&c| Some(c.card) != Some(best_play.card));

    // How the contract fares, from the player's side. Once it's settled, only tricks are left to play
    // for: the defenders score 10 a trick whatever happens (and all ten score the declarers 250).
    let ours = |s: PlayStat| if declaring { s.tricks } else { 10.0 - s.tricks };
    if let (Some(state), Some(b)) = (settled(obs), best_stat) {
        reasons.push(
            match (state, declaring) {
                (true, true) => "Your bid is made. Each trick you take now keeps 10 points from the defenders.",
                (true, false) => "Their bid is made, but each trick you take still scores 10 points.",
                (false, true) => "Your bid is beaten. Each trick you take still keeps 10 points from the defenders.",
                (false, false) => "Their bid is beaten. Each more trick you take scores another 10 points.",
            }
            .into(),
        );
        match (chosen_play, chosen_stat) {
            (Some(c), Some(cs)) if (ours(b) - ours(cs)).abs() >= 0.05 => reasons.push(format!(
                "With {} your side takes {:.1} tricks on average; with {}, {:.1}.",
                best_play.card,
                ours(b),
                c.card,
                ours(cs)
            )),
            (None, _) => reasons.push(format!("Your side takes about {:.1} tricks after this play.", ours(b))),
            _ => {}
        }
    } else if let Some(b) = best_stat {
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
            (Some(c), Some(cs)) if at_the_table(obs, c.card, b.ev - cs.ev) >= 1.0 => reasons.push(format!(
                "{}.",
                Units::of(obs).compare(&best_play.card.to_string(), &c.card.to_string(), at_the_table(obs, c.card, b.ev - cs.ev))
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
                    reasons.push(format!("Save the {}: {} It's a winner later.", c.card, top_card_note(c.card, obs.hand, trump)));
                }
            }
        } else if beats(best_play.card) {
            let last = trick.cards().len() == 3;
            let best = best_play.card;
            // Nothing left out can beat it: the Joker, the top trump, a top card in no trumps, or
            // the last card of the trick.
            let sure = last || best.is_joker() || (is_boss(best, seen, trump) && (is_trump(best, trump) || (trump.is_none() && seen.contains(Card::JOKER))));
            let ruff = is_trump(best, trump) && Some(led) != trump;
            let wins = match (ruff, sure) {
                (true, true) => format!("trumping with {best} wins it for sure"),
                (true, false) => format!("trumping with {best} wins it for now"),
                (false, true) => format!("{best} wins it for sure"),
                (false, false) => format!("{best} beats it"),
            };
            match chosen_play {
                Some(c) if !beats(c.card) => reasons.push(if last {
                    format!("You're last to play: {best} takes this trick, and {} hands it to their {winning}.", c.card)
                } else if trick.cards().len() == 2 {
                    format!("{} leaves this trick to their {winning}: your partner has played, so only an opponent is left. {}.", c.card, capitalize(&wins))
                } else {
                    format!("{} leaves their {winning} winning, with only your partner left to beat it; {wins}.", c.card)
                }),
                Some(c) if !last && trick_strength(best, led, trump) > trick_strength(c.card, led, trump) => {
                    // Yours wins for now, but an opponent still to play may hold a higher card (of
                    // the suit, or a higher trump to overtrump yours).
                    if let Some(out) = still_out_above(c.card, seen, trump) {
                        let best_text = if sure { format!("{best} wins it for sure") } else { format!("{best} is harder to beat") };
                        reasons.push(format!("{} may not hold: an opponent plays after you, and {out}. {best_text}.", c.card));
                    }
                }
                None => {
                    let how = if sure { "for sure" } else { "for now" };
                    reasons.push(if ruff { format!("Trumping with {best} wins this trick {how}.") } else { format!("{best} wins this trick {how}.") });
                }
                _ => {}
            }
        }
    } else if declaring && trump.is_some() && is_trump(best_play.card, trump) {
        // Leading: drawing trumps.
        let out = trumps(CardSet::pack(), trump).minus(seen).len();
        if out > 0 && crate::cardplay::opponents_may_hold_trumps(obs, seen) {
            reasons.push(format!("Draw trumps: the opponents may still hold {out}."));
        }
    } else if is_boss(best_play.card, seen, trump) && !best_play.card.is_joker() {
        reasons.push(top_card_note(best_play.card, obs.hand, trump));
    }
    // Leading a card that can lose the trick when the coach's lead is the top card: played out,
    // leading low instead of a side winner cost about 96 points, and a lower trump instead of the
    // top one about 139 (finding 65).
    if let Some(c) = chosen_play.filter(|c| trick.is_empty() && !c.card.is_joker() && is_boss(best_play.card, seen, trump) && !is_boss(c.card, seen, trump)) {
        if let Some(out) = still_out_above(c.card, seen, trump) {
            reasons.push(format!("{} can lose the trick: {out}.", c.card));
        }
    }
    // In no trumps, cashing the only card you hold in a suit gives up its stopper: once it's gone,
    // whoever gets the lead can run that suit against you.
    if let Some(c) = chosen_play.filter(|c| trump.is_none() && trick.is_empty() && !c.card.is_joker() && is_boss(c.card, seen, trump)) {
        let suit = effective_suit(c.card, None);
        let holding = obs.hand.iter().filter(|&h| effective_suit(h, None) == suit).count();
        let out = CardSet::pack().iter().filter(|&x| !x.is_joker() && effective_suit(x, None) == suit).filter(|&x| !seen.contains(x)).count();
        if let Some(suit) = suit.filter(|_| holding == 1 && out >= 4) {
            let joker = if seen.contains(Card::JOKER) { "" } else { " (the Joker is one way in)" };
            // Second, after how the bid fares: it's the reason, so it shouldn't be cut.
            reasons.insert(reasons.len().min(1), format!(
                "Keep {} for now: it's your only {suit}, so it's what stops them running {suit} when they get the lead{joker}, with {out} still out. Cash the suit you're setting up first: {} is a trick whenever you want it.",
                c.card, c.card
            ));
        }
    }
    let partner_winning = trick.winner(trump) == Some(me.partner());
    if chosen_play.is_some_and(|c| c.card.is_joker()) && !best_play.card.is_joker() && (trump.is_none() || partner_winning) {
        reasons.push(if crate::cardplay::opening_joker_lead(obs) {
            "Don't open with the Joker in no trumps: kept, it takes the lead back once you're out of a suit. Declarers who led it first made their bid 48% of the time, against 58% with another lead.".into()
        } else if crate::cardplay::early_joker_lead(obs).is_some() {
            "Not yet: keep the Joker through the first three tricks. Kept, it takes the lead back once you're out of a suit; declarers who led it this early took fewer tricks. From about trick 7 it's a good lead.".into()
        } else if trump.is_none() {
            "Keep the Joker: once you're out of the suit led, it wins the trick and gets you back in.".into()
        } else {
            "Keep the Joker: it can win a trick whenever you need one.".into()
        });
    }
    reasons.truncate(3);
    reasons
}

/// When a bid follows the count (7 from about 7, 8 from about 8½) but the coach prefers something
/// else, why: the count is an average, and at its thresholds the levels score about the same (lab
/// finding 32), so the coach's reading of this particular hand decides.
fn count_disagreement(obs: &Observation, bid: Contract, analysis: &Analysis) -> Option<String> {
    let count = thumb::count(obs, bid.strain).0;
    let by_count = if count >= 8.5 { 8 } else if count >= 7.0 { 7 } else { return None };
    if bid.level > by_count {
        return None;
    }
    let makes = |a: Action| analysis.options.iter().find(|o| o.action == a).and_then(|o| o.makes);
    let mine = makes(Action::Call(Call::Bid(bid)))?;
    let best = match analysis.best {
        Action::Call(Call::Bid(b)) => match makes(analysis.best) {
            Some(m) => format!("{b}, which it gives {}", pct(m)),
            None => b.to_string(),
        },
        Action::Call(Call::Pass) => "passing".into(),
        other => other.to_string(),
    };
    Some(format!(
        "Your count ({}) does point to {}, but the count is an average: at its thresholds the levels score about the same. For this hand the coach gives {bid} only {} and prefers {best}.",
        thumb::fraction(count),
        bid.level,
        pct(mine)
    ))
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
    reasons.push("A 7 bid pays from a count of about 7. Go up to 8 at about 8½, and stop at 8: a 9 almost never pays.".into());
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
                // With the Joker partner would have called 6 (the convention), so a NT mention shows aces.
                Call::Mention(_) => reasons.push("Your partner mentioned NT: usually two or more aces, but not the Joker (with it they'd have called 6).".into()),
                Call::Bid(c) => reasons.push(format!("Your partner bid {c}.")),
                Call::Pass => {}
            }
        }
    }
    let ev = |a: Action| options.iter().find(|o| o.action == a);
    // A reason already weighs the bid against passing.
    let mut weighed = false;
    match best {
        Action::Call(Call::Bid(c)) => {
            if let Some(o) = ev(best) {
                if let Some(m) = o.makes {
                    reasons.push(format!("{c} makes in about {} of similar deals.", pct(m)));
                    // A bid the count doesn't reach: say so, rather than leave the count saying pass.
                    let count = thumb::count(obs, c.strain).0;
                    let needs = if c.strain == Strain::Spades { 7.0 } else { 6.75 };
                    if count < needs {
                        reasons.insert(
                            1,
                            format!(
                                "Your count for {} is only {}, short of a bid. The coach reads more of this hand and the auction than the count does, and this one is a judgment call: it gives {c} about {}.",
                                c.strain,
                                thumb::fraction(count),
                                pct(m)
                            ),
                        );
                    }
                    // What the bid is weighed against: passing, which lets the high bid stand. Over
                    // partner's bid, or when the bid is more likely to go down than make, say so, so
                    // a bid short of even odds reads as the lesser loss it is.
                    let pass = options.iter().find(|p| p.action == Action::Call(Call::Pass));
                    if let (Some(p), Some((seat, theirs))) = (pass, high) {
                        let gain = o.ev - p.ev;
                        let sacrifice = seat.team() != obs.seat.team() && plays_for_the_game(obs);
                        // Over an opponent's bid, bids under 70% still beat passing (finding 52).
                        if gain >= 1.0 && (seat == partner || (m < 0.7 && !sacrifice)) {
                            let compare = Units::of(obs).compare(&c.to_string(), "passing", gain);
                            let reason = if seat == partner {
                                let support = theirs.strain.trump().map(|t| hand.iter().filter(|&card| is_trump(card, Some(t))).count());
                                let help = match support {
                                    Some(0) => format!(", and none of your cards is a {} trump", theirs.strain),
                                    Some(1) => format!(", and only one of your cards is a {} trump", theirs.strain),
                                    _ => String::new(),
                                };
                                format!("Passing leaves your partner in {theirs}{help}. {compare}.")
                            } else {
                                format!("Passing lets {seat}'s {theirs} play. {compare}.")
                            };
                            reasons.insert(0, reason);
                            weighed = true;
                        }
                    }
                    // Nobody has bid: an opening bid this close to even is a close call (finding 52).
                    if high.is_none() && m < 0.65 && !plays_for_the_game(obs) {
                        reasons.insert(
                            0,
                            "A close call. Nobody has bid yet, so passing costs nothing now; in the lab, opening bids the coach gives under 65% came out about even with passing, and from 65% they pay. Over an opponent's bid it's different: letting their bid play costs more than going down.".into(),
                        );
                        weighed = true;
                    }
                    // A long shot over an opponent's bid near the end is a sacrifice: say so.
                    if let Some((seat, theirs)) = high.filter(|(s, _)| s.team() != obs.seat.team()) {
                        if m < 0.5 && plays_for_the_game(obs) {
                            let them = obs.scores[seat.team().index()];
                            let reason = if them + theirs.value() >= obs.rules.target_score {
                                format!(
                                    "A sacrifice: if they make {theirs} they reach {} and win the game. Going down in {c} costs points, but keeps the game alive.",
                                    them + theirs.value()
                                )
                            } else {
                                format!("A sacrifice for the score: letting {theirs} stand hurts your chances of winning the game more than going down in {c}.")
                            };
                            reasons.insert(0, reason);
                        }
                    }
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
    if weighed {
        // The comparison names partner's bid already.
        if let Some((_, theirs)) = high {
            reasons.retain(|r| *r != format!("Your partner bid {theirs}."));
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
            if b.ev - o.ev >= 5.0 && !(weighed && o.action == Action::Call(Call::Pass)) {
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
        // A void helps only in a suit bid, where you can trump it.
        if before > 0 && left == 0 && trump.is_some() && Some(suit) != trump {
            reasons.push(format!("That leaves you void in {suit}, so you can trump it."));
        }
    }
    // In no trumps, a short queen the plan puts back (finding 67).
    let short_queen = plan.iter().copied().find(|&q| {
        let suit: Vec<Card> = obs.hand.iter().filter(|h| !h.is_joker() && h.suit() == q.suit()).collect();
        trump.is_none() && q.rank() == fh_core::cards::QUEEN && suit.len() <= 3 && suit.iter().all(|h| h.rank() < fh_core::cards::KING)
    });
    if let Some(q) = short_queen {
        if !matches!(chosen, Some(Action::Discard(d)) if d.suit() == q.suit()) {
            reasons.push(format!("In no trumps a queen with one or two small cards stops little: put back {q} and its guards, and keep the length in your other suits."));
        }
    }
    if let Some(Action::Discard(card)) = chosen {
        let suit_cards = |c: Card| obs.hand.iter().filter(|h| h.suit() == c.suit() && !h.is_joker()).count();
        let king_guard = |c: Card| {
            !c.is_joker()
                && obs.hand.contains(Card::new(c.suit().expect("a suited card"), fh_core::cards::KING))
                && !obs.hand.contains(Card::new(c.suit().expect("a suited card"), fh_core::cards::ACE))
                && suit_cards(c) == 2
        };
        if trump.is_none() && !card.is_joker() && card.rank() != fh_core::cards::ACE && king_guard(card) {
            reasons.push("In no trumps, keep a king guarded: it stops the opponents' suit (about ⅕ of a trick).".into());
        }
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
    /// When it cost a trick: the plays that would have kept the best result (every card seen).
    pub better: Vec<Play>,
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
            let cost = best.saturating_sub(ours(actual));
            let better = if cost > 0 { values.iter().filter(|&&(_, v)| ours(v) == best).map(|&(p, _)| p).collect() } else { vec![] };
            traced.push(TracedPlay { seat, card, par_after: won + actual, cost, better });
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

    /// Once the tricks taken settle the bid, the hint talks tricks, and a beaten bid goes down
    /// every time, not "97%" (a user's case: they had beaten 7NT and the review said 97%).
    #[test]
    fn once_the_bid_is_settled_the_coach_plays_for_tricks() {
        let mut found = 0;
        for seed in 0..40u64 {
            let mut game = Game::new(Rules::default(), seed);
            let mut bots: [Box<dyn Bot>; 4] = std::array::from_fn(|i| crate::make_bot("steady", seed * 4 + i as u64).unwrap());
            while game.phase() != Phase::GameOver && game.history().iter().all(|h| h.contract.is_none()) {
                let seat = game.to_act().unwrap();
                let obs = game.observe(seat);
                if obs.phase == Phase::Playing && settled(&obs).is_some() && game.legal_actions().len() > 1 && found < 6 {
                    let analysis = Coach::new(seed).analyze(&obs, &game.legal_actions(), None);
                    assert!(analysis.options.iter().all(|o| o.makes.is_none()), "no chances once settled");
                    let exact = f64::from(u8::from(settled(&obs).unwrap()));
                    assert!(analysis.play_chances(&obs).iter().all(|&(_, _, m, _)| m == exact));
                    assert!(analysis.reasons.iter().any(|r| r.contains("10 points")), "{:?}", analysis.reasons);
                    found += 1;
                }
                let action = bots[seat.index()].choose(&obs, &game.legal_actions());
                game.apply(action).unwrap();
            }
        }
        assert!(found >= 3, "only {found} settled positions");
    }

    /// Any grade short of best says what the best play was (the user: "it should tell you what the
    /// best play was").
    #[test]
    fn a_grade_short_of_best_names_the_best_play() {
        let mut checked = 0;
        for seed in 0..30u64 {
            let mut game = Game::new(Rules::default(), seed);
            let mut bots: [Box<dyn Bot>; 4] = std::array::from_fn(|i| crate::make_bot(if i == 0 { "rookie" } else { "steady" }, seed * 4 + i as u64).unwrap());
            while game.phase() != Phase::GameOver && game.history().iter().all(|h| h.contract.is_none()) && checked < 8 {
                let seat = game.to_act().unwrap();
                let obs = game.observe(seat);
                let legal = game.legal_actions();
                let action = bots[seat.index()].choose(&obs, &legal);
                if seat == Seat::SOUTH && obs.phase != Phase::Discarding && legal.len() > 1 {
                    let grade = Coach::new(seed).grade(&obs, &legal, action);
                    if grade.verdict != Verdict::Best && grade.best != action {
                        let best = call_label(grade.best);
                        assert!(grade.analysis.reasons.iter().any(|r| r.contains(&best)), "{:?} for {action}, best {best}: {:?}", grade.verdict, grade.analysis.reasons);
                        checked += 1;
                    }
                }
                game.apply(action).unwrap();
            }
        }
        assert!(checked >= 3, "only {checked} grades short of best");
    }

    /// "K♠ is the highest card left" is wrong when you hold the A♠ too.
    #[test]
    fn a_top_card_note_mentions_your_higher_cards() {
        let hand: CardSet = [Card::new(Suit::Spades, fh_core::cards::ACE), Card::new(Suit::Spades, fh_core::cards::KING)].into_iter().collect();
        let king = Card::new(Suit::Spades, fh_core::cards::KING);
        let note = top_card_note(king, hand, Some(Suit::Hearts));
        assert!(note.contains("as surely as your A♠"), "{note}");
        let ace = Card::new(Suit::Spades, fh_core::cards::ACE);
        assert!(top_card_note(ace, hand, Some(Suit::Hearts)).contains("is the highest card left"));
    }

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

    /// Over an opponent's bid near the end of a game, the coach doesn't suggest long-shot bids:
    /// passing is valued in the same units as bidding (a user's case: third to speak, partner
    /// passed, an opponent bid 8♣, the coach suggested 8♦ at 27%).
    #[test]
    fn over_an_opponents_bid_the_coach_doesnt_take_long_shots() {
        let (mut cases, mut long_shots) = (0, 0);
        for seed in 0..300u64 {
            for scores in [[0, 0], [380, 380]] {
                let rules = if scores == [0, 0] { Rules { hand_limit: Some(4), ..Rules::default() } } else { Rules::default() };
                let mut game = Game::with_dealer(rules, seed, Seat::WEST); // North, East, South, West
                game.set_scores(scores);
                game.apply(Action::Call(Call::Pass)).unwrap();
                let eight = Action::Call(Call::Bid(Contract::new(8, Strain::Clubs)));
                if game.apply(eight).is_err() {
                    continue;
                }
                let obs = game.observe(Seat::SOUTH);
                assert_eq!(obs.to_act, Some(Seat::SOUTH));
                let hint = Coach::new(seed).analyze(&obs, &game.legal_actions(), None);
                let short = scores == [0, 0];
                assert_eq!(hint.units, if short { Units::Points } else { Units::WinChance });
                cases += 1;
                if let Some(o) = hint.options.iter().find(|o| o.action == hint.best && matches!(o.action, Action::Call(Call::Bid(_)))) {
                    if o.makes.unwrap_or(1.0) < 0.35 {
                        // Near 500 a long shot can be a sacrifice; the hint must say so. (A four-hand
                        // game is played for points: no sacrifices there.)
                        if !short {
                            assert!(hint.reasons.iter().any(|r| r.starts_with("A sacrifice")), "{scores:?} {:?}", hint.reasons);
                        }
                        long_shots += usize::from(short);
                    }
                }
            }
        }
        assert!(cases >= 400, "{cases}");
        // Early in a four-hand game, long shots are rare (it was nearly every hand before the fix).
        assert!(long_shots * 20 <= cases / 2, "{long_shots} long-shot bids out of {} early hands", cases / 2);
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
        assert!((real_play_chance(0.65, BID_CURVE) - 0.65).abs() < 1e-9);
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

    /// South's opening lead at 8 no trumps, holding the Joker. The bots never lead it here
    /// (finding 54), and the coach grades it as costing the bid's chance.
    fn opening_lead_with_the_joker() -> (Observation, Vec<Action>) {
        let rules = Rules::default();
        let mut auction = fh_core::Auction::new(Seat::SOUTH, false);
        assert!(auction.apply(Call::Bid(Contract::new(8, Strain::NoTrumps)), &rules));
        for _ in 0..3 {
            assert!(auction.apply(Call::Pass, &rules));
        }
        let trick = Trick::new(Seat::SOUTH);
        let obs = Observation {
            seat: Seat::SOUTH,
            phase: Phase::Playing,
            to_act: Some(Seat::SOUTH),
            rules,
            scores: [0, 0],
            dealer: Seat::EAST,
            deal_number: 1,
            hand: CardSet::parse("Joker AS KS QS 9S AH KH 7H AD 6C").unwrap(),
            auction,
            contract: Some((Seat::SOUTH, Contract::new(8, Strain::NoTrumps))),
            kitty: None,
            discards: Some(CardSet::parse("4D 5C 4H").unwrap()),
            tricks: vec![],
            current_trick: Some(trick.clone()),
            tricks_won: [0, 0],
            hands_played: 0,
        };
        let legal = fh_core::legal_plays(obs.hand, &trick, None).into_iter().map(Action::Play).collect();
        (obs, legal)
    }

    /// In no trumps, when the plan puts back a short queen and the player keeps it, the reasons
    /// say why (finding 67).
    #[test]
    fn a_short_queen_put_back_in_no_trumps_is_explained() {
        let (mut obs, _) = opening_lead_with_the_joker();
        obs.phase = Phase::Discarding;
        obs.current_trick = None;
        obs.discards = None;
        obs.hand = CardSet::parse("Joker AS KS 9S 7S 9H 8H 6H 5H QC 6C AD 7D").unwrap();
        let card = |c: &str| Card::parse(c).unwrap();
        let plan = [card("6C"), card("QC"), card("5H")];
        let reasons = discard_reasons(&obs, &plan, Some(Action::Discard(card("5H"))));
        assert!(reasons.iter().any(|r| r.contains("put back Q♣ and its guards")), "{reasons:?}");
        let reasons = discard_reasons(&obs, &plan, Some(Action::Discard(card("6C"))));
        assert!(!reasons.iter().any(|r| r.contains("queen with one or two")), "{reasons:?}");
    }

    #[test]
    fn never_opens_no_trumps_with_the_joker() {
        let (obs, legal) = opening_lead_with_the_joker();
        let is_joker = |a: Action| matches!(a, Action::Play(p) if p.card.is_joker());
        for seed in 0..3 {
            let mut bot = SearchBot::new("Oracle", SearchConfig::ORACLE, seed);
            assert!(!is_joker(bot.choose(&obs, &legal)));
        }
        if let Some(natural) = crate::neural_play::NeuralBot::new("Natural", 1) {
            assert!(!is_joker(natural.play(&obs, &legal)));
        }
        let joker = *legal.iter().find(|&&a| is_joker(a)).unwrap();
        let grade = Coach::new(3).grade(&obs, &legal, joker);
        assert!(!is_joker(grade.best));
        // 10 in a hundred of the swing between making 8 no trumps and going down one (+220, −220).
        assert!(grade.loss >= 40.0, "loss {}", grade.loss);
        assert!(grade.analysis.reasons.iter().any(|r| r.contains("Don't open with the Joker")), "{:?}", grade.analysis.reasons);
    }

    #[test]
    fn the_top_trump_and_a_side_ace_are_even_early() {
        // South opens 7♠ holding the top trump (the Joker) with trumps out, and side aces.
        let rules = Rules::default();
        let mut auction = fh_core::Auction::new(Seat::SOUTH, false);
        assert!(auction.apply(Call::Bid(Contract::new(7, Strain::Spades)), &rules));
        for _ in 0..3 {
            assert!(auction.apply(Call::Pass, &rules));
        }
        let trick = Trick::new(Seat::SOUTH);
        let mut obs = Observation {
            seat: Seat::SOUTH,
            phase: Phase::Playing,
            to_act: Some(Seat::SOUTH),
            rules,
            scores: [0, 0],
            dealer: Seat::EAST,
            deal_number: 1,
            hand: CardSet::parse("Joker JC 10S 9S AH 8H AD 7D AC 6C").unwrap(),
            auction,
            contract: Some((Seat::SOUTH, Contract::new(7, Strain::Spades))),
            kitty: None,
            discards: Some(CardSet::parse("4D 5C 4H").unwrap()),
            tricks: vec![],
            current_trick: Some(trick.clone()),
            tricks_won: [0, 0],
            hands_played: 0,
        };
        let legal: Vec<Action> = fh_core::legal_plays(obs.hand, &trick, obs.trump()).into_iter().map(Action::Play).collect();
        let play = |c: &str| Action::Play(Play::card(Card::parse(c).unwrap()));
        assert_eq!(crate::cardplay::early_top_trump(&obs), Some(Card::JOKER));
        assert!(crate::cardplay::ace_or_top_trump(&obs, play("AH"), Action::Play(Play::card(Card::JOKER))));
        assert!(!crate::cardplay::ace_or_top_trump(&obs, play("JC"), play("AH")), "only the top trump");
        assert!(!crate::cardplay::ace_or_top_trump(&obs, play("8H"), Action::Play(Play::card(Card::JOKER))), "only an ace");

        // Whichever of the two the coach prefers, the other grades as good, at no cost, saying why.
        let joker = Action::Play(Play::card(Card::JOKER));
        let mut coach = Coach::new(5);
        let hint = coach.analyze(&obs, &legal, None);
        let (other, best) = if crate::cardplay::ace_or_top_trump(&obs, hint.best, joker) {
            (joker, hint.best)
        } else {
            assert_eq!(hint.best, joker, "the coach opens with the top trump or a side ace");
            (play("AH"), joker)
        };
        if best != joker {
            assert!(hint.close.contains(&joker), "the hint offers the top trump too");
            assert!(hint.reasons.iter().any(|r| r.contains("came out even")), "{:?}", hint.reasons);
        }
        let grade = coach.grade(&obs, &legal, other);
        assert_eq!((grade.verdict, grade.loss), (Verdict::Good, 0.0));
        assert!(grade.analysis.reasons[0].contains("came out even"), "{:?}", grade.analysis.reasons);

        // A defender, a no-trump bid, or after the third trick: not this position.
        obs.seat = Seat::WEST;
        assert_eq!(crate::cardplay::early_top_trump(&obs), None);
        obs.seat = Seat::SOUTH;
        obs.contract = Some((Seat::SOUTH, Contract::new(7, Strain::NoTrumps)));
        assert_eq!(crate::cardplay::early_top_trump(&obs), None);
    }

    /// The same holds through the third trick (finding 56): after cashing the A♠, the declarer
    /// still doesn't lead the Joker, and the coach says why if you do.
    #[test]
    fn holds_the_joker_through_the_third_trick() {
        let (mut obs, _) = opening_lead_with_the_joker();
        let plays: Vec<Play> = ["AS", "5S", "6S", "7S"].iter().map(|c| Play::card(Card::parse(c).unwrap())).collect();
        obs.tricks = vec![Trick::from_plays(Seat::SOUTH, &plays, None)];
        obs.hand = CardSet::parse("Joker KS QS 9S AH KH 7H AD 6C").unwrap();
        obs.tricks_won = [1, 0];
        let trick = Trick::new(Seat::SOUTH);
        obs.current_trick = Some(trick.clone());
        let legal: Vec<Action> = fh_core::legal_plays(obs.hand, &trick, None).into_iter().map(Action::Play).collect();
        let is_joker = |a: Action| matches!(a, Action::Play(p) if p.card.is_joker());
        assert_eq!(crate::cardplay::early_joker_lead(&obs), Some(crate::cardplay::EARLY_JOKER_CHANCE));
        for seed in 0..3 {
            let mut bot = SearchBot::new("Oracle", SearchConfig::ORACLE, seed);
            assert!(!is_joker(bot.choose(&obs, &legal)));
        }
        if let Some(natural) = crate::neural_play::NeuralBot::new("Natural", 1) {
            assert!(!is_joker(natural.play(&obs, &legal)));
        }
        let joker = *legal.iter().find(|&&a| is_joker(a)).unwrap();
        let grade = Coach::new(3).grade(&obs, &legal, joker);
        assert!(!is_joker(grade.best));
        assert!(grade.analysis.reasons.iter().any(|r| r.contains("keep the Joker through the first three tricks")), "{:?}", grade.analysis.reasons);
        // From the fourth trick, the search decides.
        obs.tricks = vec![obs.tricks[0].clone(); 3];
        assert_eq!(crate::cardplay::early_joker_lead(&obs), None);
    }

    /// A user's deal: the game from its first call, with `actions` applied.
    fn deal_from(dealer: Seat, hands: [&str; 4], kitty: &str, actions: &[Action]) -> fh_core::Game {
        let hands = hands.map(|h| CardSet::parse(h).unwrap());
        let mut game = fh_core::Game::from_deal(Rules::default(), 1, dealer, hands, CardSet::parse(kitty).unwrap());
        for &a in actions {
            game.apply(a).expect("legal");
        }
        game
    }

    /// A user's 7 no trumps: after the A♥, leading the lone A♦ gives up the only diamond stopper,
    /// and the coach says so (not just that A♣ is a winner).
    #[test]
    fn cashing_a_lone_ace_in_no_trumps_is_explained_as_losing_the_stopper() {
        let card = |c: &str| Card::parse(c).unwrap();
        let play = |c: &str| Action::Play(Play::card(card(c)));
        let game = deal_from(
            Seat::SOUTH,
            ["9S QS KS JC KC AC 4D AD KH AH", "6S 7S 10S JS 5C 5D 7H 9H JH Joker", "7C 8C QC 6D 9D 10D 4H 8H 10H QH", "5S AS 6C 7D 8D JD QD KD 5H 6H"],
            "8S 9C 10C",
            &[
                Action::Call(Call::Six(Strain::Spades)),
                Action::Call(Call::Pass),
                Action::Call(Call::Pass),
                Action::Call(Call::Bid(Contract::new(7, Strain::NoTrumps))),
                Action::Discard(card("8S")),
                Action::Discard(card("4D")),
                Action::Discard(card("9C")),
                play("AH"),
                play("9H"),
                play("4H"),
                play("5H"),
            ],
        );
        let obs = game.observe(Seat::SOUTH);
        let grade = Coach::new(5).grade(&obs, &game.legal_actions(), play("AD"));
        assert_ne!(grade.best, play("AD"));
        assert!(grade.analysis.reasons.iter().any(|r| r.starts_with("Keep A♦ for now")), "{:?}", grade.analysis.reasons);
        // Leading the 9♠ rather than the K♥ (now the top heart): the 9♠ can lose the trick.
        let reasons = play_reasons(&obs, play("KH"), Some(play("9S")), None, None);
        assert!(reasons.iter().any(|r| r == "9♠ can lose the trick: A♠, J♠ and 10♠ are still out."), "{reasons:?}");
    }

    /// One discard against another is good at worst: played out, the coach's preferred discard
    /// gains a few points, its "blunders" no more (findings 60 and 65).
    #[test]
    fn a_discard_against_another_is_good_at_worst() {
        let mut found = 0;
        for seed in 0..40 {
            let mut game = Game::new(Rules::default(), seed);
            let mut bots: [crate::search_bot::SearchBot; 4] = std::array::from_fn(|i| crate::search_bot::SearchBot::new("bot", SearchConfig::FAST, seed * 4 + i as u64));
            while let Some(seat) = game.to_act() {
                let obs = game.observe(seat);
                let legal = game.legal_actions();
                let discards: Vec<Action> = legal.iter().copied().filter(|&a| matches!(a, Action::Play(p) if throws_away(&obs, p.card))).collect();
                if game.phase() == Phase::Playing && discards.len() == legal.len() && legal.len() >= 2 {
                    // Grade each discard but the coach's: none is worse than good.
                    let mut coach = Coach::new(seed);
                    let best = coach.analyze(&obs, &legal, None).best;
                    for &a in discards.iter().filter(|&&a| a != best) {
                        let grade = coach.grade(&obs, &legal, a);
                        assert!(grade.verdict <= Verdict::Good, "seed {seed}: {a} {:?} ({:.0})", grade.verdict, grade.loss);
                    }
                    found += 1;
                    break;
                }
                let action = bots[seat.index()].choose(&obs, &legal);
                let Ok(events) = game.apply(action) else { break };
                if events.iter().any(|e| matches!(e, fh_core::Event::HandScored { .. } | fh_core::Event::PassedOut { .. })) {
                    break;
                }
            }
            if found >= 3 {
                break;
            }
        }
        assert!(found >= 3, "positions with only discards to choose from: {found}");
    }

    /// Card-play losses are what play-outs show (finding 65): nothing under a gap of 10 on the likely
    /// deals, about 18 points from a gap of 40, and still a blunder from one of 200.
    #[test]
    fn card_play_losses_are_what_play_outs_show() {
        assert_eq!(played_out(8.0), 0.0);
        assert!((played_out(40.0) - 18.0).abs() < 1e-9);
        assert_eq!(Verdict::from_loss(played_out(40.0), false), Verdict::Inaccuracy);
        assert_eq!(Verdict::from_loss(played_out(80.0), false), Verdict::Mistake);
        assert_eq!(Verdict::from_loss(played_out(200.0), false), Verdict::Blunder);
    }

    /// The reasons for following to a trick say what each card does to it: last to play, the
    /// coach's card takes it and yours hands it over; with an opponent still to play, a card that
    /// wins for now may not hold.
    #[test]
    fn following_reasons_say_what_happens_to_the_trick() {
        let card = |c: &str| Card::parse(c).unwrap();
        let play = |c: &str| Action::Play(Play::card(card(c)));
        let hands = ["AS 7S 8C 9C 10C 5D 6D 7D 5H 6H", "KS QS JS AC KC AD KD AH KH Joker", "5S 9S 10S 5C 6C 7C 8D 9D 7H 8H", "6S 8S QC JC 10D JD QD 9H 10H JH"];
        let back = [Action::Discard(card("4D")), Action::Discard(card("4H")), Action::Discard(card("QH"))];
        let reasons = |game: &fh_core::Game| {
            let obs = game.observe(Seat::SOUTH);
            play_reasons(&obs, play("AS"), Some(play("7S")), None, None)
        };
        // West's 7NT, West leads the K♠: South is last.
        let mut actions = vec![Action::Call(Call::Pass), Action::Call(Call::Pass), Action::Call(Call::Bid(Contract::new(7, Strain::NoTrumps))), Action::Call(Call::Pass)];
        actions.extend(back);
        actions.extend([play("KS"), play("5S"), play("6S")]);
        let last = reasons(&deal_from(Seat::NORTH, hands, "4D 4H QH", &actions));
        assert!(last.iter().any(|r| r == "You're last to play: A♠ takes this trick, and 7♠ hands it to their K♠."), "{last:?}");
        // Partner's 7NT, partner leads the 5♠ and East plays the 6♠: South's 7♠ wins for now.
        let mut actions = vec![Action::Call(Call::Bid(Contract::new(7, Strain::NoTrumps))), Action::Call(Call::Pass), Action::Call(Call::Pass), Action::Call(Call::Pass)];
        actions.extend(back);
        actions.extend([play("5S"), play("6S")]);
        let third = reasons(&deal_from(Seat::WEST, hands, "4D 4H QH", &actions));
        assert!(
            third.iter().any(|r| r == "7♠ may not hold: an opponent plays after you, and K♠, Q♠ and J♠ are still out among others. A♠ is harder to beat."),
            "{third:?}"
        );
        // The same, but East plays the 8♠: the 7♠ leaves it to them.
        let mut actions = vec![Action::Call(Call::Bid(Contract::new(7, Strain::NoTrumps))), Action::Call(Call::Pass), Action::Call(Call::Pass), Action::Call(Call::Pass)];
        actions.extend(back);
        actions.extend([play("5S"), play("8S")]);
        let over = reasons(&deal_from(Seat::WEST, hands, "4D 4H QH", &actions));
        assert!(over.iter().any(|r| r == "7♠ leaves this trick to their 8♠: your partner has played, so only an opponent is left. A♠ beats it."), "{over:?}");
    }

    /// The no-trump kitty plan is searched, but on deals seeded by the position: any coach gives
    /// the same plan, and once its first card is back, the rest of it is still the plan.
    #[test]
    fn the_no_trump_kitty_plan_is_steady() {
        let game = deal_from(
            Seat::SOUTH,
            ["9S QS KS JC KC AC 4D AD KH AH", "6S 7S 10S JS 5C 5D 7H 9H JH Joker", "7C 8C QC 6D 9D 10D 4H 8H 10H QH", "5S AS 6C 7D 8D JD QD KD 5H 6H"],
            "8S 9C 10C",
            &[
                Action::Call(Call::Six(Strain::Spades)),
                Action::Call(Call::Pass),
                Action::Call(Call::Pass),
                Action::Call(Call::Bid(Contract::new(7, Strain::NoTrumps))),
            ],
        );
        let obs = game.observe(Seat::SOUTH);
        let plan = Coach::new(1).kitty_plan(&obs, 3);
        assert_eq!(plan.len(), 3);
        assert_eq!(Coach::new(99).kitty_plan(&obs, 3), plan);
        let mut game = game;
        game.apply(Action::Discard(plan[0])).unwrap();
        let rest = Coach::new(7).kitty_plan(&game.observe(Seat::SOUTH), 2);
        assert_eq!(rest, plan[1..].to_vec());
    }

    /// A user's 8♣: the plan puts back 6♠ 8♠ 4♦; putting back the 4♥ instead is a tie over 400
    /// deals, but a grade on a dozen of them once called it a blunder. Now it's at worst a close
    /// call, on any seed, while putting back the right bower is still a mistake.
    #[test]
    fn a_kitty_choice_the_deals_cannot_separate_is_not_a_blunder() {
        let card = |c: &str| Card::parse(c).unwrap();
        let game = deal_from(
            Seat::WEST,
            ["6S 8S JS 10C JC 5D 6D AD 9H KH", "9S 10S QS AS 7D 8D 10D KD 5H 7H", "KS 5C 7C 8C 9C 9D 8H JH AH Joker", "5S 7S 6C QC KC AC JD QD 10H QH"],
            "4D 4H 6H",
            &[
                Action::Call(Call::Bid(Contract::new(7, Strain::Clubs))),
                Action::Call(Call::Pass),
                Action::Call(Call::Bid(Contract::new(8, Strain::Clubs))),
                Action::Call(Call::Pass),
            ],
        );
        let obs = game.observe(Seat::SOUTH);
        for seed in 0..3 {
            let grade = Coach::new(seed).grade_discard_set(&obs, &[card("6S"), card("8S"), card("4H")]);
            assert!(grade.verdict <= Verdict::Good, "seed {seed}: {:?} {:.0} {:?}", grade.verdict, grade.loss, grade.reasons);
        }
        // With partner's 7♣ read into the deals, the right bower's worth shows on every seed.
        for seed in 0..3 {
            let grade = Coach::new(seed).grade_discard_set(&obs, &[card("6S"), card("8S"), card("JC")]);
            assert!(grade.verdict >= Verdict::Mistake, "seed {seed}: {:?} {:.0}", grade.verdict, grade.loss);
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
