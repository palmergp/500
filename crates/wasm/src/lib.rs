//! wasm-bindgen bindings for the web app. All game logic lives in [`table`] and [`view`].

pub mod coach;
/// The worker's requests answered natively, for a host thinking on behalf of a phone.
#[cfg(not(target_arch = "wasm32"))]
pub mod service;
pub mod table;
pub mod view;

use serde::Serialize;
use table::{SaveData, Table};
use wasm_bindgen::prelude::*;

fn to_js<T: Serialize>(value: &T) -> Result<JsValue, JsError> {
    value.serialize(&serde_wasm_bindgen::Serializer::json_compatible()).map_err(|e| JsError::new(&e.to_string()))
}

#[wasm_bindgen]
pub struct WasmTable(Table);

#[wasm_bindgen]
impl WasmTable {
    /// A new game. `west`, `north` (your partner) and `east` are persona ids, or "any" for a
    /// random one; `hands` is the length of a short game (0 for a game to 500).
    #[wasm_bindgen(constructor)]
    pub fn new(seed: u32, west: &str, north: &str, east: &str, hands: u8) -> Result<WasmTable, JsError> {
        let players = ["human", west, north, east].map(String::from);
        Table::with_players(seed as u64, players, (hands > 0).then_some(hands)).map(WasmTable).map_err(|e| JsError::new(&e))
    }

    /// Names the players (a tournament's): West, North and East; South is "You".
    #[wasm_bindgen(js_name = setNames)]
    pub fn set_names(&mut self, west: &str, north: &str, east: &str) {
        self.0.set_names(["You", west, north, east].map(String::from));
    }

    /// Rebuilds a game from [`WasmTable::save`] output.
    pub fn load(json: &str) -> Result<WasmTable, JsError> {
        let save: SaveData = serde_json::from_str(json).map_err(|e| JsError::new(&e.to_string()))?;
        Table::restore(&save).map(WasmTable).map_err(|e| JsError::new(&e))
    }

    pub fn save(&self) -> String {
        serde_json::to_string(&self.0.save()).expect("save data serializes")
    }

    pub fn view(&self) -> Result<JsValue, JsError> {
        to_js(&self.0.view())
    }

    /// The human's action, by its `index` in `view().legal`. Returns the resulting events.
    pub fn act(&mut self, index: u32) -> Result<JsValue, JsError> {
        let events = self.0.act(index as usize).map_err(|e| JsError::new(&e))?;
        to_js(&events)
    }

    /// Your turn, played for you by a quick bot (finishing a hand whose outcome is clear).
    #[wasm_bindgen(js_name = autoAct)]
    pub fn auto_act(&mut self) -> Result<JsValue, JsError> {
        let events = self.0.auto_act().map_err(|e| JsError::new(&e))?;
        to_js(&events)
    }

    /// Lets the bot whose turn it is act; returns its events, or null when no bot is to act.
    #[wasm_bindgen(js_name = stepBot)]
    pub fn step_bot(&mut self) -> Result<JsValue, JsError> {
        match self.0.step_bot() {
            Some(events) => to_js(&events),
            None => Ok(JsValue::NULL),
        }
    }
}

#[derive(Serialize)]
struct PersonaView {
    id: &'static str,
    name: &'static str,
    blurb: &'static str,
}

/// A tournament table the person isn't at: a short game of `hands` hands between four personas.
#[wasm_bindgen(js_name = simulateGame)]
pub fn simulate_game(seed: u32, south: &str, west: &str, north: &str, east: &str, hands: u8) -> Result<JsValue, JsError> {
    let result = table::simulate(seed as u64, [south, west, north, east].map(String::from), hands).map_err(|e| JsError::new(&e))?;
    to_js(&result)
}

/// The AI players available to sit at the table.
#[wasm_bindgen]
pub fn personas() -> Result<JsValue, JsError> {
    let list: Vec<PersonaView> =
        fh_ai::PERSONAS.iter().map(|p| PersonaView { id: p.id, name: p.name, blurb: p.blurb }).collect();
    to_js(&list)
}

fn js_err(e: String) -> JsError {
    JsError::new(&e)
}

/// The coach's recommendation for South's current decision.
#[wasm_bindgen(js_name = coachHint)]
pub fn coach_hint(save: &str) -> Result<JsValue, JsError> {
    to_js(&coach::hint(save).map_err(js_err)?)
}

/// The coach's grade for action `number` of the save, or null if there was no real choice.
#[wasm_bindgen(js_name = coachGrade)]
pub fn coach_grade(save: &str, number: u32) -> Result<JsValue, JsError> {
    to_js(&coach::grade(save, number as usize).map_err(js_err)?)
}

/// The grade for the whole kitty discard starting at action `number` (all three cards together).
#[wasm_bindgen(js_name = coachGradeDiscards)]
pub fn coach_grade_discards(save: &str, number: u32) -> Result<JsValue, JsError> {
    to_js(&coach::grade_discards(save, number as usize).map_err(js_err)?)
}

/// Your decisions in a game (`{ number, discards }`), to grade them after it's over.
#[wasm_bindgen(js_name = coachDecisions)]
pub fn coach_decisions(save: &str) -> Result<JsValue, JsError> {
    to_js(&coach::decisions(save).map_err(js_err)?)
}

/// The double-dummy (all cards visible) trace of deal `number`, or null if it was passed out.
#[wasm_bindgen(js_name = coachTrace)]
pub fn coach_trace(save: &str, number: u32) -> Result<JsValue, JsError> {
    to_js(&coach::trace(save, number as usize).map_err(js_err)?)
}

/// Luck vs skill: deal `number` replayed with The Counter in your seat.
#[wasm_bindgen(js_name = coachExpert)]
pub fn coach_expert(save: &str, number: u32) -> Result<JsValue, JsError> {
    to_js(&coach::expert(save, number as usize).map_err(js_err)?)
}

/// A bidding drill: { view, save } at South's turn to call.
#[wasm_bindgen(js_name = drillBidding)]
pub fn drill_bidding(seed: u32) -> Result<JsValue, JsError> {
    to_js(&coach::drill_bidding(seed as u64).map_err(js_err)?)
}

/// A saved position (one of your past mistakes) as a puzzle: { view, save }.
#[wasm_bindgen(js_name = coachPosition)]
pub fn coach_position(save: &str) -> Result<JsValue, JsError> {
    to_js(&coach::position(save).map_err(js_err)?)
}

/// A card-play puzzle: { view, save } at a South decision with one clearly best play.
#[wasm_bindgen(js_name = drillPlay)]
pub fn drill_play(seed: u32) -> Result<JsValue, JsError> {
    to_js(&coach::drill_play(seed as u64).map_err(js_err)?)
}
