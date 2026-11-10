//! Headless simulation: many bot-vs-bot games in parallel, with rule invariants checked after
//! every action and summary statistics at the end.

use crate::make_bot;
use fh_core::{Call, Game, Phase, Rules, Strain, Team};
use std::time::Instant;

pub struct Options {
    pub games: u64,
    pub seed: u64,
    pub ns: String,
    pub ew: String,
    pub threads: u64,
}

/// Cap per game: a safety net (games end at 500 on a made bid or at -500, H23–H24).
const MAX_HANDS: usize = 200;

#[derive(Default)]
struct Stats {
    games: u64,
    unfinished: u64,
    wins: [u64; 2],
    actions: u64,
    deals: u64,
    passed_out: u64,
    forced_bids: u64,
    contracts: u64,
    made: u64,
    slams: u64,
    by_level: [u64; 11],
    by_strain: [u64; 5],
    declarer_points: i64,
    defender_points: i64,
    mentions: u64,
    sixes: u64,
}

impl Stats {
    fn merge(&mut self, o: &Stats) {
        self.games += o.games;
        self.unfinished += o.unfinished;
        self.actions += o.actions;
        self.deals += o.deals;
        self.passed_out += o.passed_out;
        self.forced_bids += o.forced_bids;
        self.contracts += o.contracts;
        self.made += o.made;
        self.slams += o.slams;
        self.declarer_points += o.declarer_points;
        self.defender_points += o.defender_points;
        self.mentions += o.mentions;
        self.sixes += o.sixes;
        for i in 0..2 {
            self.wins[i] += o.wins[i];
        }
        for i in 0..11 {
            self.by_level[i] += o.by_level[i];
        }
        for i in 0..5 {
            self.by_strain[i] += o.by_strain[i];
        }
    }
}

fn play_one(seed: u64, opts: &Options, stats: &mut Stats) {
    let mut bots: [Box<dyn fh_ai::Bot>; 4] =
        std::array::from_fn(|i| make_bot(if i % 2 == 0 { &opts.ns } else { &opts.ew }, seed * 4 + i as u64));
    let mut game = Game::new(Rules::default(), seed);
    while game.phase() != Phase::GameOver && game.history().len() < MAX_HANDS {
        let seat = game.to_act().unwrap();
        let legal = game.legal_actions();
        let action = bots[seat.index()].choose(&game.observe(seat), &legal);
        game.apply(action).unwrap_or_else(|e| panic!("game {seed}: {} chose an illegal action: {e}", bots[seat.index()].name()));
        if let Err(problem) = game.check_invariants() {
            panic!("game {seed}: invariant broken after {action}: {problem}");
        }
        stats.actions += 1;
    }
    stats.games += 1;
    match game.winner() {
        Some(team) => stats.wins[team.index()] += 1,
        None => stats.unfinished += 1,
    }
    for hand in game.history() {
        stats.deals += 1;
        for (_, call) in &hand.calls {
            match call {
                Call::Mention(_) => stats.mentions += 1,
                Call::Six(_) => stats.sixes += 1,
                _ => {}
            }
        }
        let Some((declarer, contract)) = hand.contract else {
            stats.passed_out += 1;
            continue;
        };
        let (d, o) = (declarer.team().index(), declarer.team().other().index());
        stats.contracts += 1;
        stats.by_level[contract.level as usize] += 1;
        stats.by_strain[contract.strain.index()] += 1;
        if hand.tricks_won[d] >= contract.level {
            stats.made += 1;
        }
        if hand.tricks_won[d] == 10 {
            stats.slams += 1;
        }
        let others_bid = hand.calls.iter().any(|&(s, c)| s != declarer && matches!(c, Call::Bid(_)));
        if hand.deal_number == 3 && declarer == hand.dealer && !others_bid {
            stats.forced_bids += 1;
        }
        stats.declarer_points += hand.score_change[d] as i64;
        stats.defender_points += hand.score_change[o] as i64;
    }
}

pub fn run(opts: Options) {
    let start = Instant::now();
    let threads = opts.threads.clamp(1, 64);
    let mut total = Stats::default();
    std::thread::scope(|scope| {
        let handles: Vec<_> = (0..threads)
            .map(|t| {
                let opts = &opts;
                scope.spawn(move || {
                    let mut stats = Stats::default();
                    let mut g = t;
                    while g < opts.games {
                        play_one(opts.seed.wrapping_add(g), opts, &mut stats);
                        g += threads;
                    }
                    stats
                })
            })
            .collect();
        for handle in handles {
            total.merge(&handle.join().expect("simulation thread panicked"));
        }
    });
    report(&opts, &total, start.elapsed().as_secs_f64());
}

fn pct(n: u64, d: u64) -> f64 {
    if d == 0 {
        0.0
    } else {
        100.0 * n as f64 / d as f64
    }
}

fn report(opts: &Options, s: &Stats, secs: f64) {
    println!("{} games, North–South = {}, East–West = {}  ({secs:.1}s, {:.0} games/s)", s.games, opts.ns, opts.ew, s.games as f64 / secs);
    println!("All rule invariants held after each of {} actions.", s.actions);
    println!();
    println!("Wins          N–S {:.1}%   E–W {:.1}%   unfinished after {MAX_HANDS} deals: {}",
        pct(s.wins[Team::NorthSouth.index()], s.games), pct(s.wins[Team::EastWest.index()], s.games), s.unfinished);
    println!("Deals         {} ({:.1} per game); passed out {:.1}%; forced dealer bids {:.1}% of contracts",
        s.deals, s.deals as f64 / s.games.max(1) as f64, pct(s.passed_out, s.deals), pct(s.forced_bids, s.contracts));
    println!("Signals       {:.2} mentions and {:.2} sixes per deal", s.mentions as f64 / s.deals.max(1) as f64, s.sixes as f64 / s.deals.max(1) as f64);
    println!("Contracts     made {:.1}%, slams {:.1}%; per hand: declarers {:+.0}, defenders {:+.0}",
        pct(s.made, s.contracts), pct(s.slams, s.contracts),
        s.declarer_points as f64 / s.contracts.max(1) as f64, s.defender_points as f64 / s.contracts.max(1) as f64);
    let levels: Vec<String> = (7..=10).map(|l| format!("{l}: {:.1}%", pct(s.by_level[l], s.contracts))).collect();
    println!("Levels        {}", levels.join("   "));
    let strains: Vec<String> = Strain::ALL.iter().map(|st| format!("{st}: {:.1}%", pct(s.by_strain[st.index()], s.contracts))).collect();
    println!("Strains       {}", strains.join("   "));
}
