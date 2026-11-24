//! A rule-of-thumb player: counts tricks to bid, signals honestly, draws trumps as declarer,
//! wins tricks cheaply and doesn't waste cards on a trick its partner has won.
//!
//! This is the Phase 1 baseline the search and neural bots will be measured against.

use crate::cardplay::{cheapest, is_boss, trumps};
use crate::evaluate::{has_bower, hand_tricks};
use crate::Bot;
use fh_core::cards::{ACE, KING, QUEEN};
use fh_core::rules::is_trump;
use fh_core::trick::trick_strength;
use fh_core::{effective_suit, Action, Call, Card, CardSet, Contract, Observation, Phase, Play, Rng, Strain, Suit};

/// Bidding temperament.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Style {
    /// Tricks added to every estimate: negative is cautious, positive is reckless.
    pub aggression: f32,
    /// Tricks expected from partner before they have called.
    pub partner_credit: f32,
    /// Tricks expected from the kitty.
    pub kitty_credit: f32,
    /// Random error (± tricks) in each bidding estimate: bidding "by gut".
    pub bid_noise: f32,
    /// Chance of playing a random legal card instead of the planned one.
    pub blunder_rate: f32,
    /// Whether the bot remembers cards from earlier tricks (otherwise it sees only its hand
    /// and the current trick).
    pub remembers_cards: bool,
    /// When to make a 6-call with the Joker (lab variants vary it; the house signal is `Always`).
    pub six: SixPolicy,
    /// Whether to mention or call 6 after partner has had their call. Partner can't use it in the
    /// auction then, and in 2,000 duplicate deals it made no difference (finding 35), so it's off;
    /// a lab variant turns it back on.
    pub signal_after_partner: bool,
}

/// When a player holding the Joker, without a bid, calls 6 in their best strain.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SixPolicy {
    /// Always (the house signal).
    Always,
    /// Always, except as the first to call with a rule-of-thumb count below this: pass or
    /// mention instead.
    FirstNeeds(f32),
    /// Never.
    Never,
}

impl Style {
    pub const STEADY: Style = Style {
        aggression: 0.0,
        partner_credit: 1.5,
        kitty_credit: 0.5,
        bid_noise: 0.0,
        blunder_rate: 0.0,
        remembers_cards: true,
        six: SixPolicy::Always,
        signal_after_partner: false,
    };
    pub const CAUTIOUS: Style = Style { aggression: -0.7, partner_credit: 1.2, kitty_credit: 0.4, ..Style::STEADY };
    pub const BOLD: Style = Style { aggression: 0.8, partner_credit: 1.8, kitty_credit: 0.7, ..Style::STEADY };
    pub const ROOKIE: Style = Style {
        aggression: 0.3,
        bid_noise: 1.0,
        blunder_rate: 0.12,
        remembers_cards: false,
        ..Style::STEADY
    };
}

pub struct RuleBot {
    name: String,
    style: Style,
    rng: Rng,
}

impl RuleBot {
    pub fn new(name: &str, style: Style, seed: u64) -> RuleBot {
        RuleBot { name: name.to_string(), style, rng: Rng::new(seed) }
    }

    /// Expected tricks for our side in `strain`, adjusted for what the auction has shown.
    pub fn estimate(&self, obs: &Observation, strain: Strain) -> f32 {
        let me = obs.seat;
        let mut support = self.style.partner_credit;
        for (seat, call) in obs.auction.calls() {
            let partner = seat == me.partner();
            let delta = match call {
                Call::Six(s) => 0.6 + if s == strain { 0.4 } else { 0.0 },
                Call::Mention(m) if m == strain => 0.7,
                Call::Bid(c) if c.strain == strain => 1.0,
                Call::Bid(_) => 0.3,
                Call::Pass => -0.4,
                Call::Mention(_) => 0.0,
            };
            if partner {
                support += delta;
            } else if seat != me && delta > 0.0 && !matches!(call, Call::Bid(c) if c.strain != strain) {
                support -= delta * 0.5; // the strength an opponent showed is against us
            }
        }
        hand_tricks(obs.hand, strain) + support + self.style.kitty_credit + self.style.aggression
    }

