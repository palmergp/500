//! The game state machine: deal → auction → kitty → ten tricks → score, until a team reaches the target.

use crate::bidding::{Auction, Call};
use crate::cards::{Card, CardSet, Suit};
use crate::rng::Rng;
use crate::rules::{Contract, Rules};
use crate::seat::{Seat, Team};
use crate::trick::{legal_plays, Play, Trick};
use std::fmt;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Phase {
    Bidding,
    /// The declarer has taken the kitty and is discarding three cards (H16).
    Discarding,
    Playing,
    GameOver,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Action {
    Call(Call),
    /// Discard one card; the declarer does this three times.
    Discard(Card),
    Play(Play),
}

impl fmt::Display for Action {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Action::Call(call) => call.fmt(f),
            Action::Discard(card) => f.pad(&format!("discard {card}")),
            Action::Play(play) => play.fmt(f),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RuleError {
    GameOver,
    Illegal(Action),
}

impl fmt::Display for RuleError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            RuleError::GameOver => write!(f, "the game is over"),
            RuleError::Illegal(action) => write!(f, "illegal action: {action}"),
        }
    }
}

impl std::error::Error for RuleError {}

/// Something that happened, safe to show every player: hidden cards (kitty, discards) never appear.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Event {
    Dealt { dealer: Seat, deal_number: u8, dealer_forced: bool },
    Called { seat: Seat, call: Call },
    /// Nobody bid 7+; the same dealer redeals (H14).
    PassedOut { dealer: Seat },
    ContractWon { declarer: Seat, contract: Contract },
    Discarded { seat: Seat },
    Played { seat: Seat, play: Play },
    TrickWon { winner: Seat, trick_number: u8 },
    HandScored { declarer: Seat, contract: Contract, tricks_won: [u8; 2], score_change: [i32; 2], scores: [i32; 2] },
    GameWon { team: Team },
}

/// A complete record of one deal, including passed-out deals, for review and statistics.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HandRecord {
    pub dealer: Seat,
    pub deal_number: u8,
    /// Each seat's ten cards as dealt.
    pub dealt: [CardSet; 4],
    pub kitty: CardSet,
    pub calls: Vec<(Seat, Call)>,
    /// `None` when the deal was passed out.
    pub contract: Option<(Seat, Contract)>,
    pub discards: CardSet,
    pub tricks: Vec<Trick>,
    pub tricks_won: [u8; 2],
    pub score_change: [i32; 2],
    pub scores_after: [i32; 2],
}

/// What one seat can see. Bots and hints receive only this, so they cannot use hidden cards.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Observation {
    pub seat: Seat,
    pub phase: Phase,
    pub to_act: Option<Seat>,
    pub rules: Rules,
    pub scores: [i32; 2],
    pub dealer: Seat,
    pub deal_number: u8,
    pub hand: CardSet,
    pub auction: Auction,
    pub contract: Option<(Seat, Contract)>,
    /// The kitty's three cards: known only to the declarer, once taken.
    pub kitty: Option<CardSet>,
    /// The declarer's discards so far: known only to the declarer.
    pub discards: Option<CardSet>,
    pub tricks: Vec<Trick>,
    pub current_trick: Option<Trick>,
    pub tricks_won: [u8; 2],
    /// Hands bid and played so far this game (passed-out deals don't count).
    pub hands_played: u8,
}

impl Observation {
    pub fn trump(&self) -> Option<Suit> {
        self.contract.and_then(|(_, c)| c.trump())
    }

    pub fn declarer(&self) -> Option<Seat> {
        self.contract.map(|(seat, _)| seat)
    }

    /// Every card played to a trick so far this hand.
    pub fn played(&self) -> CardSet {
        self.tricks
            .iter()
            .chain(self.current_trick.iter())
            .flat_map(|t| t.cards().iter().copied())
            .collect()
    }
}

#[derive(Clone, Debug)]
pub struct Game {
    rules: Rules,
    rng: Rng,
    scores: [i32; 2],
    phase: Phase,
    winner: Option<Team>,
    dealer: Seat,
    deal_number: u8,
    dealt: [CardSet; 4],
    hands: [CardSet; 4],
    kitty: CardSet,
    auction: Auction,
    contract: Option<(Seat, Contract)>,
    discards: CardSet,
    tricks: Vec<Trick>,
    current: Trick,
    tricks_won: [u8; 2],
    history: Vec<HandRecord>,
}

