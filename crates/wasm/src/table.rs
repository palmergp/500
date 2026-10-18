//! A game table for the browser: the human sits South, bots fill the other seats.
//! Plain Rust (no JS types), so it is tested natively; `lib.rs` wraps it for wasm-bindgen.

use crate::view::{event_view, EventView, View};
use fh_ai::Bot;
use fh_core::{Action, Game, Phase, Rules, Seat};
use serde::{Deserialize, Serialize};

pub const HUMAN: Seat = Seat::SOUTH;

/// Everything needed to rebuild a game: the seed, who sits where, and the index of every action
/// taken within the legal-action list at the time.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SaveData {
    pub version: u32,
    pub seed: u64,
    /// Persona ids for South (always "human"), West, North, East.
    pub players: [String; 4],
    pub log: Vec<u16>,
}

pub struct Table {
    game: Game,
    seed: u64,
    players: [String; 4],
    bots: [Option<Box<dyn Bot>>; 4],
    log: Vec<u16>,
}

impl Table {
    /// `west`, `north` (your partner) and `east` are persona ids.
    pub fn new(seed: u64, west: &str, north: &str, east: &str) -> Result<Table, String> {
        let players = ["human".to_string(), west.to_string(), north.to_string(), east.to_string()];
        Table::build(seed, players)
    }

    fn build(seed: u64, players: [String; 4]) -> Result<Table, String> {
        let mut bots: [Option<Box<dyn Bot>>; 4] = Default::default();
        for seat in [Seat::WEST, Seat::NORTH, Seat::EAST] {
            let id = &players[seat.index()];
            let bot = fh_ai::make_bot(id, seed.wrapping_mul(4).wrapping_add(seat.index() as u64));
            bots[seat.index()] = Some(bot.ok_or_else(|| format!("unknown player type {id:?}"))?);
        }
        Ok(Table { game: Game::new(Rules::default(), seed), seed, players, bots, log: Vec::new() })
    }

    pub fn game(&self) -> &Game {
        &self.game
    }

    pub fn players(&self) -> &[String; 4] {
        &self.players
    }

    pub fn view(&self) -> View {
        View::build(self)
    }

    pub fn human_to_act(&self) -> bool {
        self.game.to_act() == Some(HUMAN)
    }

    /// The human's action, by its index in `view().legal`.
    pub fn act(&mut self, index: usize) -> Result<Vec<EventView>, String> {
        if !self.human_to_act() {
            return Err("it is not your turn".into());
        }
        self.apply_index(index)
    }

    /// Lets the bot whose turn it is act. Returns `None` if it is the human's turn or the game is over.
    pub fn step_bot(&mut self) -> Option<Vec<EventView>> {
        let seat = self.game.to_act()?;
        let bot = self.bots[seat.index()].as_mut()?;
        let legal = self.game.legal_actions();
        let action = bot.choose(&self.game.observe(seat), &legal);
        let index = legal.iter().position(|&a| a == action).expect("bots choose legal actions");
        Some(self.apply_index(index).expect("legal index"))
    }

    fn apply_index(&mut self, index: usize) -> Result<Vec<EventView>, String> {
        let legal = self.game.legal_actions();
        let action: Action = *legal.get(index).ok_or_else(|| format!("no legal action #{index}"))?;
        let events = self.game.apply(action).map_err(|e| e.to_string())?;
        self.log.push(index as u16);
        Ok(events.iter().map(event_view).collect())
    }

    pub fn save(&self) -> SaveData {
        SaveData { version: 1, seed: self.seed, players: self.players.clone(), log: self.log.clone() }
    }

    pub fn restore(save: &SaveData) -> Result<Table, String> {
        if save.version != 1 {
            return Err(format!("unsupported save version {}", save.version));
        }
        let mut table = Table::build(save.seed, save.players.clone())?;
        for &index in &save.log {
            if table.game.phase() == Phase::GameOver {
                return Err("save continues past the end of the game".into());
            }
            table.apply_index(index as usize)?;
        }
        Ok(table)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Plays the human seat with the first legal action until the game ends.
    fn auto_play(table: &mut Table) {
        for _ in 0..100_000 {
            if table.game().phase() == Phase::GameOver {
                return;
            }
            if table.human_to_act() {
                let view = table.view();
                assert!(!view.legal.is_empty());
                // Prefer passing, then the first card: keeps the human from bidding wildly.
                let pick = view.legal.iter().position(|a| a.kind == "pass").unwrap_or(0);
                table.act(view.legal[pick].index).unwrap();
            } else {
                assert!(table.step_bot().is_some());
            }
        }
        panic!("game did not finish");
    }

    #[test]
    fn plays_a_full_game() {
        let mut table = Table::new(7, "rookie", "steady", "gambler").unwrap();
        auto_play(&mut table);
        let view = table.view();
        assert_eq!(view.phase, "gameOver");
        assert!(view.winner.is_some());
        assert!(!view.history.is_empty());
        assert!(table.step_bot().is_none());
    }

    #[test]
    fn save_and_restore_rebuild_the_same_position() {
        let mut table = Table::new(11, "rock", "steady", "rock").unwrap();
        for _ in 0..137 {
            if table.human_to_act() {
                table.act(0).unwrap();
            } else {
                table.step_bot();
            }
        }
        let save = table.save();
        let json = serde_json::to_string(&save).unwrap();
        let restored = Table::restore(&serde_json::from_str(&json).unwrap()).unwrap();
        assert_eq!(serde_json::to_value(restored.view()).unwrap(), serde_json::to_value(table.view()).unwrap());
    }

    #[test]
    fn rejects_unknown_players_and_out_of_turn_actions() {
        assert!(Table::new(1, "nobody", "steady", "steady").is_err());
        let mut table = Table::new(1, "steady", "steady", "steady").unwrap();
        while !table.human_to_act() && table.step_bot().is_some() {}
        assert!(table.human_to_act());
        assert!(table.act(999).is_err());
    }

    #[test]
    fn view_hides_other_hands_and_the_kitty() {
        let table = Table::new(3, "steady", "steady", "steady").unwrap();
        let view = table.view();
        assert_eq!(view.hand.len(), 10);
        assert!(view.kitty.is_none());
        let json = serde_json::to_string(&view).unwrap();
        for seat in [Seat::WEST, Seat::NORTH, Seat::EAST] {
            for card in table.game().hand(seat).iter() {
                assert!(!view.hand.iter().any(|c| c.id == card.id()), "{card} leaked");
            }
        }
        assert!(json.contains("\"handCounts\":[10,10,10,10]"));
    }
}
