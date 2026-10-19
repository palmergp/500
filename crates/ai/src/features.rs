//! Neural-network inputs, defined once here and used both to write training data and at
//! inference time, so training and play can never disagree about what an input means.

use fh_core::{Call, Card, CardSet, Observation, Seat, Strain};

/// Hand (43) + speaking position (4) + each other seat's call so far (3 × 32) + third-deal flag.
pub const BID_FEATURES: usize = 43 + 4 + 3 * CALL_SLOTS + 1;
/// For each strain, a distribution over 0–10 tricks for the bidder's side as declarer.
pub const BID_OUTPUTS: usize = 5 * 11;

/// No call yet, pass, mention × 5 strains, 6 × 5 strains, bid 7–10 × 5 strains.
const CALL_SLOTS: usize = 1 + 1 + 5 + 5 + 20;

/// Position of `card` in the 43-card pack (pack iteration order).
pub fn card_index(card: Card) -> usize {
    CardSet::pack().iter().position(|c| c == card).expect("card in the pack")
}

fn pack_index_table() -> [u8; 64] {
    let mut table = [255u8; 64];
    for (i, card) in CardSet::pack().iter().enumerate() {
        table[card.id() as usize] = i as u8;
    }
    table
}

fn call_slot(call: Option<Call>) -> usize {
    match call {
        None => 0,
        Some(Call::Pass) => 1,
        Some(Call::Mention(s)) => 2 + s.index(),
        Some(Call::Six(s)) => 7 + s.index(),
        Some(Call::Bid(c)) => 12 + (c.level as usize - 7) * 5 + c.strain.index(),
    }
}

/// The bidding inputs for the seat in `obs`.
pub fn bid_features(obs: &Observation) -> [f32; BID_FEATURES] {
    let mut x = [0f32; BID_FEATURES];
    let table = pack_index_table();
    for card in obs.hand.iter() {
        x[table[card.id() as usize] as usize] = 1.0;
    }
    let me = obs.seat;
    let position = (me.index() + 4 - obs.dealer.index() - 1) % 4;
    x[43 + position] = 1.0;
    let calls: Vec<(Seat, Call)> = obs.auction.calls().collect();
    for (k, offset) in [1usize, 2, 3].into_iter().enumerate() {
        let other = Seat::new(me.index() + offset);
        let call = calls.iter().find(|(s, _)| *s == other).map(|(_, c)| *c);
        x[47 + k * CALL_SLOTS + call_slot(call)] = 1.0;
    }
    x[BID_FEATURES - 1] = (obs.deal_number >= 3) as u8 as f32;
    x
}

/// Index of a strain's block of 11 outputs.
pub fn strain_block(strain: Strain) -> usize {
    strain.index() * 11
}

#[cfg(test)]
mod tests {
    use super::*;
    use fh_core::{Action, Game, Rules};

    #[test]
    fn encodes_hand_position_and_calls() {
        let mut game = Game::with_dealer(Rules::default(), 4, Seat::EAST);
        game.apply(Action::Call(Call::Six(Strain::Hearts))).unwrap(); // South
        let obs = game.observe(Seat::WEST);
        let x = bid_features(&obs);
        assert_eq!(x[..43].iter().sum::<f32>(), 10.0, "ten cards");
        assert_eq!(x[43 + 1], 1.0, "West speaks second");
        // West's partner is East; South sits on West's right.
        let right = 47 + 2 * CALL_SLOTS;
        assert_eq!(x[right + 7 + Strain::Hearts.index()], 1.0, "right-hand opponent called 6♥");
        assert_eq!(x[47], 1.0, "left-hand opponent (North) hasn't called");
        assert_eq!(x.iter().sum::<f32>(), 10.0 + 1.0 + 3.0);
        assert_eq!(card_index(Card::JOKER), 42);
    }
}

// ---- Card play ------------------------------------------------------------------------------

/// Card slots for the play network: slot 0 is the Joker, then four blocks of 12. Block 0 is the
/// trump suit ordered by strength (right bower, left bower, A, K, Q, 10 … 4); blocks 1–3 are the
/// side suits (the trump suit's same-colour partner first), each A, K, Q, J, 10 … 4. In no trumps
/// the four suits fill the blocks in ♠ ♣ ♦ ♥ order. Ordering cards by their role rather than their
/// printed suit lets what's learned in one trump suit carry over to the others.
pub const PLAY_SLOTS: usize = 49;
/// Inputs: your hand, each seat's played cards, the current trick by seat, cards known out (your
/// discards), shown-out suits, the contract, tricks won and the suit led; then what a player sees at
/// a glance: which of your cards would win the trick now, who is winning it, how many unseen cards
/// remain in each block, how many you hold in each, and whether the Joker is still out.
pub const PLAY_FEATURES: usize = PLAY_SLOTS * 11 + 16 + 9 + 3 + 5 + 4 + 4 + 4 + 1;

/// The block order of the four suits for `trump`.
fn blocks(trump: Option<fh_core::Suit>) -> [fh_core::Suit; 4] {
    use fh_core::Suit;
    match trump {
        None => Suit::ALL,
        Some(t) => {
            let partner = t.same_color();
            let mut others = Suit::ALL.into_iter().filter(|&s| s != t && s != partner);
            [t, partner, others.next().unwrap(), others.next().unwrap()]
        }
    }
}

