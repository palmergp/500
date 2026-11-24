//! `fh-lab jokerlead`: in no trumps, when The Oracle is about to lead the Joker early (with other
//! cards left), what does that lead cost against its best other lead? From that moment the hand is
//! played out twice by the same bots (same seeds), once each way, and the solver (every card
//! visible) also values both leads. The search runs without its opening-Joker rule (finding 54),
//! which this measured. Only no-trump hands are played; the rest stop after the
//! bidding.

use std::collections::BTreeMap;

use fh_ai::search_bot::{SearchBot, SearchConfig};
use fh_ai::solver::{Position, Solver};
use fh_ai::Bot;
use fh_core::{Action, Card, Event, Game, Phase, Rules, Seat, Trick};

#[derive(Default, Clone, Copy)]
struct Tally {
    n: u32,
    /// Leader's side, Joker lead minus the other lead.
    tricks: f64,
    points: f64,
    /// Sums of squares, for the confidence intervals.
    tricks2: f64,
    points2: f64,
    made_joker: u32,
    made_other: u32,
    /// Declaring-side cases (the made counts are out of these).
    declaring: u32,
    dd: f64,
    dd_worse: u32,
    dd_better: u32,
    /// The search's own margin for the Joker over the other lead, in points.
    margin: f64,
}

impl Tally {
    fn add(&mut self, o: &Tally) {
        self.n += o.n;
        self.tricks += o.tricks;
        self.points += o.points;
        self.tricks2 += o.tricks2;
        self.points2 += o.points2;
        self.made_joker += o.made_joker;
        self.made_other += o.made_other;
        self.declaring += o.declaring;
        self.dd += o.dd;
        self.dd_worse += o.dd_worse;
        self.dd_better += o.dd_better;
        self.margin += o.margin;
    }
}

type Out = BTreeMap<(String, String), Tally>;

pub fn run(deals: u64, seed: u64, threads: u64, before: usize) {
    let parts: Vec<(Out, u64)> = std::thread::scope(|scope| {
        let handles: Vec<_> = (0..threads)
            .map(|t| {
                scope.spawn(move || {
                    let mut out = Out::new();
                    let mut nt = 0;
                    let mut d = t;
                    while d < deals {
                        nt += u64::from(deal(seed + d, before, &mut out));
                        d += threads;
                    }
                    (out, nt)
                })
            })
            .collect();
        handles.into_iter().map(|h| h.join().unwrap()).collect()
    });
    let mut all = Out::new();
    let mut nt = 0;
    for (part, n) in parts {
        nt += n;
        for (k, t) in part {
            all.entry(k).or_default().add(&t);
        }
    }
    let mut totals: BTreeMap<String, Tally> = BTreeMap::new();
    for ((who, _), t) in &all {
        totals.entry(who.clone()).or_default().add(t);
    }
    println!("{deals} deals, {nt} played in no trumps. The Oracle about to lead the Joker in tricks 1–{before}, with other cards left:");
    println!("each played out both ways (the Joker, or the search's best other lead) by the same bots.\n");
    println!("| Leader | Trick | n | Tricks (Joker − other, 95%) | Points (Joker − other, 95%) | Bid made: Joker / other | Solver: Joker − other | Solver: Joker worse / better |");
    println!("|---|---|---|---|---|---|---|---|");
    let row = |who: &str, band: &str, t: &Tally| {
        let n = t.n.max(1) as f64;
        let ci = |sum: f64, sum2: f64| 1.96 * ((sum2 / n - (sum / n).powi(2)).max(0.0) / n).sqrt();
        let made = if t.declaring > 0 {
            format!("{:.0}% / {:.0}%", 100.0 * t.made_joker as f64 / t.declaring as f64, 100.0 * t.made_other as f64 / t.declaring as f64)
        } else {
            String::new()
        };
        println!(
            "| {who} | {band} | {} | {:+.2} ± {:.2} | {:+.1} ± {:.0} | {made} | {:+.2} | {:.0}% / {:.0}% |",
            t.n,
            t.tricks / n,
            ci(t.tricks, t.tricks2),
            t.points / n,
            ci(t.points, t.points2),
            t.dd / n,
            100.0 * t.dd_worse as f64 / n,
            100.0 * t.dd_better as f64 / n
        );
    };
    for ((who, band), t) in &all {
        row(who, band, t);
    }
    for (who, t) in &totals {
        row(who, "all", t);
        println!("  ({who}: average search margin {:.1} points)", t.margin / t.n.max(1) as f64);
    }
}

fn bots(seed: u64) -> [SearchBot; 4] {
    std::array::from_fn(|i| SearchBot::new("The Oracle", SearchConfig { hold_opening_joker: false, ..SearchConfig::ORACLE }, seed.wrapping_mul(4).wrapping_add(i as u64)))
}

