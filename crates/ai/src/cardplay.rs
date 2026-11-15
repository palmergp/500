//! Card-play helpers shared by bots: card ordering, "boss" cards and how cheap a card is to give up.

use fh_core::rules::{is_trump, trump_power};
use fh_core::trick::trick_strength;
use fh_core::{effective_suit, Action, Card, CardSet, Observation, Suit};

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

/// Of cards that win exactly the same tricks, the one partner can read. When partner still plays
/// to this trick and `action`'s card is in a run of touching cards (with the cards already gone
/// taken out) topped by the boss, play the boss: the ace from A-K, not the king, so a partner who
/// is short in the suit sees the trick is already won and doesn't trump it.
pub fn show_winner(obs: &Observation, action: Action) -> Action {
    let Action::Play(play) = action else { return action };
    let Some(trick) = obs.current_trick.as_ref() else { return action };
    let partner = obs.seat.partner();
    if trick.plays().any(|(seat, _)| seat == partner) {
        return action;
    }
    let trump = obs.trump();
    let Some(suit) = effective_suit(play.card, trump) else { return action };
    let gone = obs.played().union(obs.discards.unwrap_or_default());
    let run: Vec<Card> = suit_order(suit, trump).into_iter().filter(|&c| !gone.contains(c)).take_while(|&c| obs.hand.contains(c)).collect();
    match run.first() {
        Some(&boss) if run.contains(&play.card) => Action::Play(fh_core::Play { card: boss, named_suit: None }),
        _ => action,
    }
}

/// Trumps in `cards` under `trump`.
pub fn trumps(cards: CardSet, trump: Option<Suit>) -> CardSet {
    cards.iter().filter(|&c| is_trump(c, trump)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Leading with A-K (or K-Q once the ace is gone) of a side suit, the bot shows the boss so
    /// partner can read it; once partner has played to the trick, nothing changes.
    #[test]
    fn touching_winners_show_the_boss_while_partner_is_still_to_play() {
        use fh_core::{Game, Phase, Play, Rules};
        let (mut leads, mut after_partner) = (0, 0);
        for seed in 0..300u64 {
            let mut game = Game::new(Rules::default(), seed);
            let mut bots: [Box<dyn crate::Bot>; 4] = std::array::from_fn(|i| {
                // Mostly the quick rule bot; a few deals of the search bot too.
                crate::make_bot(if seed % 30 == 0 { "counter" } else { "steady" }, seed * 4 + i as u64).unwrap()
            });
            while game.phase() != Phase::GameOver && game.history().is_empty() {
                let seat = game.to_act().unwrap();
                let obs = game.observe(seat);
                let legal = game.legal_actions();
                if obs.phase == Phase::Playing {
                    let trump = obs.trump();
                    let trick = obs.current_trick.clone().unwrap();
                    let gone = obs.played().union(obs.discards.unwrap_or_default());
                    for suit in Suit::ALL.into_iter().filter(|&s| Some(s) != trump) {
                        let live: Vec<Card> = suit_order(suit, trump).into_iter().filter(|&c| !gone.contains(c)).collect();
                        let (Some(&top), Some(&next)) = (live.first(), live.get(1)) else { continue };
                        let play = |c: Card| Action::Play(Play { card: c, named_suit: None });
                        if !obs.hand.contains(top) || !obs.hand.contains(next) || !legal.contains(&play(next)) {
                            continue;
                        }
                        let shown = show_winner(&obs, play(next));
                        if trick.plays().any(|(s, _)| s == seat.partner()) {
                            assert_eq!(shown, play(next), "partner has played: seed {seed}");
                            after_partner += 1;
                        } else {
                            assert_eq!(shown, play(top), "{next} and {top} touch: show {top} (seed {seed})");
                            leads += u32::from(trick.is_empty());
                        }
                    }
                }
                let action = bots[seat.index()].choose(&obs, &legal);
                if obs.phase == Phase::Playing {
                    assert_eq!(show_winner(&obs, action), action, "the bot played the lower of touching winners (seed {seed})");
                }
                game.apply(action).unwrap();
            }
        }
        assert!(leads > 20 && after_partner > 20, "{leads} leads, {after_partner} after partner");
    }

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
