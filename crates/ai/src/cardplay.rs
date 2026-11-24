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

/// Whether this is the declarer's opening lead in no trumps, holding the Joker and other cards.
/// Search likes cashing the Joker here (in each sampled deal it then plays perfectly, needing no
/// stopper), but at the table it cost declarers 0.31 tricks a time and the bid was made 48% of
/// the time against 58% with the search's best other lead (docs/lab/README.md, finding 54).
pub fn opening_joker_lead(obs: &Observation) -> bool {
    obs.trump().is_none()
        && obs.contract.is_some()
        && obs.tricks.is_empty()
        && obs.current_trick.as_ref().is_some_and(|t| t.cards().is_empty())
        && obs.hand.contains(Card::JOKER)
        && obs.hand.len() > 1
}

/// How much worse than the best other lead an opening Joker lead in no trumps counts: the bid's
/// chance falls by 10 points in a hundred (finding 54), so 0.10 of the swing between making the
/// bid and going down one, in `utility`'s units.
pub const OPENING_JOKER_CHANCE: f64 = 0.10;

/// A Joker lead in no trumps by the declaring side in the first three tricks, holding other cards:
/// how much of the swing between making the bid and going down one it counts as costing (the
/// search rates these leads well above what they take at the table; findings 54 and 56). None
/// elsewhere: from trick 4 the search decides (tricks 4–6 came out even, 7 on the Joker lead pays).
pub fn early_joker_lead(obs: &Observation) -> Option<f64> {
    if opening_joker_lead(obs) {
        return Some(OPENING_JOKER_CHANCE);
    }
    let (declarer, _) = obs.contract?;
    let leading = obs.current_trick.as_ref().is_some_and(|t| t.cards().is_empty());
    (obs.trump().is_none()
        && declarer.team() == obs.seat.team()
        && obs.tricks.len() < 3
        && leading
        && obs.hand.contains(Card::JOKER)
        && obs.hand.len() > 1)
        .then_some(EARLY_JOKER_CHANCE)
}

/// Tricks 2–3: the bid made about a twentieth less often with the Joker lead (finding 56).
pub const EARLY_JOKER_CHANCE: f64 = 0.05;

/// Early in a suit bid (tricks 1–3), the declaring side on lead, holding the top trump left while
/// trumps may still be out: that top trump. From here, leading it or cashing a side ace came out
/// even when both were played out (finding 55), though the search usually rates the ace higher.
pub fn early_top_trump(obs: &Observation) -> Option<Card> {
    let (declarer, contract) = obs.contract?;
    let suit = contract.trump()?;
    let leading = obs.current_trick.as_ref().is_none_or(|t| t.cards().is_empty());
    if obs.seat.team() != declarer.team() || obs.tricks.len() >= 3 || !leading {
        return None;
    }
    let played = obs.played();
    let order = suit_order(suit, Some(suit));
    let top = order.iter().copied().find(|&c| !played.contains(c))?;
    let seen = played.union(obs.hand).union(obs.discards.unwrap_or_default());
    (obs.hand.contains(top) && order.iter().any(|&c| !seen.contains(c))).then_some(top)
}

/// Whether `a` and `b` are the top trump and a side ace in [`early_top_trump`]'s position, which
/// do about equally well: the coach grades either as good when it prefers the other.
pub fn ace_or_top_trump(obs: &Observation, a: Action, b: Action) -> bool {
    let Some(top) = early_top_trump(obs) else { return false };
    let trump = obs.trump();
    let card = |x: Action| match x {
        Action::Play(p) => Some(p.card),
        _ => None,
    };
    let side_ace = |c: Card| !c.is_joker() && !is_trump(c, trump) && c.rank() == fh_core::cards::ACE;
    match (card(a), card(b)) {
        (Some(x), Some(y)) => (x == top && side_ace(y)) || (y == top && side_ace(x)),
        _ => false,
    }
}

/// Whether `seat` has shown out of `suit` this hand: failed to follow when it was led.
pub fn shown_out(obs: &Observation, seat: fh_core::Seat, suit: Suit) -> bool {
    let trump = obs.trump();
    obs.tricks.iter().chain(obs.current_trick.iter()).any(|trick| {
        trick.led_suit() == Some(suit) && trick.plays().any(|(s, c)| s == seat && s != trick.leader() && effective_suit(c, trump) != Some(suit))
    })
}

/// Whether the opponents may still hold a trump: some trump is unseen, and they haven't both
/// shown out of trumps. (Trumps only partner can hold don't count: drawing them pulls partner's.)
pub fn opponents_may_hold_trumps(obs: &Observation, seen: CardSet) -> bool {
    let Some(suit) = obs.trump() else { return false };
    let unseen = trumps(CardSet::pack(), Some(suit)).minus(seen);
    let [left, right] = [obs.seat.next(), obs.seat.partner().next()];
    !unseen.is_empty() && !(shown_out(obs, left, suit) && shown_out(obs, right, suit))
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
