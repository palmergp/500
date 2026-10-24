//! The rule-of-thumb hand count from docs/rules-of-thumb.md (rules 1, 4 and 5), so the coach
//! explains bids with the same numbers the player learns. The values come from
//! `train/rules_of_thumb.py` (solver data), rounded to easy fractions.

use fh_core::cards::{ACE, JACK, KING, QUEEN};
use fh_core::rules::is_trump;
use fh_core::{effective_suit, Call, Card, CardSet, Observation, Strain, Suit};

/// What the count adds for partner and the kitty (and everything else it doesn't name): the fit's
/// constant.
pub fn constant(strain: Strain) -> f32 {
    if strain == Strain::NoTrumps {
        3.33
    } else {
        3.0
    }
}

/// The count of `hand`'s own cards in `strain`: without the constant or the auction.
pub fn card_count(hand: CardSet, strain: Strain) -> f32 {
    card_parts(hand, strain).iter().map(|p| p.1).sum()
}

/// The count for `strain` and its parts, largest first.
pub fn count(obs: &Observation, strain: Strain) -> (f32, Vec<(String, f32)>) {
    let mut parts = card_parts(obs.hand, strain);
    parts.push(("partner and kitty".into(), constant(strain)));
    for (label, value) in auction_adjustments(obs, strain) {
        if value != 0.0 {
            parts.push((label, value));
        }
    }
    let total = parts.iter().map(|p| p.1).sum();
    parts.sort_by(|a, b| b.1.abs().total_cmp(&a.1.abs()));
    (total, parts)
}

/// The values of the cards themselves.
fn card_parts(hand: CardSet, strain: Strain) -> Vec<(String, f32)> {
    let mut parts: Vec<(String, f32)> = Vec::new();
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
            // A side ace with no other card in its suit is worth less than an ace plus a singleton
            // (finding 42).
            let bare_aces = Suit::ALL
                .into_iter()
                .filter(|&s| s != t)
                .filter(|&s| {
                    let cards: Vec<Card> = hand.iter().filter(|&c| effective_suit(c, trump) == Some(s)).collect();
                    cards.len() == 1 && cards[0].rank() == ACE
                })
                .count();
            add(plural(bare_aces, "bare side ace"), -0.25 * bare_aces as f32);
            add(plural_with(ace_kings, "side king", "beside its ace"), 0.33 * ace_kings as f32);
            add(plural_with(kings, "side king", "with 2+ guards"), 0.2 * kings as f32);
            add(plural_with(thin_kings, "side king", "with one guard"), 0.125 * thin_kings as f32);
            add(plural(voids, "side void"), 0.2 * voids as f32);
            add(plural(singles, "singleton"), 0.125 * singles as f32);
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
            // The Joker and the aces overlap: some of the tricks they'd each take are the same ones.
            if hand.contains(Card::JOKER) && aces >= 2 {
                add("Joker with 2+ aces".into(), -0.25);
            }
        }
    }
    parts
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
    let left_bower_suit = strain.trump().map(|t| Strain::from_suit(t.same_color()));
    for (seat, call) in obs.auction.calls() {
        if seat == me {
            continue;
        }
        let partner = seat == me.partner();
        // Values from `train/rules_of_thumb.py` (findings 27, 31 and 37), against partner not having
        // called yet: a 6-call names the caller's best strain, so in yours it says much more, and in
        // your left bower's suit more than in the other two; a mention of another strain says the
        // mentioner's strength is elsewhere (partner's is a minus, an opponent's a small plus).
        let mut push = |label: &str, value: f32| out.push((label.to_string(), value));
        match (partner, call) {
            (true, Call::Six(_)) if nt => push("partner's 6-call", 0.8),
            (true, Call::Six(s)) if s == strain => push("partner's 6-call in it", 1.33),
            (true, Call::Six(s)) if Some(s) == left_bower_suit => push("partner's 6-call in the left bower's suit", 0.6),
            (true, Call::Six(_)) => push("partner's 6-call", 0.25),
            (true, Call::Mention(s)) if s == strain => push("partner's mention", 0.4),
            (true, Call::Mention(_)) if nt => push("partner's mention of a suit", -1.125),
            (true, Call::Mention(Strain::NoTrumps)) => push("partner's NT mention", -0.25),
            (true, Call::Mention(_)) => push("partner's mention of another suit", -0.67),
            (true, Call::Bid(c)) if c.strain == strain => push("partner's bid", 2.25),
            // Holding the Joker, you know partner didn't pass for lack of it: the pass says less.
            (true, Call::Pass) if obs.hand.contains(Card::JOKER) => push("partner passed (you hold the Joker)", if nt { -1.35 } else { -0.75 }),
            (true, Call::Pass) => push("partner passed", if nt { -1.6 } else { -1.0 }),
            // Partner bid another strain: a strong hand, which helps yours too (finding 42).
            (true, Call::Bid(_)) => push("partner's bid in another strain", if nt { 1.0 } else { 0.5 }),
            (false, Call::Six(s)) => {
                if nt {
                    push("an opponent's 6-call", -0.4);
                } else if s == strain {
                    push("an opponent's 6-call in it", -1.0);
                } else if Some(s) == left_bower_suit {
                    push("an opponent's 6-call in the left bower's suit", -0.4);
                } else {
                    push("an opponent's 6-call", -0.125);
                }
                opponents_showed = true;
            }
            (false, Call::Mention(s)) if s == strain => {
                push("an opponent's mention", if nt { -0.4 } else { -0.5 });
                opponents_showed = true;
            }
            (false, Call::Mention(_)) if nt => push("an opponent's mention of a suit", 0.5),
            (false, Call::Mention(Strain::NoTrumps)) => {} // measured at 0 for a suit bid
            (false, Call::Mention(_)) => push("an opponent's mention of another suit", 0.25),
            (false, Call::Bid(c)) if c.strain == strain => {
                push("an opponent's bid", -2.0);
                opponents_showed = true;
            }
            _ => {}
        }
    }
    let dealer = obs.dealer == me;
    if dealer && !opponents_showed {
        out.push(("speaking last".into(), if nt { 0.6 } else { 0.5 }));
    }
    // Speaking last, with nobody having bid or called 6, and without the Joker yourself: whoever held
    // it would usually have bid, or called 6, so it's in the kitty about four times in five, and
    // your count runs about 1¼ tricks low (finding 41; measured for suit bids).
    let quiet = !obs.auction.calls().any(|(_, c)| matches!(c, Call::Six(_) | Call::Bid(_)));
    if dealer && quiet && !nt && !obs.hand.contains(Card::JOKER) && obs.auction.calls().count() == 3 {
        out.push(("nobody showed the Joker: probably in the kitty".into(), 1.25));
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
