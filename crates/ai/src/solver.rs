//! Double-dummy solver for 500: with all four hands visible, how many tricks can each side take
//! with perfect play from here?
//!
//! Each contract is remapped once: every card becomes (effective suit, bit) with higher bits
//! stronger, so the left bower and Joker simply sit at the top of the trump suit. In no trumps
//! the Joker gets its own "suit" (index 4). The search is alpha-beta on trick counts with a
//! transposition table at trick boundaries and equivalent-card pruning.

use crate::cardplay::suit_order;
use fh_core::{legal_plays, Card, CardSet, Play, Seat, Suit, Team, Trick};
use std::collections::HashMap;
use std::hash::{BuildHasherDefault, Hasher};

const JOKER_SUIT: usize = 4;

/// All four hands plus the trick in progress: a perfect-information position.
#[derive(Clone, Debug)]
pub struct Position {
    pub hands: [CardSet; 4],
    pub trump: Option<Suit>,
    pub trick: Trick,
}

impl Position {
    pub fn to_play(&self) -> Seat {
        self.trick.to_play().expect("position has a player to act")
    }
}

/// Maps physical cards to (effective suit, bit) for one contract.
#[derive(Clone)]
pub struct Mapping {
    slot: [(u8, u8); 64],
    cards: [[Card; 16]; 5],
}

impl Mapping {
    pub fn new(trump: Option<Suit>) -> Mapping {
        let mut map = Mapping { slot: [(0, 0); 64], cards: [[Card::JOKER; 16]; 5] };
        for suit in Suit::ALL {
            let order = suit_order(suit, trump);
            let n = order.len();
            for (i, card) in order.into_iter().enumerate() {
                let bit = (n - 1 - i) as u8;
                map.slot[card.id() as usize] = (suit.index() as u8, bit);
                map.cards[suit.index()][bit as usize] = card;
            }
        }
        if trump.is_none() {
            map.slot[Card::JOKER.id() as usize] = (JOKER_SUIT as u8, 0);
        }
        map
    }

    fn masks(&self, set: CardSet) -> [u16; 5] {
        let mut masks = [0u16; 5];
        for card in set.iter() {
            let (suit, bit) = self.slot[card.id() as usize];
            masks[suit as usize] |= 1 << bit;
        }
        masks
    }

    fn card(&self, suit: usize, bit: u8) -> Card {
        self.cards[suit][bit as usize]
    }
}

#[derive(Copy, Clone)]
struct Move {
    suit: u8,
    bit: u8,
    /// The suit named when leading the Joker in no trumps.
    named: u8,
    order: i16,
}

#[derive(Copy, Clone)]
struct TrickState {
    leader: u8,
    count: u8,
    led: u8,
    winner: u8,
    win_key: u16,
}

/// FxHash-style hasher: much faster than SipHash for the solver's integer keys.
#[derive(Default)]
struct FastHasher(u64);

impl Hasher for FastHasher {
    fn finish(&self) -> u64 {
        self.0
    }
    fn write(&mut self, bytes: &[u8]) {
        for &b in bytes {
            self.write_u64(b as u64);
        }
    }
    fn write_u8(&mut self, n: u8) {
        self.write_u64(n as u64);
    }
    fn write_u64(&mut self, n: u64) {
        self.0 = (self.0.rotate_left(5) ^ n).wrapping_mul(0x51_7c_c1_b7_27_22_0a_95);
    }
}

/// Bounds on the value, plus the best move found (suit, bit, named suit), or `NO_MOVE`.
#[derive(Copy, Clone)]
struct Entry {
    lo: i8,
    hi: i8,
    best: (u8, u8, u8),
}

const NO_MOVE: (u8, u8, u8) = (255, 0, 0);

type Table = HashMap<([u64; 4], u8), Entry, BuildHasherDefault<FastHasher>>;

pub struct Solver {
    map: Mapping,
    trump: Option<u8>,
    hands: [[u16; 5]; 4],
    on_table: [u16; 5],
    max_team: u8,
    table: Table,
    /// Nodes searched since the solver was created.
    pub nodes: u64,
}

impl Solver {
    pub fn new(trump: Option<Suit>) -> Solver {
        Solver {
            map: Mapping::new(trump),
            trump: trump.map(|t| t.index() as u8),
            hands: [[0; 5]; 4],
            on_table: [0; 5],
            max_team: 0,
            table: Table::default(),
            nodes: 0,
        }
    }

