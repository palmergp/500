//! `fh-lab switch`: is the extra bar for leaving partner's strain worth it?
//!
//! The Oracle used to bid a different strain over partner's bid only when it expected 40 points more than
//! it needed to bid at all (a rule from the first search bidder, finding 4; removed after finding 28). For every auction where
//! that rule alone turns a bid into a pass, play the deal out both ways (the Oracle finishes the
//! auction, the hand is solved and scored for real play, as in `fh-lab levels`) and compare.

use fh_ai::search_bot::{SearchBot, SearchConfig};
use fh_core::{Action, Call, Game, Phase, Rules};

pub fn run(deals: u64, seed: u64, threads: u64) {
    let rows: Vec<(f64, f64, f64)> = std::thread::scope(|scope| {
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
    println!("Auctions where the switching bar alone turned a bid into a pass: {} in {deals} deals\n", rows.len());
    let report = |title: &str, picked: Vec<&(f64, f64, f64)>| {
        let n = picked.len() as f64;
        if n < 2.0 {
            return;
        }
        let diffs: Vec<f64> = picked.iter().map(|r| r.0).collect();
        let mean = diffs.iter().sum::<f64>() / n;
        let sd = (diffs.iter().map(|d| (d - mean).powi(2)).sum::<f64>() / (n - 1.0)).sqrt();
        let gain: f64 = picked.iter().map(|r| r.2).sum::<f64>() / n;
        println!("| {title} | {} | {:+.0} ± {:.0} | {:+.0} |", picked.len(), mean, 1.96 * sd / n.sqrt(), gain);
    };
    println!("| Cases | n | Bidding minus passing, real points (95%) | The network's expected edge |");
    println!("|---|---|---|---|");
    report("All", rows.iter().collect());
    for (lo, hi) in [(0.0f64, 0.7f64), (0.7, 0.85), (0.85, 1.01)] {
        report(&format!("Bid makes {:.0}–{:.0}% (network)", 100.0 * lo, 100.0 * hi.min(1.0)), rows.iter().filter(|r| r.1 >= lo && r.1 < hi).collect());
    }
}

/// (bid minus pass in points, the bid's make chance, the network's expected edge) per case.
fn one(seed: u64) -> Vec<(f64, f64, f64)> {
    let model = fh_ai::models::bid_model().expect("the bidding network");
    let judge = SearchBot::new("judge", SearchConfig::ORACLE, seed);
    let margin = SearchConfig::ORACLE.bid_margin;
    let mut game = Game::new(Rules::default(), seed);
    let mut bots: [Box<dyn fh_ai::Bot>; 4] = std::array::from_fn(|i| crate::make_bot("oracle", seed.wrapping_mul(4).wrapping_add(i as u64)));
    let mut out = Vec::new();
    while game.phase() == Phase::Bidding && game.history().is_empty() {
        let Some(seat) = game.to_act() else { break };
        let obs = game.observe(seat);
        let legal = game.legal_actions();
        if let Some((_, partner_bid)) = obs.auction.high_bid().filter(|(s, _)| *s == seat.partner()) {
            let (bids, baseline) = judge.neural_values(&obs, &legal, model);
            if let Some(&(contract, ev, makes)) = bids.iter().max_by(|a, b| a.1.total_cmp(&b.1)) {
                let edge = ev - baseline;
                if contract.strain != partner_bid.strain && edge > margin && edge <= margin + 40.0 && !obs.auction.must_bid() {
                    let pass = crate::levels::finish(&game, Action::Call(Call::Pass), seat, seed, false);
                    let bid = crate::levels::finish(&game, Action::Call(Call::Bid(contract)), seat, seed, false);
                    if let (Some(p), Some(b)) = (pass, bid) {
                        out.push((b - p, makes, edge));
                    }
                }
            }
        }
        let action = bots[seat.index()].choose(&obs, &legal);
        if game.apply(action).is_err() {
            break;
        }
    }
    out
}
