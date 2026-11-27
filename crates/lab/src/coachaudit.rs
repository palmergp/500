//! `fh-lab coachaudit`: does the coach's card-play advice hold up when the cards are really played?
//!
//! The coach ranks plays by solving a few dozen sampled deals with every card visible; that blind
//! spot made it overrate early Joker leads and side aces (findings 54–56) until play-outs showed
//! it. Here, at a random card play in each deal (any seat, any trick), the coach names its best
//! card and its runner-up (the best that plays differently), and both are played out to the end
//! of the hand by The Oracle in every seat, the same seed both ways. Where the coach's favourite
//! does worse at the table than its numbers say, by kind of decision, is a bias to fix.
//!
//! The same positions also go to a coach that reads the auction into its deals (partner's bid,
//! mentions and passes), to see whose predictions match the play-outs better (the third
//! experiment: finding 21 turned this off for the bots' play, but teaching is a different test).

use std::collections::BTreeMap;

use crate::playout::{finish, Mean};
use fh_ai::cardplay::is_boss;
use fh_ai::coach::Coach;
use fh_ai::search_bot::{SearchBot, SearchConfig};
use fh_ai::Bot;
use fh_core::rules::is_trump;
use fh_core::trick::trick_strength;
use fh_core::{effective_suit, Action, Card, Event, Game, Observation, Phase, Rng, Rules};

#[derive(Default, Clone)]
struct Tally {
    /// The coach's predicted gap (best − runner-up, points), the auction-reading coach's for the
    /// same pair, and the real one played out.
    predicted: Mean,
    predicted_inf: Mean,
    real: Mean,
    real_tricks: Mean,
    /// Played out, the coach's favourite did worse / better.
    worse: f64,
    better: f64,
    /// Squared error of each coach's predicted gap against the real one.
    sq: f64,
    sq_inf: f64,
    /// The auction-reading coach preferred the runner-up; and when it did, how the runner-up did.
    flipped: f64,
    flipped_real: Mean,
}

/// One audited decision: its categories and numbers.
struct Row {
    keys: Vec<String>,
    predicted: f64,
    predicted_inf: f64,
    real: f64,
    real_tricks: f64,
}

pub fn run(deals: u64, seed: u64, threads: u64) {
    let parts: Vec<(Vec<Row>, u64)> = std::thread::scope(|scope| {
        let handles: Vec<_> = (0..threads)
            .map(|t| {
                scope.spawn(move || {
                    let mut rows = Vec::new();
                    let mut trivial = 0;
                    let mut d = t;
                    while d < deals {
                        match audit(seed + d) {
                            Some(Some(row)) => rows.push(row),
                            Some(None) => trivial += 1,
                            None => {}
                        }
                        d += threads;
                    }
                    (rows, trivial)
                })
            })
            .collect();
        handles.into_iter().map(|h| h.join().unwrap()).collect()
    });
    let mut tallies: BTreeMap<String, Tally> = BTreeMap::new();
    let mut trivial = 0;
    let mut n = 0;
    for (rows, t) in parts {
        trivial += t;
        for row in rows {
            n += 1;
            for key in &row.keys {
                let tally = tallies.entry(key.clone()).or_default();
                tally.predicted.add(row.predicted);
                tally.predicted_inf.add(row.predicted_inf);
                tally.real.add(row.real);
                tally.real_tricks.add(row.real_tricks);
                tally.worse += f64::from(u8::from(row.real < 0.0));
                tally.better += f64::from(u8::from(row.real > 0.0));
                tally.sq += (row.predicted - row.real).powi(2);
                tally.sq_inf += (row.predicted_inf - row.real).powi(2);
                if row.predicted_inf < 0.0 {
                    tally.flipped += 1.0;
                    tally.flipped_real.add(row.real);
                }
            }
        }
    }
    println!("{deals} deals; {n} card plays audited ({trivial} more had only cards that play alike). At a random card play,");
    println!("the coach's best card and its runner-up, both played out by The Oracle in every seat (same seed both ways).");
    println!("Gaps are the best card's points minus the runner-up's, for the player's side: the coach's prediction, the");
    println!("auction-reading coach's for the same pair, and what happened. Real < predicted means the coach overrates its choice.\n");
    let sections: [(&str, &str); 6] = [
        ("all", "All"),
        ("role", "By player"),
        ("trick", "By trick"),
        ("bid", "By bid"),
        ("lead", "Leads: the coach's card over the runner-up"),
        ("follow", "Following: the coach's card over the runner-up"),
    ];
    for (prefix, title) in sections {
        println!("### {title}\n");
        println!("| | n | Predicted gap | Real gap (95%) | Real tricks | Favourite worse / better | RMS error: coach / reading the auction | Reading the auction preferred the runner-up: n, its real gap |");
        println!("|---|---|---|---|---|---|---|---|");
        let mut rows: Vec<(&String, &Tally)> = tallies.iter().filter(|(k, _)| k.starts_with(&format!("{prefix}:"))).collect();
        rows.sort_by(|a, b| b.1.real.n.total_cmp(&a.1.real.n));
        for (key, t) in rows.into_iter().take(16) {
            let m = t.real.n.max(1.0);
            println!(
                "| {} | {} | {:+.1} | {} | {} | {:.0}% / {:.0}% | {:.0} / {:.0} | {:.0}, {} |",
                key.split_once(": ").map_or("", |(_, k)| k),
                t.real.n,
                t.predicted.mean(),
                t.real.show(1),
                t.real_tricks.show(2),
                100.0 * t.worse / m,
                100.0 * t.better / m,
                (t.sq / m).sqrt(),
                (t.sq_inf / m).sqrt(),
                t.flipped,
                if t.flipped_real.n > 0.0 { t.flipped_real.show(0) } else { "—".into() }
            );
        }
        println!();
    }
}

