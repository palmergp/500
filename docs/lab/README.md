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

*Superseded by finding 48: these were measured when defensive points could win the game.*

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

### 29. Calling 6 with the Joker: it hardly matters

*Measured by `duel` (duplicate, the Oracle in all seats).* The house signal is to call 6 whenever you hold the Joker
and don't bid. The user asked whether the first to call should still do it with a weak suit (does partner's
knowing where the Joker is outweigh the suggestion that the suit is good, and does it scare the opponents off,
bringing a redeal?).

| Variant | Deals that came out differently | Gain per hand for the house signal (95% CI) |
|---|---|---|
| First to call, with a count below 6: pass instead of 6 | 3 of 3,000 | +0.1 (−0.1 to +0.2) |
| Never call 6 | 48 of 2,000 | +0.1 (−0.9 to +1.1) |

Thrown-in deals stayed at 0.8 per 100 hands with every variant. The bidders already value a 6-call in another suit
modestly (+½ for partner, −⅓ for an opponent, finding 27), so a weak 6-call rarely changes anyone's bid, and the
opponents aren't scared off often enough to bring redeals. Call 6 as the house signal says; it costs nothing. (The
bidding network learned from always-6 play, so these variants mislead their partners slightly; with no difference
measurable even so, a retrained network wouldn't change the answer much.)

### 30. The low side-card lead after a winning trump: mostly a tie-break

*Measured by `fh-lab lowlead` over 6,000 deals of Oracle play.* The user asked why the bots so often lead a trump
winner and then a low side card, a near-certain loser. After a trump to trick 1 that declarer won (787 hands), the
second lead was another trump 48% of the time, a side ace or king 31%, a side queen or jack 7%, and a low side card
14%. The 285 low side leads at trick 2 or 3 (declarer having won every trick so far), by the suit's length:

| Length in declarer's hand | n | Declarer trumped the suit later | Partner won the trick | Solver, all cards visible: vs the best trump lead | It was the solver's best (or tied) |
|---|---|---|---|---|---|
| 1 (makes a void) | 30 | 50% | 33% | −0.13 tricks | 67% |
| 2 | 70 | 21% | 34% | −0.31 tricks | 69% |
| 3 or more | 185 | 5% | 40% | −0.10 tricks | 77% |

- **Rarely to make a void:** two-thirds come from suits of three or more, and declarer trumped those later only 5%
  of the time.
- **Mostly a tie:** in the deals the bot imagines, leading a low card and another trump come out about equal. Its
  tie-break among plays within half a point of the best is to give up the cheapest card, and a low side card is
  cheaper than a trump. It also gives up a loser early, while declarer still holds the top trumps, and partner won
  the trick 40% of the time.
- **Not free:** with every card visible it cost 0.1–0.3 tricks on average (most from doubletons), though it was the
  best or tied lead about three times in four.

For players: after a winning trump, keep drawing trumps or cash your side winners. Lead a low side card early only
from a singleton you mean to trump, or to reach partner.

### 31. A 6-call in your left bower's suit counts double

*Refit of the bidding count, as finding 27, with 6-calls in another suit split by colour.* In a suit bid:

| 6-call in… | By an opponent | By partner |
|---|---|---|
| Your suit | −1.14 | +1.42 |
| Your left bower's suit (same colour) | **−0.51** (±0.05) | **+0.72** (±0.07) |
| Another side suit | −0.25 (±0.04) | +0.39 (±0.05) |

A 6-call in the same-colour suit often comes with that suit's jack, your left bower. The count now uses −½ / +¾
there and −¼ / +0.4 for the other two suits.

### 32. The count's thresholds against the coach: stop at 8

*Measured by `fh-lab thumbcheck` over 30,000 deals of Oracle auctions.* Wherever a player's count in their clearly
best suit sat just past a threshold, the bid the rule suggests was graded by the coach, and played out both ways
(the rule's bid and the coach's choice; solver, scored for real play).

