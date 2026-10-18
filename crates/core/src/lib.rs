//! Rules engine for the card game 500 (Five Hundred), house rules as specified in
//! `docs/rules/house-rules.md`. Rule IDs (H1, H2, …) in comments refer to that document.
//!
//! The engine is dependency-free and deterministic: a game is fully described by its rules,
//! its seed and the list of actions applied to it.

pub mod bidding;
pub mod cards;
pub mod game;
pub mod rng;
pub mod rules;
pub mod seat;
pub mod trick;

pub use bidding::{Auction, Call};
pub use cards::{Card, CardSet, Suit};
pub use game::{Action, Event, Game, HandRecord, Observation, Phase, RuleError};
pub use rng::Rng;
pub use rules::{effective_suit, Contract, Rules, Strain};
pub use seat::{Seat, Team};
pub use trick::{legal_plays, Play, Trick};
