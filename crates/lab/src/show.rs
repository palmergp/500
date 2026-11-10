//! `fh-lab show --code H7K2-9QXA`: the game a hand code names, as the player saw it, with the
//! coach's view of the decision there. Codes come from the web app's code button: a short code
//! is looked up in `%LOCALAPPDATA%/500/hands` (the host saves them there); a long one ("500:…")
//! carries the game itself.

use fh_core::{Action, Seat};
use fh_wasm::table::{hands_dir, replay, SaveData, Table};

pub fn run(code: &str) {
    let save = match load(code) {
        Ok(save) => save,
        Err(e) => {
            eprintln!("{e}");
            std::process::exit(1);
        }
    };
    let table = match Table::restore(&save) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("the code's game doesn't replay: {e}");
            std::process::exit(1);
        }
    };
    let game = table.game();
    let names: Vec<String> = Seat::ALL.iter().map(|&s| format!("{s} {}", save.players[s.index()])).collect();
    println!("Code {code}: seed {}, {}, {} actions", save.seed, save.hands.map_or("a game to 500".into(), |h| format!("a {h}-hand game")), save.log.len());
    println!("Players: {}", names.join(", "));
    let scores = game.scores();
    println!("Scores: North–South {} (South is the player), East–West {}; hands played {}", scores[0], scores[1], game.history().iter().filter(|h| h.contract.is_some()).count());

    // The deal in progress (or the last one, if the code is at the end of a hand).
    let starts = table.notes().deal_starts;
    let start = *starts.iter().rev().find(|&&s| s < save.log.len()).unwrap_or(&0);
    let mut deal = replay(&save, start).expect("replays");
    let earlier = deal.history().len();
    println!("\nDeal {} (dealer {}):", deal.history().len() + 1, deal.dealer());
    for seat in Seat::ALL {
        println!("  {:<5} {}", seat.to_string(), deal.hand(seat));
    }
    println!("  kitty {}", deal.kitty());
    println!();
    let mut trick_cards = Vec::new();
    for (i, &index) in save.log[start..].iter().enumerate() {
        let seat = deal.to_act().expect("someone to act");
        let action = deal.legal_actions()[index as usize];
        let number = start + i;
        match action {
            Action::Play(p) => {
                trick_cards.push(format!("{seat} {}", p.card));
                if trick_cards.len() == 4 {
                    println!("  trick: {}   (action {})", trick_cards.join(", "), number);
                    trick_cards.clear();
                }
            }
            Action::Discard(card) => println!("  {seat} discards {card}"),
            other => println!("  {seat} {other}   (action {number})"),
        }
        deal.apply(action).expect("replays");
    }
    if !trick_cards.is_empty() {
        println!("  trick so far: {}", trick_cards.join(", "));
    }
    // The code is at the end of this deal: how it came out.
    if let Some(h) = deal.history().get(earlier) {
        match h.contract {
            Some((declarer, contract)) => println!(
                "\n{declarer} played {contract} and took {} tricks; score change North–South {:+}, East–West {:+}",
                h.tricks_won[declarer.team().index()],
                h.score_change[0],
                h.score_change[1]
            ),
            None => println!("\nThe deal was thrown in."),
        }
    }

    // The coach at the code's position, if it's South's decision.
    match game.to_act() {
        Some(Seat::SOUTH) if game.legal_actions().len() > 1 => match fh_wasm::coach::hint_for(&save, Seat::SOUTH) {
            Ok(h) => {
                println!("\nSouth to act. Coach suggests: {}", h.best_label);
                for r in &h.reasons {
                    println!("    · {r}");
                }
                for o in &h.options {
                    let makes = o.makes.map(|m| format!("makes {:.0}%", 100.0 * m)).unwrap_or_default();
                    println!("    {:<8} ev {:>7.1}  {makes}  {}", o.label, o.ev, o.note.clone().unwrap_or_default());
                }
                let chance = |o: &fh_wasm::coach::OptionView| format!("{} {}", o.label, o.makes.map_or("—".into(), |m| format!("{:.0}%", 100.0 * m)));
                if !h.bids.is_empty() {
                    println!("  Every bid's chance: {}", h.bids.iter().map(chance).collect::<Vec<_>>().join(", "));
                }
                if !h.plays.is_empty() {
                    println!("  Every card: {}", h.plays.iter().map(|o| format!("{} ({})", chance(o), o.note.clone().unwrap_or_default())).collect::<Vec<_>>().join("; "));
                }
                // In the auction, what the expert's own bidding weighed: the best bid against the
                // baseline (letting the current high bid stand), which the hint doesn't list.
                if game.phase() == fh_core::Phase::Bidding {
                    use fh_ai::Bot;
                    let mut oracle = fh_ai::search_bot::SearchBot::new("The Oracle", fh_ai::search_bot::SearchConfig::ORACLE, save.seed);
                    let action = oracle.choose(&game.observe(Seat::SOUTH), &game.legal_actions());
                    println!("  The Oracle would {action}: {}", oracle.note().unwrap_or("(no note)"));
                }
            }
            Err(e) => println!("\n(no hint: {e})"),
        },
        Some(seat) => println!("\n{seat} to act."),
        None => println!("\nThe game is over."),
    }
}

fn load(code: &str) -> Result<SaveData, String> {
    let json = if let Some(data) = code.strip_prefix("500:") {
        String::from_utf8(base64url(data).ok_or("the long code is damaged")?).map_err(|_| "the long code is damaged".to_string())?
    } else {
        let dir = hands_dir().ok_or("no app-data folder")?;
        let path = dir.join(format!("{}.json", code.trim().to_uppercase()));
        let text = std::fs::read_to_string(&path).map_err(|_| format!("no hand with code {code} in {}", dir.display()))?;
        let record: serde_json::Value = serde_json::from_str(&text).map_err(|e| e.to_string())?;
        record["save"].as_str().ok_or("the saved hand is damaged")?.to_string()
    };
    serde_json::from_str(&json).map_err(|e| format!("not a saved game: {e}"))
}

/// Decodes unpadded base64url.
fn base64url(text: &str) -> Option<Vec<u8>> {
    let value = |c: u8| match c {
        b'A'..=b'Z' => Some(c - b'A'),
        b'a'..=b'z' => Some(c - b'a' + 26),
        b'0'..=b'9' => Some(c - b'0' + 52),
        b'-' => Some(62),
        b'_' => Some(63),
        _ => None,
    };
    let mut out = Vec::new();
    let (mut acc, mut bits) = (0u32, 0);
    for c in text.trim().bytes() {
        acc = (acc << 6) | value(c)? as u32;
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            out.push((acc >> bits) as u8);
            acc &= (1 << bits) - 1;
        }
    }
    Some(out)
}
