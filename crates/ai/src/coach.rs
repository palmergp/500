//! The coach: analyzes a decision from the player's seat, grades the choice they made, and
//! explains the difference in plain language.
//!
//! Grades use only what the player could know at the time: the coach sees the same
//! [`Observation`] the player saw and scores every option on the same sampled worlds. The
//! hindsight view (every card visible) is kept separate, in [`dd_trace`].

use crate::belief::Belief;
use crate::cardplay::{is_boss, trumps};
use crate::evaluate::{has_bower, hand_tricks};
use crate::rule_bot::{heuristic_discards, RuleBot, Style};
use crate::search_bot::{PlayStat, PlayStats, SearchBot, SearchConfig};
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

#[derive(Clone, Debug)]
pub struct Analysis {
    /// Options, best first.
    pub options: Vec<OptionEval>,
    pub best: Action,
    /// Why the best option is best (or why the player's choice falls short).
    pub reasons: Vec<String>,
    /// Sampled worlds behind the numbers.
    pub worlds: usize,
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

pub struct Coach {
    search: SearchBot,
    rules_bot: RuleBot,
}

impl Coach {
    pub fn new(seed: u64) -> Coach {
        let config = SearchConfig { play_samples: 32, min_play_samples: 20, node_budget: 8_000_000, bid_samples: 16, ..SearchConfig::COUNTER };
        Coach {
            search: SearchBot::new("Coach", config, seed),
            rules_bot: RuleBot::new("Coach", Style { aggression: SearchConfig::COUNTER.rule_aggression, ..Style::STEADY }, seed),
        }
    }

    /// The best action here and why. `chosen`, if given, is also evaluated so it can be compared.
    pub fn analyze(&mut self, obs: &Observation, legal: &[Action], chosen: Option<Action>) -> Analysis {
        match obs.phase {
            Phase::Playing => self.analyze_play(obs, legal, chosen),
            Phase::Bidding => self.analyze_call(obs, legal, chosen),
            Phase::Discarding => self.analyze_discard(obs, legal, chosen),
            Phase::GameOver => Analysis { options: vec![], best: legal[0], reasons: vec![], worlds: 0, play: None },
        }
    }

    /// Grades `chosen` against the best option.
    ///
    /// A choice within two points of the best counts as best (equivalent cards, or plays that make no
    /// difference), and the reasons then explain the player's own choice. When the sampled worlds
    /// can't separate the two plays (mean difference under two standard errors), the verdict is
    /// capped at an inaccuracy, since the gap may be noise.
    pub fn grade(&mut self, obs: &Observation, legal: &[Action], chosen: Action) -> Grade {
        let mut analysis = self.analyze(obs, legal, Some(chosen));
        let ev = |a: Action, analysis: &Analysis| analysis.options.iter().find(|o| o.action == a).map(|o| o.ev);
        let best_ev = ev(analysis.best, &analysis).unwrap_or(0.0);
        let loss = (best_ev - ev(chosen, &analysis).unwrap_or(best_ev)).max(0.0);
        if chosen != analysis.best && loss < 2.0 {
            if let Some(play) = analysis.play.as_ref() {
                let stat = |a: Action| play.stats.iter().find(|s| s.action == a).copied();
                analysis.reasons = play_reasons(obs, chosen, None, stat(chosen), None, play.worlds);
            }
            analysis.best = chosen;
            return Grade { chosen, best: chosen, loss: 0.0, verdict: Verdict::Best, analysis };
        }
        let mut verdict = if chosen == analysis.best { Verdict::Best } else { Verdict::from_loss(loss, obs.phase == Phase::Bidding) };
        if verdict > Verdict::Good && !significant(analysis.play.as_ref(), analysis.best, chosen) {
            verdict = verdict.min(Verdict::Inaccuracy);
            analysis.reasons.push("A close call: the likely deals don't clearly separate these two plays.".into());
        }
        Grade { chosen, best: analysis.best, loss, verdict, analysis }
    }

    // ---- Card play ---------------------------------------------------------------------------

