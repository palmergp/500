//! `fh-lab habits`: what the strongest bot (The Counter, in all four seats) does in recurring
//! situations, tallied over many hands. Source material for docs/rules-of-thumb.md.

use crate::make_bot;
use fh_ai::cardplay::{is_boss, trumps};
use fh_core::cards::ACE;
use fh_core::rules::is_trump;
use fh_core::trick::trick_strength;
use fh_core::{effective_suit, Action, Call, Card, Event, Game, Phase, Rules, Seat};
use std::collections::BTreeMap;

#[derive(Default)]
struct Tally(BTreeMap<String, BTreeMap<String, u64>>);

impl Tally {
    fn add(&mut self, situation: impl Into<String>, choice: impl Into<String>) {
        *self.0.entry(situation.into()).or_default().entry(choice.into()).or_default() += 1;
    }
    fn merge(&mut self, other: Tally) {
        for (s, choices) in other.0 {
            for (c, n) in choices {
                *self.0.entry(s.clone()).or_default().entry(c).or_default() += n;
            }
        }
    }
}

pub fn run(deals: u64, seed: u64, threads: u64, player: &str) {
    let tallies: Vec<Tally> = std::thread::scope(|scope| {
        let handles: Vec<_> = (0..threads)
            .map(|t| {
                scope.spawn(move || {
                    let mut tally = Tally::default();
                    let mut d = t;
                    while d < deals {
                        one_hand(seed + d, player, &mut tally);
                        d += threads;
                    }
                    tally
                })
            })
            .collect();
        handles.into_iter().map(|h| h.join().unwrap()).collect()
    });
    let mut total = Tally::default();
    for t in tallies {
        total.merge(t);
    }
    println!("Habits of {player} over {deals} hands (all four seats)\n");
    for (situation, choices) in &total.0 {
        let n: u64 = choices.values().sum();
        let mut sorted: Vec<_> = choices.iter().collect();
        sorted.sort_by(|a, b| b.1.cmp(a.1));
        let parts: Vec<String> = sorted.iter().map(|(c, k)| format!("{c} {:.0}%", 100.0 * **k as f64 / n as f64)).collect();
        println!("{situation} (n={n}): {}", parts.join(", "));
    }
}

fn one_hand(seed: u64, player: &str, tally: &mut Tally) {
    let mut game = Game::with_dealer(Rules::default(), seed, Seat::new((seed % 4) as usize));
    let mut bots: [Box<dyn fh_ai::Bot>; 4] = std::array::from_fn(|i| make_bot(player, seed * 4 + i as u64));
    let start = game.history().len();
    let mut first_defence_lead = true;
    while game.history().len() == start && game.phase() != Phase::GameOver {
        let seat = game.to_act().unwrap();
        let obs = game.observe(seat);
        let legal = game.legal_actions();
        let action = bots[seat.index()].choose(&obs, &legal);
        if legal.len() > 1 {
            observe(&obs, action, &legal, tally, &mut first_defence_lead);
        }
        let events = game.apply(action).unwrap();
        if events.iter().any(|e| matches!(e, Event::HandScored { .. })) {
            break;
        }
    }
}