    /// Tricks `team` takes from this position to the end of the hand (including the trick in
    /// progress) for each legal play of the player to act, assuming perfect play by everyone.
    pub fn analyze(&mut self, pos: &Position, team: Team) -> Vec<(Play, u8)> {
        let state = self.setup(pos, team);
        let seat = pos.to_play();
        // Each remaining trick, including the one in progress, takes one of this seat's cards.
        let tricks_left = pos.hands[seat.index()].len() as i8;
        let mut values: Vec<(Play, u8)> = Vec::new();
        let mut solved: Vec<((u8, u8, u8), u8)> = Vec::new();
        let mut guess = tricks_left / 2;
        let mover = seat.index() as u8;
        for play in legal_plays(pos.hands[seat.index()], &pos.trick, pos.trump) {
            let (suit, bit) = self.map.slot[play.card.id() as usize];
            let named = play.named_suit.map_or(suit, |s| s.index() as u8);
            let group = (suit, self.group_top(mover, suit, bit), named);
            let value = match solved.iter().find(|(g, _)| *g == group) {
                Some(&(_, v)) => v,
                None => {
                    let m = Move { suit, bit, named, order: 0 };
                    let v = Self::mtd(guess, tricks_left, |a, b| self.play(mover, state, m, a, b)) as u8;
                    guess = v as i8;
                    solved.push((group, v));
                    v
                }
            };
            values.push((play, value));
        }
        values
    }

    /// The highest card of the run of equivalent cards (no live card in between) containing `bit`.
    fn group_top(&self, seat: u8, suit: u8, bit: u8) -> u8 {
        let suit = suit as usize;
        if suit == JOKER_SUIT {
            return 0;
        }
        let mine = self.hands[seat as usize][suit];
        let live = self.hands.iter().fold(self.on_table[suit], |acc, h| acc | h[suit]);
        let mut top = bit;
        loop {
            let above = live & !((1u32 << (top + 1)) - 1) as u16;
            if above == 0 {
                return top;
            }
            let next = above.trailing_zeros() as u8;
            if mine & (1 << next) == 0 {
                return top;
            }
            top = next;
        }
    }

    /// Tricks `team` takes from this position with perfect play.
    pub fn solve(&mut self, pos: &Position, team: Team) -> u8 {
        let state = self.setup(pos, team);
        let most = pos.hands[pos.to_play().index()].len() as i8;
        Self::mtd(most / 2, most, |a, b| self.search(state, a, b)) as u8
    }

    /// The exact value of a search, found with a sequence of null-window tests (MTD(f)).
    fn mtd(guess: i8, most: i8, mut test: impl FnMut(i8, i8) -> i8) -> i8 {
        let (mut lower, mut upper) = (0i8, most);
        let mut g = guess.clamp(lower, upper);
        while lower < upper {
            let beta = if g == lower { g + 1 } else { g };
            let v = test(beta - 1, beta);
            if v < beta {
                upper = v;
            } else {
                lower = v;
            }
            g = v.clamp(lower, upper);
        }
        lower
    }

    fn setup(&mut self, pos: &Position, team: Team) -> TrickState {
        if self.max_team != team.index() as u8 {
            self.table.clear(); // table values are relative to the maximizing team
        }
        self.max_team = team.index() as u8;
        for seat in Seat::ALL {
            self.hands[seat.index()] = self.map.masks(pos.hands[seat.index()]);
        }
        self.on_table = [0; 5];
        let mut state = TrickState { leader: pos.trick.leader().index() as u8, count: 0, led: 0, winner: 0, win_key: 0 };
        if let Some(led) = pos.trick.led_suit() {
            state.led = led.index() as u8;
        }
        for (seat, card) in pos.trick.plays() {
            let (suit, bit) = self.map.slot[card.id() as usize];
            self.on_table[suit as usize] |= 1 << bit;
            let key = self.key(suit, bit, state.led);
            if state.count == 0 || key > state.win_key {
                state.winner = seat.index() as u8;
                state.win_key = key;
            }
            state.count += 1;
        }
        state
    }

