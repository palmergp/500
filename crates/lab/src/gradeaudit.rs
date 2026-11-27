//! `fh-lab gradeaudit`: when the coach grades a card play a mistake, did it cost at the table?
//!
//! Steady and Rookie play South (so there are slips to grade) with expert bots in the other seats,
//! as in `gradestab`. Every South card play the coach doesn't call best is played out both ways, the
//! player's card and the coach's, by The Oracle in every seat with the same seed: the loss the
//! coach stated against the real one, by verdict and by kind of decision. A "mistake" that costs
//! nothing at the table teaches the wrong lesson; a "good" that costs a lot misses one.

use std::collections::BTreeMap;

use crate::coachaudit::kind;
use crate::playout::{finish, Mean};
use fh_ai::cardplay::is_boss;
use fh_ai::coach::{Coach, Verdict};
use fh_ai::rule_bot::{RuleBot, Style};
use fh_ai::search_bot::{SearchBot, SearchConfig};
use fh_ai::Bot;
use fh_core::rules::is_trump;
use fh_core::{effective_suit, Action, Card, Event, Game, Observation, Phase, Rules, Seat};

#[derive(Default, Clone)]
struct Tally {
    /// The loss the coach stated, and the real one (the coach's card's points less the player's).
    stated: Mean,
    real: Mean,
    real_tricks: Mean,
    /// Played out, the player's card did worse / better than the coach's.
    cost: f64,
    gained: f64,
}

struct Row {
    keys: Vec<String>,
    stated: f64,
    real: f64,
    real_tricks: f64,
}

pub fn run(deals: u64, seed: u64, threads: u64) {
    let parts: Vec<Vec<Row>> = std::thread::scope(|scope| {
        let handles: Vec<_> = (0..threads)
            .map(|t| {
                scope.spawn(move || {
                    let mut rows = Vec::new();
                    let mut d = t;
                    while d < deals {
                        deal(seed + d, &mut rows);
                        d += threads;
                    }
                    rows
                })
            })
            .collect();
        handles.into_iter().map(|h| h.join().unwrap()).collect()
    });
    let mut tallies: BTreeMap<String, Tally> = BTreeMap::new();
    for row in parts.into_iter().flatten() {
        for key in &row.keys {
            let t = tallies.entry(key.clone()).or_default();
            t.stated.add(row.stated);
            t.real.add(row.real);
            t.real_tricks.add(row.real_tricks);
            t.cost += f64::from(u8::from(row.real > 0.0));
            t.gained += f64::from(u8::from(row.real < 0.0));
        }
    }
    println!("{deals} deals, Steady or Rookie as South, expert bots elsewhere. Every South card play the coach didn't call");
    println!("best, played out both ways (the player's card and the coach's) by The Oracle in every seat, same seed both ways.");
    println!("Losses are the coach's card's points less the player's, for the player's side: as the coach stated, and real.\n");
    let sections: [(&str, &str); 8] = [
        ("verdict", "By verdict"),
        ("slip role", "Mistakes and blunders, by player"),
        ("slip trick", "Mistakes and blunders, by trick"),
        ("slip", "Mistakes and blunders, by decision (the player's card, the coach's)"),
        ("inaccuracy", "Inaccuracies, by decision"),
        ("good", "Graded good (not the coach's card), by decision"),
        ("discard", "Discards against discards, by verdict"),
        ("stated", "By the loss stated"),
    ];
    for (prefix, title) in sections {
        println!("### {title}\n");
        println!("| | n | Stated loss | Real loss (95%) | Real tricks lost | Cost / gained |");
        println!("|---|---|---|---|---|---|");
        let mut rows: Vec<(&String, &Tally)> = tallies.iter().filter(|(k, _)| k.starts_with(&format!("{prefix}:"))).collect();
        match prefix {
            "verdict" | "discard" | "stated" => rows.sort_by(|a, b| a.1.stated.mean().total_cmp(&b.1.stated.mean())),
            _ => rows.sort_by(|a, b| b.1.real.n.total_cmp(&a.1.real.n)),
        }
        for (key, t) in rows.into_iter().take(18) {
            let m = t.real.n.max(1.0);
            println!(
                "| {} | {} | {:.1} | {} | {} | {:.0}% / {:.0}% |",
                key.split_once(": ").map_or("", |(_, k)| k),
                t.real.n,
                t.stated.mean(),
                t.real.show(1),
                t.real_tricks.show(2),
                100.0 * t.cost / m,
                100.0 * t.gained / m
            );
        }
        println!();
    }
}

