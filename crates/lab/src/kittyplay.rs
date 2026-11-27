//! `fh-lab kittyplay`: is the coach's kitty plan the set to teach?
//!
//! The coach grades your three discards against a plan made by a rule (`heuristic_discards`: the
//! set scored by voids and what each card is worth keeping, finding 40; in no trumps, one card at a
//! time keeping stoppers, finding 23). The rule was tuned with every card visible. Here, at each
//! declarer's discard, the plan and the next sets the rule ranks are compared on 20 deals sampled
//! from the declarer's view, reading the auction (as the coach's kitty grades now do, finding 58),
//! and the best of them is "the searched choice". Where it differs from the plan, both are played
//! out by The Oracle in every seat, the same seed both ways.

use std::collections::BTreeMap;

use crate::playout::{finish, Mean};
use fh_ai::belief::Belief;
use fh_ai::rule_bot::{heuristic_discards, ranked_discards, KittyWeights};
use fh_ai::search_bot::{SearchBot, SearchConfig};
use fh_ai::solver::{Position, Solver};
use fh_ai::Bot;
use fh_core::game::score_hand;
use fh_core::{Action, Card, CardSet, Contract, Event, Game, Phase, Rng, Rules, Seat, Trick};

#[derive(Default, Clone)]
struct Tally {
    hands: f64,
    differ: f64,
    /// Where they differ: the searched choice's points less the plan's, played out; and the gap
    /// the 20 deals predicted.
    real: Mean,
    real_tricks: Mean,
    predicted: Mean,
    made_plan: f64,
    made_searched: f64,
}

const WORLDS: usize = 20;
const CANDIDATES: usize = 3;

pub fn run(deals: u64, seed: u64, threads: u64) {
    let parts: Vec<BTreeMap<&'static str, Tally>> = std::thread::scope(|scope| {
        let handles: Vec<_> = (0..threads)
            .map(|t| {
                scope.spawn(move || {
                    let mut out = BTreeMap::new();
                    let mut d = t;
                    while d < deals {
                        deal(seed + d, &mut out);
                        d += threads;
                    }
                    out
                })
            })
            .collect();
        handles.into_iter().map(|h| h.join().unwrap()).collect()
    });
    let mut all: BTreeMap<&'static str, Tally> = BTreeMap::new();
    for part in parts {
        for (k, t) in part {
            let e = all.entry(k).or_default();
            e.hands += t.hands;
            e.differ += t.differ;
            e.real.merge(&t.real);
            e.real_tricks.merge(&t.real_tricks);
            e.predicted.merge(&t.predicted);
            e.made_plan += t.made_plan;
            e.made_searched += t.made_searched;
        }
    }
    println!("{deals} deals. At the declarer's discard: the coach's plan, and the best of it and the next {} sets the rule", CANDIDATES - 1);
    println!("ranks, judged on {WORLDS} deals that read the auction. Where they differ, both played out by The Oracle.\n");
    println!("| Bid | Declared hands | Searched choice differs | Predicted gain | Real gain, points (95%) | Real gain, tricks | Bid made: plan / searched |");
    println!("|---|---|---|---|---|---|---|");
    for (k, t) in &all {
        let d = t.differ.max(1.0);
        println!(
            "| {k} | {} | {:.0}% | {:+.1} | {} | {} | {:.0}% / {:.0}% |",
            t.hands,
            100.0 * t.differ / t.hands.max(1.0),
            t.predicted.mean(),
            t.real.show(1),
            t.real_tricks.show(2),
            100.0 * t.made_plan / d,
            100.0 * t.made_searched / d
        );
    }
}

fn points(rules: &Rules, contract: Contract, declarer: Seat, tricks: u8) -> f64 {
    let team = declarer.team();
    let mut won = [0u8; 2];
    won[team.index()] = tricks;
    won[1 - team.index()] = 10 - tricks;
    let change = score_hand(rules, contract, team, won, [0, 0]);
    f64::from(change[team.index()] - change[1 - team.index()])
}

fn deal(seed: u64, out: &mut BTreeMap<&'static str, Tally>) {
    let mut game = Game::new(Rules::default(), seed);
    let mut bots: [SearchBot; 4] = std::array::from_fn(|i| SearchBot::new("The Oracle", SearchConfig::ORACLE, seed * 4 + i as u64));
    while game.phase() == Phase::Bidding {
        let Some(seat) = game.to_act() else { return };
        let action = bots[seat.index()].choose(&game.observe(seat), &game.legal_actions());
        let Ok(events) = game.apply(action) else { return };
        if events.iter().any(|e| matches!(e, Event::PassedOut { .. })) {
            return;
        }
    }
    if game.phase() != Phase::Discarding {
        return;
    }
    let (declarer, contract) = game.contract().expect("a bid");
    let trump = contract.trump();
    let obs = game.observe(declarer);
    let hand = obs.hand;
    let set = |cards: &[Card]| cards.iter().copied().collect::<CardSet>();
    let plan = heuristic_discards(hand, trump, 3);
    let mut candidates = vec![plan.clone()];
    for c in ranked_discards(hand, trump, 3, KittyWeights::DEFAULT) {
        if candidates.len() >= CANDIDATES {
            break;
        }
        if !candidates.iter().any(|x| set(x) == set(&c)) {
            candidates.push(c);
        }
    }
    // Judged on deals that read the auction, as the coach's kitty grades sample them.
    let belief = Belief::new_with(&obs, true, true);
    let worlds = belief.sample_fitting(&mut Rng::new(seed ^ 0x6b17), hand, WORLDS);
    let rules = game.rules().clone();
    let mut solver = Solver::with_limit(trump, 1_500_000);
    let values: Vec<f64> = candidates
        .iter()
        .map(|c| {
            worlds
                .iter()
                .map(|w| {
                    let mut hands = w.hands;
                    hands[declarer.index()] = hand.minus(set(c));
                    points(&rules, contract, declarer, solver.solve(&Position { hands, trump, trick: Trick::new(declarer) }, declarer.team()))
                })
                .sum::<f64>()
                / worlds.len().max(1) as f64
        })
        .collect();
    let best = (0..candidates.len()).max_by(|&a, &b| values[a].total_cmp(&values[b])).expect("candidates");
    let key = if trump.is_some() { "suit bids" } else { "no trumps" };
    let t = out.entry(key).or_default();
    t.hands += 1.0;
    if best == 0 || values[best] - values[0] < 1.0 {
        return;
    }
    let play = |discards: &[Card]| {
        let mut g = game.clone();
        for &c in discards {
            g.apply(Action::Discard(c)).ok()?;
        }
        finish(g, seed.wrapping_add(9_000_011), declarer)
    };
    let (Some(a), Some(b)) = (play(&candidates[best]), play(&plan)) else { return };
    t.differ += 1.0;
    t.predicted.add(values[best] - values[0]);
    t.real.add(a.points - b.points);
    t.real_tricks.add(f64::from(a.tricks - b.tricks));
    t.made_plan += f64::from(u8::from(b.made));
    t.made_searched += f64::from(u8::from(a.made));
}
