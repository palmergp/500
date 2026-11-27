//! `fh-lab gradestab`: are the coach's grades steady, and do its reasons say why?
//!
//! Steady and Rookie play South (so there are mistakes to grade) with expert bots in the other
//! seats. A sample of South's decisions (up to three card plays, the first bid or pass with a
//! choice, and the kitty, put back with one card changed from the plan) is graded three times by
//! coaches seeded differently, as two people with the same position would see. A grade that turns
//! from good to a mistake between seeds is noise a player would take as a verdict.
//!
//! For every card play graded a mistake or blunder, the coach's reasons are sorted: specific (they
//! say what goes wrong: a stopper, partner's winner, trumps out…) or only generic (how the bid
//! fares, "X is the highest card left", the points between the plays). The kinds of decision whose
//! mistakes most often get only generic reasons are where new explanations would help most.

use std::collections::BTreeMap;

use crate::coachaudit::kind;
use fh_ai::coach::{Coach, Verdict};
use fh_ai::rule_bot::{heuristic_discards, RuleBot, Style};
use fh_ai::search_bot::{SearchBot, SearchConfig};
use fh_ai::Bot;
use fh_core::{Action, Call, Card, CardSet, Event, Game, Observation, Phase, Rng, Rules, Seat};

#[derive(Default, Clone)]
struct Stability {
    n: f64,
    /// All three grades gave the same verdict.
    same: f64,
    /// One said best or good, another mistake or blunder.
    across: f64,
    /// Points between the highest and lowest of the three losses.
    spread: f64,
}

#[derive(Default)]
struct Out {
    stability: BTreeMap<&'static str, Stability>,
    /// Mistakes and blunders in card play by kind of decision: how many, how many with only
    /// generic reasons, and a few of those as examples.
    slips: BTreeMap<String, (u32, u32, Vec<String>)>,
}

pub fn run(deals: u64, seed: u64, threads: u64) {
    let parts: Vec<Out> = std::thread::scope(|scope| {
        let handles: Vec<_> = (0..threads)
            .map(|t| {
                scope.spawn(move || {
                    let mut out = Out::default();
                    let mut d = t;
                    while d < deals {
                        deal(seed + d, &mut out);
                        d += threads;
                    }
                    out
                })
            })
            .collect();
        handles.into_iter().map(|h| h.join().unwrap()).collect()
    });
    let mut all = Out::default();
    for part in parts {
        for (k, s) in part.stability {
            let e = all.stability.entry(k).or_default();
            e.n += s.n;
            e.same += s.same;
            e.across += s.across;
            e.spread += s.spread;
        }
        for (k, (n, g, ex)) in part.slips {
            let e = all.slips.entry(k).or_default();
            e.0 += n;
            e.1 += g;
            for x in ex {
                if e.2.len() < 3 {
                    e.2.push(x);
                }
            }
        }
    }
    println!("{deals} deals, Steady or Rookie as South. Each sampled decision graded three times, by differently seeded coaches.\n");
    println!("| Decision | n | Same verdict all three times | Good on one, a mistake on another | Spread of the loss (points) |");
    println!("|---|---|---|---|---|");
    for (k, s) in &all.stability {
        let n = s.n.max(1.0);
        println!("| {k} | {} | {:.0}% | {:.1}% | {:.1} |", s.n, 100.0 * s.same / n, 100.0 * s.across / n, s.spread / n);
    }
    let total: u32 = all.slips.values().map(|v| v.0).sum();
    let generic: u32 = all.slips.values().map(|v| v.1).sum();
    println!("\nCard-play mistakes and blunders: {total}; with only generic reasons: {generic} ({:.0}%).\n", 100.0 * f64::from(generic) / f64::from(total.max(1)));
    println!("| Decision (the player's card, the coach's) | Slips | Only generic reasons |");
    println!("|---|---|---|");
    let mut rows: Vec<_> = all.slips.iter().collect();
    rows.sort_by_key(|(_, v)| std::cmp::Reverse(v.1));
    for (k, (n, g, _)) in rows.iter().take(20) {
        println!("| {k} | {n} | {g} |");
    }
    println!("\nExamples of slips with only generic reasons:\n");
    for (k, (_, _, ex)) in rows.iter().take(10) {
        for x in ex {
            println!("- **{k}**: {x}");
        }
    }
}

/// A reason that says why (not just how the bid fares or which card is high).
fn specific(reason: &str) -> bool {
    let generic = [
        "Best play:",
        "Best call:",
        "The bid is made in",
        "is the highest card left in its suit",
        "A close call",
        "Close call",
        "wins this trick.",
        "tricks on average",
        "Your bid is made.",
        "Your bid is beaten.",
        "Their bid is made",
        "Their bid is beaten.",
        "is worth about",
    ];
    let starts_with = reason.starts_with("With ") && reason.contains("the bid is made");
    !(starts_with || generic.iter().any(|g| reason.contains(g)))
}

fn note(out: &mut Out, phase: &'static str, verdicts: &[Verdict], losses: &[f64]) {
    let s = out.stability.entry(phase).or_default();
    s.n += 1.0;
    s.same += f64::from(u8::from(verdicts.iter().all(|&v| v == verdicts[0])));
    let good = verdicts.iter().any(|&v| v <= Verdict::Good);
    let slip = verdicts.iter().any(|&v| v >= Verdict::Mistake);
    s.across += f64::from(u8::from(good && slip));
    let (lo, hi) = losses.iter().fold((f64::MAX, f64::MIN), |(a, b), &x| (a.min(x), b.max(x)));
    s.spread += hi - lo;
}

