//! Tricks: following suit (H18), the Joker in no trumps (H18a) and who wins (H18b).

use crate::cards::{Card, CardSet, Suit};
use crate::rules::{effective_suit, is_trump, trump_power};
use crate::seat::Seat;
use std::fmt;

/// Playing a card. `named_suit` is set only when leading the Joker in no trumps (H18a).
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Play {
    pub card: Card,
    pub named_suit: Option<Suit>,
}

impl Play {
    pub fn card(card: Card) -> Play {
        Play { card, named_suit: None }
    }
}

impl fmt::Display for Play {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.named_suit {
            Some(suit) => f.pad(&format!("{} naming {}", self.card, suit)),
            None => f.pad(&self.card.to_string()),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Trick {
    leader: Seat,
    cards: Vec<Card>,
    led_suit: Option<Suit>,
}

impl Trick {
    pub fn new(leader: Seat) -> Trick {
        Trick { leader, cards: Vec::with_capacity(4), led_suit: None }
    }

    /// A trick with these plays already made, e.g. to set up a position. Does not check legality.
    pub fn from_plays(leader: Seat, plays: &[Play], trump: Option<Suit>) -> Trick {
        let mut trick = Trick::new(leader);
        for &play in plays.iter().take(4) {
            trick.push(play, trump);
        }
        trick
    }

    pub fn leader(&self) -> Seat {
        self.leader
    }

    /// Cards in the order played.
    pub fn cards(&self) -> &[Card] {
        &self.cards
    }

    pub fn plays(&self) -> impl Iterator<Item = (Seat, Card)> + '_ {
        self.cards.iter().enumerate().map(|(i, &card)| (Seat::new(self.leader.index() + i), card))
    }

    /// The suit others must follow: the lead's effective suit, or the suit named with a Joker lead.
    pub fn led_suit(&self) -> Option<Suit> {
        self.led_suit
    }

    pub fn is_empty(&self) -> bool {
        self.cards.is_empty()
    }

    pub fn is_complete(&self) -> bool {
        self.cards.len() == 4
    }

    pub fn to_play(&self) -> Option<Seat> {
        (!self.is_complete()).then(|| Seat::new(self.leader.index() + self.cards.len()))
    }

    /// Adds a play without checking legality; [`crate::Game`] validates first.
    pub(crate) fn push(&mut self, play: Play, trump: Option<Suit>) {
        if self.cards.is_empty() {
            self.led_suit = play.named_suit.or_else(|| effective_suit(play.card, trump));
        }
        self.cards.push(play.card);
    }

    /// The seat currently winning (or that won) the trick; `None` if no card has been played.
    pub fn winner(&self, trump: Option<Suit>) -> Option<Seat> {
        let led = self.led_suit?;
        self.plays().max_by_key(|&(_, card)| trick_strength(card, led, trump)).map(|(seat, _)| seat)
    }

    /// The card currently winning the trick.
    pub fn winning_card(&self, trump: Option<Suit>) -> Option<Card> {
        let led = self.led_suit?;
        self.cards.iter().copied().max_by_key(|&card| trick_strength(card, led, trump))
    }
}

/// How strongly `card` competes for a trick led in `led` (H18b). Cards that can't win score 0.
pub fn trick_strength(card: Card, led: Suit, trump: Option<Suit>) -> u16 {
    match trump {
        Some(t) if is_trump(card, trump) => 200 + trump_power(card, t) as u16,
        Some(_) => {
            if card.suit() == Some(led) {
                100 + card.rank() as u16
            } else {
                0
            }
        }
        None if card.is_joker() => 1000,
        None => {
            if card.suit() == Some(led) {
                100 + card.rank() as u16
            } else {
                0
            }
        }
    }
}

/// Whether `card` follows a trick led in `led`. In no trumps the Joker follows nothing (H18a).
pub fn follows_suit(card: Card, led: Suit, trump: Option<Suit>) -> bool {
    effective_suit(card, trump) == Some(led)
}

/// The legal plays for a player holding `hand` with `trick` in progress.
pub fn legal_plays(hand: CardSet, trick: &Trick, trump: Option<Suit>) -> Vec<Play> {
    let mut plays = Vec::with_capacity(hand.len() + 3);
    match trick.led_suit() {
        None => {
            for card in hand.iter() {
                if card.is_joker() && trump.is_none() {
                    plays.extend(Suit::ALL.map(|suit| Play { card, named_suit: Some(suit) }));
                } else {
                    plays.push(Play::card(card));
                }
            }
        }
        Some(led) => {
            plays.extend(hand.iter().filter(|&c| follows_suit(c, led, trump)).map(Play::card));
            if plays.is_empty() {
                plays.extend(hand.iter().map(Play::card));
            }
        }
    }
    plays
}
