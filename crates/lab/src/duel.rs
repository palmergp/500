//! `fh-lab duel`: duplicate comparison of two bots.
//!
//! Each deal is played twice from the same cards and dealer: once with bot A as North–South and
//! once with A as East–West. Luck of the cards cancels, so the paired difference in points
//! measures skill. A "deal" runs until the first hand that is actually played (redeals included).

use crate::make_bot;
use fh_core::{Event, Game, Phase, Rules, Seat, Team};
use std::time::Instant;

pub static VERBOSE: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

pub struct Options {
    pub a: String,
    pub b: String,
    pub deals: u64,
    pub seed: u64,
    pub threads: u64,
}

#[derive(Clone, Copy, Default)]
struct Side {
    declared: u64,
    made: u64,
    points: i64,
}

struct HandResult {
    /// A's points minus B's points.
    diff: i64,
    a: Side,
    b: Side,
}

/// Plays one hand of `seed` with `ns` as North–South and `ew` as East–West.
fn play_hand(seed: u64, ns: &str, ew: &str, a_is_ns: bool) -> HandResult {
    let dealer = Seat::new((seed % 4) as usize);
    let mut game = Game::with_dealer(Rules::default(), seed, dealer);
    let mut bots: [Box<dyn fh_ai::Bot>; 4] =
        std::array::from_fn(|i| make_bot(if i % 2 == 0 { ns } else { ew }, seed.wrapping_mul(4) + i as u64));
    loop {
        let seat = game.to_act().expect("hand in progress");
        let legal = game.legal_actions();
        let action = bots[seat.index()].choose(&game.observe(seat), &legal);
        if VERBOSE.load(std::sync::atomic::Ordering::Relaxed) {
            if let (fh_core::Action::Call(call), Some(note)) = (action, bots[seat.index()].note()) {
                eprintln!("deal {seed} {seat} {}: {call}  [{note}]", bots[seat.index()].name());
            }
        }
        let events = game.apply(action).expect("bots play legally");
        if events.iter().any(|e| matches!(e, Event::HandScored { .. })) || game.phase() == Phase::GameOver {
            break;
        }
    }
    let hand = game.history().iter().rev().find(|h| h.contract.is_some()).expect("a scored hand");
    let (declarer, contract) = hand.contract.unwrap();
    if VERBOSE.load(std::sync::atomic::Ordering::Relaxed) {
        eprintln!("deal {seed}: {declarer} played {contract}, took {} tricks", hand.tricks_won[declarer.team().index()]);
    }
    let a_team = if a_is_ns { Team::NorthSouth } else { Team::EastWest };
    let (a_i, b_i) = (a_team.index(), a_team.other().index());
    let mut result = HandResult { diff: (hand.score_change[a_i] - hand.score_change[b_i]) as i64, a: Side::default(), b: Side::default() };
    let side = if declarer.team() == a_team { &mut result.a } else { &mut result.b };
    side.declared += 1;
    if hand.tricks_won[declarer.team().index()] >= contract.level {
        side.made += 1;
    }
    result.a.points += hand.score_change[a_i] as i64;
    result.b.points += hand.score_change[b_i] as i64;
    result
}

pub fn run(opts: Options) {
    let start = Instant::now();
    let threads = opts.threads.clamp(1, 64);
    let results: Vec<(i64, Side, Side)> = std::thread::scope(|scope| {
        let handles: Vec<_> = (0..threads)
            .map(|t| {
                let opts = &opts;
                scope.spawn(move || {
                    let mut out = Vec::new();
                    let mut d = t;
                    while d < opts.deals {
                        let seed = opts.seed.wrapping_add(d);
                        let first = play_hand(seed, &opts.a, &opts.b, true);
                        let second = play_hand(seed, &opts.b, &opts.a, false);
                        let add = |x: Side, y: Side| Side { declared: x.declared + y.declared, made: x.made + y.made, points: x.points + y.points };
                        out.push((first.diff + second.diff, add(first.a, second.a), add(first.b, second.b)));
                        d += threads;
                    }
                    out
                })
            })
            .collect();
        handles.into_iter().flat_map(|h| h.join().expect("duel thread panicked")).collect()
    });

    let n = results.len() as f64;
    // Per deal: A's advantage in points per hand (average of the two seatings).
    let per_deal: Vec<f64> = results.iter().map(|r| r.0 as f64 / 2.0).collect();
    let mean = per_deal.iter().sum::<f64>() / n;
    let var = per_deal.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / (n - 1.0).max(1.0);
    let ci = 1.96 * (var / n).sqrt();
    let wins = per_deal.iter().filter(|&&x| x > 0.0).count();
    let losses = per_deal.iter().filter(|&&x| x < 0.0).count();
    let total = |f: fn(&(i64, Side, Side)) -> Side| {
        results.iter().map(f).fold(Side::default(), |a, b| Side { declared: a.declared + b.declared, made: a.made + b.made, points: a.points + b.points })
    };
    let (a, b) = (total(|r| r.1), total(|r| r.2));
    let secs = start.elapsed().as_secs_f64();

    println!("Duplicate: {} vs {} over {} deals (each played both ways)  [{secs:.0}s, {:.2} s per hand]", opts.a, opts.b, results.len(), secs * threads as f64 / (2.0 * n));
    println!();
    println!("{} gains {mean:+.1} points per hand over {} (95% CI {:+.1} to {:+.1}).", opts.a, opts.b, mean - ci, mean + ci);
    println!("Deals won/lost/tied by {}: {wins}/{losses}/{}", opts.a, results.len() - wins - losses);
    for (name, side) in [(&opts.a, a), (&opts.b, b)] {
        let hands = 2.0 * n;
        println!(
            "  {name:<14} declared {:.0}% of hands, made {:.0}% of those, {:+.0} points per hand",
            100.0 * side.declared as f64 / hands,
            100.0 * side.made as f64 / side.declared.max(1) as f64,
            side.points as f64 / hands
        );
    }
}
