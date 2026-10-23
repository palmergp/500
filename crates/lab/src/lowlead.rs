//! `fh-lab lowlead`: why does declarer lead a low side card right after a winning trump?
//!
//! The Oracle plays whole hands in all four seats. Whenever the declarer leads a trump to the first
//! trick, wins it, and then leads a low side card (10 or below), we note the side suit's length in
//! declarer's hand, who won that trick, and whether declarer later trumped the suit (a void made).
//! The solver, with every card visible, compares that lead with leading another trump.

use std::collections::BTreeMap;

use fh_ai::solver::{Position, Solver};
use fh_core::rules::is_trump;
use fh_core::{effective_suit, CardSet, HandRecord, Trick};

#[derive(Default, Clone, Copy)]
struct Tally {
    n: u32,
    ruffed_later: u32,
    partner_won: u32,
    opponents_won: u32,
    /// Solver tricks for declarer's side: this lead minus the best trump lead.
    vs_trump: f64,
    /// This lead was the solver's best (or tied).
    best: u32,
}

/// The low leads by length, and what declarer led to trick 2.
type Counts = (BTreeMap<String, Tally>, BTreeMap<&'static str, u32>);

pub fn run(deals: u64, seed: u64, threads: u64) {
    let parts: Vec<Counts> = std::thread::scope(|scope| {
        let handles: Vec<_> = (0..threads)
            .map(|t| {
                scope.spawn(move || {
                    let (mut cases, mut second) = (BTreeMap::new(), BTreeMap::new());
                    let mut d = t;
                    while d < deals {
                        if let Some(record) = crate::honors::play(seed + d) {
                            tally(&record, &mut cases, &mut second);
                        }
                        d += threads;
                    }
                    (cases, second)
                })
            })
            .collect();
        handles.into_iter().map(|h| h.join().unwrap()).collect()
    });
    let mut cases: BTreeMap<String, Tally> = BTreeMap::new();
    let mut second: BTreeMap<&str, u32> = BTreeMap::new();
    for (c, s) in parts {
        for (k, t) in c {
            let e = cases.entry(k).or_default();
            e.n += t.n;
            e.ruffed_later += t.ruffed_later;
            e.partner_won += t.partner_won;
            e.opponents_won += t.opponents_won;
            e.vs_trump += t.vs_trump;
            e.best += t.best;
        }
        for (k, n) in s {
            *second.entry(k).or_default() += n;
        }
    }
    let total: u32 = second.values().sum();
    println!("Suit bids over {deals} deals of Oracle play where declarer led a trump to trick 1, won it, and led again: {total}\n");
    println!("| Declarer's second lead | Share |");
    println!("|---|---|");
    for (k, n) in &second {
        println!("| {k} | {:.0}% |", 100.0 * *n as f64 / total.max(1) as f64);
    }
    println!("\nThe low side-card leads, by the suit's length in declarer's hand (including the card led):\n");
    println!("| Length | n | Declarer trumped the suit later | Partner won the trick | Opponents won it | Solver: vs the best trump lead | Solver's best lead |");
    println!("|---|---|---|---|---|---|---|");
    for (k, t) in &cases {
        let p = |x: u32| format!("{:.0}%", 100.0 * x as f64 / t.n.max(1) as f64);
        println!("| {k} | {} | {} | {} | {} | {:+.2} tricks | {} |", t.n, p(t.ruffed_later), p(t.partner_won), p(t.opponents_won), t.vs_trump / t.n.max(1) as f64, p(t.best));
    }
}

fn tally(record: &HandRecord, cases: &mut BTreeMap<String, Tally>, second: &mut BTreeMap<&'static str, u32>) {
    let Some((declarer, contract)) = record.contract else { return };
    let Some(trump) = contract.trump() else { return };
    let tr = Some(trump);
    tally_second(record, declarer, tr, second);
    // Declarer led and won every trick so far, at least one with a trump, and leads trick 2 or 3.
    let trumpish = |c: fh_core::Card| is_trump(c, tr) || c.is_joker();
    let lead = |t: &Trick| t.plays().next().map(|(_, c)| c);
    let Some(k) = (1..=2).find(|&k| {
        record.tricks.get(k).is_some_and(|next| {
            next.leader() == declarer
                && record.tricks[..k].iter().all(|t| t.leader() == declarer && t.winner(tr) == Some(declarer))
                && record.tricks[..k].iter().any(|t| lead(t).is_some_and(trumpish))
                && lead(next).is_some_and(|c| !trumpish(c) && c.rank() <= 10)
        })
    }) else {
        return;
    };
    let next = &record.tricks[k];
    let Some(lead2) = lead(next) else { return };
    // The hands as this trick begins.
    let mut hands: [CardSet; 4] = record.dealt;
    hands[declarer.index()] = hands[declarer.index()].union(record.kitty).minus(record.discards);
    for trick in &record.tricks[..k] {
        for (seat, card) in trick.plays() {
            hands[seat.index()] = hands[seat.index()].minus([card].into_iter().collect());
        }
    }
    let suit = effective_suit(lead2, tr).expect("a side card");
    let length = hands[declarer.index()].iter().filter(|&c| !c.is_joker() && !is_trump(c, tr) && effective_suit(c, tr) == Some(suit)).count();
    let mut t = Tally { n: 1, ..Tally::default() };
    match next.winner(tr) {
        Some(w) if w == declarer.partner() => t.partner_won = 1,
        Some(w) if w.team() != declarer.team() => t.opponents_won = 1,
        _ => {}
    }
    t.ruffed_later = u32::from(record.tricks.iter().skip(k + 1).any(|trick| {
        let led = trick.plays().next().and_then(|(_, c)| effective_suit(c, tr));
        led == Some(suit) && trick.plays().any(|(s, c)| s == declarer && (is_trump(c, tr) || c.is_joker()))
    }));
    let mut solver = Solver::new(tr);
    let values = solver.analyze(&Position { hands, trump: tr, trick: Trick::new(declarer) }, declarer.team());
    let value_of = |card| values.iter().filter(|(p, _)| p.card == card).map(|&(_, v)| v).max();
    let best = values.iter().map(|&(_, v)| v).max().unwrap_or(0);
    let best_trump = values.iter().filter(|(p, _)| is_trump(p.card, tr) || p.card.is_joker()).map(|&(_, v)| v).max();
    let Some(chosen) = value_of(lead2) else { return };
    let Some(best_trump) = best_trump else { return }; // no trumps left to compare with
    t.vs_trump = chosen as f64 - best_trump as f64;
    t.best = u32::from(chosen == best);
    let key = match length {
        1 => "1 (a singleton: leading it makes a void)".to_string(),
        2 => "2 (a doubleton)".to_string(),
        _ => "3 or more".to_string(),
    };
    let e = cases.entry(key).or_default();
    e.n += t.n;
    e.ruffed_later += t.ruffed_later;
    e.partner_won += t.partner_won;
    e.opponents_won += t.opponents_won;
    e.vs_trump += t.vs_trump;
    e.best += t.best;
}

/// What declarer led to trick 2 after leading a trump to trick 1 and winning it.
fn tally_second(record: &HandRecord, declarer: fh_core::Seat, tr: Option<fh_core::Suit>, second: &mut BTreeMap<&'static str, u32>) {
    let (Some(first), Some(next)) = (record.tricks.first(), record.tricks.get(1)) else { return };
    let (Some((_, lead1)), Some((_, lead2))) = (first.plays().next(), next.plays().next()) else { return };
    if first.leader() != declarer || !(is_trump(lead1, tr) || lead1.is_joker()) || first.winner(tr) != Some(declarer) || next.leader() != declarer {
        return;
    }
    let kind = if lead2.is_joker() || is_trump(lead2, tr) {
        "another trump"
    } else if lead2.rank() >= 13 {
        "a side ace or king"
    } else if lead2.rank() <= 10 {
        "a low side card (10 or below)"
    } else {
        "a side queen or jack"
    };
    *second.entry(kind).or_default() += 1;
}
