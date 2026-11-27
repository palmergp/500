//! `fh-lab kittystab`: how many deals does a kitty grade need to be steady?
//!
//! `gradestab` found kitty grades the least steady of the coach's: graded three times by
//! differently seeded coaches, a kitty one card off the plan got the same verdict only 44% of the
//! time, and good on one look but a mistake on another 15% of the time. Here each such kitty is
//! graded three times with each way of choosing the deals (today's first look of 12 and closer
//! look of 60, and larger ones), and once on 400 deals as the reference.

use crate::playout::Mean;
use fh_ai::coach::{Coach, Verdict};
use fh_ai::rule_bot::heuristic_discards;
use fh_ai::search_bot::{SearchBot, SearchConfig};
use fh_ai::Bot;
use fh_core::{Card, CardSet, Event, Game, Phase, Rng, Rules};

/// (name, first look, closer look)
const WAYS: [(&str, usize, usize); 4] = [
    ("12, then 60 (before finding 66)", 12, 60),
    ("12, then 120", 12, 120),
    ("24, then 120 (today)", 24, 120),
    ("12, then 180", 12, 180),
];
const REFERENCE: usize = 400;

#[derive(Default, Clone)]
struct Tally {
    n: f64,
    same: f64,
    across: f64,
    spread: Mean,
    /// The three grades' verdict class (good or better, inaccuracy, mistake or worse) matched the
    /// reference's.
    agree: f64,
    /// Called good when the reference says a mistake, and the reverse.
    missed: f64,
    false_alarm: f64,
    ms: Mean,
}

fn class(v: Verdict) -> u8 {
    match v {
        Verdict::Best | Verdict::Good => 0,
        Verdict::Inaccuracy => 1,
        _ => 2,
    }
}

pub fn run(deals: u64, seed: u64, threads: u64) {
    let parts: Vec<(Vec<Tally>, Mean)> = std::thread::scope(|scope| {
        let handles: Vec<_> = (0..threads)
            .map(|t| {
                scope.spawn(move || {
                    let mut out = vec![Tally::default(); WAYS.len()];
                    let mut reference_ms = Mean::default();
                    let mut d = t;
                    while d < deals {
                        deal(seed + d, &mut out, &mut reference_ms);
                        d += threads;
                    }
                    (out, reference_ms)
                })
            })
            .collect();
        handles.into_iter().map(|h| h.join().unwrap()).collect()
    });
    let mut all = vec![Tally::default(); WAYS.len()];
    let mut reference_ms = Mean::default();
    for (part, r) in parts {
        reference_ms.merge(&r);
        for (a, t) in all.iter_mut().zip(part) {
            a.n += t.n;
            a.same += t.same;
            a.across += t.across;
            a.spread.merge(&t.spread);
            a.agree += t.agree;
            a.missed += t.missed;
            a.false_alarm += t.false_alarm;
            a.ms.merge(&t.ms);
        }
    }
    let n = all[0].n.max(1.0);
    println!("{deals} deals; {} kitties one card off the coach's plan, each graded three times by differently seeded coaches", all[0].n);
    println!("for each way of choosing the deals, and once on {REFERENCE} deals as the reference ({:.0} ms a grade).\n", reference_ms.mean());
    println!("| Deals | Same verdict all three times | Good on one, a mistake on another | Spread of the loss | Matches the reference (of 3 grades) | Good where the reference says mistake | Mistake where it says good | Time a grade |");
    println!("|---|---|---|---|---|---|---|---|");
    for ((name, _, _), t) in WAYS.iter().zip(&all) {
        println!(
            "| {name} | {:.0}% | {:.1}% | {:.1} | {:.0}% | {:.1}% | {:.1}% | {:.0} ms |",
            100.0 * t.same / n,
            100.0 * t.across / n,
            t.spread.mean(),
            100.0 * t.agree / (3.0 * n),
            100.0 * t.missed / (3.0 * n),
            100.0 * t.false_alarm / (3.0 * n),
            t.ms.mean()
        );
    }
}

fn deal(seed: u64, out: &mut [Tally], reference_ms: &mut Mean) {
    let mut game = Game::new(Rules::default(), seed);
    let mut bots: [SearchBot; 4] = std::array::from_fn(|i| SearchBot::new("expert", SearchConfig::FAST, seed * 4 + i as u64));
    while game.phase() == Phase::Bidding {
        let Some(seat) = game.to_act() else { return };
        let action = bots[seat.index()].choose(&game.observe(seat), &game.legal_actions());
        let Ok(events) = game.apply(action) else { return };
        if events.iter().any(|e| matches!(e, Event::PassedOut { .. })) {
            return;
        }
    }
    if game.phase() != Phase::Discarding {
        return;
    }
    let Some(seat) = game.to_act() else { return };
    let obs = game.observe(seat);
    // The plan with one card changed, as `gradestab` grades.
    let mut rng = Rng::new(seed ^ 0x57ab);
    let plan = heuristic_discards(obs.hand, obs.trump(), 3);
    let kept: Vec<Card> = obs.hand.minus(plan.iter().copied().collect::<CardSet>()).iter().collect();
    let mut chosen = plan.clone();
    chosen[rng.below(3)] = kept[rng.below(kept.len())];

    let timed = |coach: Coach| {
        let mut coach = coach;
        let start = std::time::Instant::now();
        let g = coach.grade_discard_set(&obs, &chosen);
        (g, start.elapsed().as_secs_f64() * 1000.0)
    };
    let (reference, ms) = timed(Coach::new(seed ^ 0xfeed_beef).with_kitty_deals(0, REFERENCE));
    reference_ms.add(ms);
    let truth = class(reference.verdict);
    for (w, &(_, first, closer)) in WAYS.iter().enumerate() {
        let grades: Vec<_> = (0..3u64).map(|k| timed(Coach::new(seed ^ (k << 32) ^ 0x9e37).with_kitty_deals(first, closer))).collect();
        let t = &mut out[w];
        t.n += 1.0;
        let verdicts: Vec<Verdict> = grades.iter().map(|(g, _)| g.verdict).collect();
        t.same += f64::from(u8::from(verdicts.iter().all(|&v| v == verdicts[0])));
        let good = verdicts.iter().any(|&v| v <= Verdict::Good);
        let slip = verdicts.iter().any(|&v| v >= Verdict::Mistake);
        t.across += f64::from(u8::from(good && slip));
        let (lo, hi) = grades.iter().fold((f64::MAX, f64::MIN), |(a, b), (g, _)| (a.min(g.loss), b.max(g.loss)));
        t.spread.add(hi - lo);
        for (g, ms) in &grades {
            let c = class(g.verdict);
            t.agree += f64::from(u8::from(c == truth));
            t.missed += f64::from(u8::from(c == 0 && truth == 2));
            t.false_alarm += f64::from(u8::from(c == 2 && truth == 0));
            t.ms.add(*ms);
        }
    }
}
