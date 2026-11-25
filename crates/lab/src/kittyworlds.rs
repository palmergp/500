//! `fh-lab kittyworlds`: should the coach's kitty grades read the auction? A grade compares the
//! plan's discards with another set on deals sampled from the declarer's view. Those deals either
//! ignore the calls (as the search's card play does, finding 21) or are drawn to fit them (partner's
//! bid, mentions and passes). Over Oracle-played deals, at the declarer's discard, each way predicts
//! the gap between the plan and an alternative, and the bid's chance with the plan; both are
//! checked against the real deal, solved with every card seen. The closer prediction wins.

use fh_ai::belief::Belief;
use fh_ai::rule_bot::{heuristic_discards, ranked_discards, KittyWeights};
use fh_ai::search_bot::{SearchBot, SearchConfig};
use fh_ai::solver::{Position, Solver};
use fh_ai::Bot;
use fh_core::game::score_hand;
use fh_core::{Call, Card, CardSet, Contract, Game, Phase, Rng, Rules, Seat, Trick};

#[derive(Default, Clone, Copy)]
struct Tally {
    n: f64,
    /// Squared and absolute error of the predicted gap (plan − alternative, points), and how
    /// often its sign matched the real one where both were non-zero.
    sq: f64,
    abs: f64,
    sign_n: f64,
    sign_ok: f64,
    /// Brier score of the plan's predicted chance to make the bid.
    brier: f64,
    /// The predicted chance, and how often the bid really made (double dummy).
    predicted: f64,
    made: f64,
}

impl Tally {
    fn add(&mut self, o: &Tally) {
        self.n += o.n;
        self.sq += o.sq;
        self.abs += o.abs;
        self.sign_n += o.sign_n;
        self.sign_ok += o.sign_ok;
        self.brier += o.brier;
        self.predicted += o.predicted;
        self.made += o.made;
    }
}

/// Tallies by (who called: "partner called" / "partner passed", reading the auction or not).
type Out = std::collections::BTreeMap<(String, bool), Tally>;

const WORLDS: usize = 20;

pub fn run(deals: u64, seed: u64, threads: u64) {
    let parts: Vec<Out> = std::thread::scope(|scope| {
        let handles: Vec<_> = (0..threads)
            .map(|t| {
                scope.spawn(move || {
                    let mut out = Out::new();
                    let mut d = t;
                    while d < deals {
                        deal(seed + d, &mut out);
                        d += threads;
                    }
                    out
                })
            })
            .collect();
        handles.into_iter().map(|h| h.join().unwrap()).collect()
    });
    let mut all = Out::new();
    for part in parts {
        for (k, t) in part {
            all.entry(k).or_default().add(&t);
        }
    }
    let mut totals: std::collections::BTreeMap<bool, Tally> = Default::default();
    for ((_, read), t) in &all {
        totals.entry(*read).or_default().add(t);
    }
    println!("{deals} deals. At the declarer's discard, the plan against an alternative, predicted on {WORLDS} sampled deals that");
    println!("ignore the auction or read it, and checked against the real deal (every card seen).\n");
    println!("| Partner | Deals | n | Gap error (RMS, points) | Mean abs. error | Sign right | Make chance: predicted / real | Brier |");
    println!("|---|---|---|---|---|---|---|---|");
    let row = |who: &str, read: bool, t: &Tally| {
        let n = t.n.max(1.0);
        println!(
            "| {who} | {} | {} | {:.0} | {:.0} | {:.0}% | {:.0}% / {:.0}% | {:.3} |",
            if read { "read the auction" } else { "ignore it" },
            t.n / 2.0,
            (t.sq / n).sqrt(),
            t.abs / n,
            100.0 * t.sign_ok / t.sign_n.max(1.0),
            100.0 * t.predicted / n,
            100.0 * t.made / n,
            t.brier / n
        );
    };
    for ((who, read), t) in &all {
        row(who, *read, t);
    }
    for (read, t) in &totals {
        row("all", *read, t);
    }
}

