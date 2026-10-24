//! `fh-lab countcheck`: the count's error by situation. Through Oracle auctions, at each player's
//! call, their best-count suit is solved with them declaring (kitty taken, the bots' discards);
//! the error is real-play tricks (solver less ½) less the count, grouped by situation.

use std::collections::BTreeMap;

use fh_ai::rule_bot::heuristic_discards;
use fh_ai::solver::{Position, Solver};
use fh_core::rules::is_trump;
use fh_core::{effective_suit, Call, Card, CardSet, Game, Phase, Rules, Seat, Strain, Suit, Trick};

pub fn run(deals: u64, seed: u64, threads: u64) {
    let parts: Vec<BTreeMap<String, (u32, f64)>> = std::thread::scope(|scope| {
        let handles: Vec<_> = (0..threads)
            .map(|t| {
                scope.spawn(move || {
                    let mut out = BTreeMap::new();
                    let mut d = t;
                    while d < deals {
                        one(seed + d, &mut out);
                        d += threads;
                    }
                    out
                })
            })
            .collect();
        handles.into_iter().map(|h| h.join().unwrap()).collect()
    });
    let mut all: BTreeMap<String, (u32, f64)> = BTreeMap::new();
    for part in parts {
        for (k, (n, s)) in part {
            let e = all.entry(k).or_default();
            e.0 += n;
            e.1 += s;
        }
    }
    println!("The count's error by situation ({deals} deals; real-play tricks less the count)\n");
    println!("| Situation | n | Error |\n|---|---|---|");
    for (k, (n, s)) in &all {
        println!("| {k} | {n} | {:+.2} |", s / *n as f64);
    }
}

fn one(seed: u64, out: &mut BTreeMap<String, (u32, f64)>) {
    let mut game = Game::new(Rules::default(), seed);
    let mut bots: [Box<dyn fh_ai::Bot>; 4] = std::array::from_fn(|i| crate::make_bot("oracle", seed.wrapping_mul(4).wrapping_add(i as u64)));
    let hands: [CardSet; 4] = std::array::from_fn(|i| game.hand(Seat::new(i)));
    let kitty = game.kitty();
    while game.phase() == Phase::Bidding && game.history().is_empty() {
        let seat = game.to_act().unwrap();
        let obs = game.observe(seat);
        let legal = game.legal_actions();
        let (strain, count) = Strain::ALL.into_iter().filter(|s| s.trump().is_some()).map(|s| (s, fh_ai::thumb::count(&obs, s).0)).max_by(|a, b| a.1.total_cmp(&b.1)).unwrap();
        let trump = strain.trump();
        let mut h = hands;
        let thirteen = h[seat.index()].union(kitty);
        h[seat.index()] = thirteen.minus(heuristic_discards(thirteen, trump, 3).into_iter().collect());
        let mut solver = Solver::with_limit(trump, 4_000_000);
        let tricks = solver.solve(&Position { hands: h, trump, trick: Trick::new(seat) }, seat.team());
        if !solver.aborted() {
            let err = tricks as f64 - 0.5 - count as f64;
            let calls: Vec<(Seat, Call)> = obs.auction.calls().collect();
            let partner_call = calls.iter().find(|(s, _)| *s == seat.partner()).map(|&(_, c)| c);
            let opponents_passed = calls.iter().filter(|(s, c)| s.team() != seat.team() && *c == Call::Pass).count() == 2;
            let joker = obs.hand.contains(Card::JOKER);
            let bare_ace = Suit::ALL.into_iter().filter(|&s| Some(s) != trump).any(|s| {
                let cards: Vec<Card> = obs.hand.iter().filter(|&c| !is_trump(c, trump) && !c.is_joker() && effective_suit(c, trump) == Some(s)).collect();
                cards.len() == 1 && cards[0].rank() == fh_core::cards::ACE
            });
            let mut keys = vec!["all".to_string(), format!("seat {}", calls.len() + 1)];
            if let Some(Call::Bid(c)) = partner_call {
                if c.strain != strain {
                    keys.push("partner bid another strain".into());
                }
            }
            if partner_call == Some(Call::Pass) {
                keys.push(if joker { "partner passed, you hold the Joker" } else { "partner passed, no Joker" }.into());
            }
            if opponents_passed {
                keys.push("both opponents passed".into());
            }
            if bare_ace {
                keys.push("you hold a bare side ace".into());
            }
            let band = match count {
                c if c < 6.0 => "count under 6",
                c if c < 7.0 => "count 6–7",
                c if c < 8.0 => "count 7–8",
                c if c < 9.0 => "count 8–9",
                _ => "count 9+",
            };
            keys.push(band.into());
            for k in keys {
                let e = out.entry(k).or_default();
                e.0 += 1;
                e.1 += err;
            }
        }
        let action = bots[seat.index()].choose(&obs, &legal);
        game.apply(action).unwrap();
    }
}
