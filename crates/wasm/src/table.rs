//! A game table: people sit in seats marked "human", bots fill the rest. In the browser the one
//! person sits South; on a LAN host (`fh-host`) any seat can be a person.
//! Plain Rust (no JS types), so it is tested natively; `lib.rs` wraps it for wasm-bindgen.

use crate::view::{event_view, EventView, Lens, View};
use fh_ai::Bot;
use fh_core::{Action, Event, Game, Phase, Rules, Seat};
use serde::{Deserialize, Serialize};

pub const HUMAN: Seat = Seat::SOUTH;

/// Everything needed to rebuild a game: the seed, who sits where, and the index of every action
/// taken within the legal-action list at the time.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SaveData {
    pub version: u32,
    pub seed: u64,
    /// Persona ids for South, West, North, East; "human" for a seat a person plays.
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

    /// A new game with `players` by seat (South, West, North, East): persona ids, or "human".
    pub fn with_players(seed: u64, players: [String; 4]) -> Result<Table, String> {
        Table::build(seed, players)
    }

    fn build(seed: u64, players: [String; 4]) -> Result<Table, String> {
        let mut bots: [Option<Box<dyn Bot>>; 4] = Default::default();
        for seat in Seat::ALL {
            bots[seat.index()] = make_bot(&players[seat.index()], seed, seat)?;
        }
        Ok(Table { game: Game::new(Rules::default(), seed), seed, players, bots, log: Vec::new() })
    }

    /// Puts a person ("human") or a bot persona in `seat`, mid-game if need be.
    pub fn set_player(&mut self, seat: Seat, id: &str) -> Result<(), String> {
        self.bots[seat.index()] = make_bot(id, self.seed, seat)?;
        self.players[seat.index()] = id.to_string();
        Ok(())
    }

    pub fn game(&self) -> &Game {
        &self.game
    }

    pub fn players(&self) -> &[String; 4] {
        &self.players
    }

    /// Actions taken so far (the length of the save log).
    pub fn action_count(&self) -> usize {
        self.log.len()
    }

    pub fn view(&self) -> View {
        View::build(self)
    }

    /// South's lens: "You", then the bots' persona names.
    pub fn lens(&self) -> Lens {
        Lens::new(
            Seat::SOUTH,
            std::array::from_fn(|i| {
                if i == 0 {
                    "You".to_string()
                } else {
                    fh_ai::persona(&self.players[i]).map_or(self.players[i].clone(), |p| p.name.to_string())
                }
            }),
        )
    }

    /// Whether South, the browser's one person, is to act.
    pub fn human_to_act(&self) -> bool {
        self.game.to_act() == Some(HUMAN)
    }

    /// Whether a bot is to act (so the game moves without anyone).
    pub fn bot_to_act(&self) -> bool {
        self.game.to_act().is_some_and(|s| self.bots[s.index()].is_some())
    }

    /// The action of the person in `seat`, by its index in their view's `legal` list.
    pub fn act_as(&mut self, seat: Seat, index: usize) -> Result<Vec<Event>, String> {
        if self.game.to_act() != Some(seat) || self.bots[seat.index()].is_some() {
            return Err("it is not your turn".into());
        }
        self.apply_raw(index)
    }

    /// Lets the bot whose turn it is act. `None` if a person is to act or the game is over.
    pub fn step_bot_events(&mut self) -> Option<Vec<Event>> {
        let seat = self.game.to_act()?;
        let bot = self.bots[seat.index()].as_mut()?;
        let legal = self.game.legal_actions();
        let action = bot.choose(&self.game.observe(seat), &legal);
        let index = legal.iter().position(|&a| a == action).expect("bots choose legal actions");
        Some(self.apply_raw(index).expect("legal index"))
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
        let events = self.step_bot_events()?;
        let lens = self.lens();
        Some(events.iter().map(|e| event_view(e, &lens)).collect())
    }

    fn apply_index(&mut self, index: usize) -> Result<Vec<EventView>, String> {
        let events = self.apply_raw(index)?;
        let lens = self.lens();
        Ok(events.iter().map(|e| event_view(e, &lens)).collect())
    }

    fn apply_raw(&mut self, index: usize) -> Result<Vec<Event>, String> {
        let legal = self.game.legal_actions();
        let action: Action = *legal.get(index).ok_or_else(|| format!("no legal action #{index}"))?;
        let events = self.game.apply(action).map_err(|e| e.to_string())?;
        self.log.push(index as u16);
        Ok(events)
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
            table.apply_raw(index as usize)?;
        }
        Ok(table)
    }
}

/// The bot for `seat`, or `None` for a person. Each seat's bot gets its own seed.
fn make_bot(id: &str, seed: u64, seat: Seat) -> Result<Option<Box<dyn Bot>>, String> {
    if id == "human" {
        return Ok(None);
    }
    let bot = fh_ai::make_bot(id, seed.wrapping_mul(4).wrapping_add(seat.index() as u64));
    bot.map(Some).ok_or_else(|| format!("unknown player type {id:?}"))
}

/// The game after the first `actions` actions of a save (no bots needed).
pub fn replay(save: &SaveData, actions: usize) -> Result<Game, String> {
    let mut game = Game::new(Rules::default(), save.seed);
    for &index in save.log.iter().take(actions) {
        let legal = game.legal_actions();
        let action = *legal.get(index as usize).ok_or("save does not match the game")?;
        game.apply(action).map_err(|e| e.to_string())?;
    }
    Ok(game)
}

/// The game at the start of deal `number` (1-based, counting passed-out deals), and the number
/// of actions taken to get there.
pub fn replay_to_deal(save: &SaveData, number: usize) -> Result<(Game, usize), String> {
    let mut game = Game::new(Rules::default(), save.seed);
    for (i, &index) in save.log.iter().enumerate() {
        if game.history().len() + 1 >= number {
            return Ok((game, i));
        }
        let legal = game.legal_actions();
        let action = *legal.get(index as usize).ok_or("save does not match the game")?;
        game.apply(action).map_err(|e| e.to_string())?;
    }
    if game.history().len() + 1 >= number {
        let n = save.log.len();
        Ok((game, n))
    } else {
        Err(format!("no deal {number} in this save"))
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
    fn views_rotate_to_the_viewer() {
        let mut table = Table::new(9, "steady", "steady", "steady").unwrap();
        for _ in 0..30 {
            if table.human_to_act() {
                table.act(0).unwrap();
            } else {
                table.step_bot();
            }
        }
        let game = table.game();
        let names = ["S".to_string(), "W".to_string(), "N".to_string(), "E".to_string()];
        let east = crate::view::View::build_for(game, &Lens::new(Seat::EAST, names), table.players(), table.action_count());
        let south = table.view();
        // East sees their own hand, at index 0, with their side's score first.
        assert_eq!(east.hand.len(), game.hand(Seat::EAST).len());
        assert!(east.hand.iter().all(|c| game.hand(Seat::EAST).iter().any(|h| h.id() == c.id)));
        assert_eq!(east.hand_counts[0], game.hand(Seat::EAST).len());
        assert_eq!(east.scores, [south.scores[1], south.scores[0]]);
        assert_eq!(east.dealer, (south.dealer + 1) % 4, "South's seat 3 (East) is East's seat 0");
        assert_eq!(east.players[0].name, "E");
        assert_eq!(east.players[2].name, "W", "East's partner is West");
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
