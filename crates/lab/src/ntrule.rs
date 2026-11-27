//! `fh-lab ntrule`: the no-trump kitty rule before and after finding 67, played out.
//!
//! `ntguard` found that putting back a short queen suit (Q-x, Q-x-x, no ace or king) did better
//! than keeping it as a stopper. The rule now puts back any short suit with no ace or king first,
//! to keep length in the others. Here, at each no-trump declarer's discard (The Oracle's auctions),
//! where the old rule and the new put back different cards, both are played out by The Oracle in
//! every seat with the same seed.

use std::collections::BTreeMap;

use crate::playout::{finish, Mean};
use fh_ai::rule_bot::{nt_greedy_discards, NtShortSuits};
use fh_ai::search_bot::{SearchBot, SearchConfig};
use fh_ai::Bot;
use fh_core::cards::{ACE, KING, QUEEN};
use fh_core::{Action, Card, CardSet, Event, Game, Phase, Rules, Suit};

#[derive(Default, Clone)]
struct Tally {
    /// The new rule's plan less the old one's: points and tricks, and bids made each way.
    real: Mean,
    tricks: Mean,
    made_old: f64,
    made_new: f64,
}

pub fn run(deals: u64, seed: u64, threads: u64) {
    let parts: Vec<(BTreeMap<String, Tally>, u64)> = std::thread::scope(|scope| {
        let handles: Vec<_> = (0..threads)
            .map(|t| {
                scope.spawn(move || {
                    let mut out = BTreeMap::new();
                    let mut hands = 0;
                    let mut d = t;
                    while d < deals {
                        deal(seed + d, &mut out, &mut hands);
                        d += threads;
                    }
                    (out, hands)
                })
            })
            .collect();
        handles.into_iter().map(|h| h.join().unwrap()).collect()
    });
    let mut all: BTreeMap<String, Tally> = BTreeMap::new();
    let mut hands = 0;
    for (part, h) in parts {
        hands += h;
        for (k, t) in part {
            let e = all.entry(k).or_default();
            e.real.merge(&t.real);
            e.tricks.merge(&t.tricks);
            e.made_old += t.made_old;
            e.made_new += t.made_new;
        }
    }
    println!("{deals} deals, {hands} no-trump bids. Where the no-trump kitty rule before finding 67 (keep a short queen suit)");
    println!("and after it (put back short suits with no ace or king first) differ, both played out by The Oracle, same seed.\n");
    println!("| The new rule puts back | n | New less old, points (95%) | Tricks | Bid made: old / new |");
    println!("|---|---|---|---|---|");
    for (k, t) in &all {
        let n = t.real.n.max(1.0);
        println!("| {k} | {} | {} | {} | {:.0}% / {:.0}% |", t.real.n, t.real.show(1), t.tricks.show(2), 100.0 * t.made_old / n, 100.0 * t.made_new / n);
    }
}

/// What the new plan puts back that the old one kept.
fn what(hand: CardSet, old: &[Card], new: &[Card]) -> &'static str {
    let suit_of = |s: Suit| -> Vec<Card> { hand.iter().filter(|c| c.suit() == Some(s)).collect() };
    let changed: Vec<Card> = new.iter().copied().filter(|c| !old.contains(c)).collect();
    let short = |c: Card| {
        let cards = c.suit().map(suit_of).unwrap_or_default();
        let has = |r: u8| cards.iter().any(|x| x.rank() == r);
        (cards.len() <= 3 && !has(ACE) && !has(KING), has(QUEEN))
    };
    if changed.iter().any(|&c| short(c) == (true, true)) {
        "a short suit headed by a queen"
    } else if changed.iter().any(|&c| short(c).0) {
        "a short suit below the queen"
    } else {
        "something else"
    }
}

fn deal(seed: u64, out: &mut BTreeMap<String, Tally>, hands: &mut u64) {
    let mut game = Game::new(Rules::default(), seed);
    let mut bots: [SearchBot; 4] = std::array::from_fn(|i| SearchBot::new("The Oracle", SearchConfig::ORACLE, seed * 4 + i as u64));
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
    let Some((declarer, contract)) = game.contract() else { return };
    if contract.trump().is_some() {
        return;
    }
    *hands += 1;
    let hand = game.observe(declarer).hand;
    let old = nt_greedy_discards(hand, 3, NtShortSuits::KeepQueens);
    let new = nt_greedy_discards(hand, 3, NtShortSuits::PutBack);
    let set = |c: &[Card]| c.iter().copied().collect::<CardSet>();
    if set(&old) == set(&new) {
        return;
    }
    let cont = seed.wrapping_add(29_000_017);
    let play = |discards: &[Card]| {
        let mut g = game.clone();
        for &c in discards {
            g.apply(Action::Discard(c)).ok()?;
        }
        finish(g, cont, declarer)
    };
    let (Some(a), Some(b)) = (play(&old), play(&new)) else { return };
    let longest = Suit::ALL.into_iter().map(|s| hand.iter().filter(|c| c.suit() == Some(s)).count()).max().unwrap_or(0);
    let long = if longest >= 5 { "a five-card suit or longer" } else { "no suit longer than four" };
    let what = what(hand, &old, &new);
    for key in [what.to_string(), format!("{what}, {long}"), "all".to_string()] {
        let t = out.entry(key).or_default();
        t.real.add(b.points - a.points);
        t.tricks.add(f64::from(b.tricks - a.tricks));
        t.made_old += f64::from(u8::from(a.made));
        t.made_new += f64::from(u8::from(b.made));
    }
}
