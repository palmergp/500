//! `fh-lab mentions`: what does a mention tell partner, by what the mentioner holds?
//!
//! On random deals, for each player and each suit, we classify the player's holding in that suit
//! (as trumps: with or without a bower, and how many trumps) and solve the hand with their partner
//! declaring in it (partner takes the kitty and discards as the bots do). Partner's tricks less
//! partner's own count (the rule of thumb, whose +3 assumes an average partner and kitty) shows
//! what the holding is worth to partner: the value of mentioning it.

use std::collections::BTreeMap;

use fh_ai::rule_bot::heuristic_discards;
use fh_ai::solver::{Position, Solver};
use fh_core::rules::is_trump;
use fh_core::{Card, CardSet, Game, Rules, Seat, Strain, Suit, Trick};

pub fn run(deals: u64, seed: u64, threads: u64) {
    let parts: Vec<BTreeMap<&'static str, (u32, f64)>> = std::thread::scope(|scope| {
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
    let mut all: BTreeMap<&str, (u32, f64)> = BTreeMap::new();
    for part in parts {
        for (k, (n, s)) in part {
            let e = all.entry(k).or_default();
            e.0 += n;
            e.1 += s;
        }
    }
    let base = all.get(BASE).map_or(0.0, |&(n, s)| s / n.max(1) as f64);
    println!("Partner's tricks in a suit, less partner's own count, by what you hold in it ({deals} deals, solver, real play ≈ solver − ½)\n");
    println!("Compared with holding no bower and three trumps or fewer (when you wouldn't mention): {base:+.2}\n");
    println!("| What you hold in the suit (as trumps) | n | Worth to partner, against not mentioning |");
    println!("|---|---|---|");
    for (k, &(n, s)) in &all {
        if *k != BASE {
            println!("| {k} | {n} | {:+.2} tricks |", s / n as f64 - base);
        }
    }
}

const BASE: &str = "no bower, 3 trumps or fewer";

fn one(seed: u64, out: &mut BTreeMap<&'static str, (u32, f64)>) {
    let game = Game::new(Rules::default(), seed);
    let dealt: [CardSet; 4] = std::array::from_fn(|i| game.hand(Seat::new(i)));
    for suit in Suit::ALL {
        let trump = Some(suit);
        for declarer in Seat::ALL {
            let partner = declarer.partner();
            let held = dealt[partner.index()];
            let trumps = held.iter().filter(|&c| is_trump(c, trump) && !c.is_joker()).count();
            let bower = held.contains(Card::new(suit, fh_core::cards::JACK)) || held.contains(Card::new(suit.same_color(), fh_core::cards::JACK));
            let kind = match (bower, trumps) {
                (true, 0..=2) => "a bower, 2 trumps or fewer in all",
                (true, 3) => "a bower and 2 more trumps (3 in all)",
                (true, _) => "a bower and 3+ more trumps (4+ in all)",
                (false, 0..=3) => BASE,
                (false, 4) => "no bower, 4 trumps",
                (false, _) => "no bower, 5+ trumps",
            };
            // Partner (the declarer here) takes the kitty and discards.
            let mut hands = dealt;
            let thirteen = dealt[declarer.index()].union(game.kitty());
            hands[declarer.index()] = thirteen.minus(heuristic_discards(thirteen, trump, 3).into_iter().collect());
            let tricks = Solver::new(trump).solve(&Position { hands, trump, trick: Trick::new(declarer) }, declarer.team());
            let count = fh_ai::thumb::count(&game.observe(declarer), Strain::from_suit(suit)).0;
            let e = out.entry(kind).or_default();
            e.0 += 1;
            e.1 += tricks as f64 - 0.5 - count as f64;
        }
    }
}
