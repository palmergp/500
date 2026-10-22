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

### 14. How high to bid, by seat: competition doesn't make 9 pay

*Measured by `fh-lab levels` over 20,000 deals (19,874 bidding decisions).*

**Method:** for the first player in each deal with a count of 6½ or more in some suit, every option was played
out: pass, and 7, 8, 9 and 10 in that suit. The Oracle finished the auction for everyone. The declarer took the
kitty and discarded heuristically. The hand was solved with every card visible, scored for real play at half a
trick below the solver. Values are points for the bidder's side minus the other side's, averaged per count.

**The best option, by count and seat (no opponent has bid yet):**

| Count | First to speak | Second | Third |
|---|---|---|---|
| 7 | 7 (+17) | 7 (+76) | 7 (−10) |
| 8 | 7 (+94; 8: +69) | 7 (+126; 8: +103) | 7 (+95; 8: +80) |
| 8½ | 8 (+118; 7: +113) | 8 (+154; 7: +147) | 8 (+140; 7: +127) |
| 9 | 8 (+157; 9: +16) | 8 (+189; 9: +56) | 8 (+210; 9: +137) |
| 9½ | 8 (+212; 9: +98) | 8 (+229; 9: +153) | — |

- **A 9 bid was never the best option at any count up to 10, from any seat.** Competition (players still to call
  who might outbid you) doesn't change the no-competition answer from rule 2 in `docs/rules-of-thumb.md`.
- **Going up to 8 pays from about 8½** while others can still call, slightly earlier than with no competition.
- **After an opponent's bid,** passing beat overcalling at 8 up to a count of 8, and overcalling paid from about
  8½ (n = 902).
- Last to speak is muddied by partner's bids (7 is often no longer available), so it isn't summarized.

### 15. The coach's percentages were too pessimistic: now corrected

*Measured by `fh-lab honesty` over 2,000 hands, with expert (Oracle) play on both sides.*

At sampled decisions the coach gave its chance that the bid would be made; at the end of the hand we recorded
whether it was.

| Coach said (card play) | Actually made | Coach said (bids) | Actually made |
|---|---|---|---|
| 0–10% | 7% | 20–30% | 33% |
| 10–20% | 48% | 30–40% | 48% |
| 20–30% | 62% | 40–50% | 59% |
| 40–50% | 72% | 50–60% | 69% |
| 60–70% | 76% | 60–70% | 74% |
| 80–90% | 86% | 80–90% | 92% |
| 90–100% | 97% | 90–100% | 94% |

- **Card play:** the coach's numbers come from deals solved as if both sides played perfectly. Real defenders
  aren't perfect, so low chances were far too low: a bid the coach called 15% was made about half the time.
- **Bids:** about 10 points low across the range.
- **Fix:** the coach now shows real-play chances, mapped through these measured curves (`real_play_chance` in
  `crates/ai/src/coach.rs`). Its choices are unchanged: they rank plays by expected points, and the bias is
  mostly uniform across a decision's options. The curves describe expert play; a casual player's results will
  be a little lower.
- **Checked on 800 fresh deals** (`fh-lab honesty --seed 900000`): with the curves, the stated chance averaged 73%
  and 73% were made, for both bids and card play. Each 10% band now matches within its sampling error (the
  10–50% bands of card play have only 17–50 predictions each).

### 16. The kitty, solved

*Measured by `fh-lab declarer` over 12,000 deals (4,137 where South declares).* South declared at the level
the rule of thumb gives. Every way of putting back three side cards (and one that put back the lowest trump)
was solved with every card visible, and the best set of each kind was compared with the best of the other kind
on the same deal.

| Question | Tricks | First better | Second better |
|---|---|---|---|
| Keep every trump, or put one back? | 8.37 vs 7.67 | 57% | 0% |
| Keep the aces, or put one back? | 8.41 vs 7.94 | 40% | 0% |
| Make a void, or not? | 8.29 vs 7.90 | 39% | 5% |
| Keep a guarded king, or not? | 8.12 vs 8.15 | 16% | 18% |

