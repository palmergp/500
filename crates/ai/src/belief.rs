//! Sampling hidden cards consistently with what one seat knows: the "worlds" PIMC searches.
//!
//! Hard constraints: hand sizes, voids shown in play (by effective suit), and the Joker sitting
//! with whoever called 6. Soft evidence (mentions, bids, passes) becomes a weight per world.

use crate::evaluate::{has_bower, hand_tricks};
use fh_core::cards::ACE;
use fh_core::{effective_suit, Call, Card, CardSet, Observation, Phase, Rng, Seat, Strain, Suit};

/// One possible layout of the hidden cards.
#[derive(Clone, Debug)]
pub struct World {
    /// All four hands (the observer's is its real hand).
    pub hands: [CardSet; 4],
    /// Cards out of play that the observer can't see: the kitty during the auction, or the
    /// declarer's discards during play.
    pub out: CardSet,
    /// Relative plausibility given the auction (1.0 = fully consistent).
    pub weight: f64,
}

const OUT: usize = 4;

pub struct Belief {
    me: Seat,
    unknown: Vec<Card>,
    /// Remaining hand size per seat, and the out-of-play pile size at index 4.
    capacity: [usize; 5],
    /// For each unknown card, a bitmask of the buckets (seats 0–3, pile 4) allowed to hold it.
    allowed: Vec<u8>,
    played_by: [CardSet; 4],
    calls: Vec<(Seat, Call)>,
    inference: bool,
    trump: Option<Suit>,
    /// Each seat's discard signal (signal suit, the suit it showed out of), if reading them.
    signals: [Option<(Suit, Suit)>; 4],
}

impl Belief {
    /// `inference`: weight worlds by how well they fit the auction and partner's discard signals
    /// (otherwise all weights are 1).
    pub fn new(obs: &Observation, inference: bool) -> Belief {
        Belief::new_with(obs, inference, inference)
    }

    /// As [`Belief::new`], choosing separately whether to read discard signals.
    pub fn new_with(obs: &Observation, inference: bool, read_signals: bool) -> Belief {
        let me = obs.seat;
        let trump = obs.trump();
        let played = obs.played();
        let my_discards = obs.discards.unwrap_or_default();
        let known = obs.hand.union(my_discards);
        let unknown: Vec<Card> = CardSet::pack().minus(known).minus(played).iter().collect();

        let mut played_by = [CardSet::EMPTY; 4];
        let mut void = [[false; 4]; 4];
        let mut signals: [Option<(Suit, Suit)>; 4] = [None; 4];
        for trick in obs.tricks.iter().chain(obs.current_trick.iter()) {
            let led = trick.led_suit();
            for (seat, card) in trick.plays() {
                played_by[seat.index()].insert(card);
                if let Some(led) = led {
                    if seat != trick.leader() && effective_suit(card, trump) != Some(led) {
                        let first = !void[seat.index()].iter().any(|&v| v);
                        void[seat.index()][led.index()] = true;
                        if first && read_signals && seat != me && !fh_core::rules::is_trump(card, trump) && !card.is_joker() {
                            signals[seat.index()] = effective_suit(card, trump).map(|s| (s, led));
                        }
                    }
                }
            }
        }

        // Sizes add up to the unknown cards whenever this seat is the one to act: during the
        // auction, during its own kitty discard, and during play.
        let declarer = obs.declarer();
        let mut capacity = [0usize; 5];
        for seat in Seat::ALL.into_iter().filter(|&s| s != me) {
            capacity[seat.index()] = 10 - played_by[seat.index()].len();
        }
        capacity[OUT] = match obs.phase {
            Phase::Bidding => 3,
            _ if declarer == Some(me) => 0,
            _ => 3,
        };

        // Who must hold the Joker, if someone called 6 and it hasn't been played.
        let six = obs.auction.six().map(|(seat, _)| seat);
        let joker_buckets: Option<u8> = match six {
            Some(seat) if seat != me && !played.contains(Card::JOKER) => {
                let mut mask = 1u8 << seat.index();
                if declarer == Some(seat) && obs.phase != Phase::Bidding {
                    mask |= 1 << OUT; // the declarer may have discarded it
                }
                Some(mask)
            }
            _ => None,
        };

        let allowed = unknown
            .iter()
            .map(|&card| {
                let mut mask = 0u8;
                for seat in Seat::ALL.into_iter().filter(|&s| s != me) {
                    let s = seat.index();
                    let suit_ok = effective_suit(card, trump).is_none_or(|suit| !void[s][suit.index()]);
                    if suit_ok && capacity[s] > 0 {
                        mask |= 1 << s;
                    }
                }
                if capacity[OUT] > 0 {
                    mask |= 1 << OUT;
                }
                if card.is_joker() {
                    if let Some(only) = joker_buckets {
                        // Keep the constraint only if it is satisfiable (a human may have bluffed).
                        if mask & only != 0 {
                            mask &= only;
                        }
                    }
                }
                mask
            })
            .collect();

        Belief {
            me,
            unknown,
            capacity,
            allowed,
            played_by,
            calls: obs.auction.calls().collect(),
            inference,
            trump,
            signals,
        }
    }

