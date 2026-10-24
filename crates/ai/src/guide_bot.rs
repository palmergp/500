//! The guide's bidding (`docs/rules-of-thumb.md`), as a bot, to measure what the written rules are
//! worth: bid by the count (7 from about 7, or 6¾ in clubs, diamonds, hearts and no trumps; 8 from
//! about 8½; never 9), over an opponent only from 8½, call 6 with the Joker and mention a bower
//! when you don't bid and partner hasn't called. Card play and the kitty are The Counter's, so a
//! duplicate match against The Oracle (the same play, the bidding network) isolates the bidding.

use crate::search_bot::{SearchBot, SearchConfig};
use crate::thumb;
use crate::Bot;
use fh_core::cards::ACE;
use fh_core::{Action, Call, Card, Contract, Observation, Phase, Strain, Suit};

/// Which of the guide's rules the bot follows (lab variants drop one).
#[derive(Clone, Copy, Debug)]
pub struct GuideRules {
    /// Adjust the count for the auction (rule 4); otherwise count the cards and add the constant.
    pub auction: bool,
    /// Stop at 8 (finding 32); otherwise the old "9 from about 10".
    pub stop_at_eight: bool,
    /// Over an opponent's bid, bid only from about 8½; otherwise whenever the count reaches the level.
    pub cautious_overcalls: bool,
    /// Call 6 with the Joker and mention a bower when not bidding (before partner has called).
    pub signals: bool,
}

impl GuideRules {
    pub const ALL: GuideRules = GuideRules { auction: true, stop_at_eight: true, cautious_overcalls: true, signals: true };
}

pub struct GuideBot {
    name: String,
    rules: GuideRules,
    play: SearchBot,
    note: String,
}

impl GuideBot {
    pub fn new(name: &str, rules: GuideRules, seed: u64) -> GuideBot {
        GuideBot { name: name.to_string(), rules, play: SearchBot::new(name, SearchConfig::COUNTER, seed), note: String::new() }
    }

    fn count(&self, obs: &Observation, strain: Strain) -> f32 {
        if self.rules.auction {
            thumb::count(obs, strain).0
        } else {
            thumb::card_count(obs.hand, strain) + thumb::constant(strain)
        }
    }

    /// The highest level the count supports in `strain`, if any.
    fn level(&self, strain: Strain, count: f32) -> Option<u8> {
        let seven = if strain == Strain::Spades { 7.0 } else { 6.75 };
        if !self.rules.stop_at_eight && count >= 10.0 {
            Some(9)
        } else if count >= 8.5 {
            Some(8)
        } else if count >= seven {
            Some(7)
        } else {
            None
        }
    }

    fn call(&mut self, obs: &Observation, legal: &[Action]) -> Action {
        let is_legal = |c: Call| legal.contains(&Action::Call(c));
        let mut strains: Vec<(Strain, f32)> = Strain::ALL.into_iter().map(|s| (s, self.count(obs, s))).collect();
        strains.sort_by(|a, b| b.1.total_cmp(&a.1));
        let me = obs.seat;
        let high = obs.auction.high_bid();
        let over_opponent = high.is_some_and(|(s, _)| s.team() != me.team());
        for &(strain, count) in &strains {
            let Some(top) = self.level(strain, count) else { continue };
            if over_opponent && self.rules.cautious_overcalls && count < 8.5 {
                continue;
            }
            // The cheapest legal bid in this strain, up to the count's level.
            if let Some(level) = (7..=top).find(|&l| is_legal(Call::Bid(Contract::new(l, strain)))) {
                self.note = format!("guide: {strain} count {count:.2}, bid {level}");
                return Action::Call(Call::Bid(Contract::new(level, strain)));
            }
        }
        let best = strains[0].0;
        if obs.auction.must_bid() {
            let forced = legal.iter().find(|a| matches!(a, Action::Call(Call::Bid(c)) if c.strain == best)).or_else(|| legal.iter().find(|a| matches!(a, Action::Call(Call::Bid(_)))));
            return *forced.expect("a forced bid exists");
        }
        let partner_called = obs.auction.calls().any(|(s, _)| s == me.partner());
        if self.rules.signals && !partner_called {
            if obs.hand.contains(Card::JOKER) && is_legal(Call::Six(best)) {
                return Action::Call(Call::Six(best));
            }
            for &(strain, _) in &strains {
                let shows = match strain.trump() {
                    Some(suit) => crate::evaluate::has_bower(obs.hand, suit),
                    None => Suit::ALL.iter().filter(|&&s| obs.hand.contains(Card::new(s, ACE))).count() >= 2,
                };
                if shows && is_legal(Call::Mention(strain)) {
                    return Action::Call(Call::Mention(strain));
                }
            }
        }
        Action::Call(Call::Pass)
    }
}

impl Bot for GuideBot {
    fn name(&self) -> &str {
        &self.name
    }

    fn choose(&mut self, obs: &Observation, legal: &[Action]) -> Action {
        match obs.phase {
            Phase::Bidding => self.call(obs, legal),
            _ => self.play.choose(obs, legal),
        }
    }

    fn note(&self) -> Option<&str> {
        Some(&self.note)
    }
}