- The void is worth more with more trumps to ruff with: +0.33 tricks with 5 or fewer, +0.43 with 6, +0.52 with 7+.
- The coach's usual plan was 0.29 tricks short of the best set with hindsight. The best set uses knowledge of the
  hidden cards, so not all of that gap can be closed.
- A guarded king is no reason to keep a suit: the advice to "keep guards" is withdrawn.

### 17. The coach is fair and steady

*Measured by `fh-lab audit` over 150 deals for each South player, with The Oracle in the other seats.*

| South | Card play: Best / Good / Inaccuracy / Mistake / Blunder | Bidding: Mistake / Blunder |
|---|---|---|
| The Oracle (expert) | 84% / 9% / 5% / 1% / 0% | 0% / 0% (the coach bids as it does) |
| Steady | 82% / 10% / 5% / 1% / 2% | 18% / 8% |
| Rookie | 77% / 11% / 5% / 3% / 3% | 15% / 7% |

- **Fair:** an expert's card play is almost never called a mistake, and never a blunder. A beginner collects
  about six times as many. The cut-offs aren't too harsh.
- **Steady:** the hint asked twice from different random starts gave the same card 86% of the time. Another 11–12%
  differed only between plays each named as a close call, and 2% conflicted.
- Kitty grades are all "best" here because these bots discard by the coach's own plan: no information.

### 18. The declarer's first lead depends on the top trumps

*Measured by `fh-lab declarer --leads-only` over 12,000 deals (4,137 declared).* After the usual discards,
every opening lead was solved with every card visible, and each kind of lead was compared with leading the top
trump on the same deal (finding 12 compared them on sampled deals, without pairing by holding).

| Top trumps missing (of Joker, both bowers, ace) | Side ace | Singleton | K of K-Q | Low side card | Low trump |
|---|---|---|---|---|---|
| 0–1 | −0.06 to −0.12 | −0.01 to −0.08 | −0.18 to −0.23 | −0.23 to −0.27 | −0.54 to −0.56 |
| 2 | +0.05 to +0.08 | +0.08 to +0.11 | +0.03 to +0.04 | −0.11 to −0.15 | −0.14 to −0.17 |
| 3–4 | +0.02 to +0.27 | +0.09 to +0.37 | +0.16 | −0.17 to +0.06 | −0.01 to +0.13 |

Tricks gained against leading the top trump, ranges across trump lengths (5 or fewer, 6, 7+). The low trump is
the worst lead whenever you hold the top trumps.

- A "void-first" kitty plan (the most voids, keeping aces) made no difference against the usual plan on deals
  where they differ: 7.89 against 7.87 tricks (n = 1,852).

### 19. What counting and signals are worth to an expert

*Measured by `duel`, 600 deals each played both ways: the Oracle against itself with one skill removed.*

| The Oracle without… | The full Oracle gains per hand | 95% range |
|---|---|---|
| Tracking who has shown out of a suit | **+12.0** | +5.4 to +18.5 |
| Mentions and 6-calls (passing instead) | +3.7 | −1.5 to +8.9 |
| Reading what calls show (no auction inference) | −1.4 | −8.4 to +5.7 |

- **Counting voids is clearly worth it:** bids were made 73% of the time against 68%.
- **Signal calls:** probably a small gain, not yet separable from noise. The full Oracle declared 52% of hands
  against 48%, so mentions and 6-calls help a side find its bid.
- **Reading the auction for card play:** no measurable value for the bots yet. Their play already narrows the
  hidden cards from what's been played, and the auction adds little on top. (Partner's 6-call still places the
  Joker exactly, a hard constraint the variant keeps.)
- **Longer matches** (1,800 more deals each, seed 20,000 apart):
  - Mentions and 6-calls: **+2.0** (−1.1 to +5.2). Pooled with the first 600 deals, about +2.4 a hand: a small,
    probably real gain. The full Oracle declared 53% of hands against 47%.
  - Reading the auction and partner's signals: **−4.8** (−8.6 to −1.0). The bot plays *better* without it.
    The auction weights in `belief.rs` are rough guesses, so they mislead more than they help. Which part hurts
    (the auction or the discard signals) is being measured next.