| The rule's bid | Graded best or good | Mistake or blunder |
|---|---|---|
| 7 at a count of 7–7½, no bid yet | 94% | 2% |
| 8 at 8½–9, no bid yet | 60% | 14% |
| 9 at 10–10½ | under 1% | about 97% |
| 9 at 10½–12 | about 2% | about 90% |

Played out, the coach's choice beat the rule's bid by +139 ± 39 points where the coach called it a blunder, +42 ±
24 for a mistake, +21 ± 20 for an inaccuracy and +11 ± 21 for "good": the coach's verdicts hold up. **The rule's
"9 at about 10" was wrong**; it now says to stop at 8. At 8½, 7 and 8 score about the same on average (finding
14), so the coach decides per hand; when you follow the count and it disagrees, the grade now says so and why.

### 33. What a mention tells partner: length counts as much as the bower

*Measured by `fh-lab mentions` over 2,000 deals (all four seats, all four suits).* For each holding a player might
mention, their partner declared in that suit with the kitty (solver); partner's tricks less partner's own count,
against holdings nobody would mention (no bower, three trumps or fewer):

| What the mentioner holds (as trumps) | n | Worth to partner |
|---|---|---|
| A bower, two trumps or fewer in all | 3,838 | +0.65 tricks |
| A bower and two more trumps | 4,487 | +1.35 |
| A bower and three or more | 4,977 | +2.29 |
| No bower, four trumps | 2,282 | +1.07 |
| No bower, five or more | 666 | +1.81 |

Four small trumps tell partner more than a bower with one other card. This is the information's worth to partner
only (the opponents hear it too; finding 19 put all signal calls together at about +2 points a hand). The bots
still mention on a bower, which is what their bidding network learned to read.

### 34. Leading winners first on near-ties: no difference

*Measured by `duel`, 2,000 deals each played both ways.* The user saw partner bid 8♥ and lead a low club, an almost
certain loser. Among plays within half a point of the best (in the deals the bot imagines), the search bots give up
the cheapest card, and a low side card is cheaper than a trump (finding 30). A variant broke those ties the other
way when leading for the declaring side: the dearest card (top trumps, then winners).

| | Gain per hand for the usual tie-break (95% CI) | Deals won / lost |
|---|---|---|
| Lead winners on near-ties instead | −0.8 (−4.5 to +2.9) | 449 / 422 |

It changed 871 deals of 2,000 and gained nothing either way. With every card visible a low side lead costs about
¼ trick when declarer holds the top trumps (finding 18), but at the table, without seeing the hands, giving up a
loser early (while declarer still holds trump control) does as well. The tie-break stays.

### 35. A mention or 6-call after partner has called: no difference, so pass

*Measured by `duel`, 2,000 deals each played both ways.* The user's point: once partner has called, partner can't
use a mention or 6-call in the auction; it only tells partner something for the defence, and the opponents hear it
too. The bots never read mentions in card play (finding 21), so for them it can only change the opponents' bids.

| | Gain per hand for signalling after partner's call (95% CI) | Deals that differed |
|---|---|---|
| Pass instead | +0.3 (−0.3 to +1.0) | 26 of 2,000 |

No measurable value. **The bots and the coach now pass instead** once partner has called (unless they bid), which
is the simpler rule to teach; the lab variant `oracle-latesignals` keeps the old behaviour.

### 36. The discard signal: worth nothing to the bots, and making it cost points

*Measured by `duel`, 2,000 deals each played both ways.* The house signal: your first throw-off is a low card of
your strongest side suit, so partner knows what to lead back. The user asked what it's worth.

| Comparison | Gain per hand for the first (95% CI) |
|---|---|
| The Oracle (signals at a cost of up to 25 points) vs no signals at all | **−3.4** (−6.5 to −0.4) |
| Signals at up to 5 points vs no signals | −1.6 (−4.1 to +0.8) |
| Signals at up to 5 points vs up to 25 | **+1.8** (+0.1 to +3.4) |

