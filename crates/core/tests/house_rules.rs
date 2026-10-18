//! One or more tests per rule in docs/rules/house-rules.md (IDs in test names and comments).

use fh_core::cards::ACE;
use fh_core::game::{game_winner, score_hand};
use fh_core::rules::{effective_suit, is_trump};
use fh_core::*;

fn card(text: &str) -> Card {
    Card::parse(text).unwrap_or_else(|| panic!("bad card {text}"))
}

fn cards(text: &str) -> CardSet {
    CardSet::parse(text).unwrap_or_else(|| panic!("bad cards {text}"))
}

fn plays(text: &str) -> Vec<Play> {
    text.split_whitespace().map(|w| Play::card(card(w))).collect()
}

fn legal_cards(hand: &str, trick: &Trick, trump: Option<Suit>) -> CardSet {
    legal_plays(cards(hand), trick, trump).iter().map(|p| p.card).collect()
}

fn bid(level: u8, strain: Strain) -> Call {
    Call::Bid(Contract::new(level, strain))
}

const H: Option<Suit> = Some(Suit::Hearts);
const S: Option<Suit> = Some(Suit::Spades);
const NT: Option<Suit> = None;

// ---- Table and cards ------------------------------------------------------------------------

#[test]
fn h2_pack_has_43_cards() {
    let pack = CardSet::pack();
    assert_eq!(pack.len(), 43);
    assert!(pack.contains(Card::JOKER));
    assert!(pack.contains(card("4H")) && pack.contains(card("4D")));
    assert_eq!(Card::parse("4S"), None);
    assert_eq!(Card::parse("4C"), None);
    for suit in Suit::ALL {
        let expected = if suit.is_red() { 11 } else { 10 };
        assert_eq!(pack.of_printed_suit(suit).len(), expected, "{suit}");
    }
}

#[test]
fn h3_deal_is_ten_each_plus_three_kitty() {
    for seed in 0..50 {
        let game = Game::new(Rules::default(), seed);
        for seat in Seat::ALL {
            assert_eq!(game.hand(seat).len(), 10);
        }
        assert_eq!(game.kitty().len(), 3);
        game.check_invariants().unwrap();
    }
}

#[test]
fn h3_same_seed_same_game() {
    let a = Game::new(Rules::default(), 99);
    let b = Game::new(Rules::default(), 99);
    assert_eq!(a.dealer(), b.dealer());
    for seat in Seat::ALL {
        assert_eq!(a.hand(seat), b.hand(seat));
    }
    assert_ne!(Game::new(Rules::default(), 100).hand(Seat::SOUTH), a.hand(Seat::SOUTH));
}

// ---- Card ranking ---------------------------------------------------------------------------

/// Cards of `trump`'s effective suit, strongest first.
fn trumps_in_order(trump: Suit) -> Vec<Card> {
    let mut trumps: Vec<Card> = CardSet::pack().iter().filter(|&c| is_trump(c, Some(trump))).collect();
    trumps.sort_by_key(|&c| std::cmp::Reverse(fh_core::trick::trick_strength(c, trump, Some(trump))));
    trumps
}

#[test]
fn h4_trump_ranking_hearts() {
    let order: Vec<String> = trumps_in_order(Suit::Hearts).iter().map(|c| c.to_string()).collect();
    assert_eq!(
        order.join(" "),
        "Joker J♥ J♦ A♥ K♥ Q♥ 10♥ 9♥ 8♥ 7♥ 6♥ 5♥ 4♥",
        "13 trumps in a red suit"
    );
}

#[test]
fn h4_trump_ranking_spades() {
    let order: Vec<String> = trumps_in_order(Suit::Spades).iter().map(|c| c.to_string()).collect();
    assert_eq!(order.join(" "), "Joker J♠ J♣ A♠ K♠ Q♠ 10♠ 9♠ 8♠ 7♠ 6♠ 5♠", "12 trumps in a black suit");
}

#[test]
fn h5_left_bower_and_joker_are_trumps() {
    assert_eq!(effective_suit(card("JD"), H), Some(Suit::Hearts));
    assert_eq!(effective_suit(card("JH"), H), Some(Suit::Hearts));
    assert_eq!(effective_suit(Card::JOKER, H), Some(Suit::Hearts));
    assert_eq!(effective_suit(card("JC"), H), Some(Suit::Clubs), "black jacks are ordinary with hearts trumps");
    assert_eq!(effective_suit(card("JC"), S), Some(Suit::Spades));
}

