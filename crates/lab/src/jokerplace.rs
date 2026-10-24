//! `fh-lab jokerplace`: when you speak last and nobody called 6, where is the Joker, and what is
//! that worth to your count?
//!
//! Under the house convention (call 6 with the Joker unless partner has already called), the
//! first two callers would have shown it; the third, whose partner has called, wouldn't. So for
//! the dealer it's with the right-hand opponent or in the kitty. On random deals where the dealer
//! (South) lacks the Joker, South declares its best suit (count 6 or more), takes the kitty and
//! discards as the bots do; the hand is solved, and tricks less South's count are grouped by where
//! the Joker was.

use std::collections::BTreeMap;

use fh_ai::rule_bot::heuristic_discards;
use fh_ai::solver::{Position, Solver};
use fh_core::{Card, CardSet, Game, Rules, Seat, Strain, Trick};

pub fn run(deals: u64, seed: u64, threads: u64) {
    let parts: Vec<BTreeMap<&'static str, (u32, f64, f64)>> = std::thread::scope(|scope| {
        let handles: Vec<_> = (0..threads)
            .map(|t| {
                scope.spawn(move || {
                    let mut out = BTreeMap::new();
                    let mut d = t;
                    while d < deals {
                        one(seed + d, &mut out);
                        d += threads;
                    }
                    out
                })
            })
            .collect();
        handles.into_iter().map(|h| h.join().unwrap()).collect()
    });
    let mut all: BTreeMap<&str, (u32, f64, f64)> = BTreeMap::new();
    for part in parts {
        for (k, (n, s, sq)) in part {
            let e = all.entry(k).or_default();
            e.0 += n;
            e.1 += s;
            e.2 += sq;
        }
    }
    println!("The dealer without the Joker declares its best suit: real-play tricks less its count, by where the Joker was ({deals} deals)\n");
    println!("| The Joker was | n | Tricks less the count (95%) |\n|---|---|---|");
    for (k, &(n, s, sq)) in &all {
        let n = n as f64;
        let mean = s / n;
        let sd = (sq / n - mean * mean).max(0.0).sqrt();
        println!("| {k} | {} | {mean:+.2} ± {:.2} |", n as u32, 1.96 * sd / n.sqrt());
    }
    // Not knowing: the Joker is with each other player 10 times in 33, in the kitty 3 in 33.
    // Knowing nobody before you called 6: with the right-hand opponent 10 in 13, the kitty 3 in 13.
    let m = |k: &str| all.get(k).map_or(0.0, |&(n, s, _)| s / n as f64);
    let unknown = (10.0 * (m("with partner (North)") + m("with the left-hand opponent (West)") + m("with the right-hand opponent (East)")) + 3.0 * m("in the kitty")) / 33.0;
    let placed = (10.0 * m("with the right-hand opponent (East)") + 3.0 * m("in the kitty")) / 13.0;
    println!("\nNot knowing where it is: {unknown:+.2}. Placed by the auction (East or the kitty): {placed:+.2}. Difference: {:+.2} tricks.", placed - unknown);
}

fn one(seed: u64, out: &mut BTreeMap<&'static str, (u32, f64, f64)>) {
    let game = Game::with_dealer(Rules::default(), seed, Seat::SOUTH);
    if game.hand(Seat::SOUTH).contains(Card::JOKER) {
        return;
    }
    let obs = game.observe(Seat::SOUTH);
    let Some((strain, count)) = Strain::ALL.into_iter().filter(|s| s.trump().is_some()).map(|s| (s, fh_ai::thumb::count(&obs, s).0)).max_by(|a, b| a.1.total_cmp(&b.1)) else { return };
    if count < 6.0 {
        return;
    }
    let trump = strain.trump();
    let mut hands: [CardSet; 4] = std::array::from_fn(|i| game.hand(Seat::new(i)));
    let thirteen = hands[0].union(game.kitty());
    hands[0] = thirteen.minus(heuristic_discards(thirteen, trump, 3).into_iter().collect());
    let mut solver = Solver::with_limit(trump, 4_000_000);
    let tricks = solver.solve(&Position { hands, trump, trick: Trick::new(Seat::SOUTH) }, Seat::SOUTH.team());
    if solver.aborted() {
        return;
    }
    let place = if game.kitty().contains(Card::JOKER) {
        "in the kitty"
    } else if game.hand(Seat::NORTH).contains(Card::JOKER) {
        "with partner (North)"
    } else if game.hand(Seat::WEST).contains(Card::JOKER) {
        "with the left-hand opponent (West)"
    } else {
        "with the right-hand opponent (East)"
    };
    let r = tricks as f64 - 0.5 - count as f64;
    let e = out.entry(place).or_default();
    e.0 += 1;
    e.1 += r;
    e.2 += r * r;
}
