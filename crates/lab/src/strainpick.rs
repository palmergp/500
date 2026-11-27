//! `fh-lab strainpick`: the rule bots' hand estimate (`evaluate::hand_tricks`) picks the strain and
//! level for Steady, The Rock, The Gambler, Rookie, The Counter and The Shark. A user saw The Gambler
//! open 7♦ holding 4♦ 8♦ and A-K-x of hearts with three aces: the estimate counts the A♥ K♥ as side
//! winners (1.6) above the same cards as trumps (1.2), and nothing marks two trumps as short. Here,
//! for the first player to call in each deal, every strain is solved with every card seen (the
//! bidder takes the kitty and discards as the bots do), and estimates with a penalty for each trump
//! short of four or five are compared with today's: the strain each picks, the level each bids, and
//! what that would score (real play taken as half a trick under the solver, as the bots assume).

use fh_ai::evaluate::hand_tricks;
use fh_ai::rule_bot::heuristic_discards;
use fh_ai::solver::{Position, Solver};
use fh_core::game::score_hand;
use fh_core::rules::is_trump;
use fh_core::{Contract, Game, Rules, Seat, Strain, Trick};

/// The variants: (label, trumps short of which length, penalty per trump short).
const VARIANTS: [(&str, u8, f32); 7] = [
    ("today", 4, 0.0),
    ("−0.3 a trump short of 4", 4, 0.3),
    ("−0.6 a trump short of 4", 4, 0.6),
    ("−0.9 a trump short of 4", 4, 0.9),
    ("−0.3 a trump short of 5", 5, 0.3),
    ("−0.5 a trump short of 5", 5, 0.5),
    ("−0.7 a trump short of 5", 5, 0.7),
];

/// The styles' additions to the hand estimate: partner, kitty and aggression.
const STYLES: [(&str, f32); 3] = [("Steady", 1.5 + 0.5), ("The Counter", 1.5 + 0.5 + 0.7), ("The Gambler", 1.8 + 0.7 + 0.8)];

#[derive(Default, Clone)]
struct Tally {
    deals: f64,
    bids: f64,
    points: f64,
    points2: f64,
    /// Bids whose strain was the solver's best for this hand, and the tricks given up to it.
    best_strain: f64,
    tricks_short: f64,
    made: f64,
}

pub fn run(deals: u64, seed: u64, threads: u64) {
    let parts: Vec<Vec<Tally>> = std::thread::scope(|scope| {
        let handles: Vec<_> = (0..threads)
            .map(|t| {
                scope.spawn(move || {
                    let mut out = vec![Tally::default(); VARIANTS.len() * STYLES.len()];
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
    let mut all = vec![Tally::default(); VARIANTS.len() * STYLES.len()];
    for part in parts {
        for (a, p) in all.iter_mut().zip(part) {
            a.deals += p.deals;
            a.bids += p.bids;
            a.points += p.points;
            a.points2 += p.points2;
            a.best_strain += p.best_strain;
            a.tricks_short += p.tricks_short;
            a.made += p.made;
        }
    }
    println!("{deals} deals. The first player to call, bidding by the rule estimate (with each style's partner, kitty and");
    println!("aggression allowance) or passing below 7; every strain solved with every card seen, real play half a trick under.\n");
    println!("| Style | Estimate | Bids | Made | Strain the solver's best | Tricks short of the best strain | Points a deal (95%) |");
    println!("|---|---|---|---|---|---|---|");
    for (si, (style, _)) in STYLES.iter().enumerate() {
        for (vi, (label, _, _)) in VARIANTS.iter().enumerate() {
            let t = &all[si * VARIANTS.len() + vi];
            let n = t.deals.max(1.0);
            let b = t.bids.max(1.0);
            let ci = 1.96 * ((t.points2 / n - (t.points / n).powi(2)).max(0.0) / n).sqrt();
            println!(
                "| {style} | {label} | {:.0}% | {:.0}% | {:.0}% | {:.2} | {:+.1} ± {:.1} |",
                100.0 * t.bids / n,
                100.0 * t.made / b,
                100.0 * t.best_strain / b,
                t.tricks_short / b,
                t.points / n,
                ci
            );
        }
    }
}

fn deal(seed: u64, out: &mut [Tally]) {
    let game = Game::new(Rules::default(), seed);
    let bidder = game.to_act().expect("someone calls first");
    let hand = game.hand(bidder);
    let kitty = game.kitty();
    let rules = Rules::default();
    // The solver's tricks for the bidder's side in each strain, after taking the kitty.
    let mut tricks = [0u8; 5];
    for (i, strain) in Strain::ALL.into_iter().enumerate() {
        let trump = strain.trump();
        let full = hand.union(kitty);
        let discards: fh_core::CardSet = heuristic_discards(full, trump, 3).into_iter().collect();
        let mut hands: [fh_core::CardSet; 4] = std::array::from_fn(|s| game.hand(Seat::new(s)));
        hands[bidder.index()] = full.minus(discards);
        tricks[i] = Solver::with_limit(trump, 4_000_000).solve(&Position { hands, trump, trick: Trick::new(bidder) }, bidder.team());
    }
    let best = *tricks.iter().max().expect("five strains");
    for (si, &(_, extra)) in STYLES.iter().enumerate() {
        for (vi, &(_, short_of, penalty)) in VARIANTS.iter().enumerate() {
            let estimate = |strain: Strain| {
                let n = strain.trump().map_or(short_of, |t| hand.iter().filter(|&c| is_trump(c, Some(t)) || c.is_joker()).count() as u8);
                hand_tricks(hand, strain) - penalty * f32::from(short_of.saturating_sub(n)) + extra
            };
            let (i, strain, value) = Strain::ALL
                .into_iter()
                .enumerate()
                .map(|(i, s)| (i, s, estimate(s)))
                .max_by(|a, b| a.2.total_cmp(&b.2))
                .expect("five strains");
            let t = &mut out[si * VARIANTS.len() + vi];
            t.deals += 1.0;
            let level = value.floor().min(10.0) as u8;
            if level < 7 {
                continue;
            }
            let contract = Contract::new(level, strain);
            // Real play: half a trick under the solver, so a bid made exactly by the solver makes half the time.
            let real = f32::from(tricks[i]) - 0.5;
            let made_tricks = if real >= f32::from(level) { level.max(tricks[i].saturating_sub(1)) } else { level - 1 };
            let score = |taken: u8| {
                let team = bidder.team();
                let mut won = [0u8; 2];
                won[team.index()] = taken;
                won[1 - team.index()] = 10 - taken;
                let change = score_hand(&rules, contract, team, won, [0, 0]);
                f64::from(change[team.index()] - change[1 - team.index()])
            };
            // Exactly on the line, made half the time.
            let points = if (real - f32::from(level)).abs() < 0.01 { 0.5 * (score(level) + score(level - 1)) } else { score(made_tricks) };
            t.bids += 1.0;
            t.points += points;
            t.points2 += points * points;
            t.made += if real > f32::from(level) { 1.0 } else if (real - f32::from(level)).abs() < 0.01 { 0.5 } else { 0.0 };
            t.best_strain += f64::from(u8::from(tricks[i] == best));
            t.tricks_short += f64::from(best - tricks[i]);
        }
    }
}