    fn key(&self, suit: u8, bit: u8, led: u8) -> u16 {
        if suit as usize == JOKER_SUIT {
            1000
        } else if Some(suit) == self.trump {
            200 + bit as u16
        } else if suit == led {
            100 + bit as u16
        } else {
            0
        }
    }

    /// The table key at a trick boundary. Ranks are made relative (gaps left by played cards are
    /// squeezed out), so positions that differ only in which cards are gone share an entry.
    fn table_key(&self, leader: u8) -> ([u64; 4], u8) {
        let mut packed = [0u64; 4];
        for suit in 0..4 {
            let mut live = self.hands.iter().fold(0u16, |acc, h| acc | h[suit]);
            let mut rank = 0;
            while live != 0 {
                let bit = live & live.wrapping_neg();
                live &= live - 1;
                for (seat, hand) in self.hands.iter().enumerate() {
                    if hand[suit] & bit != 0 {
                        packed[seat] |= 1 << (suit * 16 + rank);
                        break;
                    }
                }
                rank += 1;
            }
        }
        for (seat, hand) in self.hands.iter().enumerate() {
            if hand[JOKER_SUIT] != 0 {
                packed[seat] |= 1 << 63;
            }
        }
        (packed, leader)
    }

    fn search(&mut self, t: TrickState, alpha: i8, beta: i8) -> i8 {
        self.nodes += 1;
        let seat = (t.leader + t.count) % 4;
        if t.count > 0 {
            return self.search_moves(t, seat, alpha, beta);
        }
        let left = self.hands[seat as usize].iter().map(|m| m.count_ones() as i8).sum::<i8>();
        if left == 0 {
            return 0;
        }
        if left == 1 {
            return self.last_trick(seat);
        }
        let key = self.table_key(t.leader);
        let stored = self.table.get(&key).copied();
        let (mut lo, mut hi) = stored.map_or((0, left), |e| (e.lo, e.hi));
        let hint = stored.map_or(NO_MOVE, |e| e.best);
        // Tricks the side on lead can cash for certain, and top trumps that must each win a trick.
        let quick = self.quick_tricks(seat).min(left);
        let (sure_team, sure) = self.sure_trump_tricks();
        if seat % 2 == self.max_team {
            lo = lo.max(quick);
        } else {
            hi = hi.min(left - quick);
        }
        if sure > 0 {
            if sure_team == self.max_team {
                lo = lo.max(sure.min(left));
            } else {
                hi = hi.min(left - sure.min(left));
            }
        }
        // Return at once if the known bounds already answer the window, which also guarantees
        // the narrowed window below is non-empty.
        if lo >= hi || lo >= beta {
            return lo;
        }
        if hi <= alpha {
            return hi;
        }
        let (a, b) = (alpha.max(lo), beta.min(hi));
        let (v, best) = self.search_moves_with(t, seat, a, b, hint);
        let (lo, hi) = if v <= a {
            (lo, v.min(hi))
        } else if v >= b {
            (v.max(lo), hi)
        } else {
            (v, v)
        };
        self.table.insert(key, Entry { lo, hi, best });
        v
    }

    /// With one card left each: play them out and return 1 if the maximizing side wins.
    fn last_trick(&self, leader: u8) -> i8 {
        let card_of = |seat: u8| -> (u8, u8) {
            let hand = &self.hands[seat as usize];
            let suit = (0..5).find(|&s| hand[s] != 0).expect("one card left");
            (suit as u8, hand[suit].trailing_zeros() as u8)
        };
        let (lead_suit, lead_bit) = card_of(leader);
        // A Joker led in no trumps names a suit, but on the last trick nothing can follow it anyway.
        let led = if lead_suit as usize == JOKER_SUIT { 0 } else { lead_suit };
        let mut winner = leader;
        let mut best = self.key(lead_suit, lead_bit, led);
        for i in 1..4 {
            let seat = (leader + i) % 4;
            let (suit, bit) = card_of(seat);
            let key = self.key(suit, bit, led);
            if key > best {
                best = key;
                winner = seat;
            }
        }
        (winner % 2 == self.max_team) as i8
    }

