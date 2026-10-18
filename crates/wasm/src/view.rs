//! JSON-friendly views of the table for the UI, from one player's point of view. A [`Lens`]
//! rotates every seat so the viewer is always seat 0 (bottom of the screen, "South") with their
//! partner at 2, swaps team pairs so the viewer's side is always team 0, and names the players.

use crate::table::Table;
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

/// How one player sees the table: their seat becomes index 0, their side team 0.
#[derive(Clone, Debug)]
pub struct Lens {
    pub viewer: Seat,
    /// Display names by real seat ("You" for the viewer).
    pub names: [String; 4],
}

impl Lens {
    pub fn new(viewer: Seat, names: [String; 4]) -> Lens {
        Lens { viewer, names }
    }

    /// The seat index as the viewer sees it.
    pub fn seat(&self, seat: Seat) -> usize {
        (seat.index() + 4 - self.viewer.index()) % 4
    }

    /// A per-team pair as the viewer sees it (their side first).
    pub fn teams<T: Copy>(&self, pair: [T; 2]) -> [T; 2] {
        if self.viewer.index() % 2 == 1 {
            [pair[1], pair[0]]
        } else {
            pair
        }
    }

    pub fn team(&self, team: fh_core::Team) -> usize {
        if team == self.viewer.team() {
            0
        } else {
            1
        }
    }

    /// A per-seat array reordered so index 0 is the viewer.
    pub fn seats<T: Clone>(&self, per_seat: &[T; 4]) -> [T; 4] {
        std::array::from_fn(|i| per_seat[(i + self.viewer.index()) % 4].clone())
    }

    fn is_viewer(&self, seat: Seat) -> bool {
        seat == self.viewer
    }

    fn name(&self, seat: Seat) -> &str {
        &self.names[seat.index()]
    }
}

