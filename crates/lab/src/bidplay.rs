//! `fh-lab bidplay`: when the coach's bidding advice (The Oracle's network) and its count disagree,
//! which wins at the table?
//!
//! The network learned from deals solved with every card visible, the same blind spot that made
//! the search overrate some plays (findings 54–56). In each deal the first call where the network
//! and the count's rule (the best strain by count; 7 from a count of 7, 8 from 8½, else pass; finding
//! 14) disagree is played out both ways: the rest of the auction and the play by The Oracle in
//! every seat, the same seed both ways. Points are the caller's side's, this hand.

use std::collections::BTreeMap;

use crate::playout::{finish, Mean};
use fh_ai::search_bot::{SearchBot, SearchConfig};
use fh_ai::Bot;
use fh_core::{Action, Call, Contract, Event, Game, Observation, Phase, Rules, Strain};

#[derive(Default, Clone)]
struct Tally {
    /// The network's call's points less the count's, played out.
    real: Mean,
    /// The bid was made: with the network's call, with the count's (where that side declared).
    made: [f64; 2],
    declared: [f64; 2],
}

pub fn run(deals: u64, seed: u64, threads: u64) {
    let parts: Vec<(BTreeMap<String, Tally>, u64)> = std::thread::scope(|scope| {
        let handles: Vec<_> = (0..threads)
            .map(|t| {
                scope.spawn(move || {
                    let mut out = BTreeMap::new();
                    let mut agreed = 0;
                    let mut d = t;
                    while d < deals {
                        agreed += u64::from(deal(seed + d, &mut out));
                        d += threads;
                    }
                    (out, agreed)
                })
            })
            .collect();
        handles.into_iter().map(|h| h.join().unwrap()).collect()
    });
    let mut all: BTreeMap<String, Tally> = BTreeMap::new();
    let mut agreed = 0;
    for (part, a) in parts {
        agreed += a;
        for (k, t) in part {
            let e = all.entry(k).or_default();
            e.real.merge(&t.real);
            for i in 0..2 {
                e.made[i] += t.made[i];
                e.declared[i] += t.declared[i];
            }
        }
    }
    let total = all.values().map(|t| t.real.n).sum::<f64>();
    println!("{deals} deals: {total} with a call where the network and the count disagreed ({agreed} where they never did, or only over");
    println!("mentions and 6-calls). Each played out both ways by The Oracle; points for the caller's side, network less count.\n");
    println!("| The network | n | Points, network − count (95%) | Bid made when declared: network / count |");
    println!("|---|---|---|---|");
    let mut rows: Vec<_> = all.iter().collect();
    rows.sort_by(|a, b| b.1.real.n.total_cmp(&a.1.real.n));
    let mut sum = Mean::default();
    for (k, t) in rows {
        sum.merge(&t.real);
        let rate = |i: usize| if t.declared[i] > 0.0 { format!("{:.0}%", 100.0 * t.made[i] / t.declared[i]) } else { "—".into() };
        println!("| {k} | {} | {} | {} / {} |", t.real.n, t.real.show(1), rate(0), rate(1));
    }
    println!("| all | {} | {} | |", sum.n, sum.show(1));
}

/// The count's call: its best strain, 7 from a count of 7, 8 from 8½, or pass (also when that bid
/// isn't legal over the auction so far).
fn count_call(obs: &Observation, legal: &[Action]) -> Action {
    let (strain, count) = Strain::ALL
        .into_iter()
        .map(|s| (s, fh_ai::thumb::count(obs, s).0))
        .max_by(|a, b| a.1.total_cmp(&b.1))
        .expect("five strains");
    let level = if count >= 8.5 {
        8
    } else if count >= 7.0 {
        7
    } else {
        return Action::Call(Call::Pass);
    };
    let bid = Action::Call(Call::Bid(Contract::new(level, strain)));
    if legal.contains(&bid) { bid } else { Action::Call(Call::Pass) }
}

fn describe(network: Call, count: Call) -> String {
    match (network, count) {
        (Call::Bid(a), Call::Bid(b)) if a.strain == b.strain => {
            if a.level > b.level { "bids a level higher".into() } else { "bids a level lower".into() }
        }
        (Call::Bid(a), Call::Bid(_)) if a.strain == Strain::NoTrumps => "bids no trumps, the count a suit".into(),
        (Call::Bid(_), Call::Bid(b)) if b.strain == Strain::NoTrumps => "bids a suit, the count no trumps".into(),
        (Call::Bid(_), Call::Bid(_)) => "bids another suit".into(),
        (Call::Bid(a), _) => format!("bids ({}), the count passes", a.level),
        (_, Call::Bid(b)) => format!("passes, the count bids ({})", b.level),
        _ => "other".into(),
    }
}

/// One deal; true if the network and the count never disagreed on a bid or pass.
fn deal(seed: u64, out: &mut BTreeMap<String, Tally>) -> bool {
    let mut game = Game::new(Rules::default(), seed);
    let mut bots: [SearchBot; 4] = std::array::from_fn(|i| SearchBot::new("The Oracle", SearchConfig::ORACLE, seed * 4 + i as u64));
    while game.phase() == Phase::Bidding {
        let Some(seat) = game.to_act() else { return true };
        let obs = game.observe(seat);
        let legal = game.legal_actions();
        let network = bots[seat.index()].choose(&obs, &legal);
        let count = count_call(&obs, &legal);
        let comparable = |a: Action| matches!(a, Action::Call(Call::Bid(_)) | Action::Call(Call::Pass));
        if network != count && comparable(network) && !obs.auction.must_bid() {
            let (Action::Call(n), Action::Call(c)) = (network, count) else { return true };
            let cont = seed.wrapping_add(11_000_003);
            let play = |action: Action| {
                let mut g = game.clone();
                g.apply(action).ok()?;
                finish(g, cont, seat)
            };
            let (Some(a), Some(b)) = (play(network), play(count)) else { return true };
            let t = out.entry(describe(n, c)).or_default();
            t.real.add(a.points - b.points);
            for (i, o) in [a, b].iter().enumerate() {
                if o.declared {
                    t.declared[i] += 1.0;
                    t.made[i] += f64::from(u8::from(o.made));
                }
            }
            return false;
        }
        let Ok(events) = game.apply(network) else { return true };
        if events.iter().any(|e| matches!(e, Event::PassedOut { .. })) {
            return true;
        }
    }
    true
}
