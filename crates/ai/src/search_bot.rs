//! The expert player: Perfect Information Monte Carlo search.
//!
//! For each decision it samples worlds consistent with what its seat knows ([`Belief`]), solves
//! each with the double-dummy [`Solver`], scores the outcomes in game points, and picks the
//! action with the best weighted average. All candidates are scored on the same worlds, so their
//! differences are not swamped by the luck of the sample.
//!
//! Work is bounded by solver nodes rather than wall-clock time, so decisions are reproducible and
//! the same budget works natively and in WebAssembly.

use crate::belief::{Belief, World};
use crate::cardplay::card_cost;
use crate::evaluate::{has_bower, hand_tricks};
use crate::features::{bid_features, strain_block};
use crate::nn::{softmax, Mlp};
use crate::rule_bot::{heuristic_discards, RuleBot, Style};
use crate::solver::{Position, Solver};
use crate::Bot;
use fh_core::cards::ACE;
use fh_core::game::score_hand;
use fh_core::{Action, Call, Card, CardSet, Contract, Observation, Phase, Rng, Rules, Seat, Strain, Suit, Team, Trick};

/// Solver nodes any one world may use before it is dropped (about 0.1–0.3 s).
const WORLD_NODE_LIMIT: u64 = 1_500_000;

/// One legal play's results over the sampled worlds.
#[derive(Copy, Clone, Debug)]
pub struct PlayStat {
    pub action: Action,
    /// Expected value (points, or win chance × 1000, per the bot's goal).
    pub ev: f64,
    /// Chance the declarer makes the contract.
    pub makes: f64,
    /// Declarer's expected trick count.
    pub tricks: f64,
}

impl PlayStats {
    /// Two looks at the same decision (the same legal plays, in the same order) pooled into one.
    pub fn merge(self, other: PlayStats) -> PlayStats {
        let weight = |p: &PlayStats| p.per_world.iter().map(|(w, _)| w).sum::<f64>();
        let (a, b) = (weight(&self), weight(&other));
        let total = (a + b).max(1e-12);
        let stats = self
            .stats
            .iter()
            .zip(&other.stats)
            .map(|(x, y)| PlayStat {
                action: x.action,
                ev: (x.ev * a + y.ev * b) / total,
                makes: (x.makes * a + y.makes * b) / total,
                tricks: (x.tricks * a + y.tricks * b) / total,
            })
            .collect();
        let mut per_world = self.per_world;
        per_world.extend(other.per_world);
        PlayStats { stats, per_world, worlds: self.worlds + other.worlds, valid: self.valid || other.valid }
    }
}

#[derive(Clone, Debug)]
pub struct PlayStats {
    pub stats: Vec<PlayStat>,
    /// Per world (that solved), each play's utility in `stats` order, with the world's weight:
    /// for paired comparisons between plays.
    pub per_world: Vec<(f64, Vec<f64>)>,
    /// Worlds sampled.
    pub worlds: usize,
    /// False if every world hit the solver's node limit.
    pub valid: bool,
}

