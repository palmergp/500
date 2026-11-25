//! `fh-lab show --code H7K2-9QXA`: the game a hand code names, as the player saw it, with the
//! coach's view of the decision there. Codes come from the web app's code button: a short code
//! is looked up in `%LOCALAPPDATA%/500/hands` (the host saves them there); a long one ("500:…")
//! carries the game itself. `--at N` also shows the decision at action N, whoever made it: what
//! that seat's bot would do, The Oracle's ranking of every choice, and (in card play) what each is
//! worth with every card visible.

use fh_core::{Action, Seat};
use fh_wasm::table::{hands_dir, replay, SaveData, Table};

pub fn run(code: &str, at: Option<usize>, discards: Option<&str>) {
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

    if let Some(at) = at {
        decision_at(&save, at);
        if let Some(cards) = discards {
            grade_discards(&save, at, cards);
        }
        return;
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

/** The decision at action `at`: the seat's bot, The Oracle's ranking, and the solver's values. */
/// `--discards "6S 8S 4H"` at the declarer's first discard: the coach's grade of putting those
/// back, against its plan, with five different seeds (each compares the two sets over its own
/// sampled deals), so a close kitty choice shows how steady the verdict is.
fn grade_discards(save: &SaveData, at: usize, cards: &str) {
    let Ok(game) = replay(save, at) else { return };
    let Some(seat) = game.to_act() else { return };
    let chosen: Vec<fh_core::Card> = cards.split_whitespace().filter_map(fh_core::Card::parse).collect();
    let obs = game.observe(seat);
    println!("
  Putting back {}:", chosen.iter().map(|c| c.to_string()).collect::<Vec<_>>().join(" "));
    let mut plan = Vec::new();
    for k in 0..5u64 {
        let g = fh_ai::coach::Coach::new(save.seed ^ at as u64 ^ (k << 32)).grade_discard_set(&obs, &chosen);
        println!("    seed {k}: {:?}, {:.0} points behind the plan {}", g.verdict, g.loss, g.plan.iter().map(|c| c.to_string()).collect::<Vec<_>>().join(" "));
        if let Some(r) = g.reasons.get(1) {
            println!("      {r}");
        }
        plan = g.plan;
    }
    // The two sets on many more deals: where each is better, and by how many tricks.
    let (declarer, contract) = game.contract().expect("a contract");
    let trump = contract.trump();
    // Deals that fit the auction, as the coach's kitty grades sample them (finding 58).
    let belief = fh_ai::belief::Belief::new_with(&obs, true, true);
    let worlds = belief.sample_fitting(&mut fh_core::Rng::new(save.seed ^ 0xd15c), obs.hand, 400);
    let mut solver = fh_ai::solver::Solver::with_limit(trump, 3_000_000);
    let (mut plan_better, mut yours_better, mut same, mut made) = (0, 0, 0, [0, 0]);
    let mut examples = Vec::new();
    for w in &worlds {
        let mut tricks = [0u8; 2];
        for (i, set) in [&plan, &chosen].iter().enumerate() {
            let mut hands = w.hands;
            hands[declarer.index()] = obs.hand.minus(set.iter().copied().collect());
            tricks[i] = solver.solve(&fh_ai::solver::Position { hands, trump, trick: fh_core::Trick::new(declarer) }, declarer.team());
            made[i] += usize::from(tricks[i] >= contract.level);
        }
        match tricks[0].cmp(&tricks[1]) {
            std::cmp::Ordering::Greater => {
                plan_better += 1;
                if examples.len() < 3 {
                    examples.push((w.hands, tricks));
                }
            }
            std::cmp::Ordering::Less => yours_better += 1,
            std::cmp::Ordering::Equal => same += 1,
        }
    }
    println!("    over {} deals (every card seen): the plan takes more tricks in {plan_better}, yours in {yours_better}, the same in {same}; the bid makes {} times with the plan, {} with yours", worlds.len(), made[0], made[1]);
    for (hands, tricks) in examples {
        println!("      e.g. plan {} tricks, yours {}: {}", tricks[0], tricks[1], Seat::ALL.iter().filter(|&&s| s != seat).map(|&s| format!("{s} {}", hands[s.index()])).collect::<Vec<_>>().join(" | "));
    }
}

fn decision_at(save: &SaveData, at: usize) {
    use fh_ai::search_bot::{SearchBot, SearchConfig};
    use fh_ai::solver::{Position, Solver};
    let game = match replay(save, at) {
        Ok(g) => g,
        Err(e) => return println!("
(can't replay to action {at}: {e})"),
    };
    let Some(seat) = game.to_act() else { return println!("
Nobody acts at action {at}.") };
    let legal = game.legal_actions();
    let obs = game.observe(seat);
    let persona = save.players[seat.index()].as_str();
    let chosen = save.log.get(at).map(|&i| legal[i as usize]);
    println!("
Action {at}: {seat} ({persona}) to act, holding {}", game.hand(seat));
    if let Some(c) = chosen {
        println!("  In the game: {c}");
    }
    if let Some(mut bot) = fh_ai::personas::make_bot(persona, save.seed) {
        println!("  {persona} now would choose: {}", bot.choose(&obs, &legal));
    }
    // Your own decision: the coach's grade of it, as the app showed it.
    if seat == Seat::SOUTH && save.players[0] == "human" {
        match fh_wasm::coach::grade_for(save, at, seat) {
            Ok(Some(g)) => {
                println!("  The coach: {} ({} {:.0}{}), best {}", g.verdict, g.chosen_label, g.loss, if g.units == "win" { " tenths of a percent" } else { " points" }, g.best_label);
                for r in &g.reasons {
                    println!("    · {r}");
                }
            }
            Ok(None) => println!("  (no choice to grade)"),
            Err(e) => println!("  (no grade: {e})"),
        }
    }
    if game.phase() != fh_core::Phase::Playing {
        return;
    }
    let mut oracle = SearchBot::new("The Oracle", SearchConfig::ORACLE, save.seed);
    let ranked = oracle.evaluate_plays(&obs, &legal);
    let solved = (game.phase() == fh_core::Phase::Playing).then(|| {
        let (_, contract) = game.contract().expect("a contract in play");
        let trump = contract.trump();
        let hands: [fh_core::CardSet; 4] = std::array::from_fn(|i| game.hand(Seat::new(i)));
        let trick = game.current_trick().cloned().unwrap_or_else(|| fh_core::Trick::new(seat));
        Solver::new(trump).analyze(&Position { hands, trump, trick }, seat.team())
    });
    println!("  The Oracle's ranking (expected points), and tricks for {seat}'s side from here with every card seen:");
    for (action, ev) in &ranked {
        let tricks = solved.as_ref().and_then(|v| v.iter().find(|(p, _)| Action::Play(*p) == *action).map(|&(_, t)| t));
        println!("    {:<12} ev {:>7.1}   {}", action.to_string(), ev, tricks.map_or(String::new(), |t| format!("{t} tricks")));
    }
}

pub(crate) fn load(code: &str) -> Result<SaveData, String> {
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
