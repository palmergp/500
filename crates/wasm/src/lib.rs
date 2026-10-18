//! wasm-bindgen bindings for the web app. All game logic lives in [`table`] and [`view`].

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
    /// A new game. `west`, `north` (your partner) and `east` are persona ids.
    #[wasm_bindgen(constructor)]
    pub fn new(seed: u32, west: &str, north: &str, east: &str) -> Result<WasmTable, JsError> {
        Table::new(seed as u64, west, north, east).map(WasmTable).map_err(|e| JsError::new(&e))
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

/// The AI players available to sit at the table.
#[wasm_bindgen]
pub fn personas() -> Result<JsValue, JsError> {
    let list: Vec<PersonaView> =
        fh_ai::PERSONAS.iter().map(|p| PersonaView { id: p.id, name: p.name, blurb: p.blurb }).collect();
    to_js(&list)
}
