//! `fh-lab whatif --code CODE --at N --a "Joker naming ♦" --b 4♦`: at action N of a coded game, two
//! choices played out from there by The Oracle in every seat, over the deals the player to act
//! thinks possible (sampled from their own view, as a bot sees it). Shows whether a choice that
//! went wrong at the table was a fair gamble or a real mistake.

use fh_ai::belief::Belief;
use fh_ai::search_bot::{SearchBot, SearchConfig};
use fh_ai::Bot;
use fh_core::{Action, CardSet, Event, Game, Phase, Rng, Seat};
use fh_wasm::table::replay;

type Outcome = (i32, i32, bool, bool);

pub fn run(code: &str, at: usize, a: &str, b: &str, worlds: usize, threads: u64) {
    let save = crate::show::load(code).unwrap_or_else(|e| {
        eprintln!("{e}");
        std::process::exit(1)
    });
    let game = replay(&save, at).expect("replays to the decision");
    let seat = game.to_act().expect("someone to act");
    let legal = game.legal_actions();
    let pick = |label: &str| {
        *legal
            .iter()
            .find(|x| x.to_string() == label)
            .unwrap_or_else(|| panic!("no legal action {label:?}; legal: {}", legal.iter().map(|x| x.to_string()).collect::<Vec<_>>().join(", ")))
    };
    let choices = [pick(a), pick(b)];
    let (declarer, _) = game.contract().expect("a contract in play");

    // The deal from its first action, to rebuild each world's version of it.
    let hands_done = game.history().len();
    let start = (0..=at).rev().take_while(|&n| replay(&save, n).is_ok_and(|g| g.history().len() == hands_done)).last().unwrap_or(at);
    let deal = replay(&save, start).expect("replays to the deal");
    let mut actions = Vec::new();
    let mut g = deal.clone();
    for n in start..at {
        let action = g.legal_actions()[save.log[n] as usize];
        g.apply(action).expect("replays");
        actions.push(action);
    }
    let real_discards: Vec<_> = actions.iter().filter_map(|x| if let Action::Discard(c) = x { Some(*c) } else { None }).collect();

    let obs = game.observe(seat);
    let belief = Belief::new_with(&obs, true, true);
    let sampled = belief.sample_fitting(&mut Rng::new(save.seed ^ 0x5eed), obs.hand, worlds);
    println!("{seat} at action {at}: {a} or {b}, over {} deals {seat} thinks possible", sampled.len());

    let played_by = |s: Seat| -> CardSet { game.tricks().iter().chain(game.current_trick()).flat_map(|t| t.plays()).filter(|&(p, _)| p == s).map(|(_, c)| c).collect() };
    let per_thread = sampled.len().div_ceil(threads.max(1) as usize).max(1);
    let results: Vec<Outcome> = std::thread::scope(|scope| {
        let handles: Vec<_> = sampled
            .chunks(per_thread)
            .enumerate()
            .map(|(c, chunk)| {
                let (actions, deal, real_discards, played_by) = (&actions, &deal, &real_discards, &played_by);
                scope.spawn(move || {
                    chunk
                        .iter()
                        .enumerate()
                        .filter_map(|(i, world)| {
                            // Each seat was dealt what it holds now and what it has played. The
                            // declarer's discards stand in for the kitty: picked up, then put back
                            // (the world's own discards, when the one deciding couldn't see them).
                            let hands: [CardSet; 4] = std::array::from_fn(|s| world.hands[s].union(played_by(Seat::new(s))));
                            let discards: Vec<_> = if world.out.is_empty() { real_discards.clone() } else { world.out.iter().collect() };
                            let kitty: CardSet = discards.iter().copied().collect();
                            if hands.iter().any(|h| h.len() != 10) || kitty.len() != 3 {
                                return None;
                            }
                            let mut base = Game::from_deal(deal.rules().clone(), save.seed, deal.dealer(), hands, kitty);
                            base.set_scores(deal.scores());
                            let mut next_discard = discards.iter();
                            for &action in actions {
                                let action = if let Action::Discard(_) = action { Action::Discard(*next_discard.next()?) } else { action };
                                base.apply(action).ok()?;
                            }
                            let seed = (c * 100_000 + i) as u64;
                            let mut out = [(0, 0, false); 2];
                            for (k, &choice) in choices.iter().enumerate() {
                                let mut g = base.clone();
                                g.apply(choice).ok()?;
                                out[k] = finish(g, seed, seat)?;
                            }
                            let _ = declarer;
                            Some((out[0].0 - out[1].0, out[0].1 - out[1].1, out[0].2, out[1].2))
                        })
                        .collect::<Vec<_>>()
                })
            })
            .collect();
        handles.into_iter().flat_map(|h| h.join().unwrap()).collect()
    });
    let n = results.len().max(1) as f64;
    let mean = |f: fn(&Outcome) -> f64| results.iter().map(f).sum::<f64>() / n;
    let ci = |f: fn(&Outcome) -> f64| {
        let m = mean(f);
        1.96 * (results.iter().map(|r| (f(r) - m).powi(2)).sum::<f64>() / n / n).sqrt()
    };
    println!("  played out in {} of them", results.len());
    println!("  tricks for {seat}'s side, {a} − {b}: {:+.2} ± {:.2}", mean(|r| r.0 as f64), ci(|r| r.0 as f64));
    println!("  points, {a} − {b}: {:+.1} ± {:.0}", mean(|r| r.1 as f64), ci(|r| r.1 as f64));
    println!("  the bid made: {a} {:.0}%, {b} {:.0}%", 100.0 * mean(|r| r.2 as u8 as f64), 100.0 * mean(|r| r.3 as u8 as f64));
    let worse = results.iter().filter(|r| r.0 < 0).count() as f64;
    let better = results.iter().filter(|r| r.0 > 0).count() as f64;
    println!("  {a} took fewer tricks {:.0}% of the time, more {:.0}%", 100.0 * worse / n, 100.0 * better / n);
}

/// Plays the hand to its end with The Oracle in every seat: `seat`'s side's tricks, its points
/// (its score change less the other side's), and whether the bid was made.
fn finish(mut game: Game, seed: u64, seat: Seat) -> Option<(i32, i32, bool)> {
    let before = game.scores();
    let mut bots: [SearchBot; 4] = std::array::from_fn(|i| SearchBot::new("The Oracle", SearchConfig::ORACLE, seed * 4 + i as u64));
    loop {
        let to_act = game.to_act()?;
        let action = bots[to_act.index()].choose(&game.observe(to_act), &game.legal_actions());
        let events = game.apply(action).ok()?;
        if events.iter().any(|e| matches!(e, Event::HandScored { .. })) || game.phase() == Phase::GameOver {
            break;
        }
    }
    let after = game.scores();
    let record = game.history().last()?;
    let (declarer, _) = record.contract?;
    let us = seat.team().index();
    let made = after[declarer.team().index()] > before[declarer.team().index()];
    Some((record.tricks_won[us] as i32, (after[us] - before[us]) - (after[1 - us] - before[1 - us]), made))
}