    /// Trumps that are certain to win: the run of top live trumps in the hand holding the highest
    /// one (in no trumps, the Joker). Each is played on a different trick and nothing can beat it.
    /// Returns (team, tricks).
    fn sure_trump_tricks(&self) -> (u8, i8) {
        let suit = match self.trump {
            Some(t) => t as usize,
            None => JOKER_SUIT,
        };
        let live = self.hands.iter().fold(0u16, |acc, h| acc | h[suit]);
        if live == 0 {
            return (0, 0);
        }
        let top = 1u16 << (15 - live.leading_zeros());
        let holder = (0..4).find(|&s| self.hands[s][suit] & top != 0).unwrap();
        let mut run = 0;
        let mut probe = live;
        while probe != 0 {
            let bit = 1u16 << (15 - probe.leading_zeros());
            if self.hands[holder][suit] & bit == 0 {
                break;
            }
            run += 1;
            probe &= !bit;
        }
        ((holder % 2) as u8, run)
    }

    /// Winners the player on lead (`seat`) can cash in a row without losing the lead: the top live
    /// cards of each suit in their own hand. Side suits count only once the opponents can no
    /// longer trump them (or, in no trumps, the Joker is not with an opponent).
    fn quick_tricks(&self, seat: u8) -> i8 {
        let hand = &self.hands[seat as usize];
        let opp = [(seat + 1) % 4, (seat + 3) % 4];
        let top_run = |suit: usize| -> i8 {
            let live = self.hands.iter().fold(0u16, |acc, h| acc | h[suit]);
            let mut run = 0;
            let mut probe = live;
            while probe != 0 {
                let top = 1u16 << (15 - probe.leading_zeros());
                if hand[suit] & top == 0 {
                    break;
                }
                run += 1;
                probe &= !top;
            }
            run
        };
        let mut quick = 0;
        match self.trump {
            Some(t) => {
                let t = t as usize;
                quick += top_run(t);
                let opponents_trumpless = opp.iter().all(|&o| self.hands[o as usize][t] == 0);
                // Opponents' remaining trumps fall to our top-trump run only if it covers them.
                let theirs: u32 = opp.iter().map(|&o| self.hands[o as usize][t].count_ones()).sum();
                if opponents_trumpless || (quick as u32 >= theirs && theirs == 0) {
                    for suit in (0..4).filter(|&s| s != t) {
                        quick += top_run(suit);
                    }
                }
            }
            None => {
                if hand[JOKER_SUIT] != 0 {
                    quick += 1;
                }
                let joker_with_opponent = opp.iter().any(|&o| self.hands[o as usize][JOKER_SUIT] != 0);
                if !joker_with_opponent {
                    for suit in 0..4 {
                        quick += top_run(suit);
                    }
                }
            }
        }
        quick
    }

    fn search_moves(&mut self, t: TrickState, seat: u8, alpha: i8, beta: i8) -> i8 {
        self.search_moves_with(t, seat, alpha, beta, NO_MOVE).0
    }

    /// Searches `seat`'s moves, trying `hint` first. Returns the value and the best move.
    fn search_moves_with(&mut self, t: TrickState, seat: u8, mut alpha: i8, mut beta: i8, hint: (u8, u8, u8)) -> (i8, (u8, u8, u8)) {
        let maximizing = seat % 2 == self.max_team;
        let (mut moves, n) = self.moves(seat, &t);
        if hint != NO_MOVE {
            if let Some(i) = moves[..n].iter().position(|m| (m.suit, m.bit, m.named) == hint) {
                moves[..=i].rotate_right(1);
            }
        }
        let mut best = if maximizing { -1 } else { 127 };
        let mut best_move = NO_MOVE;
        for &m in &moves[..n] {
            let v = self.play(seat, t, m, alpha, beta);
            if (maximizing && v > best) || (!maximizing && v < best) {
                best = v;
                best_move = (m.suit, m.bit, m.named);
            }
            if maximizing {
                alpha = alpha.max(best);
            } else {
                beta = beta.min(best);
            }
            if alpha >= beta {
                break;
            }
        }
        (best, best_move)
    }

