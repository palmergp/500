//! `fh-lab leadread`: should the coach read the auction when it chooses a lead?
//!
//! The card-play audit (`coachaudit`) found a coach that reads the auction into its deals no closer
//! to the play-outs overall, but where it preferred another lead, that lead did better at the table
//! (about 17 points, from 238 leads: a hint, not proof). Here every lead of a hand played by expert
//! bots goes to both coaches; where they choose cards that play differently, both are played out by
//! The Oracle in every seat, the same seed both ways.

use std::collections::BTreeMap;

use crate::coachaudit::kind;
use crate::playout::{finish, Mean};
use fh_ai::coach::Coach;
use fh_ai::search_bot::{SearchBot, SearchConfig};
use fh_ai::Bot;
use fh_core::{Action, Call, Event, Game, Phase, Rules};

/// One lead where the coaches differed: its categories, and the real gap, real tricks, the plain
/// coach's gap and the reading coach's.
type Row = (Vec<String>, [f64; 4]);

#[derive(Default, Clone)]
struct Tally {
    /// The reading coach's lead's points less the plain coach's, played out.
    real: Mean,
    real_tricks: Mean,
    /// How far apart each coach thought the two leads were (its own choice ahead).
    plain_gap: Mean,
    reading_gap: Mean,
    better: f64,
    worse: f64,
}

pub fn run(deals: u64, seed: u64, threads: u64) {
    let parts: Vec<(Vec<Row>, u64, u64)> = std::thread::scope(|scope| {
        let handles: Vec<_> = (0..threads)
            .map(|t| {
                scope.spawn(move || {
                    let mut rows = Vec::new();
                    let (mut leads, mut alike) = (0, 0);
                    let mut d = t;
                    while d < deals {
                        deal(seed + d, &mut rows, &mut leads, &mut alike);
                        d += threads;
                    }
                    (rows, leads, alike)
                })
            })
            .collect();
        handles.into_iter().map(|h| h.join().unwrap()).collect()
    });
    let mut tallies: BTreeMap<String, Tally> = BTreeMap::new();
    let (mut leads, mut alike, mut differ) = (0, 0, 0);
    for (rows, l, a) in parts {
        leads += l;
        alike += a;
        for (keys, [real, tricks, plain, reading]) in rows {
            differ += 1;
            for key in keys {
                let t = tallies.entry(key).or_default();
                t.real.add(real);
                t.real_tricks.add(tricks);
                t.plain_gap.add(plain);
                t.reading_gap.add(reading);
                t.better += f64::from(u8::from(real > 0.0));
                t.worse += f64::from(u8::from(real < 0.0));
            }
        }
    }
    println!("{deals} deals, {leads} leads with a choice. The coach as it is and a coach reading the auction chose leads that");
    println!("play differently {differ} times ({alike} more differed only between cards each valued alike). Both played out by The Oracle,");
    println!("same seed both ways; gaps are the reading coach's lead less the plain coach's, for the leader's side.\n");
    let sections: [(&str, &str); 6] = [
        ("all", "All"),
        ("role", "By leader"),
        ("trick", "By trick"),
        ("auction", "By what the leader's partner called"),
        ("bid", "By bid"),
        ("pair", "The reading coach's lead over the plain coach's"),
    ];
    for (prefix, title) in sections {
        println!("### {title}\n");
        println!("| | n | Real gap (95%) | Real tricks | Reading's lead better / worse | Plain coach's gap / reading coach's |");
        println!("|---|---|---|---|---|---|");
        let mut rows: Vec<(&String, &Tally)> = tallies.iter().filter(|(k, _)| k.starts_with(&format!("{prefix}:"))).collect();
        rows.sort_by(|a, b| b.1.real.n.total_cmp(&a.1.real.n));
        for (key, t) in rows.into_iter().take(14) {
            let m = t.real.n.max(1.0);
            println!(
                "| {} | {} | {} | {} | {:.0}% / {:.0}% | {:+.1} / {:+.1} |",
                key.split_once(": ").map_or("", |(_, k)| k),
                t.real.n,
                t.real.show(1),
                t.real_tricks.show(2),
                100.0 * t.better / m,
                100.0 * t.worse / m,
                t.plain_gap.mean(),
                t.reading_gap.mean()
            );
        }
        println!();
    }
}

