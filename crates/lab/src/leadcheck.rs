//! `fh-lab leadcheck`: two questions about the bots' leads, from hands of Oracle play in all four
//! seats, each lead compared by the solver (every card visible) with the alternatives.
//!
//! - Suit bids: the declaring side holds the top trump left, and the opponents still hold trumps.
//!   Why lead a side ace or king instead of the top trump? What each kind of lead costs against
//!   the top trump, and how often a side ace or king led there won, was trumped, or was beaten.
//! - No trumps: whenever the player on lead holds the Joker (with other cards left), does leading
//!   it (naming the best suit) beat the best other lead, and how often does the bot lead it?

use std::collections::BTreeMap;

use fh_ai::solver::{Position, Solver};
use fh_core::cards::{ACE, KING};
use fh_core::rules::is_trump;
use fh_core::trick::trick_strength;
use fh_core::{Card, CardSet, HandRecord, Seat, Trick};

#[derive(Default, Clone, Copy)]
struct Suit {
    n: u32,
    won: u32,
    ruffed: u32,
    beaten: u32,
    /// Solver tricks for the leader's side: this lead minus leading the top trump.
    vs_top: f64,
    /// At least as good as the top trump.
    as_good: u32,
}

#[derive(Default, Clone, Copy)]
struct Nt {
    n: u32,
    led: u32,
    /// Best Joker lead minus the best other lead (solver tricks for the leader's side).
    joker_edge: f64,
    joker_better: u32,
    joker_worse: u32,
    /// What the bot's own choice cost against the best lead, when it led the Joker / held it.
    cost_led: f64,
    cost_held: f64,
}

type Out = (BTreeMap<(String, String), Suit>, BTreeMap<(String, String), Nt>);

pub fn run(deals: u64, seed: u64, threads: u64) {
    let parts: Vec<Out> = std::thread::scope(|scope| {
        let handles: Vec<_> = (0..threads)
            .map(|t| {
                scope.spawn(move || {
                    let mut out: Out = Default::default();
                    let mut d = t;
                    while d < deals {
                        if let Some(record) = crate::honors::play(seed + d) {
                            tally(&record, &mut out);
                        }
                        d += threads;
                    }
                    out
                })
            })
            .collect();
        handles.into_iter().map(|h| h.join().unwrap()).collect()
    });
    let (mut suits, mut nts): Out = Default::default();
    for (s, n) in parts {
        for (k, p) in s {
            let e = suits.entry(k).or_default();
            e.n += p.n;
            e.won += p.won;
            e.ruffed += p.ruffed;
            e.beaten += p.beaten;
            e.vs_top += p.vs_top;
            e.as_good += p.as_good;
        }
        for (k, p) in n {
            let e = nts.entry(k).or_default();
            e.n += p.n;
            e.led += p.led;
            e.joker_edge += p.joker_edge;
            e.joker_better += p.joker_better;
            e.joker_worse += p.joker_worse;
            e.cost_led += p.cost_led;
            e.cost_held += p.cost_held;
        }
    }
    let pct = |x: u32, n: u32| format!("{:.0}%", 100.0 * x as f64 / n.max(1) as f64);
    println!("Leads in {deals} deals of Oracle play, each compared by the solver with every card visible.\n");
    println!("**Suit bids: the declaring side holds the top trump left, and the opponents still hold trumps**\n");
    println!("| Leader | Trick | Lead | n | Share | Won / trumped / beaten | vs leading the top trump | As good or better |");
    println!("|---|---|---|---|---|---|---|---|");
    let mut totals: BTreeMap<(String, String), u32> = BTreeMap::new();
    for ((group, _), p) in &suits {
        let (who, trick) = group.split_once('|').unwrap();
        *totals.entry((who.to_string(), trick.to_string())).or_default() += p.n;
    }
    for ((group, kind), p) in &suits {
        let (who, trick) = group.split_once('|').unwrap();
        let total = totals[&(who.to_string(), trick.to_string())];
        let outcome = if kind.contains("ace") || kind.contains("king") {
            format!("{} / {} / {}", pct(p.won, p.n), pct(p.ruffed, p.n), pct(p.beaten, p.n))
        } else {
            "".into()
        };
        println!(
            "| {who} | {trick} | {kind} | {} | {} | {outcome} | {:+.2} tricks | {} |",
            p.n,
            pct(p.n, total),
            p.vs_top / p.n.max(1) as f64,
            pct(p.as_good, p.n)
        );
    }
    println!("\n**No trumps: the player on lead holds the Joker**\n");
    println!("| Leader | Trick | n | Bot led the Joker | Best Joker lead vs best other | Joker better / worse | Bot's cost when it led it / held it |");
    println!("|---|---|---|---|---|---|---|");
    for ((who, trick), p) in &nts {
        let held = p.n - p.led;
        println!(
            "| {who} | {trick} | {} | {} | {:+.2} tricks | {} / {} | {:.2} / {:.2} |",
            p.n,
            pct(p.led, p.n),
            p.joker_edge / p.n.max(1) as f64,
            pct(p.joker_better, p.n),
            pct(p.joker_worse, p.n),
            p.cost_led / p.led.max(1) as f64,
            p.cost_held / held.max(1) as f64
        );
    }
}

