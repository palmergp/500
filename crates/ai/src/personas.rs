//! The cast of AI players (PLAN.md §5). Each persona is a bot engine plus a style.

use crate::{Bot, RandomBot, RuleBot, SearchBot, SearchConfig, Style};

pub struct Persona {
    pub id: &'static str,
    pub name: &'static str,
    pub blurb: &'static str,
}

pub const PERSONAS: [Persona; 7] = [
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
    Persona {
        id: "counter",
        name: "The Counter",
        blurb: "Remembers every card, reads the auction and plays strictly by the odds.",
    },
    Persona {
        id: "shark",
        name: "The Shark",
        blurb: "Expert play with one eye on the scoreboard: pushes hard when the game is on the line.",
    },
    Persona {
        id: "oracle",
        name: "The Oracle",
        blurb: "Bids from a neural network trained on tens of thousands of solved deals; plays like The Counter.",
    },
];

pub fn persona(id: &str) -> Option<&'static Persona> {
    PERSONAS.iter().find(|p| p.id == id)
}

/// A bot for persona `id` (or "random"); `seed` drives any randomness in its play.
///
/// For lab experiments, `steady+0.3` / `steady-0.5` is Steady with that many tricks added to its
/// bidding estimates.
pub fn make_bot(id: &str, seed: u64) -> Option<Box<dyn Bot>> {
    if let Some(offset) = id.strip_prefix("steady").filter(|rest| !rest.is_empty()) {
        let aggression: f32 = offset.parse().ok()?;
        return Some(Box::new(RuleBot::new(id, Style { aggression, ..Style::STEADY }, seed)));
    }
    let style = match id {
        "steady" => Style::STEADY,
        "rookie" => Style::ROOKIE,
        "rock" => Style::CAUTIOUS,
        "gambler" => Style::BOLD,
        "random" => return Some(Box::new(RandomBot::new(seed))),
        "counter" => return Some(Box::new(SearchBot::new("The Counter", SearchConfig::COUNTER, seed))),
        "shark" => return Some(Box::new(SearchBot::new("The Shark", SearchConfig::SHARK, seed))),
        "oracle" => return Some(Box::new(SearchBot::new("The Oracle", SearchConfig::ORACLE, seed))),
        "counter-fast" => return Some(Box::new(SearchBot::new("Counter (fast)", SearchConfig::FAST, seed))),
        // Lab variants that isolate one part of the search bot.
        "search-play" => {
            let config = SearchConfig { search_discards: false, rule_aggression: 0.0, ..SearchConfig::FAST };
            return Some(Box::new(SearchBot::new("Search play", config, seed)));
        }
        "search-bid" => {
            let config = SearchConfig { search_bidding: true, search_discards: false, ..SearchConfig::FAST };
            return Some(Box::new(SearchBot::new("Search bid", config, seed)));
        }
        "oracle-nosignals" => {
            let config = SearchConfig { signals: false, ..SearchConfig::ORACLE };
            return Some(Box::new(SearchBot::new("Oracle (no signals)", config, seed)));
        }
        "counter-search-kitty" => {
            let config = SearchConfig { search_discards: true, ..SearchConfig::COUNTER };
            return Some(Box::new(SearchBot::new("Counter (searched kitty)", config, seed)));
        }
        _ => return None,
    };
    Some(Box::new(RuleBot::new(persona(id)?.name, style, seed)))
}
