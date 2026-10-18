//! The room: one table, its four seats and everyone connected. It runs on its own thread and owns
//! the game; each connection talks to it over a channel. The room is the only place the whole deal
//! exists: each person is sent the table as their seat sees it, and the coach answers each person
//! from their own seat's knowledge.
//!
//! Seats always hold a bot, and a person may sit in front of it. Before a game anyone connected can
//! sit, stand and pick the bots; once it starts, a person who leaves (or drops off the wifi and is
//! replaced) hands their seat to its bot, and someone new can take over a bot or an empty person's
//! seat. A dropped connection keeps its seat: the game waits on that seat until they reconnect.

use std::collections::BTreeMap;
use std::hash::{BuildHasher, Hasher};
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc::{Receiver, RecvTimeoutError, Sender};
use std::sync::Arc;
use std::time::{Duration, Instant};

use fh_core::{Event, Phase, Seat};
use fh_wasm::coach;
use fh_wasm::table::{SaveData, Table};
use fh_wasm::view::{event_view, EventView, Lens, View};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tokio::sync::mpsc::UnboundedSender;

/// A connection is dropped after this long without hearing from it (clients ping every 10 s).
const SILENCE: Duration = Duration::from_secs(75);

/// Pauses before a bot acts, as in the single-player app (milliseconds, at normal pace).
const DELAY_BIDDING: u64 = 700;
const DELAY_DISCARDING: u64 = 300;
const DELAY_PLAYING: u64 = 650;
/// Extra pause after a trick, while everyone sees who won it.
const TRICK_HOLD: u64 = 1300;
/// Pause before the next deal, once everyone has seen the result.
const NEXT_DEAL: u64 = 600;

pub enum Command {
    Connect { conn: u64, tx: UnboundedSender<String> },
    Message { conn: u64, text: String },
    Disconnect { conn: u64 },
}

/// How connections reach the room.
#[derive(Clone)]
pub struct RoomHandle {
    tx: Sender<Command>,
    next: Arc<AtomicU64>,
}

impl RoomHandle {
    /// Registers a connection; the room sends it messages through `tx`. Returns its id.
    pub fn connect(&self, tx: UnboundedSender<String>) -> u64 {
        let conn = self.next.fetch_add(1, Ordering::Relaxed);
        let _ = self.tx.send(Command::Connect { conn, tx });
        conn
    }

    pub fn message(&self, conn: u64, text: String) {
        let _ = self.tx.send(Command::Message { conn, text });
    }

    pub fn disconnect(&self, conn: u64) {
        let _ = self.tx.send(Command::Disconnect { conn });
    }
}

/// Starts the room on its own thread. `state` is where the table is kept between runs of the host.
pub fn spawn(state: Option<PathBuf>) -> (RoomHandle, Option<String>) {
    let (tx, rx) = std::sync::mpsc::channel();
    let (loaded_tx, loaded_rx) = std::sync::mpsc::channel();
    // The bots aren't `Send`, so the room is built on the thread that owns it.
    let thread = move || {
        let mut room = Room::new(state);
        let _ = loaded_tx.send(room.load());
        room.run(rx);
    };
    std::thread::Builder::new().name("room".into()).spawn(thread).expect("room thread starts");
    let resumed = loaded_rx.recv().unwrap_or(None);
    (RoomHandle { tx, next: Arc::new(AtomicU64::new(1)) }, resumed)
}

#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "camelCase", rename_all_fields = "camelCase")]
enum ClientMessage {
    Hello { client_id: String, name: String },
    Ping,
    Rename { name: String },
    /// Seats are absolute here: 0 South, 1 West, 2 North, 3 East.
    Sit { seat: usize },
    Stand,
    SetBot { seat: usize, persona: String },
    SetPace { pace: String },
    Start,
    /// `expect` is the action count the client saw, so a stale click can't land on a later turn.
    Act { id: u64, index: usize, expect: usize },
    /// Seen the hand's result; ready for the next deal.
    Continue,
    EndGame,
    Hint { id: u64 },
    Grade { id: u64, number: usize },
    Trace { id: u64, number: usize },
    Expert { id: u64, number: usize },
}