/// What the search bot optimizes.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Goal {
    /// Our points minus theirs from this hand.
    Points,
    /// The chance of winning the game, from a simple model of the scores after this hand.
    WinChance,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct SearchConfig {
    /// Worlds sampled for a card-play decision, at most.
    pub play_samples: usize,
    /// Worlds always sampled for a card-play decision, however costly.
    pub min_play_samples: usize,
    /// Solver nodes to spend on one card-play decision before stopping early.
    pub node_budget: u64,
    /// Worlds per candidate strain when deciding what to bid.
    pub bid_samples: usize,
    /// Worlds per candidate discard when choosing what to put back from the kitty.
    pub discard_samples: usize,
    /// Expected points a bid must beat the alternative by: positive is cautious.
    pub bid_margin: f64,
    /// Whether worlds are weighted by how well they fit the auction.
    pub inference: bool,
    pub goal: Goal,
    /// Tricks subtracted from simulated declarer results when bidding: double-dummy analysis
    /// assumes the declarer sees every card, which real declarers don't.
    pub dd_bias: f64,
    /// Use search for bidding and for the kitty (otherwise a rule bot decides).
    pub search_bidding: bool,
    /// Bid with the neural evaluator (takes precedence over `search_bidding`).
    pub neural_bidding: bool,
    /// Make and read the discard signal (first throw-off from the strongest side suit).
    pub signals: bool,
    /// Expected points the bot may give up to make the signal. All-cards-visible analysis can't
    /// value the information, so it needs an allowance; the lab measures the real effect.
    pub signal_allowance: f64,
    pub search_discards: bool,
    /// Aggression of the rule bot used when search bidding is off. +0.7 won the lab's sweep
    /// (docs/lab/README.md, finding 6).
    pub rule_aggression: f32,
    /// Make mentions and 6-calls (lab variants turn this off to measure what they're worth).
    pub signal_calls: bool,
    /// Notice who has shown out of a suit when imagining the hidden cards (lab variants turn
    /// this off to measure what counting is worth).
    pub track_voids: bool,
    /// Read partner's and opponents' discard signals when imagining the hidden cards (with
    /// `inference`; lab variants turn it off on its own).
    pub read_signals: bool,
    /// Value passing over an opponent's bid as if they make it three times in four, whatever we
    /// hold (the old estimate; a lab variant keeps it to measure the fitted one, finding 26).
    pub fixed_opponents: bool,
    /// When the bidding fallback makes a 6-call (lab variants; the house signal is `Always`).
    pub six: crate::rule_bot::SixPolicy,
    /// Mention or call 6 after partner has called (off: finding 35; a lab variant turns it on).
    pub signal_after_partner: bool,
    /// Leading for the declaring side, break near-ties toward the dearest card (top trumps and
    /// winners) instead of the cheapest (a lab variant).
    pub lead_winners: bool,
    /// Bid with the network from `FH_ALT_BID_MODEL` instead of the embedded one (a lab variant).
    pub alt_bid_model: bool,
}

impl SearchConfig {
    /// Sized to think for well under a second per card in a browser on a mid-range laptop.
    pub const COUNTER: SearchConfig = SearchConfig {
        play_samples: 20,
        min_play_samples: 4,
        node_budget: 2_500_000,
        bid_samples: 6,
        discard_samples: 4,
        bid_margin: 10.0,
        // Weighting worlds by the auction (what bids, mentions and passes suggest) made card play
        // worse by about 3 points a hand in the lab (docs/lab/README.md, finding 21). Hard facts
        // still apply: a 6-call places the Joker, shown-out suits are voids, and partner's discard
        // signal is still read.
        inference: false,
        goal: Goal::Points,
        // Measured with `fh-lab calibrate`: double-dummy results run about half a trick above
        // what search bots actually take as declarer.
        dd_bias: 0.5,
        // Simulated bidding with a handful of worlds lost 27 points a hand to rule bidding in the
        // lab (finding 7), so the expert bids by calibrated rules until a learned evaluator exists.
        search_bidding: false,
        neural_bidding: false,
        signals: true,
        // 25 cost the Oracle 3.4 points a hand against not signalling; 5 recovers about half
        // (finding 36). The signal stays: it's the table's convention, which people read.
        signal_allowance: 5.0,
        // Searched kitty choice gained +0.1 points a hand over the heuristic (finding 8): not
        // worth the time.
        search_discards: false,
        rule_aggression: 0.7,
        signal_calls: true,
        track_voids: true,
        read_signals: true,
        fixed_opponents: false,
        six: crate::rule_bot::SixPolicy::Always,
        signal_after_partner: false,
        lead_winners: false,
        alt_bid_model: false,
    };
    pub const SHARK: SearchConfig =
        SearchConfig { bid_margin: -5.0, rule_aggression: 1.0, goal: Goal::WinChance, ..SearchConfig::COUNTER };
    /// Neural bidding (Phase 5) with The Counter's card play.
    pub const ORACLE: SearchConfig = SearchConfig { neural_bidding: true, ..SearchConfig::COUNTER };
    /// A cheaper setting for bulk lab simulations.
    pub const FAST: SearchConfig =
        SearchConfig { play_samples: 8, min_play_samples: 3, node_budget: 800_000, bid_samples: 5, discard_samples: 3, ..SearchConfig::COUNTER };
}

pub struct SearchBot {
    name: String,
    config: SearchConfig,
    rng: Rng,
    discard_plan: Vec<Card>,
    fallback: RuleBot,
    note: String,
}

impl SearchBot {
    pub fn new(name: &str, config: SearchConfig, seed: u64) -> SearchBot {
        SearchBot {
            name: name.to_string(),
            config,
            rng: Rng::new(seed),
            discard_plan: Vec::new(),
            fallback: RuleBot::new(
                name,
                Style { aggression: config.rule_aggression, six: config.six, signal_after_partner: config.signal_after_partner, ..Style::STEADY },
                seed,
            ),
            note: String::new(),
        }
    }

