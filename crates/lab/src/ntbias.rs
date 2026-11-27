//! `fh-lab ntbias`: how much does real play fall short of the solver in no trumps?
//!
//! Bids are valued from the tricks the solver takes with every card seen, less half a trick for
//! real play (`dd_bias`, finding 2: 120 deals, every strain together). `bidplay` (finding 63) found
//! the bidding network's no-trump bids, where the count bid a suit, 32 points worse at the table.
//! Here The Oracle bids as it does today, and at each call the same position goes to Oracles that
//! take more off no-trump bids (`dd_bias_nt`); where one calls differently, both calls are played
//! out (the rest of the auction and the play by today's Oracle in every seat, same seed).

use std::collections::BTreeMap;

use crate::playout::{finish, Mean};
use fh_ai::search_bot::{SearchBot, SearchConfig};
use fh_ai::Bot;
use fh_core::{Action, Call, Event, Game, Phase, Rules, Strain};

const GAPS: [f64; 3] = [0.75, 1.0, 1.25];

#[derive(Default, Clone)]
struct Tally {
    /// The variant's call less today's, played out, for the caller's side.
    real: Mean,
    tricks: Mean,
    made: [f64; 2],
    declared: [f64; 2],
}

pub fn run(deals: u64, seed: u64, threads: u64) {
    let parts: Vec<(BTreeMap<String, Tally>, u64)> = std::thread::scope(|scope| {
        let handles: Vec<_> = (0..threads)
            .map(|t| {
                scope.spawn(move || {
                    let mut out = BTreeMap::new();
                    let mut nt = 0;
                    let mut d = t;
                    while d < deals {
                        nt += deal(seed + d, &mut out);
                        d += threads;
                    }
                    (out, nt)
                })
            })
            .collect();
        handles.into_iter().map(|h| h.join().unwrap()).collect()
    });
    let mut all: BTreeMap<String, Tally> = BTreeMap::new();
    let mut nt = 0;
    for (part, n) in parts {
        nt += n;
        for (k, t) in part {
            let e = all.entry(k).or_default();
            e.real.merge(&t.real);
            e.tricks.merge(&t.tricks);
            for i in 0..2 {
                e.made[i] += t.made[i];
                e.declared[i] += t.declared[i];
            }
        }
    }
    println!("{deals} deals ({nt} won by a no-trump bid as today's Oracle bids). At each call, Oracles taking more off no-trump bids;");
    println!("where one calls differently, both calls played out by today's Oracle. Points for the caller's side, the variant less today.\n");
    println!("| No-trump gap | The variant | n | Points (95%) | Tricks | Bid made when declared: today / variant |");
    println!("|---|---|---|---|---|---|");
    for (k, t) in &all {
        let rate = |i: usize| if t.declared[i] > 0.0 { format!("{:.0}%", 100.0 * t.made[i] / t.declared[i]) } else { "—".into() };
        let (gap, what) = k.split_once(" | ").unwrap_or((k, ""));
        println!("| {gap} | {what} | {} | {} | {} | {} / {} |", t.real.n, t.real.show(1), t.tricks.show(2), rate(0), rate(1));
    }
}

fn describe(today: Call, variant: Call) -> &'static str {
    match (today, variant) {
        (Call::Bid(a), Call::Bid(b)) if a.strain == Strain::NoTrumps && b.strain != Strain::NoTrumps => "bids a suit instead of no trumps",
        (Call::Bid(a), Call::Bid(b)) if a.strain == b.strain && b.level < a.level => "bids lower",
        (Call::Bid(a), _) if a.strain == Strain::NoTrumps => "passes or signals instead of bidding no trumps",
        (Call::Bid(_), Call::Bid(_)) => "bids differently",
        _ => "other",
    }
}

/// One deal: 1 if today's Oracle ends in a no-trump bid.
fn deal(seed: u64, out: &mut BTreeMap<String, Tally>) -> u64 {
    let mut game = Game::new(Rules::default(), seed);
    let bot = |gap: f64, i: usize| SearchBot::new("The Oracle", SearchConfig { dd_bias_nt: gap, ..SearchConfig::ORACLE }, seed * 4 + i as u64);
    let mut today: [SearchBot; 4] = std::array::from_fn(|i| bot(SearchConfig::ORACLE.dd_bias_nt, i));
    let mut done = [false; GAPS.len()];
    // Play-outs already made: (call number, the call) -> outcome (the gaps often differ at the same call).
    let mut played: Vec<((usize, Action), Option<crate::playout::Outcome>)> = Vec::new();
    let mut call = 0;
    while game.phase() == Phase::Bidding {
        call += 1;
        let Some(seat) = game.to_act() else { return 0 };
        let obs = game.observe(seat);
        let legal = game.legal_actions();
        let action = today[seat.index()].choose(&obs, &legal);
        for (g, &gap) in GAPS.iter().enumerate() {
            if done[g] {
                continue;
            }
            let other = bot(gap, seat.index()).choose(&obs, &legal);
            if other == action {
                continue;
            }
            done[g] = true;
            let (Action::Call(a), Action::Call(b)) = (action, other) else { continue };
            let cont = seed.wrapping_add(23_000_011);
            let mut play = |a: Action| {
                if let Some((_, o)) = played.iter().find(|(k, _)| *k == (call, a)) {
                    return *o;
                }
                let mut g = game.clone();
                let o = g.apply(a).ok().and_then(|_| finish(g, cont, seat));
                played.push(((call, a), o));
                o
            };
            let (Some(x), Some(y)) = (play(action), play(other)) else { continue };
            for key in [format!("{gap} | {}", describe(a, b)), format!("{gap} | all")] {
                let t = out.entry(key).or_default();
                t.real.add(y.points - x.points);
                t.tricks.add(f64::from(y.tricks - x.tricks));
                for (i, o) in [x, y].iter().enumerate() {
                    if o.declared {
                        t.declared[i] += 1.0;
                        t.made[i] += f64::from(u8::from(o.made));
                    }
                }
            }
        }
        let Ok(events) = game.apply(action) else { return 0 };
        if events.iter().any(|e| matches!(e, Event::PassedOut { .. })) {
            return 0;
        }
    }
    u64::from(game.contract().is_some_and(|(_, c)| c.strain == Strain::NoTrumps))
}