    fn play(&mut self, seat: u8, t: TrickState, m: Move, alpha: i8, beta: i8) -> i8 {
        let bit = 1u16 << m.bit;
        self.hands[seat as usize][m.suit as usize] &= !bit;
        self.on_table[m.suit as usize] |= bit;
        let mut next = t;
        if t.count == 0 {
            next.led = m.named;
            next.winner = seat;
            next.win_key = self.key(m.suit, m.bit, m.named);
        } else {
            let key = self.key(m.suit, m.bit, t.led);
            if key > t.win_key {
                next.winner = seat;
                next.win_key = key;
            }
        }
        next.count += 1;
        let value = if next.count == 4 {
            let won = (next.winner % 2 == self.max_team) as i8;
            let saved = self.on_table;
            self.on_table = [0; 5];
            let start = TrickState { leader: next.winner, count: 0, led: 0, winner: 0, win_key: 0 };
            let v = won + self.search(start, alpha - won, beta - won);
            self.on_table = saved;
            v
        } else {
            self.search(next, alpha, beta)
        };
        self.on_table[m.suit as usize] &= !bit;
        self.hands[seat as usize][m.suit as usize] |= bit;
        value
    }

    /// Distinct, ordered moves for `seat`: one card per group of equivalent cards.
    fn moves(&self, seat: u8, t: &TrickState) -> ([Move; 16], usize) {
        let hand = &self.hands[seat as usize];
        let mut out = [Move { suit: 0, bit: 0, named: 0, order: 0 }; 16];
        let mut n = 0;
        let push = |m: Move, out: &mut [Move; 16], n: &mut usize| {
            out[*n] = m;
            *n += 1;
        };
        let partner_winning = t.count > 0 && t.winner % 2 == seat % 2;
        const SUITS: [usize; 4] = [0, 1, 2, 3];
        let led = t.led as usize;
        let suits: &[usize] = if t.count > 0 && hand[led] != 0 { &SUITS[led..=led] } else { &SUITS };
        for &suit in suits {
            let mine = hand[suit];
            if mine == 0 {
                continue;
            }
            let live = self.hands.iter().fold(self.on_table[suit], |acc, h| acc | h[suit]);
            let top_live = 15 - live.leading_zeros() as u8;
            let mut prev: Option<u8> = None;
            let mut bits = mine;
            while bits != 0 {
                let b = 15 - bits.leading_zeros() as u8;
                bits &= !(1 << b);
                if let Some(p) = prev {
                    let between = live & (((1u32 << p) - 1) as u16) & !(((1u32 << (b + 1)) - 1) as u16);
                    if between == 0 {
                        prev = Some(b);
                        continue; // equivalent to the card just above it
                    }
                }
                prev = Some(b);
                let order = if t.count == 0 {
                    if b == top_live {
                        100 + mine.count_ones() as i16
                    } else {
                        20 + mine.count_ones() as i16 - b as i16
                    }
                } else {
                    let key = self.key(suit as u8, b, t.led);
                    if !partner_winning && key > t.win_key {
                        100 - b as i16 // cheapest winner first
                    } else {
                        50 - b as i16 - if Some(suit as u8) == self.trump { 20 } else { 0 }
                    }
                };
                push(Move { suit: suit as u8, bit: b, named: suit as u8, order }, &mut out, &mut n);
            }
        }
        if hand[JOKER_SUIT] != 0 && (t.count == 0 || suits.len() == 4) {
            if t.count == 0 {
                for named in 0..4u8 {
                    push(Move { suit: JOKER_SUIT as u8, bit: 0, named, order: 60 }, &mut out, &mut n);
                }
            } else {
                let order = if partner_winning { 0 } else { 90 };
                push(Move { suit: JOKER_SUIT as u8, bit: 0, named: t.led, order }, &mut out, &mut n);
            }
        }
        out[..n].sort_unstable_by(|a, b| b.order.cmp(&a.order));
        (out, n)
    }