    fn call(&mut self, obs: &Observation, legal: &[Action]) -> Action {
        let is_legal = |call: Call| legal.contains(&Action::Call(call));
        let noise = self.style.bid_noise;
        let mut strains: Vec<(Strain, f32)> = Strain::ALL
            .into_iter()
            .map(|s| (s, self.estimate(obs, s) + noise * (2.0 * self.rng.unit() as f32 - 1.0)))
            .collect();
        // Best estimate first. A tie (the estimator can rate a long trump suit and the same honours
        // as a side suit equally; floating-point sums then differ in the last bit) goes to the suit
        // with more trumps, then the higher count, not to rounding noise (a user's hand, code
        // NQE9-5DH3: 9♣ over five hearts).
        let trumps = |s: Strain| obs.hand.iter().filter(|&c| s.trump().is_some() && (is_trump(c, s.trump()) || c.is_joker())).count();
        let hundredths = |e: f32| (e * 100.0).round() as i32;
        strains.sort_by(|a, b| {
            hundredths(b.1).cmp(&hundredths(a.1))
                .then_with(|| trumps(b.0).cmp(&trumps(a.0)))
                .then_with(|| crate::thumb::count(obs, b.0).0.total_cmp(&crate::thumb::count(obs, a.0).0))
        });

        let partner_high = obs.auction.high_bid().map(|(seat, _)| seat) == Some(obs.seat.partner());
        for &(strain, estimate) in &strains {
            let level = estimate.floor().min(10.0) as u8;
            if level >= 7 {
                let call = Call::Bid(Contract::new(level, strain));
                // Over partner's bid, only bid if it is clearly better for the side.
                let worth_it = !partner_high || estimate >= 8.0;
                if worth_it && is_legal(call) {
                    return Action::Call(call);
                }
            }
        }
        let best = strains[0].0;
        if obs.auction.must_bid() {
            let forced = legal.iter().find(|a| matches!(a, Action::Call(Call::Bid(c)) if c.strain == best));
            return *forced.unwrap_or(&legal[0]);
        }
        let six = match self.style.six {
            SixPolicy::Always => true,
            SixPolicy::Never => false,
            SixPolicy::FirstNeeds(count) => obs.auction.calls().next().is_some() || crate::thumb::count(obs, best).0 >= count,
        };
        let partner_called = obs.auction.calls().any(|(s, _)| s == obs.seat.partner());
        if partner_called && !self.style.signal_after_partner {
            return Action::Call(Call::Pass);
        }
        if six && obs.hand.contains(Card::JOKER) && is_legal(Call::Six(best)) {
            return Action::Call(Call::Six(best));
        }
        for &(strain, _) in &strains {
            let shows = match strain.trump() {
                Some(suit) => has_bower(obs.hand, suit),
                None => Suit::ALL.iter().filter(|&&s| obs.hand.contains(Card::new(s, ACE))).count() >= 2,
            };
            if shows && is_legal(Call::Mention(strain)) {
                return Action::Call(Call::Mention(strain));
            }
        }
        Action::Call(Call::Pass)
    }

    fn discard(&self, obs: &Observation) -> Action {
        // The plan for all the discards still to make, so the set is judged as a whole.
        let left = obs.hand.len().saturating_sub(10).max(1);
        Action::Discard(heuristic_discards(obs.hand, obs.trump(), left)[0])
    }

    fn play(&mut self, obs: &Observation, legal: &[Action]) -> Action {
        if self.style.blunder_rate > 0.0 && self.rng.unit() < self.style.blunder_rate as f64 {
            return legal[self.rng.below(legal.len())];
        }
        let trick = obs.current_trick.as_ref().expect("playing phase has a trick");
        let options: Vec<Play> = legal.iter().filter_map(|a| if let Action::Play(p) = a { Some(*p) } else { None }).collect();
        let cards: Vec<Card> = options.iter().map(|p| p.card).collect();
        let seen = if self.style.remembers_cards {
            obs.played().union(obs.hand).union(obs.discards.unwrap_or_default())
        } else {
            obs.hand.union(trick.cards().iter().copied().collect())
        };

        let card = if trick.is_empty() {
            self.lead(obs, &cards, seen)
        } else {
            self.follow(obs, &cards, seen)
        };
        let play = options.iter().filter(|p| p.card == card).max_by_key(|p| {
            // When leading the Joker in no trumps, name our longest suit.
            p.named_suit.map_or(0, |s| obs.hand.of_printed_suit(s).len())
        });
        crate::cardplay::show_winner(obs, Action::Play(*play.expect("chosen card is legal")))
    }