- Most of the cost was in making the signal: the bots gave up to 25 expected points to throw the right card.
  With 5 the loss halves.
- The information itself gains the bots nothing measurable (reading partner's signal alone was −1.1 ± 1.9,
  finding 21). Their search already narrows the hidden cards from what has been played, and a signal that's
  only made when cheap is also a less reliable one to read.
- **Change:** the bots signal only when it costs at most 5 points. They keep the convention, since it's the table's
  rule and a person partnering a bot reads it; with people, who can't count every card, the signal is probably
  worth more than it is to the bots. That isn't measured here.

### 37. Mentions of another strain: partner's costs ⅔ of a trick

*Refit of the whole count (`train/rules_of_thumb.py`, 254,000 suit and 63,000 no-trump decisions) with every call
split by the strain it names, against partner not having called yet.* The user's hand (code 8T7A-EBDG): partner
mentioned ♠, an opponent mentioned NT, and the count said 8.7 for diamonds while the coach gave 8♦ only 54%.

| Call, when counting a suit | Tricks (95%) | Old count |
|---|---|---|
| Partner mentioned another suit | **−0.68** (±0.03) | 0 |
| Partner mentioned NT | −0.25 (±0.05) | 0 |
| An opponent mentioned another suit | +0.26 (±0.02) | 0 |
| An opponent mentioned NT | **0.00** (±0.04) | 0 |

Counting no trumps: partner's mention of a suit −1.15, an opponent's +0.47.

- **Partner's mention of another suit is a minus:** partner's strength is elsewhere. Better than a pass (−1), worse
  than not knowing.
- **An opponent's NT mention doesn't matter on average** to a suit bid. On the user's hand the coach's network read
  it as a minus (8♦ fell from 68% to 54%), an interaction with that hand, not a rule.
