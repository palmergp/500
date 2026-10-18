//! The auction: one call per player, starting left of the dealer (H8–H15).

use crate::cards::Suit;
use crate::rules::{Contract, Rules, Strain, MAX_BID_LEVEL, MIN_BID_LEVEL};
use crate::seat::Seat;
use std::fmt;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Call {
    Pass,
    /// "This is my strongest strain, but I can't bid" (H10). Never scores.
    Mention(Strain),
    /// "I hold the Joker, and this is my best suit" (H11). Never becomes the contract.
    Six(Suit),
    /// A real bid of 7–10 tricks (H13).
    Bid(Contract),
}

impl fmt::Display for Call {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let text = match self {
            Call::Pass => "Pass".to_string(),
            Call::Mention(strain) => format!("Mention {strain}"),
            Call::Six(suit) => format!("6{suit} (Joker)"),
            Call::Bid(contract) => contract.to_string(),
        };
        f.pad(&text)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Auction {
    dealer: Seat,
    dealer_forced: bool,
    calls: Vec<Call>,
}

impl Auction {
    /// `dealer_forced`: this is the last allowed deal, so the dealer must bid if nobody else has (H14).
    pub fn new(dealer: Seat, dealer_forced: bool) -> Auction {
        Auction { dealer, dealer_forced, calls: Vec::with_capacity(4) }
    }

    pub fn dealer(&self) -> Seat {
        self.dealer
    }

    pub fn dealer_forced(&self) -> bool {
        self.dealer_forced
    }

    /// Calls so far, with the seat that made each.
    pub fn calls(&self) -> impl Iterator<Item = (Seat, Call)> + '_ {
        let first = self.dealer.next();
        self.calls.iter().enumerate().map(move |(i, &call)| (Seat::new(first.index() + i), call))
    }

    pub fn is_complete(&self) -> bool {
        self.calls.len() == 4
    }

    pub fn to_call(&self) -> Option<Seat> {
        (!self.is_complete()).then(|| Seat::new(self.dealer.index() + 1 + self.calls.len()))
    }

    /// The highest real bid and its bidder. Bids must rise, so this is the latest bid.
    pub fn high_bid(&self) -> Option<(Seat, Contract)> {
        self.calls().filter_map(|(seat, call)| match call {
            Call::Bid(contract) => Some((seat, contract)),
            _ => None,
        }).last()
    }

    pub fn highest_mention(&self) -> Option<Strain> {
        self.calls().filter_map(|(_, call)| match call {
            Call::Mention(strain) => Some(strain),
            _ => None,
        }).max()
    }

    /// Who called 6, and in which suit.
    pub fn six(&self) -> Option<(Seat, Suit)> {
        self.calls().find_map(|(seat, call)| match call {
            Call::Six(suit) => Some((seat, suit)),
            _ => None,
        })
    }

    /// Mentions and 6-calls are allowed only while nobody has called 6 or bid (H12).
    pub fn signals_open(&self) -> bool {
        self.six().is_none() && self.high_bid().is_none()
    }

    /// Whether the player to call is the dealer and must bid (H14).
    pub fn must_bid(&self) -> bool {
        self.dealer_forced && self.to_call() == Some(self.dealer) && self.high_bid().is_none()
    }

    pub fn legal_calls(&self, rules: &Rules) -> Vec<Call> {
        let mut calls = Vec::new();
        if self.is_complete() {
            return calls;
        }
        if !self.must_bid() {
            calls.push(Call::Pass);
            if self.signals_open() {
                if rules.mentions {
                    let floor = self.highest_mention();
                    calls.extend(
                        Strain::ALL.into_iter().filter(|&s| floor.is_none_or(|m| s > m)).map(Call::Mention),
                    );
                }
                if rules.joker_six {
                    calls.extend(Suit::ALL.into_iter().map(Call::Six));
                }
            }
        }
        let high = self.high_bid().map(|(_, contract)| contract);
        for level in MIN_BID_LEVEL..=MAX_BID_LEVEL {
            for strain in Strain::ALL {
                let contract = Contract::new(level, strain);
                if high.is_none_or(|h| contract > h) {
                    calls.push(Call::Bid(contract));
                }
            }
        }
        calls
    }

    /// Records `call` if it is legal; returns whether it was.
    pub fn apply(&mut self, call: Call, rules: &Rules) -> bool {
        let legal = self.legal_calls(rules).contains(&call);
        if legal {
            self.calls.push(call);
        }
        legal
    }
}