    pub fn config(&self) -> &SearchConfig {
        &self.config
    }

    /// What to optimize from now on: points, or the chance of winning the game.
    pub fn set_goal(&mut self, goal: Goal) {
        self.config.goal = goal;
    }

    /// A fresh seed from the bot's random stream, for callers that sample worlds themselves.
    pub fn next_seed(&mut self) -> u64 {
        self.rng.next_u64()
    }

    /// The value of a hand's outcome to `me`, given the declaring side's trick count.
    pub(crate) fn utility(&self, obs: &Observation, contract: Contract, declarer: Seat, declarer_tricks: u8) -> f64 {
        let rules = &obs.rules;
        let side = declarer.team();
        let tricks = if side == Team::NorthSouth {
            [declarer_tricks, 10 - declarer_tricks]
        } else {
            [10 - declarer_tricks, declarer_tricks]
        };
        let change = score_hand(rules, contract, side, tricks, obs.scores);
        let us = obs.seat.team().index();
        match self.config.goal {
            Goal::Points => (change[us] - change[1 - us]) as f64,
            Goal::WinChance => {
                let after = [obs.scores[0] + change[0], obs.scores[1] + change[1]];
                let hands_left = rules.hand_limit.map(|n| n.saturating_sub(obs.hands_played + 1));
                1000.0 * win_chance(rules, after, side, obs.seat.team(), hands_left)
            }
        }
    }

    // ---- Card play ---------------------------------------------------------------------------

    /// Statistics for each legal play over the same sampled worlds: expected value, the chance
    /// the declarer makes the contract, and the declarer's expected tricks. In `legal` order.
    pub fn play_stats(&mut self, obs: &Observation, legal: &[Action]) -> PlayStats {
        let (samples, min, budget) = (self.config.play_samples, self.config.min_play_samples, self.config.node_budget);
        self.play_stats_sampled(obs, legal, samples, min, budget)
    }

    /// As [`SearchBot::play_stats`], with an explicit number of worlds (at most `samples`, at least
    /// `min_samples` unless the node budget runs out first).
    pub fn play_stats_sampled(&mut self, obs: &Observation, legal: &[Action], samples_wanted: usize, min_samples: usize, budget: u64) -> PlayStats {
        let (declarer, contract) = obs.contract.expect("playing without a contract");
        let trump = contract.trump();
        let trick = obs.current_trick.clone().expect("playing phase has a trick");
        let read_signals = self.config.signals && self.config.read_signals;
        let belief = Belief::new_full(obs, self.config.inference, read_signals, self.config.track_voids);
        let mut solver = Solver::with_limit(trump, WORLD_NODE_LIMIT);
        let won_so_far = obs.tricks_won[declarer.team().index()];

        let mut stats: Vec<PlayStat> = legal.iter().map(|&action| PlayStat { action, ev: 0.0, makes: 0.0, tricks: 0.0 }).collect();
        let mut per_world = Vec::new();
        let mut weight_sum = 0.0;
        let mut samples = 0;
        while samples < samples_wanted {
            if samples >= min_samples && solver.nodes >= budget {
                break;
            }
            let world = belief.sample(&mut self.rng, obs.hand);
            let pos = Position { hands: world.hands, trump, trick: trick.clone() };
            let values = solver.analyze(&pos, declarer.team());
            samples += 1;
            if solver.aborted() {
                continue;
            }
            let mut utilities = Vec::with_capacity(stats.len());
            for stat in stats.iter_mut() {
                let Action::Play(play) = stat.action else { continue };
                let tricks = won_so_far + values.iter().find(|(p, _)| *p == play).map_or(0, |&(_, v)| v);
                let u = self.utility(obs, contract, declarer, tricks);
                utilities.push(u);
                stat.ev += world.weight * u;
                stat.tricks += world.weight * tricks as f64;
                if tricks >= contract.level {
                    stat.makes += world.weight;
                }
            }
            per_world.push((world.weight, utilities));
            weight_sum += world.weight;
        }
        if weight_sum > 0.0 {
            for stat in stats.iter_mut() {
                stat.ev /= weight_sum;
                stat.makes /= weight_sum;
                stat.tricks /= weight_sum;
            }
        }
        PlayStats { stats, per_world, worlds: samples, valid: weight_sum > 0.0 }
    }

