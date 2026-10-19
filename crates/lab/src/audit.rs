//! `fh-lab audit`: is the coach fair and steady?
//!
//! Fair: South is played by an expert (The Oracle), a solid player (Steady) and a beginner
//! (Rookie), with The Oracle in the other seats, and the coach grades every South decision that
//! had a real choice. An expert should rarely earn a "mistake" or "blunder"; a beginner often. If
//! experts collect many, the cut-offs are too harsh.
//!
//! Steady: for each graded card play, the coach is asked again from a different random start. The
//! hint should agree with itself, or at least name the other answer as a close call.

use fh_ai::coach::{Coach, Verdict};
use fh_core::{Event, Game, Phase, Rules, Seat};

const VERDICTS: [Verdict; 5] = [Verdict::Best, Verdict::Good, Verdict::Inaccuracy, Verdict::Mistake, Verdict::Blunder];

#[derive(Default, Clone)]
struct Tally {
    /// By phase (bidding, kitty, play), then verdict.
    verdicts: [[usize; 5]; 3],
    loss: [f64; 3],
    /// Card plays asked twice: same answer, consistent (each names the other as a close call or
    /// the same), or conflicting.
    same: usize,
    consistent: usize,
    conflicting: usize,
}

impl Tally {
    fn add(&mut self, other: &Tally) {
        for p in 0..3 {
            for v in 0..5 {
                self.verdicts[p][v] += other.verdicts[p][v];
            }
            self.loss[p] += other.loss[p];
        }
        self.same += other.same;
        self.consistent += other.consistent;
        self.conflicting += other.conflicting;
    }
}

pub fn run(deals: u64, seed: u64, threads: u64) {
    for south in ["oracle", "steady", "rookie"] {
        let tally = std::thread::scope(|scope| {
            let handles: Vec<_> = (0..threads)
                .map(|t| {
                    scope.spawn(move || {
                        let mut tally = Tally::default();
                        let mut d = t;
                        while d < deals {
                            tally.add(&one(seed + d, south));
                            d += threads;
                        }
                        tally
                    })
                })
                .collect();
            let mut total = Tally::default();
            for h in handles {
                total.add(&h.join().unwrap());
            }
            total
        });
        println!("## South played by {south} ({deals} deals)\n");
        println!("| Decisions | n | Best | Good | Inaccuracy | Mistake | Blunder | Points given up per decision |");
        println!("|---|---|---|---|---|---|---|---|");
        for (p, name) in ["Bidding", "Kitty", "Card play"].iter().enumerate() {
            let n: usize = tally.verdicts[p].iter().sum();
            if n == 0 {
                continue;
            }
            let cells: Vec<String> = tally.verdicts[p].iter().map(|&c| format!("{:.0}%", 100.0 * c as f64 / n as f64)).collect();
            println!("| {name} | {n} | {} | {:.1} |", cells.join(" | "), tally.loss[p] / n as f64);
        }
        let asked = tally.same + tally.consistent + tally.conflicting;
        if asked > 0 {
            let pct = |x: usize| 100.0 * x as f64 / asked as f64;
            println!(
                "\nHints asked twice on {asked} card plays: same answer {:.0}%, a close call either way {:.0}%, conflicting {:.0}%.\n",
                pct(tally.same),
                pct(tally.consistent),
                pct(tally.conflicting)
            );
        }
    }
}

fn one(seed: u64, south: &str) -> Tally {
    let mut tally = Tally::default();
    let mut game = Game::new(Rules::default(), seed);
    let mut bots: [Box<dyn fh_ai::Bot>; 4] =
        std::array::from_fn(|i| crate::make_bot(if i == 0 { south } else { "oracle" }, seed.wrapping_mul(4).wrapping_add(i as u64)));
    loop {
        let Some(seat) = game.to_act() else { return tally };
        let obs = game.observe(seat);
        let legal = game.legal_actions();
        let action = bots[seat.index()].choose(&obs, &legal);
        if seat == Seat::SOUTH && legal.len() > 1 {
            let n = game.history().len() as u64 * 64 + obs.tricks.len() as u64 * 4 + obs.current_trick.as_ref().map_or(0, |t| t.cards().len()) as u64;
            let grade = Coach::new(seed ^ n).grade(&obs, &legal, action);
            let phase = match obs.phase {
                Phase::Bidding => 0,
                Phase::Discarding => 1,
                _ => 2,
            };
            tally.verdicts[phase][VERDICTS.iter().position(|&v| v == grade.verdict).unwrap()] += 1;
            tally.loss[phase] += grade.loss;
            if phase == 2 {
                // The hint again, from another random start.
                let first = Coach::new(seed ^ n).analyze(&obs, &legal, None);
                let again = Coach::new(seed ^ n ^ 0x005E_C04D).analyze(&obs, &legal, None);
                if first.best == again.best {
                    tally.same += 1;
                } else if first.close.contains(&again.best) || again.close.contains(&first.best) {
                    tally.consistent += 1;
                } else {
                    tally.conflicting += 1;
                }
            }
        }
        let events = game.apply(action).unwrap();
        if events.iter().any(|e| matches!(e, Event::HandScored { .. } | Event::GameWon { .. })) {
            return tally;
        }
    }
}
