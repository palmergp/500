//! `fh-lab`: play 500 in the terminal, or run headless bot simulations.
//!
//!   fh-lab play [--seed N] [--watch]
//!   fh-lab sim  [--games N] [--seed N] [--ns STYLE] [--ew STYLE] [--threads N]
//!
//! STYLE is one of: steady, cautious, bold, random.

mod sim;
mod terminal;

use std::collections::HashMap;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let (command, flags) = match args.split_first() {
        Some((command, rest)) => (command.as_str(), parse_flags(rest)),
        None => ("help", HashMap::new()),
    };
    match command {
        "play" => terminal::run(flag(&flags, "seed", now_seed()), flags.contains_key("watch")),
        "sim" => sim::run(sim::Options {
            games: flag(&flags, "games", 1000),
            seed: flag(&flags, "seed", 1),
            ns: flags.get("ns").cloned().unwrap_or_else(|| "steady".into()),
            ew: flags.get("ew").cloned().unwrap_or_else(|| "steady".into()),
            threads: flag(&flags, "threads", std::thread::available_parallelism().map_or(4, |n| n.get() as u64)),
        }),
        _ => {
            println!("usage:\n  fh-lab play [--seed N] [--watch]\n  fh-lab sim [--games N] [--seed N] [--ns STYLE] [--ew STYLE] [--threads N]");
            println!("STYLE: steady | cautious | bold | random");
        }
    }
}

fn parse_flags(args: &[String]) -> HashMap<String, String> {
    let mut flags = HashMap::new();
    let mut i = 0;
    while i < args.len() {
        let name = args[i].trim_start_matches("--").to_string();
        match args.get(i + 1).filter(|v| !v.starts_with("--")) {
            Some(value) => {
                flags.insert(name, value.clone());
                i += 2;
            }
            None => {
                flags.insert(name, String::new());
                i += 1;
            }
        }
    }
    flags
}

fn flag(flags: &HashMap<String, String>, name: &str, default: u64) -> u64 {
    match flags.get(name) {
        Some(v) => v.parse().unwrap_or_else(|_| panic!("--{name} needs a number, got {v:?}")),
        None => default,
    }
}

fn now_seed() -> u64 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0, |d| d.as_nanos() as u64)
}

/// A bot of the named style for `seat`.
pub fn make_bot(style: &str, seed: u64) -> Box<dyn fh_ai::Bot> {
    use fh_ai::{RandomBot, RuleBot, Style};
    match style {
        "steady" => Box::new(RuleBot::new("Steady", Style::STEADY)),
        "cautious" => Box::new(RuleBot::new("Cautious", Style::CAUTIOUS)),
        "bold" => Box::new(RuleBot::new("Bold", Style::BOLD)),
        "random" => Box::new(RandomBot::new(seed)),
        other => panic!("unknown bot style {other:?} (steady, cautious, bold, random)"),
    }
}
