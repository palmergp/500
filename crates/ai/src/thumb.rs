//! The rule-of-thumb hand count from docs/rules-of-thumb.md (rules 1, 4 and 5), so the coach
//! explains bids with the same numbers the player learns. The values come from
//! `train/rules_of_thumb.py` (solver data), rounded to easy fractions.

use fh_core::cards::{ACE, JACK, KING, QUEEN};
use fh_core::rules::is_trump;
use fh_core::{effective_suit, Call, Card, Observation, Strain, Suit};

/// The count for `strain` and its parts, largest first.
pub fn count(obs: &Observation, strain: Strain) -> (f32, Vec<(String, f32)>) {
    let mut parts: Vec<(String, f32)> = Vec::new();
    let hand = obs.hand;
    let mut add = |label: String, value: f32| {
        if value != 0.0 {
            parts.push((label, value));
        }
    };
    match strain.trump() {
        Some(t) => {
            let trump = Some(t);
            let has = |c: Card| hand.contains(c);
            let honours = [
                (Card::JOKER, "Joker", 1.5),
                (Card::new(t, JACK), "right bower", 1.25),
                (Card::new(t.same_color(), JACK), "left bower", 1.0),
                (Card::new(t, ACE), "ace of trumps", 0.75),
                (Card::new(t, KING), "king of trumps", 0.67),
                (Card::new(t, QUEEN), "queen of trumps", 0.5),
            ];
            for (card, name, value) in honours {
                if has(card) {
                    add(name.to_string(), value);
                }
            }
            let small = hand.iter().filter(|&c| is_trump(c, trump)).count() - honours.iter().filter(|h| has(h.0)).count();
            add(format!("{small} other trump{}", if small == 1 { "" } else { "s" }), 0.5 * small as f32);
            let mut side_aces = 0;
            // Side kings by how they're held (finding 25): beside the ace, with two or more guards, with one.
            let (mut ace_kings, mut kings, mut thin_kings) = (0, 0, 0);
            let mut voids = 0;
            let mut singles = 0;
            for s in Suit::ALL.into_iter().filter(|&s| s != t) {
                let cards: Vec<Card> = hand.iter().filter(|&c| effective_suit(c, trump) == Some(s)).collect();
                side_aces += cards.iter().filter(|c| c.rank() == ACE).count();
                if cards.iter().any(|c| c.rank() == KING) {
                    match (cards.iter().any(|c| c.rank() == ACE), cards.len()) {
                        (true, _) => ace_kings += 1,
                        (false, 1) => {}
                        (false, 2) => thin_kings += 1,
                        (false, _) => kings += 1,
                    }
                }
                voids += cards.is_empty() as usize;
                singles += (cards.len() == 1) as usize;
            }
            add(plural(side_aces, "side ace"), 0.67 * side_aces as f32);
            add(plural_with(ace_kings, "side king", "beside its ace"), 0.33 * ace_kings as f32);
            add(plural_with(kings, "side king", "with 2+ guards"), 0.2 * kings as f32);
            add(plural_with(thin_kings, "side king", "with one guard"), 0.125 * thin_kings as f32);
            add(plural(voids, "side void"), 0.2 * voids as f32);
            add(plural(singles, "singleton"), 0.125 * singles as f32);
            add("partner and kitty".into(), 3.0);
        }
        None => {
            if hand.contains(Card::JOKER) {
                add("Joker".into(), 2.25);
            }
            let mut aces = 0;
            let mut kings = 0;
            let mut thin_kings = 0;
            let mut queens = 0;
            let mut long = 0;
            for s in Suit::ALL {
                let cards = hand.of_printed_suit(s);
                let len = cards.len();
                let has = |r: u8| cards.contains(Card::new(s, r));
                aces += has(ACE) as usize;
                // A king with a single small guard is worth less (finding 25); beside the ace it's a full ½.
                if has(KING) && len >= 2 {
                    if len == 2 && !has(ACE) {
                        thin_kings += 1;
                    } else {
                        kings += 1;
                    }
                }
                queens += (has(QUEEN) && len >= 3) as usize;
                if has(ACE) {
                    long += len.saturating_sub(3);
                }
            }
            add(plural(aces, "ace"), 1.0 * aces as f32);
            add(plural(kings, "guarded king"), 0.5 * kings as f32);
            add(plural_with(thin_kings, "king", "with one guard"), 0.33 * thin_kings as f32);
            add(plural(queens, "guarded queen"), 0.25 * queens as f32);
            add(plural(long, "long card"), 0.125 * long as f32);
            add("partner and kitty".into(), 3.33);
        }
    }
    for (label, value) in auction_adjustments(obs, strain) {
        add(label, value);
    }
    let total = parts.iter().map(|p| p.1).sum();
    parts.sort_by(|a, b| b.1.abs().total_cmp(&a.1.abs()));
    (total, parts)
}

