//! JSON-friendly views of the table for the UI, always from the human's (South's) point of view.

use crate::table::{Table, HUMAN};
use fh_ai::cardplay::suit_order;
use fh_ai::persona;
use fh_core::cards::JACK;
use fh_core::rules::is_trump;
use fh_core::{Action, Call, Card, CardSet, Event, HandRecord, Phase, Seat, Strain, Suit, Trick};
use serde::Serialize;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CardView {
    pub id: u8,
    pub rank: &'static str,
    /// "S", "C", "D", "H", or null for the Joker.
    pub suit: Option<char>,
    pub label: String,
    pub red: bool,
    /// "right" or "left" when the card is a bower in the current contract.
    pub bower: Option<&'static str>,
    pub trump: bool,
}

pub fn card_view(card: Card, trump: Option<Suit>) -> CardView {
    let bower = match (card.suit(), trump) {
        (Some(s), Some(t)) if card.rank() == JACK && s == t => Some("right"),
        (Some(s), Some(t)) if card.rank() == JACK && s == t.same_color() => Some("left"),
        _ => None,
    };
    CardView {
        id: card.id(),
        rank: card.rank_label(),
        suit: card.suit().map(Suit::letter),
        label: card.to_string(),
        red: card.suit().is_some_and(Suit::is_red),
        bower,
        trump: is_trump(card, trump),
    }
}

/// Cards in display order: grouped by effective suit, strongest first, colors alternating,
/// with the trump suit (or the Joker in no trumps / before the auction ends) first.
pub fn display_order(cards: CardSet, trump: Option<Suit>) -> Vec<Card> {
    const ALTERNATING: [Suit; 4] = [Suit::Spades, Suit::Hearts, Suit::Clubs, Suit::Diamonds];
    let start = trump.map_or(0, |t| ALTERNATING.iter().position(|&s| s == t).unwrap());
    let mut out = Vec::with_capacity(cards.len());
    if trump.is_none() && cards.contains(Card::JOKER) {
        out.push(Card::JOKER);
    }
    for i in 0..4 {
        let suit = ALTERNATING[(start + i) % 4];
        out.extend(suit_order(suit, trump).into_iter().filter(|&c| cards.contains(c)));
    }
    out
}

fn cards_view(cards: CardSet, trump: Option<Suit>) -> Vec<CardView> {
    display_order(cards, trump).into_iter().map(|c| card_view(c, trump)).collect()
}

pub fn strain_code(strain: Strain) -> &'static str {
    ["S", "C", "D", "H", "NT"][strain.index()]
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ActionView {
    pub index: usize,
    /// "pass", "mention", "six", "bid", "discard" or "play".
    pub kind: &'static str,
    pub label: String,
    pub level: Option<u8>,
    pub strain: Option<&'static str>,
    pub card: Option<u8>,
    pub named_suit: Option<char>,
}

