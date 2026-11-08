//! The engine worker's requests (`web/src/engine/protocol.ts`), answered natively: the host runs
//! these for a browser on a phone, whose own copy of the engine (WebAssembly in a worker) is far
//! slower than the computer hosting the game. Each connection gets its own [`Service`] (one game
//! and its coach requests, answered in order), the way each browser worker holds its own table.
//!
//! This is solo play: the browser already holds the game's save (every hand), so returning it is
//! no leak. A host's shared table (`fh-host`'s room) never sends saves.

use std::time::Instant;

use serde::Deserialize;
use serde_json::{json, Value};

use crate::coach;
use crate::table::{self, SaveData, Table};

/// One request, as the page sends it to its worker (the `id` is read separately).
#[derive(Deserialize)]
#[serde(tag = "op", rename_all = "camelCase")]
enum Request {
    Personas,
    Create { seed: f64, west: String, north: String, east: String, hands: u8, names: Option<[String; 3]> },
    Simulate { seed: f64, players: [String; 4], hands: u8 },
    Load { save: String },
    Act { index: usize },
    StepBot,
    AutoAct,
    Hint { save: String },
    Grade { save: String, number: usize },
    GradeDiscards { save: String, number: usize },
    Trace { save: String, number: usize },
    Expert { save: String, number: usize },
    DrillBidding { seed: f64 },
    DrillPlay { seed: f64 },
    Position { save: String },
}

/// A seed as the worker reads one (`seed >>> 0`): the low 32 bits.
fn seed(value: f64) -> u64 {
    (value as i64) as u32 as u64
}

fn to_value<T: serde::Serialize>(value: T) -> Result<Value, String> {
    serde_json::to_value(value).map_err(|e| e.to_string())
}

/// The game one connection is playing.
#[derive(Default)]
pub struct Service {
    table: Option<Table>,
}

impl Service {
    /// Answers one request (JSON text) with the worker's reply: `{ id, ok, result | error }`.
    pub fn handle_json(&mut self, text: &str) -> String {
        let started = Instant::now();
        let parsed: Result<Value, _> = serde_json::from_str(text);
        let id = parsed.as_ref().ok().and_then(|v| v.get("id")).cloned().unwrap_or(Value::Null);
        let outcome = parsed
            .map_err(|e| e.to_string())
            .and_then(|v| serde_json::from_value::<Request>(v).map_err(|e| e.to_string()))
            .and_then(|request| self.handle(request, started));
        let reply = match outcome {
            Ok(result) => json!({ "id": id, "ok": true, "result": result }),
            Err(error) => json!({ "id": id, "ok": false, "error": error }),
        };
        reply.to_string()
    }

    fn table(&mut self) -> Result<&mut Table, String> {
        self.table.as_mut().ok_or_else(|| "no game: create or load one first".to_string())
    }

    /// The table after a request: `{ view, save, events, ms }`, as the worker's `snapshot`.
    fn snapshot(&mut self, events: Value, started: Instant) -> Result<Value, String> {
        let table = self.table()?;
        let save = serde_json::to_string(&table.save()).map_err(|e| e.to_string())?;
        Ok(json!({
            "view": to_value(table.view())?,
            "save": save,
            "events": events,
            "ms": started.elapsed().as_secs_f64() * 1000.0,
        }))
    }

