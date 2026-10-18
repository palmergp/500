//! Strains, contracts, card ranking under a contract, and the configurable rule settings.

use crate::cards::{Card, Suit, JACK};
use std::fmt;

/// A trump suit or no trumps, in bid order (H9): ♠ < ♣ < ♦ < ♥ < NT.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Strain {
    Spades,
    Clubs,
    Diamonds,
    Hearts,
    NoTrumps,
}

impl Strain {
    pub const ALL: [Strain; 5] = [
        Strain::Spades,
        Strain::Clubs,
        Strain::Diamonds,
        Strain::Hearts,
        Strain::NoTrumps,
    ];

    pub fn index(self) -> usize {
        self as usize
    }

    pub fn from_suit(suit: Suit) -> Strain {
        Strain::ALL[suit.index()]
    }

    /// The trump suit, or `None` for no trumps.
    pub fn trump(self) -> Option<Suit> {
        Suit::ALL.get(self.index()).copied()
    }
}

impl fmt::Display for Strain {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.trump() {
            Some(suit) => f.pad(&suit.symbol().to_string()),
            None => f.pad("NT"),
        }
    }
}

pub const MIN_BID_LEVEL: u8 = 7;
pub const MAX_BID_LEVEL: u8 = 10;

/// A real bid (H13). Ordering by (level, strain) is the same as ordering by value.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Contract {
    pub level: u8,
    pub strain: Strain,
}

impl Contract {
    pub fn new(level: u8, strain: Strain) -> Contract {
        assert!(
            (MIN_BID_LEVEL..=MAX_BID_LEVEL).contains(&level),
            "bid level {level} is outside 7..=10"
        );
        Contract { level, strain }
    }

    /// The contract's value (H19): 7♠ = 140, rising 20 per strain and 100 per level.
    pub fn value(self) -> i32 {
        40 + 20 * self.strain.index() as i32 + 100 * (self.level as i32 - 6)
    }

    pub fn trump(self) -> Option<Suit> {
        self.strain.trump()
    }
}

impl fmt::Display for Contract {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.pad(&format!("{}{}", self.level, self.strain))
    }
}

/// The suit a card belongs to for following suit and winning tricks (H5, H7).
///
/// With a trump suit, the Joker and the left bower belong to it. In no trumps the Joker has no suit.
pub fn effective_suit(card: Card, trump: Option<Suit>) -> Option<Suit> {
    match (card.suit(), trump) {
        (None, trump) => trump,
        (Some(suit), Some(trump)) if card.rank() == JACK && suit == trump.same_color() => Some(trump),
        (Some(suit), _) => Some(suit),
    }
}

pub fn is_trump(card: Card, trump: Option<Suit>) -> bool {
    trump.is_some() && effective_suit(card, trump) == trump
}

/// Rank within the trump suit (H4): Joker 20, right bower 19, left bower 18, otherwise the card's rank.
/// Only meaningful for cards where [`is_trump`] holds.
pub fn trump_power(card: Card, trump: Suit) -> u8 {
    if card.is_joker() {
        20
    } else if card.rank() == JACK {
        if card.suit() == Some(trump) {
            19
        } else {
            18
        }
    } else {
        card.rank()
    }
}

/// Rule settings. [`Rules::default`] is the house ruleset in `docs/rules/house-rules.md`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Rules {
    /// Score that wins the game (H23).
    pub target_score: i32,
    /// Whether mentions are allowed (H10).
    pub mentions: bool,
    /// Whether "6 of a suit" Joker signals are allowed (H11).
    pub joker_six: bool,
    /// Redeals allowed after a deal with no bid before the dealer is forced to bid (H14).
    pub redeals_before_forced_bid: u8,
    /// Score for taking all ten tricks when the contract is worth less (H22).
    pub slam_score: i32,
    /// Points defenders earn per trick (H21).
    pub defender_points_per_trick: i32,
    /// Whether defensive points can take a team to the target (H23). If not, they stop 10 short.
    pub defenders_can_win: bool,
}

impl Default for Rules {
    fn default() -> Self {
        Rules {
            target_score: 500,
            mentions: true,
            joker_six: true,
            redeals_before_forced_bid: 2,
            slam_score: 250,
            defender_points_per_trick: 10,
            defenders_can_win: true,
        }
    }
}