impl Game {
    /// A new game; `seed` determines the first dealer and every deal (H3).
    pub fn new(rules: Rules, seed: u64) -> Game {
        let mut rng = Rng::new(seed);
        let dealer = Seat::new(rng.below(4));
        Game::with_dealer(rules, seed, dealer)
    }

    pub fn with_dealer(rules: Rules, seed: u64, dealer: Seat) -> Game {
        let mut game = Game::empty(rules, Rng::new(seed ^ 0x5EED_0500), dealer);
        game.deal_shuffled();
        game
    }

    /// A game starting from a fixed deal, for tests and puzzles. Later deals come from `seed`.
    /// Panics unless the hands (10 cards each) and kitty (3) make up the whole pack.
    pub fn from_deal(rules: Rules, seed: u64, dealer: Seat, hands: [CardSet; 4], kitty: CardSet) -> Game {
        let all = hands.iter().fold(kitty, |acc, &h| acc.union(h));
        let sizes_ok = hands.iter().all(|h| h.len() == 10) && kitty.len() == 3;
        assert!(sizes_ok && all == CardSet::pack(), "from_deal needs 4×10 cards + 3 kitty = the 43-card pack");
        let mut game = Game::empty(rules, Rng::new(seed), dealer);
        game.set_deal(hands, kitty);
        game
    }

    fn empty(rules: Rules, rng: Rng, dealer: Seat) -> Game {
        Game {
            rules,
            rng,
            scores: [0, 0],
            phase: Phase::Bidding,
            winner: None,
            dealer,
            deal_number: 1,
            dealt: [CardSet::EMPTY; 4],
            hands: [CardSet::EMPTY; 4],
            kitty: CardSet::EMPTY,
            auction: Auction::new(dealer, false),
            contract: None,
            discards: CardSet::EMPTY,
            tricks: Vec::with_capacity(10),
            current: Trick::new(dealer),
            tricks_won: [0, 0],
            history: Vec::new(),
        }
    }

    fn deal_shuffled(&mut self) {
        let mut pack: Vec<Card> = CardSet::pack().iter().collect();
        self.rng.shuffle(&mut pack);
        let mut hands = [CardSet::EMPTY; 4];
        for (i, chunk) in pack[..40].chunks(10).enumerate() {
            hands[(self.dealer.index() + 1 + i) % 4] = chunk.iter().copied().collect();
        }
        self.set_deal(hands, pack[40..].iter().copied().collect());
    }

    fn set_deal(&mut self, hands: [CardSet; 4], kitty: CardSet) {
        self.dealt = hands;
        self.hands = hands;
        self.kitty = kitty;
        let forced = self.deal_number > self.rules.redeals_before_forced_bid;
        self.auction = Auction::new(self.dealer, forced);
        self.contract = None;
        self.discards = CardSet::EMPTY;
        self.tricks.clear();
        self.tricks_won = [0, 0];
        self.phase = Phase::Bidding;
    }

    // ---- Read access -------------------------------------------------------------------------

    pub fn rules(&self) -> &Rules {
        &self.rules
    }
    pub fn phase(&self) -> Phase {
        self.phase
    }
    pub fn scores(&self) -> [i32; 2] {
        self.scores
    }

    /// Starts this game from a given score (North-South, East-West), for lab studies of
    /// end-of-game play. Call it before any hand is scored.
    pub fn set_scores(&mut self, scores: [i32; 2]) {
        assert!(self.history.iter().all(|h| h.contract.is_none()), "set the score before any hand is played");
        self.scores = scores;
    }
    pub fn winner(&self) -> Option<Team> {
        self.winner
    }
    pub fn dealer(&self) -> Seat {
        self.dealer
    }
    pub fn deal_number(&self) -> u8 {
        self.deal_number
    }
    pub fn auction(&self) -> &Auction {
        &self.auction
    }
    pub fn contract(&self) -> Option<(Seat, Contract)> {
        self.contract
    }
    pub fn trump(&self) -> Option<Suit> {
        self.contract.and_then(|(_, c)| c.trump())
    }
    /// A seat's current cards. This is hidden information: give players [`Game::observe`] instead.
    pub fn hand(&self, seat: Seat) -> CardSet {
        self.hands[seat.index()]
    }
    pub fn kitty(&self) -> CardSet {
        self.kitty
    }
    pub fn tricks(&self) -> &[Trick] {
        &self.tricks
    }
    pub fn current_trick(&self) -> Option<&Trick> {
        (self.phase == Phase::Playing).then_some(&self.current)
    }
    pub fn tricks_won(&self) -> [u8; 2] {
        self.tricks_won
    }
    pub fn history(&self) -> &[HandRecord] {
        &self.history
    }

