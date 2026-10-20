//! `fh-lab opponents`: how often does an opponent's bid make, given your hand?
//!
//! The Oracle plays whole hands. Whenever a player passes over an opponent's bid that ends up as the
//! contract, we note the player's own strength in that strain (the bidding network's expected
//! tricks for the player's side as declarer there), the level, and whether the contract made. The
//! coach values passing over an opponent from a fit to these rows.
//!
//! Output: one line per case, "<expected tricks> <level> <made 0/1>", to stdout.

use fh_ai::features::{bid_features, strain_block};
use fh_core::{Action, Call, Contract, Event, Game, Rules, Seat};

pub fn run(deals: u64, seed: u64, threads: u64) {
    let rows: Vec<(f64, u8, bool)> = std::thread::scope(|scope| {
        let handles: Vec<_> = (0..threads)
            .map(|t| {
                scope.spawn(move || {
                    let mut rows = Vec::new();
                    let mut d = t;
                    while d < deals {
                        rows.extend(one(seed + d));
                        d += threads;
                    }
                    rows
                })
            })
            .collect();
        handles.into_iter().flat_map(|h| h.join().unwrap()).collect()
    });
    for (tricks, level, made) in rows {
        println!("{tricks:.3} {level} {}", u8::from(made));
    }
}

fn one(seed: u64) -> Vec<(f64, u8, bool)> {
    let model = fh_ai::models::bid_model().expect("the bidding network");
    let mut game = Game::new(Rules::default(), seed);
    let mut bots: [Box<dyn fh_ai::Bot>; 4] = std::array::from_fn(|i| crate::make_bot("oracle", seed.wrapping_mul(4).wrapping_add(i as u64)));
    let mut pending: Vec<(Seat, Contract, f64)> = Vec::new();
    loop {
        let Some(seat) = game.to_act() else { return vec![] };
        let obs = game.observe(seat);
        let action = bots[seat.index()].choose(&obs, &game.legal_actions());
        if let (Action::Call(Call::Pass), Some((bidder, contract))) = (action, obs.auction.high_bid()) {
            if bidder.team() != seat.team() {
                let logits = model.forward(&bid_features(&obs));
                let block = strain_block(contract.strain);
                let z: Vec<f64> = logits[block..block + 11].iter().map(|&v| v as f64).collect();
                let top = z.iter().cloned().fold(f64::MIN, f64::max);
                let e: Vec<f64> = z.iter().map(|v| (v - top).exp()).collect();
                let sum: f64 = e.iter().sum();
                let expected = e.iter().enumerate().map(|(t, p)| t as f64 * p / sum).sum();
                pending.push((bidder, contract, expected));
            }
        }
        let events = game.apply(action).unwrap();
        if events.iter().any(|e| matches!(e, Event::PassedOut { .. })) {
            pending.clear();
        }
        if let Some(Event::HandScored { declarer, contract, tricks_won, .. }) = events.iter().find(|e| matches!(e, Event::HandScored { .. })) {
            let made = tricks_won[declarer.team().index()] >= contract.level;
            return pending
                .into_iter()
                .filter(|&(b, c, _)| b == *declarer && c == *contract)
                .map(|(_, c, t)| (t, c.level, made))
                .collect();
        }
    }
}
