//! `fh-lab bench`: double-dummy solver speed on full ten-card deals.

use fh_ai::cardplay::card_cost;
use fh_ai::solver::{Position, Solver};
use fh_core::{Card, CardSet, Game, Rules, Seat, Strain, Team, Trick};
use std::time::Instant;

pub fn run(deals: u64, seed: u64) {
    let mut times = Vec::new();
    let mut analyze_times = Vec::new();
    let mut nodes = 0u64;
    for d in 0..deals {
        let game = Game::new(Rules::default(), seed + d);
        let strain = Strain::ALL[(d % 5) as usize];
        let trump = strain.trump();
        // South declares: takes the kitty and drops its three cheapest cards.
        let mut hands: [CardSet; 4] = std::array::from_fn(|i| game.hand(Seat::new(i)));
        let mut south: Vec<Card> = hands[0].union(game.kitty()).iter().collect();
        south.sort_by_key(|&c| card_cost(c, trump));
        hands[0] = south[3..].iter().copied().collect();
        let pos = Position { hands, trump, trick: Trick::new(Seat::SOUTH) };

        let start = Instant::now();
        let mut solver = Solver::new(trump);
        solver.solve(&pos, Team::NorthSouth);
        times.push(start.elapsed().as_secs_f64() * 1000.0);
        nodes += solver.nodes;

        let start = Instant::now();
        Solver::new(trump).analyze(&pos, Team::NorthSouth);
        analyze_times.push(start.elapsed().as_secs_f64() * 1000.0);
    }
    report("solve (best value only)", &mut times);
    report("analyze (value of every lead)", &mut analyze_times);
    println!("mean nodes per solve: {:.0}", nodes as f64 / deals as f64);
}

fn report(label: &str, times: &mut [f64]) {
    times.sort_by(f64::total_cmp);
    let pct = |p: f64| times[((times.len() - 1) as f64 * p) as usize];
    let mean = times.iter().sum::<f64>() / times.len() as f64;
    println!(
        "{label:<32} mean {mean:7.2} ms   median {:7.2}   p90 {:7.2}   p99 {:7.2}   max {:7.2}",
        pct(0.5),
        pct(0.9),
        pct(0.99),
        times[times.len() - 1]
    );
}