    pub fn to_act(&self) -> Option<Seat> {
        match self.phase {
            Phase::Bidding => self.auction.to_call(),
            Phase::Discarding => self.contract.map(|(seat, _)| seat),
            Phase::Playing => self.current.to_play(),
            Phase::GameOver => None,
        }
    }

    pub fn legal_actions(&self) -> Vec<Action> {
        match self.phase {
            Phase::Bidding => self.auction.legal_calls(&self.rules).into_iter().map(Action::Call).collect(),
            Phase::Discarding => {
                let (declarer, _) = self.contract.expect("discarding without a contract");
                self.hands[declarer.index()].iter().map(Action::Discard).collect()
            }
            Phase::Playing => {
                let seat = self.current.to_play().expect("playing a complete trick");
                legal_plays(self.hands[seat.index()], &self.current, self.trump())
                    .into_iter()
                    .map(Action::Play)
                    .collect()
            }
            Phase::GameOver => Vec::new(),
        }
    }

    /// Hands bid and played so far (passed-out deals don't count).
    pub fn hands_played(&self) -> u8 {
        self.history.iter().filter(|h| h.contract.is_some()).count() as u8
    }

    pub fn observe(&self, seat: Seat) -> Observation {
        let is_declarer = self.contract.map(|(s, _)| s) == Some(seat);
        let kitty_taken = self.phase != Phase::Bidding;
        Observation {
            seat,
            phase: self.phase,
            to_act: self.to_act(),
            rules: self.rules.clone(),
            scores: self.scores,
            dealer: self.dealer,
            deal_number: self.deal_number,
            hand: self.hands[seat.index()],
            auction: self.auction.clone(),
            contract: self.contract,
            kitty: (is_declarer && kitty_taken).then_some(self.kitty),
            discards: is_declarer.then_some(self.discards),
            tricks: self.tricks.clone(),
            current_trick: self.current_trick().cloned(),
            tricks_won: self.tricks_won,
            hands_played: self.hands_played(),
        }
    }

    // ---- Playing -----------------------------------------------------------------------------

    /// Applies the next player's action, returning what happened.
    pub fn apply(&mut self, action: Action) -> Result<Vec<Event>, RuleError> {
        if self.phase == Phase::GameOver {
            return Err(RuleError::GameOver);
        }
        if !self.legal_actions().contains(&action) {
            return Err(RuleError::Illegal(action));
        }
        let seat = self.to_act().expect("an unfinished game always has a player to act");
        let mut events = Vec::new();
        match action {
            Action::Call(call) => {
                self.auction.apply(call, &self.rules);
                events.push(Event::Called { seat, call });
                if self.auction.is_complete() {
                    self.finish_auction(&mut events);
                }
            }
            Action::Discard(card) => {
                self.hands[seat.index()].remove(card);
                self.discards.insert(card);
                events.push(Event::Discarded { seat });
                if self.discards.len() == 3 {
                    self.phase = Phase::Playing;
                    self.current = Trick::new(seat); // H17: the declarer leads first
                }
            }
            Action::Play(play) => {
                self.hands[seat.index()].remove(play.card);
                let trump = self.trump();
                self.current.push(play, trump);
                events.push(Event::Played { seat, play });
                if self.current.is_complete() {
                    let winner = self.current.winner(trump).expect("complete trick has a winner");
                    self.tricks_won[winner.team().index()] += 1;
                    let finished = std::mem::replace(&mut self.current, Trick::new(winner));
                    self.tricks.push(finished);
                    events.push(Event::TrickWon { winner, trick_number: self.tricks.len() as u8 });
                    if self.tricks.len() == 10 {
                        self.finish_hand(&mut events);
                    }
                }
            }
        }
        Ok(events)
    }

    fn finish_auction(&mut self, events: &mut Vec<Event>) {
        match self.auction.high_bid() {
            Some((declarer, contract)) => {
                self.contract = Some((declarer, contract));
                self.hands[declarer.index()] = self.hands[declarer.index()].union(self.kitty);
                self.phase = Phase::Discarding;
                events.push(Event::ContractWon { declarer, contract });
            }
            None => {
                // H14: thrown in; the same dealer redeals.
                self.record_hand([0, 0]);
                events.push(Event::PassedOut { dealer: self.dealer });
                self.deal_number += 1;
                self.start_deal(events);
            }
        }
    }