fn call_view(seat: Seat, call: Call, lens: &Lens) -> CallView {
    let kind = match call {
        Call::Pass => "pass",
        Call::Mention(_) => "mention",
        Call::Six(_) => "six",
        Call::Bid(_) => "bid",
    };
    CallView { seat: lens.seat(seat), kind, label: call.to_string() }
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

fn trick_view(trick: &Trick, trump: Option<Suit>, lens: &Lens) -> TrickView {
    let named = trick.cards().first().filter(|c| c.is_joker() && trump.is_none()).and(trick.led_suit());
    TrickView {
        leader: lens.seat(trick.leader()),
        plays: trick.plays().map(|(seat, card)| PlayView { seat: lens.seat(seat), card: card_view(card, trump) }).collect(),
        named_suit: named.map(Suit::letter),
        winner: trick.is_complete().then(|| trick.winner(trump)).flatten().map(|s| lens.seat(s)),
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

pub fn hand_view(number: usize, record: &HandRecord, lens: &Lens) -> HandView {
    let trump = record.contract.and_then(|(_, c)| c.trump());
    let played = record.contract.is_some();
    HandView {
        number,
        dealer: lens.seat(record.dealer),
        deal_number: record.deal_number,
        passed_out: !played,
        contract: record.contract.map(|(seat, c)| contract_view(seat, c, lens)),
        made: record.contract.map(|(seat, c)| record.tricks_won[seat.team().index()] >= c.level),
        tricks_won: lens.teams(record.tricks_won),
        score_change: lens.teams(record.score_change),
        scores_after: lens.teams(record.scores_after),
        calls: record.calls.iter().map(|&(s, c)| call_view(s, c, lens)).collect(),
        dealt: if played { lens.seats(&record.dealt).iter().map(|&h| cards_view(h, trump)).collect() } else { vec![] },
        kitty: if played { cards_view(record.kitty, trump) } else { vec![] },
        discards: if played { cards_view(record.discards, trump) } else { vec![] },
        tricks: record.tricks.iter().map(|t| trick_view(t, trump, lens)).collect(),
    }
}

fn contract_view(declarer: Seat, c: fh_core::Contract, lens: &Lens) -> ContractView {
    ContractView { declarer: lens.seat(declarer), level: c.level, strain: strain_code(c.strain), value: c.value(), label: c.to_string() }
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
    /// Actions taken so far in the game: identifies a decision for the coach.
    pub action_count: usize,
    /// Every card played to a trick this hand, for the card tracker.
    pub played: Vec<CardView>,
    /// Trumps you have not seen (not in your hand, not played, not among your discards).
    pub trumps_unseen: Option<usize>,
    /// For each seat, the suits (♠ ♣ ♦ ♥, by effective suit) it has shown out of this hand.
    pub shown_out: [[bool; 4]; 4],
    /// For each suit (♠ ♣ ♦ ♥, by effective suit, so bowers count as trumps), cards you have
    /// not seen: not in your hand, not played, not among your discards.
    pub unseen_by_suit: [usize; 4],
    /// Whether you have not seen the Joker (in no trumps it belongs to no suit).
    pub joker_unseen: bool,
}

impl View {
    /// South's view of a local table.
    pub fn build(table: &Table) -> View {
        let names = std::array::from_fn(|i| {
            let id = &table.players()[i];
            if i == 0 {
                "You".to_string()
            } else {
                persona(id).map_or(id.clone(), |p| p.name.to_string())
            }
        });
        View::build_for(table.game(), &Lens::new(Seat::SOUTH, names), table.players(), table.action_count())
    }

    /// The table as `lens.viewer` sees it. `personas` are persona ids by real seat ("human" for
    /// people), `action_count` the number of actions taken so far.
    pub fn build_for(game: &fh_core::Game, lens: &Lens, personas: &[String; 4], action_count: usize) -> View {
        let viewer = lens.viewer;
        let obs = game.observe(viewer);
        let trump = obs.trump();
        let human_to_act = game.to_act() == Some(viewer);
        View {
            phase: match obs.phase {
                Phase::Bidding => "bidding",
                Phase::Discarding => "discarding",
                Phase::Playing => "playing",
                Phase::GameOver => "gameOver",
            },
            to_act: obs.to_act.map(|s| lens.seat(s)),
            human_to_act,
            dealer: lens.seat(obs.dealer),
            deal_number: obs.deal_number,
            dealer_forced: obs.auction.dealer_forced(),
            must_bid: obs.auction.must_bid(),
            scores: lens.teams(obs.scores),
            target: obs.rules.target_score,
            winner: game.winner().map(|t| lens.team(t)),
            hand: cards_view(obs.hand, trump),
            hand_counts: lens.seats(&Seat::ALL.map(|s| game.hand(s).len())),
            calls: obs.auction.calls().map(|(s, c)| call_view(s, c, lens)).collect(),
            contract: obs.contract.map(|(s, c)| contract_view(s, c, lens)),
            kitty: obs.kitty.map(|k| cards_view(k, trump)),
            discards: obs.discards.map(|d| cards_view(d, trump)),
            trick: obs.current_trick.as_ref().map(|t| trick_view(t, trump, lens)),
            last_trick: obs.tricks.last().map(|t| trick_view(t, trump, lens)),
            tricks_won: lens.teams(obs.tricks_won),
            legal: if human_to_act {
                game.legal_actions().into_iter().enumerate().map(|(i, a)| action_view(i, a)).collect()
            } else {
                vec![]
            },
            players: (0..4)
                .map(|i| {
                    let seat = Seat::new(i + viewer.index());
                    let id = &personas[seat.index()];
                    let p = persona(id);
                    PlayerView {
                        seat: i,
                        name: lens.name(seat).to_string(),
                        persona: id.clone(),
                        blurb: p.map_or(String::new(), |p| p.blurb.to_string()),
                    }
                })
                .collect(),
            history: game.history().iter().enumerate().map(|(i, h)| hand_view(i + 1, h, lens)).collect(),
            action_count,
            played: cards_view(obs.played(), trump),
            trumps_unseen: trump.map(|_| {
                let seen = obs.hand.union(obs.played()).union(obs.discards.unwrap_or_default());
                CardSet::pack().minus(seen).iter().filter(|&c| is_trump(c, trump)).count()
            }),
            shown_out: lens.seats(&shown_out(&obs)),
            unseen_by_suit: {
                let seen = obs.hand.union(obs.played()).union(obs.discards.unwrap_or_default());
                let unseen = CardSet::pack().minus(seen);
                Suit::ALL.map(|s| unseen.iter().filter(|&c| fh_core::effective_suit(c, trump) == Some(s)).count())
            },
            joker_unseen: !obs.hand.union(obs.played()).union(obs.discards.unwrap_or_default()).contains(Card::JOKER),
        }
    }
}

/// The suits each seat has failed to follow this hand (public information).
fn shown_out(obs: &fh_core::Observation) -> [[bool; 4]; 4] {
    let trump = obs.trump();
    let mut out = [[false; 4]; 4];
    for trick in obs.tricks.iter().chain(obs.current_trick.iter()) {
        let Some(led) = trick.led_suit() else { continue };
        for (seat, card) in trick.plays() {
            if seat != trick.leader() && fh_core::effective_suit(card, trump) != Some(led) {
                out[seat.index()][led.index()] = true;
            }
        }
    }
    out
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

pub fn event_view(event: &Event, lens: &Lens) -> EventView {
    // "You play" / "The Rock plays".
    let who = |seat: &Seat, you: &str, they: &str| -> String {
        if lens.is_viewer(*seat) {
            format!("You {you}")
        } else {
            format!("{} {they}", lens.name(*seat))
        }
    };
    let (kind, seat, text) = match event {
        Event::Dealt { dealer, deal_number, dealer_forced } => (
            "dealt",
            Some(*dealer),
            if *dealer_forced {
                format!("{} (deal {deal_number} of 3; the dealer must bid if nobody else does)", who(dealer, "deal", "deals"))
            } else {
                who(dealer, "deal", "deals")
            },
        ),
        Event::Called { seat, call } => ("called", Some(*seat), format!("{}: {call}", lens.name(*seat))),
        Event::PassedOut { dealer } => ("passedOut", Some(*dealer), format!("Nobody bid 7 or more. {}.", who(dealer, "redeal", "redeals"))),
        Event::ContractWon { declarer, contract } => (
            "contractWon",
            Some(*declarer),
            format!("{} at {contract} ({} points)", who(declarer, "win the bid", "wins the bid"), contract.value()),
        ),
        Event::Discarded { seat } => ("discarded", Some(*seat), who(seat, "discard", "discards")),
        Event::Played { seat, play } => ("played", Some(*seat), format!("{} {play}", who(seat, "play", "plays"))),
        Event::TrickWon { winner, trick_number } => ("trickWon", Some(*winner), format!("{} {trick_number}", who(winner, "win trick", "wins trick"))),
        Event::HandScored { declarer, contract, tricks_won, .. } => {
            let made = tricks_won[declarer.team().index()] >= contract.level;
            let verb = if made { "made" } else { "went down in" };
            let side = if declarer.team() == lens.viewer.team() { "Your side" } else { "They" };
            ("handScored", Some(*declarer), format!("{side} {verb} {contract}"))
        }
        Event::GameWon { team } => (
            "gameWon",
            None,
            if *team == lens.viewer.team() { "Your side wins the game!".to_string() } else { "They win the game.".to_string() },
        ),
    };
    EventView { kind, seat: seat.map(|s| lens.seat(s)), text }
}
