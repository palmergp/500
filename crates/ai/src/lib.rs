//! AI players for 500. Every bot receives only its own seat's [`Observation`], so it cannot see
//! hidden cards.

pub mod cardplay;
pub mod evaluate;
pub mod personas;
pub mod rule_bot;

use fh_core::{Action, Game, Observation, Phase, Rng, Team};

pub use personas::{make_bot, persona, Persona, PERSONAS};
pub use rule_bot::{RuleBot, Style};

pub trait Bot {
    fn name(&self) -> &str;
    /// Chooses one of `legal`, which is never empty.
    fn choose(&mut self, obs: &Observation, legal: &[Action]) -> Action;
}

/// Picks uniformly among the legal actions. Useful for testing, hopeless as an opponent.
pub struct RandomBot {
    rng: Rng,
}

impl RandomBot {
    pub fn new(seed: u64) -> RandomBot {
        RandomBot { rng: Rng::new(seed) }
    }
}

impl Bot for RandomBot {
    fn name(&self) -> &str {
        "Random"
    }

    fn choose(&mut self, _obs: &Observation, legal: &[Action]) -> Action {
        legal[self.rng.below(legal.len())]
    }
}

/// Plays `game` with `bots[seat index]` choosing for each seat, for at most `max_actions` actions.
/// Returns the winner, or `None` if the game was still going.
pub fn play_out(game: &mut Game, bots: &mut [Box<dyn Bot>; 4], max_actions: usize) -> Option<Team> {
    for _ in 0..max_actions {
        if game.phase() == Phase::GameOver {
            break;
        }
        let seat = game.to_act().expect("an unfinished game has a player to act");
        let legal = game.legal_actions();
        let action = bots[seat.index()].choose(&game.observe(seat), &legal);
        game.apply(action).unwrap_or_else(|e| panic!("{} chose an illegal action: {e}", bots[seat.index()].name()));
    }
    game.winner()
}
