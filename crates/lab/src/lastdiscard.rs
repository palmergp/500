//! `fh-lab lastdiscard`: two discards are obvious (a low doubleton, put back to make a void); where
//! does the third go? The small card beside an ace (leaving the ace alone, a second void once it's
//! cashed), or the lowest card of a long weak suit?
//!
//! South declares at the rule-of-thumb level (as in `fh-lab declarer`). Where the 13 cards hold
//! exactly three side suits: a doubleton with no ace or king, an ace with one small card, and three
//! or more cards with no ace, both plans are solved on the same deal with every card visible.

use std::collections::BTreeMap;

use fh_ai::solver::{Position, Solver};
use fh_core::cards::{ACE, KING};
use fh_core::rules::is_trump;
use fh_core::{effective_suit, Action, Call, Card, CardSet, Contract, Game, Rules, Seat, Strain, Suit, Trick};

#[derive(Default, Clone, Copy)]
struct Pair {
    n: u32,
    void: f64,
    keep: f64,
    void_made: f64,
    keep_made: f64,
    void_better: u32,
    keep_better: u32,
    /// The bots' and coach's kitty plan put back the small card beside the ace.
    plan_voids: u32,
}

pub fn run(deals: u64, seed: u64, threads: u64) {
    let parts: Vec<BTreeMap<String, Pair>> = std::thread::scope(|scope| {
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
    let mut all: BTreeMap<String, Pair> = BTreeMap::new();
    for part in parts {
        for (k, p) in part {
            let e = all.entry(k).or_default();
            e.n += p.n;
            e.void += p.void;
            e.keep += p.keep;
            e.void_made += p.void_made;
            e.keep_made += p.keep_made;
            e.void_better += p.void_better;
            e.keep_better += p.keep_better;
            e.plan_voids += p.plan_voids;
        }
    }
    println!("After voiding a low doubleton: the third discard beside an ace (\"ace\") or from the long weak suit (\"long\")? ({deals} deals)\n");
    println!("| Long suit, trumps | Deals | Tricks, ace | Tricks, long | Bid made, ace | Bid made, long | Ace better | Long better | Coach's plan |");
    println!("|---|---|---|---|---|---|---|---|---|");
    for (k, p) in &all {
        let n = p.n as f64;
        println!(
            "| {k} | {} | {:.2} | {:.2} | {:.0}% | {:.0}% | {:.0}% | {:.0}% | {:.0}% |",
            p.n,
            p.void / n,
            p.keep / n,
            100.0 * p.void_made / n,
            100.0 * p.keep_made / n,
            100.0 * p.void_better as f64 / n,
            100.0 * p.keep_better as f64 / n,
            100.0 * p.plan_voids as f64 / n
        );
    }
}

fn one(seed: u64, out: &mut BTreeMap<String, Pair>) {
    let mut game = Game::with_dealer(Rules::default(), seed, Seat::EAST); // South speaks first
    let obs = game.observe(Seat::SOUTH);
    let Some((strain, count)) = Strain::ALL.into_iter().filter(|s| s.trump().is_some()).map(|s| (s, fh_ai::thumb::count(&obs, s).0)).max_by(|a, b| a.1.total_cmp(&b.1)) else { return };
    if count < 7.0 {
        return;
    }
    let level = if count >= 8.5 { 8 } else { 7 };
    let trump = strain.trump();
    game.apply(Action::Call(Call::Bid(Contract::new(level, strain)))).unwrap();
    for _ in 0..3 {
        game.apply(Action::Call(Call::Pass)).unwrap();
    }
    let thirteen = game.hand(Seat::SOUTH);
    let trumpish = |c: Card| is_trump(c, trump) || c.is_joker();
    let in_suit = |s: Suit| -> Vec<Card> { thirteen.iter().filter(|&c| !trumpish(c) && effective_suit(c, trump) == Some(s)).collect() };
    let sides: Vec<Suit> = Suit::ALL.into_iter().filter(|&s| Some(s) != trump).collect();
    if sides.iter().any(|&s| in_suit(s).is_empty()) {
        return;
    }
    let low_pair = sides.iter().copied().find(|&s| {
        let c = in_suit(s);
        c.len() == 2 && c.iter().all(|c| c.rank() < KING)
    });
    let ace_x = sides.iter().copied().find(|&s| {
        let c = in_suit(s);
        c.len() == 2 && c.iter().any(|c| c.rank() == ACE)
    });
    let (Some(x_suit), Some(a_suit)) = (low_pair, ace_x) else { return };
    let Some(&long_suit) = sides.iter().find(|&&s| s != x_suit && s != a_suit) else { return };
    let long = in_suit(long_suit);
    if long.len() < 3 || long.iter().any(|c| c.rank() == ACE) {
        return;
    }
    let small_beside_ace = *in_suit(a_suit).iter().find(|c| c.rank() != ACE).unwrap();
    let lowest_long = *long.iter().min_by_key(|c| c.rank()).unwrap();
    let ace_plan: CardSet = in_suit(x_suit).into_iter().chain([small_beside_ace]).collect();
    let long_plan: CardSet = in_suit(x_suit).into_iter().chain([lowest_long]).collect();
    let long_len = long.len();
    let others: [CardSet; 4] = std::array::from_fn(|i| game.hand(Seat::new(i)));
    let mut solver = Solver::with_limit(trump, 4_000_000);
    let mut solve = |discards: CardSet| -> Option<u8> {
        let mut hands = others;
        hands[0] = thirteen.minus(discards);
        let t = solver.solve(&Position { hands, trump, trick: Trick::new(Seat::SOUTH) }, Seat::SOUTH.team());
        (!solver.aborted()).then_some(t)
    };
    let (Some(v), Some(k)) = (solve(ace_plan), solve(long_plan)) else { return };
    // Real play runs half a trick below the solver: exactly the bid counts as half made.
    let made = |t: u8| if t > level { 1.0 } else if t == level { 0.5 } else { 0.0 };
    let trumps = thirteen.iter().filter(|&c| trumpish(c)).count();
    let key = format!("{} cards, {} trumps", if long_len >= 5 { "5+".to_string() } else { long_len.to_string() }, trumps);
    for key in [key, "all".to_string()] {
        let e = out.entry(key.to_string()).or_default();
        e.n += 1;
        e.void += v as f64;
        e.keep += k as f64;
        e.void_made += made(v);
        e.keep_made += made(k);
        e.void_better += u32::from(v > k);
        e.keep_better += u32::from(k > v);
        let plan: CardSet = fh_ai::rule_bot::heuristic_discards(thirteen, trump, 3).into_iter().collect();
        e.plan_voids += u32::from(plan.contains(small_beside_ace));
    }
}
