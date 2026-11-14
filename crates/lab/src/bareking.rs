//! `fh-lab bareking`: after the kitty, a side king with small cards is the only suit you can shorten
//! (no other side suit of one to three cards). Keep the guarded king, or put back its small cards
//! and leave the king alone (a singleton)? And, where it can be done, emptying the suit, for
//! comparison (finding 38 measured that against keeping K-x). Deals are split by whether another
//! side suit has four cards (three discards could cut it to a singleton) or nothing else can be
//! shortened at all (every other side suit empty or five or more).
//!
//! South declares at the count's level (as in `fh-lab kxkitty`). The king's suit has no ace and no
//! other card above the 10. Plans, each solved on the same deal with every card visible:
//! - keep: put back the three lowest other side cards (never an ace);
//! - bare: put back the king's small cards, then the lowest other side cards to make three;
//! - void (K-x and K-x-x only): put back the whole suit, then the lowest other side card.

use std::collections::BTreeMap;

use fh_ai::solver::{Position, Solver};
use fh_core::cards::{ACE, KING};
use fh_core::rules::is_trump;
use fh_core::{effective_suit, Action, Call, Card, CardSet, Contract, Game, Rules, Seat, Strain, Suit, Trick};

#[derive(Default, Clone, Copy)]
struct Tally {
    n: u32,
    keep: f64,
    bare: f64,
    keep_made: f64,
    bare_made: f64,
    bare_better: u32,
    keep_better: u32,
    /// Deals where voiding was possible, and its totals.
    void_n: u32,
    void: f64,
    void_made: f64,
    /// What the coach's kitty plan does: keeps the king guarded, leaves it bare, or voids the suit.
    plan: [u32; 3],
}