- Small trumps are worth the same whether or not you hold a top trump (+0.49 and +0.47 each).
- With every call separated, the rest of the auction values shift a little (partner's pass −1, partner's mention of
  your suit +0.4, an opponent's of it −½; 6-calls +1⅓ / +0.6 / +¼ for partner and −1 / −0.4 / −⅛ for an opponent).
  The count, the guide, the bidding-count reference and the lessons use the new values. The user's hand now counts
  8.0, which says 7♦, as the coach did.

### 38. A king with one small card, the only suit you could empty: void it

*Measured by `fh-lab kxkitty` over 300,000 deals (513 where it applies).* South declares at the count's level, and
after the kitty the only side suit of one to three cards is a king with one small card (no ace). Two plans, solved
on the same deals: put back the king, its guard and the lowest other side card (void), or keep the guarded king and
put back the three lowest other side cards (never an ace).

| Trumps after the kitty | Deals | Tricks: void / keep | Bid made: void / keep | Void better / keep better |
|---|---|---|---|---|
| 5 or fewer | 83 | 7.76 / 7.28 | 64% / 56% | 51% / 10% |
| 6 | 173 | 8.64 / 8.14 | 88% / 77% | 54% / 14% |
| 7 or more | 257 | 8.57 / 8.30 | 85% / 78% | 40% / 16% |
| All | 513 | **8.46 / 8.08** | **83% / 74%** | 47% / 14% |

The void wins by about 0.4 tricks at every trump length, as finding 16 found for voids in general: a guarded king
in a suit bid often falls to the ace or gets trumped, while the void lets you trump that suit. The coach's kitty
plan already voids it (98% of these deals). In no trumps it's the other way round (finding 23): keep the king.

### 39. The third discard: beside an ace, or from a long weak suit? Barely matters

*Measured by `fh-lab lastdiscard` over 400,000 deals (4,726 where it applies).* South declares; after the kitty the
side suits are a low doubleton (put back: a void), an ace with one small card, and a suit of three or more without
the ace. The third discard: the small card beside the ace (leaving the ace alone, a second void once it's cashed),
or the lowest card of the long suit. Solved on the same deals:

| Long suit, trumps | Deals | Tricks: beside the ace / long suit | Bid made | Better: ace / long |
|---|---|---|---|---|
| 3 cards, 6 trumps | 3,152 | 7.97 / 8.04 | 73% / 74% | 14% / 19% |
| 4 cards, 5 trumps | 1,411 | 7.82 / 7.77 | 71% / 69% | 18% / 11% |
| 5+ cards, 4 trumps | 162 | 7.48 / 7.52 | 63% / 64% | 14% / 17% |
| All | 4,726 | 7.91 / 7.95 | 72% / 72% | 15% / 16% |

Within 0.1 trick either way, and the two plans tie on about 70% of deals. With a four-card weak suit, leaving the
ace alone is slightly better; with three, shortening the long suit is. The coach's plan puts back the card beside
the ace 86% of the time. The two discards that make the void are what count (finding 16).

### 40. The kitty plan, judged as a set: +0.19 tricks where it differs

The user's hand (code X5MF-5PK6): 7♦ with K♠ 5♠, K♣ J♣, Q♥ 6♥ 5♥ 4♥ and five trumps. Every three-card discard
scored on 120 likely deals:

| Put back | Tricks | Bid made |
|---|---|---|
| J♣ K♣ 5♠ (void ♣, K♠ alone) | 7.85 | 77% |
| Any club or spade void, plus a low heart | 7.76 | 73–74% |
| Three low hearts (Q♥ alone) | 7.46 | 62% |
| The coach's plan then: 5♠ J♣ 4♥ (two bare kings, no void) | 7.38 | 58% |

The plan chose one card at a time: 5♠ first, after which K♠ was alone and no longer "guarded", so cheap; J♣ the
same; then a low heart. It never saw that K♣ J♣ together make a void. The new plan (`set_discards`) scores whole
sets: each void is worth its measured value (0.33 / 0.43 / 0.52 tricks by trump length, finding 16) times 1.5, each
suit cut to one card 0.1, and each card put back what it's worth to keep (ace 1, king 0.15, queen 0.05, lower 0).
The weights were tuned by `fh-lab kittyplan` on 15,000 deals and checked on 40,000 fresh ones (solver, every card
visible):

| Plan | Tricks | Bid made |
|---|---|---|
| One card at a time | 7.80 | 69% |
| As a set | **8.00** | **73%** |

The plans differ on 25% of declared hands (+0.19 ± 0.03 tricks there, about +0.05 a declared hand overall). The
bots and the coach use it for suit bids; no trumps keeps its own plan (finding 23). For this hand it now says
5♠ K♠ J♣: void spades.

**The principle:** empty a short suit before you shorten a long one. Putting back three low hearts left a singleton
Q♥, a loser the first time hearts are led, and kept both kings as losers in their two-card suits; the void lets you
trump that suit from its first lead, and the king you put back was rarely a trick anyway (0.15).

### 41. Speaking last after a quiet auction: the Joker is probably in the kitty

The user asked: last to call, everyone passed or mentioned, so where's the Joker? Under the convention (call 6 with
the Joker unless partner has already called), the first two callers would have shown it; the third, whose partner
has called, wouldn't. So it's with the right-hand opponent or in the kitty. On card counts alone that's 10 to 3 for
the opponent, but a player with the Joker usually bids.

*Measured by `fh-lab dealerjoker` over 120,000 deals of Oracle auctions,* the dealer (South) without the Joker,
its best suit solved with South declaring (kitty taken):

| | n | Tricks less the network's estimate | Less the count |
|---|---|---|---|
| Any auction (every fifth deal, for reference) | 18,308 | +0.09 | −0.25 |
| **Nobody bid or called 6** | **1,899** | **+0.33** | **+1.03** |
| …the Joker in the kitty (81%) | 1,536 | +0.65 | +1.36 |
| …the Joker with the right-hand opponent (19%) | 363 | −1.02 | −0.39 |

Where the Joker is matters by two tricks (`fh-lab jokerplace`: with partner or in the kitty +0.8, with an opponent
−1.3, against the count). In a quiet auction it's in the kitty four times in five, and the count ran 1.3 tricks low
against its usual error. **The count now adds 1¼ there** (suit bids, without the Joker yourself). The bidding network
was only about ¼ trick low.

### 42. What the count misses, found by search

*Measured by `train/mine_count.py` (candidate features against the count's error, each added to the full fit) on
the old bidding data (444,000 suit decisions, rule bots bidding) and on fresh data (475,000, the Oracle bidding at
all four seats, today's conventions); kept only where both agree, then checked by `fh-lab countcheck` (the count's
error by situation on fresh Oracle auctions, real-play tricks from the solver).*

| Situation | Old data | Fresh data | Count's error before | After |
|---|---|---|---|---|
| Partner bid another strain | +0.60 / +0.22 (by seat) | +0.58 / +0.63 | +0.63 | −0.01 |
| Partner passed, and you hold the Joker | +0.27 | +0.23 | +0.29 | +0.09 |
| A bare side ace (no other card in its suit) | −0.24 | −0.22 | −0.26 | −0.07 |

The count now adds **+½ for partner's bid in another strain** (+1 for no trumps), makes partner's pass **−¾ when you
hold the Joker**, takes **¼ off a bare side ace**, and in no trumps takes ¼ off **the Joker with two or more aces**
(−0.31 / −0.26).

Not adopted:
- **Both opponents passed:** +0.6 in both fits, but −0.35 in the count's error on Oracle auctions (the dealer's
  bonuses already cover much of it).
- **The auction values in general:** refit on Oracle-only data they change a lot (partner's pass −0.4 instead of
  −1), because the Oracle bids far more readily (it almost never calls 6: it bids). They depend on how the other
  players bid, so the count keeps values from the mixed data. Likewise the dealer runs −0.3 against Oracle bidders.
- **Strong hands are overcounted:** from a count of 8 each extra point is worth only about 0.4 trick (error −0.4 at
  8–9, −1.0 at 9+, in both data sets). The thresholds were fitted on the count as it is, and the guide already says
  stop at 8, so the count isn't changed; the guide says why.

### 43. The bidding network, retrained under today's conventions

*Fresh data: `fh-lab dataset --bidder oracle`, 30,000 deals in six chunks (120,000 decisions), with the bots as they
now bid and signal (no 6-calls or mentions after partner's call, cheap signals, the set kitty plan).* Two networks
were trained (`train/train_bid.py`) and compared with the old one on the same 12,000 held-out fresh decisions:

| Network | Log loss | Error | Bias | 8+ tricks: stated / actual |
|---|---|---|---|---|
| The old one | 1.983 | 1.530 | −0.11 | 33.2% / 35.0% |
| Fresh data only | 1.971 | 1.515 | −0.08 | 33.4% / 35.0% |
| **Fresh and old data** | **1.969** | **1.514** | **−0.03** | **34.7% / 35.0%** |

In play (duplicate, 2,000 deals, `oracle-altnet` against the Oracle) the new one made no difference: −0.4 a hand
(−4.6 to +3.8). It's embedded anyway, since what it states (the coach's percentages) is more honest. The lab variant
`oracle-altnet` bids with any network named by `FH_ALT_BID_MODEL`.

### 44. How often a card-play hint is wrong

*Measured by `fh-lab hintcheck`: 600 hands of Counter play; one of South's plays each (a random trick with a choice)
gets the coach's hint and a 256-deal look from the same belief.*

| Decisions | n | Same card as the deep look | Points given up | Off by 5+ | Hint time (native) |
|---|---|---|---|---|---|
| All | 406 | 88% | 0.9 | 5% | 0.6 s |
| Following suit | 295 | 96% | 0.3 | 2% | 0.4 s |
| **Leading** | 111 | **69%** | **2.6** | 15% | 1.0 s |
| 2–4 legal cards | 250 | 96–99% | 0.1–0.4 | 1–2% | 0.3–0.5 s |
| 5+ legal cards | 156 | 73% | 2.1 | 12% | 0.9 s |
| Trick 1 | 39 | 87% | 1.7 | 8% | 3.1 s |
| Tricks 8–10 | 94 | 99% | 0.2 | 1% | under 0.01 s |

Hints are reliable when following suit and late in the hand; leads with many cards are where they differ from a
deeper look, by 2.6 expected points on average. Those already take the closer look (96 deals): forcing it changed
nothing. The rest is the gap between 96 and 256 deals, part of it the deep look's own noise; closing it would mean
two or three times the wait on the first trick.

### 45. Card-play rules, checked with the solver

*Measured by `fh-lab plays`: 2,000 hands of Counter play; at each decision fitting a rule, the real position solved
with every card visible, the rule's card against the best card it rules out (for second hand low, against the
cheapest card that would win).*

