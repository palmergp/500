//! `fh-lab acelead`: in a suit bid, when the declaring side leads early holding the top trump left,
//! the opponents still hold trumps, and The Oracle (and so the coach) would cash a side ace, does
//! the top trump do better? From that moment the hand is played out twice by the same bots (same
//! seeds), once each way, and the solver (every card visible) also values both leads. Finding 53
//! compared them with the solver alone; this plays them out, as finding 54 did for the Joker.

use std::collections::BTreeMap;

use fh_ai::cardplay::suit_order;
use fh_ai::search_bot::{SearchBot, SearchConfig};
use fh_ai::solver::{Position, Solver};
use fh_ai::Bot;
use fh_core::cards::ACE;
use fh_core::rules::is_trump;
use fh_core::{Action, Card, Event, Game, Phase, Rules, Seat, Trick};

#[derive(Default, Clone, Copy)]
struct Tally {
    n: u32,
    /// Leader's side, top trump minus side ace.
    tricks: f64,
    points: f64,
    tricks2: f64,
    points2: f64,
    made_trump: u32,
    made_ace: u32,
    dd: f64,
    dd_worse: u32,
    dd_better: u32,
    /// The search's own margin for the ace over the top trump, in points.
    margin: f64,
    /// The ace was trumped when led (in the play-out with the ace).
    ruffed: u32,
}

impl Tally {
    fn add(&mut self, o: &Tally) {
        self.n += o.n;
        self.tricks += o.tricks;
        self.points += o.points;
        self.tricks2 += o.tricks2;
        self.points2 += o.points2;
        self.made_trump += o.made_trump;
        self.made_ace += o.made_ace;
        self.dd += o.dd;
        self.dd_worse += o.dd_worse;
        self.dd_better += o.dd_better;
        self.margin += o.margin;
        self.ruffed += o.ruffed;
    }
}

type Out = BTreeMap<(usize, usize), Tally>;

pub fn run(deals: u64, seed: u64, threads: u64) {
    let parts: Vec<(Out, u64)> = std::thread::scope(|scope| {
        let handles: Vec<_> = (0..threads)
            .map(|t| {
                scope.spawn(move || {
                    let mut out = Out::new();
                    let mut suit = 0;
                    let mut d = t;
                    while d < deals {
                        suit += u64::from(deal(seed + d, &mut out));
                        d += threads;
                    }
                    (out, suit)
                })
            })
            .collect();
        handles.into_iter().map(|h| h.join().unwrap()).collect()
    });
    let mut all = Out::new();
    let mut suit = 0;
    for (part, n) in parts {
        suit += n;
        for (k, t) in part {
            all.entry(k).or_default().add(&t);
        }
    }
    println!("{deals} deals, {suit} played in a suit. The declaring side about to lead a side ace in tricks 1–3 while holding the");
    println!("top trump left, the opponents holding trumps: each played out both ways (the top trump, or the ace) by The Oracle.\n");
    println!("| Trick | Opponents' trumps | n | Tricks (top trump − ace, 95%) | Points (95%) | Bid made: trump / ace | Ace trumped | Solver: trump − ace | Solver: trump worse / better |");
    println!("|---|---|---|---|---|---|---|---|---|");
    let row = |trick: &str, held: &str, t: &Tally| {
        let n = t.n.max(1) as f64;
        let ci = |sum: f64, sum2: f64| 1.96 * ((sum2 / n - (sum / n).powi(2)).max(0.0) / n).sqrt();
        println!(
            "| {trick} | {held} | {} | {:+.2} ± {:.2} | {:+.1} ± {:.0} | {:.0}% / {:.0}% | {:.0}% | {:+.2} | {:.0}% / {:.0}% |",
            t.n,
            t.tricks / n,
            ci(t.tricks, t.tricks2),
            t.points / n,
            ci(t.points, t.points2),
            100.0 * t.made_trump as f64 / n,
            100.0 * t.made_ace as f64 / n,
            100.0 * t.ruffed as f64 / n,
            t.dd / n,
            100.0 * t.dd_worse as f64 / n,
            100.0 * t.dd_better as f64 / n
        );
    };
    let held = |h: usize| if h >= 4 { "4+".to_string() } else { h.to_string() };
    for ((k, h), t) in &all {
        row(&(k + 1).to_string(), &held(*h), t);
    }
    let mut by_held: BTreeMap<usize, Tally> = BTreeMap::new();
    let mut total = Tally::default();
    for ((_, h), t) in &all {
        by_held.entry(*h).or_default().add(t);
        total.add(t);
    }
    for (h, t) in &by_held {
        row("1–3", &held(*h), t);
    }
    row("1–3", "all", &total);
    println!("  (average search margin for the ace: {:.1} points)", total.margin / total.n.max(1) as f64);
}

