# Strategy lab: findings

Measurements from `fh-lab` under the house rules (`docs/rules/house-rules.md`). Each entry says how it was
measured, so it can be re-run. "Duplicate" means every deal is played twice with the partnerships swapped,
which cancels most of the luck of the cards.

## Tools

| Command | What it measures |
|---|---|
| `fh-lab sim --games N --ns A --ew B` | Whole games between two bots, with every rule invariant checked after every action |
| `fh-lab duel --a A --b B --deals N` | Duplicate match: points per hand that bot A gains over bot B, with a 95% confidence interval |
| `fh-lab calibrate --deals N` | How well simulated double-dummy bidding predicts real play |
| `fh-lab bench --deals N` | Double-dummy solver speed on full ten-card deals |

## Findings (2026-10-17)

### 1. Simple bots bid far too timidly under these rules

*Measured by `sim` with 20,000 games per matchup.*

| Matchup (rule bots) | Win rate |
|---|---|
| Steady vs Cautious | Steady 74% |
| Bold vs Cautious | Bold 61% |
| Bold vs Steady | about even (48–52%) |

Even Steady passes out 47% of deals. Once-around bidding with defensive points rewards bidding. Why:

- A passed-out deal scores nothing for anyone.
- A made 7-level contract is worth 140–220.

### 2. A rough trick count underrates hands by about a trick

*Measured by `calibrate` with 120 deals.* South speaks first and declares its best strain at the 7 level.

