//! `fh-lab plays`: card-play rules of thumb, checked with the solver.
//!
//! The Counter plays whole hands in all four seats. At every decision that fits one of the rules
//! below, the position is solved with every card visible (the real hands), and the rule's card is
//! compared with the best card the rule would rule out. Hindsight (every card visible) favours
//! neither side of a comparison in particular; it measures the cards' potential, as findings 18
//! and 30 do.
//!
//! Defence: second hand low; third hand high; cover an honour with an honour; return partner's
//! suit; lead partner's signalled suit; lead a trump against the declarer.
//! The declarer: draw trumps while the opponents have some; then cash side winners; in no trumps,
//! lead the Joker early.

use std::collections::BTreeMap;

use fh_ai::cardplay::is_boss;
use fh_ai::solver::{Position, Solver};
use fh_core::rules::is_trump;
use fh_core::trick::trick_strength;
use fh_core::{effective_suit, Action, Card, CardSet, Game, Phase, Play, Rules, Seat, Suit};

#[derive(Default, Clone, Copy)]
struct Tally {
    n: u32,
    /// Tricks for the player's side: the rule's card less the best other card.
    diff: f64,
    rule_best: u32,
    other_better: u32,
}

pub fn run(deals: u64, seed: u64, threads: u64) {
    let parts: Vec<BTreeMap<String, Tally>> = std::thread::scope(|scope| {
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
    let mut all: BTreeMap<String, Tally> = BTreeMap::new();
    for part in parts {
        for (k, t) in part {
            let e = all.entry(k).or_default();
            e.n += t.n;
            e.diff += t.diff;
            e.rule_best += t.rule_best;
            e.other_better += t.other_better;
        }
    }
    println!("Card-play rules against the best card they rule out ({deals} hands; solver, every card visible)\n");
    println!("| Rule | Decisions | Tricks gained by the rule | Rule's card best (or tied) | Another card better |");
    println!("|---|---|---|---|---|");
    for (k, t) in &all {
        let n = t.n as f64;
        if t.n < 30 {
            continue;
        }
        println!("| {k} | {} | {:+.3} | {:.0}% | {:.0}% |", t.n, t.diff / n, 100.0 * t.rule_best as f64 / n, 100.0 * t.other_better as f64 / n);
    }
}

fn record(out: &mut BTreeMap<String, Tally>, key: String, rule: u8, other: u8) {
    let e = out.entry(key).or_default();
    e.n += 1;
    e.diff += rule as f64 - other as f64;
    e.rule_best += u32::from(rule >= other);
    e.other_better += u32::from(other > rule);
}

fn one(seed: u64, out: &mut BTreeMap<String, Tally>) {
    let mut game = Game::new(Rules::default(), seed);
    let mut bots: [Box<dyn fh_ai::Bot>; 4] = std::array::from_fn(|i| crate::make_bot("counter", seed.wrapping_mul(4).wrapping_add(i as u64)));
    let mut solver_cache: Option<Solver> = None;
    // Per hand: the first suit each defender led, and each player's first throw-off (the signal).
    let mut first_led: [Option<Suit>; 4] = [None; 4];
    let mut signal: [Option<Suit>; 4] = [None; 4];
    loop {
        if game.phase() == Phase::GameOver || !game.history().is_empty() {
            return;
        }
        let seat = game.to_act().unwrap();
        let obs = game.observe(seat);
        let legal = game.legal_actions();
        if obs.phase == Phase::Playing && legal.len() > 1 {
            let (declarer, contract) = obs.contract.unwrap();
            let trump = contract.trump();
            let solver = solver_cache.get_or_insert_with(|| Solver::with_limit(trump, 4_000_000));
            let hands: [CardSet; 4] = std::array::from_fn(|i| game.hand(Seat::new(i)));
            let trick = obs.current_trick.clone().unwrap();
            let values = solver.analyze(&Position { hands, trump, trick: trick.clone() }, seat.team());
            if !solver.aborted() {
                let value = |c: Card| values.iter().filter(|(p, _)| p.card == c).map(|&(_, v)| v).max();
                let best_of = |cards: &[Card]| cards.iter().filter_map(|&c| value(c)).max();
                let cards: Vec<Card> = legal.iter().filter_map(|a| if let Action::Play(p) = a { Some(p.card) } else { None }).collect();
                let trumpish = |c: Card| is_trump(c, trump) || c.is_joker();
                let defending = seat.team() != declarer.team();
                let played = trick.cards().len();
                let led = trick.led_suit();
                let seen: CardSet = obs.played().union(obs.hand);
                let low_first = |v: &mut Vec<Card>, s: Suit| v.sort_by_key(|&c| trick_strength(c, s, trump));
                // Following suit with a choice of low and high.
                if let Some(s) = led {
                    let mut follow: Vec<Card> = cards.iter().copied().filter(|&c| effective_suit(c, trump) == Some(s)).collect();
                    low_first(&mut follow, s);
                    if defending && follow.len() >= 2 {
                        let winner = trick.winning_card(trump).unwrap();
                        let to_beat = trick_strength(winner, s, trump);
                        let beats: Vec<Card> = follow.iter().copied().filter(|&c| trick_strength(c, s, trump) > to_beat).collect();
                        let low = follow[0];
                        let leader_is_declarer_side = trick.leader().team() == declarer.team();
                        // Second hand low: the declarer's side led, you can beat it with an honour.
                        if played == 1 && leader_is_declarer_side && beats.iter().any(|c| c.rank() >= 12 || trumpish(*c)) && !beats.contains(&low) {
                            // Against the cheapest card that wins (the best of several, picked with
                            // hindsight, would favour winning unfairly).
                            if let (Some(r), Some(o)) = (value(low), value(beats[0])) {
                                record(out, "Defence: second hand low (vs the cheapest card that wins)".into(), r, o);
                            }
                        }
                        // Cover an honour with an honour: the declarer's side led a J, Q or K.
                        let led_card = trick.cards()[0];
                        if played == 1 && leader_is_declarer_side && (11..=13).contains(&led_card.rank()) && !trumpish(led_card) && !beats.is_empty() && !beats.contains(&low) {
                            if let (Some(r), Some(o)) = (best_of(&beats[..1]), value(low)) {
                                record(out, "Defence: cover an honour with an honour (vs ducking)".into(), r, o);
                            }
                        }
                        // Third hand high: partner led, the declarer's side is winning, you can beat it.
                        let partner_led = trick.leader() == seat.partner();
                        let declarer_side_winning = trick.winner(trump).is_some_and(|w| w.team() == declarer.team());
                        if played == 2 && partner_led && declarer_side_winning && !beats.is_empty() && !beats.contains(&low) {
                            if let (Some(r), Some(o)) = (value(beats[0]), value(low)) {
                                record(out, "Defence: third hand high (cheapest card that wins, vs playing low)".into(), r, o);
                            }
                        }
                    }
                }
                // Leading.
                if played == 0 {
                    let by_suit = |s: Suit| -> Vec<Card> { cards.iter().copied().filter(|&c| !trumpish(c) && effective_suit(c, trump) == Some(s)).collect() };
                    let trumps_held: Vec<Card> = cards.iter().copied().filter(|&c| trumpish(c)).collect();
                    let non_trumps: Vec<Card> = cards.iter().copied().filter(|&c| !trumpish(c)).collect();
                    if defending {
                        if let Some(s) = first_led[seat.partner().index()].filter(|&s| Some(s) != trump) {
                            let mine = by_suit(s);
                            let others: Vec<Card> = cards.iter().copied().filter(|c| !mine.contains(c)).collect();
                            if let (Some(r), Some(o)) = (best_of(&mine), best_of(&others)) {
                                record(out, "Defence: return partner's first-led suit (vs any other lead)".into(), r, o);
                            }
                        }
                        if let Some(s) = signal[seat.partner().index()] {
                            let mine = by_suit(s);
                            let others: Vec<Card> = cards.iter().copied().filter(|c| !mine.contains(c)).collect();
                            if let (Some(r), Some(o)) = (best_of(&mine), best_of(&others)) {
                                record(out, "Defence: lead partner's signalled suit (vs any other lead)".into(), r, o);
                            }
                        }
                        if trump.is_some() {
                            if let (Some(r), Some(o)) = (best_of(&trumps_held), best_of(&non_trumps)) {
                                record(out, "Defence: lead a trump (vs the best side lead)".into(), r, o);
                            }
                        }
                    } else if seat == declarer {
                        if trump.is_some() {
                            // The true count (every hand visible): trumps the opponents hold.
                            let opp_trumps = Seat::ALL.iter().filter(|o| o.team() != seat.team()).map(|o| hands[o.index()].iter().filter(|&c| trumpish(c)).count()).sum::<usize>();
                            if opp_trumps > 0 {
                                let key = format!("Declarer: draw trumps (vs the best side lead), opponents holding {}", if opp_trumps >= 3 { "3+".to_string() } else { opp_trumps.to_string() });
                                if let (Some(r), Some(o)) = (best_of(&trumps_held), best_of(&non_trumps)) {
                                    record(out, key, r, o);
                                }
                            } else {
                                let boss: Vec<Card> = non_trumps.iter().copied().filter(|&c| is_boss(c, seen, trump)).collect();
                                let rest: Vec<Card> = cards.iter().copied().filter(|c| !boss.contains(c)).collect();
                                if let (Some(r), Some(o)) = (best_of(&boss), best_of(&rest)) {
                                    record(out, "Declarer: opponents out of trumps, cash a side winner (vs anything else)".into(), r, o);
                                }
                            }
                        } else if cards.contains(&Card::JOKER) {
                            let trick_no = obs.tricks_won[0] + obs.tricks_won[1];
                            let joker = best_of(&[Card::JOKER]);
                            let rest: Vec<Card> = cards.iter().copied().filter(|&c| !c.is_joker()).collect();
                            let key = format!("Declarer, no trumps: lead the Joker at trick {} (vs the best other lead)", if trick_no < 3 { "1–3" } else { "4+" });
                            if let (Some(r), Some(o)) = (joker, best_of(&rest)) {
                                record(out, key, r, o);
                            }
                        }
                    }
                }
            }
        }
        let action = bots[seat.index()].choose(&obs, &legal);
        if let Action::Play(Play { card, .. }) = action {
            let trick = obs.current_trick.clone().unwrap();
            let trump = obs.contract.and_then(|(_, c)| c.trump());
            if trick.cards().is_empty() {
                if first_led[seat.index()].is_none() {
                    first_led[seat.index()] = effective_suit(card, trump);
                }
            } else if let Some(led) = trick.led_suit() {
                let follows = effective_suit(card, trump) == Some(led);
                let trumped = is_trump(card, trump) || card.is_joker();
                if !follows && !trumped && signal[seat.index()].is_none() {
                    signal[seat.index()] = effective_suit(card, trump);
                }
            }
        }
        game.apply(action).unwrap();
    }
}