/// What kind of card a play is, in its trick.
pub(crate) fn kind(obs: &Observation, card: Card) -> &'static str {
    let trump = obs.trump();
    let seen = obs.played().union(obs.hand).union(obs.discards.unwrap_or_default());
    let trick = obs.current_trick.as_ref();
    match trick.and_then(|t| t.led_suit()) {
        None => {
            if card.is_joker() {
                "the Joker"
            } else if is_trump(card, trump) {
                if is_boss(card, seen, trump) { "the top trump" } else { "a lower trump" }
            } else if is_boss(card, seen, trump) {
                "a side winner"
            } else if card.rank() >= fh_core::cards::KING {
                "a side king"
            } else {
                "a low side card"
            }
        }
        Some(led) => {
            let t = trick.expect("a trick");
            let winning = t.winning_card(trump).expect("a card led");
            let beats = trick_strength(card, led, trump) > trick_strength(winning, led, trump);
            let follows = effective_suit(card, trump) == Some(led);
            let partner_winning = t.winner(trump) == Some(obs.seat.partner());
            match (follows, beats) {
                (true, true) if partner_winning => "over partner's card",
                (true, true) if is_boss(card, seen, trump) => "a sure winner",
                (true, true) => "a card that wins for now",
                (true, false) => "a low card",
                (false, true) if card.is_joker() => "the Joker",
                (false, true) if partner_winning => "a trump over partner's winner",
                (false, true) => "a trump",
                (false, false) => "a discard",
            }
        }
    }
}

/// One deal: None if no card play was reached, Some(None) if the play had no real choice.
fn audit(seed: u64) -> Option<Option<Row>> {
    let mut game = Game::new(Rules::default(), seed);
    let mut bots: [SearchBot; 4] = std::array::from_fn(|i| SearchBot::new("lead-up", SearchConfig::FAST, seed * 4 + i as u64));
    let target = Rng::new(seed ^ 0xa0d17).below(40);
    let mut plays = 0;
    loop {
        let seat = game.to_act()?;
        if game.phase() == Phase::Playing {
            if plays == target {
                break;
            }
            plays += 1;
        }
        let action = bots[seat.index()].choose(&game.observe(seat), &game.legal_actions());
        let events = game.apply(action).ok()?;
        if events.iter().any(|e| matches!(e, Event::HandScored { .. } | Event::PassedOut { .. })) {
            return None;
        }
    }
    let seat = game.to_act()?;
    let legal = game.legal_actions();
    if legal.len() < 2 {
        return Some(None);
    }
    let obs = game.observe(seat);
    let analysis = Coach::new(seed).analyze(&obs, &legal, None);
    let Some((runner, runner_ev)) = analysis.runner_up() else { return Some(None) };
    let best = analysis.best;
    let predicted = analysis.value(best)? - runner_ev;
    let reading = Coach::with_inference(seed, true).analyze(&obs, &legal, None);
    let predicted_inf = reading.value(best)? - reading.value(runner)?;

    let cont = seed.wrapping_add(7_000_003);
    let a = finish({
        let mut g = game.clone();
        g.apply(best).ok()?;
        g
    }, cont, seat)?;
    let b = finish({
        let mut g = game.clone();
        g.apply(runner).ok()?;
        g
    }, cont, seat)?;

    let (declarer, contract) = game.contract()?;
    let role = if seat == declarer {
        "the declarer"
    } else if seat.team() == declarer.team() {
        "the declarer's partner"
    } else {
        "a defender"
    };
    let leading = obs.current_trick.as_ref().is_none_or(|t| t.cards().is_empty());
    let trick = obs.tricks.len();
    let band = match trick {
        0 => "trick 1",
        1..=2 => "tricks 2–3",
        3..=5 => "tricks 4–6",
        _ => "tricks 7–10",
    };
    let card = |a: Action| match a {
        Action::Play(p) => p.card,
        _ => unreachable!("card play"),
    };
    let pair = format!("{} over {}", kind(&obs, card(best)), kind(&obs, card(runner)));
    let bid = if contract.trump().is_some() { "a suit bid" } else { "no trumps" };
    let keys = vec![
        "all: all".to_string(),
        format!("role: {role}, {}", if leading { "leading" } else { "following" }),
        format!("trick: {band}"),
        format!("bid: {bid}"),
        format!("{}: {role}: {pair}", if leading { "lead" } else { "follow" }),
        format!("{}: {pair}", if leading { "lead" } else { "follow" }),
    ];
    Some(Some(Row { keys, predicted, predicted_inf, real: a.points - b.points, real_tricks: f64::from(a.tricks - b.tricks) }))
}
