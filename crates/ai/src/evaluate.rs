//! Rough hand evaluation: how many tricks a hand is worth on its own in each strain.
//!
//! These weights are hand-tuned starting points; the strategy lab will replace them with
//! values measured from simulated deals.

use fh_core::cards::{ACE, JACK, KING, QUEEN};
use fh_core::rules::is_trump;
use fh_core::{effective_suit, Card, CardSet, Strain, Suit};

pub fn hand_tricks(hand: CardSet, strain: Strain) -> f32 {
    match strain.trump() {
        Some(trump) => suit_tricks(hand, trump),
        None => no_trump_tricks(hand),
    }
}

fn suit_tricks(hand: CardSet, trump: Suit) -> f32 {
    let has = |c: Card| hand.contains(c);
    let n = hand.iter().filter(|&c| is_trump(c, Some(trump))).count() as f32;
    let right = Card::new(trump, JACK);
    let left = Card::new(trump.same_color(), JACK);
    let ace = Card::new(trump, ACE);
    let king = Card::new(trump, KING);

    let mut tricks = 0.0;
    if has(Card::JOKER) {
        tricks += 1.0;
    }
    if has(right) {
        tricks += 1.0;
    }
    if has(left) {
        tricks += if has(Card::JOKER) || has(right) {
            0.9
        } else if n >= 2.0 {
            0.7
        } else {
            0.4
        };
    }
    if has(ace) {
        tricks += if n >= 3.0 { 0.8 } else { 0.5 };
    }
    if has(king) && n >= 3.0 {
        tricks += 0.4;
    }
    // Small trumps win by ruffing or as long cards, more so in a long suit.
    let honours = [Card::JOKER, right, left, ace, king].into_iter().filter(|&c| has(c)).count() as f32;
    tricks += (n - honours).max(0.0) * 0.35 + (n - 4.0).max(0.0) * 0.4;

    for side in Suit::ALL.into_iter().filter(|&s| s != trump) {
        let len = hand.iter().filter(|&c| effective_suit(c, Some(trump)) == Some(side)).count();
        let side_ace = has(Card::new(side, ACE));
        if side_ace {
            tricks += 0.9;
        }
        if has(Card::new(side, KING)) {
            tricks += if side_ace {
                0.7
            } else if len >= 2 {
                0.4
            } else {
                0.1
            };
        }
        if len == 0 && n >= 3.0 {
            tricks += 0.6;
        } else if len == 1 && n >= 4.0 {
            tricks += 0.3;
        }
    }
    tricks
}

fn no_trump_tricks(hand: CardSet) -> f32 {
    let has = |c: Card| hand.contains(c);
    let mut tricks = if has(Card::JOKER) { 1.0 } else { 0.0 };
    for suit in Suit::ALL {
        let len = hand.of_printed_suit(suit).len();
        let (a, k, q) = (has(Card::new(suit, ACE)), has(Card::new(suit, KING)), has(Card::new(suit, QUEEN)));
        if a {
            tricks += 1.0;
        }
        if k {
            tricks += if a {
                0.9
            } else if len >= 2 {
                0.5
            } else {
                0.0
            };
        }
        if q && a && k {
            tricks += 0.6;
        }
        if a && len >= 4 {
            tricks += (len - 3) as f32 * 0.5;
        }
        let stopped = a || (k && len >= 2) || (q && len >= 3);
        if !stopped {
            tricks -= 0.7; // the defenders can run this suit
        }
    }
    tricks.max(0.0)
}

/// Whether the hand holds a bower in this suit (either jack of its color).
pub fn has_bower(hand: CardSet, suit: Suit) -> bool {
    hand.contains(Card::new(suit, JACK)) || hand.contains(Card::new(suit.same_color(), JACK))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strong_trump_hand_rates_highly() {
        let hand = CardSet::parse("Joker JH JD AH KH 9H AS AC 5D 6D").unwrap();
        let hearts = hand_tricks(hand, Strain::Hearts);
        assert!(hearts > 7.0, "{hearts}");
        assert!(hearts > hand_tricks(hand, Strain::Spades));
    }

    #[test]
    fn weak_hand_rates_low() {
        let hand = CardSet::parse("5S 7S 9C 6C 4D 8D 10D 5H 6H 9H").unwrap();
        for strain in Strain::ALL {
            assert!(hand_tricks(hand, strain) < 3.0, "{strain}");
        }
    }
}