#[test]
fn h5_heart_lead_with_only_left_bower_must_play_it() {
    let trick = Trick::from_plays(Seat::WEST, &plays("5H"), H);
    assert_eq!(legal_cards("JD 9D 8S AC", &trick, H), cards("JD"));
}

#[test]
fn h5_diamond_lead_left_bower_does_not_follow() {
    let trick = Trick::from_plays(Seat::WEST, &plays("KD"), H);
    assert_eq!(legal_cards("JD 9D 8S", &trick, H), cards("9D"), "must play the 9♦, not J♦");
}

#[test]
fn h5_only_diamond_is_left_bower_means_void() {
    let trick = Trick::from_plays(Seat::WEST, &plays("KD"), H);
    assert_eq!(legal_cards("JD 8S AC", &trick, H), cards("JD 8S AC"), "void in diamonds: any card");
    // Trumping with the J♦ wins the diamond trick.
    let trick = Trick::from_plays(Seat::WEST, &plays("KD AD JD"), H);
    assert_eq!(trick.winner(H), Some(Seat::EAST));
}

#[test]
fn h5_leading_left_bower_is_a_trump_lead() {
    let trick = Trick::from_plays(Seat::WEST, &plays("JD"), H);
    assert_eq!(trick.led_suit(), Some(Suit::Hearts));
    assert_eq!(legal_cards("4H AD KD", &trick, H), cards("4H"));
    assert_eq!(legal_cards("AD KD 5S", &trick, H), cards("AD KD 5S"), "no trumps: any card");
}

#[test]
fn h5_joker_must_follow_trump_lead_when_only_trump() {
    let trick = Trick::from_plays(Seat::WEST, &plays("9H"), H);
    assert_eq!(legal_cards("Joker AS KC", &trick, H), cards("Joker"));
}

#[test]
fn h6_side_suit_loses_its_jack() {
    let diamonds = CardSet::pack().iter().filter(|&c| effective_suit(c, H) == Some(Suit::Diamonds)).count();
    assert_eq!(diamonds, 10, "A K Q 10 9 8 7 6 5 4 of diamonds");
}

#[test]
fn h7_no_trumps_jacks_are_ordinary() {
    assert_eq!(effective_suit(card("JD"), NT), Some(Suit::Diamonds));
    let trick = Trick::from_plays(Seat::WEST, &plays("KD"), NT);
    assert_eq!(legal_cards("JD 9H", &trick, NT), cards("JD"));
    let trick = Trick::from_plays(Seat::WEST, &plays("10D JD QD 9D"), NT);
    assert_eq!(trick.winner(NT), Some(Seat::EAST), "Q beats J beats 10");
}

// ---- Bidding --------------------------------------------------------------------------------

#[test]
fn h8_once_around_from_dealers_left() {
    let mut game = Game::with_dealer(Rules::default(), 1, Seat::EAST);
    let mut order = Vec::new();
    while game.phase() == Phase::Bidding && order.len() < 4 {
        order.push(game.to_act().unwrap());
        game.apply(Action::Call(Call::Pass)).unwrap();
    }
    assert_eq!(order, vec![Seat::SOUTH, Seat::WEST, Seat::NORTH, Seat::EAST], "dealer calls last");
}

#[test]
fn h10_mentions_must_outrank_earlier_mentions() {
    let rules = Rules::default();
    let mut auction = Auction::new(Seat::EAST, false);
    assert!(auction.apply(Call::Mention(Strain::Diamonds), &rules));
    let legal = auction.legal_calls(&rules);
    for strain in [Strain::Spades, Strain::Clubs, Strain::Diamonds] {
        assert!(!legal.contains(&Call::Mention(strain)), "{strain} can't be mentioned after ♦");
    }
    assert!(legal.contains(&Call::Mention(Strain::Hearts)));
    assert!(legal.contains(&Call::Mention(Strain::NoTrumps)));
    // A 6 of any suit, or a real bid, is still allowed after a mention.
    for suit in Suit::ALL {
        assert!(legal.contains(&Call::Six(suit)));
    }
    assert!(legal.contains(&bid(7, Strain::Spades)));
}

