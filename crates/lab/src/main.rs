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
//!   fh-lab handpuzzles [--count N]  (full-hand puzzles for Learn)
//!
//! STYLE is a persona: steady, rookie, rock, gambler (or random).

mod audit;
mod bench;
mod calibrate;
mod coach;
mod dataset;
mod declarer;
mod habits;
mod handpuzzles;
mod honesty;
mod honors;
mod leads;
mod levels;
mod opponents;
mod switch;
mod lowlead;
mod thumbcheck;
mod mentions;
mod show;
mod kxkitty;
mod bareking;
mod leadcheck;
mod acelead;
mod whatif;
mod offlead;
mod kittyworlds;
mod strainpick;
mod playout;
mod coachaudit;
mod gradestab;
mod kittyplay;
mod bidplay;
mod jokerlead;
mod bidbar;
mod lastdiscard;
mod kittyplan;
mod jokerplace;
mod dealerjoker;
mod kittysearch;
mod hintcheck;
mod plays;
mod countcheck;
mod playdata;
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
            flags.get("us").map(|us| levels::Score {
                us: us.parse().expect("--us needs a number"),
                them: flags.get("them").map_or(0, |t| t.parse().expect("--them needs a number")),
            }),
        ),
        "playdata" => playdata::run(
            flag(&flags, "deals", 1000),
            flag(&flags, "seed", 2_000_000),
            flag(&flags, "threads", std::thread::available_parallelism().map_or(4, |n| n.get() as u64)),
            flags.get("out").map_or("train/data/play", |s| s.as_str()),
        ),
        "honors" => honors::run(
            flag(&flags, "deals", 2000),
            flag(&flags, "seed", 800_000),
            flag(&flags, "threads", std::thread::available_parallelism().map_or(4, |n| n.get() as u64)),
        ),
        "countcheck" => countcheck::run(
            flag(&flags, "deals", 4000),
            flag(&flags, "seed", 4_100_000),
            flag(&flags, "threads", std::thread::available_parallelism().map_or(4, |n| n.get() as u64)),
        ),
        "plays" => plays::run(
            flag(&flags, "deals", 2000),
            flag(&flags, "seed", 3_100_000),
            flag(&flags, "threads", std::thread::available_parallelism().map_or(4, |n| n.get() as u64)),
        ),
        "hintcheck" => hintcheck::run(
            flag(&flags, "deals", 600),
            flag(&flags, "seed", 2_900_000),
            flag(&flags, "threads", std::thread::available_parallelism().map_or(4, |n| n.get() as u64)),
            flag(&flags, "reference", 256) as usize,
        ),
        "kittysearch" => kittysearch::run(
            flag(&flags, "deals", 6000),
            flag(&flags, "seed", 2_800_000),
            flag(&flags, "threads", std::thread::available_parallelism().map_or(4, |n| n.get() as u64)),
            flag(&flags, "worlds", 24) as usize,
            flag(&flags, "top", 5) as usize,
        ),
        "dealerjoker" => dealerjoker::run(
            flag(&flags, "deals", 20000),
            flag(&flags, "seed", 2_700_000),
            flag(&flags, "threads", std::thread::available_parallelism().map_or(4, |n| n.get() as u64)),
        ),
        "jokerplace" => jokerplace::run(
            flag(&flags, "deals", 30000),
            flag(&flags, "seed", 2_600_000),
            flag(&flags, "threads", std::thread::available_parallelism().map_or(4, |n| n.get() as u64)),
        ),
        "kittyplan" => kittyplan::run(
            flag(&flags, "deals", 40000),
            flag(&flags, "seed", 2_500_000),
            flag(&flags, "threads", std::thread::available_parallelism().map_or(4, |n| n.get() as u64)),
        ),
        "lastdiscard" => lastdiscard::run(
            flag(&flags, "deals", 300000),
            flag(&flags, "seed", 2_400_000),
            flag(&flags, "threads", std::thread::available_parallelism().map_or(4, |n| n.get() as u64)),
        ),
        "kxkitty" => kxkitty::run(
            flag(&flags, "deals", 40000),
            flag(&flags, "seed", 2_300_000),
            flag(&flags, "threads", std::thread::available_parallelism().map_or(4, |n| n.get() as u64)),
        ),
        "bareking" => bareking::run(
            flag(&flags, "deals", 1_000_000),
            flag(&flags, "seed", 7_100_000),
            flag(&flags, "threads", std::thread::available_parallelism().map_or(4, |n| n.get() as u64)),
        ),
        "bidbar" => bidbar::run(
            flag(&flags, "deals", 20000),
            flag(&flags, "seed", 7_300_000),
            flag(&flags, "threads", std::thread::available_parallelism().map_or(4, |n| n.get() as u64)),
        ),
        "acelead" => acelead::run(
            flag(&flags, "deals", 4000),
            flag(&flags, "seed", 7_500_000),
            flag(&flags, "threads", std::thread::available_parallelism().map_or(4, |n| n.get() as u64)),
        ),
        "jokerlead" => jokerlead::run(
            flag(&flags, "deals", 20000),
            flag(&flags, "seed", 7_400_000),
            flag(&flags, "threads", std::thread::available_parallelism().map_or(4, |n| n.get() as u64)),
            flag(&flags, "before", 6) as usize,
        ),
        "leadcheck" => leadcheck::run(
            flag(&flags, "deals", 3000),
            flag(&flags, "seed", 7_200_000),
            flag(&flags, "threads", std::thread::available_parallelism().map_or(4, |n| n.get() as u64)),
        ),
        "kittyplay" => kittyplay::run(
            flag(&flags, "deals", 3000),
            flag(&flags, "seed", 8_300_000),
            flag(&flags, "threads", std::thread::available_parallelism().map_or(4, |n| n.get() as u64)),
        ),
        "bidplay" => bidplay::run(
            flag(&flags, "deals", 6000),
            flag(&flags, "seed", 8_400_000),
            flag(&flags, "threads", std::thread::available_parallelism().map_or(4, |n| n.get() as u64)),
        ),
        "gradestab" => gradestab::run(
            flag(&flags, "deals", 800),
            flag(&flags, "seed", 8_200_000),
            flag(&flags, "threads", std::thread::available_parallelism().map_or(4, |n| n.get() as u64)),
        ),
        "coachaudit" => coachaudit::run(
            flag(&flags, "deals", 3000),
            flag(&flags, "seed", 8_100_000),
            flag(&flags, "threads", std::thread::available_parallelism().map_or(4, |n| n.get() as u64)),
        ),
        "strainpick" => strainpick::run(
            flag(&flags, "deals", 4000),
            flag(&flags, "seed", 8_000_000),
            flag(&flags, "threads", std::thread::available_parallelism().map_or(4, |n| n.get() as u64)),
        ),
        "kittyworlds" => kittyworlds::run(
            flag(&flags, "deals", 600),
            flag(&flags, "seed", 7_900_000),
            flag(&flags, "threads", std::thread::available_parallelism().map_or(4, |n| n.get() as u64)),
        ),
        "offlead" if flags.contains_key("playout") => offlead::playout(
            flag(&flags, "deals", 8000),
            flag(&flags, "seed", 7_800_000),
            flag(&flags, "threads", std::thread::available_parallelism().map_or(4, |n| n.get() as u64)),
        ),
        "offlead" => offlead::run(
            flag(&flags, "deals", 3000),
            flag(&flags, "seed", 7_700_000),
            flag(&flags, "threads", std::thread::available_parallelism().map_or(4, |n| n.get() as u64)),
        ),
        "whatif" => whatif::run(
            flags.get("code").map_or("", |s| s.as_str()),
            flag(&flags, "at", 0) as usize,
            flags.get("a").map_or("", |s| s.as_str()),
            flags.get("b").map_or("", |s| s.as_str()),
            flag(&flags, "worlds", 200) as usize,
            flag(&flags, "threads", std::thread::available_parallelism().map_or(4, |n| n.get() as u64)),
        ),
        "show" => show::run(flags.get("code").map_or("", |s| s.as_str()), flags.get("at").and_then(|s| s.parse().ok()), flags.get("discards").map(|s| s.as_str())),
        "handpuzzles" => handpuzzles::run(
            flag(&flags, "count", 40) as usize,
            flag(&flags, "seed", 7_000_000),
            flag(&flags, "threads", std::thread::available_parallelism().map_or(4, |n| n.get() as u64)),
            flags.get("out").map_or("web/src/learn/handPuzzles.json", |s| s.as_str()),
        ),
        "mentions" => mentions::run(
            flag(&flags, "deals", 3000),
            flag(&flags, "seed", 2_000_000),
            flag(&flags, "threads", std::thread::available_parallelism().map_or(4, |n| n.get() as u64)),
        ),
        "thumbcheck" => thumbcheck::run(
            flag(&flags, "deals", 4000),
            flag(&flags, "seed", 1_900_000),
            flag(&flags, "threads", std::thread::available_parallelism().map_or(4, |n| n.get() as u64)),
        ),
        "lowlead" => lowlead::run(
            flag(&flags, "deals", 3000),
            flag(&flags, "seed", 1_400_000),
            flag(&flags, "threads", std::thread::available_parallelism().map_or(4, |n| n.get() as u64)),
        ),
        "switch" => switch::run(
            flag(&flags, "deals", 20000),
            flag(&flags, "seed", 1_300_000),
            flag(&flags, "threads", std::thread::available_parallelism().map_or(4, |n| n.get() as u64)),
        ),
        "opponents" => opponents::run(
            flag(&flags, "deals", 3000),
            flag(&flags, "seed", 900_000),
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
            flags.contains_key("leads-only"),
            flags.contains_key("nt"),
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
