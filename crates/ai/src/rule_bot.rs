//! A rule-of-thumb player: counts tricks to bid, signals honestly, draws trumps as declarer,
//! wins tricks cheaply and doesn't waste cards on a trick its partner has won.
//!
//! This is the Phase 1 baseline the search and neural bots will be measured against.

use crate::cardplay::{cheapest, is_boss, trumps};
use crate::evaluate::{has_bower, hand_tricks};
use crate::Bot;
use fh_core::cards::{ACE, KING};
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
}

impl Style {
    pub const STEADY: Style = Style {
        aggression: 0.0,
        partner_credit: 1.5,
        kitty_credit: 0.5,
        bid_noise: 0.0,
        blunder_rate: 0.0,
        remembers_cards: true,
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
        strains.sort_by(|a, b| b.1.total_cmp(&a.1));

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
        if obs.hand.contains(Card::JOKER) && is_legal(Call::Six(best)) {
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
        Action::Discard(heuristic_discards(obs.hand, obs.trump(), 1)[0])
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
        Action::Play(*play.expect("chosen card is legal"))
    }

    fn lead(&self, obs: &Observation, cards: &[Card], seen: CardSet) -> Card {
        let trump = obs.trump();
        let hand = obs.hand;
        let declaring = obs.declarer().map(|d| d.team()) == Some(obs.seat.team());
        let my_trumps = trumps(hand, trump);

        // Declarers draw trumps while the defenders may still hold some.
        if declaring && !my_trumps.is_empty() {
            let outstanding = trumps(CardSet::pack(), trump).minus(seen);
            if !outstanding.is_empty() {
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

/// The `count` cards to discard from `hand`, chosen greedily by [`keep_value`].
pub fn heuristic_discards(hand: CardSet, trump: Option<Suit>, count: usize) -> Vec<Card> {
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
    use fh_core::{Game, Rules, Team};

    fn rule_bots() -> [Box<dyn Bot>; 4] {
        std::array::from_fn(|_| Box::new(RuleBot::new("Steady", Style::STEADY, 0)) as Box<dyn Bot>)
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
