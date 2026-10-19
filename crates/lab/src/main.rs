//! `fh-lab`: play 500 in the terminal, or run headless bot simulations.
//!
//!   fh-lab play [--seed N] [--watch]
//!   fh-lab sim  [--games N] [--seed N] [--ns STYLE] [--ew STYLE] [--threads N]
//!   fh-lab bench [--deals N]   (double-dummy solver speed)
//!   fh-lab duel  [--a BOT] [--b BOT] [--deals N] [--seed N] [--threads N]   (duplicate match)
//!   fh-lab levels  [--deals N]   (how high to bid, by seat)
//!   fh-lab honesty [--deals N]   (are the coach's percentages right?)
//!   fh-lab audit [--deals N]     (are the coach's grades fair and its hints steady?)
//!   fh-lab declarer [--deals N]  (the kitty and the first lead, solved)
//!
//! STYLE is a persona: steady, rookie, rock, gambler (or random).

mod audit;
mod bench;
mod calibrate;
mod coach;
mod dataset;
mod declarer;
mod habits;
mod honesty;
mod leads;
mod levels;
mod duel;
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
        "duel" => duel::run({
            duel::VERBOSE.store(flags.contains_key("verbose"), std::sync::atomic::Ordering::Relaxed);
            duel::Options {
            a: flags.get("a").cloned().unwrap_or_else(|| "counter-fast".into()),
            b: flags.get("b").cloned().unwrap_or_else(|| "steady".into()),
            deals: flag(&flags, "deals", 200),
            seed: flag(&flags, "seed", 1),
            threads: flag(&flags, "threads", std::thread::available_parallelism().map_or(4, |n| n.get() as u64)),
        }}),
        "calibrate" => calibrate::run(
            flag(&flags, "deals", 100),
            flag(&flags, "seed", 1),
            flag(&flags, "threads", std::thread::available_parallelism().map_or(4, |n| n.get() as u64)),
            flags.get("play").map_or("search-play", |s| s.as_str()),
        ),
        "coach" => coach::run(flag(&flags, "seed", 1), flags.get("south").map_or("steady", |s| s.as_str())),
        "dataset" => dataset::run(
            flag(&flags, "deals", 1000),
            flag(&flags, "seed", 1_000_000),
            flag(&flags, "threads", std::thread::available_parallelism().map_or(4, |n| n.get() as u64)),
            flags.get("out").map_or("train/data/bid", |s| s.as_str()),
            flags.get("bidder").map_or("rule", |s| s.as_str()),
        ),
        "habits" => habits::run(
            flag(&flags, "deals", 500),
            flag(&flags, "seed", 70_000),
            flag(&flags, "threads", std::thread::available_parallelism().map_or(4, |n| n.get() as u64)),
            flags.get("bot").map_or("counter", |s| s.as_str()),
        ),
        "leads" => leads::run(
            flag(&flags, "deals", 400),
            flag(&flags, "seed", 120_000),
            flag(&flags, "threads", std::thread::available_parallelism().map_or(4, |n| n.get() as u64)),
        ),
        "levels" => levels::run(
            flag(&flags, "deals", 2000),
            flag(&flags, "seed", 300_000),
            flag(&flags, "threads", std::thread::available_parallelism().map_or(4, |n| n.get() as u64)),
        ),
        "audit" => audit::run(
            flag(&flags, "deals", 150),
            flag(&flags, "seed", 500_000),
            flag(&flags, "threads", std::thread::available_parallelism().map_or(4, |n| n.get() as u64)),
        ),
        "declarer" => declarer::run(
            flag(&flags, "deals", 3000),
            flag(&flags, "seed", 600_000),
            flag(&flags, "threads", std::thread::available_parallelism().map_or(4, |n| n.get() as u64)),
        ),
        "honesty" => honesty::run(
            flag(&flags, "deals", 300),
            flag(&flags, "seed", 400_000),
            flag(&flags, "threads", std::thread::available_parallelism().map_or(4, |n| n.get() as u64)),
        ),
        "bench" => bench::run(flag(&flags, "deals", 200), flag(&flags, "seed", 1)),
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
            println!("  fh-lab bench [--deals N]
  fh-lab duel [--a BOT] [--b BOT] [--deals N] [--seed N] [--threads N]");
            println!("STYLE / BOT: steady | rookie | rock | gambler | counter | shark | counter-fast | random");
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

/// A bot for the named persona.
pub fn make_bot(persona: &str, seed: u64) -> Box<dyn fh_ai::Bot> {
    fh_ai::make_bot(persona, seed)
        .unwrap_or_else(|| panic!("unknown bot {persona:?} (steady, rookie, rock, gambler, random)"))
}
