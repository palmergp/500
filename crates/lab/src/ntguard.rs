//! `fh-lab ntguard`: in a no-trump kitty, keep a short suit's guarded honour, or put the suit back?
//!
//! Lesson 6 says that in no trumps a void gains nothing and a guarded king (K-x) is worth about a
//! fifth of a trick (finding 23, every card seen). But the coach's searched no-trump plan (finding
//! 62), which took 0.34 tricks more played out, often empties a short suit to keep length in the
//! long ones, honour and all. Here, at each no-trump declarer's discard holding a short suit (two
//! cards, or three with a queen) with a king or queen but no ace, the rule's plan (which keeps it)
//! is played out against putting the whole suit back, by The Oracle in every seat.

use std::collections::BTreeMap;

use crate::playout::{finish, Mean};
use fh_ai::rule_bot::heuristic_discards;
use fh_ai::search_bot::{SearchBot, SearchConfig};
use fh_ai::Bot;
use fh_core::cards::{ACE, KING, QUEEN};
use fh_core::{Action, Card, CardSet, Event, Game, Phase, Rules, Suit};

#[derive(Default, Clone)]
struct Tally {
    /// Putting the suit back, less keeping it: points and tricks, and bids made each way.
    real: Mean,
    tricks: Mean,
    made_keep: f64,
    made_void: f64,
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
            e.made_keep += t.made_keep;
            e.made_void += t.made_void;
        }
    }
    println!("{deals} deals, {hands} no-trump bids. Where the declarer held a short suit with a king or queen and no ace, and the");
    println!("rule's kitty plan kept it: that plan against putting the whole suit back (the rest by the rule), played out by The Oracle.\n");
    println!("| Short suit | n | Putting it back, points (95%) | Tricks | Bid made: kept / put back |");
    println!("|---|---|---|---|---|");
    for (k, t) in &all {
        let n = t.real.n.max(1.0);
        println!("| {k} | {} | {} | {} | {:.0}% / {:.0}% |", t.real.n, t.real.show(1), t.tricks.show(2), 100.0 * t.made_keep / n, 100.0 * t.made_void / n);
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
    let rule = heuristic_discards(hand, None, 3);
    let set = |c: &[Card]| c.iter().copied().collect::<CardSet>();
    let longest = Suit::ALL.into_iter().map(|s| hand.iter().filter(|c| c.suit() == Some(s)).count()).max().unwrap_or(0);
    for suit in Suit::ALL {
        let cards: Vec<Card> = hand.iter().filter(|c| c.suit() == Some(suit)).collect();
        let has = |r: u8| cards.iter().any(|c| c.rank() == r);
        let short = cards.len() == 2 || (cards.len() == 3 && has(QUEEN) && !has(KING));
        if !short || has(ACE) || !(has(KING) || has(QUEEN)) {
            continue;
        }
        // The rule keeps the honour; the other way puts the whole suit back.
        let honour = cards.iter().copied().max_by_key(|c| c.rank()).expect("cards");
        if rule.contains(&honour) {
            continue;
        }
        let rest = heuristic_discards(hand.minus(set(&cards)), None, 3 - cards.len());
        let void: Vec<Card> = cards.iter().copied().chain(rest).collect();
        let cont = seed.wrapping_add(19_000_013);
        let play = |discards: &[Card]| {
            let mut g = game.clone();
            for &c in discards {
                g.apply(Action::Discard(c)).ok()?;
            }
            finish(g, cont, declarer)
        };
        let (Some(keep), Some(back)) = (play(&rule), play(&void)) else { return };
        let what = match (has(KING), has(QUEEN), cards.len()) {
            (true, true, _) => "K-Q",
            (true, false, _) => "K-x",
            (false, true, 2) => "Q-x",
            _ => "Q-x-x",
        };
        let long = if longest >= 5 { "a five-card suit or longer" } else { "no suit longer than four" };
        for key in [what.to_string(), format!("{what}, {long}"), "all".to_string()] {
            let t = out.entry(key).or_default();
            t.real.add(back.points - keep.points);
            t.tricks.add(f64::from(back.tricks - keep.tricks));
            t.made_keep += f64::from(u8::from(keep.made));
            t.made_void += f64::from(u8::from(back.made));
        }
        return; // one suit a deal
    }
}
