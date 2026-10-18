//! `fh-lab leads`: which opening leads work, measured with the expert search.
//!
//! At the declarer's opening lead and at the defenders' first lead, every legal lead is scored on
//! the same sampled worlds (what the leader could know). Leads are grouped into classes; for each
//! situation and class we report how often the class held the best lead and the average cost of
//! the best lead in that class against the overall best.

use crate::make_bot;
use fh_ai::cardplay::{is_boss, trumps};
use fh_ai::search_bot::{SearchBot, SearchConfig};
use fh_core::cards::ACE;
use fh_core::rules::is_trump;
use fh_core::{effective_suit, Action, Call, Card, Event, Game, Observation, Phase, Rules, Seat, Strain};
use std::collections::BTreeMap;

type Table = BTreeMap<String, BTreeMap<String, (u64, u64, f64)>>; // situation → class → (offered, best, total regret)

fn class(card: Card, obs: &Observation, partner_mention: Option<Strain>) -> String {
    let trump = obs.trump();
    let seen = obs.played().union(obs.hand).union(obs.discards.unwrap_or_default());
    if card.is_joker() {
        return "the Joker".into();
    }
    if let (Some(m), Some(suit)) = (partner_mention.and_then(|m| m.trump()), effective_suit(card, trump)) {
        if Some(m) != trump && suit == m && !is_trump(card, trump) {
            return "partner's mentioned suit".into();
        }
    }
    if is_trump(card, trump) {
        return if is_boss(card, seen, trump) { "top trump" } else { "low trump" }.into();
    }
    if card.rank() == ACE || is_boss(card, seen, trump) {
        "side ace / winner".into()
    } else {
        "low side card".into()
    }
}

fn record(table: &mut Table, situation: String, obs: &Observation, bot: &mut SearchBot) {
    let legal = {
        let trick = obs.current_trick.clone().unwrap();
        fh_core::legal_plays(obs.hand, &trick, obs.trump()).into_iter().map(Action::Play).collect::<Vec<_>>()
    };
    if legal.len() < 2 {
        return;
    }
    let stats = bot.play_stats(obs, &legal);
    if !stats.valid {
        return;
    }
    let partner = obs.seat.partner();
    let partner_mention = obs.auction.calls().find_map(|(s, c)| match c {
        Call::Mention(m) if s == partner => Some(m),
        _ => None,
    });
    let best = stats.stats.iter().map(|s| s.ev).fold(f64::MIN, f64::max);
    let mut by_class: BTreeMap<String, f64> = BTreeMap::new();
    for s in &stats.stats {
        let Action::Play(p) = s.action else { continue };
        let c = class(p.card, obs, partner_mention);
        let e = by_class.entry(c).or_insert(f64::MIN);
        *e = e.max(s.ev);
    }
    let entry = table.entry(situation).or_default();
    for (c, ev) in by_class {
        let slot = entry.entry(c).or_insert((0, 0, 0.0));
        slot.0 += 1;
        if best - ev < 0.5 {
            slot.1 += 1;
        }
        slot.2 += best - ev;
    }
}