| Estimate | Tricks |
|---|---|
| Rough evaluation + 2 (the rule bots' credit for partner and kitty) | 5.66 |
| Simulated double-dummy prediction | 7.16 |
| True double-dummy, actual cards | 7.07 |
| Actual play by search bots | 6.62 |

- The simulation is unbiased against true double-dummy.
- Double-dummy runs about **half a trick above real play**, because real players can't see the hidden cards.
- The search bot now subtracts 0.5 tricks when bidding (`dd_bias`).
- The rule-of-thumb lesson: count your own tricks and add **about 3** for partner and the kitty, not 2.

### 3. Search card play beats rule-of-thumb play

*Measured by `duel` with 48 deals: search play vs Steady, both using Steady's bidding.*

- **+14.5 points per hand** (95% CI −4 to +33).
- Search bots made 79% of their contracts; Steady made 73%.

### 4. Trusting partner's bid matters

The first search bidder sampled its partner's hidden hand without regard to what partner had bid. As a result it often "rescued" partner into its own suit, and went down.

Two fixes:

- **Rejection sampling:** keep each sampled world with a probability equal to how well it fits every call.
- **A higher bar for switching:** require an extra 80-point edge before switching away from partner's strain.

Contracts made rose from 39% to 66–67%.

### 5. The first full search bot vs Steady

The first Counter bid by simulation. *Measured by `duel`.*

| Version | Deals | Gain per hand (95% CI) |
|---|---|---|
| Fast settings | 200 | +12.4 (−7.7 to +32.6) |
| Browser settings | 400 | +5.7 (−8.2 to +19.6) |

The edge was positive but not proven, so findings 6 and 7 below take it apart.

### 6. The best rule bidder adds about 0.7 tricks to Steady's estimates

*Measured by `sim` with 40,000 games each: Steady with an aggression offset vs plain Steady.*

| Offset | −0.3 | +0.2 | +0.4 | +0.6 | +0.8 | +1.0 |
|---|---|---|---|---|---|---|
| Win rate vs Steady | 43.4% | 53.3% | 56.5% | **57.5%** | 57.2% | 55.1% |

Head to head, the optimum is flat around +0.6 to +0.9:

| Matchup | Win rate for +0.6 |
|---|---|
| +0.6 vs +0.3 | 52.7% |
| +0.6 vs +0.9 | 50.2% |
| +0.6 vs +1.2 | 53.8% |

The expert bots use +0.7.

**Lesson for players:** bid a little more than feels safe. In trick terms, credit partner and the kitty with about 2.7 tricks.

### 7. Card play is where search wins today; simulated bidding loses

*Measured by `duel` with 400 deals each.*

| Comparison | Gain per hand (95% CI) | Declared | Made |
|---|---|---|---|
| Search play vs Steady (both bid like Steady) | **+9.4** (+0.7 to +18.2) | 50% of hands | 82% (vs Steady's 78%) |
| Search bidding vs Steady's bidding (both play by search) | **−27.0** (−41.8 to −12.1) | 84% of hands | 58% |

With 5–10 sampled worlds per bid, the estimates are too noisy. Choosing the best of several noisy options also overstates its value. So the expert now bids by the calibrated rule (+0.7) and searches only card play and the kitty.

A learned hand evaluator, trained on millions of solver results, is the planned fix (Phase 5).

### 8. The expert bots now clearly beat the rule bots

*Measured by `duel`.* The Counter and The Shark both bid by the +0.7 rule and play by search. The Shark also bids at +1.0 and plays to maximize its chance of winning the game.

| Comparison | Deals | Gain per hand (95% CI) |
|---|---|---|
| **The Counter vs Steady** | 400 | **+29.0** (+15.9 to +42.1) |
| **The Shark vs Steady** | 200 | **+23.4** (+1.5 to +45.4) |
| Searched kitty discards vs the heuristic (both Counters) | 300 | +0.1 (−8.5 to +8.6) |

- The Counter declared 70% of hands and made 75% of them. Steady made 78% of its own contracts.
- Searched discards add nothing measurable, so the Counter uses the faster heuristic: void a short side suit, keep aces, guarded kings and all trumps.
- The searched version stays available as `counter-search-kitty`.

### 9. A neural bidding evaluator beats the tuned rule

*Measured by `fh-lab dataset`, `train/train_bid.py` and `duel`.*

**The model:** a two-layer network (256 units).
- **Inputs:** one bidder's hand, position and the calls so far.
- **Output:** a trick distribution for each strain.
- **Training data:** 64,000 decisions from 16,000 deals, each labelled with the double-dummy result for every strain.

On held-out decisions:

| | Mean error |
|---|---|
| The network | **1.53 tricks** |
| Strain-average baseline | 2.00 tricks |
| Calibration (per strain) | within 0.07 tricks |

Much of the remaining error is irreducible: the bidder can't see partner's hand or the kitty.

The Oracle bids from these distributions and plays like The Counter:

| Matchup (duplicate) | Deals | Gain per hand (95% CI) |
|---|---|---|
| Oracle vs Counter | 400 | +11.2 (−2.2 to +24.5) |
| Oracle vs Counter, new deals | 400 | +8.1 (−5.0 to +21.2) |
| **Pooled** | 800 | **+9.7** (about +0.3 to +19) |
| **After self-play retraining**: same 400 deals as the first row | 400 | **+12.4** (−0.5 to +25.4) |

**Self-play retraining:** 12,000 more deals with The Oracle bidding at all four seats, so partner's calls carry the Oracle's own meaning, then retraining on all 112,000 decisions.
- Mean error went from 1.528 to 1.521 tricks; every strain is now calibrated within 0.04 tricks.
- The make rate rose from 71% to 74%. That version is the one embedded now.

- The Oracle declares about 68% of hands and makes 71%. The Counter declares 32% and makes 77%.
- So the Oracle wins by competing for more contracts, at a slightly lower make rate.

### 10. Measured bidding values and expert habits

These now live in `docs/rules-of-thumb.md`. They come from `train/rules_of_thumb.py` on the same data, and from `fh-lab habits` (The Counter in all four seats, 600 hands).

### 11. The discard signal costs expert bots nothing measurable

*Measured by `duel` with 400 deals: the Oracle with the discard signal vs the Oracle without.*

**The convention:** the first throw-off comes from the strongest side suit. Search bots may give up as much as 25 expected points (all-cards-visible) to make it, because that analysis can't value information.

**Result: −0.8 points a hand** (95% CI −5.9 to +4.4). Both sides declared 50% and made 76%.

Neutral for experts, so the bots keep it: it's the house convention, and human partners rely on it.

### 12. Opening leads, scored

*Measured by `fh-lab leads` over 600 deals.*

**Method:** at the declarer's opening lead and the defenders' first lead, every legal lead was scored on the same 32 sampled worlds. Each type of lead got a "best" rate and a cost: the average expected points it gives up against the best lead. The Oracle played the auctions; full tables are in `train/data/leads.txt` (regenerate with the command).

**Findings:**

- **Declarer with 5+ trumps:** the top trump is the best lead 70–74% of the time (cost 2–4). A side ace costs 7–8; a low trump 24–32.
- **Declarer with 4 or fewer trumps:** the side ace is best (cost 7).
- **Declarer holding the Joker:** the next-highest trump costs 4.7, the Joker itself 9.1.
- **Partner called 6:** a small trump was best 62% of the time (n = 16).
- **No trumps:** side winners are best 87% of the time; leading the Joker costs 49.
- **Defenders in a suit contract:** a side ace costs 8 (best 68%), a low side card 18, partner's mentioned suit 20, a low trump 44.

**Correction:** finding 10's habit-based "cash a side ace first" holds only with four or fewer trumps.

### 13. Suit or no trumps, and length or strength

*Measured by `train/strain_choice.py` on 107,000 bidding hands*, each solved in every strain. Scores use the 7 level and the real-play correction.

**No trumps against the bidder's longest trump suit:**

| You hold | NT better |
|---|---|
| No Joker | about 30% |
| The Joker | 55–80% |
| Joker + 2 aces + no suit longer than 4 | 79% |
| 0 aces | 22–28% |
| 2 aces | about 50% |
| 3+ aces | 63% |
| Joker, 8 level | 44% |
| No Joker, 8 level | 20% |

- Stoppers change the figure only a few points once aces are accounted for.
- The long suit's colour barely matters: 57% for red, 59% for black, with the Joker.

**Length or strength, in suit contracts:**
- A top trump (the Joker or a bower) is worth about 0.7 tricks more than a small trump, at any length.
- An extra small trump adds 0.48 tricks with none of the top three, 0.32 with one, 0.26 with two.

**In no trumps:** a long suit without the ace yields fewer tricks as it grows (5.1 at four cards, 4.3 at seven or more). With A-K at the head, length barely helps (6.3–6.4).

## Open questions for the lab

- **Bid estimates:** they rest on few sampled worlds, so they are noisy. A fitted evaluator, trained on solver output, would be faster and steadier. This is planned for the Phase 5 neural work.
- **Signals:** what a mention or a 6-call is worth, compared with what it tells the opponents.
- **Seat position:** what a once-around auction implies for each seat.
- **The forced dealer:** strategy on the third deal.
- **Score-aware bidding:** when defenders can go out on tricks.
- **The playbook:** the 30 hypotheses in `docs/research/02-strategy-playbook.md` still need checking under these rules.
