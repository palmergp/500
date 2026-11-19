//! The cast of AI players (PLAN.md §5). Each persona is a bot engine plus a style.

use crate::{Bot, RandomBot, RuleBot, SearchBot, SearchConfig, Style};

pub struct Persona {
    pub id: &'static str,
    pub name: &'static str,
    pub blurb: &'static str,
}

pub const PERSONAS: [Persona; 8] = [
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
    Persona {
        id: "natural",
        name: "The Natural",
        blurb: "Plays by instinct: a network that learned card play from the expert's search. Quick, and stronger than the rule players.",
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
        // The play network, with The Oracle's bidding and kitty (docs/lab/README.md, finding 22).
        "natural" => return crate::neural_play::NeuralBot::new("The Natural", seed).map(|b| Box::new(b) as Box<dyn Bot>),
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
        "neural" => return crate::neural_play::NeuralBot::new("Neural play", seed).map(|b| Box::new(b) as Box<dyn Bot>),
        // The Oracle, skipping its search when the play network is confident (lab: quick-0.8 etc.).
        quick if quick.starts_with("quick-") => {
            let p: f32 = quick.strip_prefix("quick-")?.parse().ok()?;
            return crate::neural_play::NeuralBot::quick("Oracle (quick)", seed, p).map(|b| Box::new(b) as Box<dyn Bot>);
        }
        "oracle-nocalls" => {
            let config = SearchConfig { signal_calls: false, ..SearchConfig::ORACLE };
            return Some(Box::new(SearchBot::new("Oracle (no mentions or 6-calls)", config, seed)));
        }
        "oracle-noinference" => {
            let config = SearchConfig { inference: false, read_signals: false, ..SearchConfig::ORACLE };
            return Some(Box::new(SearchBot::new("Oracle (ignores what calls show)", config, seed)));
        }
        "oracle-nobidweights" => {
            // Reads discard signals but ignores what the auction suggests about hands.
            let config = SearchConfig { inference: false, ..SearchConfig::ORACLE };
            return Some(Box::new(SearchBot::new("Oracle (ignores the auction)", config, seed)));
        }
        "oracle-nosignalread" => {
            // Weighs the auction but ignores discard signals.
            let config = SearchConfig { read_signals: false, ..SearchConfig::ORACLE };
            return Some(Box::new(SearchBot::new("Oracle (ignores discard signals)", config, seed)));
        }
        "oracle-fixedpass" => {
            let config = SearchConfig { fixed_opponents: true, ..SearchConfig::ORACLE };
            return Some(Box::new(SearchBot::new("Oracle (fixed pass value)", config, seed)));
        }
        // Item 11 of the lab list: is a 6-call on a weak hand worth it as the first to call?
        "oracle-firstsix6" => {
            let config = SearchConfig { six: crate::rule_bot::SixPolicy::FirstNeeds(6.0), ..SearchConfig::ORACLE };
            return Some(Box::new(SearchBot::new("Oracle (first to call: 6 only from a count of 6)", config, seed)));
        }
        "oracle-nosix" => {
            let config = SearchConfig { six: crate::rule_bot::SixPolicy::Never, ..SearchConfig::ORACLE };
            return Some(Box::new(SearchBot::new("Oracle (never calls 6)", config, seed)));
        }
        "oracle-leadwinners" => {
            let config = SearchConfig { lead_winners: true, ..SearchConfig::ORACLE };
            return Some(Box::new(SearchBot::new("Oracle (declarer leads winners on near-ties)", config, seed)));
        }
        "oracle-openjoker" => {
            let config = SearchConfig { hold_opening_joker: false, ..SearchConfig::ORACLE };
            return Some(Box::new(SearchBot::new("Oracle (may lead the Joker to the first no-trump trick)", config, seed)));
        }
        "oracle-latesignals" => {
            let config = SearchConfig { signal_after_partner: true, ..SearchConfig::ORACLE };
            return Some(Box::new(SearchBot::new("Oracle (mentions and 6-calls after partner's call too)", config, seed)));
        }
        "oracle-dearsignal" => {
            // The old allowance (finding 36).
            let config = SearchConfig { signal_allowance: 25.0, ..SearchConfig::ORACLE };
            return Some(Box::new(SearchBot::new("Oracle (signals at up to 25 points)", config, seed)));
        }
        // The guide's bidding (docs/rules-of-thumb.md) with The Counter's play, and variants that
        // drop one rule (docs/lab/README.md: the guide as a whole).
        guide if guide.starts_with("guide") => {
            use crate::guide_bot::{GuideBot, GuideRules};
            let all = GuideRules::ALL;
            let rules = match guide {
                "guide" => all,
                "guide-noauction" => GuideRules { auction: false, ..all },
                "guide-nine" => GuideRules { stop_at_eight: false, ..all },
                "guide-overcalls" => GuideRules { cautious_overcalls: false, ..all },
                "guide-nosignals" => GuideRules { signals: false, ..all },
                "guide-bold" => GuideRules { boldness: 0.5, ..all },
                "guide-bolder" => GuideRules { boldness: 1.0, ..all },
                "guide-boldest" => GuideRules { boldness: 1.5, ..all },
                _ => return None,
            };
            return Some(Box::new(GuideBot::new("The guide", rules, seed)));
        }
        "oracle-altnet" => {
            let config = SearchConfig { alt_bid_model: true, ..SearchConfig::ORACLE };
            return Some(Box::new(SearchBot::new("Oracle (the network in FH_ALT_BID_MODEL)", config, seed)));
        }
        "oracle-novoids" => {
            let config = SearchConfig { track_voids: false, ..SearchConfig::ORACLE };
            return Some(Box::new(SearchBot::new("Oracle (doesn't track voids)", config, seed)));
        }
        "counter-search-kitty" => {
            let config = SearchConfig { search_discards: true, ..SearchConfig::COUNTER };
            return Some(Box::new(SearchBot::new("Counter (searched kitty)", config, seed)));
        }
        _ => return None,
    };
    Some(Box::new(RuleBot::new(persona(id)?.name, style, seed)))
}