fn plural(n: usize, what: &str) -> String {
    match n {
        1 => format!("1 {what}"),
        _ => format!("{n} {what}s"),
    }
}

/// "2 side kings with one guard": the noun takes the plural, not the tail.
fn plural_with(n: usize, what: &str, tail: &str) -> String {
    format!("{} {tail}", plural(n, what))
}

/// Rule 4: what the other calls are worth for `strain`.
fn auction_adjustments(obs: &Observation, strain: Strain) -> Vec<(String, f32)> {
    let nt = strain == Strain::NoTrumps;
    let me = obs.seat;
    let mut out = Vec::new();
    let mut opponents_showed = false;
    for (seat, call) in obs.auction.calls() {
        if seat == me {
            continue;
        }
        let partner = seat == me.partner();
        match (partner, call) {
            (true, Call::Six(_)) => out.push(("partner's 6-call".into(), if nt { 1.0 } else { 0.75 })),
            (true, Call::Mention(s)) if s == strain => out.push(("partner's mention".into(), if nt { 0.7 } else { 0.6 })),
            (true, Call::Bid(c)) if c.strain == strain => out.push(("partner's bid".into(), 2.25)),
            (true, Call::Pass) => out.push(("partner passed".into(), if nt { -1.3 } else { -0.9 })),
            (false, Call::Six(_)) => {
                out.push(("an opponent's 6-call".into(), if nt { -0.7 } else { -0.5 }));
                opponents_showed = true;
            }
            (false, Call::Mention(s)) if s == strain => {
                out.push(("an opponent's mention".into(), if nt { -0.7 } else { -0.67 }));
                opponents_showed = true;
            }
            (false, Call::Bid(c)) if c.strain == strain => {
                out.push(("an opponent's bid".into(), -2.0));
                opponents_showed = true;
            }
            _ => {}
        }
    }
    let dealer = obs.dealer == me;
    if dealer && !opponents_showed {
        out.push(("speaking last".into(), if nt { 0.6 } else { 0.5 }));
    }
    out
}

/// A fraction-friendly rendering of a trick value: 1½, ¾, ⅔ …
pub fn fraction(value: f32) -> String {
    let sign = if value < 0.0 { "−" } else { "" };
    let v = value.abs();
    let whole = v.floor() as i32;
    let frac = v - whole as f32;
    let symbol = [(0.125, "⅛"), (0.2, "⅕"), (0.25, "¼"), (0.33, "⅓"), (0.5, "½"), (0.6, "0.6"), (0.67, "⅔"), (0.7, "0.7"), (0.75, "¾"), (0.9, "0.9")]
        .iter()
        .find(|(f, _)| (frac - f).abs() < 0.02)
        .map(|(_, s)| *s);
    match (whole, symbol, frac < 0.02) {
        (w, _, true) => format!("{sign}{w}"),
        (0, Some(s), _) => format!("{sign}{s}"),
        (w, Some(s), _) if s.starts_with('0') => format!("{sign}{:.1}", w as f32 + frac),
        (w, Some(s), _) => format!("{sign}{w}{s}"),
        (_, None, _) => format!("{sign}{v:.1}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fh_core::{Auction, CardSet, Phase, Rules, Seat};

    #[test]
    fn counts_a_strong_heart_hand() {
        let obs = Observation {
            seat: Seat::SOUTH,
            phase: Phase::Bidding,
            to_act: Some(Seat::SOUTH),
            rules: Rules::default(),
            scores: [0, 0],
            dealer: Seat::EAST,
            deal_number: 1,
            hand: CardSet::parse("Joker JH 9H 7H 5H AS 8S 6C 5C KD").unwrap(),
            auction: Auction::new(Seat::EAST, false),
            contract: None,
            kitty: None,
            discards: None,
            tricks: vec![],
            current_trick: None,
            tricks_won: [0, 0],
            hands_played: 0,
        };
        let (total, parts) = count(&obs, Strain::Hearts);
        // Joker 1.5 + right bower 1.25 + 3 small trumps 1.5 + side ace 0.67 + K♦ singleton 0.125 + 3.
        assert!((total - 8.045).abs() < 0.01, "{total} {parts:?}");
        assert_eq!(fraction(1.5), "1½");
        assert_eq!(fraction(0.67), "⅔");
        assert_eq!(fraction(-0.9), "−0.9");
        assert_eq!(fraction(3.0), "3");
    }
}