fn deal(seed: u64, rows: &mut Vec<Row>, leads: &mut u64, alike: &mut u64) {
    let mut game = Game::new(Rules::default(), seed);
    let mut bots: [SearchBot; 4] = std::array::from_fn(|i| SearchBot::new("expert", SearchConfig::FAST, seed * 4 + i as u64));
    let mut i = 0u64;
    while let Some(seat) = game.to_act() {
        let obs = game.observe(seat);
        let legal = game.legal_actions();
        let leading = obs.current_trick.as_ref().is_none_or(|t| t.cards().is_empty());
        if game.phase() == Phase::Playing && leading && legal.len() > 1 {
            *leads += 1;
            i += 1;
            let coach_seed = seed.wrapping_mul(16).wrapping_add(i);
            let plain = Coach::new(coach_seed).analyze(&obs, &legal, None);
            let reading = Coach::with_inference(coach_seed, true).analyze(&obs, &legal, None);
            if plain.best != reading.best {
                let (Some(p_own), Some(p_other), Some(r_own), Some(r_other)) =
                    (plain.value(plain.best), plain.value(reading.best), reading.value(reading.best), reading.value(plain.best))
                else {
                    return;
                };
                let (plain_gap, reading_gap) = (p_own - p_other, r_own - r_other);
                if plain_gap < 2.0 && reading_gap < 2.0 {
                    *alike += 1;
                } else {
                    let cont = seed.wrapping_add(17_000_009).wrapping_add(i);
                    let out = |a: Action| {
                        let mut g = game.clone();
                        g.apply(a).ok()?;
                        finish(g, cont, seat)
                    };
                    let (Some(r), Some(p)) = (out(reading.best), out(plain.best)) else { return };
                    let Some((declarer, contract)) = game.contract() else { return };
                    let role = if seat == declarer {
                        "the declarer"
                    } else if seat.team() == declarer.team() {
                        "the declarer's partner"
                    } else {
                        "a defender"
                    };
                    let band = match obs.tricks.len() {
                        0 => "trick 1",
                        1..=2 => "tricks 2–3",
                        3..=5 => "tricks 4–6",
                        _ => "tricks 7–10",
                    };
                    let partner = seat.partner();
                    let partner_call = obs.auction.calls().filter(|&(s, _)| s == partner).map(|(_, c)| c).fold("passed", |acc, c| match c {
                        Call::Bid(_) => "bid",
                        Call::Mention(_) | Call::Six(_) if acc == "passed" => "mentioned or called six",
                        _ => acc,
                    });
                    let card = |a: Action| match a {
                        Action::Play(p) => p.card,
                        _ => unreachable!("card play"),
                    };
                    let keys = vec![
                        "all: all".to_string(),
                        format!("role: {role}"),
                        format!("trick: {band}"),
                        format!("auction: partner {partner_call}"),
                        format!("auction: {role}, partner {partner_call}"),
                        format!("bid: {}", if contract.trump().is_some() { "a suit bid" } else { "no trumps" }),
                        format!("pair: {} over {}", kind(&obs, card(reading.best)), kind(&obs, card(plain.best))),
                    ];
                    rows.push((keys, [r.points - p.points, f64::from(r.tricks - p.tricks), plain_gap, reading_gap]));
                }
            }
        }
        let action = bots[seat.index()].choose(&obs, &legal);
        let Ok(events) = game.apply(action) else { return };
        if events.iter().any(|e| matches!(e, Event::HandScored { .. } | Event::PassedOut { .. })) {
            return;
        }
    }
}