fn band(trick: usize) -> &'static str {
    match trick {
        0 => "1",
        1..=2 => "2–3",
        3..=5 => "4–6",
        _ => "7–10",
    }
}

fn tally(record: &HandRecord, out: &mut Out) {
    let Some((declarer, contract)) = record.contract else { return };
    let trump = contract.trump();
    let mut hands: [CardSet; 4] = record.dealt;
    hands[declarer.index()] = hands[declarer.index()].union(record.kitty).minus(record.discards);
    let mut solver = Solver::new(trump);
    for (k, trick) in record.tricks.iter().enumerate() {
        let leader = trick.leader();
        let Some((_, lead)) = trick.plays().next() else { break };
        let ours = leader.team();
        let declaring = ours == declarer.team();
        match trump {
            Some(t) if declaring && k < 9 => {
                let trumpish = |c: Card| is_trump(c, trump) || c.is_joker();
                let live: Vec<Card> = Seat::ALL.iter().flat_map(|s| hands[s.index()].iter()).filter(|&c| trumpish(c)).collect();
                let theirs = Seat::ALL.iter().filter(|s| s.team() != ours).any(|s| hands[s.index()].iter().any(trumpish));
                let top = live.iter().copied().max_by_key(|&c| trick_strength(c, t, trump));
                if let Some(top) = top.filter(|&c| theirs && hands[leader.index()].contains(c)) {
                    let values = solver.analyze(&Position { hands, trump, trick: Trick::new(leader) }, ours);
                    let value = |card: Card| values.iter().filter(|(p, _)| p.card == card).map(|&(_, v)| v).max();
                    if let (Some(chosen), Some(top_v)) = (value(lead), value(top)) {
                        let kind = if lead == top {
                            "top trump"
                        } else if trumpish(lead) {
                            "another trump"
                        } else if lead.rank() == ACE {
                            "side ace"
                        } else if lead.rank() == KING {
                            "side king"
                        } else {
                            "other side card"
                        };
                        let who = if leader == declarer { "declarer" } else { "partner" };
                        let e = out.0.entry((format!("{who}|{}", band(k)), kind.to_string())).or_default();
                        e.n += 1;
                        e.vs_top += chosen as f64 - top_v as f64;
                        e.as_good += u32::from(chosen >= top_v);
                        if !trumpish(lead) {
                            let winner = trick.winner(trump);
                            let winning = trick.winning_card(trump);
                            if winner.map(|w| w.team()) == Some(ours) {
                                e.won += 1;
                            } else if winning.is_some_and(trumpish) {
                                e.ruffed += 1;
                            } else {
                                e.beaten += 1;
                            }
                        }
                    }
                }
            }
            None if hands[leader.index()].contains(Card::JOKER) && hands[leader.index()].len() > 1 => {
                let values = solver.analyze(&Position { hands, trump, trick: Trick::new(leader) }, ours);
                let best = values.iter().map(|&(_, v)| v).max().unwrap_or(0);
                let joker = values.iter().filter(|(p, _)| p.card.is_joker()).map(|&(_, v)| v).max();
                let other = values.iter().filter(|(p, _)| !p.card.is_joker()).map(|&(_, v)| v).max();
                let chosen = values.iter().filter(|(p, _)| p.card == lead).map(|&(_, v)| v).max();
                if let (Some(j), Some(o), Some(c)) = (joker, other, chosen) {
                    let who = if declaring { "declaring side" } else { "defender" };
                    let e = out.1.entry((who.to_string(), band(k).to_string())).or_default();
                    e.n += 1;
                    e.joker_edge += j as f64 - o as f64;
                    e.joker_better += u32::from(j > o);
                    e.joker_worse += u32::from(j < o);
                    if lead.is_joker() {
                        e.led += 1;
                        e.cost_led += (best - c) as f64;
                    } else {
                        e.cost_held += (best - c) as f64;
                    }
                }
            }
            _ => {}
        }
        for (seat, card) in trick.plays() {
            hands[seat.index()].remove(card);
        }
    }
}