    /// Each legal play with its expected value, best first. Among plays within half a point of the
    /// best, the cheapest card comes first, so the bot doesn't waste high cards.
    pub fn evaluate_plays(&mut self, obs: &Observation, legal: &[Action]) -> Vec<(Action, f64)> {
        let result = self.play_stats(obs, legal);
        self.evaluate_plays_from(obs, &result)
    }

    /// Ranks plays from statistics already gathered by [`SearchBot::play_stats`].
    pub fn evaluate_plays_from(&self, obs: &Observation, result: &PlayStats) -> Vec<(Action, f64)> {
        let trump = obs.trump();
        let cost = |a: &Action| match a {
            Action::Play(p) => card_cost(p.card, trump),
            _ => 0,
        };
        let legal: Vec<Action> = result.stats.iter().map(|s| s.action).collect();
        if !result.valid {
            // Every world was pathological: fall back to the cheapest legal card.
            return vec![(*legal.iter().min_by_key(|a| cost(a)).expect("legal actions"), 0.0)];
        }
        let mut scored: Vec<(Action, f64)> = result.stats.iter().map(|s| (s.action, s.ev)).collect();
        scored.sort_by(|a, b| b.1.total_cmp(&a.1));
        let top = scored[0].1;
        let near = (0..scored.len()).filter(|&i| top - scored[i].1 < 0.5);
        let declaring_lead = obs.current_trick.as_ref().is_some_and(|t| t.cards().is_empty())
            && obs.contract.is_some_and(|(d, _)| d.team() == obs.seat.team());
        let pick = if self.config.lead_winners && declaring_lead {
            near.max_by_key(|&i| cost(&scored[i].0)).unwrap()
        } else {
            near.min_by_key(|&i| cost(&scored[i].0)).unwrap()
        };
        scored[..=pick].rotate_right(1);
        scored
    }

    /// At a first show-out, the discard signal if it costs at most `signal_allowance` points.
    fn with_signal(&self, obs: &Observation, ranked: &[(Action, f64)]) -> Option<Action> {
        if !self.config.signals {
            return None;
        }
        let trick = obs.current_trick.as_ref()?;
        let led = trick.led_suit()?;
        let trump = obs.trump();
        let following = ranked.iter().any(|(a, _)| matches!(a, Action::Play(p) if fh_core::effective_suit(p.card, trump) == Some(led)));
        if following || !crate::signals::first_time_void(obs, obs.seat) {
            return None;
        }
        let card = crate::signals::signal_card(obs.hand, trump, led)?;
        let best = ranked[0].1;
        let allowance = self.config.signal_allowance;
        ranked.iter().find(|(a, ev)| matches!(a, Action::Play(p) if p.card == card) && best - ev <= allowance).map(|&(a, _)| a)
    }

    // ---- Kitty -------------------------------------------------------------------------------

    /// Candidate sets of `count` discards: the heuristic choice, plus variants that void each short
    /// side suit.
    pub(crate) fn discard_candidates(hand: CardSet, trump: Option<Suit>, count: usize) -> Vec<Vec<Card>> {
        let mut candidates = vec![heuristic_discards(hand, trump, count)];
        let is_trump = |c: Card| fh_core::rules::is_trump(c, trump) || c.is_joker();
        for suit in Suit::ALL {
            let cards: Vec<Card> = hand
                .iter()
                .filter(|&c| !is_trump(c) && fh_core::effective_suit(c, trump) == Some(suit))
                .collect();
            if cards.is_empty() || cards.len() > count {
                continue;
            }
            let rest = hand.minus(cards.iter().copied().collect());
            let mut set = cards.clone();
            set.extend(heuristic_discards(rest, trump, count - cards.len()));
            set.sort();
            if !candidates.iter().any(|c| {
                let mut c = c.clone();
                c.sort();
                c == set
            }) {
                candidates.push(set);
            }
        }
        candidates
    }

    fn plan_discards(&mut self, obs: &Observation) -> Vec<Card> {
        let (declarer, contract) = obs.contract.expect("discarding without a contract");
        let trump = contract.trump();
        let count = 3 - obs.discards.map_or(0, |d| d.len());
        let candidates = Self::discard_candidates(obs.hand, trump, count);
        if candidates.len() == 1 {
            return candidates.into_iter().next().unwrap();
        }
        let belief = Belief::new(obs, self.config.inference);
        let worlds = belief.sample_fitting(&mut self.rng, obs.hand, self.config.discard_samples);
        let mut solver = Solver::with_limit(trump, WORLD_NODE_LIMIT);
        let mut best = (f64::MIN, 0);
        for (i, discards) in candidates.iter().enumerate() {
            let mut total = 0.0;
            let mut weights = 0.0;
            for world in &worlds {
                let mut hands = world.hands;
                hands[declarer.index()] = obs.hand.minus(discards.iter().copied().collect());
                let pos = Position { hands, trump, trick: Trick::new(declarer) };
                let tricks = solver.solve(&pos, declarer.team());
                if solver.aborted() {
                    continue;
                }
                total += world.weight * self.utility(obs, contract, declarer, tricks);
                weights += world.weight;
            }
            let value = total / weights.max(1e-9);
            if value > best.0 {
                best = (value, i);
            }
        }
        candidates[best.1].clone()
    }

