//! `fh-lab declarer`: two questions for the declarer, answered with the solver.
//!
//! South speaks first and, with a biddable hand (count 7 or more in its best suit), declares at
//! the level the rule of thumb gives (7, 8 from 8½, 9 from 10). Every card is visible to the
//! solver, so the numbers measure the cards' potential, not guesswork.
//!
//! The kitty: every way of putting back three non-trump cards (and one that puts back the lowest
//! trump), solved; what do the best sets have in common?
//!
//! The first lead: after the usual discards, every opening lead, solved; which kind of lead works
//! best, by trump holding?

use std::collections::BTreeMap;

use fh_ai::rule_bot::heuristic_discards;
use fh_ai::solver::{Position, Solver};
use fh_core::cards::{ACE, KING, QUEEN};
use fh_core::rules::is_trump;
use fh_core::{effective_suit, Action, Call, Card, CardSet, Contract, Game, Play, Rules, Seat, Strain, Suit, Trick};

#[derive(Default)]
struct Stats {
    n: f64,
    tricks: f64,
    made: f64,
}

impl Stats {
    fn add(&mut self, tricks: u8, level: u8) {
        self.n += 1.0;
        self.tricks += tricks as f64;
        // Real play runs half a trick below the solver: exactly the bid counts as half made.
        self.made += if tricks > level { 1.0 } else if tricks == level { 0.5 } else { 0.0 };
    }
}

/// Two choices compared on the same deals: average tricks of each, and how often each wins.
#[derive(Default)]
struct Pair {
    n: f64,
    first: f64,
    second: f64,
    first_better: f64,
    second_better: f64,
}

/// Results for one deal: kitty comparisons and leads, keyed by category.
#[derive(Default)]
struct DealResult {
    kitty: BTreeMap<String, Pair>,
    leads: BTreeMap<(String, String), Stats>,
    /// Each kind of lead against the top trump, on deals where both are possible.
    paired_leads: BTreeMap<(String, String), Pair>,
    deals: usize,
}

fn merge_pair(t: &mut Pair, s: &Pair) {
    t.n += s.n;
    t.first += s.first;
    t.second += s.second;
    t.first_better += s.first_better;
    t.second_better += s.second_better;
}

