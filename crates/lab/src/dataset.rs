//! `fh-lab dataset`: training data for the neural bidding evaluator.
//!
//! For each deal, the four players bid with the expert's rule bidding. At each player's turn we
//! record what they could see (`features::bid_features`). The label is what that player's side
//! actually takes as declarer in each strain: the player adds the real kitty, discards by the
//! kitty heuristic, leads, and the double-dummy solver plays all four hands perfectly.
//!
//! Output: `<out>.x` (f32 rows of BID_FEATURES), `<out>.y` (u8 rows of 5 trick counts, 255 when a
//! solve hit the node limit), and `<out>.json` describing the shapes.

use fh_ai::features::{bid_features, BID_FEATURES};
use fh_ai::rule_bot::heuristic_discards;
use fh_ai::solver::{Position, Solver};
use fh_ai::{Bot, RuleBot, SearchConfig, Style};
use fh_core::{CardSet, Game, Phase, Rules, Seat, Strain, Trick};
use std::io::Write;
use std::time::Instant;

pub fn run(deals: u64, seed: u64, threads: u64, out: &str, bidder: &str) {
    let start = Instant::now();
    let rows: Vec<([f32; BID_FEATURES], [u8; 5])> = std::thread::scope(|scope| {
        let handles: Vec<_> = (0..threads)
            .map(|t| {
                scope.spawn(move || {
                    let mut rows = Vec::new();
                    let mut d = t;
                    while d < deals {
                        rows.extend(one_deal(seed.wrapping_add(d), bidder));
                        d += threads;
                        if t == 0 && d % (threads * 250) == 0 {
                            eprintln!("  {} of {deals} deals, {:.0}s", d, start.elapsed().as_secs_f64());
                        }
                    }
                    rows
                })
            })
            .collect();
        handles.into_iter().flat_map(|h| h.join().unwrap()).collect()
    });

    let mut x = std::io::BufWriter::new(std::fs::File::create(format!("{out}.x")).expect("create .x"));
    let mut y = std::io::BufWriter::new(std::fs::File::create(format!("{out}.y")).expect("create .y"));
    for (features, labels) in &rows {
        for f in features {
            x.write_all(&f.to_le_bytes()).unwrap();
        }
        y.write_all(labels).unwrap();
    }
    let meta = format!(
        "{{\"rows\": {}, \"features\": {BID_FEATURES}, \"labels\": 5, \"label_meaning\": \"tricks for the bidder's side as declarer in S C D H NT (255 = unsolved)\", \"deals\": {deals}, \"seed\": {seed}}}\n",
        rows.len()
    );
    std::fs::write(format!("{out}.json"), meta).expect("write .json");
    println!("{} rows from {deals} deals in {:.0}s → {out}.x / .y / .json", rows.len(), start.elapsed().as_secs_f64());
}

/// `bidder`: "rule" (the expert's rule bidding) or any persona id, e.g. "oracle" for self-play.
fn one_deal(seed: u64, bidder: &str) -> Vec<([f32; BID_FEATURES], [u8; 5])> {
    let dealer = Seat::new((seed % 4) as usize);
    let mut game = Game::with_dealer(Rules::default(), seed, dealer);
    let hands: [CardSet; 4] = std::array::from_fn(|i| game.hand(Seat::new(i)));
    let kitty = game.kitty();
    let style = Style { aggression: SearchConfig::COUNTER.rule_aggression, ..Style::STEADY };
    let mut bots: [Box<dyn Bot>; 4] = std::array::from_fn(|i| -> Box<dyn Bot> {
        match bidder {
            "rule" => Box::new(RuleBot::new("data", style, seed * 4 + i as u64)),
            id => crate::make_bot(id, seed * 4 + i as u64),
        }
    });

    // Features at each player's turn.
    let mut features: Vec<(Seat, [f32; BID_FEATURES])> = Vec::with_capacity(4);
    while game.phase() == Phase::Bidding && game.deal_number() == 1 && features.len() < 4 {
        let seat = game.to_act().unwrap();
        let obs = game.observe(seat);
        features.push((seat, bid_features(&obs)));
        let legal = game.legal_actions();
        let action = bots[seat.index()].choose(&obs, &legal);
        game.apply(action).unwrap();
    }

    // Labels: each seat as declarer in each strain.
    features
        .into_iter()
        .map(|(seat, x)| {
            let mut labels = [255u8; 5];
            for strain in Strain::ALL {
                let trump = strain.trump();
                let mut h = hands;
                let thirteen = h[seat.index()].union(kitty);
                let drop: CardSet = heuristic_discards(thirteen, trump, 3).into_iter().collect();
                h[seat.index()] = thirteen.minus(drop);
                let mut solver = Solver::with_limit(trump, 3_000_000);
                let tricks = solver.solve(&Position { hands: h, trump, trick: Trick::new(seat) }, seat.team());
                if !solver.aborted() {
                    labels[strain.index()] = tricks;
                }
            }
            (x, labels)
        })
        .collect()
}