### 20. Bidding to the score

*Measured by `fh-lab levels --us N --them N`, 6,000 deals at each score.* As finding 14, but each option is
valued by the bidder's side's chance of winning the game (certain when the hand takes someone to 500, otherwise
estimated from the score gap).

| Count | 300 vs 460: pass / 7 / 8 | 460 vs 300: pass / 7 / 8 | 450 vs 450: pass / 7 / 8 |
|---|---|---|---|
| 6½ | 32 / 41 / **43** | 57 / **59** / 54 | 43 / **52** / 46 |
| 7 | 36 / 46 / **48** | 64 / **66** / 59 | 50 / **59** / 51 |
| 8 | 39 / 58 / **66** | 73 / **81** / 74 | 58 / **76** / 69 |
| 9 | 40 / 66 / **86** | 78 / **90** / 89 | 62 / **88** / 87 |

- **Behind, with the opponents near 500:** bid even light hands, and prefer the level that reaches 500 if made
  (8 here, except in hearts and no trumps, where 7 at 300 already reaches 500).
- **Ahead, near 500:** 7 is enough. Over an opponent's bid, passing was best up to a count of 7, since four
  defensive tricks also win.
- **Both near 500:** bid 7 even light. The declarers count first.
- 9 and 10 were never best at any of these scores.

### 21. The auction weights hurt card play, so they're off

*Measured by `duel`, 1,800 deals each, the Oracle against itself with one part of its reading removed.*

| The Oracle without… | The full Oracle gains per hand | 95% range |
|---|---|---|
| Weighting deals by the auction (what bids, mentions and passes suggest) | −3.1 | −6.8 to +0.7 |
| Reading partner's discard signal | −1.1 | −3.0 to +0.7 |
| Both (finding 19) | −4.8 | −8.6 to −1.0 |