    fn lead(&self, obs: &Observation, cards: &[Card], seen: CardSet) -> Card {
        let trump = obs.trump();
        let hand = obs.hand;
        let declaring = obs.declarer().map(|d| d.team()) == Some(obs.seat.team());
        let my_trumps = trumps(hand, trump);

        // Declarers draw trumps while the defenders may still hold some (not once both have shown
        // out: the trumps left are partner's, and drawing them only pulls partner's trumps).
        if declaring && !my_trumps.is_empty() {
            let out = if self.style.remembers_cards {
                crate::cardplay::opponents_may_hold_trumps(obs, seen)
            } else {
                !trumps(CardSet::pack(), trump).minus(seen).is_empty()
            };
            if out {
                let top = my_trumps.iter().max_by_key(|&c| trick_strength(c, trump.unwrap(), trump)).unwrap();
                return if is_boss(top, seen, trump) {
                    top
                } else {
                    cheapest(&my_trumps.iter().collect::<Vec<_>>(), hand, trump)
                };
            }
        }
        // Cash a sure winner in a side suit.
        let side: Vec<Card> = cards.iter().copied().filter(|&c| !is_trump(c, trump) && !c.is_joker()).collect();
        if let Some(&boss) = side.iter().filter(|&&c| is_boss(c, seen, trump)).max_by_key(|c| c.rank()) {
            return boss;
        }
        // Partner signalled their strongest suit: lead to it.
        if let Some(suit) = crate::signals::discard_signals(obs)[obs.seat.partner().index()] {
            if let Some(&low) = side.iter().filter(|&&c| effective_suit(c, trump) == Some(suit)).min_by_key(|c| c.rank()) {
                return low;
            }
        }
        // Otherwise lead low from our longest side suit, keeping trumps back.
        let longest = side.iter().copied().max_by_key(|&c| {
            let len = hand.iter().filter(|&h| effective_suit(h, trump) == effective_suit(c, trump)).count();
            (len, std::cmp::Reverse(c.rank()))
        });
        longest.unwrap_or_else(|| cheapest(cards, hand, trump))
    }

    fn follow(&self, obs: &Observation, cards: &[Card], seen: CardSet) -> Card {
        let trump = obs.trump();
        let hand = obs.hand;
        let trick = obs.current_trick.as_ref().unwrap();
        let led = trick.led_suit().unwrap();
        let winner = trick.winner(trump).unwrap();
        let winning = trick.winning_card(trump).unwrap();
        let last = trick.cards().len() == 3;

        let partner_winning = winner == obs.seat.partner();
        if partner_winning && (last || is_boss(winning, seen, trump)) {
            let void = !cards.iter().any(|&c| effective_suit(c, trump) == Some(led));
            if void && crate::signals::first_time_void(obs, obs.seat) {
                if let Some(card) = crate::signals::signal_card(hand, trump, led).filter(|c| cards.contains(c)) {
                    return card;
                }
            }
            return cheapest(cards, hand, trump);
        }
        let to_beat = trick_strength(winning, led, trump);
        let winners: Vec<Card> = cards.iter().copied().filter(|&c| trick_strength(c, led, trump) > to_beat).collect();
        let void = !cards.iter().any(|&c| effective_suit(c, trump) == Some(led));
        // The discard signal: the first throw-off comes from our strongest side suit.
        let signal = || {
            if void && crate::signals::first_time_void(obs, obs.seat) {
                crate::signals::signal_card(hand, trump, led).filter(|c| cards.contains(c))
            } else {
                None
            }
        };
        if winners.is_empty() {
            return signal().unwrap_or_else(|| cheapest(cards, hand, trump));
        }
        let cheapest_winner = cheapest(&winners, hand, trump);
        let ruffing = is_trump(cheapest_winner, trump) && Some(led) != trump;
        let second_hand = trick.cards().len() == 1;
        if last || ruffing || is_boss(cheapest_winner, seen, trump) || !second_hand {
            cheapest_winner
        } else {
            cheapest(cards, hand, trump) // second hand low
        }
    }
}

