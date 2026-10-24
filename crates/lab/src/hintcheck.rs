//! `fh-lab hintcheck`: how often is a card-play hint wrong?
//!
//! Hands are played by The Counter in all four seats. In each, one of South's card plays (a random
//! trick with a real choice) gets the coach's hint, as the app shows it, and a much deeper look:
//! 256 deals sampled from what South can see (the same belief the coach uses). The deeper look
//! values every legal card; the hint's cost is how far its card falls short of the deeper look's
//! best, in expected points.

use std::collections::BTreeMap;

use fh_ai::coach::Coach;
use fh_ai::search_bot::{SearchBot, SearchConfig};
use fh_core::{Action, Game, Phase, Rng, Rules, Seat};

#[derive(Default, Clone, Copy)]
struct Tally {
    n: u32,
    loss: f64,
    over5: u32,
    over20: u32,
    same: u32,
}

pub fn run(deals: u64, seed: u64, threads: u64, reference: usize) {
    let parts: Vec<BTreeMap<String, Tally>> = std::thread::scope(|scope| {
        let handles: Vec<_> = (0..threads)
            .map(|t| {
                scope.spawn(move || {
                    let mut out = BTreeMap::new();
                    let mut d = t;
                    while d < deals {
                        one(seed + d, &mut out, reference);
                        d += threads;
                    }
                    out
                })
            })
            .collect();
        handles.into_iter().map(|h| h.join().unwrap()).collect()
    });
    let mut all: BTreeMap<String, Tally> = BTreeMap::new();
    for part in parts {
        for (k, t) in part {
            let e = all.entry(k).or_default();
            e.n += t.n;
            e.loss += t.loss;
            e.over5 += t.over5;
            e.over20 += t.over20;
            e.same += t.same;
        }
    }
    println!("Card-play hints against a {reference}-deal look ({deals} hands, one decision each)\n");
    println!("| Decisions | n | Same card as the deep look | Points the hint gives up | Off by 5+ | Off by 20+ |");
    println!("|---|---|---|---|---|---|");
    for (k, t) in &all {
        let n = t.n as f64;
        println!("| {k} | {} | {:.0}% | {:.1} | {:.0}% | {:.0}% |", t.n, 100.0 * t.same as f64 / n, t.loss / n, 100.0 * t.over5 as f64 / n, 100.0 * t.over20 as f64 / n);
    }
}

fn one(seed: u64, out: &mut BTreeMap<String, Tally>, reference: usize) {
    let mut game = Game::new(Rules::default(), seed);
    let mut bots: [Box<dyn fh_ai::Bot>; 4] = std::array::from_fn(|i| crate::make_bot("counter", seed.wrapping_mul(4).wrapping_add(i as u64)));
    let target_trick = Rng::new(seed ^ 0x41C).below(10);
    loop {
        if game.phase() == Phase::GameOver || !game.history().is_empty() {
            return;
        }
        let seat = game.to_act().unwrap();
        let obs = game.observe(seat);
        let legal = game.legal_actions();
        let trick_number = obs.tricks_won[0] + obs.tricks_won[1];
        if seat == Seat::SOUTH && obs.phase == Phase::Playing && legal.len() > 1 && trick_number as usize >= target_trick {
            let hint = Coach::new(seed).analyze(&obs, &legal, None);
            let mut deep = SearchBot::new("deep", SearchConfig::COUNTER, seed ^ 0xDEE9);
            let stats = deep.play_stats_sampled(&obs, &legal, reference, reference, u64::MAX);
            let ev = |a: Action| stats.stats.iter().find(|s| s.action == a).map(|s| s.ev);
            let best = stats.stats.iter().map(|s| s.ev).fold(f64::MIN, f64::max);
            let Some(mine) = ev(hint.best) else { return };
            let loss = (best - mine).max(0.0);
            let (declarer, _) = obs.contract.unwrap();
            let role = if declarer == Seat::SOUTH {
                "declarer"
            } else if declarer == Seat::NORTH {
                "declarer's partner"
            } else {
                "defender"
            };
            let leading = obs.current_trick.as_ref().is_some_and(|t| t.cards().is_empty());
            let when = match trick_number {
                0 => "trick 1",
                1..=3 => "tricks 2–4",
                4..=6 => "tricks 5–7",
                _ => "tricks 8–10",
            };
            for key in [
                "all".to_string(),
                format!("as {role}"),
                when.to_string(),
                (if leading { "leading" } else { "following" }).to_string(),
                format!("{} legal cards", if legal.len() >= 5 { "5+".to_string() } else { legal.len().to_string() }),
            ] {
                let e = out.entry(key).or_default();
                e.n += 1;
                e.loss += loss;
                e.over5 += u32::from(loss >= 5.0);
                e.over20 += u32::from(loss >= 20.0);
                e.same += u32::from(loss < 0.5);
            }
            return;
        }
        let action = bots[seat.index()].choose(&obs, &legal);
        game.apply(action).unwrap();
    }
}
