//! `fh-lab playdata`: training data for the neural card-play network.
//!
//! The Oracle plays whole hands in all four seats. At every card-play decision with a real choice
//! we record what the player could see (`features::play_features`) and the search's value for each
//! legal card, by slot (`features::play_slot`), NaN for cards that can't be played. The search then
//! plays its best card, so the positions are the kind an expert reaches.
//!
//! Output: `<out>.x` (f32 rows of PLAY_FEATURES), `<out>.y` (f32 rows of PLAY_SLOTS), `<out>.json`.

use fh_ai::features::{play_features, play_slot, PLAY_FEATURES, PLAY_SLOTS};
use fh_ai::{Bot, SearchBot, SearchConfig};
use fh_core::{Action, Event, Game, Phase, Rules};
use std::io::Write;
use std::time::Instant;

type Row = (Vec<f32>, Vec<f32>);

pub fn run(deals: u64, seed: u64, threads: u64, out: &str) {
    let start = Instant::now();
    let rows: Vec<Row> = std::thread::scope(|scope| {
        let handles: Vec<_> = (0..threads)
            .map(|t| {
                scope.spawn(move || {
                    let mut rows = Vec::new();
                    let mut d = t;
                    while d < deals {
                        rows.extend(one(seed.wrapping_add(d)));
                        d += threads;
                        if t == 0 && d % (threads * 200) == 0 {
                            eprintln!("  {d} deals, {:.0}s", start.elapsed().as_secs_f64());
                        }
                    }
                    rows
                })
            })
            .collect();
        handles.into_iter().flat_map(|h| h.join().unwrap()).collect()
    });
    let mut x = std::fs::File::create(format!("{out}.x")).expect("write .x");
    let mut y = std::fs::File::create(format!("{out}.y")).expect("write .y");
    for (features, values) in &rows {
        for v in features {
            x.write_all(&v.to_le_bytes()).unwrap();
        }
        for v in values {
            y.write_all(&v.to_le_bytes()).unwrap();
        }
    }
    let meta = format!(
        "{{\"rows\": {}, \"features\": {PLAY_FEATURES}, \"slots\": {PLAY_SLOTS}, \"deals\": {deals}, \"seed\": {seed}, \"label_meaning\": \"search value (points) of each legal card by slot; NaN where the card can't be played\"}}",
        rows.len()
    );
    std::fs::write(format!("{out}.json"), meta).expect("write .json");
    println!("{} decisions from {deals} deals in {:.0}s -> {out}.x/.y/.json", rows.len(), start.elapsed().as_secs_f64());
}

fn one(seed: u64) -> Vec<Row> {
    let mut game = Game::new(Rules::default(), seed);
    let mut bots: [SearchBot; 4] = std::array::from_fn(|i| SearchBot::new("teacher", SearchConfig::ORACLE, seed.wrapping_mul(4).wrapping_add(i as u64)));
    let mut rows = Vec::new();
    loop {
        let Some(seat) = game.to_act() else { return rows };
        let obs = game.observe(seat);
        let legal = game.legal_actions();
        let action = if obs.phase == Phase::Playing && legal.len() > 1 {
            let ranked = bots[seat.index()].evaluate_plays(&obs, &legal);
            let trump = obs.trump();
            let mut values = vec![f32::NAN; PLAY_SLOTS];
            for &(a, ev) in &ranked {
                if let Action::Play(p) = a {
                    let slot = play_slot(p.card, trump);
                    // A Joker lead in no trumps comes once per named suit: keep the best.
                    if values[slot].is_nan() || ev as f32 > values[slot] {
                        values[slot] = ev as f32;
                    }
                }
            }
            rows.push((play_features(&obs), values));
            ranked[0].0
        } else {
            bots[seat.index()].choose(&obs, &legal)
        };
        let events = game.apply(action).unwrap();
        if events.iter().any(|e| matches!(e, Event::HandScored { .. } | Event::GameWon { .. })) {
            return rows;
        }
    }
}
