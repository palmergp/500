//! `fh-lab kittyplan`: the set-based kitty plan (`set_discards`) against the one-card-at-a-time
//! plan (`greedy_discards`), on deals where they differ. `KITTY_SWEEP=1` tries a grid of weights. South declares a suit at the count's
//! level (as in `fh-lab declarer`); both plans are solved on the same deal with every card visible.

use fh_ai::rule_bot::{greedy_discards, set_discards_with, KittyWeights};
use fh_ai::solver::{Position, Solver};
use fh_core::{Action, Call, CardSet, Contract, Game, Rules, Seat, Strain, Trick};

pub fn run(deals: u64, seed: u64, threads: u64) {
    let sweep = std::env::var("KITTY_SWEEP").is_ok();
    let grid: Vec<KittyWeights> = if sweep {
        let mut g = Vec::new();
        for void_scale in [1.0, 1.5] {
            for king in [0.15] {
                for singleton in [0.15, 0.2, 0.3, 0.45] {
                    g.push(KittyWeights { void_scale, king, queen: 0.05, singleton });
                }
            }
        }
        g
    } else {
        vec![KittyWeights::DEFAULT]
    };
    for w in grid {
        println!("Weights {w:?}");
        run_one(deals, seed, threads, w);
    }
}

fn run_one(deals: u64, seed: u64, threads: u64, w: KittyWeights) {
    let parts: Vec<[f64; 7]> = std::thread::scope(|scope| {
        let handles: Vec<_> = (0..threads)
            .map(|t| {
                scope.spawn(move || {
                    let mut out = [0.0; 7];
                    let mut d = t;
                    while d < deals {
                        one(seed + d, &mut out, w);
                        d += threads;
                    }
                    out
                })
            })
            .collect();
        handles.into_iter().map(|h| h.join().unwrap()).collect()
    });
    let mut s = [0.0; 7];
    for p in parts {
        for i in 0..7 {
            s[i] += p[i];
        }
    }
    let [declared, n, new, old, new_made, old_made, sq] = s;
    let mean = (new - old) / n;
    let sd = (sq / n - mean * mean).max(0.0).sqrt();
    println!("{deals} deals, {declared} declared; the plans differ on {n} ({:.0}%)\n", 100.0 * n / declared);
    println!("| Plan | Tricks | Bid made |\n|---|---|---|");
    println!("| One card at a time (old) | {:.2} | {:.0}% |", old / n, 100.0 * old_made / n);
    println!("| As a set (new) | {:.2} | {:.0}% |", new / n, 100.0 * new_made / n);
    println!("\nNew minus old: {mean:+.3} tricks (95% ± {:.3})", 1.96 * sd / n.sqrt());
}

fn one(seed: u64, out: &mut [f64; 7], w: KittyWeights) {
    let mut game = Game::with_dealer(Rules::default(), seed, Seat::EAST);
    let obs = game.observe(Seat::SOUTH);
    let Some((strain, count)) = Strain::ALL.into_iter().filter(|s| s.trump().is_some()).map(|s| (s, fh_ai::thumb::count(&obs, s).0)).max_by(|a, b| a.1.total_cmp(&b.1)) else { return };
    if count < 7.0 {
        return;
    }
    let level = if count >= 8.5 { 8 } else { 7 };
    let trump = strain.trump();
    game.apply(Action::Call(Call::Bid(Contract::new(level, strain)))).unwrap();
    for _ in 0..3 {
        game.apply(Action::Call(Call::Pass)).unwrap();
    }
    out[0] += 1.0;
    let thirteen = game.hand(Seat::SOUTH);
    let old: CardSet = greedy_discards(thirteen, trump, 3).into_iter().collect();
    let new: CardSet = set_discards_with(thirteen, trump, 3, w).into_iter().collect();
    if old == new {
        return;
    }
    let others: [CardSet; 4] = std::array::from_fn(|i| game.hand(Seat::new(i)));
    let mut solver = Solver::with_limit(trump, 4_000_000);
    let mut solve = |d: CardSet| -> Option<u8> {
        let mut hands = others;
        hands[0] = thirteen.minus(d);
        let t = solver.solve(&Position { hands, trump, trick: Trick::new(Seat::SOUTH) }, Seat::SOUTH.team());
        (!solver.aborted()).then_some(t)
    };
    let (Some(a), Some(b)) = (solve(new), solve(old)) else { return };
    let made = |t: u8| if t > level { 1.0 } else if t == level { 0.5 } else { 0.0 };
    out[1] += 1.0;
    out[2] += a as f64;
    out[3] += b as f64;
    out[4] += made(a);
    out[5] += made(b);
    out[6] += (a as f64 - b as f64).powi(2);
}
