//! A plain terminal game: you are South, partnered with North, against two bots.

use crate::make_bot;
use fh_ai::cardplay::suit_order;
use fh_core::{Action, Card, CardSet, Event, Game, Phase, Rules, Seat, Suit, Team};
use std::io::{self, BufRead, Write};

pub fn run(seed: u64, watch: bool) {
    let mut game = Game::new(Rules::default(), seed);
    let mut bots: [Box<dyn fh_ai::Bot>; 4] = std::array::from_fn(|i| make_bot("steady", seed * 4 + i as u64));
    println!("500 — seed {seed}. You are South; your partner is North. First to 500 wins.");
    if watch {
        println!("(watching: a bot plays your seat)");
    }
    print_deal(&game, watch);
    let stdin = io::stdin();
    let mut lines = stdin.lock().lines();

    while game.phase() != Phase::GameOver {
        let seat = game.to_act().unwrap();
        let legal = game.legal_actions();
        let action = if seat == Seat::SOUTH && !watch {
            show_your_turn(&game);
            match ask(&legal, &mut lines) {
                Some(action) => action,
                None => return,
            }
        } else {
            bots[seat.index()].choose(&game.observe(seat), &legal)
        };
        let events = game.apply(action).expect("legal action");
        for event in events {
            narrate(&game, &event, watch);
        }
    }
}

fn ask(legal: &[Action], lines: &mut impl Iterator<Item = io::Result<String>>) -> Option<Action> {
    for (i, action) in legal.iter().enumerate() {
        print!("{:>3}) {:<16}", i + 1, action.to_string());
        if (i + 1) % 5 == 0 {
            println!();
        }
    }
    println!();
    loop {
        print!("Your choice (1-{}, q to quit): ", legal.len());
        io::stdout().flush().ok();
        let line = lines.next()?.ok()?;
        let line = line.trim();
        if line.eq_ignore_ascii_case("q") {
            return None;
        }
        match line.parse::<usize>() {
            Ok(n) if (1..=legal.len()).contains(&n) => return Some(legal[n - 1]),
            _ => println!("Please type a number from the list."),
        }
    }
}

/// A hand grouped by effective suit, strongest first; the trump suit (with its bowers) comes last.
pub fn format_hand(hand: CardSet, trump: Option<Suit>) -> String {
    let mut suits: Vec<Suit> = Suit::ALL.into_iter().filter(|&s| Some(s) != trump).collect();
    suits.extend(trump);
    let mut groups: Vec<String> = suits
        .iter()
        .filter_map(|&suit| {
            let cards: Vec<String> =
                suit_order(suit, trump).into_iter().filter(|&c| hand.contains(c)).map(|c| c.to_string()).collect();
            let label = if Some(suit) == trump { format!("{suit} trumps") } else { suit.to_string() };
            (!cards.is_empty()).then(|| format!("{label}: {}", cards.join(" ")))
        })
        .collect();
    if trump.is_none() && hand.contains(Card::JOKER) {
        groups.push("Joker".into());
    }
    groups.join("   ")
}

fn show_your_turn(game: &Game) {
    let obs = game.observe(Seat::SOUTH);
    println!();
    if let Some(trick) = &obs.current_trick {
        let played: Vec<String> = trick.plays().map(|(s, c)| format!("{s} {c}")).collect();
        println!("Trick {}: {}", obs.tricks.len() + 1, if played.is_empty() { "you lead".into() } else { played.join(", ") });
    }
    if obs.phase == Phase::Discarding {
        println!("You won the bid. Kitty: {}. Discard {} more.", obs.kitty.unwrap(), 3 - obs.discards.unwrap().len());
    }
    if obs.auction.must_bid() {
        println!("Third deal with no bid: as dealer you must bid.");
    }
    println!("Your hand  {}", format_hand(obs.hand, obs.trump()));
}

fn print_deal(game: &Game, watch: bool) {
    let note = if game.auction().dealer_forced() { " — dealer must bid if nobody else does" } else { "" };
    println!();
    println!("=== Deal {} of 3 · dealer {} · N–S {}  E–W {}{note}", game.deal_number(), game.dealer(), game.scores()[0], game.scores()[1]);
    let seats: &[Seat] = if watch { &Seat::ALL } else { &[Seat::SOUTH] };
    for &seat in seats {
        println!("{:<6} {}", seat.name(), format_hand(game.hand(seat), None));
    }
}

fn narrate(game: &Game, event: &Event, watch: bool) {
    match event {
        Event::Dealt { .. } => print_deal(game, watch),
        Event::Called { seat, call } => println!("{seat:<6} {call}"),
        Event::PassedOut { dealer } => println!("Nobody bid 7 or more: {dealer} redeals."),
        Event::ContractWon { declarer, contract } => {
            println!("{declarer} wins the bid at {contract} ({} points) and takes the kitty.", contract.value())
        }
        Event::Discarded { .. } => {}
        Event::Played { seat, play } => println!("  {seat:<6} {play}"),
        Event::TrickWon { winner, trick_number } => {
            let won = game.tricks_won();
            println!("  -> {winner} wins trick {trick_number}  (N–S {}, E–W {})", won[0], won[1]);
        }
        Event::HandScored { declarer, contract, tricks_won, score_change, scores } => {
            let team = declarer.team().index();
            let verdict = if tricks_won[team] >= contract.level { "made" } else { "went down in" };
            println!();
            println!("{} {verdict} {contract}: N–S {:+}, E–W {:+}  →  N–S {}, E–W {}",
                declarer.team(), score_change[0], score_change[1], scores[0], scores[1]);
        }
        Event::GameWon { team } => {
            let you = if *team == Team::NorthSouth { "You and North win!" } else { "East–West win." };
            println!("\n*** {team} reach {} — {you}", game.scores()[team.index()]);
        }
    }
}