/// A card's slot (0–48) for the play network.
pub fn play_slot(card: Card, trump: Option<fh_core::Suit>) -> usize {
    use fh_core::cards::{ACE, JACK};
    if card.is_joker() {
        return 0;
    }
    let order = blocks(trump);
    let suit = fh_core::effective_suit(card, trump).expect("a suited card");
    let block = order.iter().position(|&s| s == suit).expect("a suit");
    let within = match trump {
        Some(t) if suit == t => {
            if card == Card::new(t, JACK) {
                0
            } else if card.rank() == JACK {
                1 // the left bower
            } else {
                2 + (ACE - card.rank()) as usize - usize::from(card.rank() < JACK)
            }
        }
        _ => (ACE - card.rank()) as usize,
    };
    1 + 12 * block + within
}

/// What `obs.seat` can see at a card-play decision, for the play network.
pub fn play_features(obs: &Observation) -> Vec<f32> {
    let trump = obs.trump();
    let me = obs.seat;
    let rel = |s: Seat| (s.index() + 4 - me.index()) % 4;
    let mut x = vec![0.0f32; PLAY_FEATURES];
    let set = |x: &mut Vec<f32>, base: usize, cards: &mut dyn Iterator<Item = Card>| {
        for c in cards {
            x[base + play_slot(c, trump)] = 1.0;
        }
    };
    set(&mut x, 0, &mut obs.hand.iter());
    let order = blocks(trump);
    let mut voids = [[false; 4]; 4];
    for trick in obs.tricks.iter().chain(obs.current_trick.iter()) {
        let led = trick.led_suit();
        for (seat, card) in trick.plays() {
            x[PLAY_SLOTS * (1 + rel(seat)) + play_slot(card, trump)] = 1.0;
            if let Some(led) = led {
                if seat != trick.leader() && fh_core::effective_suit(card, trump) != Some(led) {
                    voids[rel(seat)][order.iter().position(|&s| s == led).unwrap()] = true;
                }
            }
        }
    }
    if let Some(trick) = &obs.current_trick {
        for (seat, card) in trick.plays() {
            x[PLAY_SLOTS * (5 + rel(seat)) + play_slot(card, trump)] = 1.0;
        }
    }
    set(&mut x, PLAY_SLOTS * 9, &mut obs.discards.unwrap_or_default().iter());
    let mut at = PLAY_SLOTS * 10;
    for seat_voids in voids {
        for v in seat_voids {
            x[at] = f32::from(u8::from(v));
            at += 1;
        }
    }
    if let Some((declarer, contract)) = obs.contract {
        x[at] = f32::from(u8::from(trump.is_none()));
        x[at + 1 + (contract.level.clamp(7, 10) - 7) as usize] = 1.0;
        x[at + 5 + rel(declarer)] = 1.0;
    }
    at += 9;
    let us = me.team().index();
    x[at] = obs.tricks_won[us] as f32 / 10.0;
    x[at + 1] = obs.tricks_won[1 - us] as f32 / 10.0;
    x[at + 2] = obs.tricks.len() as f32 / 10.0;
    at += 3;
    match obs.current_trick.as_ref().and_then(|t| t.led_suit()) {
        Some(led) => x[at + order.iter().position(|&s| s == led).unwrap()] = 1.0,
        None => x[at + 4] = 1.0,
    }
    at += 5;
    // At a glance: which of my cards would win the trick now (any card wins an empty trick's lead
    // for now, so leave those blank), and who is winning it.
    let base = at;
    at += PLAY_SLOTS;
    if let Some(trick) = obs.current_trick.as_ref().filter(|t| !t.is_empty()) {
        let led = trick.led_suit().expect("a card was led");
        let top = trick.winning_card(trump).map_or(0, |c| fh_core::trick::trick_strength(c, led, trump));
        for card in obs.hand.iter() {
            if fh_core::trick::trick_strength(card, led, trump) > top {
                x[base + play_slot(card, trump)] = 1.0;
            }
        }
        if let Some(winner) = trick.winner(trump) {
            x[at + rel(winner)] = 1.0;
        }
    }
    at += 4;
    // Unseen cards per block (not in my hand, not played, not among my discards), and my holding.
    let seen = obs.hand.union(obs.played()).union(obs.discards.unwrap_or_default());
    let block_of = |c: Card| (play_slot(c, trump).max(1) - 1) / 12;
    for card in CardSet::pack().minus(seen).iter().filter(|c| !c.is_joker() || trump.is_some()) {
        x[at + block_of(card)] += 1.0 / 12.0;
    }
    at += 4;
    for card in obs.hand.iter().filter(|c| !c.is_joker() || trump.is_some()) {
        x[at + block_of(card)] += 1.0 / 12.0;
    }
    at += 4;
    x[at] = f32::from(u8::from(!seen.contains(Card::JOKER)));
    x
}

#[cfg(test)]
mod play_tests {
    use super::*;
    use fh_core::{CardSet, Suit};

    /// Every card of the pack gets its own slot, for each trump suit and no trumps.
    #[test]
    fn play_slots_are_distinct() {
        for trump in [None, Some(Suit::Spades), Some(Suit::Clubs), Some(Suit::Diamonds), Some(Suit::Hearts)] {
            let mut seen = std::collections::HashSet::new();
            for card in CardSet::pack().iter() {
                let slot = play_slot(card, trump);
                assert!(slot < PLAY_SLOTS);
                assert!(seen.insert(slot), "{card} shares slot {slot} with trump {trump:?}");
            }
        }
        // The bowers lead the trump block.
        assert_eq!(play_slot(Card::parse("JH").unwrap(), Some(Suit::Hearts)), 1);
        assert_eq!(play_slot(Card::parse("JD").unwrap(), Some(Suit::Hearts)), 2);
        assert_eq!(play_slot(Card::parse("AH").unwrap(), Some(Suit::Hearts)), 3);
    }
}