- The harm is almost all in the auction weights: hand-set likelihoods ("a mention comes with a bower 85% of the
  time", "a passer rarely held 8½") that the lab never measured. They pushed the sampled deals away from the real
  ones more than toward them.
- **Change:** card play (The Counter, The Shark, The Oracle and the coach's hints and kitty comparisons) no longer
  weights deals by the auction. Hard facts stay: a 6-call places the Joker, shown-out suits are voids, and the
  discard signal is still read (neutral here, and it's the house convention).
- Measuring the real likelihoods from self-play, and weighting by those, might turn this into a gain.
- **Recalibrated:** without the auction weights the bots defend better and bids are made more often, so the coach's
  card-play percentages ran about 7 points low again. The play curve was refit from 6,200 new predictions and
  checked on 800 fresh deals: 74% stated, 71% made overall (bids: 73% and 71%), within the few points that
  different samples of deals vary by.

### 22. A card-play network: The Natural

*Data from `fh-lab playdata` (the Oracle's search value for every legal card at 600,000 decisions from 20,000
deals); trained by `train/train_play.py`; played by `crates/ai/src/neural_play.rs`.*

**The network:** 585 inputs (cards by role: trump bowers, ace and so on, not printed suit), plus what a player sees
at a glance (which cards would win the trick now, who is winning it, unseen cards per suit). Two hidden layers
(768 and 384), one score per card slot. Stored in 16-bit floats (1.5 MB, no measurable loss).

| | Points given up per decision against the search's best |
|---|---|
| Always the highest legal card | 18.4 |
| First network (523 inputs, 360,000 decisions) | 5.1 |
| **This network** | **4.5** |

**In play (600 duplicate deals each), with The Oracle's bidding and kitty:**

| Match | Per hand | 95% range |
|---|---|---|
| The Natural vs Steady | **+23.4** | +13.0 to +33.9 |
| The Natural vs The Oracle | **−22.6** | −31.0 to −14.1 |

- A strong intermediate player that's instant: about 1 ms a card natively. In the browser no card took over 50 ms,
  against up to a second for the search players. It's now a persona, **The Natural**.
- Imitation stalls well short of the search: more data and richer inputs helped against Steady (+4.6 → +23.4)
  but not against the Oracle (−23 both times).
- **Skipping the search when the network is confident doesn't pay:** it's rarely confident, since many
  positions have several near-equal cards. At 60% confidence it skips only 16% of searches, giving up 1.5
  points each time.
- Growing it further would need search on top of the network (or self-play training), not more imitation.

### 23. The kitty in no trumps: keep stoppers, don't hunt voids

*Measured by `fh-lab declarer --nt` over 12,000 deals (1,496 where South declares no trumps from a no-trump count
of 7 or more).* As finding 16, with aces kept (always right in no trumps).

| Question | Tricks | First better | Second better |
|---|---|---|---|
| Make a void, or not? | 8.75 vs 8.82 | 11% | 14% |
| Keep a guarded king, or not? | 8.89 vs 8.70 | 23% | 9% |
| The new no-trump plan, or the old suit-style plan? (where they differ, n = 659) | 8.49 vs 8.43 | 20% | 19% |
| A "never make a void" plan, or the old plan? (n = 762) | 8.34 vs 8.43 | 15% | 27% |

- Voids neither help nor hurt in no trumps (the Joker stops any suit, and an unstopped suit costs little: see
  finding 13). Guarded kings do help, unlike in a suit bid.
- The bots and the coach now use a no-trump plan (`nt_keep_value` in `crates/ai/src/rule_bot.rs`): keep aces,
  guarded kings and their guards, guarded queens and length under an ace; put back low cards from suits with no
  honours. It's only slightly better than the old plan, but it no longer reasons as if there were trumps. The coach
  no longer says a no-trump void lets you "trump it".
- Simply avoiding voids is worse: the cheapest such sets break guarded kings.
- The plan is still 0.46 tricks short of the best set with hindsight (0.29 in suit bids).

### 24. Bug: bidding over an opponent near the end of a game

The value of passing over an opponent's bid was a fixed estimate in points ("they make it three times in
four"), while bids, near the end of a game and in short games, are valued as the chance of winning the game
(finding 20). Compared in mixed units, nearly any bid looked better than passing. Over an opponent's 8♣ at the
start of a four-hand game, the coach suggested a long-shot bid (under 35% to make) on 275 of 300 hands. The
Shark, which always plays for the game, had the same flaw.

**Fixed:** passing is now valued by the same utility as the bids, from a spread of the opponents' likely tricks
(make 75% of the time, half of those with a trick to spare). Long shots dropped to 7 of 300. The ones left are
mostly near 500 (380 each), where the opponents' bid would win them the game: a deliberate sacrifice, and the hint
now says so (`over_an_opponents_bid_the_coach_doesnt_take_long_shots`).

**Four-hand games are now played for points, not the game** (the user's choice): with no target to race to,
the coach maximizes your score less theirs on each hand, so it never sacrifices. Games to 500 still switch to
playing for the game within 160 of the target.

### 25. Side kings and queens

*Measured by `fh-lab honors` over 4,000 deals of Oracle play in all four seats, and by refitting the bidding count
(`train/rules_of_thumb.py` data) with kings and queens split by how they're held.* Full tables in
`docs/bidding-count.md`.

| Holding | Won a trick (suit / NT) | Worth when bidding (suit / NT) |
|---|---|---|
| King alone | 20% / 32% | +0.08 / +0.20 |
| King with 1 guard | 36% / 51% | +0.13 / +0.32 |
| King with 2+ guards | 23% / 55% | +0.20 / +0.51 |
| King with the ace (on top of the ace) | 53% / 83% | +0.30 / +0.50 |
| Queen with 2+ guards | 14% / 32% | +0.07 / +0.23 |
| Queen with the king | 21% / 42% | +0.08 / +0.20 |

- In suit bids side queens are worth almost nothing (under 0.1), however guarded. In no trumps a queen with two
  guards is a real stopper (¼).
- Long side suits get trumped: in a suit bid a king with three or more guards won only 15%, against 36% with one.
  Yet for bidding it's worth more, since length also stops the opponents running the suit.
- The ace's seat matters: a guarded king without its ace won 19% with the ace on its left (playing after it) and
  41% with the ace in the kitty (suit bids).
- **The count now splits kings** (`crates/ai/src/thumb.rs`): suit bids ⅓ beside the ace, ⅕ with two or more
  guards, ⅛ with one; no trumps ½, but ⅓ with a single guard.

### 26. Passing over an opponent's bid: how often they make it depends on your hand

*Measured by `fh-lab opponents` over 6,000 deals of Oracle play: 5,328 times a player passed over an opponent's
bid that became the final bid.* "Your tricks" is the bidding network's estimate of what your side would take
declaring in the opponents' strain.

| Your tricks in their strain | Times | They made it |
|---|---|---|
| 1–2 | 203 | 89% |
| 2–3 | 1,312 | 86% |
| 3–4 | 2,247 | 81% |
| 4–5 | 1,243 | 69% |
| 5–6 | 272 | 41% |
| 6–7 | 46 | 20% |

Overall they made 77%, close to the old fixed estimate of 75%. Holding your hand fixed, each level higher roughly
halves the odds against them going down (logistic fit: `opponents_make` in `crates/ai/src/search_bot.rs`; at 8,
with 4 tricks of yours, 52%). Bids of 9 and up were too rare to measure.

The pass baseline now uses this fit in place of the fixed 75%. In a 1,500-deal duplicate match it gained **+2.9
points per hand** over the old value (95% CI −0.2 to +5.9; 144 deals won, 101 lost, the rest tied): a small, likely
gain. It declared less often (46% of hands against 54%) and made more of its bids (75% against 71%). The hint
for a pass over an opponent's bid now quotes this chance ("they make 8♣ about 52% of the time").

### 27. A 6-call in the suit you'd bid says much more

*Refit of the bidding count (`train/rules_of_thumb.py` data, 254,000 suit decisions), with 6-calls split by whether
they name the strain you're counting.* Everything else in the hand held equal:

| Call | In the suit you're counting | In another suit |
|---|---|---|
| An opponent called 6 | **−1.14** (±0.05) | −0.34 (±0.03) |
| Partner called 6 | **+1.42** (±0.07) | +0.50 (±0.04) |

The old count used one value for each (−½ and +¾). A 6-call shows the Joker *and* the caller's best suit, so when it's
your suit the length (and likely a bower) is against you or with you. The count now uses −1⅛ / −⅓ and +1.4 / +½
(`crates/ai/src/thumb.rs`); no trumps is unchanged (+1, −0.7). The user spotted this from a hand: an opponent's 6♦
before their own diamond bid counted only −½.

### 28. The bar for leaving partner's strain cost points

*Measured by `fh-lab switch` over 20,000 deals.* The Oracle wanted 40 points more than usual before bidding a
different strain over partner's bid, a rule from the first search bidder (finding 4), which sampled partner's hand
badly. The bidding network reads partner's bid itself. In the 1,642 auctions where that rule alone turned a bid into a
pass, the deal was played out both ways (solver, scored for real play):

| Cases | n | Bidding minus passing (95%) |
|---|---|---|
| All | 1,642 | **+41 ± 10** |
| Bid makes under 70% | 738 | +38 ± 16 |
| Bid makes 70–85% | 771 | +45 ± 13 |
| Bid makes over 85% | 133 | +37 ± 26 |

**Removed** for the neural bidder (the Oracle and the coach). The user's hand: partner bid 7♦, their ♦ count was
9¾, and the coach said pass with 8NT at 85% and 7NT at 93%; that was the rule at work.

## Open questions for the lab

- **Bid estimates:** they rest on few sampled worlds, so they are noisy. A fitted evaluator, trained on solver output, would be faster and steadier. This is planned for the Phase 5 neural work.
- **Signals:** what a mention or a 6-call is worth, compared with what it tells the opponents.
- **Seat position:** what a once-around auction implies for each seat. Partly answered for bid levels (finding 14).
- **The forced dealer:** strategy on the third deal.
- **Score-aware bidding:** when defenders can go out on tricks. Answered for bid levels (finding 20).
- **The playbook:** the 30 hypotheses in `docs/research/02-strategy-playbook.md` still need checking under these rules.
