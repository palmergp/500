//! `fh-lab coach`: play a hand with a bot in South's seat and print the coach's grade and
//! reasons for every South decision, plus the hindsight trace. For checking explanations.

use crate::make_bot;
use fh_ai::coach::{dd_trace, Coach};
use fh_core::{Game, Phase, Rules, Seat};

pub fn run(seed: u64, south: &str) {
    let mut game = Game::new(Rules::default(), seed);
    let mut bots: [Box<dyn fh_ai::Bot>; 4] =
        std::array::from_fn(|i| make_bot(if i == 0 { south } else { "steady" }, seed * 4 + i as u64));
    let mut coach = Coach::new(seed);
    let start = game.history().len();
    println!("Seed {seed}. South ({south}) hand: {}", game.hand(Seat::SOUTH));
    while game.history().len() == start && game.phase() != Phase::GameOver {
        let seat = game.to_act().unwrap();
        let legal = game.legal_actions();
        let obs = game.observe(seat);
        let action = bots[seat.index()].choose(&obs, &legal);
        if seat == Seat::SOUTH && legal.len() > 1 {
            let grade = coach.grade(&obs, &legal, action);
            let best = if grade.best == action { String::new() } else { format!("  (best: {})", grade.best) };
            println!("\nSouth {action}: {} (−{:.0}){best}", grade.verdict.label(), grade.loss);
            for reason in &grade.analysis.reasons {
                println!("    · {reason}");
            }
        } else if obs.phase == Phase::Bidding || seat == Seat::SOUTH {
            println!("{seat} {action}");
        }
        game.apply(action).unwrap();
    }
    if let Some(record) = game.history().get(start) {
        if let Some((par, plays)) = dd_trace(record) {
            let (declarer, contract) = record.contract.unwrap();
            println!("\n{declarer} played {contract}; took {} (perfect play from the start: {par})", record.tricks_won[declarer.team().index()]);
            for p in plays.iter().filter(|p| p.cost > 0) {
                println!("    {} {} cost {} trick(s) with all cards visible (par after: {})", p.seat, p.card, p.cost, p.par_after);
            }
        }
    }
}