impl ClientMessage {
    /// The request id, for messages that expect a reply.
    fn id(&self) -> Option<u64> {
        match self {
            ClientMessage::Act { id, .. }
            | ClientMessage::Hint { id }
            | ClientMessage::Grade { id, .. }
            | ClientMessage::Trace { id, .. }
            | ClientMessage::Expert { id, .. } => Some(*id),
            _ => None,
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
struct Person {
    client: String,
    name: String,
}

#[derive(Clone, Serialize, Deserialize)]
struct SeatState {
    person: Option<Person>,
    bot: String,
}

struct Conn {
    tx: UnboundedSender<String>,
    client: Option<String>,
    name: String,
    last_seen: Instant,
    /// The last lobby message sent, so unchanged lobbies aren't resent.
    last_lobby: String,
}

/// What the room keeps on disk.
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Saved {
    seats: Vec<SeatState>,
    save: Option<SaveData>,
    game_id: String,
    pace: String,
    waiting: Vec<usize>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct LobbyView<'a> {
    /// Your seat (absolute), if you have one.
    you: Option<usize>,
    seats: Vec<SeatView>,
    /// A game is on (it may be over, with everyone still looking at the result).
    playing: bool,
    over: bool,
    pace: &'a str,
    /// People the table waits on to continue after a hand.
    waiting: Vec<String>,
    personas: Vec<PersonaView>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SeatView {
    person: Option<PersonView>,
    bot: String,
    bot_name: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct PersonView {
    name: String,
    connected: bool,
}

#[derive(Serialize)]
struct PersonaView {
    id: &'static str,
    name: &'static str,
    blurb: &'static str,
}

/// The table for one person after an action: the same shape the browser engine produces, minus
/// the save (which would reveal every hand).
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Snapshot {
    view: View,
    events: Vec<EventView>,
    save: &'static str,
    ms: u32,
    game: String,
}

pub struct Room {
    conns: BTreeMap<u64, Conn>,
    seats: [SeatState; 4],
    table: Option<Table>,
    game_id: String,
    pace: String,
    /// When the bot to act next should move.
    next_bot: Option<Instant>,
    /// Seats the table waits on to continue after a hand.
    waiting: Vec<Seat>,
    state: Option<PathBuf>,
}

impl Room {
    pub fn new(state: Option<PathBuf>) -> Room {
        let bots = ["oracle", "rock", "oracle", "gambler"];
        Room {
            conns: BTreeMap::new(),
            seats: bots.map(|b| SeatState { person: None, bot: b.to_string() }),
            table: None,
            game_id: String::new(),
            pace: "normal".into(),
            next_bot: None,
            waiting: Vec::new(),
            state,
        }
    }

    fn run(mut self, rx: Receiver<Command>) {
        loop {
            let now = Instant::now();
            let wake = self.next_bot.map_or(Duration::from_secs(1), |at| at.saturating_duration_since(now));
            match rx.recv_timeout(wake.min(Duration::from_secs(1))) {
                Ok(command) => self.command(command),
                Err(RecvTimeoutError::Timeout) => {}
                Err(RecvTimeoutError::Disconnected) => return,
            }
            self.drop_silent();
            if self.next_bot.is_some_and(|at| Instant::now() >= at) {
                self.next_bot = None;
                self.step_bot();
            }
            self.sync_lobby();
        }
    }

    pub fn command(&mut self, command: Command) {
        match command {
            Command::Connect { conn, tx } => {
                let c = Conn { tx, client: None, name: String::new(), last_seen: Instant::now(), last_lobby: String::new() };
                self.conns.insert(conn, c);
            }
            Command::Disconnect { conn } => {
                self.conns.remove(&conn);
                self.forget_waiting_on_absent();
            }
            Command::Message { conn, text } => {
                let Some(c) = self.conns.get_mut(&conn) else { return };
                c.last_seen = Instant::now();
                match serde_json::from_str::<ClientMessage>(&text) {
                    Ok(message) => {
                        let id = message.id();
                        if let Err(e) = self.message(conn, message) {
                            match id {
                                Some(id) => self.send(conn, json!({ "type": "reply", "id": id, "ok": false, "error": e })),
                                None => self.send(conn, json!({ "type": "notice", "text": e })),
                            }
                        }
                    }
                    Err(e) => self.send(conn, json!({ "type": "notice", "text": format!("bad message: {e}") })),
                }
            }
        }
    }

    fn message(&mut self, conn: u64, message: ClientMessage) -> Result<(), String> {
        match message {
            ClientMessage::Ping => {}
            ClientMessage::Hello { client_id, name } => {
                if client_id.is_empty() || client_id.len() > 64 {
                    return Err("bad client id".into());
                }
                let name = clean_name(&name);
                let c = self.conns.get_mut(&conn).expect("connected");
                c.client = Some(client_id.clone());
                c.name = name.clone();
                if let Some(seat) = self.seat_of(&client_id) {
                    self.seats[seat.index()].person.as_mut().expect("seated").name = name;
                    self.send_snapshot(conn, seat, &[], None);
                }
            }
            ClientMessage::Rename { name } => {
                let name = clean_name(&name);
                let client = self.client(conn)?;
                self.conns.get_mut(&conn).expect("connected").name = name.clone();
                if let Some(seat) = self.seat_of(&client) {
                    self.seats[seat.index()].person.as_mut().expect("seated").name = name;
                    self.resend_views();
                }
            }
            ClientMessage::Sit { seat } => self.sit(conn, seat_at(seat)?)?,
            ClientMessage::Stand => {
                let client = self.client(conn)?;
                let seat = self.seat_of(&client).ok_or("you aren't sitting anywhere")?;
                self.seats[seat.index()].person = None;
                self.waiting.retain(|&s| s != seat);
                if let Some(table) = &mut self.table {
                    table.set_player(seat, &self.seats[seat.index()].bot)?;
                    self.schedule(&[]);
                    self.resend_views();
                }
            }
            ClientMessage::SetBot { seat, persona } => {
                self.seated(conn)?;
                let seat = seat_at(seat)?;
                if fh_ai::persona(&persona).is_none() {
                    return Err(format!("unknown player type {persona:?}"));
                }
                let state = &mut self.seats[seat.index()];
                if let Some(person) = &state.person {
                    if self.conns.values().any(|c| c.client.as_deref() == Some(&person.client)) {
                        return Err(format!("{} is sitting there", person.name));
                    }
                    // Replacing someone who dropped off.
                    state.person = None;
                    self.waiting.retain(|&s| s != seat);
                }
                state.bot = persona.clone();
                if let Some(table) = &mut self.table {
                    table.set_player(seat, &persona)?;
                    self.schedule(&[]);
                    self.resend_views();
                }
            }
            ClientMessage::SetPace { pace } => {
                self.seated(conn)?;
                if !["slow", "normal", "fast"].contains(&pace.as_str()) {
                    return Err("unknown pace".into());
                }
                self.pace = pace;
            }
            ClientMessage::Start => {
                self.seated(conn)?;
                if self.table.is_some() {
                    return Err("a game is already on".into());
                }
                let players = self.seats.clone().map(|s| if s.person.is_some() { "human".to_string() } else { s.bot });
                self.table = Some(Table::with_players(random_u64(), players)?);
                self.game_id = format!("lan-{:016x}", random_u64());
                self.waiting.clear();
                self.resend_views();
                self.schedule(&[]);
            }
            ClientMessage::Act { id, index, expect } => {
                let seat = self.seated(conn)?;
                let table = self.table.as_mut().ok_or("no game is on")?;
                if !self.waiting.is_empty() {
                    return Err("waiting for everyone to see the result".into());
                }
                if table.action_count() != expect {
                    return Err("the table has moved on".into());
                }
                let events = table.act_as(seat, index)?;
                self.after_action(&events, Some((conn, id)));
            }
            ClientMessage::Continue => {
                let seat = self.seated(conn)?;
                let before = self.waiting.len();
                self.waiting.retain(|&s| s != seat);
                if before > 0 && self.waiting.is_empty() {
                    self.schedule(&[]);
                }
            }
            ClientMessage::EndGame => {
                self.seated(conn)?;
                self.table = None;
                self.waiting.clear();
                self.next_bot = None;
                // Seats of people who left are freed for the next game.
                let present: Vec<String> = self.conns.values().filter_map(|c| c.client.clone()).collect();
                for state in &mut self.seats {
                    if state.person.as_ref().is_some_and(|p| !present.contains(&p.client)) {
                        state.person = None;
                    }
                }
                for conn in self.conns.values_mut() {
                    let _ = conn.tx.send(json!({ "type": "ended" }).to_string());
                }
            }
            ClientMessage::Hint { id } => self.coach(conn, id, |save, seat| coach::hint_for(save, seat).map(to_value))?,
            ClientMessage::Grade { id, number } => {
                self.coach(conn, id, move |save, seat| coach::grade_for(save, number, seat).map(to_value))?
            }
            ClientMessage::Trace { id, number } => {
                self.coach(conn, id, move |save, seat| coach::trace_for(save, number, seat).map(to_value))?
            }
            ClientMessage::Expert { id, number } => {
                self.coach(conn, id, move |save, seat| coach::expert_for(save, number, seat).map(to_value))?
            }
        }
        self.store();
        Ok(())
    }

    fn sit(&mut self, conn: u64, seat: Seat) -> Result<(), String> {
        let client = self.client(conn)?;
        let current = self.seat_of(&client);
        if current == Some(seat) {
            return Ok(());
        }
        if let Some(person) = &self.seats[seat.index()].person {
            if self.conns.values().any(|c| c.client.as_deref() == Some(&person.client)) {
                return Err(format!("{} is sitting there", person.name));
            }
        }
        if let Some(old) = current {
            if self.table.is_some() {
                // Two seats would mean seeing two hands.
                return Err("you're already playing in another seat".into());
            }
            self.seats[old.index()].person = None;
        }
        let name = self.conns[&conn].name.clone();
        self.seats[seat.index()].person = Some(Person { client, name });
        if let Some(table) = &mut self.table {
            table.set_player(seat, "human")?;
            self.waiting.retain(|&s| s != seat);
            self.schedule(&[]);
            self.resend_views();
        }
        Ok(())
    }

    /// Runs a coach request for the asker's seat on its own thread, so play never waits on it.
    fn coach(
        &mut self,
        conn: u64,
        id: u64,
        run: impl FnOnce(&SaveData, Seat) -> Result<Value, String> + Send + 'static,
    ) -> Result<(), String> {
        let seat = self.seated(conn)?;
        let save = self.table.as_ref().ok_or("no game is on")?.save();
        let tx = self.conns[&conn].tx.clone();
        std::thread::spawn(move || {
            let reply = match run(&save, seat) {
                Ok(result) => json!({ "type": "reply", "id": id, "ok": true, "result": result }),
                Err(error) => json!({ "type": "reply", "id": id, "ok": false, "error": error }),
            };
            let _ = tx.send(reply.to_string());
        });
        Ok(())
    }

    /// Lets the bot to act move, if nobody is being waited on.
    pub fn step_bot(&mut self) -> bool {
        if !self.waiting.is_empty() {
            return false;
        }
        let Some(events) = self.table.as_mut().and_then(|t| t.step_bot_events()) else { return false };
        self.after_action(&events, None);
        self.store();
        true
    }

    /// Sends everyone the new table, answering `reply` (the connection and request id that acted).
    fn after_action(&mut self, events: &[Event], reply: Option<(u64, u64)>) {
        let over = self.table.as_ref().is_some_and(|t| t.game().phase() == Phase::GameOver);
        if !over && events.iter().any(|e| matches!(e, Event::HandScored { .. })) {
            self.waiting = self.present_seats();
        }
        for (&conn, c) in &self.conns {
            let Some(seat) = c.client.as_deref().and_then(|client| self.seat_of(client)) else { continue };
            let id = reply.filter(|&(r, _)| r == conn).map(|(_, id)| id);
            self.send_snapshot(conn, seat, events, id);
        }
        self.schedule(events);
    }

    /// Times the next bot move after `events`.
    fn schedule(&mut self, events: &[Event]) {
        self.next_bot = None;
        let Some(table) = &self.table else { return };
        if !self.waiting.is_empty() || !table.bot_to_act() {
            return;
        }
        let mut ms = match table.game().phase() {
            Phase::Bidding => DELAY_BIDDING,
            Phase::Discarding => DELAY_DISCARDING,
            Phase::Playing => DELAY_PLAYING,
            Phase::GameOver => return,
        };
        if events.iter().any(|e| matches!(e, Event::TrickWon { .. })) {
            ms += TRICK_HOLD;
        }
        if events.is_empty() {
            ms = ms.max(NEXT_DEAL);
        }
        let factor = match self.pace.as_str() {
            "slow" => 1.6,
            "fast" => 0.45,
            _ => 1.0,
        };
        self.next_bot = Some(Instant::now() + Duration::from_millis((ms as f64 * factor) as u64));
    }

    fn send_snapshot(&self, conn: u64, seat: Seat, events: &[Event], reply: Option<u64>) {
        let Some(table) = &self.table else { return };
        let lens = self.lens(seat);
        let snapshot = Snapshot {
            view: View::build_for(table.game(), &lens, table.players(), table.action_count()),
            events: events.iter().map(|e| event_view(e, &lens)).collect(),
            save: "",
            ms: 0,
            game: self.game_id.clone(),
        };
        let message = match reply {
            Some(id) => json!({ "type": "reply", "id": id, "ok": true, "result": snapshot }),
            None => json!({ "type": "snapshot", "snapshot": snapshot }),
        };
        self.send(conn, message);
    }

    /// Everyone's view again, with no events (after a seat changes hands or a name changes).
    fn resend_views(&self) {
        for (&conn, c) in &self.conns {
            if let Some(seat) = c.client.as_deref().and_then(|client| self.seat_of(client)) {
                self.send_snapshot(conn, seat, &[], None);
            }
        }
    }

    /// Names as `viewer` sees them: "You", people's names and the bots' persona names.
    fn lens(&self, viewer: Seat) -> Lens {
        Lens::new(
            viewer,
            std::array::from_fn(|i| {
                let state = &self.seats[i];
                match &state.person {
                    _ if i == viewer.index() => "You".to_string(),
                    Some(person) => person.name.clone(),
                    None => fh_ai::persona(&state.bot).map_or(state.bot.clone(), |p| p.name.to_string()),
                }
            }),
        )
    }

    /// Sends each connection the lobby, if it changed since the last one it got.
    pub fn sync_lobby(&mut self) {
        let views: Vec<(u64, String)> = self.conns.keys().map(|&conn| (conn, self.lobby_for(conn))).collect();
        for (conn, text) in views {
            let c = self.conns.get_mut(&conn).expect("connected");
            if c.last_lobby != text {
                let _ = c.tx.send(text.clone());
                c.last_lobby = text;
            }
        }
    }

    fn lobby_for(&self, conn: u64) -> String {
        let you = self.conns[&conn].client.as_deref().and_then(|client| self.seat_of(client)).map(|s| s.index());
        let lobby = LobbyView {
            you,
            seats: self
                .seats
                .iter()
                .map(|s| SeatView {
                    person: s.person.as_ref().map(|p| PersonView { name: p.name.clone(), connected: self.present(&p.client) }),
                    bot: s.bot.clone(),
                    bot_name: fh_ai::persona(&s.bot).map_or(s.bot.clone(), |p| p.name.to_string()),
                })
                .collect(),
            playing: self.table.is_some(),
            over: self.table.as_ref().is_some_and(|t| t.game().phase() == Phase::GameOver),
            pace: &self.pace,
            waiting: self.waiting.iter().filter_map(|s| self.seats[s.index()].person.as_ref().map(|p| p.name.clone())).collect(),
            personas: fh_ai::PERSONAS.iter().map(|p| PersonaView { id: p.id, name: p.name, blurb: p.blurb }).collect(),
        };
        let mut value = serde_json::to_value(lobby).expect("lobby serializes");
        value["type"] = "lobby".into();
        value.to_string()
    }

    fn send(&self, conn: u64, message: Value) {
        if let Some(c) = self.conns.get(&conn) {
            let _ = c.tx.send(message.to_string());
        }
    }

    fn client(&self, conn: u64) -> Result<String, String> {
        self.conns.get(&conn).and_then(|c| c.client.clone()).ok_or_else(|| "say hello first".into())
    }

    fn seated(&self, conn: u64) -> Result<Seat, String> {
        let client = self.client(conn)?;
        self.seat_of(&client).ok_or_else(|| "take a seat first".into())
    }

    fn seat_of(&self, client: &str) -> Option<Seat> {
        Seat::ALL.into_iter().find(|s| self.seats[s.index()].person.as_ref().is_some_and(|p| p.client == client))
    }

    fn present(&self, client: &str) -> bool {
        self.conns.values().any(|c| c.client.as_deref() == Some(client))
    }

    fn present_seats(&self) -> Vec<Seat> {
        Seat::ALL.into_iter().filter(|s| self.seats[s.index()].person.as_ref().is_some_and(|p| self.present(&p.client))).collect()
    }

    /// Nobody waits on a person who isn't here to press "Next hand".
    fn forget_waiting_on_absent(&mut self) {
        let before = self.waiting.len();
        let present = self.present_seats();
        self.waiting.retain(|s| present.contains(s));
        if before > 0 && self.waiting.is_empty() {
            self.schedule(&[]);
        }
    }

    /// Drops connections that have gone quiet (a phone that slept without closing its socket), so
    /// their seat shows as away and can be handed to a bot.
    fn drop_silent(&mut self) {
        let now = Instant::now();
        let before = self.conns.len();
        self.conns.retain(|_, c| now.duration_since(c.last_seen) < SILENCE);
        if self.conns.len() != before {
            self.forget_waiting_on_absent();
        }
    }

    fn store(&self) {
        let Some(path) = &self.state else { return };
        let saved = Saved {
            seats: self.seats.to_vec(),
            save: self.table.as_ref().map(|t| t.save()),
            game_id: self.game_id.clone(),
            pace: self.pace.clone(),
            waiting: self.waiting.iter().map(|s| s.index()).collect(),
        };
        let json = serde_json::to_string_pretty(&saved).expect("state serializes");
        let temp = path.with_extension("tmp");
        if std::fs::write(&temp, json).and_then(|_| std::fs::rename(&temp, path)).is_err() {
            eprintln!("could not save the table to {}", path.display());
        }
    }

    /// Picks up the table from the last run, if there was one. Returns a description of it.
    fn load(&mut self) -> Option<String> {
        let text = std::fs::read_to_string(self.state.as_ref()?).ok()?;
        let saved: Saved = serde_json::from_str(&text).ok()?;
        let seats: [SeatState; 4] = saved.seats.try_into().ok()?;
        if !seats.iter().all(|s| fh_ai::persona(&s.bot).is_some()) {
            return None;
        }
        let table = match &saved.save {
            Some(save) => Some(Table::restore(save).ok()?),
            None => None,
        };
        self.seats = seats;
        self.pace = saved.pace;
        self.game_id = saved.game_id;
        self.waiting = saved.waiting.into_iter().filter_map(|i| seat_at(i).ok()).collect();
        let description = table.as_ref().map(|t| {
            let scores = t.game().scores();
            format!("a game in progress (deal {}, {}–{})", t.game().history().len() + 1, scores[0], scores[1])
        });
        self.table = table;
        self.schedule(&[]);
        description
    }
}

fn to_value<T: Serialize>(value: T) -> Value {
    serde_json::to_value(value).expect("coach views serialize")
}

fn seat_at(index: usize) -> Result<Seat, String> {
    Seat::ALL.get(index).copied().ok_or_else(|| format!("no seat {index}"))
}

fn clean_name(name: &str) -> String {
    let name: String = name.trim().chars().filter(|c| !c.is_control()).take(16).collect();
    if name.is_empty() {
        "Player".into()
    } else {
        name
    }
}

fn random_u64() -> u64 {
    let mut hasher = std::collections::hash_map::RandomState::new().build_hasher();
    hasher.write_u128(std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0, |d| d.as_nanos()));
    hasher.finish()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver};

    struct Client {
        conn: u64,
        rx: UnboundedReceiver<String>,
    }

    impl Client {
        fn join(room: &mut Room, conn: u64, id: &str, name: &str) -> Client {
            let (tx, rx) = unbounded_channel();
            room.command(Command::Connect { conn, tx });
            let mut client = Client { conn, rx };
            client.say(room, json!({ "type": "hello", "clientId": id, "name": name }));
            client
        }

        fn say(&mut self, room: &mut Room, message: Value) {
            room.command(Command::Message { conn: self.conn, text: message.to_string() });
            room.sync_lobby();
        }

        /// Everything received since the last call.
        fn inbox(&mut self) -> Vec<Value> {
            std::iter::from_fn(|| self.rx.try_recv().ok()).map(|t| serde_json::from_str(&t).unwrap()).collect()
        }

        /// The latest view among `messages`.
        fn latest_view(messages: &[Value]) -> Option<Value> {
            messages
                .iter()
                .rev()
                .find_map(|m| match m["type"].as_str() {
                    Some("snapshot") => Some(m["snapshot"]["view"].clone()),
                    Some("reply") if m["result"]["view"].is_object() => Some(m["result"]["view"].clone()),
                    _ => None,
                })
        }
    }

    fn lobby(messages: &[Value]) -> Value {
        messages.iter().rev().find(|m| m["type"] == "lobby").cloned().expect("a lobby message")
    }

    #[test]
    fn two_people_play_a_whole_game_and_see_only_their_own_cards() {
        let mut room = Room::new(None);
        let mut ann = Client::join(&mut room, 1, "ann", "Ann");
        let mut bob = Client::join(&mut room, 2, "bob", "Bob");
        ann.say(&mut room, json!({ "type": "sit", "seat": 0 }));
        bob.say(&mut room, json!({ "type": "sit", "seat": 1 }));
        bob.say(&mut room, json!({ "type": "setBot", "seat": 2, "persona": "steady" }));
        bob.say(&mut room, json!({ "type": "setBot", "seat": 3, "persona": "steady" }));
        let l = lobby(&ann.inbox());
        assert_eq!(l["you"], 0);
        assert_eq!(l["seats"][1]["person"]["name"], "Bob");
        assert_eq!(lobby(&bob.inbox())["you"], 1);

        ann.say(&mut room, json!({ "type": "start" }));
        let mut views = [None, None];
        let mut request = 100;
        for _ in 0..20_000 {
            for (i, client) in [&mut ann, &mut bob].into_iter().enumerate() {
                let inbox = client.inbox();
                assert!(inbox.iter().all(|m| m["type"] != "notice" && m["ok"] != false), "{inbox:?}");
                if let Some(v) = Client::latest_view(&inbox) {
                    views[i] = Some(v);
                }
            }
            let table = room.table.as_ref().unwrap();
            let game = table.game();
            // Each person sees exactly their own hand, at the bottom.
            for (i, seat) in [Seat::SOUTH, Seat::WEST].into_iter().enumerate() {
                let view = views[i].as_ref().expect("a view");
                if view["actionCount"] == table.action_count() {
                    let ids: Vec<u64> = view["hand"].as_array().unwrap().iter().map(|c| c["id"].as_u64().unwrap()).collect();
                    let mine: Vec<u64> = game.hand(seat).iter().map(|c| c.id() as u64).collect();
                    assert_eq!(ids.len(), mine.len());
                    assert!(ids.iter().all(|id| mine.contains(id)));
                }
            }
            if game.phase() == Phase::GameOver {
                let view = views[0].as_ref().unwrap();
                assert_eq!(view["phase"], "gameOver");
                return;
            }
            if !room.waiting.is_empty() {
                ann.say(&mut room, json!({ "type": "continue" }));
                bob.say(&mut room, json!({ "type": "continue" }));
                assert!(room.waiting.is_empty());
                continue;
            }
            match game.to_act() {
                Some(seat) if seat == Seat::SOUTH || seat == Seat::WEST => {
                    let (client, view) = if seat == Seat::SOUTH { (&mut ann, &views[0]) } else { (&mut bob, &views[1]) };
                    let view = view.as_ref().unwrap();
                    assert_eq!(view["humanToAct"], true);
                    let legal = view["legal"].as_array().unwrap();
                    let pick = legal.iter().find(|a| a["kind"] == "pass").unwrap_or(&legal[0]);
                    request += 1;
                    let expect = view["actionCount"].clone();
                    client.say(&mut room, json!({ "type": "act", "id": request, "index": pick["index"], "expect": expect }));
                }
                _ => assert!(room.step_bot()),
            }
        }
        panic!("the game did not finish");
    }

    #[test]
    fn seats_survive_a_dropped_connection_and_coaching_is_private() {
        let mut room = Room::new(None);
        let mut ann = Client::join(&mut room, 1, "ann", "Ann");
        ann.say(&mut room, json!({ "type": "sit", "seat": 2 }));
        ann.say(&mut room, json!({ "type": "start" }));
        // North sees themselves at the bottom, named "You".
        let view = Client::latest_view(&ann.inbox()).unwrap();
        assert_eq!(view["players"][0]["name"], "You");
        assert_eq!(view["players"][2]["name"], "The Oracle", "South's bot is North's partner");

        // Ann's phone drops; someone else can't take her seat while she's connected, but can after.
        let mut cat = Client::join(&mut room, 3, "cat", "Cat");
        cat.say(&mut room, json!({ "type": "sit", "seat": 2 }));
        assert!(cat.inbox().iter().any(|m| m["type"] == "notice"));
        room.command(Command::Disconnect { conn: 1 });
        room.sync_lobby();
        let l = lobby(&cat.inbox());
        assert_eq!(l["seats"][2]["person"]["connected"], false);

        // Ann comes back on a new connection and gets her table back.
        let mut ann = Client::join(&mut room, 4, "ann", "Ann");
        let inbox = ann.inbox();
        assert_eq!(lobby(&inbox)["you"], 2);
        assert!(Client::latest_view(&inbox).is_some());

        // Bots play until it's Ann's turn, then she asks for a hint: only she gets the answer.
        while room.step_bot() {}
        assert_eq!(room.table.as_ref().unwrap().game().to_act(), Some(Seat::NORTH));
        ann.say(&mut room, json!({ "type": "hint", "id": 7 }));
        let reply = (0..600)
            .find_map(|_| {
                std::thread::sleep(Duration::from_millis(50));
                ann.inbox().into_iter().find(|m| m["type"] == "reply")
            })
            .expect("a hint");
        assert_eq!(reply["id"], 7);
        assert_eq!(reply["ok"], true, "{reply}");
        assert!(cat.inbox().iter().all(|m| m["type"] != "reply"));
        // Cat, not seated, can't ask the coach about anyone's hand.
        cat.say(&mut room, json!({ "type": "hint", "id": 8 }));
        assert!(cat.inbox().iter().any(|m| m["type"] == "reply" && m["ok"] == false));
    }

    #[test]
    fn a_stale_click_is_refused() {
        let mut room = Room::new(None);
        let mut ann = Client::join(&mut room, 1, "ann", "Ann");
        ann.say(&mut room, json!({ "type": "sit", "seat": 0 }));
        ann.say(&mut room, json!({ "type": "start" }));
        while room.step_bot() {}
        let count = room.table.as_ref().unwrap().action_count();
        ann.inbox();
        ann.say(&mut room, json!({ "type": "act", "id": 1, "index": 0, "expect": count + 1 }));
        assert!(ann.inbox().iter().any(|m| m["type"] == "reply" && m["ok"] == false && m["id"] == 1));
        assert_eq!(room.table.as_ref().unwrap().action_count(), count);
    }

    #[test]
    fn the_table_is_kept_between_runs() {
        let path = std::env::temp_dir().join(format!("fh-host-test-{}.json", random_u64()));
        let mut room = Room::new(Some(path.clone()));
        let mut ann = Client::join(&mut room, 1, "ann", "Ann");
        ann.say(&mut room, json!({ "type": "sit", "seat": 0 }));
        ann.say(&mut room, json!({ "type": "start" }));
        while room.step_bot() {}
        let count = room.table.as_ref().unwrap().action_count();
        let mut again = Room::new(Some(path.clone()));
        assert!(again.load().is_some());
        assert_eq!(again.table.as_ref().unwrap().action_count(), count);
        assert_eq!(again.seats[0].person.as_ref().unwrap().name, "Ann");
        let _ = std::fs::remove_file(path);
    }
}
