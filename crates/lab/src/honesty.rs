//! `fh-lab honesty`: are the coach's percentages honest? Expert bots (the Oracle) play whole hands.
//! At sampled decisions the coach states its chance that the bid is made after the play actually
//! chosen (for card play) or for the bid actually made (for the winning bid); at the end of the
//! hand we record whether it was. Grouped by stated chance, the made rate should match.

use fh_ai::coach::Coach;
use fh_core::{Action, Call, Event, Game, Phase, Rng, Rules, Seat};

/// One prediction: the stated chance, whether the bid was made, and when it was made.
struct Prediction {
    stated: f64,
    made: bool,
    /// "bid", "early play" (tricks 1–3), "mid play" (4–7) or "late play" (8–10).
    stage: &'static str,
}

pub fn run(deals: u64, seed: u64, threads: u64) {
    let rows: Vec<Prediction> = std::thread::scope(|scope| {
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
    println!("{} predictions from {deals} deals\n", rows.len());
    for stage in ["bid", "early play", "mid play", "late play", "all play"] {
        let picked: Vec<&Prediction> = rows.iter().filter(|p| p.stage == stage || (stage == "all play" && p.stage != "bid")).collect();
        if picked.is_empty() {
            continue;
        }
        let stated = picked.iter().map(|p| p.stated).sum::<f64>() / picked.len() as f64;
        let made = picked.iter().filter(|p| p.made).count() as f64 / picked.len() as f64;
        println!("## {stage}: {} predictions; stated {:.0}% on average, made {:.0}%\n", picked.len(), 100.0 * stated, 100.0 * made);
        println!("| Coach said | n | Average stated | Actually made |");
        println!("|---|---|---|---|");
        for b in 0..10 {
            let (lo, hi) = (b as f64 / 10.0, (b + 1) as f64 / 10.0);
            let bucket: Vec<&&Prediction> = picked.iter().filter(|p| p.stated >= lo && (p.stated < hi || (b == 9 && p.stated <= 1.0))).collect();
            if bucket.len() < 15 {
                continue;
            }
            let s = bucket.iter().map(|p| p.stated).sum::<f64>() / bucket.len() as f64;
            let m = bucket.iter().filter(|p| p.made).count() as f64 / bucket.len() as f64;
            println!("| {:.0}–{:.0}% | {} | {:.0}% | {:.0}% |", 100.0 * lo, 100.0 * hi, bucket.len(), 100.0 * s, 100.0 * m);
        }
        println!();
    }
}

fn one(seed: u64) -> Vec<Prediction> {
    let mut game = Game::new(Rules::default(), seed);
    let mut bots: [Box<dyn fh_ai::Bot>; 4] = std::array::from_fn(|i| crate::make_bot("oracle", seed.wrapping_mul(4).wrapping_add(i as u64)));
    let mut rng = Rng::new(seed ^ 0x40E5);
    // Three play decisions per hand, at random tricks, plus the winning bid.
    let mut sample_at: Vec<usize> = (0..3).map(|_| rng.below(40)).collect();
    sample_at.sort_unstable();
    let mut plays_seen = 0;
    let mut pending: Vec<(f64, &'static str)> = Vec::new();
    let mut bid_pending: Option<(Seat, Action, f64)> = None;
    loop {
        let Some(seat) = game.to_act() else { return vec![] };
        let obs = game.observe(seat);
        let legal = game.legal_actions();
        let action = bots[seat.index()].choose(&obs, &legal);
        match obs.phase {
            Phase::Bidding if matches!(action, Action::Call(Call::Bid(_))) => {
                // The coach's chance for this bid; kept if it turns out to be the winning one.
                let analysis = Coach::new(seed ^ game.history().len() as u64).analyze(&obs, &legal, Some(action));
                let stated = analysis.options.iter().find(|o| o.action == action).and_then(|o| o.makes);
                if let Some(stated) = stated {
                    bid_pending = Some((seat, action, stated));
                }
            }
            Phase::Playing if legal.len() > 1 => {
                if sample_at.contains(&plays_seen) {
                    let analysis = Coach::new(seed ^ (plays_seen as u64) << 8).analyze(&obs, &legal, Some(action));
                    if let Some(stated) = analysis.options.iter().find(|o| o.action == action).and_then(|o| o.makes) {
                        let trick = obs.tricks.len() + 1;
                        pending.push((stated, if trick <= 3 { "early play" } else if trick <= 7 { "mid play" } else { "late play" }));
                    }
                }
                plays_seen += 1;
            }
            _ => {}
        }
        let events = game.apply(action).unwrap();
        if events.iter().any(|e| matches!(e, Event::PassedOut { .. })) {
            return vec![];
        }
        if let Some(Event::HandScored { declarer, contract, tricks_won, .. }) = events.iter().find(|e| matches!(e, Event::HandScored { .. })) {
            let made = tricks_won[declarer.team().index()] >= contract.level;
            let mut out: Vec<Prediction> = pending.into_iter().map(|(stated, stage)| Prediction { stated, made, stage }).collect();
            if let Some((seat, Action::Call(Call::Bid(bid)), stated)) = bid_pending {
                if seat == *declarer && bid == *contract {
                    out.push(Prediction { stated, made, stage: "bid" });
                }
            }
            return out;
        }
    }
}