    fn finish_hand(&mut self, events: &mut Vec<Event>) {
        let (declarer, contract) = self.contract.expect("scoring without a contract");
        let team = declarer.team();
        let change = score_hand(&self.rules, contract, team, self.tricks_won, self.scores);
        for t in Team::ALL {
            self.scores[t.index()] += change[t.index()];
        }
        self.record_hand(change);
        events.push(Event::HandScored {
            declarer,
            contract,
            tricks_won: self.tricks_won,
            score_change: change,
            scores: self.scores,
        });
        let winner = match self.rules.hand_limit {
            Some(limit) => short_game_winner(limit, self.hands_played(), self.scores),
            None => game_winner(&self.rules, self.scores, team),
        };
        if let Some(winner) = winner {
            self.winner = Some(winner);
            self.phase = Phase::GameOver;
            events.push(Event::GameWon { team: winner });
        } else {
            self.dealer = self.dealer.next(); // H3
            self.deal_number = 1;
            self.start_deal(events);
        }
    }

    fn start_deal(&mut self, events: &mut Vec<Event>) {
        self.deal_shuffled();
        events.push(Event::Dealt {
            dealer: self.dealer,
            deal_number: self.deal_number,
            dealer_forced: self.auction.dealer_forced(),
        });
    }

    fn record_hand(&mut self, score_change: [i32; 2]) {
        self.history.push(HandRecord {
            dealer: self.dealer,
            deal_number: self.deal_number,
            dealt: self.dealt,
            kitty: self.kitty,
            calls: self.auction.calls().collect(),
            contract: self.contract,
            discards: self.discards,
            tricks: self.tricks.clone(),
            tricks_won: self.tricks_won,
            score_change,
            scores_after: self.scores,
        });
    }

    /// Checks that every card is accounted for exactly once and that scores match the history.
    pub fn check_invariants(&self) -> Result<(), String> {
        let played: CardSet = self.tricks.iter().chain([&self.current]).flat_map(|t| t.cards().iter().copied()).collect();
        let mut parts = vec![self.discards, played];
        parts.extend(self.hands);
        if self.phase == Phase::Bidding {
            parts.push(self.kitty);
        }
        let total: usize = parts.iter().map(|p| p.len()).sum();
        let union = parts.iter().fold(CardSet::EMPTY, |acc, &p| acc.union(p));
        if self.phase != Phase::GameOver && (total != 43 || union != CardSet::pack()) {
            return Err(format!("cards not partitioned: {total} cards in {:?}", self.phase));
        }
        for t in Team::ALL {
            let sum: i32 = self.history.iter().map(|h| h.score_change[t.index()]).sum();
            if sum != self.scores[t.index()] {
                return Err(format!("{t} score {} != history total {sum}", self.scores[t.index()]));
            }
        }
        if self.tricks_won.iter().map(|&n| n as usize).sum::<usize>() != self.tricks.len() {
            return Err("trick count mismatch".into());
        }
        Ok(())
    }
}

/// The score change for both teams after a hand (H20–H22). Applies the defensive cap when
/// `rules.defenders_can_win` is off.
pub fn score_hand(rules: &Rules, contract: Contract, declarers: Team, tricks_won: [u8; 2], scores: [i32; 2]) -> [i32; 2] {
    let (d, o) = (declarers.index(), declarers.other().index());
    let mut change = [0; 2];
    change[d] = if tricks_won[d] >= contract.level {
        if tricks_won[d] == 10 {
            contract.value().max(rules.slam_score)
        } else {
            contract.value()
        }
    } else {
        -contract.value()
    };
    let defence = rules.defender_points_per_trick * tricks_won[o] as i32;
    change[o] = if rules.defenders_can_win {
        defence
    } else {
        let cap = (rules.target_score - 10).max(scores[o]);
        (scores[o] + defence).min(cap) - scores[o]
    };
    change
}

/// The winner of a short game (H25) after `hands` hands: the higher score once the limit is
/// reached, or nobody yet if the scores are tied (another hand is played).
pub fn short_game_winner(limit: u8, hands: u8, scores: [i32; 2]) -> Option<Team> {
    if hands < limit || scores[0] == scores[1] {
        None
    } else if scores[0] > scores[1] {
        Some(Team::NorthSouth)
    } else {
        Some(Team::EastWest)
    }
}

/// The winning team after a hand, counting the declarers first (H23).
pub fn game_winner(rules: &Rules, scores: [i32; 2], declarers: Team) -> Option<Team> {
    if scores[declarers.index()] >= rules.target_score {
        Some(declarers)
    } else if scores[declarers.other().index()] >= rules.target_score {
        Some(declarers.other())
    } else {
        None
    }
}