#[test]
fn h11_h12_after_a_six_only_pass_or_bid() {
    let rules = Rules::default();
    let mut auction = Auction::new(Seat::EAST, false);
    assert!(auction.apply(Call::Mention(Strain::Clubs), &rules));
    assert!(auction.apply(Call::Six(Suit::Spades), &rules), "6 of a lower suit than the mention is fine");
    let legal = auction.legal_calls(&rules);
    assert!(legal.iter().all(|c| matches!(c, Call::Pass | Call::Bid(_))));
    assert!(legal.contains(&bid(7, Strain::Spades)), "next real bid must be at least 7");
    assert_eq!(auction.high_bid(), None, "a 6 is not a bid");
}

#[test]
fn h12_no_signals_after_a_real_bid() {
    let rules = Rules::default();
    let mut auction = Auction::new(Seat::EAST, false);
    assert!(auction.apply(bid(7, Strain::Clubs), &rules));
    assert!(!auction.apply(Call::Mention(Strain::NoTrumps), &rules));
    assert!(!auction.apply(Call::Six(Suit::Hearts), &rules));
}

#[test]
fn h13_bids_must_beat_the_previous_bid() {
    let rules = Rules::default();
    let mut auction = Auction::new(Seat::EAST, false);
    assert!(auction.apply(bid(7, Strain::Hearts), &rules));
    assert!(!auction.apply(bid(7, Strain::Diamonds), &rules));
    assert!(!auction.apply(bid(7, Strain::Hearts), &rules));
    assert!(auction.apply(bid(7, Strain::NoTrumps), &rules));
    assert!(auction.apply(bid(8, Strain::Spades), &rules), "8♠ (240) beats 7NT (220)");
    assert_eq!(auction.high_bid(), Some((Seat::NORTH, Contract::new(8, Strain::Spades))));
}

fn pass_out(game: &mut Game) {
    for _ in 0..4 {
        game.apply(Action::Call(Call::Pass)).unwrap();
    }
}

#[test]
fn h14_two_redeals_then_dealer_must_bid() {
    let mut game = Game::with_dealer(Rules::default(), 5, Seat::NORTH);
    let first_deal = game.hand(Seat::SOUTH);
    pass_out(&mut game);
    assert_eq!((game.dealer(), game.deal_number()), (Seat::NORTH, 2), "same dealer redeals");
    assert_ne!(game.hand(Seat::SOUTH), first_deal, "fresh cards");
    pass_out(&mut game);
    assert_eq!((game.dealer(), game.deal_number()), (Seat::NORTH, 3));
    assert!(game.auction().dealer_forced());
    for _ in 0..3 {
        game.apply(Action::Call(Call::Pass)).unwrap();
    }
    assert_eq!(game.to_act(), Some(Seat::NORTH));
    let legal = game.legal_actions();
    assert!(legal.iter().all(|a| matches!(a, Action::Call(Call::Bid(_)))), "only bids: {legal:?}");
    assert!(legal.contains(&Action::Call(bid(7, Strain::Spades))));
    game.apply(Action::Call(bid(7, Strain::Spades))).unwrap();
    assert_eq!(game.phase(), Phase::Discarding);
    assert_eq!(game.history().len(), 2, "two passed-out deals recorded");
}

#[test]
fn h14_forced_dealer_may_pass_if_someone_else_bid() {
    let mut game = Game::with_dealer(Rules::default(), 5, Seat::NORTH);
    pass_out(&mut game);
    pass_out(&mut game);
    game.apply(Action::Call(bid(7, Strain::Hearts))).unwrap(); // East
    game.apply(Action::Call(Call::Pass)).unwrap();
    game.apply(Action::Call(Call::Pass)).unwrap();
    assert!(game.legal_actions().contains(&Action::Call(Call::Pass)));
}

#[test]
fn a2_signals_only_deal_is_passed_out() {
    let mut game = Game::with_dealer(Rules::default(), 6, Seat::NORTH);
    for call in [Call::Mention(Strain::Clubs), Call::Six(Suit::Hearts), Call::Pass, Call::Pass] {
        game.apply(Action::Call(call)).unwrap();
    }
    assert_eq!((game.phase(), game.deal_number()), (Phase::Bidding, 2));
}

