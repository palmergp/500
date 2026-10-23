//! `fh-lab thumbcheck`: does the coach agree with its own rule of thumb?
//!
//! Through Oracle auctions, wherever a player's count in a suit sits just past a threshold (7 for
//! a 7 bid, 8½ for 8, 10 for 9) and that bid is legal, grade the bid as if they made it. A player
//! who follows the rule shouldn't often hear "blunder".

use std::collections::BTreeMap;

use fh_ai::coach::Coach;
use fh_core::{Action, Call, Contract, Game, Phase, Rules, Strain};

pub fn run(deals: u64, seed: u64, threads: u64) {
    let parts: Vec<Vec<Row>> = std::thread::scope(|scope| {
        let handles: Vec<_> = (0..threads)
            .map(|t| {
                scope.spawn(move || {
                    let mut out = Vec::new();
                    let mut d = t;
                    while d < deals {
                        out.extend(one(seed + d));
                        d += threads;
                    }
                    out
                })
            })
            .collect();
        handles.into_iter().map(|h| h.join().unwrap()).collect()
    });
    let rows: Vec<_> = parts.into_iter().flatten().collect();
    let mut by: BTreeMap<(String, String), (u32, f64)> = BTreeMap::new();
    let mut verdicts: BTreeMap<String, BTreeMap<String, u32>> = BTreeMap::new();
    // Played out: the coach's choice minus the rule's bid, by verdict.
    let mut real: BTreeMap<String, (u32, f64, f64)> = BTreeMap::new();
    for r in &rows {
        let e = by.entry((r.case.clone(), r.situation.clone())).or_default();
        e.0 += 1;
        e.1 += r.makes;
        *verdicts.entry(format!("{} · {}", r.case, r.situation)).or_default().entry(r.verdict.clone()).or_default() += 1;
        if let Some(d) = r.coach_minus_rule {
            let e = real.entry(r.verdict.clone()).or_default();
            e.0 += 1;
            e.1 += d;
            e.2 += d * d;
        }
    }
    println!("Bids at the rule-of-thumb threshold, graded by the coach ({} cases, {deals} deals)\n", rows.len());
    println!("| Case | n | Coach's make chance | best | good | inaccuracy | mistake | blunder |");
    println!("|---|---|---|---|---|---|---|---|");
    for ((case, situation), (n, makes)) in &by {
        let v = &verdicts[&format!("{case} · {situation}")];
        let p = |k: &str| format!("{:.0}%", 100.0 * *v.get(k).unwrap_or(&0) as f64 / *n as f64);
        println!("| {case} · {situation} | {n} | {:.0}% | {} | {} | {} | {} | {} |", 100.0 * makes / *n as f64, p("best"), p("good"), p("inaccuracy"), p("mistake"), p("blunder"));
    }
    println!("
Played out both ways (solver, scored for real play): the coach's choice minus the rule's bid, by the verdict the rule's bid got
");
    println!("| Verdict on the rule's bid | n | Coach's choice gains (points, 95%) |");
    println!("|---|---|---|");
    for (verdict, (n, sum, sq)) in &real {
        let n = *n as f64;
        let mean = sum / n;
        let sd = ((sq / n - mean * mean) * n / (n - 1.0).max(1.0)).max(0.0).sqrt();
        println!("| {verdict} | {} | {mean:+.0} ± {:.0} |", n as u32, 1.96 * sd / n.sqrt());
    }
}

struct Row {
    case: String,
    verdict: String,
    makes: f64,
    situation: String,
    coach_minus_rule: Option<f64>,
}

fn one(seed: u64) -> Vec<Row> {
    let mut game = Game::new(Rules::default(), seed);
    let mut bots: [Box<dyn fh_ai::Bot>; 4] = std::array::from_fn(|i| crate::make_bot("oracle", seed.wrapping_mul(4).wrapping_add(i as u64)));
    let mut out = Vec::new();
    while game.phase() == Phase::Bidding && game.history().is_empty() {
        let Some(seat) = game.to_act() else { break };
        let obs = game.observe(seat);
        let legal = game.legal_actions();
        for strain in [Strain::Spades, Strain::Clubs, Strain::Diamonds, Strain::Hearts] {
            let count = fh_ai::thumb::count(&obs, strain).0;
            let (level, band) = match count {
                c if (7.0..7.5).contains(&c) => (7, "7–7½"),
                c if (8.5..9.0).contains(&c) => (8, "8½–9"),
                c if (10.0..10.5).contains(&c) => (9, "10–10½"),
                c if (10.5..11.0).contains(&c) => (9, "10½–11"),
                c if (11.0..12.0).contains(&c) => (9, "11–12"),
                c if c >= 12.0 => (9, "12+"),
                _ => continue,
            };
            let bid = Action::Call(Call::Bid(Contract::new(level, strain)));
            if !legal.contains(&bid) || obs.auction.must_bid() {
                continue;
            }
            let situation = match obs.auction.high_bid() {
                None => "no bid yet",
                Some((s, _)) if s == seat.partner() => "over partner's bid",
                Some(_) => "over an opponent's bid",
            };
            let grade = Coach::new(seed ^ level as u64).grade(&obs, &legal, bid);
            let makes = grade.analysis.options.iter().find(|o| o.action == bid).and_then(|o| o.makes).unwrap_or(0.0);
            // Only where this suit is clearly the player's best count, so the rule points at it.
            let others = [Strain::Spades, Strain::Clubs, Strain::Diamonds, Strain::Hearts].into_iter().filter(|&s| s != strain).map(|s| fh_ai::thumb::count(&obs, s).0).fold(f32::MIN, f32::max);
            if others > count - 0.5 {
                continue;
            }
            let coach_minus_rule = if grade.best != bid {
                let rule = crate::levels::finish(&game, bid, seat, seed, false);
                let coach = crate::levels::finish(&game, grade.best, seat, seed, false);
                rule.zip(coach).map(|(r, c)| c - r)
            } else {
                Some(0.0)
            };
            out.push(Row {
                case: format!("{level} at count {band}"),
                verdict: format!("{:?}", grade.verdict).to_lowercase(),
                makes,
                situation: situation.to_string(),
                coach_minus_rule,
            });
        }
        let action = bots[seat.index()].choose(&obs, &legal);
        if game.apply(action).is_err() {
            break;
        }
    }
    out
}