/// The declaring side's points from `tricks` (its score change less the other side's).
fn points(rules: &Rules, contract: Contract, declarer: Seat, tricks: u8) -> f64 {
    let team = declarer.team();
    let mut won = [0u8; 2];
    won[team.index()] = tricks;
    won[1 - team.index()] = 10 - tricks;
    let change = score_hand(rules, contract, team, won, [0, 0]);
    (change[team.index()] - change[1 - team.index()]) as f64
}

fn deal(seed: u64, out: &mut Out) {
    let mut game = Game::new(Rules::default(), seed);
    let mut bots: [SearchBot; 4] = std::array::from_fn(|i| SearchBot::new("The Oracle", SearchConfig::ORACLE, seed * 4 + i as u64));
    loop {
        let Some(seat) = game.to_act() else { return };
        if game.phase() == Phase::Discarding {
            let (declarer, contract) = game.contract().expect("a contract");
            let trump = contract.trump();
            let obs = game.observe(declarer);
            let hand = obs.hand;
            let plan = heuristic_discards(hand, trump, 3);
            let mut rng = Rng::new(seed ^ 0xc0ffee);
            // Two alternatives: the next set the kitty heuristic ranks, and a random swap of one
            // card (anything from a near tie to putting back a trump).
            let ranked = ranked_discards(hand, trump, 3, KittyWeights::DEFAULT);
            let near = ranked.into_iter().find(|s| set(s) != set(&plan));
            let kept: Vec<Card> = hand.minus(set(&plan)).iter().collect();
            let mut swap = plan.clone();
            swap[rng.below(3)] = kept[rng.below(kept.len())];
            let alternatives: Vec<Vec<Card>> = near.into_iter().chain(std::iter::once(swap)).collect();
            let partner = declarer.partner();
            let called = obs.auction.calls().any(|(s, c)| s == partner && c != Call::Pass);
            let who = if called { "called" } else { "passed" };
            let rules = game.rules().clone();
            let mut solver = Solver::with_limit(trump, 3_000_000);
            let mut solve = |hands: [CardSet; 4], discards: &[Card]| {
                let mut hands = hands;
                hands[declarer.index()] = hand.minus(set(discards));
                solver.solve(&Position { hands, trump, trick: Trick::new(declarer) }, declarer.team())
            };
            let real: [CardSet; 4] = std::array::from_fn(|i| game.hand(Seat::new(i)));
            let real_plan = solve(real, &plan);
            for read in [false, true] {
                let belief = Belief::new_with(&obs, read, true);
                let worlds = belief.sample_fitting(&mut Rng::new(seed ^ 0x5a5a), hand, WORLDS);
                let plan_tricks: Vec<u8> = worlds.iter().map(|w| solve(w.hands, &plan)).collect();
                let chance = plan_tricks.iter().filter(|&&t| t >= contract.level).count() as f64 / worlds.len().max(1) as f64;
                let real_made = f64::from(u8::from(real_plan >= contract.level));
                for alt in &alternatives {
                    let predicted = worlds
                        .iter()
                        .zip(&plan_tricks)
                        .map(|(w, &p)| points(&rules, contract, declarer, p) - points(&rules, contract, declarer, solve(w.hands, alt)))
                        .sum::<f64>()
                        / worlds.len().max(1) as f64;
                    let truth = points(&rules, contract, declarer, real_plan) - points(&rules, contract, declarer, solve(real, alt));
                    let t = out.entry((format!("partner {who}"), read)).or_default();
                    t.n += 1.0;
                    t.sq += (predicted - truth).powi(2);
                    t.abs += (predicted - truth).abs();
                    if truth != 0.0 && predicted.abs() > 1.0 {
                        t.sign_n += 1.0;
                        t.sign_ok += f64::from(u8::from((predicted > 0.0) == (truth > 0.0)));
                    }
                    t.brier += (chance - real_made).powi(2);
                    t.predicted += chance;
                    t.made += real_made;
                }
            }
            return;
        }
        let action = bots[seat.index()].choose(&game.observe(seat), &game.legal_actions());
        if game.apply(action).is_err() {
            return;
        }
    }
}

fn set(cards: &[Card]) -> CardSet {
    cards.iter().copied().collect()
}