// ---- Kitty and play -------------------------------------------------------------------------

/// Deals with dealer East, South bids `contract` and everyone else passes.
fn south_declares(seed: u64, contract: Contract) -> Game {
    let mut game = Game::with_dealer(Rules::default(), seed, Seat::EAST);
    game.apply(Action::Call(Call::Bid(contract))).unwrap();
    for _ in 0..3 {
        game.apply(Action::Call(Call::Pass)).unwrap();
    }
    game
}

#[test]
fn h16_declarer_takes_kitty_privately_and_discards_three() {
    let mut game = south_declares(11, Contract::new(7, Strain::Hearts));
    let kitty = game.kitty();
    assert_eq!(game.hand(Seat::SOUTH).len(), 13);
    assert_eq!(game.observe(Seat::SOUTH).kitty, Some(kitty));
    for seat in [Seat::WEST, Seat::NORTH, Seat::EAST] {
        let obs = game.observe(seat);
        assert_eq!((obs.kitty, obs.discards), (None, None), "{seat} must not see the kitty");
        assert_eq!(obs.hand, game.hand(seat));
    }
    // Any three cards may go, including the Joker or a bower.
    let hand = game.hand(Seat::SOUTH);
    for c in hand.iter().take(3) {
        game.apply(Action::Discard(c)).unwrap();
    }
    assert_eq!(game.hand(Seat::SOUTH).len(), 10);
    assert_eq!(game.phase(), Phase::Playing);
    assert_eq!(game.to_act(), Some(Seat::SOUTH), "H17: declarer leads");
    game.check_invariants().unwrap();
}

#[test]
fn h18a_no_trump_joker_only_when_void() {
    let trick = Trick::from_plays(Seat::WEST, &plays("9S"), NT);
    assert_eq!(legal_cards("Joker 5S AH", &trick, NT), cards("5S"), "can follow, so no Joker");
    assert_eq!(legal_cards("Joker AH KD", &trick, NT), cards("Joker AH KD"), "void: Joker optional");
}

#[test]
fn h18a_no_trump_joker_lead_names_any_suit() {
    let leads = legal_plays(cards("Joker 5S"), &Trick::new(Seat::SOUTH), NT);
    let named: Vec<Option<Suit>> = leads.iter().filter(|p| p.card.is_joker()).map(|p| p.named_suit).collect();
    assert_eq!(named, Suit::ALL.map(Some).to_vec(), "any of the four suits");
    let lead = Play { card: Card::JOKER, named_suit: Some(Suit::Diamonds) };
    let trick = Trick::from_plays(Seat::SOUTH, &[lead], NT);
    assert_eq!(trick.led_suit(), Some(Suit::Diamonds));
    assert_eq!(legal_cards("AD 5S", &trick, NT), cards("AD"), "others must follow the named suit");
    let trick = Trick::from_plays(Seat::SOUTH, &[lead, Play::card(card("AD")), Play::card(card("KD")), Play::card(card("QD"))], NT);
    assert_eq!(trick.winner(NT), Some(Seat::SOUTH), "the Joker wins");
}

#[test]
fn h18a_no_trump_joker_wins_when_played_void() {
    let trick = Trick::from_plays(Seat::WEST, &plays("AS KS Joker 5S"), NT);
    assert_eq!(trick.winner(NT), Some(Seat::EAST));
}

#[test]
fn h18b_trick_winners_in_suit_contracts() {
    let cases = [
        ("AS KS 4H QS", Seat::EAST),      // a low trump beats the led ace (order: W N E S)
        ("4H JD AH KH", Seat::NORTH),     // left bower beats ace of trumps
        ("JD JH AH KH", Seat::NORTH),     // right bower beats left bower
        ("JH Joker 5H 6H", Seat::NORTH),  // Joker beats right bower
        ("KC AC QC AS", Seat::NORTH),     // no trump played: highest of the led suit
    ];
    for (cards_played, winner) in cases {
        let trick = Trick::from_plays(Seat::WEST, &plays(cards_played), H);
        assert_eq!(trick.winner(H), Some(winner), "{cards_played}");
    }
}

// ---- Scoring and winning --------------------------------------------------------------------