    // ---- Bidding -----------------------------------------------------------------------------

    /// Simulated trick counts if `declarer` plays `strain`: for each world, the declarer takes the
    /// kitty, discards heuristically, leads, and both sides play perfectly.
    pub(crate) fn simulate(&mut self, worlds: &[World], declarer: Seat, strain: Strain) -> Vec<(u8, f64)> {
        let trump = strain.trump();
        let mut solver = Solver::with_limit(trump, WORLD_NODE_LIMIT);
        let results: Vec<(u8, f64)> = worlds
            .iter()
            .filter_map(|world| {
                let mut hands = world.hands;
                let thirteen = hands[declarer.index()].union(world.out);
                let drop: CardSet = heuristic_discards(thirteen, trump, 3).into_iter().collect();
                hands[declarer.index()] = thirteen.minus(drop);
                let pos = Position { hands, trump, trick: Trick::new(declarer) };
                let tricks = solver.solve(&pos, declarer.team());
                (!solver.aborted()).then_some((tricks, world.weight))
            })
            .collect();
        if results.is_empty() {
            // Every world was pathological: assume an average result rather than none.
            return vec![(6, 1.0)];
        }
        results
    }

    /// Expected utility of `contract` over simulated results, after removing `dd_bias` tricks
    /// from the declarer (split between the two neighbouring trick counts).
    /// Simulated declarer trick counts (with weights) if `declarer` plays `strain`, sampling worlds
    /// from `obs`'s point of view. Used by the lab to calibrate bidding.
    pub fn simulate_from(&mut self, obs: &Observation, declarer: Seat, strain: Strain, samples: usize) -> Vec<(u8, f64)> {
        let belief = Belief::new(obs, self.config.inference);
        let worlds = belief.sample_fitting(&mut self.rng, obs.hand, samples);
        self.simulate(&worlds, declarer, strain)
    }

    pub(crate) fn expected(&self, obs: &Observation, results: &[(u8, f64)], contract: Contract, declarer: Seat) -> f64 {
        let bias = self.config.dd_bias.max(0.0);
        let (whole, frac) = (bias.floor() as u8, bias.fract());
        let mut total = 0.0;
        for &(tricks, w) in results {
            let low = tricks.saturating_sub(whole + 1);
            let high = tricks.saturating_sub(whole);
            total += w * ((1.0 - frac) * self.utility(obs, contract, declarer, high) + frac * self.utility(obs, contract, declarer, low));
        }
        total / results.iter().map(|&(_, w)| w).sum::<f64>().max(1e-9)
    }

    /// Bidding with the neural evaluator: it predicts, from what this seat can see, how many tricks
    /// our side takes as declarer in each strain, averaged over the deals it learned from. Each
    /// legal bid is scored on that distribution (less the play gap, `dd_bias`) and compared with
    /// passing.
    /// The neural evaluator's view of every legal bid: (contract, expected points, make chance),
    /// plus the expected points of not bidding (the current high bid standing, or a redeal).
    pub fn neural_values(&self, obs: &Observation, legal: &[Action], model: &Mlp) -> (Vec<(Contract, f64, f64)>, f64) {
        let logits = model.forward(&bid_features(obs));
        let dist = |strain: Strain| -> Vec<(u8, f64)> {
            let block = strain_block(strain);
            softmax(&logits[block..block + 11]).into_iter().enumerate().map(|(t, p)| (t as u8, p)).collect()
        };
        let me = obs.seat;
        let baseline = match obs.auction.high_bid() {
            None => 0.0,
            Some((seat, contract)) if seat == me.partner() => self.expected(obs, &dist(contract.strain), contract, me),
            Some((seat, contract)) => {
                // An opponent's contract: how often it makes depends on what we hold in their strain
                // (finding 26), two times in three with a trick to spare. Valued by the same utility as
                // the bids (points, or the chance of winning the game), so the two always compare in
                // the same units.
                let ours: f64 = dist(contract.strain).iter().map(|&(t, p)| t as f64 * p).sum();
                let p = if self.config.fixed_opponents { 0.75 } else { opponents_make(contract.level, ours) };
                let level = contract.level;
                let spread = [(level.saturating_add(1).min(10), p * 2.0 / 3.0), (level, p / 3.0), (level.saturating_sub(1), 1.0 - p)];
                spread.iter().map(|&(tricks, p)| p * self.utility(obs, contract, seat, tricks)).sum()
            }
        };
        let mut bids = Vec::new();
        for strain in Strain::ALL {
            let d = dist(strain);
            for level in 7..=10 {
                let contract = Contract::new(level, strain);
                if !legal.contains(&Action::Call(Call::Bid(contract))) {
                    continue;
                }
                let value = self.expected(obs, &d, contract, me);
                let makes: f64 = d.iter().filter(|(t, _)| *t as f64 - self.config.dd_bias >= level as f64).map(|(_, p)| p).sum();
                bids.push((contract, value, makes));
            }
        }
        (bids, baseline)
    }