| Rule | Decisions | Tricks the rule gains | Its card best (or tied) |
|---|---|---|---|
| Defence: third hand high (cheapest winner, vs low) | 125 | **+0.46** | 99% |
| Defence: cover an honour with an honour (vs ducking) | 188 | **+0.43** | 99% |
| Declarer: draw trumps, opponents holding 1 | 976 | +0.09 | 92% |
| Declarer: draw trumps, opponents holding 2 | 982 | +0.05 | 89% |
| Declarer: draw trumps, opponents holding 3+ | 3,625 | +0.01 | 89% |
| Declarer: opponents out of trumps, cash a side winner | 963 | +0.05 | 99% |
| Defence: lead partner's signalled suit | 733 | −0.05 | 91% |
| Defence: return partner's first-led suit | 373 | −0.05 | 88% |
| Defence: second hand low (vs the cheapest winner), suit bids | 802 | −0.15 | 83% |
| Defence: lead a trump (vs the best side lead) | 1,562 | **−0.21** | 78% |

Seeing every card favours plays that don't depend on the opponents guessing, so second hand low (whose value is
the declarer's guess) is understated; the others are clear. The guide's rules 9, 11 and 13 now say so.

### 46. The guide as a bot: its thresholds are right; the gap is judgment

*Measured by `duel`, 1,000 deals each.* `guide` bids strictly by the guide (the count, its thresholds, the auction
values, 6-calls and mentions, stop at 8) and plays like The Counter.

