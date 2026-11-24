//! Card play by a network trained to imitate the expert's search (`train/train_play.py`, from
//! `fh-lab playdata`). One pass through the network instead of a search: fast enough for any
//! device. Bidding and the kitty are The Oracle's.

use crate::features::{play_features, play_slot};
use crate::nn::Mlp;
use crate::{Bot, SearchBot, SearchConfig};
use fh_core::{Action, Observation, Phase};

pub struct NeuralBot {
    name: String,
    expert: SearchBot,
    model: &'static Mlp,
    /// With `Some(p)`, play the network's card only when it gives that card at least probability
    /// `p` among the legal cards; otherwise search as The Oracle does (a quick but still expert
    /// player). With `None`, always play the network's card.
    confident: Option<f32>,
}

impl NeuralBot {
    /// `None` if no play model is embedded.
    pub fn new(name: &str, seed: u64) -> Option<NeuralBot> {
        Some(NeuralBot { name: name.to_string(), expert: SearchBot::new(name, SearchConfig::ORACLE, seed), model: crate::models::play_model()?, confident: None })
    }

    /// The Oracle, skipping the search when the network is at least `p` sure of a card.
    pub fn quick(name: &str, seed: u64, p: f32) -> Option<NeuralBot> {
        NeuralBot::new(name, seed).map(|b| NeuralBot { confident: Some(p), ..b })
    }

    /// The network's probability for each legal action's card (softmax over the legal slots).
    fn probabilities(&self, obs: &Observation, legal: &[Action]) -> Vec<f32> {
        let trump = obs.trump();
        let scores = self.model.forward(&play_features(obs));
        let raw: Vec<f32> = legal
            .iter()
            .map(|a| match a {
                Action::Play(p) => scores[play_slot(p.card, trump)],
                _ => f32::MIN,
            })
            .collect();
        // Each slot counts once (a Joker lead in no trumps is several actions with one slot).
        let mut slots: Vec<(usize, f32)> = Vec::new();
        for (a, &r) in legal.iter().zip(&raw) {
            if let Action::Play(p) = a {
                let slot = play_slot(p.card, trump);
                if !slots.iter().any(|&(s, _)| s == slot) {
                    slots.push((slot, r));
                }
            }
        }
        let top = slots.iter().map(|&(_, r)| r).fold(f32::MIN, f32::max);
        let total: f32 = slots.iter().map(|&(_, r)| (r - top).exp()).sum();
        raw.iter().map(|&r| (r - top).exp() / total).collect()
    }

    /// The network's card: the legal play whose slot scores highest.
    pub fn play(&self, obs: &Observation, legal: &[Action]) -> Action {
        let trump = obs.trump();
        let scores = self.model.forward(&play_features(obs));
        let score = |a: &Action| match a {
            Action::Play(p) => scores[play_slot(p.card, trump)],
            _ => f32::MIN,
        };
        // Never the Joker to the first three tricks of a no-trump hand as declarer (findings 54, 56).
        let hold_joker = crate::cardplay::early_joker_lead(obs).is_some();
        let allowed = |a: &&Action| !(hold_joker && matches!(a, Action::Play(p) if p.card.is_joker()));
        let best = legal.iter().filter(allowed).copied().max_by(|a, b| score(a).total_cmp(&score(b))).expect("legal plays");
        // Of touching cards, the one partner can read (the ace from A-K).
        let best = crate::cardplay::show_winner(obs, best);
        // Leading the Joker in no trumps: name the suit you hold most of (then the higher card).
        let Action::Play(p) = best else { return best };
        if p.named_suit.is_none() {
            return best;
        }
        let length = |s: fh_core::Suit| obs.hand.iter().filter(|c| c.suit() == Some(s)).count();
        let top = |s: fh_core::Suit| obs.hand.iter().filter(|c| c.suit() == Some(s)).map(|c| c.rank()).max().unwrap_or(0);
        legal
            .iter()
            .copied()
            .filter(|a| matches!(a, Action::Play(q) if q.card == p.card))
            .max_by_key(|a| match a {
                Action::Play(q) => q.named_suit.map_or((0, 0), |s| (length(s), top(s))),
                _ => (0, 0),
            })
            .unwrap_or(best)
    }
}

impl Bot for NeuralBot {
    fn name(&self) -> &str {
        &self.name
    }

    fn choose(&mut self, obs: &Observation, legal: &[Action]) -> Action {
        if obs.phase == Phase::Playing && legal.len() > 1 {
            if let Some(p) = self.confident {
                let probabilities = self.probabilities(obs, legal);
                if probabilities.iter().copied().fold(0.0, f32::max) < p {
                    return self.expert.choose(obs, legal);
                }
            }
            self.play(obs, legal)
        } else {
            self.expert.choose(obs, legal)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fh_core::{Event, Game, Rules};

    /// The network plays whole hands legally, and quickly.
    #[test]
    fn plays_whole_hands_legally_and_fast() {
        let Some(bot) = NeuralBot::new("test", 1) else { return }; // no model embedded
        let (mut plays, mut time) = (0, std::time::Duration::ZERO);
        for seed in 0..20u64 {
            let mut game = Game::new(Rules::default(), seed);
            let mut others: [Box<dyn Bot>; 4] = std::array::from_fn(|i| crate::make_bot("steady", seed * 4 + i as u64).unwrap());
            loop {
                let seat = game.to_act().unwrap();
                let obs = game.observe(seat);
                let legal = game.legal_actions();
                let action = if obs.phase == Phase::Playing && legal.len() > 1 {
                    let start = std::time::Instant::now();
                    let a = bot.play(&obs, &legal);
                    time += start.elapsed();
                    plays += 1;
                    assert!(legal.contains(&a));
                    a
                } else {
                    others[seat.index()].choose(&obs, &legal)
                };
                if game.apply(action).unwrap().iter().any(|e| matches!(e, Event::HandScored { .. } | Event::GameWon { .. })) {
                    break;
                }
            }
        }
        assert!(plays > 100);
        let per = time / plays;
        assert!(per < std::time::Duration::from_millis(20), "{per:?} per card");
        eprintln!("{plays} neural plays, {per:?} each");
    }
}