    /// Cards the observer can't see.
    pub fn unknown(&self) -> &[Card] {
        &self.unknown
    }

    /// Deals the unknown cards into a world. Falls back to ignoring voids if the constraints
    /// can't be met (which would mean an earlier inference was wrong).
    pub fn sample(&self, rng: &mut Rng, my_hand: CardSet) -> World {
        let (hands, out) = self
            .deal(rng, &self.allowed)
            .or_else(|| {
                let relaxed: Vec<u8> = self.allowed.iter().map(|_| self.any_bucket()).collect();
                self.deal(rng, &relaxed)
            })
            .expect("capacities always fit the unknown cards");
        let mut hands = hands;
        hands[self.me.index()] = my_hand;
        let weight = if self.inference { self.weight(&hands) } else { 1.0 };
        World { hands, out, weight }
    }

    /// `n` worlds that fit the auction: candidates are drawn and kept with probability equal to
    /// their fit, so kept worlds look like hands that would have made these calls. If too few
    /// pass after `n * 25` draws, the best-fitting rejects fill in, keeping their weights.
    pub fn sample_fitting(&self, rng: &mut Rng, my_hand: CardSet, n: usize) -> Vec<World> {
        let mut kept = Vec::with_capacity(n);
        let mut rejects: Vec<World> = Vec::new();
        for _ in 0..n * 25 {
            if kept.len() == n {
                break;
            }
            let mut world = self.sample(rng, my_hand);
            if rng.unit() < world.weight {
                world.weight = 1.0;
                kept.push(world);
            } else {
                rejects.push(world);
            }
        }
        if kept.len() < n {
            rejects.sort_by(|a, b| b.weight.total_cmp(&a.weight));
            kept.extend(rejects.into_iter().take(n - kept.len()));
        }
        kept
    }

    fn any_bucket(&self) -> u8 {
        (0..5).filter(|&b| self.capacity[b] > 0).fold(0, |m, b| m | 1 << b)
    }

    fn deal(&self, rng: &mut Rng, allowed: &[u8]) -> Option<([CardSet; 4], CardSet)> {
        let mut order: Vec<usize> = (0..self.unknown.len()).collect();
        for _ in 0..50 {
            rng.shuffle(&mut order);
            // Most constrained cards first.
            order.sort_by_key(|&i| allowed[i].count_ones());
            let mut cap = self.capacity;
            let mut buckets = [CardSet::EMPTY; 5];
            let mut ok = true;
            for &i in &order {
                let open: u32 = (0..5).filter(|&b| allowed[i] & (1 << b) != 0).map(|b| cap[b] as u32).sum();
                if open == 0 {
                    ok = false;
                    break;
                }
                let mut pick = rng.below(open as usize) as u32;
                for b in 0..5 {
                    if allowed[i] & (1 << b) == 0 {
                        continue;
                    }
                    if pick < cap[b] as u32 {
                        buckets[b].insert(self.unknown[i]);
                        cap[b] -= 1;
                        break;
                    }
                    pick -= cap[b] as u32;
                }
            }
            if ok {
                let hands = [buckets[0], buckets[1], buckets[2], buckets[3]];
                return Some((hands, buckets[OUT]));
            }
        }
        None
    }