fn action_view(index: usize, action: Action) -> ActionView {
    let mut view = ActionView {
        index,
        kind: "pass",
        label: action.to_string(),
        level: None,
        strain: None,
        card: None,
        named_suit: None,
    };
    match action {
        Action::Call(Call::Pass) => {}
        Action::Call(Call::Mention(s)) => {
            view.kind = "mention";
            view.strain = Some(strain_code(s));
        }
        Action::Call(Call::Six(s)) => {
            view.kind = "six";
            view.level = Some(6);
            view.strain = Some(strain_code(s));
        }
        Action::Call(Call::Bid(c)) => {
            view.kind = "bid";
            view.level = Some(c.level);
            view.strain = Some(strain_code(c.strain));
        }
        Action::Discard(card) => {
            view.kind = "discard";
            view.card = Some(card.id());
        }
        Action::Play(play) => {
            view.kind = "play";
            view.card = Some(play.card.id());
            view.named_suit = play.named_suit.map(Suit::letter);
        }
    }
    view
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CallView {
    pub seat: usize,
    pub kind: &'static str,
    pub label: String,
}

fn call_view(seat: Seat, call: Call) -> CallView {
    let kind = match call {
        Call::Pass => "pass",
        Call::Mention(_) => "mention",
        Call::Six(_) => "six",
        Call::Bid(_) => "bid",
    };
    CallView { seat: seat.index(), kind, label: call.to_string() }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ContractView {
    pub declarer: usize,
    pub level: u8,
    pub strain: &'static str,
    pub value: i32,
    pub label: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayView {
    pub seat: usize,
    pub card: CardView,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TrickView {
    pub leader: usize,
    pub plays: Vec<PlayView>,
    /// The suit named when the Joker was led in no trumps.
    pub named_suit: Option<char>,
    pub winner: Option<usize>,
}

fn trick_view(trick: &Trick, trump: Option<Suit>) -> TrickView {
    let named = trick.cards().first().filter(|c| c.is_joker() && trump.is_none()).and(trick.led_suit());
    TrickView {
        leader: trick.leader().index(),
        plays: trick.plays().map(|(seat, card)| PlayView { seat: seat.index(), card: card_view(card, trump) }).collect(),
        named_suit: named.map(Suit::letter),
        winner: trick.is_complete().then(|| trick.winner(trump)).flatten().map(Seat::index),
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayerView {
    pub seat: usize,
    pub name: String,
    pub persona: String,
    pub blurb: String,
}

/// One line of the score pad, plus (for played hands) everything needed to review the hand.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HandView {
    pub number: usize,
    pub dealer: usize,
    pub deal_number: u8,
    pub passed_out: bool,
    pub contract: Option<ContractView>,
    pub made: Option<bool>,
    pub tricks_won: [u8; 2],
    pub score_change: [i32; 2],
    pub scores_after: [i32; 2],
    pub calls: Vec<CallView>,
    /// All four hands as dealt, the kitty and the discards: empty for passed-out deals,
    /// which are thrown in unseen.
    pub dealt: Vec<Vec<CardView>>,
    pub kitty: Vec<CardView>,
    pub discards: Vec<CardView>,
    pub tricks: Vec<TrickView>,
}

fn hand_view(number: usize, record: &HandRecord) -> HandView {
    let trump = record.contract.and_then(|(_, c)| c.trump());
    let played = record.contract.is_some();
    HandView {
        number,
        dealer: record.dealer.index(),
        deal_number: record.deal_number,
        passed_out: !played,
        contract: record.contract.map(|(seat, c)| contract_view(seat, c)),
        made: record.contract.map(|(seat, c)| record.tricks_won[seat.team().index()] >= c.level),
        tricks_won: record.tricks_won,
        score_change: record.score_change,
        scores_after: record.scores_after,
        calls: record.calls.iter().map(|&(s, c)| call_view(s, c)).collect(),
        dealt: if played { record.dealt.iter().map(|&h| cards_view(h, trump)).collect() } else { vec![] },
        kitty: if played { cards_view(record.kitty, trump) } else { vec![] },
        discards: if played { cards_view(record.discards, trump) } else { vec![] },
        tricks: record.tricks.iter().map(|t| trick_view(t, trump)).collect(),
    }
}

fn contract_view(declarer: Seat, c: fh_core::Contract) -> ContractView {
    ContractView { declarer: declarer.index(), level: c.level, strain: strain_code(c.strain), value: c.value(), label: c.to_string() }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct View {
    /// "bidding", "discarding", "playing" or "gameOver".
    pub phase: &'static str,
    pub to_act: Option<usize>,
    pub human_to_act: bool,
    pub dealer: usize,
    pub deal_number: u8,
    pub dealer_forced: bool,
    pub must_bid: bool,
    pub scores: [i32; 2],
    pub target: i32,
    pub winner: Option<usize>,
    pub hand: Vec<CardView>,
    pub hand_counts: [usize; 4],
    pub calls: Vec<CallView>,
    pub contract: Option<ContractView>,
    pub kitty: Option<Vec<CardView>>,
    pub discards: Option<Vec<CardView>>,
    pub trick: Option<TrickView>,
    pub last_trick: Option<TrickView>,
    pub tricks_won: [u8; 2],
    pub legal: Vec<ActionView>,
    pub players: Vec<PlayerView>,
    pub history: Vec<HandView>,
}

impl View {
    pub fn build(table: &Table) -> View {
        let game = table.game();
        let obs = game.observe(HUMAN);
        let trump = obs.trump();
        let human_to_act = table.human_to_act();
        View {
            phase: match obs.phase {
                Phase::Bidding => "bidding",
                Phase::Discarding => "discarding",
                Phase::Playing => "playing",
                Phase::GameOver => "gameOver",
            },
            to_act: obs.to_act.map(Seat::index),
            human_to_act,
            dealer: obs.dealer.index(),
            deal_number: obs.deal_number,
            dealer_forced: obs.auction.dealer_forced(),
            must_bid: obs.auction.must_bid(),
            scores: obs.scores,
            target: obs.rules.target_score,
            winner: game.winner().map(|t| t.index()),
            hand: cards_view(obs.hand, trump),
            hand_counts: Seat::ALL.map(|s| game.hand(s).len()),
            calls: obs.auction.calls().map(|(s, c)| call_view(s, c)).collect(),
            contract: obs.contract.map(|(s, c)| contract_view(s, c)),
            kitty: obs.kitty.map(|k| cards_view(k, trump)),
            discards: obs.discards.map(|d| cards_view(d, trump)),
            trick: obs.current_trick.as_ref().map(|t| trick_view(t, trump)),
            last_trick: obs.tricks.last().map(|t| trick_view(t, trump)),
            tricks_won: obs.tricks_won,
            legal: if human_to_act {
                game.legal_actions().into_iter().enumerate().map(|(i, a)| action_view(i, a)).collect()
            } else {
                vec![]
            },
            players: Seat::ALL
                .iter()
                .map(|&seat| {
                    let id = &table.players()[seat.index()];
                    let p = persona(id);
                    PlayerView {
                        seat: seat.index(),
                        name: if seat == HUMAN { "You".into() } else { p.map_or(id.clone(), |p| p.name.to_string()) },
                        persona: id.clone(),
                        blurb: p.map_or(String::new(), |p| p.blurb.to_string()),
                    }
                })
                .collect(),
            history: game.history().iter().enumerate().map(|(i, h)| hand_view(i + 1, h)).collect(),
        }
    }
}

/// A short description of an event, for the table's message line and animation timing.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EventView {
    /// "dealt", "called", "passedOut", "contractWon", "discarded", "played", "trickWon",
    /// "handScored" or "gameWon".
    pub kind: &'static str,
    pub seat: Option<usize>,
    pub text: String,
}

pub fn event_view(event: &Event) -> EventView {
    let (kind, seat, text) = match event {
        Event::Dealt { dealer, deal_number, dealer_forced } => (
            "dealt",
            Some(*dealer),
            if *dealer_forced {
                format!("{dealer} deals (deal {deal_number} of 3; the dealer must bid if nobody else does)")
            } else {
                format!("{dealer} deals")
            },
        ),
        Event::Called { seat, call } => ("called", Some(*seat), format!("{seat}: {call}")),
        Event::PassedOut { dealer } => ("passedOut", Some(*dealer), format!("Nobody bid 7 or more. {dealer} redeals.")),
        Event::ContractWon { declarer, contract } => (
            "contractWon",
            Some(*declarer),
            format!("{declarer} wins the bid at {contract} ({} points)", contract.value()),
        ),
        Event::Discarded { seat } => ("discarded", Some(*seat), format!("{seat} discards")),
        Event::Played { seat, play } => ("played", Some(*seat), format!("{seat} plays {play}")),
        Event::TrickWon { winner, trick_number } => ("trickWon", Some(*winner), format!("{winner} wins trick {trick_number}")),
        Event::HandScored { declarer, contract, tricks_won, .. } => {
            let made = tricks_won[declarer.team().index()] >= contract.level;
            let verb = if made { "made" } else { "went down in" };
            ("handScored", Some(*declarer), format!("{} {verb} {contract}", declarer.team()))
        }
        Event::GameWon { team } => ("gameWon", None, format!("{team} win the game!")),
    };
    EventView { kind, seat: seat.map(Seat::index), text }
}