fn observe(obs: &fh_core::Observation, action: Action, legal: &[Action], tally: &mut Tally, first_defence_lead: &mut bool) {
    let Some((declarer, contract)) = obs.contract else {
        if let Action::Call(call) = action {
            let joker = obs.hand.contains(Card::JOKER);
            let kind = match call {
                Call::Pass => "pass",
                Call::Mention(_) => "mention",
                Call::Six(_) => "6-call",
                Call::Bid(_) => "bid",
            };
            if joker {
                tally.add("Auction: holding the Joker, the call was", kind);
            }
        }
        return;
    };
    let trump = contract.trump();
    let me = obs.seat;
    let declaring = declarer.team() == me.team();
    match action {
        Action::Discard(card) => {
            if obs.discards.is_some_and(|d| d.is_empty()) {
                // Judge the full plan by the first discard's suit length.
                let suit = effective_suit(card, trump);
                let len = obs.hand.iter().filter(|&c| effective_suit(c, trump) == suit && !is_trump(c, trump)).count();
                tally.add("Kitty: first discard came from a side suit of length", len.to_string());
            }
        }
        Action::Play(play) => {
            let trick = obs.current_trick.clone().unwrap();
            let seen = obs.played().union(obs.hand).union(obs.discards.unwrap_or_default());
            let card = play.card;
            let kind = if card.is_joker() {
                "the Joker".to_string()
            } else if is_trump(card, trump) {
                if is_boss(card, seen, trump) { "top trump" } else { "low trump" }.to_string()
            } else if card.rank() == ACE || is_boss(card, seen, trump) {
                "side winner".to_string()
            } else {
                "low side card".to_string()
            };
            let contract_kind = if trump.is_some() { "suit" } else { "NT" };
            if trick.is_empty() {
                if obs.tricks.is_empty() && me == declarer {
                    let n = trumps(obs.hand, trump).len();
                    let bucket = match n {
                        0..=3 => "0-3",
                        4 => "4",
                        5 => "5",
                        _ => "6+",
                    };
                    tally.add(format!("Declarer's opening lead ({contract_kind}), holding {bucket} trumps"), kind);
                } else if me == declarer && trump.is_some() {
                    let theirs_out = trumps(fh_core::CardSet::pack(), trump).minus(seen).len();
                    if theirs_out > 0 && !trumps(obs.hand, trump).is_empty() {
                        let when = match obs.tricks.len() + 1 {
                            2..=3 => "tricks 2-3",
                            _ => "trick 4 on",
                        };
                        let what = if is_trump(card, trump) { "led a trump" } else { "led a side suit" };
                        tally.add(format!("Declarer on lead, {when}, trumps still out"), what);
                    }
                } else if !declaring && *first_defence_lead {
                    *first_defence_lead = false;
                    tally.add(format!("Defenders' first lead ({contract_kind})"), kind);
                }
            } else {
                let led = trick.led_suit().unwrap();
                let winning = trick.winning_card(trump).unwrap();
                let partner_winning = trick.winner(trump) == Some(me.partner());
                let can_follow = legal.iter().all(|a| matches!(a, Action::Play(p) if effective_suit(p.card, trump) == Some(led)));
                let beats = trick_strength(card, led, trump) > trick_strength(winning, led, trump);
                let position = trick.cards().len() + 1;
                if position == 2 && can_follow && !partner_winning {
                    tally.add(format!("Second hand, following suit ({contract_kind})"), if beats { "played to win" } else { "played low" });
                }
                if !can_follow && legal.len() > 1 {
                    // Only count real choices: a trump that would take the lead, and a non-trump to throw instead.
                    let ruff_wins = legal.iter().any(|a| matches!(a, Action::Play(p) if (is_trump(p.card, trump) || (trump.is_none() && p.card.is_joker())) && trick_strength(p.card, led, trump) > trick_strength(winning, led, trump)));
                    let can_discard = legal.iter().any(|a| matches!(a, Action::Play(p) if !is_trump(p.card, trump) && !p.card.is_joker()));
                    if ruff_wins && can_discard {
                        let situation = if partner_winning {
                            if is_boss(winning, seen, trump) { "Void, partner winning with a sure winner" } else { "Void, partner winning (could be overtaken)" }
                        } else {
                            "Void, opponent winning, a trump would win"
                        };
                        let choice = if is_trump(card, trump) || (trump.is_none() && card.is_joker()) { "trumped" } else { "discarded" };
                        tally.add(format!("{situation} ({contract_kind})"), choice);
                    }
                }
            }
            if card.is_joker() {
                let when = match obs.tricks.len() + 1 {
                    1..=2 => "tricks 1-2",
                    3..=5 => "tricks 3-5",
                    _ => "tricks 6-10",
                };
                let who = if declaring { "declaring side" } else { "defenders" };
                tally.add(format!("Joker played by the {who} ({contract_kind})"), when);
            }
        }
        _ => {}
    }
}