    /// How well a world fits the calls each other player made, as a product of likelihood
    /// factors. Mentions should come with a bower (or two aces for NT); bids with a hand that
    /// plausibly supports them; passes with hands that couldn't bid much.
    fn weight(&self, hands: &[CardSet; 4]) -> f64 {
        let mut weight = 1.0;
        for &(seat, call) in &self.calls {
            if seat == self.me {
                continue;
            }
            // What the player held when calling (cards played since then included).
            let held = hands[seat.index()].union(self.played_by[seat.index()]);
            let factor = match call {
                Call::Mention(strain) => match strain.trump() {
                    Some(suit) if has_bower(held, suit) => 1.0,
                    Some(_) => 0.15,
                    None => {
                        let aces = Suit::ALL.iter().filter(|&&s| held.contains(Card::new(s, ACE))).count();
                        if aces >= 2 || held.contains(Card::JOKER) {
                            1.0
                        } else {
                            0.4
                        }
                    }
                },
                Call::Bid(contract) => {
                    // A bidder's hand should roughly support the bid. The rough evaluation plus 3
                    // tricks (partner and kitty) matches actual play (`fh-lab calibrate`).
                    let estimate = hand_tricks(held, contract.strain) + 3.0;
                    logistic(2.0 * (estimate - contract.level as f32 + 0.5))
                }
                Call::Six(strain) => {
                    // The Joker is placed by a hard constraint; the strain should be a real one.
                    let best = Strain::ALL.iter().map(|&s| hand_tricks(held, s)).fold(0.0f32, f32::max);
                    logistic(2.0 * (hand_tricks(held, strain) - best + 1.5))
                }
                Call::Pass => {
                    // Someone who passed rarely held a hand worth 8 or more.
                    let best = Strain::ALL.iter().map(|&s| hand_tricks(held, s)).fold(0.0f32, f32::max);
                    1.0 - 0.8 * logistic(2.0 * (best + 3.0 - 8.5))
                }
            };
            weight *= factor;
        }
        // A discard signal says "this is my strongest side suit".
        for (seat, signal) in self.signals.iter().enumerate() {
            if let Some((suit, void_in)) = signal {
                let held = hands[seat].union(self.played_by[seat]);
                let strongest = crate::signals::strongest_side_suit(held, self.trump, Some(*void_in));
                weight *= if strongest == Some(*suit) { 1.0 } else { 0.35 };
            }
        }
        weight
    }
}

fn logistic(x: f32) -> f64 {
    1.0 / (1.0 + (-x as f64).exp())
}

#[cfg(test)]
mod tests {
    use super::*;
    use fh_core::{Action, Contract, Game, Rules};

    #[test]
    fn worlds_respect_hand_sizes_and_the_joker_signal() {
        // Dealer East: South, West, North, East call in turn. Find a seed where West holds the Joker.
        let seed = (0..200).find(|&s| Game::with_dealer(Rules::default(), s, Seat::EAST).hand(Seat::WEST).contains(Card::JOKER)).unwrap();
        let mut game = Game::with_dealer(Rules::default(), seed, Seat::EAST);
        game.apply(Action::Call(Call::Pass)).unwrap();
        game.apply(Action::Call(Call::Six(Strain::Hearts))).unwrap();
        let obs = game.observe(Seat::NORTH);
        let belief = Belief::new(&obs, true);
        let mut rng = Rng::new(1);
        for _ in 0..200 {
            let world = belief.sample(&mut rng, obs.hand);
            assert_eq!(world.hands.map(|h| h.len()), [10, 10, 10, 10]);
            assert_eq!(world.out.len(), 3);
            assert!(world.hands[Seat::WEST.index()].contains(Card::JOKER), "6 shows the Joker");
            let all = world.hands.iter().fold(world.out, |a, &h| a.union(h));
            assert_eq!(all, CardSet::pack());
        }
    }

    #[test]
    fn worlds_respect_shown_voids() {
        // Play a hand with rule bots until someone fails to follow suit, then check the void holds.
        for seed in 0..50 {
            let mut game = Game::with_dealer(Rules::default(), seed, Seat::EAST);
            game.apply(Action::Call(Call::Bid(Contract::new(7, Strain::Hearts)))).unwrap();
            for _ in 0..3 {
                game.apply(Action::Call(Call::Pass)).unwrap();
            }
            let mut bots: [Box<dyn crate::Bot>; 4] = std::array::from_fn(|i| crate::make_bot("steady", i as u64).unwrap());
            for _ in 0..30 {
                let seat = game.to_act().unwrap();
                let action = bots[seat.index()].choose(&game.observe(seat), &game.legal_actions());
                game.apply(action).unwrap();
            }
            if game.phase() != Phase::Playing {
                continue;
            }
            let me = game.to_act().unwrap();
            let obs = game.observe(me);
            let belief = Belief::new(&obs, false);
            let mut rng = Rng::new(seed);
            for _ in 0..20 {
                let world = belief.sample(&mut rng, obs.hand);
                for seat in Seat::ALL {
                    assert_eq!(world.hands[seat.index()].len(), game.hand(seat).len(), "seed {seed} {seat}");
                }
                // Every seat holds only cards of suits it hasn't shown out of.
                for trick in game.tricks() {
                    let led = trick.led_suit().unwrap();
                    for (seat, card) in trick.plays() {
                        if seat != me && effective_suit(card, game.trump()) != Some(led) {
                            assert!(world.hands[seat.index()].iter().all(|c| effective_suit(c, game.trump()) != Some(led)));
                        }
                    }
                }
            }
        }
    }
}
