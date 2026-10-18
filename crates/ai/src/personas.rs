//! The cast of AI players (PLAN.md §5). Each persona is a bot engine plus a style.

use crate::{Bot, RandomBot, RuleBot, Style};

pub struct Persona {
    pub id: &'static str,
    pub name: &'static str,
    pub blurb: &'static str,
}

pub const PERSONAS: [Persona; 4] = [
    Persona {
        id: "steady",
        name: "Steady",
        blurb: "Sound, honest bidding and sensible play. A dependable partner.",
    },
    Persona {
        id: "rookie",
        name: "Rookie",
        blurb: "Bids by gut, forgets which cards are gone and makes the odd blunder.",
    },
    Persona {
        id: "rock",
        name: "The Rock",
        blurb: "Bids only sure things and plays it safe. Make them pay for their caution.",
    },
    Persona {
        id: "gambler",
        name: "The Gambler",
        blurb: "Bids a trick too high and loves a swing. Let them overreach, then set them.",
    },
];

pub fn persona(id: &str) -> Option<&'static Persona> {
    PERSONAS.iter().find(|p| p.id == id)
}

/// A bot for persona `id` (or "random"); `seed` drives any randomness in its play.
pub fn make_bot(id: &str, seed: u64) -> Option<Box<dyn Bot>> {
    let style = match id {
        "steady" => Style::STEADY,
        "rookie" => Style::ROOKIE,
        "rock" => Style::CAUTIOUS,
        "gambler" => Style::BOLD,
        "random" => return Some(Box::new(RandomBot::new(seed))),
        _ => return None,
    };
    Some(Box::new(RuleBot::new(persona(id)?.name, style, seed)))
}
