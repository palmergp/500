//! Regression positions for the expert bot: plays a competent human expects, several taken from
//! common complaints about other 500 apps (docs/research/03-ai-and-trainer-landscape.md §4.2).
//!
//! Each position is built as one seat's observation; the hidden cards are whatever remains, so
//! the bot must find the play without seeing the other hands.

use fh_ai::{Bot, SearchBot, SearchConfig};
use fh_core::rules::is_trump;
use fh_core::{legal_plays, Action, Auction, Call, Card, CardSet, Contract, Observation, Phase, Play, Rules, Seat, Strain, Trick};

fn cards(text: &str) -> CardSet {
    CardSet::parse(text).unwrap_or_else(|| panic!("bad cards {text}"))
}

struct Spot {
    seat: Seat,
    declarer: Seat,
    contract: Contract,
    hand: &'static str,
    leader: Seat,
    trick: &'static str,
    /// For the declarer: the kitty and the discards.
    kitty: Option<(&'static str, &'static str)>,
}

impl Spot {
    fn observation(&self) -> Observation {
        let rules = Rules::default();
        let dealer = Seat::new(self.declarer.index() + 3); // the declarer speaks first
        let mut auction = Auction::new(dealer, false);
        assert!(auction.apply(Call::Bid(self.contract), &rules));
        for _ in 0..3 {
            assert!(auction.apply(Call::Pass, &rules));
        }
        let trump = self.contract.trump();
        let plays: Vec<Play> = self.trick.split_whitespace().map(|w| Play::card(Card::parse(w).unwrap())).collect();
        let trick = Trick::from_plays(self.leader, &plays, trump);
        assert_eq!(trick.to_play(), Some(self.seat), "the spot must be this seat's turn");
        Observation {
            seat: self.seat,
            phase: Phase::Playing,
            to_act: Some(self.seat),
            rules,
            scores: [0, 0],
            dealer,
            deal_number: 1,
            hand: cards(self.hand),
            auction,
            contract: Some((self.declarer, self.contract)),
            kitty: self.kitty.map(|(k, _)| cards(k)),
            discards: self.kitty.map(|(_, d)| cards(d)),
            tricks: Vec::new(),
            current_trick: Some(trick),
            tricks_won: [0, 0],
        }
    }

    /// The card the expert plays here.
    fn choose(&self) -> Card {
        let obs = self.observation();
        let trick = obs.current_trick.clone().unwrap();
        let legal: Vec<Action> = legal_plays(obs.hand, &trick, obs.trump()).into_iter().map(Action::Play).collect();
        let config = SearchConfig { play_samples: 12, min_play_samples: 12, ..SearchConfig::COUNTER };
        match SearchBot::new("test", config, 7).choose(&obs, &legal) {
            Action::Play(play) => play.card,
            other => panic!("expected a play, got {other}"),
        }
    }
}

#[test]
fn does_not_overtake_partners_winner_when_last() {
    // South declares 7♠ and leads a diamond; West (East's partner) wins with the A♦.
    // East, last to play, should keep the K♦ and follow with the 6♦.
    let spot = Spot {
        seat: Seat::EAST,
        declarer: Seat::SOUTH,
        contract: Contract::new(7, Strain::Spades),
        hand: "KD 6D 9C 8C 7C 10H 9H 8H 6S 7S",
        leader: Seat::SOUTH,
        trick: "5D AD 4D",
        kitty: None,
    };
    assert_eq!(spot.choose(), Card::parse("6D").unwrap());
}

#[test]
fn does_not_trump_partners_winner() {
    // Same trick, but East has no diamonds: it should discard, not trump partner's ace.
    let spot = Spot {
        seat: Seat::EAST,
        declarer: Seat::SOUTH,
        contract: Contract::new(7, Strain::Spades),
        hand: "9C 8C 7C 6C 10H 9H 8H 6S 7S 8S",
        leader: Seat::SOUTH,
        trick: "5D AD 4D",
        kitty: None,
    };
    let card = spot.choose();
    assert!(!is_trump(card, Some(fh_core::Suit::Spades)), "trumped partner's ace with {card}");
}

#[test]
fn keeps_the_joker_when_partner_is_winning_in_no_trumps() {
    // 7NT by South. West wins the trick with the A♦; East is void in diamonds and holds the Joker.
    let spot = Spot {
        seat: Seat::EAST,
        declarer: Seat::SOUTH,
        contract: Contract::new(7, Strain::NoTrumps),
        hand: "Joker 9C 8C 7C 6C 10H 9H 8H 6S 7S",
        leader: Seat::SOUTH,
        trick: "5D AD 4D",
        kitty: None,
    };
    assert_ne!(spot.choose(), Card::JOKER);
}

#[test]
fn declarer_draws_trumps_first() {
    // South declares 9♥ holding the six top trumps and four side winners. Cashing a side ace
    // first risks a ruff; drawing trumps first can't lose.
    let spot = Spot {
        seat: Seat::SOUTH,
        declarer: Seat::SOUTH,
        contract: Contract::new(9, Strain::Hearts),
        hand: "Joker JH JD AH KH QH AS KS AC KC",
        leader: Seat::SOUTH,
        trick: "",
        kitty: Some(("KH QH KC", "4D 5D 6D")),
    };
    let card = spot.choose();
    assert!(is_trump(card, Some(fh_core::Suit::Hearts)), "led {card} instead of a trump");
}

#[test]
fn follows_suit_with_the_left_bower_only_when_it_is_the_only_trump() {
    // Hearts trumps, a heart is led and East's only trump is the J♦: the play is forced.
    let spot = Spot {
        seat: Seat::EAST,
        declarer: Seat::SOUTH,
        contract: Contract::new(7, Strain::Hearts),
        hand: "JD 9D 8D 9C 8C 7C 6S 7S 8S 9S",
        leader: Seat::SOUTH,
        trick: "AH 5H 6H",
        kitty: None,
    };
    assert_eq!(spot.choose(), Card::parse("JD").unwrap());
}