    /// How often the opponents' high bid makes, given our hand (`opponents_make`); `None` unless
    /// an opponent holds the high bid.
    pub fn opponents_chance(&self, obs: &Observation, model: &Mlp) -> Option<f64> {
        let (seat, contract) = obs.auction.high_bid()?;
        if seat.team() == obs.seat.team() {
            return None;
        }
        let logits = model.forward(&bid_features(obs));
        let block = strain_block(contract.strain);
        let ours: f64 = softmax(&logits[block..block + 11]).iter().enumerate().map(|(t, p)| t as f64 * p).sum();
        Some(opponents_make(contract.level, ours))
    }

    fn neural_call(&mut self, obs: &Observation, legal: &[Action], model: &Mlp) -> Action {
        let (bids, baseline) = self.neural_values(obs, legal, model);
        let best = bids.iter().copied().max_by(|a, b| a.1.total_cmp(&b.1)).map(|(c, v, m)| (v, c, m));
        let must_bid = obs.auction.must_bid();
        if let Some((value, contract, makes)) = best {
            self.note = format!("neural: best {contract} makes {:.0}%, EV {value:+.0} vs baseline {baseline:+.0}", 100.0 * makes);
            // No extra bar for leaving partner's strain: the network already reads partner's bid, and
            // where a 40-point bar alone made it pass, bidding scored 41 points more (finding 28).
            if must_bid || value > baseline + self.config.bid_margin {
                return Action::Call(Call::Bid(contract));
            }
        }
        if must_bid {
            return *legal.iter().find(|a| matches!(a, Action::Call(Call::Bid(_)))).expect("a forced bid exists");
        }
        // No bid: signal honestly, as the rule bot does (offering it only the non-bids).
        let calls: Vec<Action> = legal.iter().filter(|a| !matches!(a, Action::Call(Call::Bid(_)))).copied().collect();
        self.fallback.choose(obs, &calls)
    }