/// `nt`: South declares no trumps instead (a no-trump count of 7 or more): the kitty questions
/// only, since there are no trumps to lead.
pub fn run(deals: u64, seed: u64, threads: u64, leads_only: bool, nt: bool) {
    let parts: Vec<DealResult> = std::thread::scope(|scope| {
        let handles: Vec<_> = (0..threads)
            .map(|t| {
                scope.spawn(move || {
                    let mut out = DealResult::default();
                    let mut d = t;
                    while d < deals {
                        one(seed + d, &mut out, leads_only, nt);
                        d += threads;
                    }
                    out
                })
            })
            .collect();
        handles.into_iter().map(|h| h.join().unwrap()).collect()
    });
    let mut all = DealResult::default();
    for part in parts {
        all.deals += part.deals;
        for (k, s) in part.kitty {
            merge_pair(all.kitty.entry(k).or_default(), &s);
        }
        for (k, s) in part.paired_leads {
            merge_pair(all.paired_leads.entry(k).or_default(), &s);
        }
        for (k, s) in part.leads {
            let t = all.leads.entry(k).or_default();
            t.n += s.n;
            t.tricks += s.tricks;
            t.made += s.made;
        }
    }
    println!("{} deals where South declares\n", all.deals);
    println!("## The kitty: the best discards of one kind against the best of another, on the same deals\n");
    println!("| Question | Deals | Tricks (first) | Tricks (second) | First better | Second better |");
    println!("|---|---|---|---|---|---|");
    for (k, s) in &all.kitty {
        if s.n < 30.0 {
            continue;
        }
        println!(
            "| {k} | {:.0} | {:.2} | {:.2} | {:.0}% | {:.0}% |",
            s.n,
            s.first / s.n,
            s.second / s.n,
            100.0 * s.first_better / s.n,
            100.0 * s.second_better / s.n
        );
    }
    println!("\n## The first lead against leading your top trump, on the same deals: tricks gained (how often better / worse)\n");
    let pholdings: Vec<String> = all.paired_leads.keys().map(|(h, _)| h.clone()).collect::<std::collections::BTreeSet<_>>().into_iter().collect();
    let pkinds: Vec<String> = all.paired_leads.keys().map(|(_, k)| k.clone()).collect::<std::collections::BTreeSet<_>>().into_iter().collect();
    println!("| Holding | {} |", pkinds.join(" | "));
    println!("|---|{}", "---|".repeat(pkinds.len()));
    for h in &pholdings {
        let cells: Vec<String> = pkinds
            .iter()
            .map(|k| match all.paired_leads.get(&(h.clone(), k.clone())) {
                Some(s) if s.n >= 30.0 => format!(
                    "{:+.2} ({:.0}% / {:.0}%, n={:.0})",
                    (s.first - s.second) / s.n,
                    100.0 * s.first_better / s.n,
                    100.0 * s.second_better / s.n,
                    s.n
                ),
                _ => "—".into(),
            })
            .collect();
        println!("| {h} | {} |", cells.join(" | "));
    }
    println!("\n## The first lead, by trump holding, unpaired: average tricks (made %) for each kind of lead\n");
    let holdings: Vec<String> = all.leads.keys().map(|(h, _)| h.clone()).collect::<std::collections::BTreeSet<_>>().into_iter().collect();
    let kinds: Vec<String> = all.leads.keys().map(|(_, k)| k.clone()).collect::<std::collections::BTreeSet<_>>().into_iter().collect();
    println!("| Holding | {} |", kinds.join(" | "));
    println!("|---|{}", "---|".repeat(kinds.len()));
    for h in &holdings {
        let cells: Vec<String> = kinds
            .iter()
            .map(|k| match all.leads.get(&(h.clone(), k.clone())) {
                Some(s) if s.n >= 30.0 => format!("{:.2} ({:.0}%, n={:.0})", s.tricks / s.n, 100.0 * s.made / s.n, s.n),
                _ => "—".into(),
            })
            .collect();
        println!("| {h} | {} |", cells.join(" | "));
    }
}