/// Plays the hand on; the leader's side's tricks and points (ours minus theirs), and whether the
/// bid was made.
fn finish(mut game: Game, seed: u64, team: fh_core::Team) -> (i32, i32, bool) {
    let before = game.scores();
    let mut bots = bots(seed);
    loop {
        let seat = game.to_act().expect("hand in progress");
        let action = bots[seat.index()].choose(&game.observe(seat), &game.legal_actions());
        let events = game.apply(action).expect("legal");
        if events.iter().any(|e| matches!(e, Event::HandScored { .. })) || game.phase() == Phase::GameOver {
            break;
        }
    }
    let after = game.scores();
    let record = game.history().last().expect("a scored hand");
    let (declarer, _) = record.contract.expect("a played hand");
    let us = team.index();
    let points = (after[us] - before[us]) - (after[1 - us] - before[1 - us]);
    let made = after[declarer.team().index()] > before[declarer.team().index()];
    (record.tricks_won[us] as i32, points, made)
}

/// One deal; true if it was played in no trumps.
fn deal(seed: u64, before: usize, out: &mut Out) -> bool {
    let mut game = Game::new(Rules::default(), seed);
    let mut players = bots(seed);
    loop {
        let Some(seat) = game.to_act() else { return false };
        if game.phase() == Phase::Playing {
            let Some((declarer, contract)) = game.contract() else { return false };
            if contract.trump().is_some() {
                return false;
            }
            let k = game.tricks().len();
            if k >= before {
                return true;
            }
            let leading = game.current_trick().is_none_or(|t| t.cards().is_empty());
            let hand = game.hand(seat);
            if leading && hand.contains(Card::JOKER) && hand.len() > 1 {
                let obs = game.observe(seat);
                let legal = game.legal_actions();
                let ranked = players[seat.index()].evaluate_plays(&obs, &legal);
                let is_joker = |a: &Action| matches!(a, Action::Play(p) if p.card.is_joker());
                if is_joker(&ranked[0].0) {
                    let (joker, joker_ev) = ranked[0];
                    let &(other, other_ev) = ranked.iter().find(|(a, _)| !is_joker(a)).expect("other cards");
                    let team = seat.team();
                    let cont = seed.wrapping_add(1_000_003);
                    let mut a = game.clone();
                    a.apply(joker).expect("legal");
                    let mut b = game.clone();
                    b.apply(other).expect("legal");
                    let (ta, pa, ma) = finish(a, cont, team);
                    let (tb, pb, mb) = finish(b, cont, team);
                    let hands: [fh_core::CardSet; 4] = std::array::from_fn(|i| game.hand(Seat::new(i)));
                    let values = Solver::new(None).analyze(&Position { hands, trump: None, trick: Trick::new(seat) }, team);
                    let value = |act: Action| values.iter().filter(|(p, _)| Action::Play(*p) == act).map(|&(_, v)| v).max().unwrap_or(0);
                    let dd = value(joker) as i32 - value(other) as i32;
                    let declaring = team == declarer.team();
                    let who = if declaring { "declaring side" } else { "defender" };
                    let band = match k {
                        0 => "trick 1",
                        1..=2 => "tricks 2–3",
                        3..=5 => "tricks 4–6",
                        _ => "tricks 7+",
                    };
                    // What the Joker names: a suit the leader still holds (its own suit, where
                    // partner may hold the rest), or one it's out of.
                    let named = match joker {
                        Action::Play(p) => p.named_suit,
                        _ => None,
                    };
                    let kind = match named {
                        Some(suit) if hand.iter().any(|c| !c.is_joker() && fh_core::effective_suit(c, None) == Some(suit)) => "names its own suit",
                        Some(_) => "names a suit it's out of",
                        None => "names no suit",
                    };
                    let t = out.entry((who.to_string(), format!("{band}, {kind}"))).or_default();
                    t.n += 1;
                    t.tricks += (ta - tb) as f64;
                    t.points += (pa - pb) as f64;
                    t.tricks2 += ((ta - tb) as f64).powi(2);
                    t.points2 += ((pa - pb) as f64).powi(2);
                    if declaring {
                        t.declaring += 1;
                        t.made_joker += u32::from(ma);
                        t.made_other += u32::from(mb);
                    }
                    t.dd += dd as f64;
                    t.dd_worse += u32::from(dd < 0);
                    t.dd_better += u32::from(dd > 0);
                    t.margin += joker_ev - other_ev;
                    return true;
                }
                let action = ranked[0].0;
                game.apply(action).expect("legal");
                continue;
            }
        }
        let action = players[seat.index()].choose(&game.observe(seat), &game.legal_actions());
        if game.apply(action).is_err() {
            return false;
        }
        if game.history().iter().any(|h| h.contract.is_some()) {
            return false;
        }
    }
}