    fn call(&mut self, obs: &Observation, legal: &[Action]) -> Action {
        let is_legal = |call: Call| legal.contains(&Action::Call(call));
        let belief = Belief::new(obs, self.config.inference);
        let worlds = belief.sample_fitting(&mut self.rng, obs.hand, self.config.bid_samples);
        let me = obs.seat;

        // The baseline: what happens if we pass and the current high bid stands (0 if none).
        let baseline = match obs.auction.high_bid() {
            Some((seat, contract)) => {
                let results = self.simulate(&worlds, seat, contract.strain);
                self.expected(obs, &results, contract, seat)
            }
            None => 0.0,
        };

        // Our best strains by rough evaluation; simulate the top two (plus NT if it is close).
        let mut strains: Vec<(Strain, f32)> = Strain::ALL.iter().map(|&s| (s, hand_tricks(obs.hand, s))).collect();
        strains.sort_by(|a, b| b.1.total_cmp(&a.1));
        // NT replaces the second choice when it is close, keeping the work to two simulations.
        let mut candidates: Vec<Strain> = strains.iter().take(2).map(|&(s, _)| s).collect();
        if !candidates.contains(&Strain::NoTrumps) && strains.iter().any(|&(s, e)| s == Strain::NoTrumps && e + 1.0 >= strains[0].1) {
            candidates[1] = Strain::NoTrumps;
        }

        let mut best: Option<(f64, Contract, f64)> = None;
        for &strain in &candidates {
            let results = self.simulate(&worlds, me, strain);
            for level in 7..=10 {
                let contract = Contract::new(level, strain);
                if !is_legal(Call::Bid(contract)) {
                    continue;
                }
                let value = self.expected(obs, &results, contract, me);
                if best.is_none_or(|(v, _, _)| value > v) {
                    let weight: f64 = results.iter().map(|r| r.1).sum();
                    let bias = self.config.dd_bias;
                    let makes = results.iter().filter(|r| r.0 as f64 - bias >= level as f64).map(|r| r.1).sum::<f64>() / weight;
                    best = Some((value, contract, makes));
                }
            }
        }

        let must_bid = obs.auction.must_bid();
        if let Some((value, contract, makes)) = best {
            self.note = format!("best {contract}: makes {:.0}%, EV {value:+.0} vs baseline {baseline:+.0}", 100.0 * makes);
            // Taking partner out of their strain rests on our least reliable estimate (their
            // hidden hand), so demand a clear edge.
            let partner_strain = obs.auction.high_bid().filter(|(s, _)| *s == me.partner()).map(|(_, c)| c.strain);
            let margin = match partner_strain {
                Some(s) if s != contract.strain => self.config.bid_margin + 80.0,
                _ => self.config.bid_margin,
            };
            if must_bid || value > baseline + margin {
                return Action::Call(Call::Bid(contract));
            }
        }
        if must_bid {
            return *legal.iter().find(|a| matches!(a, Action::Call(Call::Bid(_)))).expect("a forced bid exists");
        }

        // No bid: signal to partner honestly, as the house rules intend.
        let best_strain = strains[0].0;
        if obs.hand.contains(Card::JOKER) && is_legal(Call::Six(best_strain)) {
            return Action::Call(Call::Six(best_strain));
        }
        for &(strain, _) in &strains {
            let shows = match strain.trump() {
                Some(suit) => has_bower(obs.hand, suit),
                None => Suit::ALL.iter().filter(|&&s| obs.hand.contains(Card::new(s, ACE))).count() >= 2,
            };
            if shows && is_legal(Call::Mention(strain)) {
                return Action::Call(Call::Mention(strain));
            }
        }
        Action::Call(Call::Pass)
    }
}

/// How often an opponent's bid at `level` makes, given `ours`: the tricks our side would expect as
/// declarer in their strain (the bidding network's estimate). A logistic fit to 5,328 Oracle hands
/// (`fh-lab opponents`, docs/lab/README.md finding 26): with 3 tricks of ours they make 7 about 88%
/// of the time, with 5 only 55%. Bids above 8 were too rare to measure, so 9 and 10 count as 9.
pub fn opponents_make(level: u8, ours: f64) -> f64 {
    let above = (level.saturating_sub(7)).min(2) as f64;
    1.0 / (1.0 + (-(4.694 - 1.015 * above - 0.898 * ours)).exp())
}

/// A rough chance that `team` wins the game from scores `after` this hand. Terminal positions
/// follow the rules (declarers count first); otherwise a logistic curve in the score gap, steeper
/// as the leader nears the target. The lab will replace this with measured values.
///
/// In a short game (H25) `hands_left` is the number of hands still to come after this one: with
/// none left the higher score wins (a tie plays on, so it is even); otherwise the gap counts for
/// less the more hands remain.
pub fn win_chance(rules: &Rules, after: [i32; 2], declarers: Team, team: Team, hands_left: Option<u8>) -> f64 {
    let (us, them) = (after[team.index()] as f64, after[1 - team.index()] as f64);
    if let Some(left) = hands_left {
        if left == 0 {
            return if us > them { 1.0 } else if us < them { 0.0 } else { 0.5 };
        }
        return 1.0 / (1.0 + (-(us - them) / (160.0 * (left as f64).sqrt())).exp());
    }
    let target = rules.target_score;
    let d = declarers.index();
    if after[d] >= target {
        return if team == declarers { 1.0 } else { 0.0 };
    }
    if after[1 - d] >= target {
        return if team == declarers { 0.0 } else { 1.0 };
    }
    let closeness = 1.0 + (us.max(them).max(0.0) / target as f64);
    1.0 / (1.0 + (-(us - them) * closeness / 250.0).exp())
}

impl Bot for SearchBot {
    fn name(&self) -> &str {
        &self.name
    }

    fn note(&self) -> Option<&str> {
        (!self.note.is_empty()).then_some(self.note.as_str())
    }