    fn analyze_play(&mut self, obs: &Observation, legal: &[Action], chosen: Option<Action>) -> Analysis {
        let result = self.search.play_stats(obs, legal);
        let ranked = self.search.evaluate_plays_from(obs, &result);
        let best = ranked[0].0;
        let stat = |a: Action| result.stats.iter().find(|s| s.action == a).copied();
        let mut options: Vec<OptionEval> =
            ranked.iter().map(|&(action, ev)| OptionEval { action, ev, makes: stat(action).map(|s| s.makes) }).collect();
        options.dedup_by_key(|o| o.action);
        let reasons = play_reasons(obs, best, chosen, stat(best), chosen.and_then(stat), result.worlds);
        Analysis { options, best, reasons, worlds: result.worlds, play: Some(result) }
    }

    // ---- Bidding -----------------------------------------------------------------------------

    fn analyze_call(&mut self, obs: &Observation, legal: &[Action], chosen: Option<Action>) -> Analysis {
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
                    OptionEval { action, ev, makes: Some(makes) }
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
        Analysis { best: recommended, options, reasons, worlds: worlds.len(), play: None }
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
        Analysis { options, best, reasons, worlds: 12, play: None }
    }

    /// Expected points for two sets of discards on the same worlds.
    fn compare_discards(&mut self, obs: &Observation, a: &[Card], b: &[Card]) -> (f64, f64) {
        let (declarer, contract) = obs.contract.expect("discarding without a contract");
        let trump = contract.trump();
        let belief = Belief::new(obs, true);
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

/// Whether `best` beats `chosen` by more than two standard errors of the paired, per-world
/// difference. Decisions without per-world data (bids, discards) count as significant.
fn significant(play: Option<&PlayStats>, best: Action, chosen: Action) -> bool {
    let Some(play) = play else { return true };
    let index = |a: Action| play.stats.iter().position(|s| s.action == a);
    let (Some(b), Some(c)) = (index(best), index(chosen)) else { return true };
    let diffs: Vec<f64> = play.per_world.iter().map(|(_, u)| u[b] - u[c]).collect();
    let n = diffs.len() as f64;
    if n < 3.0 {
        return false;
    }
    let mean = diffs.iter().sum::<f64>() / n;
    let var = diffs.iter().map(|d| (d - mean).powi(2)).sum::<f64>() / (n - 1.0);
    mean > 2.0 * (var / n).sqrt()
}

// ---- Explanations ---------------------------------------------------------------------------

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
        let goal = if declaring { "makes" } else { "goes down" };
        match (chosen_play, chosen_stat) {
            (Some(c), Some(cs)) if (side(b.makes) - side(cs.makes)).abs() >= 0.05 => reasons.push(format!(
                "With {} the contract {goal} in {} of likely deals; with {}, {}.",
                best_play.card,
                pct(side(b.makes)),
                c.card,
                pct(side(cs.makes))
            )),
            (Some(c), Some(cs)) if b.ev - cs.ev >= 2.0 => reasons.push(format!(
                "{} averages {:.0} points more than {} across {worlds} likely deals.",
                best_play.card,
                b.ev - cs.ev,
                c.card
            )),
            (None, _) => reasons.push(format!("The contract {goal} in {} of likely deals after this play.", pct(side(b.makes)))),
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
    let mut strains: Vec<(Strain, f32)> = Strain::ALL.iter().map(|&s| (s, hand_tricks(hand, s))).collect();
    strains.sort_by(|a, b| b.1.total_cmp(&a.1));
    let (top, alone) = strains[0];
    reasons.push(format!(
        "Your hand counts about {alone:.1} tricks in {top} by itself; with partner and the kitty, expect about {:.1}.",
        alone + PARTNER_AND_KITTY
    ));
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
                    reasons.push(format!("{c} makes in about {} of simulated deals.", pct(m)));
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
    if let (Some(c), Some(b)) = (chosen.filter(|&c| c != best), ev(best)) {
        if let Some(o) = ev(c) {
            if b.ev - o.ev >= 5.0 {
                reasons.push(format!("{} rates about {:.0} points better than {} here.", call_label(best), b.ev - o.ev, call_label(c)));
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
