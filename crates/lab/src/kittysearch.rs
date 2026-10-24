//! `fh-lab kittysearch`: should the coach choose the kitty by searching likely deals, rather than
//! trusting the set plan? South declares a suit at the count's level (as in `fh-lab declarer`).
//! Candidates: the set plan's best few sets, and every set that empties a suit. The search scores
//! each on deals sampled from what South can see (points, as the bots value a hand) and picks the
//! best. Where it differs from the set plan, both are solved on the real deal.

use fh_ai::belief::Belief;
use fh_ai::rule_bot::{ranked_discards, set_discards, KittyWeights};
use fh_ai::solver::{Position, Solver};
use fh_core::game::score_hand;
use fh_core::rules::is_trump;
use fh_core::{effective_suit, Action, Call, Card, CardSet, Contract, Game, Rng, Rules, Seat, Strain, Suit, Trick};

pub fn run(deals: u64, seed: u64, threads: u64, worlds: usize, top: usize) {
    let parts: Vec<[f64; 8]> = std::thread::scope(|scope| {
        let handles: Vec<_> = (0..threads)
            .map(|t| {
                scope.spawn(move || {
                    let mut out = [0.0; 8];
                    let mut d = t;
                    while d < deals {
                        one(seed + d, &mut out, worlds, top);
                        d += threads;
                    }
                    out
                })
            })
            .collect();
        handles.into_iter().map(|h| h.join().unwrap()).collect()
    });
    let mut s = [0.0; 8];
    for p in parts {
        for i in 0..8 {
            s[i] += p[i];
        }
    }
    let [declared, n, search, plan, search_made, plan_made, sq, candidates] = s;
    let mean = (search - plan) / n.max(1.0);
    let sd = (sq / n.max(1.0) - mean * mean).max(0.0).sqrt();
    println!("{deals} deals, {declared} declared, {:.1} candidates each on {worlds} likely deals; the search differs from the set plan on {n} ({:.0}%)\n", candidates / declared.max(1.0), 100.0 * n / declared.max(1.0));
    println!("| Plan | Tricks | Bid made |\n|---|---|---|");
    println!("| The set plan | {:.2} | {:.0}% |", plan / n.max(1.0), 100.0 * plan_made / n.max(1.0));
    println!("| The search | {:.2} | {:.0}% |", search / n.max(1.0), 100.0 * search_made / n.max(1.0));
    println!("\nSearch minus plan: {mean:+.3} tricks (95% ± {:.3}) where they differ; {:+.3} per declared hand", 1.96 * sd / n.max(1.0).sqrt(), (search - plan) / declared.max(1.0));
}

fn one(seed: u64, out: &mut [f64; 8], samples: usize, top: usize) {
    let rules = Rules::default();
    let mut game = Game::with_dealer(rules.clone(), seed, Seat::EAST);
    let obs = game.observe(Seat::SOUTH);
    let Some((strain, count)) = Strain::ALL.into_iter().filter(|s| s.trump().is_some()).map(|s| (s, fh_ai::thumb::count(&obs, s).0)).max_by(|a, b| a.1.total_cmp(&b.1)) else { return };
    if count < 7.0 {
        return;
    }
    let level = if count >= 8.5 { 8 } else { 7 };
    let contract = Contract::new(level, strain);
    let trump = strain.trump();
    game.apply(Action::Call(Call::Bid(contract))).unwrap();
    for _ in 0..3 {
        game.apply(Action::Call(Call::Pass)).unwrap();
    }
    out[0] += 1.0;
    let obs = game.observe(Seat::SOUTH);
    let thirteen = obs.hand;
    let plan: Vec<Card> = set_discards(thirteen, trump, 3);
    let key = |v: &[Card]| -> CardSet { v.iter().copied().collect() };
    let mut candidates: Vec<CardSet> = ranked_discards(thirteen, trump, 3, KittyWeights::DEFAULT).iter().take(top).map(|v| key(v)).collect();
    // Every set that empties a side suit (the rest from the plan's ranking).
    let trumpish = |c: Card| is_trump(c, trump) || c.is_joker();
    for suit in Suit::ALL.into_iter().filter(|&s| Some(s) != trump) {
        let cards: CardSet = thirteen.iter().filter(|&c| !trumpish(c) && effective_suit(c, trump) == Some(suit)).collect();
        if cards.is_empty() || cards.len() > 3 {
            continue;
        }
        if let Some(set) = ranked_discards(thirteen, trump, 3, KittyWeights::DEFAULT).into_iter().map(|v| key(&v)).find(|s| cards.iter().all(|c| s.contains(c))) {
            if !candidates.contains(&set) {
                candidates.push(set);
            }
        }
    }
    out[7] += candidates.len() as f64;
    let belief = Belief::new_with(&obs, false, true);
    let worlds = belief.sample_fitting(&mut Rng::new(seed ^ 0x5EA), thirteen, samples);
    let mut solver = Solver::with_limit(trump, 4_000_000);
    let points = |tricks: u8| {
        let mut t = [0u8; 2];
        t[0] = tricks;
        t[1] = 10 - tricks;
        let change = score_hand(&rules, contract, Seat::SOUTH.team(), t, [0, 0]);
        (change[0] - change[1]) as f64
    };
    let mut best = (f64::MIN, candidates[0]);
    for &set in &candidates {
        let mut total = 0.0;
        let mut w = 0.0;
        for world in &worlds {
            let mut hands = world.hands;
            hands[0] = thirteen.minus(set);
            let t = solver.solve(&Position { hands, trump, trick: Trick::new(Seat::SOUTH) }, Seat::SOUTH.team());
            if solver.aborted() {
                continue;
            }
            // Real play about half a trick below the solver.
            total += world.weight * 0.5 * (points(t) + points(t.saturating_sub(1)));
            w += world.weight;
        }
        let value = total / w.max(1e-9);
        if value > best.0 + 1e-9 {
            best = (value, set);
        }
    }
    let chosen = best.1;
    let plan_set = key(&plan);
    if chosen == plan_set {
        return;
    }
    let real: [CardSet; 4] = std::array::from_fn(|i| game.hand(Seat::new(i)));
    let mut solve = |d: CardSet| -> Option<u8> {
        let mut hands = real;
        hands[0] = thirteen.minus(d);
        let t = solver.solve(&Position { hands, trump, trick: Trick::new(Seat::SOUTH) }, Seat::SOUTH.team());
        (!solver.aborted()).then_some(t)
    };
    let (Some(a), Some(b)) = (solve(chosen), solve(plan_set)) else { return };
    let made = |t: u8| if t > level { 1.0 } else if t == level { 0.5 } else { 0.0 };
    out[1] += 1.0;
    out[2] += a as f64;
    out[3] += b as f64;
    out[4] += made(a);
    out[5] += made(b);
    out[6] += (a as f64 - b as f64).powi(2);
}