    fn choose(&mut self, obs: &Observation, legal: &[Action]) -> Action {
        if legal.len() == 1 {
            return legal[0];
        }
        if !self.config.signal_calls && obs.phase == Phase::Bidding {
            // A lab variant that never mentions or calls 6: those become passes.
            let plain: Vec<Action> = legal.iter().filter(|a| !matches!(a, Action::Call(Call::Mention(_) | Call::Six(_)))).copied().collect();
            if plain.len() < legal.len() {
                let mut variant = SearchConfig { signal_calls: true, ..self.config };
                std::mem::swap(&mut self.config, &mut variant);
                let action = self.choose(obs, &plain);
                std::mem::swap(&mut self.config, &mut variant);
                return action;
            }
        }
        match obs.phase {
            Phase::Bidding if self.config.neural_bidding => match if self.config.alt_bid_model { crate::models::alt_bid_model() } else { crate::models::bid_model() } {
                Some(model) => self.neural_call(obs, legal, model),
                None => self.fallback.choose(obs, legal),
            },
            Phase::Bidding if !self.config.search_bidding => self.fallback.choose(obs, legal),
            Phase::Discarding if !self.config.search_discards => self.fallback.choose(obs, legal),
            Phase::Bidding => self.call(obs, legal),
            Phase::Discarding => {
                self.discard_plan.retain(|&c| obs.hand.contains(c));
                if self.discard_plan.is_empty() {
                    self.discard_plan = self.plan_discards(obs);
                }
                Action::Discard(self.discard_plan.remove(0))
            }
            Phase::Playing => {
                let ranked = self.evaluate_plays(obs, legal);
                self.with_signal(obs, &ranked).unwrap_or(ranked[0].0)
            }
            Phase::GameOver => legal[0],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fh_core::{Event, Game, Rules};

    #[test]
    fn plays_complete_hands_legally() {
        for seed in 0..2 {
            let mut game = Game::new(Rules::default(), seed);
            let mut bots: [SearchBot; 4] = std::array::from_fn(|i| {
                let config = SearchConfig { play_samples: 3, min_play_samples: 1, bid_samples: 2, discard_samples: 2, ..SearchConfig::FAST };
                SearchBot::new("test", config, seed * 4 + i as u64)
            });
            loop {
                let seat = game.to_act().unwrap();
                let legal = game.legal_actions();
                let action = bots[seat.index()].choose(&game.observe(seat), &legal);
                assert!(legal.contains(&action), "seed {seed}: illegal {action}");
                let events = game.apply(action).unwrap();
                game.check_invariants().unwrap();
                if events.iter().any(|e| matches!(e, Event::HandScored { .. })) {
                    break;
                }
            }
        }
    }

    #[test]
    fn neural_bidding_loads_and_bids_legally() {
        assert!(crate::models::bid_model().is_some(), "embedded model must load");
        for seed in 0..20 {
            let game = Game::new(Rules::default(), seed);
            let seat = game.to_act().unwrap();
            let legal = game.legal_actions();
            let action = SearchBot::new("t", SearchConfig::ORACLE, seed).choose(&game.observe(seat), &legal);
            assert!(legal.contains(&action));
        }
    }

    #[test]
    fn win_chance_follows_the_rules_at_the_target() {
        let rules = Rules::default();
        let ns = Team::NorthSouth;
        assert_eq!(win_chance(&rules, [510, 520], ns, ns, None), 1.0, "declarers count first");
        assert_eq!(win_chance(&rules, [480, 500], ns, ns, None), 0.0, "defenders can win");
        let even = win_chance(&rules, [200, 200], ns, ns, None);
        assert!((even - 0.5).abs() < 1e-9);
        assert!(win_chance(&rules, [400, 100], ns, ns, None) > 0.8);
    }

    #[test]
    fn win_chance_in_a_short_game_counts_the_hands_left() {
        let rules = Rules { hand_limit: Some(4), ..Rules::default() };
        let ns = Team::NorthSouth;
        assert_eq!(win_chance(&rules, [30, 20], ns, ns, Some(0)), 1.0, "last hand: ahead wins");
        assert_eq!(win_chance(&rules, [30, 30], ns, ns, Some(0)), 0.5, "a tie plays on");
        let late = win_chance(&rules, [200, 0], ns, ns, Some(1));
        let early = win_chance(&rules, [200, 0], ns, ns, Some(3));
        assert!(late > early && early > 0.5);
    }
}
