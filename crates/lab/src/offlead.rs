//! `fh-lab offlead`: why does a declarer open with a low side card (say, an off-suit 6) that looks
//! like a sure loser? Over Oracle-played deals, the declarer's opening lead is classified (top
//! trump, low trump, side winner, low side card by how many the declarer holds in that suit), and
//! the solver (every card visible) values it against the best lead and against the top trump.

use std::collections::BTreeMap;

use fh_ai::cardplay::{is_boss, suit_order};
use fh_ai::search_bot::{SearchBot, SearchConfig};
use fh_ai::solver::{Position, Solver};
use fh_ai::Bot;
use fh_core::rules::is_trump;
use fh_core::{effective_suit, Action, Game, Phase, Rules, Seat, Trick};

#[derive(Default, Clone, Copy)]
struct Tally {
    n: u32,
    /// Solver tricks: the lead against the best lead (≤ 0), and against the top trump.
    vs_best: f64,
    vs_top: f64,
    top_n: u32,
    best: u32,
}

pub fn run(deals: u64, seed: u64, threads: u64) {
    let parts: Vec<BTreeMap<String, Tally>> = std::thread::scope(|scope| {
        let handles: Vec<_> = (0..threads)
            .map(|t| {
                scope.spawn(move || {
                    let mut out = BTreeMap::new();
                    let mut d = t;
                    while d < deals {
                        deal(seed + d, &mut out);
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
            e.n += t.n;
            e.vs_best += t.vs_best;
            e.vs_top += t.vs_top;
            e.top_n += t.top_n;
            e.best += t.best;
        }
    }
    let total: u32 = all.iter().filter(|(k, _)| k.starts_with("suit")).map(|(_, t)| t.n).sum();
    let total_nt: u32 = all.iter().filter(|(k, _)| k.starts_with("no trumps")).map(|(_, t)| t.n).sum();
    println!("{deals} deals. The Oracle's opening lead as declarer, by kind; the solver's value (every card seen) against");
    println!("the best lead and against leading the top trump instead.\n");
    println!("| Bid | Lead | Share | Best lead (solver) | Tricks vs the best lead | Tricks vs the top trump |");
    println!("|---|---|---|---|---|---|");
    for (k, t) in &all {
        let n = t.n.max(1) as f64;
        let of = if k.starts_with("suit") { total } else { total_nt }.max(1) as f64;
        let vs_top = if t.top_n > 0 { format!("{:+.2}", t.vs_top / t.top_n as f64) } else { "—".into() };
        let (bid, lead) = k.split_once(": ").unwrap_or((k, ""));
        println!("| {bid} | {lead} | {:.0}% ({}) | {:.0}% | {:+.2} | {vs_top} |", 100.0 * t.n as f64 / of, t.n, 100.0 * t.best as f64 / n, t.vs_best / n);
    }
}

fn deal(seed: u64, out: &mut BTreeMap<String, Tally>) {
    let mut game = Game::new(Rules::default(), seed);
    let mut bots: [SearchBot; 4] = std::array::from_fn(|i| SearchBot::new("The Oracle", SearchConfig::ORACLE, seed * 4 + i as u64));
    loop {
        let Some(seat) = game.to_act() else { return };
        if game.phase() == Phase::Playing {
            let Some((_, contract)) = game.contract() else { return };
            let trump = contract.trump();
            let action = bots[seat.index()].choose(&game.observe(seat), &game.legal_actions());
            let Action::Play(play) = action else { return };
            let hand = game.hand(seat);
            let seen = hand;
            let card = play.card;
            let suit_len = |c: fh_core::Card| hand.iter().filter(|&h| effective_suit(h, trump) == effective_suit(c, trump)).count();
            let top = trump.and_then(|t| suit_order(t, trump).into_iter().find(|&c| hand.contains(c) && is_boss(c, seen, trump)));
            let kind = if card.is_joker() {
                "the Joker".to_string()
            } else if is_trump(card, trump) {
                if Some(card) == top { "top trump".into() } else { "a lower trump".into() }
            } else if is_boss(card, seen, trump) {
                "a side winner (ace)".into()
            } else if card.rank() >= fh_core::cards::KING {
                "a side king".into()
            } else {
                match suit_len(card) {
                    1 => "a low side card: singleton".into(),
                    2 => "a low side card: from two".into(),
                    3 => "a low side card: from three".into(),
                    _ => "a low side card: from four or more".into(),
                }
            };
            let hands: [fh_core::CardSet; 4] = std::array::from_fn(|i| game.hand(Seat::new(i)));
            let values = Solver::new(trump).analyze(&Position { hands, trump, trick: Trick::new(seat) }, seat.team());
            let value = |c: fh_core::Card| values.iter().filter(|(p, _)| p.card == c).map(|&(_, v)| v as f64).fold(f64::MIN, f64::max);
            let best = values.iter().map(|&(_, v)| v as f64).fold(f64::MIN, f64::max);
            let bid = if trump.is_some() { "suit" } else { "no trumps" };
            let t = out.entry(format!("{bid}: {kind}")).or_default();
            t.n += 1;
            t.vs_best += value(card) - best;
            t.best += u32::from(value(card) >= best);
            if let Some(top) = top.filter(|&c| c != card) {
                t.top_n += 1;
                t.vs_top += value(card) - value(top);
            }
            return;
        }
        let action = bots[seat.index()].choose(&game.observe(seat), &game.legal_actions());
        if game.apply(action).is_err() {
            return;
        }
    }
}