/// How much the declarer wants to keep `card` from `hand`: trumps most, then aces and guarded
/// kings; cards from short suits are cheaper, since discarding them builds toward a void.
pub fn keep_value(card: Card, hand: CardSet, trump: Option<Suit>) -> i32 {
    if is_trump(card, trump) || card.is_joker() {
        return 1000 + card.rank() as i32;
    }
    if trump.is_none() {
        return nt_keep_value(card, hand);
    }
    let suit = effective_suit(card, trump);
    let in_suit = hand.iter().filter(|&h| effective_suit(h, trump) == suit).count();
    let mut value = card.rank() as i32;
    if card.rank() == ACE {
        value += 40;
    } else if card.rank() == KING && in_suit >= 2 {
        value += 15; // guarded king
    }
    if in_suit <= 2 && card.rank() != ACE {
        value -= 8; // shortening a suit toward a void
    }
    value
}

/// In no trumps a void can't be trumped, so there's no reason to make one; what matters is
/// stoppers and running suits (docs/lab/README.md, finding 23). Keep aces, guarded kings and their
/// guards, guarded queens and length under an ace; throw low cards from suits with no honours.
fn nt_keep_value(card: Card, hand: CardSet) -> i32 {
    let suit = card.suit();
    let in_suit: Vec<Card> = hand.iter().filter(|c| c.suit() == suit).collect();
    let has = |rank: u8| in_suit.iter().any(|c| c.rank() == rank);
    let mut value = card.rank() as i32;
    match card.rank() {
        ACE => value += 40,
        KING if in_suit.len() >= 2 => value += 15,
        QUEEN if in_suit.len() >= 3 => value += 8,
        _ => {}
    }
    // The small card that guards a king (or one of the two guarding a queen).
    let guards = |honour: u8, needed: usize| has(honour) && !has(ACE) && card.rank() < honour && in_suit.len() <= needed;
    if guards(KING, 2) || guards(QUEEN, 3) {
        value += 12;
    }
    // Length under an ace runs once the ace is cashed.
    if has(ACE) && card.rank() != ACE {
        value += 6;
    }
    value
}

/// The `count` cards to discard from `hand`, chosen greedily by [`keep_value`].
/// The kitty plan for a suit bid, judged as a whole set rather than one card at a time: each side
/// suit emptied is worth about 0.4 tricks (more with more trumps: findings 16 and 38), and each card
/// put back costs roughly what it's worth to keep (aces always kept; a king about 0.15, a queen
/// 0.05, low cards nothing; a guard is no reason to keep a king). Ties go to the lower cards.
/// Choosing one card at a time missed voids: with K♠ 5♠ and K♣ J♣ it put back 5♠ and J♣, then
/// saw two bare kings as worthless and threw a heart, making no void at all.
pub fn set_discards(hand: CardSet, trump: Option<Suit>, count: usize) -> Vec<Card> {
    set_discards_with(hand, trump, count, KittyWeights::DEFAULT)
}

/// The weights `set_discards` judges a kitty plan by (tuned in the lab: `fh-lab kittyplan`).
#[derive(Clone, Copy, Debug)]
pub struct KittyWeights {
    /// Multiplies the measured worth of a void (0.33 / 0.43 / 0.52 tricks by trump length).
    pub void_scale: f64,
    pub king: f64,
    pub queen: f64,
    /// For each side suit cut to a single card.
    pub singleton: f64,
}

impl KittyWeights {
    pub const DEFAULT: KittyWeights = KittyWeights { void_scale: 1.5, king: 0.15, queen: 0.05, singleton: 0.1 };
}

pub fn set_discards_with(hand: CardSet, trump: Option<Suit>, count: usize, w: KittyWeights) -> Vec<Card> {
    ranked_discards(hand, trump, count, w).into_iter().next().expect("a set")
}