fn bots(seed: u64) -> [SearchBot; 4] {
    std::array::from_fn(|i| SearchBot::new("The Oracle", SearchConfig::ORACLE, seed.wrapping_mul(4).wrapping_add(i as u64)))
}

/// Plays the hand on; the leader's side's tricks and points (ours minus theirs), whether the bid
/// was made, and whether the first card led was trumped.
fn finish(mut game: Game, seed: u64, team: fh_core::Team, led: Card) -> (i32, i32, bool, bool) {
    let before = game.scores();
    let trump = game.contract().and_then(|(_, c)| c.trump());
    let mut bots = bots(seed);
    let mut ruffed = false;
    let first = game.tricks().len();
    loop {
        let seat = game.to_act().expect("hand in progress");
        let action = bots[seat.index()].choose(&game.observe(seat), &game.legal_actions());
        if let Action::Play(p) = action {
            if game.tricks().len() == first && p.card != led && is_trump(p.card, trump) {
                ruffed = true;
            }
        }
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
    (record.tricks_won[us] as i32, points, made, ruffed)
}

/// One deal; true if it was played in a suit.
fn deal(seed: u64, out: &mut Out) -> bool {
    let mut game = Game::new(Rules::default(), seed);
    let mut players = bots(seed);
    loop {
        let Some(seat) = game.to_act() else { return false };
        if game.phase() == Phase::Playing {
            let Some((declarer, contract)) = game.contract() else { return false };
            let Some(suit) = contract.trump() else { return false };
            let trump = Some(suit);
            let k = game.tricks().len();
            if k >= 3 {
                return true;
            }
            let leading = game.current_trick().is_none_or(|t| t.cards().is_empty());
            let hand = game.hand(seat);
            let played: fh_core::CardSet = game.tricks().iter().flat_map(|t| t.cards().iter().copied()).collect();
            let boss = suit_order(suit, trump).into_iter().find(|c| !played.contains(*c));
            let opponents: usize = [seat.next(), seat.partner().next()].iter().map(|&s| game.hand(s).iter().filter(|&c| is_trump(c, trump)).count()).sum();
            if leading && seat.team() == declarer.team() && boss.is_some_and(|b| hand.contains(b)) && opponents > 0 {
                let boss = boss.unwrap();
                let obs = game.observe(seat);
                let legal = game.legal_actions();
                let ranked = players[seat.index()].evaluate_plays(&obs, &legal);
                let side_ace = |a: &Action| matches!(a, Action::Play(p) if p.card.rank() == ACE && !is_trump(p.card, trump));
                if side_ace(&ranked[0].0) {
                    let (ace, ace_ev) = ranked[0];
                    let top = Action::Play(fh_core::Play::card(boss));
                    let top_ev = ranked.iter().find(|(a, _)| *a == top).map_or(0.0, |&(_, ev)| ev);
                    let Action::Play(ace_play) = ace else { unreachable!() };
                    let team = seat.team();
                    let cont = seed.wrapping_add(1_000_003);
                    let mut a = game.clone();
                    a.apply(top).expect("legal");
                    let mut b = game.clone();
                    b.apply(ace).expect("legal");
                    let (ta, pa, ma, _) = finish(a, cont, team, boss);
                    let (tb, pb, mb, ruffed) = finish(b, cont, team, ace_play.card);
                    let hands: [fh_core::CardSet; 4] = std::array::from_fn(|i| game.hand(Seat::new(i)));
                    let values = Solver::new(trump).analyze(&Position { hands, trump, trick: Trick::new(seat) }, team);
                    let value = |act: Action| values.iter().filter(|(p, _)| Action::Play(*p) == act).map(|&(_, v)| v).max().unwrap_or(0);
                    let dd = value(top) as i32 - value(ace) as i32;
                    let t = out.entry((k, opponents.min(4))).or_default();
                    t.n += 1;
                    t.tricks += (ta - tb) as f64;
                    t.points += (pa - pb) as f64;
                    t.tricks2 += ((ta - tb) as f64).powi(2);
                    t.points2 += ((pa - pb) as f64).powi(2);
                    t.made_trump += u32::from(ma);
                    t.made_ace += u32::from(mb);
                    t.ruffed += u32::from(ruffed);
                    t.dd += dd as f64;
                    t.dd_worse += u32::from(dd < 0);
                    t.dd_better += u32::from(dd > 0);
                    t.margin += ace_ev - top_ev;
                    return true;
                }
                game.apply(ranked[0].0).expect("legal");
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
