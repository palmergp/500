//! Playing a position out to the end of its hand with The Oracle in every seat, for the lab's
//! "both ways" experiments: the same seed on both branches, so the only difference is the choice.

use fh_ai::search_bot::{SearchBot, SearchConfig};
use fh_ai::Bot;
use fh_core::{Event, Game, Phase, Seat};

/// How the hand came out for `seat`'s side.
#[derive(Clone, Copy, Debug)]
pub struct Outcome {
    /// Tricks its side took (0 if the deal was thrown in).
    pub tricks: i32,
    /// Its score change less the other side's.
    pub points: f64,
    /// The hand was bid and the bid made.
    pub made: bool,
    /// Its side declared.
    pub declared: bool,
}

/// Plays on from `game` (any phase: calls, discards and cards) until the hand is scored or thrown
/// in, with The Oracle in every seat.
pub fn finish(mut game: Game, seed: u64, seat: Seat) -> Option<Outcome> {
    let before = game.scores();
    let mut bots: [SearchBot; 4] = std::array::from_fn(|i| SearchBot::new("The Oracle", SearchConfig::ORACLE, seed.wrapping_mul(4).wrapping_add(i as u64)));
    let us = seat.team().index();
    loop {
        let to_act = game.to_act()?;
        let action = bots[to_act.index()].choose(&game.observe(to_act), &game.legal_actions());
        let events = game.apply(action).ok()?;
        if events.iter().any(|e| matches!(e, Event::PassedOut { .. })) {
            return Some(Outcome { tricks: 0, points: 0.0, made: false, declared: false });
        }
        if events.iter().any(|e| matches!(e, Event::HandScored { .. })) || game.phase() == Phase::GameOver {
            break;
        }
    }
    let after = game.scores();
    let record = game.history().last()?;
    let (declarer, _) = record.contract?;
    let points = f64::from((after[us] - before[us]) - (after[1 - us] - before[1 - us]));
    let made = after[declarer.team().index()] > before[declarer.team().index()];
    Some(Outcome { tricks: i32::from(record.tricks_won[us]), points, made, declared: declarer.team() == seat.team() })
}

/// A running mean with its 95% interval.
#[derive(Default, Clone, Copy, Debug)]
pub struct Mean {
    pub n: f64,
    sum: f64,
    sum2: f64,
}

impl Mean {
    pub fn add(&mut self, x: f64) {
        self.n += 1.0;
        self.sum += x;
        self.sum2 += x * x;
    }
    pub fn merge(&mut self, o: &Mean) {
        self.n += o.n;
        self.sum += o.sum;
        self.sum2 += o.sum2;
    }
    pub fn mean(&self) -> f64 {
        self.sum / self.n.max(1.0)
    }
    pub fn ci(&self) -> f64 {
        let n = self.n.max(1.0);
        1.96 * ((self.sum2 / n - self.mean().powi(2)).max(0.0) / n).sqrt()
    }
    pub fn show(&self, digits: usize) -> String {
        format!("{:+.digits$} ± {:.digits$}", self.mean(), self.ci())
    }
}
