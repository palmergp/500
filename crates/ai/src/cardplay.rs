//! Card-play helpers shared by bots: card ordering, "boss" cards and how cheap a card is to give up.

use fh_core::rules::{is_trump, trump_power};
use fh_core::trick::trick_strength;
use fh_core::{effective_suit, Card, CardSet, Suit};

/// The cards of `suit` (as an effective suit under `trump`), strongest first.
pub fn suit_order(suit: Suit, trump: Option<Suit>) -> Vec<Card> {
    let mut cards: Vec<Card> =
        CardSet::pack().iter().filter(|&c| effective_suit(c, trump) == Some(suit)).collect();
    cards.sort_by_key(|&c| std::cmp::Reverse(trick_strength(c, suit, trump)));
    cards
}

/// Whether every card that outranks `card` in its suit is already accounted for in `seen`
/// (played, or in the bot's own hand). The Joker in no trumps is always boss.
pub fn is_boss(card: Card, seen: CardSet, trump: Option<Suit>) -> bool {
    let Some(suit) = effective_suit(card, trump) else {
        return true;
    };
    suit_order(suit, trump).into_iter().take_while(|&c| c != card).all(|c| seen.contains(c))
}

/// How costly a card is to give up: low side cards are cheapest, trumps and the Joker dearest.
pub fn card_cost(card: Card, trump: Option<Suit>) -> u16 {
    match trump {
        Some(t) if is_trump(card, trump) => 100 + trump_power(card, t) as u16,
        None if card.is_joker() => 200,
        _ => card.rank() as u16,
    }
}

/// The cheapest card of `cards` to give up; ties go to the card from the shorter suit in `hand`.
pub fn cheapest(cards: &[Card], hand: CardSet, trump: Option<Suit>) -> Card {
    let suit_len = |c: Card| hand.iter().filter(|&h| effective_suit(h, trump) == effective_suit(c, trump)).count();
    *cards
        .iter()
        .min_by_key(|&&c| (card_cost(c, trump), suit_len(c), c.id()))
        .expect("cheapest of no cards")
}

/// Trumps in `cards` under `trump`.
pub fn trumps(cards: CardSet, trump: Option<Suit>) -> CardSet {
    cards.iter().filter(|&c| is_trump(c, trump)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn boss_accounts_for_bowers() {
        let hearts = Some(Suit::Hearts);
        let ace = Card::parse("AH").unwrap();
        let seen = CardSet::parse("Joker JH").unwrap();
        assert!(!is_boss(ace, seen, hearts), "the left bower J♦ is still out");
        assert!(is_boss(ace, seen.union(CardSet::parse("JD").unwrap()), hearts));
        // In no trumps the jacks are ordinary, so A♦ is simply the top diamond.
        assert!(is_boss(Card::parse("AD").unwrap(), CardSet::EMPTY, None));
    }
}
