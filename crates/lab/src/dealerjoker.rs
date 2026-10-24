//! `fh-lab dealerjoker`: the dealer, without the Joker, with nobody having called 6: how well do the
//! bidding network and the count judge the dealer's best suit, now that the third caller (whose
//! partner has called) no longer shows the Joker (finding 35)?
//!
//! The Oracle bids for West, North and East; at the dealer's (South's) call we note the network's
//! expected tricks for South's best-count suit and the count, then solve that suit with South
//! declaring (kitty taken, the bots' discards). Grouped by where the Joker really was.

use std::collections::BTreeMap;

use fh_ai::features::{bid_features, strain_block};
use fh_ai::nn::softmax;
use fh_ai::rule_bot::heuristic_discards;
use fh_ai::solver::{Position, Solver};
use fh_core::{Call, Card, CardSet, Game, Phase, Rules, Seat, Strain, Trick};

#[derive(Default, Clone, Copy)]
struct Row {
    n: u32,
    net_error: f64,
    count_error: f64,
}

pub fn run(deals: u64, seed: u64, threads: u64) {
    let parts: Vec<BTreeMap<String, Row>> = std::thread::scope(|scope| {
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
    let mut all: BTreeMap<String, Row> = BTreeMap::new();
    for part in parts {
        for (k, r) in part {
            let e = all.entry(k).or_default();
            e.n += r.n;
            e.net_error += r.net_error;
            e.count_error += r.count_error;
        }
    }
    println!("The dealer without the Joker, nobody having called 6 ({deals} deals): actual tricks (solver) less the estimate\n");
    println!("| The Joker was | n | Less the network's estimate | Less the count (real play) |\n|---|---|---|---|");
    for (k, r) in &all {
        let n = r.n as f64;
        println!("| {k} | {} | {:+.2} | {:+.2} |", r.n, r.net_error / n, r.count_error / n);
    }
}

fn one(seed: u64, out: &mut BTreeMap<String, Row>) {
    let model = fh_ai::models::bid_model().expect("the bidding network");
    let mut game = Game::with_dealer(Rules::default(), seed, Seat::SOUTH);
    if game.hand(Seat::SOUTH).contains(Card::JOKER) {
        return;
    }
    let mut bots: [Box<dyn fh_ai::Bot>; 4] = std::array::from_fn(|i| crate::make_bot("oracle", seed.wrapping_mul(4).wrapping_add(i as u64)));
    while game.phase() == Phase::Bidding && game.to_act() != Some(Seat::SOUTH) {
        let seat = game.to_act().unwrap();
        let action = bots[seat.index()].choose(&game.observe(seat), &game.legal_actions());
        game.apply(action).unwrap();
    }
    let obs = game.observe(Seat::SOUTH);
    let quiet = !obs.auction.calls().any(|(_, c)| matches!(c, Call::Six(_) | Call::Bid(_)));
    // A reference: every fifth deal whatever the auction (the count's usual error for the dealer).
    if !quiet && !seed.is_multiple_of(5) {
        return;
    }
    let Some((strain, count)) = Strain::ALL.into_iter().filter(|s| s.trump().is_some()).map(|s| (s, fh_ai::thumb::count(&obs, s).0)).max_by(|a, b| a.1.total_cmp(&b.1)) else { return };
    let logits = model.forward(&bid_features(&obs));
    let block = strain_block(strain);
    let expected: f64 = softmax(&logits[block..block + 11]).iter().enumerate().map(|(t, p)| t as f64 * p).sum();
    let trump = strain.trump();
    let mut hands: [CardSet; 4] = std::array::from_fn(|i| game.hand(Seat::new(i)));
    let thirteen = hands[0].union(game.kitty());
    hands[0] = thirteen.minus(heuristic_discards(thirteen, trump, 3).into_iter().collect());
    let mut solver = Solver::with_limit(trump, 4_000_000);
    let tricks = solver.solve(&Position { hands, trump, trick: Trick::new(Seat::SOUTH) }, Seat::SOUTH.team()) as f64;
    if solver.aborted() {
        return;
    }
    let place = if game.kitty().contains(Card::JOKER) {
        "in the kitty"
    } else if game.hand(Seat::EAST).contains(Card::JOKER) {
        "with East (third to call, after their partner)"
    } else {
        "with West or North (who should have called 6)"
    };
    let keys: Vec<&str> = if quiet { vec![place, "all, nobody bid or called 6"] } else { vec![] };
    let reference = if seed.is_multiple_of(5) { vec!["reference: any auction (every fifth deal)"] } else { vec![] };
    for key in keys.into_iter().chain(reference) {
        let e = out.entry(key.to_string()).or_default();
        e.n += 1;
        e.net_error += tricks - expected;
        e.count_error += tricks - 0.5 - count as f64;
    }
}
