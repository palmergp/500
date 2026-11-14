//! `fh-lab handpuzzles`: full-hand puzzles for Learn → Full hands. South has won the auction with a
//! hard bid (8 or more, as many tricks as the deal holds with every card seen) and plays it out from
//! the kitty against strong bots. A deal is kept only when it rewards thinking, not luck:
//!
//! - it can be made (the solver, every card seen, makes the bid after the usual kitty discard);
//! - the expert, seeing only its own cards, makes it every time (three runs, different samples of
//!   the hidden cards), so a line found by reasoning works against these defenders;
//! - a careless declarer (the steady rule bot) goes down, so there's something to find.
//!
//! The defenders and partner are The Oracle, seeded by the deal: given the same plays they answer
//! the same way, so a puzzle is the same puzzle every time.
//!
//!   fh-lab handpuzzles [--count 40] [--seed 7000000] [--threads N] [--out web/src/learn/handPuzzles.json]

use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::Mutex;

use fh_ai::rule_bot::set_discards;
use fh_ai::solver::{Position, Solver};
use fh_ai::{Bot, SearchBot, SearchConfig};
use fh_core::{Action, Call, CardSet, Contract, Phase, Seat, Strain, Trick};
use fh_wasm::table::{SaveData, Table};

const DEFENDERS: &str = "oracle";

pub fn run(count: usize, seed: u64, threads: u64, out: &str) {
    let found: Mutex<Vec<(u64, Contract, SaveData)>> = Mutex::new(Vec::new());
    let next = AtomicU64::new(seed);
    let tried = AtomicUsize::new(0);
    std::thread::scope(|scope| {
        for _ in 0..threads {
            scope.spawn(|| loop {
                if found.lock().unwrap().len() >= count {
                    return;
                }
                let s = next.fetch_add(1, Ordering::Relaxed);
                tried.fetch_add(1, Ordering::Relaxed);
                if let Some((contract, save)) = puzzle(s) {
                    let mut f = found.lock().unwrap();
                    if f.len() < count {
                        f.push((s, contract, save));
                        eprintln!("{} of {count}: deal {s}, {contract} ({} deals tried)", f.len(), tried.load(Ordering::Relaxed));
                    }
                }
            });
        }
    });
    let mut found = found.into_inner().unwrap();
    // Easier first: by level, then by deal.
    found.sort_by_key(|(s, c, _)| (c.level, *s));
    let list: Vec<serde_json::Value> = found
        .iter()
        .enumerate()
        .map(|(i, (_, contract, save))| {
            serde_json::json!({
                "id": i + 1,
                "contract": contract.to_string(),
                "save": serde_json::to_string(save).unwrap(),
            })
        })
        .collect();
    std::fs::write(out, serde_json::to_string_pretty(&list).unwrap() + "\n").unwrap();
    println!("{} puzzles written to {out} ({} deals tried)", list.len(), tried.load(Ordering::Relaxed));
}

/// The deal's puzzle, if it makes a good one: South's bid and the save at South's first discard.
fn puzzle(seed: u64) -> Option<(Contract, SaveData)> {
    let contract = hard_bid(seed)?;
    // The auction: everyone else passes, South bids. Every seat a person, so each call can be made.
    let mut table = Table::with_players(seed, ["human"; 4].map(String::from), Some(1)).ok()?;
    while table.game().phase() == Phase::Bidding {
        let seat = table.game().to_act()?;
        let want = if seat == Seat::SOUTH { Action::Call(Call::Bid(contract)) } else { Action::Call(Call::Pass) };
        let index = table.game().legal_actions().iter().position(|&a| a == want)?;
        table.act_as(seat, index).ok()?;
    }
    if table.game().contract() != Some((Seat::SOUTH, contract)) || !table.human_to_act() {
        return None;
    }
    let mut save = table.save();
    save.players = ["human", DEFENDERS, DEFENDERS, DEFENDERS].map(String::from);
    save.names = Some(["You", "West", "North", "East"].map(String::from));
    // Something to find: careless play goes down.
    let steady = fh_ai::make_bot("steady", seed)?;
    if play_out(&save, steady)? {
        return None;
    }
    // And reasoning, not a lucky guess, makes it: the expert, every time.
    for run in 0..3u64 {
        let expert = Box::new(SearchBot::new("expert", SearchConfig::COUNTER, seed ^ (run + 1).wrapping_mul(0x9E37_79B9_7F4A_7C15)));
        if !play_out(&save, expert)? {
            return None;
        }
    }
    Some((contract, save))
}

/// The hardest bid the deal holds for South: the strain whose solved result (every card seen, the
/// usual kitty discard, declarer to lead) scores most, at 8 or more.
fn hard_bid(seed: u64) -> Option<Contract> {
    let table = Table::with_players(seed, ["human"; 4].map(String::from), Some(1)).ok()?;
    let game = table.game();
    let thirteen = game.hand(Seat::SOUTH).union(game.kitty());
    let mut best: Option<Contract> = None;
    for strain in Strain::ALL {
        let trump = strain.trump();
        let discard: CardSet = set_discards(thirteen, trump, 3).into_iter().collect();
        let mut hands: [CardSet; 4] = std::array::from_fn(|i| game.hand(Seat::new(i)));
        hands[0] = thirteen.minus(discard);
        let mut solver = Solver::with_limit(trump, 6_000_000);
        let tricks = solver.solve(&Position { hands, trump, trick: Trick::new(Seat::SOUTH) }, Seat::SOUTH.team());
        if solver.aborted() || tricks < 8 {
            continue;
        }
        let contract = Contract::new(tricks.min(10), strain);
        if best.is_none_or(|b| contract.value() > b.value()) {
            best = Some(contract);
        }
    }
    best
}

/// Plays the hand out from `save` with `south` playing South's cards: whether the bid made.
fn play_out(save: &SaveData, mut south: Box<dyn Bot>) -> Option<bool> {
    let mut table = Table::restore(save).ok()?;
    while table.game().phase() != Phase::GameOver {
        if table.human_to_act() {
            let game = table.game();
            let legal = game.legal_actions();
            let action = south.choose(&game.observe(Seat::SOUTH), &legal);
            let index = legal.iter().position(|&a| a == action)?;
            table.act_as(Seat::SOUTH, index).ok()?;
        } else {
            table.step_bot_events()?;
        }
    }
    let hand = table.game().history().last()?;
    let (_, contract) = hand.contract?;
    Some(hand.tricks_won[Seat::SOUTH.team().index()] >= contract.level)
}
