//! Cards, suits and card sets (a 64-bit bitboard).

use std::fmt;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Suit {
    Spades,
    Clubs,
    Diamonds,
    Hearts,
}

impl Suit {
    /// In bid order: ♠ < ♣ < ♦ < ♥.
    pub const ALL: [Suit; 4] = [Suit::Spades, Suit::Clubs, Suit::Diamonds, Suit::Hearts];

    pub fn index(self) -> usize {
        self as usize
    }

    pub fn is_red(self) -> bool {
        matches!(self, Suit::Diamonds | Suit::Hearts)
    }

    /// The other suit of the same color: home of the left bower when `self` is trumps.
    pub fn same_color(self) -> Suit {
        match self {
            Suit::Spades => Suit::Clubs,
            Suit::Clubs => Suit::Spades,
            Suit::Diamonds => Suit::Hearts,
            Suit::Hearts => Suit::Diamonds,
        }
    }

    /// Lowest rank in the 43-card pack (H2): red suits go down to 4, black suits to 5.
    pub fn lowest_rank(self) -> u8 {
        if self.is_red() {
            4
        } else {
            5
        }
    }

    pub fn symbol(self) -> char {
        match self {
            Suit::Spades => '♠',
            Suit::Clubs => '♣',
            Suit::Diamonds => '♦',
            Suit::Hearts => '♥',
        }
    }

    pub fn letter(self) -> char {
        match self {
            Suit::Spades => 'S',
            Suit::Clubs => 'C',
            Suit::Diamonds => 'D',
            Suit::Hearts => 'H',
        }
    }

    pub fn parse(c: char) -> Option<Suit> {
        match c {
            'S' | 's' | '♠' => Some(Suit::Spades),
            'C' | 'c' | '♣' => Some(Suit::Clubs),
            'D' | 'd' | '♦' => Some(Suit::Diamonds),
            'H' | 'h' | '♥' => Some(Suit::Hearts),
            _ => None,
        }
    }
}

impl fmt::Display for Suit {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.symbol())
    }
}

pub const JACK: u8 = 11;
pub const QUEEN: u8 = 12;
pub const KING: u8 = 13;
pub const ACE: u8 = 14;

/// A card, stored as `suit * 16 + rank` (rank 4..=14, ace high), or 63 for the Joker.
#[derive(Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Card(u8);

impl Card {
    pub const JOKER: Card = Card(63);

    pub const fn new(suit: Suit, rank: u8) -> Card {
        Card(suit as u8 * 16 + rank)
    }

    /// The card with this id, if it is in the 43-card pack.
    pub fn from_id(id: u8) -> Option<Card> {
        let card = Card(id);
        card.in_pack().then_some(card)
    }

    pub fn id(self) -> u8 {
        self.0
    }

    pub fn is_joker(self) -> bool {
        self == Card::JOKER
    }

    /// The printed suit; `None` for the Joker. For play, use [`crate::effective_suit`] instead.
    pub fn suit(self) -> Option<Suit> {
        (!self.is_joker()).then(|| Suit::ALL[(self.0 / 16) as usize])
    }

    /// 4..=14 (jack 11, queen 12, king 13, ace 14); 0 for the Joker.
    pub fn rank(self) -> u8 {
        if self.is_joker() {
            0
        } else {
            self.0 % 16
        }
    }

    pub fn bit(self) -> u64 {
        1u64 << self.0
    }

    fn in_pack(self) -> bool {
        if self.is_joker() {
            return true;
        }
        let suit_index = self.0 / 16;
        let rank = self.0 % 16;
        suit_index < 4 && rank >= Suit::ALL[suit_index as usize].lowest_rank() && rank <= ACE
    }

    /// Parses "A♥", "AH", "10d", "Td", "j♦", "Joker" or "JK".
    pub fn parse(text: &str) -> Option<Card> {
        let text = text.trim();
        if text.eq_ignore_ascii_case("joker") || text.eq_ignore_ascii_case("jk") {
            return Some(Card::JOKER);
        }
        let suit_char = text.chars().last()?;
        let suit = Suit::parse(suit_char)?;
        let rank_text = &text[..text.len() - suit_char.len_utf8()];
        let rank = match rank_text.to_ascii_uppercase().as_str() {
            "A" => ACE,
            "K" => KING,
            "Q" => QUEEN,
            "J" => JACK,
            "T" | "10" => 10,
            digits => digits.parse().ok()?,
        };
        Card::from_id(Card::new(suit, rank).0)
    }