    fn handle(&mut self, request: Request, started: Instant) -> Result<Value, String> {
        match request {
            Request::Personas => {
                Ok(fh_ai::PERSONAS.iter().map(|p| json!({ "id": p.id, "name": p.name, "blurb": p.blurb })).collect())
            }
            Request::Create { seed: s, west, north, east, hands, names } => {
                let players = ["human".to_string(), west, north, east];
                let mut table = Table::with_players(seed(s), players, (hands > 0).then_some(hands))?;
                if let Some([w, n, e]) = names {
                    table.set_names(["You".to_string(), w, n, e]);
                }
                self.table = Some(table);
                self.snapshot(json!([]), started)
            }
            Request::Simulate { seed: s, players, hands } => to_value(table::simulate(seed(s), players, hands)?),
            Request::Load { save } => {
                let data: SaveData = serde_json::from_str(&save).map_err(|e| e.to_string())?;
                self.table = Some(Table::restore(&data)?);
                self.snapshot(json!([]), started)
            }
            Request::Act { index } => {
                let events = to_value(self.table()?.act(index)?)?;
                self.snapshot(events, started)
            }
            Request::AutoAct => {
                let events = to_value(self.table()?.auto_act()?)?;
                self.snapshot(events, started)
            }
            Request::StepBot => match self.table()?.step_bot() {
                Some(events) => {
                    let events = to_value(events)?;
                    self.snapshot(events, started)
                }
                None => Ok(Value::Null),
            },
            Request::Hint { save } => to_value(coach::hint(&save)?),
            Request::Grade { save, number } => to_value(coach::grade(&save, number)?),
            Request::GradeDiscards { save, number } => to_value(coach::grade_discards(&save, number)?),
            Request::Trace { save, number } => to_value(coach::trace(&save, number)?),
            Request::Expert { save, number } => to_value(coach::expert(&save, number)?),
            Request::DrillBidding { seed: s } => to_value(coach::drill_bidding(seed(s))?),
            Request::DrillPlay { seed: s } => to_value(coach::drill_play(seed(s))?),
            Request::Position { save } => to_value(coach::position(&save)?),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ask(service: &mut Service, request: Value) -> Value {
        serde_json::from_str(&service.handle_json(&request.to_string())).unwrap()
    }

    #[test]
    fn a_game_plays_through_the_service_as_through_the_worker() {
        let mut service = Service::default();
        let personas = ask(&mut service, json!({ "id": 1, "op": "personas" }));
        assert_eq!(personas["id"], 1);
        assert!(personas["result"].as_array().unwrap().iter().any(|p| p["id"] == "steady"));

        let created = ask(
            &mut service,
            json!({ "id": 2, "op": "create", "seed": 4242, "west": "steady", "north": "steady", "east": "steady", "hands": 4, "names": ["Ada", "Finn", "Tess"] }),
        );
        assert_eq!(created["ok"], true, "{created}");
        let snapshot = &created["result"];
        assert!(snapshot["save"].as_str().unwrap().contains("\"seed\":4242"));
        assert!(snapshot["view"]["legal"].is_array());
        assert_eq!(snapshot["events"], json!([]));
        assert!(snapshot["view"].to_string().contains("Finn"));

        // Bots act until it's South's turn, then South acts; the save restores the same table.
        let mut id = 3;
        let last_save = loop {
            assert!(id < 200, "South never got a turn");
            let step = ask(&mut service, json!({ "id": id, "op": "stepBot" }));
            id += 1;
            assert_eq!(step["ok"], true, "{step}");
            if step["result"].is_null() {
                let acted = ask(&mut service, json!({ "id": id, "op": "act", "index": 0 }));
                assert_eq!(acted["ok"], true, "{acted}");
                assert!(!acted["result"]["events"].as_array().unwrap().is_empty());
                break acted["result"]["save"].as_str().unwrap().to_string();
            }
        };

        // A fresh connection picks up from the save (what the page does after reconnecting).
        let mut other = Service::default();
        let loaded = ask(&mut other, json!({ "id": 1, "op": "load", "save": last_save }));
        assert_eq!(loaded["result"]["save"].as_str(), Some(last_save.as_str()));

        // The coach answers from the save alone.
        let hint = ask(&mut other, json!({ "id": 2, "op": "grade", "save": last_save, "number": 0 }));
        assert_eq!(hint["ok"], true, "{hint}");
    }

    #[test]
    fn bad_requests_are_answered_with_errors() {
        let mut service = Service::default();
        let reply = ask(&mut service, json!({ "id": 7, "op": "act", "index": 0 }));
        assert_eq!(reply["id"], 7);
        assert_eq!(reply["ok"], false);
        let reply = ask(&mut service, json!({ "id": 8, "op": "nonsense" }));
        assert_eq!(reply["ok"], false);
        let reply: Value = serde_json::from_str(&service.handle_json("not json")).unwrap();
        assert_eq!(reply["ok"], false);
    }

    #[test]
    fn seeds_are_read_as_the_worker_reads_them() {
        assert_eq!(seed(4242.0), 4242);
        assert_eq!(seed(4_294_967_295.0), 4_294_967_295);
        assert_eq!(seed(4_294_967_296.0 + 5.0), 5);
    }
}