fn one(seed: u64, out: &mut DealResult, leads_only: bool, nt: bool) {
    let rules = Rules::default();
    let mut game = Game::with_dealer(rules, seed, Seat::EAST); // South speaks first
    let obs = game.observe(Seat::SOUTH);
    let best_suit = Strain::ALL
        .into_iter()
        .filter(|s| s.trump().is_some())
        .map(|s| (s, fh_ai::thumb::count(&obs, s).0))
        .max_by(|a, b| a.1.total_cmp(&b.1));
    let Some((strain, count)) = (if nt { Some((Strain::NoTrumps, fh_ai::thumb::count(&obs, Strain::NoTrumps).0)) } else { best_suit }) else {
        return;
    };
    if count < 7.0 {
        return;
    }
    let level = if count >= 10.0 { 9 } else if count >= 8.5 { 8 } else { 7 };
    let trump = strain.trump();
    // The lead study needs a trump suit; in no trumps only the kitty is studied.
    let t = trump.unwrap_or(Suit::Spades);
    game.apply(Action::Call(Call::Bid(Contract::new(level, strain)))).unwrap();
    for _ in 0..3 {
        game.apply(Action::Call(Call::Pass)).unwrap();
    }
    out.deals += 1;
    let thirteen = game.hand(Seat::SOUTH);
    let others: [CardSet; 4] = std::array::from_fn(|i| game.hand(Seat::new(i)));
    let mut solver = Solver::with_limit(trump, 4_000_000);
    let solve = |solver: &mut Solver, south: CardSet, trick: Trick| -> Option<u8> {
        let mut hands = others;
        hands[0] = south;
        let tricks = solver.solve(&Position { hands, trump, trick }, Seat::SOUTH.team());
        (!solver.aborted()).then_some(tricks)
    };
    let trumpish = |c: Card| is_trump(c, trump) || c.is_joker();
    let side = |suit: Suit, hand: CardSet| hand.iter().filter(|&c| !trumpish(c) && effective_suit(c, trump) == Some(suit)).count();

    // ---- The kitty ----------------------------------------------------------------------------
    // In no trumps aces are kept (sure winners), which keeps the number of sets manageable.
    let side_cards: Vec<Card> = thirteen.iter().filter(|&c| !trumpish(c) && !(nt && c.rank() == ACE)).collect();
    let mut sets: Vec<Vec<Card>> = Vec::new();
    for i in 0..side_cards.len() {
        for j in i + 1..side_cards.len() {
            for k in j + 1..side_cards.len() {
                sets.push(vec![side_cards[i], side_cards[j], side_cards[k]]);
            }
        }
    }
    if sets.is_empty() || sets.len() > 120 {
        return;
    }
    let plan_cards: Vec<Card> = heuristic_discards(thirteen, trump, 3);
    let plan: CardSet = plan_cards.iter().copied().collect();
    // A void-first plan: of the sets that keep every ace, the one emptying the most side suits,
    // then the lowest cards. Compared with the usual plan on the same deal.
    let voids_made = |set: &[Card]| {
        let hand = thirteen.minus(set.iter().copied().collect());
        Suit::ALL.iter().filter(|&&s| Some(s) != trump && side(s, hand) == 0).count()
    };
    let void_first = sets
        .iter()
        .filter(|set| set.iter().all(|c| c.rank() != ACE))
        .max_by_key(|set| (voids_made(set), std::cmp::Reverse(set.iter().map(|c| c.rank() as u32).sum::<u32>())));
    if let Some(vf) = void_first {
        let vf_set: CardSet = vf.iter().copied().collect();
        if vf_set != plan {
            if let (Some(x), Some(y)) =
                (solve(&mut solver, thirteen.minus(vf_set), Trick::new(Seat::SOUTH)), solve(&mut solver, thirteen.minus(plan), Trick::new(Seat::SOUTH)))
            {
                let pair = out.kitty.entry("Void-first plan, or the usual plan? (where they differ)".into()).or_default();
                pair.n += 1.0;
                pair.first += x as f64;
                pair.second += y as f64;
                pair.first_better += f64::from(x > y);
                pair.second_better += f64::from(y > x);
            }
        }
    }
    // In no trumps: the plan the bots used before the no-trump rule (shortening toward voids, as in a
    // suit bid), against the current plan.
    if nt {
        let old: CardSet = old_suit_style_discards(thirteen).into_iter().collect();
        if old != plan {
            if let (Some(x), Some(y)) = (solve(&mut solver, thirteen.minus(plan), Trick::new(Seat::SOUTH)), solve(&mut solver, thirteen.minus(old), Trick::new(Seat::SOUTH))) {
                let pair = out.kitty.entry("No-trump plan, or the old suit-style plan? (where they differ)".into()).or_default();
                pair.n += 1.0;
                pair.first += x as f64;
                pair.second += y as f64;
                pair.first_better += f64::from(x > y);
                pair.second_better += f64::from(y > x);
            }
        }
    }
    // In no trumps: a plan that never empties a suit (so every suit can still be stopped), against
    // the usual plan.
    if nt {
        let no_void = sets
            .iter()
            .filter(|set| set.iter().all(|c| c.rank() != ACE))
            .min_by_key(|set| (voids_made(set), set.iter().map(|c| c.rank() as u32).sum::<u32>()));
        if let Some(nv) = no_void {
            let nv_set: CardSet = nv.iter().copied().collect();
            if nv_set != plan {
                if let (Some(x), Some(y)) =
                    (solve(&mut solver, thirteen.minus(nv_set), Trick::new(Seat::SOUTH)), solve(&mut solver, thirteen.minus(plan), Trick::new(Seat::SOUTH)))
                {
                    let pair = out.kitty.entry("No-void plan, or the usual plan? (where they differ)".into()).or_default();
                    pair.n += 1.0;
                    pair.first += x as f64;
                    pair.second += y as f64;
                    pair.first_better += f64::from(x > y);
                    pair.second_better += f64::from(y > x);
                }
            }
        }
    }
    if leads_only {
        sets.clear();
    }
    let voids = |hand: CardSet| Suit::ALL.iter().filter(|&&s| Some(s) != trump && side(s, hand) == 0).count();
    let voids_before = voids(thirteen);
    let guarded_kings = |hand: CardSet| {
        Suit::ALL.iter().filter(|&&s| Some(s) != trump && hand.contains(Card::new(s, KING)) && !hand.contains(Card::new(s, ACE)) && side(s, hand) >= 2).count()
    };
    let aces = |hand: CardSet| Suit::ALL.iter().filter(|&&s| Some(s) != trump && hand.contains(Card::new(s, ACE))).count();
    let mut best_by: BTreeMap<&str, u8> = BTreeMap::new();
    let mut note = |key: &'static str, tricks: u8| {
        let e = best_by.entry(key).or_insert(0);
        *e = (*e).max(tricks);
    };
    let mut plan_tricks = None;
    for set in &sets {
        let discards: CardSet = set.iter().copied().collect();
        let hand = thirteen.minus(discards);
        let Some(tricks) = solve(&mut solver, hand, Trick::new(Seat::SOUTH)) else { continue };
        note("any", tricks);
        note(if voids(hand) > voids_before { "makes a void" } else { "makes no void" }, tricks);
        note(if guarded_kings(hand) < guarded_kings(thirteen) { "breaks a guarded king" } else { "keeps guarded kings" }, tricks);
        note(if aces(hand) < aces(thirteen) { "puts back an ace" } else { "keeps aces" }, tricks);
        if discards == plan {
            plan_tricks = Some(tricks);
        }
    }
    // One set that puts back the lowest trump with the plan's two lowest side cards.
    let lowest_trump = thirteen.iter().filter(|&c| trumpish(c)).min_by_key(|&c| fh_core::trick::trick_strength(c, t, trump));
    if let Some(lt) = lowest_trump.filter(|_| !leads_only && !nt) {
        let discards: CardSet = std::iter::once(lt).chain(plan_cards.iter().copied().filter(|&c| !trumpish(c)).take(2)).collect();
        if discards.len() == 3 {
            if let Some(tricks) = solve(&mut solver, thirteen.minus(discards), Trick::new(Seat::SOUTH)) {
                note("puts back a trump", tricks);
            }
        }
    }
    if let Some(p) = plan_tricks {
        note("the coach's usual plan", p);
    }
    let trump_count = thirteen.iter().filter(|&c| trumpish(c)).count();
    let bucket = match trump_count {
        0..=5 => "5 or fewer trumps",
        6 => "6 trumps",
        _ => "7+ trumps",
    };
    let questions: [(&str, &str, &str); 5] = [
        ("Make a void, or not?", "makes a void", "makes no void"),
        ("Keep a guarded king, or not?", "keeps guarded kings", "breaks a guarded king"),
        ("Keep the aces, or not?", "keeps aces", "puts back an ace"),
        ("Keep every trump, or put one back?", "any", "puts back a trump"),
        ("Best possible, or the coach's plan?", "any", "the coach's usual plan"),
    ];
    for (question, a, b) in questions {
        if let (Some(&x), Some(&y)) = (best_by.get(a), best_by.get(b)) {
            for key in [format!("{question} (all)"), format!("{question} ({bucket})")] {
                let pair = out.kitty.entry(key).or_default();
                pair.n += 1.0;
                pair.first += x as f64;
                pair.second += y as f64;
                pair.first_better += f64::from(x > y);
                pair.second_better += f64::from(y > x);
            }
        }
    }

    if nt {
        return;
    }

    // ---- The first lead, after the usual discards ----------------------------------------------
    let hand = thirteen.minus(plan);
    let trumps: Vec<Card> = hand.iter().filter(|&c| trumpish(c)).collect();
    let tops = [Card::JOKER, Card::new(t, fh_core::cards::JACK), Card::new(t.same_color(), fh_core::cards::JACK), Card::new(t, ACE)];
    let missing_tops = tops.iter().filter(|c| !hand.contains(**c)).count();
    let holding = format!(
        "{} trumps, missing {} of Joker/bowers/A",
        match trumps.len() {
            0..=5 => "5 or fewer",
            6 => "6",
            _ => "7+",
        },
        match missing_tops {
            0 | 1 => "0–1",
            2 => "2",
            _ => "3–4",
        }
    );
    let top_trump = trumps.iter().copied().max_by_key(|&c| fh_core::trick::trick_strength(c, t, trump));
    let mut by_kind: BTreeMap<&str, u8> = BTreeMap::new();
    let low_trump = trumps.iter().copied().min_by_key(|&c| fh_core::trick::trick_strength(c, t, trump));
    for card in hand.iter() {
        let kind = if trumpish(card) {
            if Some(card) == top_trump {
                "top trump"
            } else if Some(card) == low_trump {
                "low trump"
            } else {
                continue;
            }
        } else {
            let suit = effective_suit(card, trump).expect("a side card");
            let len = side(suit, hand);
            if len == 1 {
                "singleton"
            } else if card.rank() == ACE {
                "side ace"
            } else if card.rank() == KING && hand.contains(Card::new(suit, QUEEN)) {
                "K of K-Q"
            } else if hand.iter().filter(|&c| !trumpish(c) && effective_suit(c, trump) == Some(suit)).all(|c| c.rank() >= card.rank()) {
                "low side card"
            } else {
                continue;
            }
        };
        let trick = Trick::from_plays(Seat::SOUTH, &[Play::card(card)], trump);
        let mut rest = hand;
        rest.remove(card);
        if let Some(tricks) = solve(&mut solver, rest, trick) {
            out.leads.entry((holding.clone(), kind.to_string())).or_default().add(tricks, level);
            let best = by_kind.entry(kind).or_insert(0);
            *best = (*best).max(tricks);
        }
    }
    if let Some(&top) = by_kind.get("top trump") {
        for (&kind, &tricks) in &by_kind {
            if kind == "top trump" {
                continue;
            }
            for h in [holding.clone(), "all holdings".to_string()] {
                let pair = out.paired_leads.entry((h, kind.to_string())).or_default();
                pair.n += 1.0;
                pair.first += tricks as f64;
                pair.second += top as f64;
                pair.first_better += f64::from(tricks > top);
                pair.second_better += f64::from(top > tricks);
            }
        }
    }
}

/// The kitty plan the bots used in no trumps before it had its own rule: the suit-bid heuristic
/// (keep aces and guarded kings, shorten toward voids), for comparison.
fn old_suit_style_discards(hand: CardSet) -> Vec<Card> {
    let value = |card: Card, hand: CardSet| -> i32 {
        if card.is_joker() {
            return 1000;
        }
        let in_suit = hand.iter().filter(|h| h.suit() == card.suit()).count();
        let mut v = card.rank() as i32;
        if card.rank() == ACE {
            v += 40;
        } else if card.rank() == fh_core::cards::KING && in_suit >= 2 {
            v += 15;
        }
        if in_suit <= 2 && card.rank() != ACE {
            v -= 8;
        }
        v
    };
    let mut hand = hand;
    let mut out = Vec::new();
    for _ in 0..3 {
        let card = hand.iter().min_by_key(|&c| (value(c, hand), c.id())).expect("cards");
        hand.remove(card);
        out.push(card);
    }
    out
}
