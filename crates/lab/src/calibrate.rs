//! `fh-lab calibrate`: how well does simulated double-dummy bidding predict real results?
//!
//! For each deal, South (first to speak) considers its best strain by rough evaluation and we
//! compare three trick counts for South's side as declarer:
//!   predicted — the bidding simulation's mean over sampled worlds (what the bot believes),
//!   true DD   — a double-dummy solve of the actual cards (perfect play, all cards seen),
//!   actual    — South declares 7 of that strain and four search bots play it out.

use crate::make_bot;
use fh_ai::evaluate::hand_tricks;
use fh_ai::rule_bot::heuristic_discards;
use fh_ai::solver::{Position, Solver};
use fh_ai::{SearchBot, SearchConfig};
use fh_core::{Action, Call, CardSet, Contract, Event, Game, Rules, Seat, Strain, Trick};

/// One deal: (predicted tricks, true double-dummy tricks, actual tricks, rough evaluation).
type Row = (f64, u8, u8, f32);

pub fn run(deals: u64, seed: u64, threads: u64, player: &str) {
    let rows: Vec<Row> = std::thread::scope(|scope| {
        let handles: Vec<_> = (0..threads)
            .map(|t| {
                scope.spawn(move || {
                    let mut rows = Vec::new();
                    let mut d = t;
                    while d < deals {
                        rows.push(one(seed + d, player));
                        d += threads;
                    }
                    rows
                })
            })
            .collect();
        handles.into_iter().flat_map(|h| h.join().unwrap()).collect()
    });
    let n = rows.len() as f64;
    let mean = |f: &dyn Fn(&Row) -> f64| rows.iter().map(f).sum::<f64>() / n;
    println!("{} deals; South declares its best strain at the 7 level; play by {player}", rows.len());
    println!("  predicted (simulation)  {:.2} tricks", mean(&|r| r.0));
    println!("  true double-dummy       {:.2} tricks", mean(&|r| r.1 as f64));
    println!("  actual play             {:.2} tricks", mean(&|r| r.2 as f64));
    println!("  rough evaluation + 2    {:.2} tricks  (hand_tricks plus 2 for partner and kitty)", mean(&|r| r.3 as f64 + 2.0));
    println!("  bias: predicted − actual {:+.2}, true DD − actual {:+.2}", mean(&|r| r.0 - r.2 as f64), mean(&|r| r.1 as f64 - r.2 as f64));
    let made = |f: &dyn Fn(&Row) -> bool| 100.0 * rows.iter().filter(|r| f(r)).count() as f64 / n;
    println!("  7-level made: predicted mean ≥ 7 on {:.0}% of deals; actually made {:.0}%", made(&|r| r.0 >= 7.0), made(&|r| r.2 >= 7));
}

fn one(seed: u64, player: &str) -> Row {
    let dealer = Seat::EAST; // South speaks first, knowing nothing about the others
    let game = Game::with_dealer(Rules::default(), seed, dealer);
    let obs = game.observe(Seat::SOUTH);
    let strain = Strain::ALL.into_iter().max_by(|&a, &b| hand_tricks(obs.hand, a).total_cmp(&hand_tricks(obs.hand, b))).unwrap();
    let rough = hand_tricks(obs.hand, strain);

    let mut bot = SearchBot::new("calibrate", SearchConfig::COUNTER, seed);
    let sims = bot.simulate_from(&obs, Seat::SOUTH, strain, 16);
    let weight: f64 = sims.iter().map(|s| s.1).sum();
    let predicted = sims.iter().map(|&(t, w)| t as f64 * w).sum::<f64>() / weight;

    // True double-dummy with the real cards, using the same heuristic discards.
    let trump = strain.trump();
    let mut hands: [CardSet; 4] = std::array::from_fn(|i| game.hand(Seat::new(i)));
    let thirteen = hands[0].union(game.kitty());
    hands[0] = thirteen.minus(heuristic_discards(thirteen, trump, 3).into_iter().collect());
    let true_dd = Solver::new(trump).solve(&Position { hands, trump, trick: Trick::new(Seat::SOUTH) }, Seat::SOUTH.team());

    // Actual play: South declares 7 of the strain; everyone else passes.
    let mut game = Game::with_dealer(Rules::default(), seed, dealer);
    game.apply(Action::Call(Call::Bid(Contract::new(7, strain)))).unwrap();
    for _ in 0..3 {
        game.apply(Action::Call(Call::Pass)).unwrap();
    }
    let mut bots: [Box<dyn fh_ai::Bot>; 4] = std::array::from_fn(|i| make_bot(player, seed * 4 + i as u64));
    loop {
        let seat = game.to_act().unwrap();
        let action = bots[seat.index()].choose(&game.observe(seat), &game.legal_actions());
        if game.apply(action).unwrap().iter().any(|e| matches!(e, Event::HandScored { .. })) {
            break;
        }
    }
    let actual = game.history().last().unwrap().tricks_won[0];
    (predicted, true_dd, actual, rough)
}