fn deal(seed: u64, out: &mut Out) {
    let mut game = Game::new(Rules::default(), seed);
    let south: Box<dyn Bot> = if seed.is_multiple_of(2) {
        Box::new(RuleBot::new("Steady", Style::STEADY, seed))
    } else {
        Box::new(RuleBot::new("Rookie", Style::ROOKIE, seed))
    };
    let mut others: [SearchBot; 4] = std::array::from_fn(|i| SearchBot::new("expert", SearchConfig::FAST, seed * 4 + i as u64));
    let mut south = south;
    let mut rng = Rng::new(seed ^ 0x57ab);
    // South's decisions with a choice: (position, legal, choice); the kitty separately.
    let mut plays: Vec<(Observation, Vec<Action>, Action)> = Vec::new();
    let mut call: Option<(Observation, Vec<Action>, Action)> = None;
    let mut kitty: Option<Observation> = None;
    while let Some(seat) = game.to_act() {
        let obs = game.observe(seat);
        let legal = game.legal_actions();
        let action = if seat == Seat::SOUTH { south.choose(&obs, &legal) } else { others[seat.index()].choose(&obs, &legal) };
        if seat == Seat::SOUTH && legal.len() > 1 {
            match game.phase() {
                Phase::Playing => plays.push((obs.clone(), legal.clone(), action)),
                Phase::Bidding if call.is_none() => call = Some((obs.clone(), legal.clone(), action)),
                Phase::Discarding if kitty.is_none() => kitty = Some(obs.clone()),
                _ => {}
            }
        }
        let Ok(events) = game.apply(action) else { break };
        if events.iter().any(|e| matches!(e, Event::HandScored { .. } | Event::PassedOut { .. })) {
            break;
        }
    }
    let coaches = |k: u64| {
        fh_ai::coach::clear_play_cache();
        Coach::new(seed ^ (k << 32) ^ 0x9e37)
    };
    // Card plays: up to three, at random.
    for _ in 0..3.min(plays.len()) {
        let (obs, legal, chosen) = plays.swap_remove(rng.below(plays.len()));
        let grades: Vec<_> = (0..3).map(|k| coaches(k).grade(&obs, &legal, chosen)).collect();
        note(out, "card play", &grades.iter().map(|g| g.verdict).collect::<Vec<_>>(), &grades.iter().map(|g| g.loss).collect::<Vec<_>>());
        let g = &grades[0];
        if g.verdict >= Verdict::Mistake {
            let (Action::Play(c), Action::Play(b)) = (chosen, g.best) else { continue };
            let lead = obs.current_trick.as_ref().is_none_or(|t| t.cards().is_empty());
            let key = format!("{} {} instead of {}", if lead { "lead" } else { "follow" }, kind(&obs, c.card), kind(&obs, b.card));
            let only_generic = !g.analysis.reasons.iter().any(|r| specific(r));
            let e = out.slips.entry(key).or_default();
            e.0 += 1;
            if only_generic {
                e.1 += 1;
                if e.2.len() < 3 {
                    let trick: Vec<String> = obs.current_trick.as_ref().map_or(vec![], |t| t.cards().iter().map(|c| c.to_string()).collect());
                    let bid = obs.contract.map_or(String::new(), |(s, c)| format!("{c} by {s}"));
                    e.2.push(format!(
                        "{bid}, trick {}, holding {}{}: played {}, coach {} ({:.0} points). Reasons: {}",
                        obs.tricks.len() + 1,
                        obs.hand,
                        if trick.is_empty() { String::new() } else { format!(", trick so far {}", trick.join(" ")) },
                        c.card,
                        b,
                        g.loss,
                        g.analysis.reasons.join(" | ")
                    ));
                }
            }
        }
    }
    // The first call with a choice.
    if let Some((obs, legal, chosen)) = call {
        if matches!(chosen, Action::Call(Call::Bid(_)) | Action::Call(Call::Pass)) {
            let grades: Vec<_> = (0..3).map(|k| coaches(k).grade(&obs, &legal, chosen)).collect();
            note(out, "bidding", &grades.iter().map(|g| g.verdict).collect::<Vec<_>>(), &grades.iter().map(|g| g.loss).collect::<Vec<_>>());
        }
    }
    // The kitty: the plan with one card changed, so there's something to grade.
    if let Some(obs) = kitty {
        let plan = heuristic_discards(obs.hand, obs.trump(), 3);
        let kept: Vec<Card> = obs.hand.minus(plan.iter().copied().collect::<CardSet>()).iter().collect();
        let mut chosen = plan.clone();
        chosen[rng.below(3)] = kept[rng.below(kept.len())];
        let grades: Vec<_> = (0..3).map(|k| coaches(k).grade_discard_set(&obs, &chosen)).collect();
        note(out, "kitty", &grades.iter().map(|g| g.verdict).collect::<Vec<_>>(), &grades.iter().map(|g| g.loss).collect::<Vec<_>>());
    }
}