/// Every kitty plan `set_discards` considers, best first by its score (for a search to choose among
/// the top few). With too few side cards there's just the one plan.
pub fn ranked_discards(hand: CardSet, trump: Option<Suit>, count: usize, w: KittyWeights) -> Vec<Vec<Card>> {
    let trumpish = |c: Card| is_trump(c, trump) || c.is_joker();
    let side: Vec<Card> = hand.iter().filter(|&c| !trumpish(c)).collect();
    if side.len() <= count || count == 0 || count > 3 {
        // Too few side cards: all of them, then the lowest trumps (as before).
        let mut out: Vec<Card> = side.iter().copied().take(count).collect();
        let mut rest = hand.minus(out.iter().copied().collect());
        while out.len() < count {
            let card = rest.iter().min_by_key(|&c| (keep_value(c, rest, trump), c.id())).expect("cards to discard");
            rest.remove(card);
            out.push(card);
        }
        return vec![out];
    }
    let trumps = hand.iter().filter(|&c| trumpish(c)).count();
    let void_worth = w.void_scale
        * match trumps {
            0..=5 => 0.33,
            6 => 0.43,
            _ => 0.52,
        };
    let worth = |c: Card| match c.rank() {
        ACE => 1.0,
        KING => w.king,
        QUEEN => w.queen,
        _ => 0.0,
    };
    let length = |set: &[Card], suit: Option<Suit>| side.iter().filter(|c| effective_suit(**c, trump) == suit && !set.contains(c)).count();
    let score = |set: &[Card]| {
        let lost: f64 = set.iter().map(|&c| worth(c)).sum();
        let mut suits: Vec<Option<Suit>> = set.iter().map(|&c| effective_suit(c, trump)).collect();
        suits.dedup();
        let voids = suits.iter().filter(|&&s| length(set, s) == 0).count();
        let singles = suits.iter().filter(|&&s| length(set, s) == 1).count();
        void_worth * voids as f64 + w.singleton * singles as f64 - lost
    };
    let rank_sum = |set: &[Card]| set.iter().map(|c| c.rank() as u32).sum::<u32>();
    let mut all: Vec<(Vec<Card>, f64)> = Vec::new();
    let n = side.len();
    match count {
        1 => (0..n).for_each(|i| all.push((vec![side[i]], 0.0))),
        2 => (0..n).for_each(|i| (i + 1..n).for_each(|j| all.push((vec![side[i], side[j]], 0.0)))),
        _ => (0..n).for_each(|i| (i + 1..n).for_each(|j| (j + 1..n).for_each(|k| all.push((vec![side[i], side[j], side[k]], 0.0))))),
    }
    for (set, s) in all.iter_mut() {
        *s = score(set);
    }
    // Best first; ties to the lower cards.
    all.sort_by(|a, b| b.1.total_cmp(&a.1).then(rank_sum(&a.0).cmp(&rank_sum(&b.0))));
    all.into_iter()
        .map(|(mut set, _)| {
            // Put back the cheapest first (the order the discards are made in).
            set.sort_by_key(|&c| (keep_value(c, hand, trump), c.id()));
            set
        })
        .collect()
}

/// The kitty plan the bots and the coach use: judged as a set in a suit bid (`set_discards`,
/// finding 40), card by card in no trumps (`nt_keep_value`, finding 23).
pub fn heuristic_discards(hand: CardSet, trump: Option<Suit>, count: usize) -> Vec<Card> {
    if trump.is_some() {
        set_discards(hand, trump, count)
    } else {
        greedy_discards(hand, trump, count)
    }
}

/// One card at a time, the lowest `keep_value` first (the suit-bid plan before finding 40).
pub fn greedy_discards(hand: CardSet, trump: Option<Suit>, count: usize) -> Vec<Card> {
    let mut hand = hand;
    let mut out = Vec::with_capacity(count);
    for _ in 0..count.min(hand.len()) {
        let card = hand.iter().min_by_key(|&c| (keep_value(c, hand, trump), c.id())).expect("cards to discard");
        hand.remove(card);
        out.push(card);
    }
    out
}

impl Bot for RuleBot {
    fn name(&self) -> &str {
        &self.name
    }

