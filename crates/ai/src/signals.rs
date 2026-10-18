//! The discard signal, a partnership convention: the first time a player can't follow suit and
//! throws a non-trump, they throw from their strongest side suit. It tells partner where to lead
//! when they run out of winners. Everyone at the table sees it, so opponents learn it too.

use fh_core::cards::{ACE, JACK, KING, QUEEN};
use fh_core::rules::is_trump;
use fh_core::{effective_suit, Card, CardSet, Observation, Seat, Suit};

/// How strong a side-suit holding is, for signalling: honours count most, length a little.
pub fn suit_strength(cards: &[Card]) -> f32 {
    let honours: f32 = cards
        .iter()
        .map(|c| match c.rank() {
            ACE => 4.0,
            KING => 3.0,
            QUEEN => 2.0,
            JACK => 1.0,
            _ => 0.0,
        })
        .sum();
    honours + 0.5 * cards.len() as f32
}

fn side_cards(hand: CardSet, suit: Suit, trump: Option<Suit>) -> Vec<Card> {
    hand.iter().filter(|&c| !is_trump(c, trump) && !c.is_joker() && effective_suit(c, trump) == Some(suit)).collect()
}

/// The suit to signal with: the strongest side suit that has a card to spare (two or more cards,
/// so the signal doesn't throw away a lone winner).
pub fn strongest_side_suit(hand: CardSet, trump: Option<Suit>, exclude: Option<Suit>) -> Option<Suit> {
    Suit::ALL
        .into_iter()
        .filter(|&s| Some(s) != trump && Some(s) != exclude)
        .map(|s| (s, side_cards(hand, s, trump)))
        .filter(|(_, cards)| cards.len() >= 2)
        .max_by(|a, b| suit_strength(&a.1).total_cmp(&suit_strength(&b.1)))
        .map(|(s, _)| s)
}

/// The card that makes the signal: the lowest card of the strongest side suit.
pub fn signal_card(hand: CardSet, trump: Option<Suit>, led: Suit) -> Option<Card> {
    let suit = strongest_side_suit(hand, trump, Some(led))?;
    side_cards(hand, suit, trump).into_iter().min_by_key(|c| c.rank())
}

/// Each seat's discard signal so far this hand: the suit of its first non-trump discard.
pub fn discard_signals(obs: &Observation) -> [Option<Suit>; 4] {
    let trump = obs.trump();
    let mut signals = [None; 4];
    let mut showed_out = [false; 4];
    for trick in obs.tricks.iter().chain(obs.current_trick.iter()) {
        let Some(led) = trick.led_suit() else { continue };
        for (seat, card) in trick.plays() {
            let s = seat.index();
            if seat == trick.leader() || effective_suit(card, trump) == Some(led) || showed_out[s] {
                continue;
            }
            showed_out[s] = true;
            if !is_trump(card, trump) && !card.is_joker() {
                signals[s] = effective_suit(card, trump);
            }
        }
    }
    signals
}

/// Whether `seat` is about to show out of a suit for the first time this hand.
pub fn first_time_void(obs: &Observation, seat: Seat) -> bool {
    let trump = obs.trump();
    !obs.tricks.iter().chain(obs.current_trick.iter()).any(|trick| {
        trick.led_suit().is_some_and(|led| {
            trick.plays().any(|(s, c)| s == seat && s != trick.leader() && effective_suit(c, trump) != Some(led))
        })
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn signals_with_the_low_card_of_the_strongest_side_suit() {
        let hand = CardSet::parse("AH KH 5H 9C 7C 6S JD 4D").unwrap();
        // Spades trumps; a diamond is led and we have diamonds, but ask what we'd signal if void in clubs.
        assert_eq!(strongest_side_suit(hand, Some(Suit::Spades), Some(Suit::Clubs)), Some(Suit::Hearts));
        assert_eq!(signal_card(hand, Some(Suit::Spades), Suit::Clubs), Some(Card::parse("5H").unwrap()));
        // A lone ace is not thrown away to signal.
        let hand = CardSet::parse("AH 9C 8C 6S").unwrap();
        assert_eq!(strongest_side_suit(hand, Some(Suit::Spades), None), Some(Suit::Clubs));
    }
}
