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