    pub fn rank_label(self) -> &'static str {
        match self.rank() {
            0 => "Joker",
            4 => "4",
            5 => "5",
            6 => "6",
            7 => "7",
            8 => "8",
            9 => "9",
            10 => "10",
            JACK => "J",
            QUEEN => "Q",
            KING => "K",
            _ => "A",
        }
    }
}

impl fmt::Display for Card {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.suit() {
            None => f.pad("Joker"),
            Some(suit) => f.pad(&format!("{}{}", self.rank_label(), suit.symbol())),
        }
    }
}

impl fmt::Debug for Card {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

/// A set of cards as a bitboard. Iterates in id order: by printed suit (♠ ♣ ♦ ♥), low to high, Joker last.
#[derive(Copy, Clone, Default, PartialEq, Eq, Hash)]
pub struct CardSet(u64);

impl CardSet {
    pub const EMPTY: CardSet = CardSet(0);

    /// The 43-card pack (H2).
    pub fn pack() -> CardSet {
        let mut set = CardSet::EMPTY;
        for suit in Suit::ALL {
            for rank in suit.lowest_rank()..=ACE {
                set.insert(Card::new(suit, rank));
            }
        }
        set.insert(Card::JOKER);
        set
    }

    pub fn from_bits(bits: u64) -> CardSet {
        CardSet(bits)
    }

    pub fn bits(self) -> u64 {
        self.0
    }

    pub fn contains(self, card: Card) -> bool {
        self.0 & card.bit() != 0
    }

    pub fn insert(&mut self, card: Card) {
        self.0 |= card.bit();
    }

    /// Removes `card`, returning whether it was present.
    pub fn remove(&mut self, card: Card) -> bool {
        let present = self.contains(card);
        self.0 &= !card.bit();
        present
    }

    pub fn len(self) -> usize {
        self.0.count_ones() as usize
    }

    pub fn is_empty(self) -> bool {
        self.0 == 0
    }

    pub fn union(self, other: CardSet) -> CardSet {
        CardSet(self.0 | other.0)
    }

    pub fn intersect(self, other: CardSet) -> CardSet {
        CardSet(self.0 & other.0)
    }

    pub fn minus(self, other: CardSet) -> CardSet {
        CardSet(self.0 & !other.0)
    }

    /// Cards of this printed suit (ignores bowers changing suit; see [`crate::effective_suit`]).
    pub fn of_printed_suit(self, suit: Suit) -> CardSet {
        // The Joker's id (63) falls inside the hearts block, so mask it out.
        CardSet(self.0 & (0xFFFF << (suit as u32 * 16)) & !Card::JOKER.bit())
    }

    pub fn iter(self) -> CardIter {
        CardIter(self.0)
    }

    /// Parses space-separated cards, e.g. "Joker JH JD AH 4C". Fails on an unknown or repeated card.
    pub fn parse(text: &str) -> Option<CardSet> {
        let mut set = CardSet::EMPTY;
        for word in text.split_whitespace() {
            let card = Card::parse(word)?;
            if set.contains(card) {
                return None;
            }
            set.insert(card);
        }
        Some(set)
    }
}

impl FromIterator<Card> for CardSet {
    fn from_iter<I: IntoIterator<Item = Card>>(iter: I) -> Self {
        let mut set = CardSet::EMPTY;
        for card in iter {
            set.insert(card);
        }
        set
    }
}

impl fmt::Display for CardSet {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let words: Vec<String> = self.iter().map(|c| c.to_string()).collect();
        write!(f, "{}", words.join(" "))
    }
}

impl fmt::Debug for CardSet {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[{self}]")
    }
}

pub struct CardIter(u64);

impl Iterator for CardIter {
    type Item = Card;

    fn next(&mut self) -> Option<Card> {
        if self.0 == 0 {
            return None;
        }
        let id = self.0.trailing_zeros() as u8;
        self.0 &= self.0 - 1;
        Some(Card(id))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_and_display_round_trip() {
        for card in CardSet::pack().iter() {
            assert_eq!(Card::parse(&card.to_string()), Some(card));
        }
        assert_eq!(Card::parse("10d"), Some(Card::new(Suit::Diamonds, 10)));
        assert_eq!(Card::parse("TD"), Some(Card::new(Suit::Diamonds, 10)));
        assert_eq!(Card::parse("jk"), Some(Card::JOKER));
        assert_eq!(Card::parse("4S"), None, "black 4s are not in the pack");
        assert_eq!(Card::parse("3H"), None);
        assert_eq!(Card::parse("ZZ"), None);
    }
}
