//! `fh-lab bidbar`: the coach recommends some bids it gives well under 70%. Do they pay?
//!
//! The Oracle bids in every seat (the coach's bidding recommendation is the Oracle's). At each call
//! where it bids (and isn't forced to), the deal is played out both ways, bidding and passing: the
//! Oracle finishes the auction, the hand is solved and scored for real play (as in `fh-lab switch`).
//! Cases are split by the chance the coach shows for the bid and by what passing would leave.

use std::collections::BTreeMap;

use fh_ai::coach::{real_play_chance, BID_CURVE};
use fh_ai::search_bot::{SearchBot, SearchConfig};
use fh_core::{Action, Call, Game, Phase, Rules};

type Rows = Vec<(String, f64, f64)>;

pub fn run(deals: u64, seed: u64, threads: u64) {
    let rows: Rows = std::thread::scope(|scope| {
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
    println!("The Oracle's bids over {deals} deals, each played out both ways (bid, or pass and let the auction go on): {}\n", rows.len());
    println!("| Passing would leave | Coach shows | n | Bidding minus passing, points (95%) | Average shown |");
    println!("|---|---|---|---|---|");
    let mut groups: BTreeMap<(String, usize), Vec<(f64, f64)>> = BTreeMap::new();
    let bands = [0.0, 0.55, 0.6, 0.65, 0.7, 0.8, 1.01];
    let label = |i: usize| if i == bands.len() - 2 { "80% or more".to_string() } else if i == 0 { "under 55%".to_string() } else { format!("{:.0}–{:.0}%", 100.0 * bands[i], 100.0 * bands[i + 1]) };
    for (situation, shown, diff) in &rows {
        let band = (0..bands.len() - 1).find(|&i| *shown < bands[i + 1]).unwrap();
        for key in [situation.clone(), "any".to_string()] {
            groups.entry((key, band)).or_default().push((*diff, *shown));
        }
    }
    for ((situation, band), v) in &groups {
        let n = v.len() as f64;
        if v.len() < 5 {
            continue;
        }
        let mean = v.iter().map(|r| r.0).sum::<f64>() / n;
        let sd = (v.iter().map(|r| (r.0 - mean).powi(2)).sum::<f64>() / (n - 1.0)).sqrt();
        let shown = v.iter().map(|r| r.1).sum::<f64>() / n;
        println!("| {situation} | {} | {} | {:+.0} ± {:.0} | {:.0}% |", label(*band), v.len(), mean, 1.96 * sd / n.sqrt(), 100.0 * shown);
    }
}

/// (what passing leaves, the chance the coach shows, bid minus pass in points) per Oracle bid.
fn one(seed: u64) -> Rows {
    let model = fh_ai::models::bid_model().expect("the bidding network");
    let judge = SearchBot::new("judge", SearchConfig::ORACLE, seed);
    let mut game = Game::new(Rules::default(), seed);
    let mut bots: [Box<dyn fh_ai::Bot>; 4] = std::array::from_fn(|i| crate::make_bot("oracle", seed.wrapping_mul(4).wrapping_add(i as u64)));
    let mut out = Vec::new();
    while game.phase() == Phase::Bidding && game.history().is_empty() {
        let Some(seat) = game.to_act() else { break };
        let obs = game.observe(seat);
        let legal = game.legal_actions();
        let action = bots[seat.index()].choose(&obs, &legal);
        if let Action::Call(Call::Bid(contract)) = action {
            if !obs.auction.must_bid() {
                let (bids, _) = judge.neural_values(&obs, &legal, model);
                let makes = bids.iter().find(|b| b.0 == contract).map_or(0.0, |b| b.2);
                let situation = match obs.auction.high_bid() {
                    None => "no bid yet",
                    Some((s, _)) if s == seat.partner() => "partner's bid",
                    Some(_) => "an opponent's bid",
                };
                let pass = crate::levels::finish(&game, Action::Call(Call::Pass), seat, seed, false);
                let bid = crate::levels::finish(&game, action, seat, seed, false);
                if let (Some(p), Some(b)) = (pass, bid) {
                    out.push((situation.to_string(), real_play_chance(makes, BID_CURVE), b - p));
                }
            }
        }
        if game.apply(action).is_err() {
            break;
        }
    }
    out
}