    /// The physical card for an effective (suit, bit), for debugging and tests.
    pub fn card_at(&self, suit: usize, bit: u8) -> Card {
        self.map.card(suit, bit)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fh_core::{Game, Rules};

    fn set(text: &str) -> CardSet {
        CardSet::parse(text).unwrap()
    }

    /// Brute-force minimax over legal plays, for checking the solver on small endings.
    fn brute(hands: [CardSet; 4], trump: Option<Suit>, leader: Seat, plays: Vec<Play>, team: Team) -> u8 {
        let trick = Trick::from_plays(leader, &plays, trump);
        let seat = trick.to_play().unwrap();
        let maximizing = seat.team() == team;
        let mut best: Option<u8> = None;
        for play in legal_plays(hands[seat.index()], &trick, trump) {
            let mut h = hands;
            h[seat.index()].remove(play.card);
            let mut next = plays.clone();
            next.push(play);
            let value = if next.len() == 4 {
                let winner = Trick::from_plays(leader, &next, trump).winner(trump).unwrap();
                let won = (winner.team() == team) as u8;
                if h[0].is_empty() {
                    won
                } else {
                    won + brute(h, trump, winner, Vec::new(), team)
                }
            } else {
                brute(h, trump, leader, next, team)
            };
            best = Some(match best {
                None => value,
                Some(b) if maximizing => b.max(value),
                Some(b) => b.min(value),
            });
        }
        best.unwrap()
    }

    #[test]
    fn matches_brute_force_on_random_endings() {
        let mut rng = fh_core::Rng::new(9);
        for case in 0..600 {
            // Deal 4 cards each from a random pack, random trump, random leader.
            let mut pack: Vec<Card> = CardSet::pack().iter().collect();
            rng.shuffle(&mut pack);
            let hands: [CardSet; 4] = std::array::from_fn(|i| pack[i * 4..i * 4 + 4].iter().copied().collect());
            let trump = [None, Some(Suit::Spades), Some(Suit::Clubs), Some(Suit::Diamonds), Some(Suit::Hearts)][rng.below(5)];
            let leader = Seat::new(rng.below(4));
            // Start some cases mid-trick, after 1–3 random legal plays.
            let mut hands = hands;
            let mut plays: Vec<Play> = Vec::new();
            for _ in 0..rng.below(4) {
                let trick = Trick::from_plays(leader, &plays, trump);
                let seat = trick.to_play().unwrap();
                let options = legal_plays(hands[seat.index()], &trick, trump);
                let play = options[rng.below(options.len())];
                hands[seat.index()].remove(play.card);
                plays.push(play);
            }
            let pos = Position { hands, trump, trick: Trick::from_plays(leader, &plays, trump) };
            let team = Team::ALL[rng.below(2)];
            let expected = brute(hands, trump, leader, plays.clone(), team);
            let mut solver = Solver::new(trump);
            assert_eq!(solver.solve(&pos, team), expected, "case {case}: {hands:?} trump {trump:?} leader {leader}");
            let values: Vec<u8> = solver.analyze(&pos, team).iter().map(|&(_, v)| v).collect();
            let mover = pos.to_play();
            let best = if mover.team() == team { values.iter().max() } else { values.iter().min() }.copied();
            assert_eq!(best, Some(expected), "case {case}: analyze disagrees");
        }
    }

    #[test]
    fn joker_and_bowers_in_a_simple_ending() {
        // Hearts trumps. South holds the top three trumps and leads: three tricks.
        let hands = [set("Joker JH JD"), set("AH KH QH"), set("5S 6S 7S"), set("8S 9S 10S")];
        let pos = Position { hands, trump: Some(Suit::Hearts), trick: Trick::new(Seat::SOUTH) };
        assert_eq!(Solver::new(Some(Suit::Hearts)).solve(&pos, Team::NorthSouth), 3);
        // In no trumps the jacks are ordinary. West's A♥ wins (South must follow with J♥), then
        // South trumps a heart with the Joker and cashes J♦: East–West take just one trick.
        let pos = Position { hands, trump: None, trick: Trick::new(Seat::WEST) };
        assert_eq!(Solver::new(None).solve(&pos, Team::EastWest), 1);
    }

    #[test]
    fn solves_full_deals() {
        for seed in 0..5 {
            let game = Game::new(Rules::default(), seed);
            let mut hands: [CardSet; 4] = std::array::from_fn(|i| game.hand(Seat::new(i)));
            // Give the kitty's 3 cards to South and discard South's 3 lowest (by id) to keep 10 each.
            let south = hands[0].union(game.kitty());
            let dropped: CardSet = south.iter().take(3).collect();
            hands[0] = south.minus(dropped);
            let pos = Position { hands, trump: Some(Suit::Hearts), trick: Trick::new(Seat::SOUTH) };
            let mut solver = Solver::new(Some(Suit::Hearts));
            let ns = solver.solve(&pos, Team::NorthSouth);
            let ew = Solver::new(Some(Suit::Hearts)).solve(&pos, Team::EastWest);
            assert_eq!(ns + ew, 10, "seed {seed}: the two sides' optimal tricks must sum to 10");
        }
    }
}
