//! `fh-lab honors`: how often a side-suit king or queen wins a trick in expert play.
//!
//! The Oracle plays whole hands in all four seats (its bidding, kitty and card play). For every
//! side-suit king and queen dealt to any player (in no trumps every suit is a side suit), we note
//! how it was held (alone, with one or more guards, with the ace or king beside it), who held it
//! (declarer, declarer's partner, a defender), where the suit's ace was, and whether the card
//! itself won a trick. Cards the declarer put back in the kitty are left out.

use std::collections::BTreeMap;

use fh_core::cards::{ACE, KING, QUEEN};
use fh_core::rules::is_trump;
use fh_core::{effective_suit, Card, CardSet, Event, Game, HandRecord, Rules, Seat};

#[derive(Default, Clone, Copy)]
struct Tally {
    held: u32,
    won: u32,
}

pub fn run(deals: u64, seed: u64, threads: u64) {
    let parts: Vec<BTreeMap<String, Tally>> = std::thread::scope(|scope| {
        let handles: Vec<_> = (0..threads)
            .map(|t| {
                scope.spawn(move || {
                    let mut out = BTreeMap::new();
                    let mut d = t;
                    while d < deals {
                        if let Some(record) = play(seed + d) {
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
    let mut all: BTreeMap<String, Tally> = BTreeMap::new();
    for part in parts {
        for (k, t) in part {
            let e = all.entry(k).or_default();
            e.held += t.held;
            e.won += t.won;
        }
    }
    println!("Side-suit kings and queens over {deals} deals of expert play: how often the card itself won a trick\n");
    println!("| Holding | Times held | Won a trick |");
    println!("|---|---|---|");
    for (k, t) in &all {
        if t.held >= 40 {
            println!("| {k} | {} | {:.0}% |", t.held, 100.0 * t.won as f64 / t.held as f64);
        }
    }
}

/// One hand of Oracle play; the record of the first deal that was bid and played.
pub(crate) fn play(seed: u64) -> Option<HandRecord> {
    let mut game = Game::new(Rules::default(), seed);
    let mut bots: [Box<dyn fh_ai::Bot>; 4] = std::array::from_fn(|i| crate::make_bot("oracle", seed.wrapping_mul(4).wrapping_add(i as u64)));
    loop {
        let seat = game.to_act()?;
        let action = bots[seat.index()].choose(&game.observe(seat), &game.legal_actions());
        let events = game.apply(action).ok()?;
        if events.iter().any(|e| matches!(e, Event::HandScored { .. })) {
            return game.history().iter().find(|h| h.contract.is_some()).cloned();
        }
    }
}

fn tally(record: &HandRecord, out: &mut BTreeMap<String, Tally>) {
    let Some((declarer, contract)) = record.contract else { return };
    let trump = contract.trump();
    let kind = if trump.is_some() { "suit bid" } else { "no trumps" };
    // Which cards won tricks.
    let mut winners = CardSet::EMPTY;
    for trick in &record.tricks {
        if let Some(card) = trick.winning_card(trump) {
            winners.insert(card);
        }
    }
    for seat in Seat::ALL {
        let hand = record.dealt[seat.index()];
        let role = if seat == declarer {
            "declarer"
        } else if seat == declarer.partner() {
            "declarer's partner"
        } else {
            "defender"
        };
        for card in hand.iter() {
            if card.is_joker() || is_trump(card, trump) || (card.rank() != KING && card.rank() != QUEEN) {
                continue;
            }
            if seat == declarer && record.discards.contains(card) {
                continue; // put back in the kitty
            }
            let suit = effective_suit(card, trump).expect("a side card");
            let in_suit = hand.iter().filter(|&c| !is_trump(c, trump) && !c.is_joker() && effective_suit(c, trump) == Some(suit)).count();
            let ace = Card::new(suit, ACE);
            let king = Card::new(suit, KING);
            let (name, holding) = if card.rank() == KING {
                let with_ace = hand.contains(ace);
                let shape = match (with_ace, in_suit) {
                    (true, _) => "with the ace (A-K)".to_string(),
                    (false, 1) => "alone".to_string(),
                    (false, 2) => "with 1 guard".to_string(),
                    (false, 3) => "with 2 guards".to_string(),
                    (false, _) => "with 3+ guards".to_string(),
                };
                ("King", shape)
            } else {
                let shape = if hand.contains(king) {
                    "with the king (K-Q)".to_string()
                } else {
                    match in_suit {
                        1 => "alone".to_string(),
                        2 => "with 1 guard".to_string(),
                        3 => "with 2 guards".to_string(),
                        _ => "with 3+ guards".to_string(),
                    }
                };
                ("Queen", shape)
            };
            let won = winners.contains(card);
            let mut add = |key: String| {
                let t = out.entry(key).or_default();
                t.held += 1;
                t.won += u32::from(won);
            };
            add(format!("{kind} · {name} {holding} · all players"));
            add(format!("{kind} · {name} {holding} · {role}"));
            // For an unaccompanied king or queen: where the suit's ace was (left, partner, right).
            if card.rank() == KING && !hand.contains(ace) && in_suit >= 2 {
                let ace_at = Seat::ALL.into_iter().find(|s| record.dealt[s.index()].contains(ace));
                let place = match ace_at.map(|s| (s.index() + 4 - seat.index()) % 4) {
                    Some(1) => "ace on your left (it plays after you)",
                    Some(2) => "ace with your partner",
                    Some(3) => "ace on your right (it plays before you)",
                    _ => "ace in the kitty",
                };
                add(format!("{kind} · King guarded, no ace · {place}"));
            }
        }
    }
}
