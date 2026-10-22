//! `fh-lab levels`: how high to bid, by seat. Competition is what the no-competition rule ("go up a
//! level only at about 80%") leaves out: in once-around bidding, the players still to call can
//! outbid you.
//!
//! For the first player in each deal with a biddable hand (a rule-of-thumb count of 6½ or more in
//! some suit), try passing and each level 7–10 in that suit. The Oracle finishes the auction for
//! everyone; the declarer takes the kitty and discards as the bots do, and the hand is solved with
//! every card visible, scored for real play at half a trick below the solver (a solver result of
//! exactly the bid counts as half made). Comparing the options on the same deal shows which level
//! scores best for the bidder's side, by count and by how many players are still to call.

use fh_ai::rule_bot::heuristic_discards;
use fh_ai::solver::{Position, Solver};
use fh_core::game::score_hand;
use fh_core::{Action, Call, CardSet, Contract, Game, Phase, Rules, Seat, Strain, Trick};

/// Passing, then levels 7–10.
const OPTIONS: usize = 5;

struct Row {
    /// Players still to call after this one (3 = first to speak, 0 = last).
    still_to_call: usize,
    count: f32,
    /// Someone on the other side made a real bid before this call.
    opponent_bid: bool,
    /// Points for the bidder's side minus the other side's, per option (None = not legal).
    points: [Option<f64>; OPTIONS],
}

/// With a score given, options are valued by the chance of winning the game (in percent) for the
/// bidder's side rather than by points: certain when the hand takes someone to 500, otherwise
/// estimated from the score gap.
#[derive(Clone, Copy)]
pub struct Score {
    pub us: i32,
    pub them: i32,
}

pub fn run(deals: u64, seed: u64, threads: u64, score: Option<Score>) {
    let rows: Vec<Row> = std::thread::scope(|scope| {
        let handles: Vec<_> = (0..threads)
            .map(|t| {
                scope.spawn(move || {
                    let mut rows = Vec::new();
                    let mut d = t;
                    while d < deals {
                        rows.extend(one(seed + d, score));
                        d += threads;
                    }
                    rows
                })
            })
            .collect();
        handles.into_iter().flat_map(|h| h.join().unwrap()).collect()
    });
    println!("{} bidding decisions from {deals} deals (the first biddable hand in each)", rows.len());
    match score {
        Some(s) => println!("Your side at {}, theirs at {}: values are your side's chance of winning the game (%).\n", s.us, s.them),
        None => println!("Values are points for your side minus theirs.\n"),
    }
    let name = |i: usize| if i == 0 { "pass".to_string() } else { format!("bid {}", i + 6) };
    report("All positions", &rows.iter().collect::<Vec<_>>(), &name);
    for still in (0..4).rev() {
        let label = match still {
            3 => "First to speak (3 players still to call)",
            2 => "Second to speak (2 still to call)",
            1 => "Third to speak (1 still to call)",
            _ => "Last to speak (no one can outbid you)",
        };
        let picked: Vec<&Row> = rows.iter().filter(|r| r.still_to_call == still && !r.opponent_bid).collect();
        report(&format!("{label}, no opponent has bid"), &picked, &name);
    }
    let picked: Vec<&Row> = rows.iter().filter(|r| r.opponent_bid).collect();
    report("After an opponent's bid", &picked, &name);
}

/// Average points per option by count; the best option in bold.
fn report(title: &str, rows: &[&Row], name: &dyn Fn(usize) -> String) {
    println!("## {title} ({} decisions)\n", rows.len());
    let header: Vec<String> = (0..OPTIONS).map(name).collect();
    println!("| Count | n | {} | Best |", header.join(" | "));
    println!("|---|---|{}---|", "---|".repeat(OPTIONS));
    let mut e = 6.5;
    while e <= 11.01 {
        let bucket: Vec<&&Row> = rows.iter().filter(|r| r.count >= e - 0.25 && r.count < e + 0.25).collect();
        if bucket.len() >= 40 {
            // Average only over decisions where the option was legal; an option legal in under
            // 80% of the bucket isn't compared.
            let means: Vec<Option<f64>> = (0..OPTIONS)
                .map(|i| {
                    let values: Vec<f64> = bucket.iter().filter_map(|r| r.points[i]).collect();
                    (values.len() * 5 >= bucket.len() * 4).then(|| values.iter().sum::<f64>() / values.len() as f64)
                })
                .collect();
            let best = (0..OPTIONS).filter(|&i| means[i].is_some()).max_by(|&a, &b| means[a].unwrap().total_cmp(&means[b].unwrap()));
            let cells: Vec<String> = means
                .iter()
                .enumerate()
                .map(|(i, m)| match m {
                    Some(v) if Some(i) == best => format!("**{v:+.0}**"),
                    Some(v) => format!("{v:+.0}"),
                    None => "—".into(),
                })
                .collect();
            println!("| {e:.1} | {} | {} | {} |", bucket.len(), cells.join(" | "), best.map_or("—".into(), name));
        }
        e += 0.5;
    }
    println!();
}