#[test]
fn h19_contract_values_match_house_table() {
    let table = [
        (7, [140, 160, 180, 200, 220]),
        (8, [240, 260, 280, 300, 320]),
        (9, [340, 360, 380, 400, 420]),
        (10, [440, 460, 480, 500, 520]),
    ];
    for (level, values) in table {
        for (strain, value) in Strain::ALL.into_iter().zip(values) {
            assert_eq!(Contract::new(level, strain).value(), value, "{level}{strain}");
        }
    }
}

const NS: Team = Team::NorthSouth;

#[test]
fn h20_h21_made_and_failed_contracts() {
    let rules = Rules::default();
    let eight_hearts = Contract::new(8, Strain::Hearts);
    assert_eq!(score_hand(&rules, eight_hearts, NS, [8, 2], [0, 0]), [300, 20]);
    assert_eq!(score_hand(&rules, eight_hearts, NS, [9, 1], [0, 0]), [300, 10], "no overtricks");
    assert_eq!(score_hand(&rules, eight_hearts, NS, [7, 3], [0, 0]), [-300, 30]);
    assert_eq!(score_hand(&rules, eight_hearts, Team::EastWest, [3, 7], [0, 0]), [30, -300]);
}

#[test]
fn h22_slam_scores_250_or_more() {
    let rules = Rules::default();
    assert_eq!(score_hand(&rules, Contract::new(7, Strain::Spades), NS, [10, 0], [0, 0]), [250, 0]);
    assert_eq!(score_hand(&rules, Contract::new(8, Strain::Spades), NS, [10, 0], [0, 0]), [250, 0]);
    assert_eq!(score_hand(&rules, Contract::new(8, Strain::Clubs), NS, [10, 0], [0, 0]), [260, 0]);
}

#[test]
fn h23_defenders_can_win_and_declarers_count_first() {
    let rules = Rules::default();
    assert_eq!(game_winner(&rules, [480, 500], NS), Some(Team::EastWest), "defensive points win");
    assert_eq!(game_winner(&rules, [520, 510], NS), Some(NS), "both over 500: declarers first");
    assert_eq!(game_winner(&rules, [499, 450], NS), None);
}

#[test]
fn h24_no_losing_floor() {
    assert_eq!(game_winner(&Rules::default(), [-900, 300], NS), None);
}

#[test]
fn defensive_cap_variant() {
    let rules = Rules { defenders_can_win: false, ..Rules::default() };
    let seven = Contract::new(7, Strain::Spades);
    assert_eq!(score_hand(&rules, seven, NS, [5, 5], [0, 470]), [-140, 20], "stops at 490");
    assert_eq!(score_hand(&rules, seven, NS, [5, 5], [0, -100]), [-140, 50]);
}

// ---- Whole games ----------------------------------------------------------------------------

/// Random play overbids and, with no losing floor (H24), rarely finishes a game, so this caps the length.
/// Finished games are checked by the bot simulations in `fh-lab`.
#[test]
fn random_play_stays_legal() {
    for seed in 0..300 {
        let mut game = Game::new(Rules::default(), seed);
        let mut rng = Rng::new(seed + 1_000_000);
        for _ in 0..3_000 {
            if game.phase() == Phase::GameOver {
                let winner = game.winner().unwrap();
                assert!(game.scores()[winner.index()] >= 500);
                assert!(game.apply(Action::Call(Call::Pass)).is_err());
                break;
            }
            let legal = game.legal_actions();
            assert!(!legal.is_empty(), "seed {seed}: no legal action in {:?}", game.phase());
            assert!(game.to_act().is_some());
            let action = legal[rng.below(legal.len())];
            game.apply(action).unwrap();
            game.check_invariants().unwrap_or_else(|e| panic!("seed {seed}: {e}"));
        }
    }
}

#[test]
fn illegal_actions_are_rejected_without_changing_state() {
    let mut game = Game::with_dealer(Rules::default(), 3, Seat::EAST);
    let before = game.hand(Seat::SOUTH);
    assert!(game.apply(Action::Discard(before.iter().next().unwrap())).is_err());
    assert!(game.apply(Action::Play(Play::card(Card::new(Suit::Hearts, ACE)))).is_err());
    assert_eq!(game.to_act(), Some(Seat::SOUTH));
    assert_eq!(game.hand(Seat::SOUTH), before);
}