fn one_deal(seed: u64, table: &mut Table) {
    let mut game = Game::with_dealer(Rules::default(), seed, Seat::new((seed % 4) as usize));
    let mut bots: [Box<dyn fh_ai::Bot>; 4] = std::array::from_fn(|i| make_bot("oracle", seed * 4 + i as u64));
    let config = SearchConfig { play_samples: 32, min_play_samples: 32, node_budget: u64::MAX, ..SearchConfig::COUNTER };
    let mut analyst = SearchBot::new("analyst", config, seed);
    let start = game.history().len();
    let mut defenders_led = false;
    while game.history().len() == start && game.phase() != Phase::GameOver {
        let seat = game.to_act().unwrap();
        let obs = game.observe(seat);
        if obs.phase == Phase::Playing && obs.current_trick.as_ref().is_some_and(|t| t.is_empty()) {
            let (declarer, contract) = obs.contract.unwrap();
            let kind = if contract.trump().is_some() { "suit" } else { "NT" };
            let partner_calls: Vec<Call> = obs.auction.calls().filter(|(s, _)| *s == seat.partner()).map(|(_, c)| c).collect();
            let partner_six = partner_calls.iter().any(|c| matches!(c, Call::Six(_)));
            if obs.tricks.is_empty() && seat == declarer {
                let n = trumps(obs.hand, contract.trump()).len();
                let bucket = match n {
                    0..=4 => "≤4",
                    5 => "5",
                    _ => "6+",
                };
                record(table, format!("Declarer's opening lead ({kind}), {bucket} trumps"), &obs, &mut analyst);
                if kind == "suit" {
                    if partner_six {
                        record(table, "Declarer's opening lead (suit), partner called 6 (has the Joker)".into(), &obs, &mut analyst);
                    }
                    if obs.hand.contains(Card::JOKER) {
                        record(table, "Declarer's opening lead (suit), holding the Joker".into(), &obs, &mut analyst);
                    }
                    let mentioned_trumps = partner_calls.iter().any(|c| matches!(c, Call::Mention(m) if *m == contract.strain));
                    if mentioned_trumps {
                        record(table, "Declarer's opening lead (suit), partner mentioned trumps (a bower)".into(), &obs, &mut analyst);
                    }
                }
            } else if seat.team() != declarer.team() && !defenders_led {
                defenders_led = true;
                record(table, format!("Defenders' first lead ({kind})"), &obs, &mut analyst);
                if partner_calls.iter().any(|c| matches!(c, Call::Mention(m) if Some(*m) != Some(contract.strain) && m.trump().is_some())) {
                    record(table, format!("Defenders' first lead ({kind}), partner mentioned a side suit"), &obs, &mut analyst);
                }
                if partner_six {
                    record(table, format!("Defenders' first lead ({kind}), partner called 6"), &obs, &mut analyst);
                }
                if defenders_led && seat != declarer {
                    // Once both first leads are recorded, the rest of the hand doesn't matter.
                    break;
                }
            }
        }
        let legal = game.legal_actions();
        let action = bots[seat.index()].choose(&obs, &legal);
        if game.apply(action).unwrap().iter().any(|e| matches!(e, Event::HandScored { .. })) {
            break;
        }
    }
}

pub fn run(deals: u64, seed: u64, threads: u64) {
    let tables: Vec<Table> = std::thread::scope(|scope| {
        let handles: Vec<_> = (0..threads)
            .map(|t| {
                scope.spawn(move || {
                    let mut table = Table::new();
                    let mut d = t;
                    while d < deals {
                        one_deal(seed + d, &mut table);
                        d += threads;
                    }
                    table
                })
            })
            .collect();
        handles.into_iter().map(|h| h.join().unwrap()).collect()
    });
    let mut total = Table::new();
    for t in tables {
        for (s, classes) in t {
            for (c, (o, b, r)) in classes {
                let slot = total.entry(s.clone()).or_default().entry(c).or_insert((0, 0, 0.0));
                slot.0 += o;
                slot.1 += b;
                slot.2 += r;
            }
        }
    }
    println!("Opening leads over {deals} deals (32 worlds per decision; cost = expected points versus the best lead)\n");
    for (situation, classes) in &total {
        let n = classes.values().map(|v| v.0).max().unwrap_or(0);
        println!("{situation} (n≈{n})");
        let mut rows: Vec<_> = classes.iter().collect();
        rows.sort_by(|a, b| (a.1 .2 / a.1 .0 as f64).total_cmp(&(b.1 .2 / b.1 .0 as f64)));
        for (c, (o, b, r)) in rows {
            println!("    {c:<26} best {:>3.0}% of the time it was available, average cost {:>5.1}  (available {o})", 100.0 * *b as f64 / *o as f64, r / *o as f64);
        }
    }
}