    fn choose(&mut self, obs: &Observation, legal: &[Action]) -> Action {
        match obs.phase {
            Phase::Bidding => self.call(obs, legal),
            Phase::Discarding => self.discard(obs),
            Phase::Playing => self.play(obs, legal),
            Phase::GameOver => legal[0],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{play_out, RandomBot};
    use fh_core::{Game, Rules, Seat, Team};

    /// Once both opponents have shown out of trumps, the declarer stops leading them: the trumps
    /// left are partner's, and drawing them only pulls partner's (a user saw The Gambler do it).
    #[test]
    fn stops_drawing_trumps_once_the_opponents_are_out() {
        use fh_core::{Call, Contract, Observation, Phase, Play, Strain, Trick};
        let rules = Rules::default();
        let mut auction = fh_core::Auction::new(Seat::SOUTH, false);
        assert!(auction.apply(Call::Bid(Contract::new(7, Strain::Spades)), &rules));
        for _ in 0..3 {
            assert!(auction.apply(Call::Pass, &rules));
        }
        let trick = |leader: Seat, cards: &str| {
            let plays: Vec<Play> = cards.split(' ').map(|c| Play::card(Card::parse(c).unwrap())).collect();
            Trick::from_plays(leader, &plays, Some(Suit::Spades))
        };
        // South leads the right bower and the ace of trumps; West shows out on the first, East on the second.
        let tricks = vec![trick(Seat::SOUTH, "JS 7H 5S 6S"), trick(Seat::SOUTH, "AS 8H 7S 9D")];
        let obs = |tricks: Vec<Trick>| Observation {
            seat: Seat::SOUTH,
            phase: Phase::Playing,
            to_act: Some(Seat::SOUTH),
            rules: rules.clone(),
            scores: [0, 0],
            dealer: Seat::EAST,
            deal_number: 1,
            hand: CardSet::parse("QS 10S AH KH QH 6C 5C 4D").unwrap(),
            auction: auction.clone(),
            contract: Some((Seat::SOUTH, Contract::new(7, Strain::Spades))),
            kitty: None,
            discards: Some(CardSet::parse("4H 7C 5D").unwrap()),
            tricks_won: [tricks.len() as u8, 0],
            tricks,
            current_trick: Some(Trick::new(Seat::SOUTH)),
            hands_played: 0,
        };
        let lead = |o: &Observation| {
            let legal: Vec<Action> = fh_core::legal_plays(o.hand, o.current_trick.as_ref().unwrap(), o.trump()).into_iter().map(Action::Play).collect();
            match RuleBot::new("Gambler", Style::BOLD, 1).choose(o, &legal) {
                Action::Play(p) => p.card,
                other => panic!("{other:?}"),
            }
        };
        let trump = |c: Card| is_trump(c, Some(Suit::Spades));
        assert!(!trump(lead(&obs(tricks.clone()))), "the opponents are out of trumps");
        // With East not yet shown out (followed with a spade), it draws trumps as before.
        let east_follows = vec![trick(Seat::SOUTH, "JS 7H 5S 6S"), trick(Seat::SOUTH, "AS 8H 7S 8S")];
        assert!(trump(lead(&obs(east_follows))), "East may still hold trumps");
    }

    /// A user's hand (code NQE9-5DH3): Joker, J♣ 7♣, A♦ K♦ 4♦, A♥ K♥ J♥ 10♥. The rule bots' estimate
    /// rates clubs and hearts exactly equal; the tie goes to hearts, the longer trump suit.
    #[test]
    fn a_tied_estimate_goes_to_the_longer_trump_suit() {
        use fh_core::cards::{ACE, JACK, KING};
        let c = |s: Suit, r: u8| Card::new(s, r);
        let hand: CardSet = [
            Card::JOKER,
            c(Suit::Clubs, JACK),
            c(Suit::Clubs, 7),
            c(Suit::Diamonds, ACE),
            c(Suit::Diamonds, KING),
            c(Suit::Diamonds, 4),
            c(Suit::Hearts, ACE),
            c(Suit::Hearts, KING),
            c(Suit::Hearts, JACK),
            c(Suit::Hearts, 10),
        ]
        .into_iter()
        .collect();
        let (clubs, hearts) = (crate::evaluate::hand_tricks(hand, Strain::Clubs), crate::evaluate::hand_tricks(hand, Strain::Hearts));
        assert!((clubs - hearts).abs() < 0.001, "{clubs} vs {hearts}");
        // The decision as East, first to speak, holding that hand.
        let game = Game::with_dealer(Rules::default(), 7, Seat::NORTH);
        let mut obs = game.observe(Seat::EAST);
        obs.hand = hand;
        let mut bot = RuleBot::new("g", Style::BOLD, 1);
        let legal = game.legal_actions();
        match bot.choose(&obs, &legal) {
            Action::Call(Call::Bid(contract)) => assert_eq!(contract.strain, Strain::Hearts, "{contract}"),
            other => panic!("expected a bid, got {other}"),
        }
    }

    /// A user's kitty (code X5MF-5PK6): 7♦ with K♠ 5♠, K♣ J♣ and four hearts. One card at a time put
    /// back 5♠ J♣ 4♥, leaving two bare kings and no void; judged as a set, the plan empties a suit.
    #[test]
    fn the_kitty_plan_makes_a_void_rather_than_two_bare_kings() {
        use fh_core::cards::{JACK, KING, QUEEN};
        let c = |s: Suit, r: u8| Card::new(s, r);
        let hand: CardSet = [
            c(Suit::Spades, 5),
            c(Suit::Spades, KING),
            c(Suit::Clubs, JACK),
            c(Suit::Clubs, KING),
            c(Suit::Diamonds, 5),
            c(Suit::Diamonds, 7),
            c(Suit::Diamonds, 10),
            c(Suit::Diamonds, JACK),
            c(Suit::Diamonds, KING),
            c(Suit::Hearts, 4),
            c(Suit::Hearts, 5),
            c(Suit::Hearts, 6),
            c(Suit::Hearts, QUEEN),
        ]
        .into_iter()
        .collect();
        let trump = Some(Suit::Diamonds);
        let plan: CardSet = heuristic_discards(hand, trump, 3).into_iter().collect();
        let left = hand.minus(plan);
        let void = [Suit::Spades, Suit::Clubs].iter().any(|&s| left.iter().all(|x| x.suit() != Some(s)));
        assert!(void, "plan {plan} leaves {left}");
        // Asked one discard at a time (as the bots and the coach do), the plan holds together.
        let mut rest = hand;
        for left in (1..=3).rev() {
            let card = heuristic_discards(rest, trump, left)[0];
            assert!(plan.contains(card), "{card} isn't in {plan}");
            rest.remove(card);
        }
    }

    fn rule_bots() -> [Box<dyn Bot>; 4] {
        std::array::from_fn(|_| Box::new(RuleBot::new("Steady", Style::STEADY, 0)) as Box<dyn Bot>)
    }

    /// In no trumps the kitty plan keeps a guarded king and its guard (stoppers), instead of
    /// emptying the suit as it would in a suit bid.
    #[test]
    fn no_trump_discards_keep_stoppers() {
        let hand = CardSet::parse("Joker AS KS 9S AH 8H 6H KC 5C 7D 6D 5D 4H").unwrap();
        let discards: CardSet = heuristic_discards(hand, None, 3).into_iter().collect();
        assert!(!discards.contains(Card::parse("KC").unwrap()) && !discards.contains(Card::parse("5C").unwrap()), "{discards:?}");
        assert!(discards.iter().all(|c| c.rank() != ACE && !c.is_joker()));
    }

    #[test]
    fn rule_bots_finish_games() {
        for seed in 0..40 {
            let mut game = Game::new(Rules::default(), seed);
            let winner = play_out(&mut game, &mut rule_bots(), 100_000);
            assert!(winner.is_some(), "seed {seed} did not finish: {:?}", game.scores());
            game.check_invariants().unwrap();
        }
    }

    #[test]
    fn rule_bots_beat_random_bots() {
        let mut wins = 0;
        let games = 40;
        for seed in 0..games {
            let mut bots: [Box<dyn Bot>; 4] = std::array::from_fn(|i| {
                if i % 2 == 0 {
                    Box::new(RuleBot::new("Steady", Style::STEADY, 0)) as Box<dyn Bot>
                } else {
                    Box::new(RandomBot::new(seed * 4 + i as u64)) as Box<dyn Bot>
                }
            });
            let mut game = Game::new(Rules::default(), seed);
            if play_out(&mut game, &mut bots, 100_000) == Some(Team::NorthSouth) {
                wins += 1;
            }
        }
        assert!(wins >= games * 9 / 10, "rule bots won only {wins}/{games}");
    }

    #[test]
    fn joker_holder_signals_six_when_too_weak_to_bid() {
        let rules = Rules::default();
        let hand = CardSet::parse("Joker 5S 7S 9C 6C 4D 8D 10D 5H 6H").unwrap();
        let others = CardSet::pack().minus(hand);
        let mut rest = others.iter();
        let mut take = |n: usize| -> CardSet { (&mut rest).take(n).collect() };
        let hands = [hand, take(10), take(10), take(10)];
        let kitty = take(3);
        let game = Game::from_deal(rules, 1, fh_core::Seat::EAST, hands, kitty);
        let mut bot = RuleBot::new("Steady", Style::STEADY, 0);
        let action = bot.choose(&game.observe(fh_core::Seat::SOUTH), &game.legal_actions());
        assert!(matches!(action, Action::Call(Call::Six(_))), "{action}");
    }
}