| Match | Gain per hand (95% CI) | Declared: first / second | Made |
|---|---|---|---|
| The guide vs the Oracle (same play) | **−17.3** (−24.2 to −10.5) | 35% / 65% | 75% / 74% |
| Guide ½ trick bolder vs the guide | −7.6 (−14.1 to −1.0) | 61% / 39% | 65% / 74% |
| Guide 1 trick bolder vs the guide | −20.6 (−29.0 to −12.1) | 73% / 27% | 58% / 78% |
| Guide 1½ tricks bolder vs the guide | −57.7 (−67.9 to −47.4) | 83% / 17% | 47% / 76% |
| The guide vs without the auction values | −2.3 (−7.1 to +2.6) | | |
| The guide vs overcalling freely | −2.0 (−5.6 to +1.7) | | |
| The guide vs without 6-calls and mentions | +3.4 (−0.2 to +6.9) | | |

Bidding by the count costs about 17 points a hand against the network, which declares nearly twice as often at the
same success rate: it finds the hands worth bidding that the count undervalues. Simply bidding more boldly makes
things worse, so the thresholds are right for the count; the network's edge is in reading the whole hand. Of the
guide's other rules only the signals showed a likely gain; the auction values and the overcall rule were within
noise of nothing (they may matter more against players who read them than against bots).