fn bots(seed: u64) -> [Box<dyn fh_ai::Bot>; 4] {
    std::array::from_fn(|i| crate::make_bot("oracle", seed.wrapping_mul(4).wrapping_add(i as u64)))
}

fn one(seed: u64, score: Option<Score>) -> Option<Row> {
    let rules = Rules::default();
    let mut game = Game::new(rules, seed);
    let mut bidders = bots(seed);
    while game.phase() == Phase::Bidding && game.history().is_empty() {
        let seat = game.to_act()?;
        let obs = game.observe(seat);
        let (strain, count) = Strain::ALL
            .into_iter()
            .filter(|s| s.trump().is_some())
            .map(|s| (s, fh_ai::thumb::count(&obs, s).0))
            .max_by(|a, b| a.1.total_cmp(&b.1))?;
        let legal = game.legal_actions();
        let bid = |level| Action::Call(Call::Bid(Contract::new(level, strain)));
        if count >= 6.5 && legal.contains(&bid(10)) && !obs.auction.must_bid() {
            if let Some(s) = score {
                // The studied player's side is "us". (The Oracle's earlier calls don't depend on
                // the score, so setting it here changes nothing before this call.)
                let mut scores = [s.them, s.them];
                scores[seat.team().index()] = s.us;
                game.set_scores(scores);
            }
            let made = obs.auction.calls().count();
            let opponent_bid = obs.auction.calls().any(|(s, c)| s.team() != seat.team() && matches!(c, Call::Bid(_)));
            let options: [Action; OPTIONS] = [Action::Call(Call::Pass), bid(7), bid(8), bid(9), bid(10)];
            let points = options.map(|option| legal.contains(&option).then(|| finish(&game, option, seat, seed, score.is_some())).flatten());
            return Some(Row { still_to_call: 3 - made, count, opponent_bid, points });
        }
        let action = bidders[seat.index()].choose(&obs, &legal);
        game.apply(action).ok()?;
    }
    None
}

/// Plays out `option` for `seat`: the auction by the Oracle, the hand double-dummy. Points for the
/// seat's side minus the other side's; a thrown-in deal scores 0.
pub(crate) fn finish(start: &Game, option: Action, seat: Seat, seed: u64, by_win_chance: bool) -> Option<f64> {
    let rules = Rules::default();
    let mut game = start.clone();
    let mut bidders = bots(seed ^ 0xB1D);
    game.apply(option).ok()?;
    while game.phase() == Phase::Bidding {
        if !game.history().is_empty() {
            // Passed out: redealt, the score unchanged.
            return Some(if by_win_chance { win_percent(game.scores(), None, seat) } else { 0.0 });
        }
        let to_act = game.to_act()?;
        let action = bidders[to_act.index()].choose(&game.observe(to_act), &game.legal_actions());
        game.apply(action).ok()?;
    }
    let (declarer, contract) = game.contract()?;
    let trump = contract.trump();
    while game.phase() == Phase::Discarding {
        let hand = game.hand(declarer);
        let card = heuristic_discards(hand, trump, hand.len() - 10)[0];
        game.apply(Action::Discard(card)).ok()?;
    }
    let hands: [CardSet; 4] = std::array::from_fn(|i| game.hand(Seat::new(i)));
    let leader = game.to_act()?;
    let dd = Solver::new(trump).solve(&Position { hands, trump, trick: Trick::new(leader) }, declarer.team());
    let side = declarer.team();
    let mut total = 0.0;
    for real in [dd, dd.saturating_sub(1)] {
        let mut tricks = [0u8; 2];
        tricks[side.index()] = real;
        tricks[side.other().index()] = 10 - real;
        let change = score_hand(&rules, contract, side, tricks, game.scores());
        let us = seat.team().index();
        total += 0.5
            * if by_win_chance {
                let before = game.scores();
                win_percent([before[0] + change[0], before[1] + change[1]], Some(side), seat)
            } else {
                (change[us] - change[1 - us]) as f64
            };
    }
    Some(total)
}

/// The chance (percent) that `seat`'s side wins the game from `scores` after a hand declared by
/// `declarers` (declarers count first at 500); a thrown-in deal has no declarers.
fn win_percent(scores: [i32; 2], declarers: Option<fh_core::Team>, seat: Seat) -> f64 {
    let rules = Rules::default();
    let first = declarers.unwrap_or(seat.team());
    100.0 * fh_ai::search_bot::win_chance(&rules, scores, first, seat.team(), None)
}