pub fn run(deals: u64, seed: u64, threads: u64) {
    let parts: Vec<BTreeMap<(bool, u8, String), Tally>> = std::thread::scope(|scope| {
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
    let mut all: BTreeMap<(bool, u8, String), Tally> = BTreeMap::new();
    for part in parts {
        for (k, p) in part {
            let e = all.entry(k).or_default();
            e.n += p.n;
            e.keep += p.keep;
            e.bare += p.bare;
            e.keep_made += p.keep_made;
            e.bare_made += p.bare_made;
            e.bare_better += p.bare_better;
            e.keep_better += p.keep_better;
            e.void_n += p.void_n;
            e.void += p.void;
            e.void_made += p.void_made;
            for i in 0..3 {
                e.plan[i] += p.plan[i];
            }
        }
    }
    println!("A side king with small cards is the only suit you can shorten: keep it guarded, or leave it bare? ({deals} deals)\n");
    let mut section = None;
    for ((four, len, trumps), p) in &all {
        if section != Some(*four) {
            section = Some(*four);
            println!(
                "
**{}**
",
                if *four { "Another side suit of four cards (three discards could make it a singleton)" } else { "Nothing else to shorten: every other side suit empty or five or more" }
            );
            println!("| King's suit | Trumps | Deals | Tricks: keep / bare / void | Bid made: keep / bare / void | Bare better / keep better | Coach's plan: keep / bare / void |");
            println!("|---|---|---|---|---|---|---|");
        }
        let n = p.n as f64;
        let suit = ["", "", "K-x", "K-x-x", "K-x-x-x"][*len as usize];
        let (void, void_made) = if p.void_n > 0 {
            (format!("{:.2}", p.void / p.void_n as f64), format!("{:.0}%", 100.0 * p.void_made / p.void_n as f64))
        } else {
            ("—".into(), "—".into())
        };
        println!(
            "| {suit} | {trumps} | {} | {:.2} / {:.2} / {void} | {:.0}% / {:.0}% / {void_made} | {:.0}% / {:.0}% | {:.0}% / {:.0}% / {:.0}% |",
            p.n,
            p.keep / n,
            p.bare / n,
            100.0 * p.keep_made / n,
            100.0 * p.bare_made / n,
            100.0 * p.bare_better as f64 / n,
            100.0 * p.keep_better as f64 / n,
            100.0 * p.plan[0] as f64 / n,
            100.0 * p.plan[1] as f64 / n,
            100.0 * p.plan[2] as f64 / n,
        );
    }
}

fn one(seed: u64, out: &mut BTreeMap<(bool, u8, String), Tally>) {
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
    // A king with two to four cards in all, no ace, nothing else above the 10.
    let kings: Vec<Suit> = sides
        .iter()
        .copied()
        .filter(|&s| {
            let cards = in_suit(s);
            (2..=4).contains(&cards.len()) && cards.iter().any(|c| c.rank() == KING) && cards.iter().all(|c| c.rank() == KING || c.rank() <= 10)
        })
        .collect();
    if kings.len() != 1 {
        return;
    }
    let k_suit = kings[0];
    // The only suit you can shorten: no other side suit of one to three cards.
    if sides.iter().any(|&s| s != k_suit && (1..=3).contains(&in_suit(s).len())) {
        return;
    }
    let king_cards = in_suit(k_suit);
    let smalls: Vec<Card> = king_cards.iter().copied().filter(|c| c.rank() != KING).collect();
    let king = king_cards.iter().copied().find(|c| c.rank() == KING).unwrap();
    // The other side cards, lowest first, never an ace.
    let mut spare: Vec<Card> = sides.iter().filter(|&&s| s != k_suit).flat_map(|&s| in_suit(s)).filter(|c| c.rank() != ACE).collect();
    spare.sort_by_key(|c| c.rank());
    if spare.len() < 3 {
        return;
    }
    let keep: CardSet = spare.iter().copied().take(3).collect();
    let bare: CardSet = smalls.iter().copied().chain(spare.iter().copied()).take(3).collect();
    let void: Option<CardSet> = (king_cards.len() <= 3).then(|| king_cards.iter().copied().chain(spare.iter().copied()).take(3).collect());
    let others: [CardSet; 4] = std::array::from_fn(|i| game.hand(Seat::new(i)));
    let mut solver = Solver::with_limit(trump, 4_000_000);
    let mut solve = |discards: CardSet| -> Option<u8> {
        let mut hands = others;
        hands[0] = thirteen.minus(discards);
        let t = solver.solve(&Position { hands, trump, trick: Trick::new(Seat::SOUTH) }, Seat::SOUTH.team());
        (!solver.aborted()).then_some(t)
    };
    let (Some(k), Some(b)) = (solve(keep), solve(bare)) else { return };
    let v = match void {
        Some(set) => match solve(set) {
            Some(t) => Some(t),
            None => return,
        },
        None => None,
    };
    // Real play runs half a trick below the solver: exactly the bid counts as half made.
    let made = |t: u8| if t > level { 1.0 } else if t == level { 0.5 } else { 0.0 };
    let trumps = thirteen.iter().filter(|&c| trumpish(c)).count();
    let bin = match trumps {
        0..=5 => "5 or fewer",
        6 => "6",
        _ => "7 or more",
    };
    let plan: CardSet = fh_ai::rule_bot::heuristic_discards(thirteen, trump, 3).into_iter().collect();
    let left = king_cards.iter().filter(|&&c| !plan.contains(c)).count();
    let plan_kind = if left == 0 {
        2
    } else if left == 1 && !plan.contains(king) {
        1
    } else {
        0
    };
    let four = sides.iter().any(|&s| s != k_suit && in_suit(s).len() == 4);
    for key in [bin, "all"] {
        let e = out.entry((four, king_cards.len() as u8, key.to_string())).or_default();
        e.n += 1;
        e.keep += k as f64;
        e.bare += b as f64;
        e.keep_made += made(k);
        e.bare_made += made(b);
        e.bare_better += u32::from(b > k);
        e.keep_better += u32::from(k > b);
        if let Some(v) = v {
            e.void_n += 1;
            e.void += v as f64;
            e.void_made += made(v);
        }
        e.plan[plan_kind] += 1;
    }
}