### 47. The coach's percentages, recalibrated

*Measured by `fh-lab honesty` (2,000 hands of Oracle play each).* After the set kitty plan (finding 40), cheap
signals (36), passing after partner's call (35) and the retrained bidding network (43), the coach had drifted to
optimistic: bids stated 75% on average and 68% made, card play 77% and 72% (seed 4,200,000; bids stated 60–70% made
53%). Both real-play curves (`BID_CURVE`, `PLAY_CURVE` in `crates/ai/src/coach.rs`) were refit by mapping each old
point through the measured stated-to-actual table, and checked on fresh deals (seed 4,300,000):

| | Stated | Made |
|---|---|---|
| Bids, before | 75% | 68% |
| Bids, after (fresh deals) | **69%** | **70%** |
| Card play, before | 77% | 72% |
| Card play, after (fresh deals) | **72%** | **73%** |

Every band with a few hundred predictions now matches within about 5 points. The coach's choices don't change:
it ranks options by expected points; the curves only translate its chances for display.

### 48. Bidding to the score, when only a made bid wins

*Measured by `fh-lab levels --deals 6000 --us N --them N` (seed 300,000) after the winning rules changed (H23–H24,
2026-11-09): only declarers who make their bid win at 500, defensive points never win, and −500 loses.* As finding
20, each option valued by the bidder's side's chance of winning the game. All positions:

| Count | 300 vs 460: pass / 7 / 8 | 460 vs 300: pass / 7 / 8 | 450 vs 450: pass / 7 / 8 |
|---|---|---|---|
| 6½ | 35 / **43** / 42 | 54 / **59** / 54 | 44 / **52** / 45 |
| 7 | 38 / **49** / 48 | 61 / **66** / 59 | 49 / **60** / 51 |
| 7½ | 43 / 56 / **59** | 70 / **74** / 68 | 57 / **69** / 61 |
| 8 | 43 / 61 / **67** | 73 / **80** / 74 | 60 / **76** / 69 |
| 9 | 46 / 68 / **82** | 79 / **88** / 86 | 66 / **85** / 83 |

After an opponent's bid (7 is no longer available), at counts 6½ / 7 / 7½:

| | pass | bid 8 |
|---|---|---|
| 300 vs 460 | 24 / 32 / 36 | **31 / 44 / 57** |
| 460 vs 300 | 41 / 53 / 65 | **44 / 56 / 68** |
| 450 vs 450 | 31 / 43 / 48 | **35 / 48 / 61** |

- **Behind, with the opponents near 500:** still bid even light hands (passing is worth a little more than before,
  35% against 32% at 6½, since they can no longer go out on defense). With a light hand 7 is as good as 8; from a
  count of 7½ the level that reaches 500 if made (8 here) is clearly best.
- **Ahead, near 500:** 7 is enough. Over an opponent's bid, **bid 8 rather than pass**, the reverse of finding 20:
  defensive tricks no longer win, so letting them play gives them the chance to go out on their bid.
- **Both near 500:** bid 7 even light, as before.
- 9 and 10 were never best at these scores. Only 267 decisions came after an opponent's bid, so that table is the
  rougher one.

## Open questions for the lab

- **Bid estimates:** they rest on few sampled worlds, so they are noisy. A fitted evaluator, trained on solver output, would be faster and steadier. This is planned for the Phase 5 neural work.
- **Signals:** what a mention or a 6-call is worth, compared with what it tells the opponents.
- **Seat position:** what a once-around auction implies for each seat. Partly answered for bid levels (finding 14).
- **The forced dealer:** strategy on the third deal.
- **Score-aware bidding:** answered for bid levels (finding 48, which replaces 20 under the made-bid rule).
- **The playbook:** the 30 hypotheses in `docs/research/02-strategy-playbook.md` still need checking under these rules.