/// What a discard throws: a winner, a card guarding a higher one, a suit's last card, or a low card.
pub(crate) fn discard_kind(obs: &Observation, card: Card) -> &'static str {
    let trump = obs.trump();
    if is_trump(card, trump) {
        return "a trump";
    }
    let seen = obs.played().union(obs.hand).union(obs.discards.unwrap_or_default());
    if is_boss(card, seen, trump) {
        return "a winner";
    }
    let suit = effective_suit(card, trump);
    let same: Vec<Card> = obs.hand.iter().filter(|&c| effective_suit(c, trump) == suit).collect();
    if same.len() == 1 {
        return "a suit's last card";
    }
    let guards = same.iter().any(|&c| c.rank() > card.rank() && c.rank() >= fh_core::cards::QUEEN && !is_boss(c, seen, trump));
    if guards && same.len() <= 3 {
        "a guard"
    } else {
        "a low card"
    }
}

/// The kind of decision: the player's card and the coach's.
pub(crate) fn decision(obs: &Observation, chosen: Card, best: Card) -> String {
    let lead = obs.current_trick.as_ref().is_none_or(|t| t.cards().is_empty());
    let what = |c: Card| {
        let k = kind(obs, c);
        if k == "a discard" { format!("a discard of {}", discard_kind(obs, c)) } else { k.to_string() }
    };
    format!("{} {} instead of {}", if lead { "lead" } else { "follow" }, what(chosen), what(best))
}

fn deal(seed: u64, rows: &mut Vec<Row>) {
    let mut game = Game::new(Rules::default(), seed);
    let mut south: Box<dyn Bot> = if seed.is_multiple_of(2) {
        Box::new(RuleBot::new("Steady", Style::STEADY, seed))
    } else {
        Box::new(RuleBot::new("Rookie", Style::ROOKIE, seed))
    };
    let mut others: [SearchBot; 4] = std::array::from_fn(|i| SearchBot::new("expert", SearchConfig::FAST, seed * 4 + i as u64));
    // South's card plays with a choice: the game just before, and the card.
    let mut plays: Vec<(Game, Action)> = Vec::new();
    while let Some(seat) = game.to_act() {
        let obs = game.observe(seat);
        let legal = game.legal_actions();
        let action = if seat == Seat::SOUTH { south.choose(&obs, &legal) } else { others[seat.index()].choose(&obs, &legal) };
        if seat == Seat::SOUTH && legal.len() > 1 && game.phase() == Phase::Playing {
            plays.push((game.clone(), action));
        }
        let Ok(events) = game.apply(action) else { return };
        if events.iter().any(|e| matches!(e, Event::HandScored { .. } | Event::PassedOut { .. })) {
            break;
        }
    }
    for (i, (before, chosen)) in plays.into_iter().enumerate() {
        let obs = before.observe(Seat::SOUTH);
        let legal = before.legal_actions();
        let grade = Coach::new(seed.wrapping_mul(16).wrapping_add(i as u64)).grade(&obs, &legal, chosen);
        if grade.verdict == Verdict::Best || grade.best == chosen {
            continue;
        }
        let (Action::Play(c), Action::Play(b)) = (chosen, grade.best) else { continue };
        let cont = seed.wrapping_add(13_000_007 + i as u64);
        let out = |a: Action| {
            let mut g = before.clone();
            g.apply(a).ok()?;
            finish(g, cont, Seat::SOUTH)
        };
        let (Some(theirs), Some(coach)) = (out(chosen), out(grade.best)) else { continue };
        let Some((declarer, contract)) = before.contract() else { continue };
        let close = grade.analysis.reasons.iter().any(|r| r.to_lowercase().contains("close call"));
        let label = grade.verdict.label();
        let what = decision(&obs, c.card, b.card);
        let role = if declarer == Seat::SOUTH {
            "the declarer"
        } else if declarer.team() == Seat::SOUTH.team() {
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
        let bucket = match grade.loss {
            l if l < 10.0 => "under 10",
            l if l < 25.0 => "10–25",
            l if l < 60.0 => "25–60",
            l if l < 120.0 => "60–120",
            _ => "120 or more",
        };
        let mut keys = vec![
            format!("verdict: {label}"),
            format!("verdict: {label}, {}", if close { "called a close call" } else { "not called close" }),
            format!("stated: {bucket}"),
        ];
        let suit_bid = if contract.trump().is_some() { "" } else { " (no trumps)" };
        match grade.verdict {
            Verdict::Mistake | Verdict::Blunder => {
                keys.push(format!("slip role: {role}"));
                keys.push(format!("slip trick: {band}"));
                keys.push(format!("slip: {what}"));
                if !suit_bid.is_empty() {
                    keys.push(format!("slip: {what}{suit_bid}"));
                }
            }
            Verdict::Inaccuracy => keys.push(format!("inaccuracy: {what}")),
            _ => keys.push(format!("good: {what}")),
        }
        if what.starts_with("follow a discard") && what.contains("instead of a discard") {
            keys.push(format!("discard: {label}"));
            keys.push(format!("discard: {label}, {role}"));
        }
        rows.push(Row { keys, stated: grade.loss, real: coach.points - theirs.points, real_tricks: f64::from(coach.tricks - theirs.tricks) });
    }
}
