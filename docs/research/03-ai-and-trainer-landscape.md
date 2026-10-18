# 500 (Five Hundred): AI Techniques and Trainer Landscape

Research report 03. Compiled 17 October 2026.

Scope: Part 1 covers AI for imperfect-information trick-taking games (search, inference, learning, bidding, double-dummy solving, difficulty and personality, bot evaluation, and how engines explain decisions). Part 2 covers existing 500 apps and websites and the trainers in bridge, chess, poker and backgammon worth borrowing from. Section 5 gives recommendations for a TypeScript web app with AI in Web Workers and a Node-based simulation lab, and notes what changes if Python is used instead.

Reading notes:

- "500" always means the trick-taking game, not Rummy 500.
- Figures quoted from sources are cited inline. Statements marked **(estimate)** or **(recommendation)** are this report's own engineering judgement, not findings from the sources.
- Google Play pages are rendered with JavaScript and could not be fetched directly. Google Play review snippets below come from search-engine extracts and third-party trackers. Searches restricted to Reddit returned no relevant threads about 500 apps, so community evidence comes mainly from app-store reviews.

## Contents

1. [Executive summary](#1-executive-summary)
2. [What kind of AI problem is 500?](#2-what-kind-of-ai-problem-is-500)
3. [Part 1: AI for imperfect-information trick-taking games](#3-part-1-ai-for-imperfect-information-trick-taking-games)
4. [Part 2: Product landscape](#4-part-2-product-landscape)
5. [Recommendations](#5-recommendations)
6. [Sources](#6-sources)

---

## 1. Executive summary

**What the research says**

1. **The proven recipe is sampling plus exact search plus inference.** Perfect Information Monte Carlo (PIMC) samples complete deals consistent with what a player knows, solves each one with a fast perfect-information ("double-dummy") solver, and picks the action that does best on average. It produced GIB in bridge ([Ginsberg 2001](https://jair.org/index.php/jair/article/view/10279)), Kermit in Skat ([Buro et al. 2009](https://cs.du.edu/~sturtevant/papers/skat.pdf)) and a Schnapsen agent that beat human experts ([Wisser 2015](https://www.ijcai.org/Proceedings/15/Papers/025.pdf)). Skat is the closest well-studied relative of 500: 10 cards each, a small talon the declarer picks up and discards to, an auction, jacks as top trumps, and a "null" contract much like misère.
2. **PIMC's theoretical flaws are real but small in trick-taking games.** The flaws are strategy fusion and non-locality. In Skat, PIMC's endgame losses against a CFR solution came to about 0.063 tournament points per deal, against a standard deviation of 778 over a 180-deal series ([Long et al. 2010](https://ojs.aaai.org/index.php/AAAI/article/view/7562)).
3. **The biggest gains come from inference and from the objective, not from fancier search.** Adding inference lifted Kermit from 779 to 996 points per 36-game series ([Buro et al. 2009](https://cs.du.edu/~sturtevant/papers/skat.pdf)). Learned inference added several more tournament points per game ([Solinas et al. 2019](https://arxiv.org/abs/1903.09604); [Rebstock et al. 2019](https://arxiv.org/abs/1905.10911)). By contrast, the αμ algorithm improved bridge declarer play by at most about 3 percentage points of contracts made, at roughly 4–13 times PIMC's cost ([Cazenave & Ventos](https://arxiv.org/abs/1911.07960)). Choosing actions by expected *score* rather than expected *tricks* fixes missed safety plays ([Bethe 2021](https://cs.nyu.edu/media/publications/bethe_pm_thesis_final.pdf)).
4. **ISMCTS is a sound commercial alternative but not clearly stronger here.** It runs 2,500 iterations in under 0.25 s on a 2011 phone in AI Factory Spades ([Whitehouse et al. 2013](https://ojs.aaai.org/index.php/AIIDE/article/view/12679)). It is poor at bidding. In Skat and Schnapsen, UCT/MCTS did not beat PIMC ([Buro et al. 2009](https://cs.du.edu/~sturtevant/papers/skat.pdf); [Wisser 2015](https://www.ijcai.org/Proceedings/15/Papers/025.pdf)). In Doppelkopf, an ensemble of searches on fixed sampled deals clearly beat drawing a new deal on every rollout ([Sievers & Helmert 2015](https://ai.dmi.unibas.ch/papers/sievers-helmert-ki2015.pdf)).
5. **Neural networks are best used as components** (bid evaluation, card-location inference, human-like priors) trained on our own simulation data. ReBeL and Student of Games needed data-center compute ([Brown et al. 2020](https://arxiv.org/abs/2007.13544); [Schmid et al. 2023](https://arxiv.org/abs/2112.03178)). Their guarantees also assume two-player zero-sum games, which a four-player partnership game without table talk is not ([Celli & Gatti 2018](https://arxiv.org/abs/1711.06930)).
6. **Bidding works best as a hybrid:** a fast learned hand evaluator, simulation for close decisions, a tunable risk threshold (Kermit bids only if estimated win probability is at least B = 0.6), and awareness of the game score. Double-dummy (DD) bid evaluation is prone to optimism, as one Bridge Winners reviewer found ([IntoBridge review](https://bridgewinners.com/article/view/intobridge-review/)). DD evaluation of null/misère is pessimistic, because DD defenders always find the weakness ([Buro et al. 2009](https://cs.du.edu/~sturtevant/papers/skat.pdf)).
7. **The 500 market has one strong incumbent and many thin products.** NeuralPlay's *Five Hundred (500) – Expert AI* is rated 4.6 on the US and Australian App Stores ([US](https://apps.apple.com/us/app/five-hundred-500-expert-ai/id1559298243), [AU](https://apps.apple.com/au/app/five-hundred-500-expert-ai/id1559298243)) and has 100K+ Android installs ([APKCombo](https://apkcombo.com/five-hundred-500-expert-ai/com.neuralplay.android.fivehundred/)). It offers six AI levels, deep rule options, hints and a play checker.
   - **Recurring complaints across 500 apps:** overbidding partners; partners who waste the Joker or a trump on their partner's winner; mishandled misère/nullo; bidding that ignores the game score; perceived rigged deals; missing variants.
   - **The gap:** no 500 product explains *why* a play is right, grades how bad a mistake was, separates luck from skill, or offers structured drills.
8. **Trainers worth borrowing from:**
   - chess.com and lichess: loss-based move classes, "Retry" / "Learn from your mistakes", puzzles mined from real games with Glicko-2 ratings.
   - Backgammon programs XG and GNU Backgammon: an error-rate metric over non-obvious decisions, plus luck analysis.
   - GTO Wizard (poker): EV loss per decision and an aggregate score.
   - Funbridge and BBO: duplicate comparison with the field. Funbridge's analysis also has a double-dummy hindsight problem to avoid.
   - Bridge Master: graded problems where the defense punishes your mistake.
   - Lead Solver: simulation statistics for each opening lead.

**Top recommendations** (details in Section 5)

- **Engine:** PIMC over inference-weighted samples, with a purpose-built 500 solver (trump and no-trump play with Joker rules, plus three-hand misère). Score actions by expected score or by probability of winning the game. Break ties between near-equal plays toward plausible, information-hiding moves. Write it all in TypeScript and run it in Web Workers, using `worker_threads` for the Node lab. Port only the solver to Rust or C++ compiled to WebAssembly, and only if benchmarks demand it.
- **Bidding:** a learned "hand + kitty" trick-distribution evaluator trained on lab data, plus Monte Carlo simulation of the rest of the auction and the play for close calls. Risk threshold and score-state utility double as personality knobs.
- **Difficulty:** rule-based bots for the bottom levels; search bots with tiered budgets, inference and noise for the upper levels. Personalities are orthogonal parameter sets, calibrated on an Elo ladder. Partners should always be at least competent.
- **Evaluation:** duplicate deals with swapped seats, paired statistics, SPRT for engine changes, a rating ladder for levels, and an in-game "flag this play" report.
- **Coach:** expected value judged on what the player could actually know (not on all four hands) for every decision, graded loss classes, contrastive explanations drawn from the sampled deals, a split of luck and skill, bidding EV tables, mined puzzles, and "what the expert bot did on this deal".

---

## 2. What kind of AI problem is 500?

| Property | 500 (4-player, Australian default) | Skat | Contract bridge | Spades |
|---|---|---|---|---|
| Players | 4 in fixed partnerships | 3: soloist vs a temporary pair | 4 in partnerships | 4 in partnerships |
| Deck and hands | 43 cards (red 4–A, black 5–A, Joker); 10 each plus a 3-card kitty | 32 cards; 10 each plus a 2-card skat | 52; 13 each | 52; 13 each |
| Tricks | 10 | 10 | 13 | 13 |
| Declarer exchange | Takes the kitty, discards 3 | Picks up the skat, discards 2 | None | None |
| Exposed hand | None (open misère: after trick 1) | None (except null ouvert) | Dummy | None |
| Top trumps | Joker, right bower, left bower | The four jacks | Ace | Ace |
| Auction | 6♠ = 40 up to 10NT = 520, misère 250, open misère 500 | Bid on game value | Conventional auction | Numeric bids, nil |
| "Lose every trick" contract | Misère / open misère, partner sits out | Null / null ouvert | None | Nil |

Sources: [Pagat: Five Hundred](https://www.pagat.com/euchre/500.html), [Buro et al. 2009](https://cs.du.edu/~sturtevant/papers/skat.pdf), [Bethe 2021](https://cs.nyu.edu/media/publications/bethe_pm_thesis_final.pdf), [Whitehouse et al. 2013](https://ojs.aaai.org/index.php/AIIDE/article/view/12679).

**Implications for the AI design**

1. **There is no dummy, so hidden information is large.**
   - At trick one, each player sees only their own 10 cards; the declarer also knows their 3 discards.
   - **(Estimate)** A defender faces 33!/(10!·10!·10!·3!) ≈ 3.0×10^16 possible layouts of the unseen cards (three hands of 10 plus 3 discards). The declarer faces 30!/(10!)^3 ≈ 5.6×10^12.
   - Enumeration is impossible, so sampling is required.
   - Skat has the same "no dummy, 10 cards, talon" structure, and PIMC works well there.
2. **Information is revealed quickly.** Every trick exposes 4 cards and possibly voids.
   - In Skat and Hearts, Long et al. measured very high leaf correlation (0.8 to almost 1.0) and a disambiguation factor near 0.6 ([Long et al. 2010](https://ojs.aaai.org/index.php/AAAI/article/view/7562)). Leaf correlation means late choices rarely change the result; disambiguation measures how fast uncertainty shrinks. That combination is where PIMC loses little to an optimal player.
   - **(Recommendation)** Measure the same three properties for 500 in the lab. This confirms PIMC suits the game.
3. **It is a team game without communication.**
   - Finding a team-maxmin equilibrium with a correlation device, when teammates cannot communicate during play, is FNP-hard ([Celli & Gatti 2018](https://arxiv.org/abs/1711.06930)).
   - ReBeL's guarantees cover only two-player zero-sum games ([Brown et al. 2020](https://arxiv.org/abs/2007.13544)).
   - This is why practical engines for partnership card games use sampling, search and inference rather than equilibrium solvers.
4. **Suit membership depends on the contract.**
   - The left bower changes suit. With a red suit as trumps, trumps number 13 (11 suit cards, the left bower and the Joker) and the other red suit drops to 10. With a black suit as trumps, trumps number 12.
   - In no-trumps the Joker is the top card. Under Australian defaults, the contractor may nominate it as the top card of a suit before play. Otherwise it belongs to no suit and may be played to another player's lead only when void in the suit led. When it is led, its holder names a suit that has not yet been led ([Pagat](https://www.pagat.com/euchre/500.html)).
   - House rules vary widely, so the rules engine and solver must be driven by a variant configuration.
5. **The score state changes what is correct.**
   - You win only by *making a contract* that takes you to 500. Falling to −500 loses. Defenders score 10 per trick. A slam on a contract worth less than 250 scores 250 ([Pagat](https://www.pagat.com/euchre/500.html)).
   - Strategy writer Bryce Francis stresses that "the difference between the scores is more important than what those scores are." He also recommends lockout calls and going down in a small call rather than letting opponents make a big one ([play500.com](https://play500.com/calling1.html)).
   - So utility must be non-linear in tricks and aware of the score.

---

## 3. Part 1: AI for imperfect-information trick-taking games

### 3.1 PIMC (determinization) with double-dummy solvers

**How it works.** Sample "worlds" (complete deals) consistent with what is known. Solve each with perfect-information alpha-beta search. Play the move that wins most often or on average ([Buro et al. 2009](https://cs.du.edu/~sturtevant/papers/skat.pdf)). Examples:

- **GIB (bridge):** at the 1998 event in France it sampled 500 deals and took about 10 minutes per deal. Its deals were biased to fit the bidding and play, and weighted toward likely opponent errors ([Ginsberg, Dr. Dobb's 2000](https://jacobfilipp.com/DrDobbs/articles/DDJ/2000/0004/0004c/0004c.htm)). It was considered the strongest bridge program of its day ([Ginsberg 2001](https://arxiv.org/abs/1106.0669)).
- **500 products:** NeuralPlay's developer says its level 6 uses "Monte Carlo simulation with some randomness" ([App Store reply](https://apps.apple.com/us/app/five-hundred-500-expert-ai/id1559298243)). playboardgames.org's Hard bot runs PIMC over deals consistent with the cards and voids it has seen ([playboardgames.org](https://playboardgames.org/play/500)).

**Known flaws**

- **Strategy fusion.** PIMC assumes it can pick a different strategy in each sampled world, when in reality it must act the same way across indistinguishable situations.
- **Non-locality.** A node's true value can depend on parts of the tree outside its subtree, because opponents steer play using private information ([Long et al. 2010](https://ojs.aaai.org/index.php/AAAI/article/view/7562), after Frank & Basin 1998).
- **No information-gathering plays.** Each solved world assumes everything will be known later, so "the approach never suggests making an information-gathering play" and tends to "defer important decisions" ([Ginsberg 2000](https://jacobfilipp.com/DrDobbs/articles/DDJ/2000/0004/0004c/0004c.htm)).
- **Opponents treated as clairvoyant.** In Skat null games, DD analysis "almost always" predicts the soloist loses, because the defenders see the weakness ([Buro et al. 2009](https://cs.du.edu/~sturtevant/papers/skat.pdf)). Wisser shows small games where both PIMC and vector minimax pick the *worst* strategy because they assume a perfectly informed opponent ([Wisser 2015](https://www.ijcai.org/Proceedings/15/Papers/025.pdf)). This matters a great deal for 500 misère.
- **Equal is not equivalent.** Moves that are equal double-dummy can reveal different information.
  - In bridge, PIMC breaks such ties randomly about 4 times per deal for each side, and the choice is worth up to ±0.15 tricks per deal.
  - Leading a singleton king of trumps "equal" to other leads is one documented error.
  - All of Wbridge5, Jack, Bridge Baron and GIB showed such random tie-break mistakes ([Bethe 2021](https://cs.nyu.edu/media/publications/bethe_pm_thesis_final.pdf)).
- **Wrong objective.** Maximizing average tricks misses safety plays when making the contract is what matters ([Bethe 2021](https://cs.nyu.edu/media/publications/bethe_pm_thesis_final.pdf)). In 500, where failing a contract costs its full value, this is acute.
- **Biased samples.** AI Factory's Spades dealer filled high bidders' hands with high cards first, which skewed what remained. In one reported nil-bid hand the inferred probability that South held the 4♦ was one third, yet 87% of the generated deals gave it to South, and the AI partner misplayed as a result ([Whitehouse et al. 2013](https://ojs.aaai.org/index.php/AIIDE/article/view/12679)).

**Why PIMC still works.** Using synthetic trees, Long et al. found PIMC loses little when leaf correlation and disambiguation are high, which they are in Skat and Hearts. In Skat about 15% of games are still undecided with three tricks left. On those, PIMC lost 0.42 tournament points per deal against a CFR solution, which works out to 0.063 per deal overall. Over a 180-deal series that is 11.3 points, against a standard deviation of 778 ([Long et al. 2010](https://ojs.aaai.org/index.php/AAAI/article/view/7562)). Wisser calls PIMC "simple, fast, robust and scalable" ([Wisser 2015](https://www.ijcai.org/Proceedings/15/Papers/025.pdf)).

**How strong programs mitigate the flaws**

| Mitigation | Idea | Evidence | Cost | Value for 500 |
|---|---|---|---|---|
| Inference-weighted sampling | Weight or filter worlds by how likely the bids and plays were under each world | Kermit 996 vs 779 points per 36 games with vs without inference; learned inference worth several more tournament points per game ([Kermit](https://cs.du.edu/~sturtevant/papers/skat.pdf), [Solinas](https://arxiv.org/abs/1903.09604)) | Low to moderate | Very high |
| Utility-aware objective | Maximize expected score or make probability, or tricks minus one standard deviation | Safer plays found on about 0.39 of deals with better IMP results ([Bethe](https://cs.nyu.edu/media/publications/bethe_pm_thesis_final.pdf)) | Negligible | Very high |
| Information-aware tie-breaks | Among equal plays, prefer the one that gives opponents less to work with | DD-vs-result gap for the robot's defense narrowed from −0.14 to −0.09 tricks per deal ([Bethe](https://cs.nyu.edu/media/publications/bethe_pm_thesis_final.pdf)) | Small to moderate | High, also helps play look sensible |
| Knowledge "plausibility" skew | Heuristics bias early search toward sensible moves, decaying as search runs | Far fewer human-perceived blunders, no loss of strength ([Whitehouse et al.](https://ojs.aaai.org/index.php/AIIDE/article/view/12679)) | Small | High for partner experience |
| Presumed Payoff / Presumed Value PIMC (PPP/PVP) | Model opponents somewhere between random and clairvoyant; value outcomes by probability of winning the match from the resulting score | PVP beat plain PIMC 53.2% over 30,000 matches and humans 58.9%, at about +30% compute ([Wisser](https://www.ijcai.org/Proceedings/15/Papers/025.pdf)) | Low | High (misère, defense, score state) |
| Vector minimax / payoff-reduction minimax | Force one strategy across worlds | Weaker than plain PIMC in Kinderschnapsen, at about +600% compute ([Wisser](https://www.ijcai.org/Proceedings/15/Papers/025.pdf)) | High | Low |
| αμ | Search the declarer's moves jointly across worlds, M moves deep | 3NT make rate 60.2% to 63.0% (M=2, 20 worlds); 0.096 s/move for PIMC vs 0.363 s (M=2) and 1.228 s (M=3) with optimizations ([Cazenave & Ventos](https://arxiv.org/abs/1911.07960)) | High | Medium, endgames only |
| Extended PIMC (EPIMC) | Postpone the perfect-information leaf evaluation to reduce fusion | Gains reported in fusion-heavy games ([Arjonilla et al. 2024](https://arxiv.org/abs/2408.02380)) | Moderate | Experimental |
| Recursive PIMC (IIMC) | PIMC whose playouts are themselves imperfect-information players | State of the art in Skat ([Solinas et al.](https://arxiv.org/abs/1903.09604); [Furtak thesis](https://era.library.ualberta.ca/items/df9f1055-df09-43ed-910b-38da4e9d0792)) but "takes much more time to complete than PIMC" ([Cazenave et al. 2021](https://arxiv.org/abs/2101.12639)) | High | Lab-only |
| GIB lattice / achievable sets | Search over sets of layouts in which declarer makes | About +0.1 IMP per deal, declarer play only ([Long et al.](https://ojs.aaai.org/index.php/AAAI/article/view/7562)) | High | Low |
| More samples, ensembles, seed choice | Reduce Monte Carlo noise | AI Factory: 1,200 → 5,000 iterations gave +7–8% win rate. Better seed selection boosted Wbridge5 (Ventos et al. 2017, cited in [Bethe](https://cs.nyu.edu/media/publications/bethe_pm_thesis_final.pdf)) | Linear in samples | Use the time budget |
| Learned card play instead of DD | Neural rollouts trained on human play | GIBBO's neural rollouts "avoid unrealistic assumptions (like perfect information and perfect play)" ([GIBBO interview](https://news.bridgebase.com/?p=87069)). BBO's Ben plays from a network trained on human play and, since v1.2, uses DD only when the hidden hands are well known ([About Ben](https://news.bridgebase.com/about-ben-on-bbo/)) | Needs data | Later phase |

### 3.2 Information Set MCTS (ISMCTS) and MCTS variants

**The method.** ISMCTS searches a tree of information sets instead of game states. Each iteration samples a fresh world and descends only the moves legal in it, so one tree collects statistics across many worlds. This avoids per-world strategy fusion at the decision being made ([Cowling, Powley & Whitehouse 2012](https://eprints.whiterose.ac.uk/75048/)). Whitehouse's thesis reports ISMCTS outperforming PIMC search across several board and card games ([Whitehouse 2014](https://etheses.whiterose.ac.uk/id/eprint/8117/)). ISMCTS also beat AI Factory's knowledge-based AI in Dou Di Zhu ([Whitehouse et al. 2013](https://ojs.aaai.org/index.php/AIIDE/article/view/12679)).

**Commercial case study: AI Factory Spades** ([Whitehouse et al. 2013](https://ojs.aaai.org/index.php/AIIDE/article/view/12679))

- **Scale and budget.** The app had over 2.5M downloads and a 4.5 average from more than 78,000 Google Play reviews. The 4- and 5-star levels used ISMCTS with 1,000 and 2,500 iterations. 2,500 iterations took under 0.25 s on a Samsung Galaxy S II, using about 140 KB of tree memory. Players complain if a move takes over about 250 ms.
- **Card play only.** ISMCTS "is poor at making bidding decisions", so bidding stayed heuristic. Playouts stop at the end of the current round and return the normalized score difference, with bag penalties counted immediately.
- **Strength vs. perception.** ISMCTS matched or beat the hand-tuned AI, but beta testers reported "oddball" plays:
  - About half the reports were partners failing to protect a human's nil bid.
  - The other half were bots wasting high cards.
  - Most reports concerned the partner, because partner errors cost the human the game.
- **The fixes.**
  - Heuristic "skew" values in [0.5, 1.5] multiplied the rewards of each move, decaying toward 1 as the search proceeded. This nudged early search toward sensible moves without overruling it.
  - Tuning took days, not the weeks or months a purely heuristic AI needs.
- **Players choose the extremes.** Few players pick intermediate levels: casual players choose 1-star bots, frequent players almost always 5-star.

**Other card games**

| Game | Finding |
|---|---|
| Hearts | UCT proved stronger than the previous best Hearts programs (Sturtevant 2008, as summarized in [Buro et al. 2009](https://cs.du.edu/~sturtevant/papers/skat.pdf)). The strong "xinxin" bot combines PIMC with UCT evaluations ([Rebstock et al. 2024](https://arxiv.org/abs/2404.13150)). |
| Skat | UCT "was unable to defeat PIMC search" ([Buro et al. 2009](https://cs.du.edu/~sturtevant/papers/skat.pdf)). |
| Schnapsen (simplified "Kinderschnapsen" variant) | The best MCTS agent (1,000 iterations) won only 36.9% of matches against PIMC ([Wisser 2015](https://www.ijcai.org/Proceedings/15/Papers/025.pdf)). |
| Doppelkopf | **Ensemble UCT** (10 sampled deals × 1,000 rollouts, separate trees) scored +4.52 points per game against −1.25 for **single UCT** (a new deal every rollout). UCT's hand evaluation for solo and announcement decisions "tends to be overly optimistic". 1,000 test games with 10,000 rollouts per decision took about 19.5 hours ([Sievers & Helmert 2015](https://ai.dmi.unibas.ch/papers/sievers-helmert-ki2015.pdf)). |
| Jass | Comparing MCTS variants with deep networks, Niklaus et al. found the networks beat rule-based methods for trump selection and card play ([Niklaus et al. 2020](https://zenodo.org/records/5769551)). Jass AIs "do not outperform top players yet" ([survey](https://arxiv.org/abs/1906.04439)), though the best compete with good amateurs ([University of Bern](https://www.digitale-nachhaltigkeit.unibe.ch/studies/bachelor_s__and_master_s_theses_at_inf/game_ais/can_the_best_jass_ais_beat_the_top_humans/index_eng.html)). |

**Takeaway for 500 (recommendation).** Make PIMC with an exact 500 solver the backbone of card play. Keep ISMCTS with heuristic playouts as a lab baseline, and as a fallback where a solver is too slow, such as early tricks on weak phones. Do not use either for bidding without a dedicated hand evaluator.

### 3.3 Inference: using the bidding and play history

| System | How it infers hidden cards | Result |
|---|---|---|
| GIB (bridge) | Builds constraints from the auction, generates matching hands, then filters with the bidding module (about 1–2 s). Weights play by likely opponent mistakes ([Ginsberg 2000](https://jacobfilipp.com/DrDobbs/articles/DDJ/2000/0004/0004c/0004c.htm)). | Expert-level card play |
| Kermit (Skat) | Learns P(world given bids) from offline data, using **features** such as suit lengths and high cards rather than whole worlds. It treats features as independent and multiplies per-feature terms looked up from the data, adjusted for how many worlds share each feature. The soloist conditions on opponents' bids; defenders condition on the announced contract and consider all of the soloist's possible discards ([Buro et al. 2009](https://cs.du.edu/~sturtevant/papers/skat.pdf)). | +217 points per 36-game series; inference by defenders mattered most |
| Neural card-location inference (Skat) | A network trained on about 20M human games predicts each card's location; sample probabilities multiply per card ([Solinas et al. 2019](https://arxiv.org/abs/1903.09604)). | +3.4 to +4.8 tournament points per game in suit and null games (grand not significant); 0.286 s vs 0.093 s per move at 320 samples |
| Policy-based inference (Skat) | Weights each sampled state by the product of action probabilities under learned human-like policies ([Rebstock et al. 2019](https://arxiv.org/abs/1905.10911)). | Beat card-location inference by 2.32/0.64/1.57 tournament points per game (suit/grand/null), mostly on defense |
| AI Factory Spades | Hand-written probability table per card, updated by rules; hard zeros for voids ([Whitehouse et al.](https://ojs.aaai.org/index.php/AIIDE/article/view/12679)). | Worked, but biased dealing caused visible errors |
| Ben (bridge) | An "info" model is used when sampling the hidden hands, in both bidding and card play; samples are drawn consistent with the auction ([Ben repo](https://github.com/lorserker/ben); [author's comments](https://bridgewinners.com/bwcomments/user/lorand-dali/?page=2)). | Basis of BBO's robot |

**Lessons for 500**

- **Hard constraints come first.** These are voids, cards played, the declarer's own discards, a nominated Joker, and the exposed hand in open misère.
- **Make soft constraints probabilistic.** Kermit notes that deterministic bots give P(move given world) of exactly 0 or 1, which "makes the prediction brittle" when the other player doesn't play like you ([Buro et al. 2009](https://cs.du.edu/~sturtevant/papers/skat.pdf)).
- **Better inference can still play worse.** A variant that put *all* probability on the true state lost to plain card-location inference in Skat suit and grand games (−3.25 and −8.49 points per game), though it won in null games ([Rebstock et al. 2019](https://arxiv.org/abs/1905.10911)). Validate inference by playing strength, not only by how accurately it predicts.
- **Negative inference is still unexploited.** If an opponent would have defeated the contract on some layouts but didn't, those layouts become less likely. Bethe notes no bridge program used this ([Bethe 2021](https://cs.nyu.edu/media/publications/bethe_pm_thesis_final.pdf)).
- **Uniform sampling under constraints needs care.** Counting consistent card assignments is #P-hard in general, and Sievers & Helmert used an approximate dealer for that reason ([Sievers & Helmert](https://ai.dmi.unibas.ch/papers/sievers-helmert-ki2015.pdf)). **(Recommendation)** 500's hard constraints are mostly per-seat voids, known cards and hand sizes, so exact uniform sampling is cheap. Count the assignments suit by suit with a small dynamic program over the remaining hand sizes. Then apply soft weights by importance weighting or resampling.

### 3.4 Neural and game-theoretic approaches: needs and practicality

| System | Game | What is learned | Data / compute | Result |
|---|---|---|---|---|
| Ben (open source, GPL-3) | Bridge | Recurrent bidding network; networks for opening leads and for suggesting or evaluating plays; an "info" model for sampling. Per its author, card play is mostly decided by Bo Haglund's double-dummy solver (DDS) ([repo](https://github.com/lorserker/ben); [author](https://bridgewinners.com/bwcomments/user/lorand-dali/?page=2)) | Bidding network trained on about 0.5M deals; "a million lessons take only a few hours" ([BridgeWinners](https://bridgewinners.com/article/view/bridge-ai-how-neural-networks-learn-to-bid/)); Python and TensorFlow | Basis of BBO's "Ben"; README lists limits such as being unable to interpret bid explanations or defensive signals |
| Ben on BBO | Bridge | Card play trained on human play; DD only when hands are well known (v1.2) | 100M human ACBL hands ([About Ben](https://news.bridgebase.com/about-ben-on-bbo/)) | Deployed; bid explanations "borrowed" from GIB |
| NooK (NukkAI) | Bridge, declarer play in 3NT only | Not public | Not public | Beat 8 world champions over 800 deals in a restricted format ([IndiaAI](https://indiaai.gov.in/article/hybrid-ai-beats-eight-world-champions-at-bridge)). Critics suspect it exploited the deterministic robot defenders it was trained and tested against ([analysis](https://www.greaterwrong.com/posts/yHxmJch8dJoH6dwwz/so-has-ai-conquered-bridge)) |
| Skat policy networks | Skat | Bidding and card-play policies from human data | Millions of human games | Learned bidding beat Kermit by 1.17 points per game; naive imitation lost 1.35; imitation card play slightly weaker than search but "orders of magnitude faster" ([Rebstock et al.](https://arxiv.org/abs/1905.10907)) |
| GO-MCTS | Hearts, Skat, The Crew | Transformer generating observation sequences, trained by population self-play ([Rebstock et al. 2024](https://arxiv.org/abs/2404.13150)) | Not stated in abstract | "Promising" |
| Hearts TD learning | Hearts | Linear evaluation with constructed features | Self-play | Beat the best known search-based Hearts players ([Sturtevant & White](https://webdocs.cs.ualberta.ca/~nathanst/papers/heartslearning.pdf)) |
| Euchre DQN/NFSP | Euchre | Model-free reinforcement learning (DQN and Neural Fictitious Self-Play) via the RLCard toolkit | Student project | Beat random play, but won only about half the time against smart rule-based agents ([Pugh 2020](https://web.stanford.edu/class/aa228/reports/2020/final165.pdf)) |
| DouZero | Dou Dizhu | Deep Monte Carlo self-play | One server, 4 GPUs, days | Ranked first of 344 agents on Botzone ([Zha et al. 2021](https://arxiv.org/abs/2106.06135)) |
| Bridge-bidding RL | Bridge | Supervised pre-training, then PPP and fictitious self-play | 1M supervised boards plus 12.5M DD-scored boards | +1.24 IMP per board vs Wbridge5. Prior results: +0.85 (Lockhart), +0.63 (Tian), +0.41 (Gong), +0.25 (Rong). Reinforcement learning alone could not beat Wbridge5 ([Kita et al. 2024](https://arxiv.org/abs/2406.10306)) |
| Uncontested-bidding DQN | Bridge | Deep Q-network | 1M DD-labeled deals | −4.45 IMPs per deal vs constructive par. GIB managed −4.15 and humans −3.77. The network learned to pass far too often ([Bethe 2021](https://cs.nyu.edu/media/publications/bethe_pm_thesis_final.pdf)) |
| ReBeL | Heads-up no-limit hold'em, Liar's Dice | RL plus search over public belief states | Hold'em data generation on 90 DGX-1 machines (8 V100 GPUs each); Liar's Dice on 1 GPU and 60 CPU threads ([Brown et al.](https://arxiv.org/abs/2007.13544)) | Superhuman hold'em; guarantees for two-player zero-sum only |
| Student of Games | Chess, Go, poker, Scotland Yard | Growing-tree CFR plus learned values | TPU resources similar to AlphaZero's 3,500 TPUv4 actors; must enumerate information states per public state ([Schmid et al.](https://arxiv.org/abs/2112.03178)) | Strong, below specialist systems |
| Pluribus | 6-player no-limit hold'em | CFR "blueprint" strategy plus real-time search | 12,400 CPU core-hours (about $144) on a 64-core server ([InfoQ](https://www.infoq.com/news/2019/07/AI-beat-poker-facebook-cmu)) | Superhuman |
| AlphaZe** | Stratego, DarkHex | AlphaZero-style RL for imperfect information | Not stated in abstract | Competitive baseline, below DeepNash ([Blüml et al. 2023](https://doi.org/10.3389/frai.2023.1014561)) |

**Practicality for a small project (estimate)**

- **Very practical:** small networks (or Kermit-style lookup tables) trained on lab data, on a laptop GPU or even a CPU. Good targets:
  - a bidding evaluator that predicts trick distributions per trump from hand plus kitty;
  - a card-location inference model;
  - a "human-likeness" prior for tie-breaking.
  - Run them in the browser with [ONNX Runtime Web](https://www.npmjs.com/package/onnxruntime-web) (WebAssembly, optionally GPU) inside a Web Worker.
- **Feasible but optional:** end-to-end self-play reinforcement learning in the style of DouZero (days on a multi-GPU box). The bridge results show RL bidding needs supervised pre-training and can collapse into degenerate strategies, such as passing too often.
- **Not practical or not suitable:** ReBeL, Student of Games and DeepStack-style public-belief methods. They need large compute, and their theory does not cover four-player team play.
- **Frameworks for experiments:** [OpenSpiel](https://github.com/google-deepmind/open_spiel/blob/master/docs/games.md) includes Bridge, Euchre, Hearts, Spades, Oh Hell, Skat (simplified bidding) and Slovenian Tarok.

### 3.5 Bidding AI: rule-based, simulation-based, learned

**Rule-based bidding**

- **Typical form:** count likely tricks and adjust. AI Factory sums per-card trick probabilities, then adjusts with a "personality factor" (timid vs aggressive) and a small random term ([Whitehouse et al.](https://ojs.aaai.org/index.php/AIIDE/article/view/12679)).
- **500 examples:** NeuralPlay's level 3 uses "human-designed rules" ([App Store reply](https://apps.apple.com/us/app/five-hundred-500-expert-ai/id1559298243)). playboardgames.org's Medium bot prices bids from a hand-strength estimate ([playboardgames.org](https://playboardgames.org/play/500)).
- **Strengths:** fast, predictable, explainable.
- **Weakness:** brittle outside covered situations. Old GIB "would always pass" when no rule matched ([GIBBO interview](https://news.bridgebase.com/?p=87069)).

**Simulation-based bidding**

- **GIB's "Borel" simulation:** project the rest of the auction from a rules database, deal consistent hands, score each candidate bid double-dummy, and take the maximum ([Ginsberg 2001](https://jair.org/index.php/jair/article/view/10279)).
- **Kermit (Skat), the model to copy for 500:**
  - A learned "10+2" evaluator estimates win probability for a 10-card hand plus the 2 discards, per game type.
  - An expectimax search runs over the unknown skat (231 possibilities), the game type (6) and the discard (66), up to 92,862 leaves in a fraction of a second.
  - It bids only if the value is at least B = 0.6, chosen by self-play. Lowering B "leads to riskier bidding"; a clone at 0.55 also played at expert level ([Buro et al. 2009](https://cs.du.edu/~sturtevant/papers/skat.pdf)).
- **playboardgames.org's Hard 500 bot:** samples the unseen cards many times, simulates each deal "through the kitty, discards, and play", and bids the contract with the best average ([playboardgames.org](https://playboardgames.org/play/500)).
- **Where simulation fails:** computers are good at opening bids and at final "decision" bids, but weak at the in-between bids that pass information ([Bethe 2021](https://cs.nyu.edu/media/publications/bethe_pm_thesis_final.pdf)). A reviewer of Ben-style bidders found that DD-based bidding "seems to me to lead to undue optimism" ([IntoBridge review](https://bridgewinners.com/article/view/intobridge-review/)).

**Learned bidding**

- **Ben's bidding:** a recurrent network trained on generated auctions ([Ben training README](https://github.com/lorserker/ben/tree/main/scripts/training/bidding)).
- **Reinforcement learning:** RL bidders in bridge reach about +1.24 IMP per board against Wbridge5, given supervised pre-training ([Kita et al. 2024](https://arxiv.org/abs/2406.10306)).
- **Skat:** learned bidding beat Kermit, while naive imitation of humans was worse ([Rebstock et al.](https://arxiv.org/abs/1905.10907)).

**500-specific considerations**

- **The kitty adds a chance layer.**
  - The bidder does not know which 3 of the 33 unseen cards are in the kitty: C(33,3) = 5,456 possibilities.
  - The declarer then chooses 3 discards from 13: C(13,3) = 286 options.
  - **(Estimate)** A Kermit-style exhaustive expectimax (about 5,456 kitties × 5 denominations × 286 discards) is too big. Sample kitties and prune discards with heuristics or a learned discard policy.
- **Auctions carry information.** In Australian rules, players who pass are out of that auction, and bidding continues until three players pass. In American single-round rules, each player bids once. Inkle bids signal to partner, and "Youngstown" description bids show holdings ([Pagat](https://www.pagat.com/euchre/500.html)). Bots must both *produce* and *interpret* the convention pack the table uses.
- **Misère needs its own evaluator.** DD is pessimistic for misère, as for Skat null. Open misère is closer to DD once the hand is exposed after trick one.
- **Bidding is driven by the score state.** Lockout calls, sacrifices and "must bid to win" rules all depend on it. Reviewers complain about partners that bid the maximum when a smaller bid would win the game ([Extreme 500 reviews](https://apps.apple.com/us/app/-/id775846912)).

### 3.6 Double-dummy solver techniques and realistic performance

**Techniques used by strong solvers**

- **Alpha-beta search on trick counts.** Tricks already won and sure winners give natural bounds. "Sure winners act as a definitive lower bound" ([Wikipedia: Computer bridge](https://en.wikipedia.org/wiki/Computer_bridge)).
- **Target searches.** Ask "can the declarer side take at least t tricks?" as a series of narrow-window yes/no searches.
- **Transposition tables.** Cache positions and their bounds.
- **Partition search.** Ginsberg's partition search makes the table generalize across positions with the same value. Bethe credits it with making double-dummy solving practical, since alpha-beta alone "was still not fast enough" ([Bethe 2021](https://cs.nyu.edu/media/publications/bethe_pm_thesis_final.pdf); [Ginsberg 2001](https://jair.org/index.php/jair/article/view/10279)).
- **Equivalent-card pruning.**
  - Kermit searches only one card from each group of equivalent cards (for example 7 and 8 in hand, or 7 and 9 once the 8 is gone). It warns not to treat a card currently winning the trick as already played.
  - It also merges near-equivalent groups early in the hand (queens with kings, tens with aces) at a small accuracy cost ([Buro et al. 2009](https://cs.du.edu/~sturtevant/papers/skat.pdf)).
- **Move ordering.**
  - DDS weights each legal card by whether it wins the trick, the seat's position in the trick, suit lengths and opponents' holdings, ruff/discard situations and "killer" moves from the transposition table, while keeping "a good mixture of moves" ([DDS heuristic-sorting](https://raw.githubusercontent.com/dds-bridge/dds/main/doc/heuristic-sorting.md)).
  - Kermit's "fastest-cut-first" ordering prefers moves likely to cut off the search cheaply; it cut search effort by about 40% ([Buro et al. 2009](https://cs.du.edu/~sturtevant/papers/skat.pdf)).
- **Bitboards.** Card sets are 32-bit words; features are computed with bit operations ([Buro et al. 2009](https://cs.du.edu/~sturtevant/papers/skat.pdf)).
- **Documentation.** DDS ships algorithm documentation (`Alg-dds_x.pdf`) and benchmarks in its [doc folder](https://github.com/dds-bridge/dds/tree/main/doc).

**Measured performance (bridge and Skat)**

| Measurement | Number | Source |
|---|---|---|
| macroxue solver, 1,000 random 52-card deals, full analysis, single core | 74.2 s total; median 0.05 s, p99 0.37 s per deal; rare hard deals take roughly 1 s to 48 s per denomination | [macroxue/bridge-solver](https://github.com/macroxue/bridge-solver) |
| bridge-craftwork port (same search) | Native: about 110 ms per 20-cell table on one core (14 ms on 12 cores). WebAssembly: about 250 ms per table. Per-card analysis of a deal: about 128 ms in WebAssembly vs about 90 ms native. Worst case: 39.7 s and 1.6 GB | [bridge-craftwork/bridge-solver](https://github.com/bridge-craftwork/bridge-solver) |
| DDS per sampled deal in a bridge PIMC player | About 50 ms at trick 1, about 30 ms at trick 2 | [Bethe 2021](https://cs.nyu.edu/media/publications/bethe_pm_thesis_final.pdf) |
| Lead Solver: DDS on 2,000 deals for one denomination and seat | About 38 s (about 0.02 s per deal) | [Lead Solver](https://lajollabridge.com/Software/Lead-Solver/Lead-Solver-About.htm) |
| Skat PIMC (10 tricks, 3 hands) at 320 sampled worlds per move | 0.093 s per move, so about 0.3 ms per world (derived) | [Solinas et al. 2019](https://arxiv.org/abs/1903.09604) |

**JavaScript and WebAssembly options**

- **For bridge:**
  - DDS 3.x ships a WebAssembly build ([dds-bridge/dds](https://github.com/dds-bridge/dds); [web folder](https://github.com/dds-bridge/dds/tree/main/web)).
  - [@bridge-tools/dd](https://www.npmjs.com/package/@bridge-tools/dd) wraps DDS compiled to WebAssembly, crediting Graham Hazel's [guide to running DDS in a browser](https://grahamhazel.com/blog/2023/02/06/double-dummy-analysis-in-a-web-browser/).
  - [Bridge Solver Online](https://mirgo2.co.uk/bridgesolver/) runs Haglund's solver entirely in the browser.
- **In TypeScript:** a small partnership trick-taking project, koejon, specified a PIMC bot with an exact alpha-beta solver, transposition table and move ordering in `src/bots/solver.ts`; the PR merged in October 2026, though no timings were found ([koejon #86](https://github.com/Winandvdb/koejon/issues/86)).
- **None of these handle 500.** Bridge solvers have no Joker, no bowers, no 10-card hands, no misère with three hands, and no no-trump Joker rules, so 500 needs its own solver. The techniques transfer directly.
- **Threading in the browser.** Multithreaded WebAssembly needs `SharedArrayBuffer`, which requires a secure, cross-origin-isolated page ([MDN](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/SharedArrayBuffer)). DDS's web build ships a `coi-serviceworker.js`, which by its name appears to be the usual shim for this. Independent Web Workers, each running its own share of the sampled deals, avoid the requirement.

**Realistic numbers for a 500 solver (estimate)**

- **Per solve:**
  - 500 has 40 plies (10 tricks × 4 cards), against 52 in bridge and 30 in Skat.
  - Given Skat's ~0.3 ms per world, a well-engineered 500 solver should average well under a few milliseconds per solve (one deal, one trump, one leader) natively.
  - WebAssembly costs about 1.4–2.3× native (bridge-craftwork numbers). Hand-optimized TypeScript with typed arrays and 32-bit masks is likely within a small factor of WebAssembly.
  - Misère solves should be much faster, because the search ends as soon as the declarer is forced to win a trick.
- **Per decision:** PIMC needs a value for every distinct legal card in every sample. That is roughly 4–8 root moves early in the hand, unless the solver returns all move values in one pass, as DDS can.
  - 100–200 samples per decision at trick 1 is about 0.2–1 s in one Web Worker on a desktop.
  - Later tricks are far cheaper.
- **Tails:** set node and time limits, with a heuristic fallback, so a pathological deal cannot freeze a turn.

### 3.7 Difficulty levels and distinct, human-like personalities

**Levers that work**

- **Search budget.**
  - AI Factory used iteration count as its difficulty dial, suggesting low budgets produce "more human-like" mistakes than randomly perturbing an evaluation ([Whitehouse et al.](https://ojs.aaai.org/index.php/AIIDE/article/view/12679)).
  - Stockfish's Skill Level searches several candidate lines internally and plays a weaker one with a level-dependent probability. Its `UCI_Elo` is calibrated against engines, not humans ([Stockfish UCI docs](https://github.com/official-stockfish/Stockfish/wiki/UCI-&-Commands)). Calibrate against humans, not just bots.
- **Rule tiers versus search tiers.**
  - NeuralPlay: level 3 rules, level 6 Monte Carlo ([App Store](https://apps.apple.com/us/app/five-hundred-500-expert-ai/id1559298243)).
  - playboardgames.org:
    - Easy plays "a fair share of random legal cards" and ignores its partner.
    - Medium is rule-based with no card memory.
    - Hard samples and searches.
    - It reports Hard beating Medium in about 60% of matches and Easy in about 78%, and Hard buying "roughly four auctions in five" ([playboardgames.org](https://playboardgames.org/play/500); self-reported).
- **Risk preference.** Kermit's bid threshold B ([Buro et al.](https://cs.du.edu/~sturtevant/papers/skat.pdf)) and AI Factory's timid/aggressive factor ([Whitehouse et al.](https://ojs.aaai.org/index.php/AIIDE/article/view/12679)). Wisser's PVP shows that how a bot values the match score changes its risk-taking ([Wisser](https://www.ijcai.org/Proceedings/15/Papers/025.pdf)).
- **Noise with a temperature.** Pick moves with probability rising with their value, via a softmax. The logit quantal-response model treats the temperature as a rationality parameter: as it goes to zero the player becomes optimal, and as it grows play becomes uniformly random ([QRE](https://en.wikipedia.org/wiki/Quantal_response_equilibrium)). Expensive errors then stay rare while cheap ones happen, as with humans.
- **Human-likeness from data.**
  - Maia trained one engine per rating band on 12 million human games each, and predicts human moves far better than Stockfish or Leela ([McIlroy-Young et al. 2020](https://arxiv.org/abs/2006.01855); [Maia](https://maiachess.com/blog/introducing_maia)).
  - Biasing MCTS with 27,592 human Spades games produced agents that play more like humans while staying competitive ([Devlin et al. 2016](https://ojs.aaai.org/index.php/AIIDE/article/view/12858); [Baier et al. 2018](https://repository.falmouth.ac.uk/2872/)).
  - "Procedural personas" evolve MCTS selection rules to act out archetypal styles ([Holmgård et al.](https://arxiv.org/abs/1802.06881)).
- **Limited memory.** A faithful model of human forgetting is hard to build, but deliberately discarding information is a tunable knob ([Value of Recall](https://arxiv.org/abs/2412.19659)). For 500, "forgetting" can mean not using played low cards or voids when sampling. NeuralPlay ships a "built-in card counter", which suggests tracking cards is hard for its users ([App Store](https://apps.apple.com/us/app/five-hundred-500-expert-ai/id1559298243)).
- **Dynamic difficulty.** MCTS can target an even outcome by changing its selection policy in real time ([Demediuk et al. 2017](https://opus.lib.uts.edu.au/handle/10453/127485)). This is better used sparingly in a trainer, where users want stable, rated opponents.

**Perception beats raw strength.** "The objective measures of strength (such as win rate)... do not necessarily provide a good metric for quality from a commercial AI perspective" ([Whitehouse et al.](https://ojs.aaai.org/index.php/AIIDE/article/view/12679)). In partnership games, partner blunders dominate complaints (Section 4.2).

### 3.8 Evaluating bots fairly

- **Duplicate deals and seat rotation.**
  - Kermit played each of 800 deals with both sides as soloist and defenders, and full games across all six seatings (ABB, BAB, ...) ([Buro et al.](https://cs.du.edu/~sturtevant/papers/skat.pdf)).
  - Sievers & Helmert repeated games "in every possible permutation of player positions" ([Doppelkopf](https://ai.dmi.unibas.ch/papers/sievers-helmert-ki2015.pdf)).
  - AI Factory reused the same 1,000 deal seeds for every configuration and reported Clopper-Pearson 95% intervals ([Whitehouse et al.](https://ojs.aaai.org/index.php/AIIDE/article/view/12679)).
  - The Annual Computer Poker Competition replayed each match with the same cards and the programs' seats exchanged, so a lucky deal for one side becomes the other side's next ([Zinkevich & Littman](https://www.martin.zinkevich.org/publications/competition.pdf)).
- **Variance reduction beyond duplicate.** AIVAT uses a heuristic value estimate and known bot strategies to remove luck from results, cutting the standard deviation of a human-vs-machine poker match by 85% ([Burch et al.](https://arxiv.org/abs/1612.06915)).
- **Sequential testing.** Stockfish's fishtest runs SPRT (sequential probability ratio test), pairing games with colors swapped and scoring each pair as a unit, which uses testing resources more efficiently. Typical bounds are given in Elo ([Stockfish 11 notes](https://stockfishchess.org/blog/2020/stockfish-11/); [SPRT overview](https://www.chessprogramming.org/Sequential_Probability_Ratio_Test)).
- **Ratings.** Elo and Bradley–Terry ratings suit fixed bot teams. TrueSkill infers individual skill from team results and tracks uncertainty ([Herbrich et al.](https://www.microsoft.com/en-us/research/publication/trueskilltm-a-bayesian-skill-rating-system/)).
- **Sample sizes are large.**
  - Skat's 778-point series standard deviation dwarfs real engine differences ([Long et al.](https://ojs.aaai.org/index.php/AAAI/article/view/7562)).
  - Wisser used 30,000–100,000 matches with 98% Agresti-Coull intervals ([Wisser](https://www.ijcai.org/Proceedings/15/Papers/025.pdf)).
  - **(Estimate)** For paired duplicate boards, you need about N ≈ (z_α/2 + z_β)² σ_d² / δ² boards, where σ_d is the standard deviation of the paired per-board difference and δ the difference to detect. With an assumed σ_d of 100 points, detecting δ = 5 points per board at 95% confidence and 80% power needs about 3,100 board pairs. δ = 2 needs about 20,000.
- **Avoid overfitting to your own bots.** NooK may have learned to exploit the deterministic defenders it was tested against ([analysis](https://www.greaterwrong.com/posts/yHxmJch8dJoH6dwwz/so-has-ai-conquered-bridge)). Test against a varied pool of bots and against humans.
- **Opponent and skill modeling.** Perfect Information Post-Mortem Analysis compares a player's moves after the game with the moves a perfect-information search would choose. It estimates opponent strength within a few games and improved Kermit against weaker and stronger players ([Long & Buro 2011](https://www.ijcai.org/Proceedings/11/Papers/110.pdf)). The same idea can estimate a user's skill for the coach.

### 3.9 How strong engines explain decisions for teaching

| Product | Unit of evaluation | Classification | Teaching loop |
|---|---|---|---|
| lichess | Win% = 50 + 50·(2/(1+e^(−0.00368208·cp)) − 1); per-move accuracy = 103.1668·e^(−0.04354·ΔWin%) − 3.1669 | Inaccuracy / mistake / blunder at drops of 0.1 / 0.2 / 0.3 in winning chances on a −1 to +1 scale, i.e. about 5 / 10 / 15 Win% points ([accuracy](https://lichess.org/page/accuracy); [forum](https://lichess.org/forum/general-chess-discussion/what-constitutes-an-inaccuracy)) | "Learn from your mistakes" replays errors before revealing the engine move ([blog](https://lichess.org/blog/WFvLpiQAACMA8e9D/learn-from-your-mistakes)). Puzzles are mined from 600M analysed games, tagged by theme, and each attempt is a Glicko-2 game between player and puzzle ([database](https://database.lichess.org/); [lichess-puzzler](https://github.com/ornicar/lichess-puzzler)) |
| chess.com Game Review | Expected points (0 to 1) from rating and evaluation | Best 0, Excellent ≤0.02, Good ≤0.05, Inaccuracy ≤0.10, Mistake ≤0.20, Blunder >0.20 points lost. "Great", "Brilliant" and "Miss" are special classes, more generous for newer players ([support](https://support.chess.com/en/articles/8572705-how-are-moves-classified-what-is-a-blunder-or-brilliant-move)) | Key moments, Coach explanations of *why*, Retry on mistakes with adjusted accuracy, linked puzzles and lessons ([Game Review](https://support.chess.com/en/articles/8584089-how-does-game-review-work)) |
| GTO Wizard (poker) | EV lost vs a game-theory-optimal (GTO) solution | Inaccuracy (yellow) = an action the solution takes less than 3.5% of the time; Mistake (red) = never taken or loses EV. A GTO score above 85% is good, and 100% is unrealistic ([help](https://help.gtowizard.com/tips-and-tricks-to-study-analyzed-hands/)) | Sort by EV loss. "Separate outcome from process": biggest losses aren't necessarily biggest mistakes ([help](https://help.gtowizard.com/tips-and-tricks-to-study-analyzed-hands/); [Analyze mode](https://help.gtowizard.com/analyze-mode-guide/)) |
| PokerSnowie | EV of each decision vs its own play | "Blunders" vs "errors"; tiny mixed-strategy deviations marked minor ([review](https://pokerfuse.com/features/reviews/review-poker-snowie-promises-perfect-gameplay-training-with-new-gto-based-learning-software-01-10/)) | Per-decision advice, including mixed strategies |
| eXtreme Gammon / GNU Backgammon | Equity lost | Performance Rating = average equity lost per *non-obvious* decision × 500. Tiers: 0–2.5 world champion, 5–7.5 expert, 17.5–22.5 casual ([XG](https://en.wikipedia.org/wiki/EXtreme_Gammon)) | **Luck analysis**: each roll rated against the average roll ("very lucky" above 0.6), reported alongside the error rate ([GNU Backgammon](https://www.gnu.org/software/gnubg/manual/html_node/Luck-analysis.html)) |
| Funbridge (Argine) | Double-dummy tricks, plus comparison with the field | Shows "at which trick you played the wrong card, how many tricks you let go" and better cards; flags bids inconsistent with your conventions | The most-played contracts and how many tricks other players took in the same trump suit. Commenters note DD analysis flags moves that "aren't really mistakes" and asked for single-dummy analysis ([Funbridge blog](https://funbridge.com/blog/?p=31217)) |
| NeuralPlay Bridge / Five Hundred | Difference from the AI's choice | Binary: "differs from AI" | Hints, bid/play checker, double-dummy explorer (Bridge), trick-by-trick review, replay ([Bridge](https://www.neuralplay.com/bridge.html); [500](https://www.neuralplay.com/fivehundred.html)) |
| Bridge Baron | Rules and engine | Bidding hints "with an explanation" | Info buttons for conventions and signals; deals viewable face-up ([App Store](https://apps.apple.com/us/app/-/id1465176932)) |
| Jack 6 | Engine | "Professor Jack" analyses your play and advises (vendor claim) | Adjustable opponents, DD problem solver ([listing](https://chess.co.uk/products/jack-6-bridge-program-for-players-at-any-level-pc-download-version)) |
| Bridge Master (Fred Gitelman) | Graded declarer problems at 5 levels | The defense exploits your error so the contract fails | An explanation of the correct line after every deal ([2019 description](https://site.pianola.net/sheffield/news/20190413)) |
| Lead Solver | DD over deals generated to match the auction | Average defensive tricks and % sets per lead | Distribution tables; warns DD "already knows where every card is" ([Lead Solver](https://lajollabridge.com/Software/Lead-Solver/Lead-Solver-About.htm)) |

**Lessons for a 500 coach**

1. **Measure loss in an outcome-relevant currency.** Chess uses expected points, poker EV, backgammon equity. For 500, use game-win probability where possible, and points or make-percentage within a hand.
2. **Exclude forced and obvious decisions from skill metrics,** as backgammon's Performance Rating does.
3. **Judge from the player's information, not hindsight.** Funbridge's double-dummy grading is criticized for exactly this.
4. **Report luck separately from errors,** as GNU Backgammon does.
5. **Close the loop with retries and mined puzzles,** as lichess and chess.com do.

---

## 4. Part 2: Product landscape

### 4.1 500 apps and websites

| Product | Platform | Rules and variants | AI | Learning features | Notes |
|---|---|---|---|---|---|
| **Five Hundred (500) – Expert AI** (NeuralPlay) | iOS, Android, Amazon | Presets: Australian, Australian without Nullo, American, Canadian, Cutthroat (3-player), American Double Nullo. Options: kitty size and deck, 1–2 jokers, single or multiple bidding rounds, inkle bids, nullo variants, no-trump/Joker rules, Avondale/Original/Perfect scoring, slam award, defender scoring, must bid to win, game length ([NeuralPlay](https://www.neuralplay.com/fivehundred.html); [App Store](https://apps.apple.com/us/app/five-hundred-500-expert-ai/id1559298243)) | Six levels; level 6 Monte Carlo with randomness, level 3 rules; "unique play styles for each AI seat" | Hints, bid/play checker, card counter, trick-by-trick review, replay, undo, practice mode (hints and undo off), play all four hands, statistics | Rated 4.6 from 2.2K (US) and 2.4K (AU) ratings. Android: 100K+ installs ([APKCombo](https://apkcombo.com/five-hundred-500-expert-ai/com.neuralplay.android.fivehundred/)), about 4.3 from about 1.8K ratings ([Chrome-Stats](https://chrome-stats.com/d/id1559298243)). Founder has an AI PhD; NeuralPlay's bridge AI uses Monte Carlo simulation ([Google story](https://www.google.ca/ads/publisher/stories/neuralplay)). No 2-player mode; developer says per-seat levels are "on our list" |
| **500 – Multiplayer card game** (rotundtapir, open source, GPL-3.0) | Android (F-Droid), web | Australian 43-card; Misère and Open Misère; house rules to turn misère or no-trumps off; 2-, 4- and 6-player formats ([F-Droid](https://f-droid.org/en/packages/io.github.rotundtapir.fivehundred/)) | "Heuristic bot strategy" ([GitHub](https://github.com/rotundtapir/500)) | Interactive tutorial; finished tricks stay visible; score sheet | Kotlin Multiplatform with a Kotlin/Wasm web build ([web](https://rotundtapir.github.io/500/)). Online rooms by code; bots fill empty seats; v0.6.5 in Sep 2026 |
| **playboardgames.org/play/500** | Web | Australian partnership; misère only after a 7-level bid; open misère 500; Joker playable in no-trumps only when void | Easy (partly random), Medium (rules, no memory), Hard (sampling bids through kitty and play; PIMC with void tracking). Self-reported: Hard beats Medium about 60%, Easy about 78% | None beyond a how-to guide; illegal cards greyed | Private rooms, no pass-and-play ([site](https://playboardgames.org/play/500)) |
| **PiecePals** | Web | Australian (43) or American (45) variant | Easy / Medium / Hard | None noted | Real-time multiplayer, bots fill seats ([site](https://piecepals.com/play/five-hundred)) |
| **Trickster 500** | Web and apps | Australian (multi-round, misère) and American (single bid, inkles, nullo); 3, 5 or 6-card kitty; 3-player cutthroat; 2v2v2 and 3v3 | Not described on the page. An App Store reviewer calls Trickster's bots "useless" ([Trickster](https://www.trickstercards.com/games/500/); [App Store](https://apps.apple.com/app/id982267355)) | How-to and beginner pages | Video chat, daily challenges, leaderboards |
| **Board Game Arena: 500** | Web | 2, 4 or 6 players; American single-bid or Australian; misère and open misère; "reduced misère" option | No bots mentioned | None noted | Available since April 2025 ([BGA](https://en.boardgamearena.com/gamepanel?game=fivehundred)) |
| **Ultimate 500** (Mark Deitch) | iPad ($0.99; Lite free) | Customizable per the "500 wiki" | "Intelligent AI opponents"; AI can fill online seats | Suggest feature for bids and cards; undo | 3.3★ from 4 ratings; "Bad A.I., so-so interface"; online invites broken ([App Store](https://apps.apple.com/app/id641417505); [Lite](https://apps.apple.com/app/id956406859)) |
| **Extreme 500** (James Swiney) | iOS | 4-player partnership or 3-player; misère; three scoring systems; adjustable deck | "Intelligent and challenging AI" | None noted | 4.4★ from 577 ratings; strong partner complaints (Section 4.2) ([App Store](https://apps.apple.com/us/app/-/id775846912)) |
| **500 Card Game** (OEngines) | iOS, Android | American, Australian, custom | "Smart AI with adaptive difficulty" | None noted | Coins, daily quests, ads, in-app purchases; misère freeze bug reported ([App Store](https://apps.apple.com/us/app/-/id6480395820); [Play](https://play.google.com/store/apps/details?id=com.oengines.online500cards)) |
| **Five Hundred (500)** (tencardgame) | Android | Not detailed | Unknown | Unknown | Complaints of stacked deals, partner seeming to know your hand, partner raising your bid ([Play](https://play.google.com/store/apps/details?id=com.tencardgame.ten_five_hundred); [AppBrain](https://www.appbrain.com/app/five-hundred-500/com.tencardgame.ten_five_hundred)) |
| **Five Hundred – Card Games** (Topy Games) | Android | Includes nullo | Unknown | Unknown | Complaints about partner not helping defeat opponents' nullo and Joker ranking in nullo ([Play](https://play.google.com/store/apps/details?id=fivehundred.free.online.offline.classic.card.game)) |
| **500+plus** (Michael Glickman / Palmcrust) | iOS | 3-player only; tweakable scoring, Joker use, misère limits | Two computer players | Rules guide | 3.0★ from 1 rating ("Can NOT bid") ([App Store](https://apps.apple.com/us/app/-/id1521924208)) |
| **Five Hundred** (Oliver Giles, 2012) | Linux, macOS, Windows | Not stated | Scriptable in Lua | None | Free software, Qt, dormant ([site](https://fivehundred.sourceforge.net/)) |
| Casual web clones: [cardgames.club](https://www.cardgames.club/five-hundred/), [500cardgame.wtf](https://500cardgame.wtf/) | Web | Basic | Fixed partner and opponents | None | Additional Android listings: [atakala](https://play.google.com/store/apps/details?id=com.atakala.fivehundred&hl=en), [RHH Technology](https://play.google.com/store/apps/details?id=com.rhhtechnology.fivehundred&hl=en_US), [Five2](https://play.google.com/store/apps/details?id=com.five2.play.fivehundred&hl=en_AU) |

### 4.2 What players complain about

1. **Partners who overbid or bid hands they cannot make.**
   - "...sometimes your partner (the computer) will bid a hand that they don't have the cards to win" ([NeuralPlay review](https://apps.apple.com/us/app/five-hundred-500-expert-ai/id1559298243)).
   - "Why does my partner insist on calling the highest number of tricks possible when we only need a smaller number to win?" ([Extreme 500](https://apps.apple.com/us/app/-/id775846912)).
   - A partner who "automatically raises it to eight" over the player's seven bid, and opponents who call 7 or 8 "every second hand" ([tencardgame, via search extracts](https://play.google.com/store/apps/details?id=com.tencardgame.ten_five_hundred&hl=en_US)).
   - Another reviewer complained about "the AI continually going NT about 95% of the time" ([NeuralPlay AU](https://apps.apple.com/au/app/five-hundred-500-expert-ai/id1559298243)).
2. **Wasting the Joker or trumps on partner's winner.**
   - "North (your partner) will waste a trump by overtrumping when you already have the trick" ([NeuralPlay](https://apps.apple.com/us/app/five-hundred-500-expert-ai/id1559298243)).
   - Extreme 500 reviewers describe partners playing the Joker on the reviewer's ace and "messing up a run", and ask that the partner "not play the joker on No Trump when I play an ace" ([Extreme 500](https://apps.apple.com/us/app/-/id775846912)).
   - A NeuralPlay review describes a partner that "will lead trump" when only the partnership holds trumps, and that interrupts a no-trump run by playing the Joker unnecessarily.
   - This is the same "wastes a high card" complaint AI Factory found in Spades ([Whitehouse et al.](https://ojs.aaai.org/index.php/AIIDE/article/view/12679)).
3. **Misère/nullo handling.**
   - "...super frustrating when your team mate constantly calls open nullo" ([NeuralPlay AU](https://apps.apple.com/au/app/five-hundred-500-expert-ai/id1559298243)).
   - A partner who "purposefully keeps not helping me make the other team lose their nullo" ([Topy, via search extracts](https://play.google.com/store/apps/details?id=fivehundred.free.online.offline.classic.card.game&hl=en_US)).
   - Bots bidding misère below the current bid ([Extreme 500](https://apps.apple.com/us/app/-/id775846912)).
4. **Suspicion of rigged deals or cheating.**
   - "West gets dealt the joker an inordinate amount of the time." The developer replied that deals are not biased by seat ([NeuralPlay](https://apps.apple.com/us/app/five-hundred-500-expert-ai/id1559298243)).
   - "Not a fair supposedly random deal of cards" ([NeuralPlay AU](https://apps.apple.com/au/app/five-hundred-500-expert-ai/id1559298243)).
   - "The cards are stacked against you and the computer player who are partners actually know what you have" ([tencardgame, via search extracts](https://play.google.com/store/apps/details?id=com.tencardgame.ten_five_hundred&hl=en_US)).
   - Elsewhere, one "AI cheating" report turned out to be a real rules bug: the AI could play an illegal card because its moves were never validated. The developer then checked 200,000 simulated games for illegal moves ([Whisthub](https://www.whisthub.com/blog/the-kingen-ai-has-been-cheating)).
5. **Missing modes and options.**
   - "NO 2 Player Version"; a request that "each computer player level should be able to be set individually"; no real-time play with humans ([NeuralPlay AU](https://apps.apple.com/au/app/five-hundred-500-expert-ai/id1559298243)).
   - House rules such as having to make a bid to win the game, which NeuralPlay supports through its "Must bid to win" setting ([NeuralPlay US](https://apps.apple.com/us/app/five-hundred-500-expert-ai/id1559298243)).
6. **Rules bugs and user interface.**
   - Trump beaten by a non-trump; freezes on the last trick or during misère (search extracts for [OEngines](https://play.google.com/store/apps/details?id=com.oengines.online500cards&hl=en_US) and other listings).
   - Unchangeable rule checkboxes and tiny bid buttons ([Ultimate 500](https://apps.apple.com/app/id641417505)).
7. **What players praise.** Rule customization, and learning from the game: one reviewer says it made them "a more aggressive bidder" ([NeuralPlay](https://apps.apple.com/us/app/five-hundred-500-expert-ai/id1559298243)).

### 4.3 Trainers worth borrowing from

| Feature | Who does it | How to adapt for 500 (recommendation) |
|---|---|---|
| Graded mistake classes by expected loss | chess.com, lichess, GTO Wizard | Grade each bid, discard, lead and play by loss in game-win probability (or points), with thresholds tuned on lab and human data |
| Coach explanations and key moments | chess.com Game Review | Short "why" sentences from the engine's own analysis (Section 5.5), not free-form text |
| Retry the position | chess.com, lichess | "Try again" from any flagged decision, then play on against the same hidden cards |
| Puzzles mined from games, rated with Glicko-2 | lichess | Mine "only move" positions from lab and user games; tag by theme (Joker management, bower counting, misère defense) |
| Error rate over non-obvious decisions; luck analysis | XG, GNU Backgammon | 500 error rate per phase; deal, kitty and layout luck to answer "rigged" complaints with data |
| EV-loss sorting; "outcome vs process" | GTO Wizard | Review sorted by largest losses, not by hands lost |
| Duplicate comparison with the field | Funbridge, BBO robot tournaments scored like duplicate ([BBO](https://online.bridgebase.com/v2/doc_about_robot_tourneys.html)) | Daily deal played by everyone and by expert bots; show what the field and the bot did |
| Challenge the AI | Funbridge (Argine won 80% of 5-deal challenges, [blog](https://funbridge.com/blog/?p=12678)) | Head-to-head duplicate challenge against the expert bot |
| Bidding checks against your agreements | Funbridge, NeuralPlay, Bridge Baron | Flag bids inconsistent with the chosen convention pack (inkle, "Youngstown" description bids) |
| Graded problem sets where the defense punishes errors | Bridge Master; ACBL's free *Learn to Play Bridge* (lessons, practice against robots, skill tests; [BBO forum](https://www.bridgebase.com/forums/topic/62906-acbl-software-learn-to-play-bridge/)) | Levelled 500 problems in which bots defend against the user's specific mistake |
| Simulation tables for a decision | Lead Solver | For leads, discards and bids: a table of make-%, average tricks and EV per option over consistent deals |
| Hint and checker | NeuralPlay, Ultimate 500 | Keep, but add severity and reasons |
| Human-like opponents by level | Maia | If human data accumulates, train level-specific human models as opponents and partners |

**Bridge engine context.** Jack (several World Computer Bridge Championship titles), Wbridge5 (champion 2005, 2007–08, 2016–18, and the unofficial 2022 event) and Shark Bridge are strong players but are documented mainly as opponents rather than teachers ([Wikipedia](https://en.wikipedia.org/wiki/Computer_bridge)). Their lesson for us is engine quality, not teaching interface.

### 4.4 Gaps we can fill

- **No 500 product grades or explains decisions.** NeuralPlay's checker flags only differences from its own choice; its site lists hints, checker and review but no explanation of *why* ([NeuralPlay](https://www.neuralplay.com/fivehundred.html)).
- **No luck/skill separation or deal-fairness evidence,** despite persistent "rigged" complaints.
- **Partner quality is the main source of frustration.** No product markets a partner that reads bids, covers misère, and avoids overtaking its partner.
- **No drills, puzzles, bidding quizzes, or duplicate comparisons** with other players or bots.
- **Variant breadth exists only at NeuralPlay.** Our engine must be variant-driven from day one: Australian 4-player default, American single-round, misère and Joker rule toggles, 2/3/6-player later.

---

## 5. Recommendations

### 5.1 AI architecture for the TypeScript web app (recommendation)

**Modules** (one TypeScript codebase shared by the UI, the Web-Worker AI, and the Node lab)

```
rules/      pure, deterministic engine: variant config, seeded dealing, legal moves, trick resolution,
            scoring, game end, hand-history format. The ONLY implementation of the rules.
observe/    Observation = exactly what one seat may know (own cards, auction, plays, own discards,
            exposed open-misère hand). Bots and coach receive ONLY this. Never the full state.
belief/     hard constraints (voids, known cards, Joker nomination, hand sizes); exact uniform sampler
            via per-suit counting; soft likelihoods from bid/play models; importance weights.
solver/     500 perfect-information solver: contract-specific card remapping (bowers to trump suit,
            Joker as top trump or suitless NT card with nomination/lead rules), 4-hand trick play,
            3-hand misère with "zero tricks" goal; alpha-beta on trick targets, transposition table
            with bounds, rank-equivalence pruning, move ordering, node/time limits.
search/     PIMC driver: same sampled worlds for every candidate action; utility models (expected
            hand score, make-probability, game-win probability from a lab-estimated table); tie-breakers
            (plausibility prior, information leakage); optional PPP-style opponent-knowledge factor;
            later: alpha-mu/EPIMC endgame mode.
bidding/    hand+kitty evaluator (trick distribution per denomination + misère risk); candidate bids;
            auction rollouts with the bidding policy for all seats; convention packs (produce and
            interpret).
bots/       level x personality -> parameter object (budgets, noise, risk, memory, conventions).
coach/      per-decision EV tables, loss classes, explanations, luck/skill, review timeline, puzzles.
lab/        Node worker_threads runner, duplicate scheduler, metrics, SPRT, ratings, dataset export.
```

**Key design decisions**

1. **Strict information hiding.**
   - The bot interface accepts an `Observation`, never the game state.
   - Add a property test: re-deal the hidden cards consistently with the observation and assert the bot's decision distribution is unchanged.
   - This answers the "AI cheats" suspicion architecturally ([Whitehouse et al.](https://ojs.aaai.org/index.php/AIIDE/article/view/12679); [Whisthub](https://www.whisthub.com/blog/the-kingen-ai-has-been-cheating)).
2. **Remap each contract once.**
   - At the start of play, map the 43 physical cards to "effective suit + rank" for that contract: left bower into trumps, Joker on top, the other jack's suit shortened.
   - The solver core then works on four 16-bit suit masks per hand. In JavaScript use `Uint16Array` or 32-bit integers and avoid `BigInt` in hot loops.
3. **Write a 500 solver; don't adapt a bridge one.** Port the techniques: trick-target alpha-beta, transposition table, equivalence, ordering. Bridge solvers lack the Joker, bowers, misère and 10-card hands.
4. **Score actions by utility, not tricks.**
   - Default: expected change in score difference within the hand, including defender points and the slam rule.
   - Game mode: expected probability of winning the game, using a table P(win | our score, their score, dealer) estimated by lab self-play, as in Wisser's PVP ([Wisser](https://www.ijcai.org/Proceedings/15/Papers/025.pdf)).
   - Risk-tolerant and risk-averse personalities transform this utility.
5. **Paired evaluation of candidates.** Evaluate every candidate action on the same sampled worlds. Common random numbers make differences far less noisy, which matters for both play and coaching.
6. **Treat misère as its own problem.** Pure double-dummy is pessimistic for the declarer. Use a PPP-style opponent-knowledge factor, or rollouts with realistic defender policies; validate in the lab.
7. **Plausible play.** Among actions within an EV tolerance:
   - prefer the one that reveals less (Bethe's information-score idea);
   - prefer the one a human would choose (heuristic or learned prior);
   - never overtake a partner's sure winner with the Joker or a trump unless it gains.
8. **Anytime search and time budgets.**
   - Workers run until a deadline and return the current best. The UI shows "thinking" for longer decisions.
   - **(Estimate)** Budgets: about 300–800 ms per card on desktop and 150–300 ms on phones, honoring AI Factory's 250 ms comfort guideline for routine plays. Up to 2–3 s for bids and kitty discards; longer (streaming) for coach analysis.
   - Use one worker per logical core, each solving its own share of the samples, to avoid the cross-origin-isolation requirements of shared memory.
9. **Lab throughput (estimate).**
   - A full-strength PIMC hand (about 40 card decisions plus bidding) likely costs a few core-seconds. One million such hands is roughly weeks of single-core time, or days on a 16–32 core machine.
   - Use tiers:
     - rule or learned-policy bots for bulk data (millions of hands);
     - PIMC with small sample counts for most evaluation;
     - full-strength runs for final checks;
     - DD "par" solves per deal (a few dozen solves) to label training data cheaply. Correct DD's bias with PIMC-labelled subsets.
10. **WebAssembly later, if at all.** Start in TypeScript. Benchmark early on a fixed corpus that includes deals with many voids. If the solver is the bottleneck, port only `solver/` to Rust or C++ compiled to WebAssembly behind the same interface. Prior ports ran about 1.4–2.3× slower than native ([bridge-craftwork](https://github.com/bridge-craftwork/bridge-solver)).
11. **Learned components behind interfaces.** Bidding evaluator, inference model, plausibility prior: start with hand-written or table versions, swap in small networks via ONNX Runtime Web once lab data exists.

### 5.2 What changes if Python is used

- **Search speed.** Pure-Python game-tree search is far too slow for a solver. The solver must be native: C, C++ or Rust via pybind11 or PyO3, or Numba.
- **Browser delivery.** Python does not run AI well in the browser. You would either serve AI from a backend (latency, cost, no offline play) or still need a JavaScript or WebAssembly engine for the client.
- **Best split if you want Python's ecosystem.** Keep the TypeScript (or Rust-core) engine as the single source of rules and search. Use Python for the lab's analysis and training: pandas or polars, PyTorch, notebooks, and frameworks such as [OpenSpiel](https://github.com/google-deepmind/open_spiel/blob/master/docs/games.md). Python can drive the engine through a CLI or JSON-lines protocol, or through a Rust core compiled both to WebAssembly and to a Python extension. Export models to ONNX for the browser.
- **If rules are implemented twice,** maintain shared "golden" test vectors (deals, legal-move sets, scores, solver values) and run them in both languages in CI. Rule bugs are both the most damaging and the most visible errors ([Whisthub](https://www.whisthub.com/blog/the-kingen-ai-has-been-cheating)).

### 5.3 Difficulty and personalities

**Level ladder (recommendation; calibrate with the rating ladder in 5.4)**

| Level | Card play | Bidding | Inference and memory | Noise |
|---|---|---|---|---|
| 1 Beginner | Rule bot: follow suit, win cheaply, basic trump drawing | Simple trick count ± random | Forgets non-trump cards; ignores voids | High temperature |
| 2 Casual | Rule bot plus top-trump counting | Evaluator, conservative threshold | Tracks trumps and voids | Medium |
| 3 Club | PIMC, about 20 samples, hard constraints only | Evaluator plus light simulation | Played cards and voids | Low (softmax over EV in points) |
| 4 Strong | PIMC, 60–100 samples, bid-based inference | Simulation for close bids; misère model | Plus bids | Very low |
| 5 Expert | PIMC, 150–300 samples, bid and play inference, opponent-knowledge factor, game-state utility, plausible tie-breaks | Full simulation; convention-aware | Full | Tie-breaks only |
| 6 Analyst | As level 5 with larger budgets and endgame upgrades (later αμ/EPIMC) | As level 5 | Full | None; used by the coach |

**Personality parameters** (orthogonal to level, recommendation)

- **Bid threshold:** required make-probability or EV margin. Cautious about 0.65, balanced about 0.58, aggressive about 0.50, with Kermit's 0.6 and 0.55 as reference points ([Buro et al.](https://cs.du.edu/~sturtevant/papers/skat.pdf)).
- **Risk aversion:** maximize mean minus λ·standard deviation, or a lower-tail average (CVaR). Affects safety plays and sacrifices.
- **Lockout and sacrifice appetite:** how much negative EV the bot accepts to stop the opponents' bigger contract, scaled by their distance to 500.
- **Misère appetite:** threshold multiplier for misère and open misère. Keep it low by default; frequent open-misère calls annoy players.
- **Style priors used for tie-breaking:** "trump drawer" vs "cross-ruffer"; "Joker as control" vs "Joker early"; lead conventions.
- **Partner trust:** how literally the bot reads partner bids during inference.
- **Memory and attention:** probability of using each played low card or void when sampling. This gives human-like forgetting at lower levels.
- **Search budget and temperature:** as in the level table. Noise is a softmax over EV in points, so costly blunders stay rare ([QRE](https://en.wikipedia.org/wiki/Quantal_response_equilibrium)).

**Rules of thumb**

- Partners default to at least level 3 and never show "beginner" partner behavior unless the user asks.
- Allow per-seat level and style, a feature reviewers requested.
- Expose personalities with names and one-line descriptions ("Cautious Carol: bids only solid hands, defends patiently").
- Keep strength roughly equal across personalities within a level, verified by rating.
- Watch auction share: a Hard bot buying four auctions in five ([playboardgames.org](https://playboardgames.org/play/500)) may be strong but feels oppressive.

### 5.4 How to evaluate bots (recommendation)

1. **Deal bank.** Fixed seeds, stratified by hand type: strong trump hands, no-trump hands, misère candidates, Joker and bower distributions. Keep separate test and development sets.
2. **Duplicate protocol.** Play every deal twice with partnerships swapped (A as N/S then E/W) and the same dealer. Score the per-deal paired difference in (our points − their points).
3. **Full games.** Play games to 500 using the same deal sequences with seats swapped; report game-win rate with confidence intervals.
4. **Mixed partnerships.** Pair bot X with bot Y to measure partner compatibility, a proxy for human partners.
5. **Statistics.**
   - Paired means with bootstrap confidence intervals.
   - SPRT for accepting or rejecting engine changes, scored over deal pairs as fishtest scores game pairs ([chessprogramming](https://www.chessprogramming.org/Sequential_Probability_Ratio_Test)).
   - Sample sizes from the formula in 3.8.
   - Later, AIVAT-style control variates using the solver's per-deal par value ([Burch et al.](https://arxiv.org/abs/1612.06915)).
6. **Ratings.** An Elo / Bradley–Terry ladder of fixed configurations, anchored by random and rule bots. Check that levels are monotone and evenly spaced. Use TrueSkill or Glicko-2 for users if rated play is added.
7. **Behavior diagnostics:**
   - auction share;
   - contract make-rate by level and denomination;
   - misère frequency and success;
   - set rate;
   - average tricks against single-dummy expectation;
   - "partner overtake" count;
   - "oddball play" detector hits;
   - decision-time percentiles.
8. **Humans in the loop.**
   - Add a "Flag this play" button that captures seed, observation and engine output, as AI Factory's beta testers did ([Whitehouse et al.](https://ojs.aaai.org/index.php/AIIDE/article/view/12679)).
   - Run blind A/B tests of partner quality.
   - Add golden regression positions for every confirmed complaint, such as "don't Joker partner's ace in no-trumps".
9. **Check PIMC suitability.** Measure leaf correlation, bias and disambiguation for 500 ([Long et al.](https://ojs.aaai.org/index.php/AAAI/article/view/7562)).
10. **Avoid self-play monoculture.** Evaluate against a varied pool of bots (rule, PIMC and ISMCTS bots across personalities) and against humans ([NooK critique](https://www.greaterwrong.com/posts/yHxmJch8dJoH6dwwz/so-has-ai-conquered-bridge)).

### 5.5 Coaching features to build, and how (recommendation)

**Core evaluation method (shared by every coaching feature).** For a decision, sample worlds from *the user's* information state with the same inference the expert bot uses. Evaluate every candidate action on the same worlds with the level 6 engine, then report:

- EV per action, in points and, in game mode, game-win probability;
- make-probability for the declaring side;
- a confidence interval.

Flag a decision only if its loss exceeds the threshold *and* the interval excludes "close call". Show double-dummy hindsight separately and label it as such, avoiding Funbridge's problem ([Funbridge](https://funbridge.com/blog/?p=31217)).

**MVP**

1. **Hint with reasons.** Top three options with EV and make-%, plus one short reason. Reasons come from:
   - *contrastive sample summaries*, e.g. "your play fails in 38% of consistent layouts, mostly when East holds the left bower and can overruff";
   - *principle detectors*, such as trumps not drawn, partner's winner overtaken, Joker spent early, discarding from partner's suit, "kitty count".
2. **Graded checker.** Optional live alerts by severity; starting thresholds are given after this list.
3. **Hand review.**
   - A timeline with an EV graph, key moments (largest drops), and per-decision classes.
   - Bidding, kitty discard and lead analysis tables in the style of Lead Solver.
   - A "what partner's bid told you" panel.
   - "Replay from here" (retry).
4. **Luck vs skill per hand and per session,** as in GNU Backgammon:
   - deal luck: expected value of the hand at the deal against the average;
   - kitty luck;
   - layout luck: actual result against the single-dummy expectation of the line played;
   - skill: the sum of decision losses.
5. **Deal-fairness evidence,** to answer "rigged" complaints with data.
   - A "deal fairness" page: Joker and bower frequency by seat, hand-strength histograms, confidence bands. Seeded, exportable deals.
   - Never condition deals on difficulty. One cribbage app states that its seeded shuffle is independent of difficulty, which changes only how the computer plays, and that judging fairness takes many recorded games, not a few memorable ones ([Cribbage JD, via search summary](https://cardsjd.com/cribbage/fair-shuffle/)).
   - For online play, publish a hash of the shuffle seed before the deal and reveal the seed afterwards so players can verify the deal.
6. **Card-reading aids.** A toggleable belief heatmap ("where is the Joker?") from the engine's sampler, and an optional counter. In training mode the counter is hidden and the user is quizzed ("how many trumps are out?").

**Starting thresholds for the checker (tune with lab and human data)**

- Loss in hand EV: inaccuracy 10–25 points, mistake 25–60, blunder above 60.
- Or loss in game-win probability: 2–5% / 5–10% / above 10%.
- Forced and equivalent cards are excluded.

**Version 2**

7. **Skill profile.** Error rate per phase (bidding, discard, lead, declarer play, defense, misère) excluding obvious decisions, as in backgammon's Performance Rating. Trend charts, and post-mortem skill estimation in the spirit of PIPMA ([Long & Buro 2011](https://www.ijcai.org/Proceedings/11/Papers/110.pdf)).
8. **Puzzle trainer.** Mine positions from lab and user games where the best action beats the second best by a wide margin. Tag by theme and rate users and puzzles with Glicko-2, as lichess does ([database](https://database.lichess.org/)). Prioritize puzzles built from the user's own mistakes.
9. **Duplicate against bots and the field.** "Play this deal again" with expert bots in your seat for comparison; daily deal with leaderboard (Funbridge/BBO-style).
10. **Bidding trainer.** Quizzes showing the EV table across bids including pass; P(make), P(set) and the effect of the score state; convention-pack consistency checks.

**Version 3**

11. **Graded lesson tracks** in the style of Bridge Master: problems where bots defend against your mistake. Topics include drawing trumps and counting bowers, Joker management in no-trumps, misère declarer play and defense, kitty discards, and lockout bidding.
12. **Personal tendency reports,** e.g. "you underbid hands with the Joker and one bower by about 0.4 tricks".
13. **Optional natural-language paraphrase** of the coach's structured findings. Only engine-produced facts are verbalized; nothing is computed by a language model.

### 5.6 Pitfalls to avoid

1. **Information leaks and unvalidated AI moves.** Both destroy trust. Use the observation-only interface, legality assertions in every simulated hand, and a property test that decisions don't depend on hidden cards ([Whisthub](https://www.whisthub.com/blog/the-kingen-ai-has-been-cheating)).
2. **Coaching from hindsight.** Grade on the user's information set; label double-dummy views as "with all cards visible" ([Funbridge comments](https://funbridge.com/blog/?p=31217)).
3. **Double-dummy bias in the wrong direction.** It is optimistic for bidding ([IntoBridge review](https://bridgewinners.com/article/view/intobridge-review/)) and pessimistic for misère and null ([Kermit](https://cs.du.edu/~sturtevant/papers/skat.pdf)). Model defender ignorance; validate bidding by simulated play, not double-dummy alone.
4. **Maximizing tricks instead of score.** It misses safety plays and lockout bids ([Bethe](https://cs.nyu.edu/media/publications/bethe_pm_thesis_final.pdf); [play500.com](https://play500.com/calling1.html)).
5. **Biased samplers.** "Deal the bidder's high cards first" generators distort inference ([AI Factory nil example](https://ojs.aaai.org/index.php/AIIDE/article/view/12679)). Sample exactly under hard constraints, then reweight.
6. **Random tie-breaks and Monte Carlo noise read as stupidity.** In 6 of AI Factory's beta reports, the bad move was chosen in under 5% of reruns. Use paired evaluation, tolerance bands, plausibility and information-hiding tie-breaks.
7. **Partner blunders are disproportionately noticed.** Invest first in partner behaviors humans check: no overtaking partner's winner, sensible Joker use, covering partner's misère, reading partner's bids, score-aware bidding.
8. **Over-aggressive "expert" bots.** High auction share and frequent no-trump or open-misère calls feel unfair even when correct ([NeuralPlay AU reviews](https://apps.apple.com/au/app/five-hundred-500-expert-ai/id1559298243)). Calibrate to human norms; let users pick styles.
9. **Underpowered evaluation.** Luck dominates. Without duplicate and paired statistics you will "tune" on noise ([Long et al.](https://ojs.aaai.org/index.php/AAAI/article/view/7562)).
10. **Performance cliffs.** Rare deals take tens of seconds in bridge solvers ([macroxue](https://github.com/macroxue/bridge-solver); [bridge-craftwork](https://github.com/bridge-craftwork/bridge-solver)). Use node and time limits, anytime search, fallbacks; never block the UI thread.
11. **Variant sprawl and rule ambiguity.** No-trump Joker rules, misère eligibility, defender scoring and "must bid to win" differ by region and house ([Pagat](https://www.pagat.com/euchre/500.html)). Keep the rules data-driven, with a test matrix per preset.
12. **Premature deep RL.** RL bidding without supervised pre-training underperformed, or learned to pass ([Kita et al.](https://arxiv.org/abs/2406.10306); [Bethe](https://cs.nyu.edu/media/publications/bethe_pm_thesis_final.pdf)). Use learning to approximate what the search and simulation already know, then improve.
13. **Non-reproducible analysis.** Seed every shuffle and every search, and store seeds with hand histories so reviews, bug reports and puzzles replay exactly.

### 5.7 Suggested build order (recommendation)

1. **Rules engine.** Variant configuration (Australian 4-player default plus American single-round, misère and Joker toggles), seeded dealing, scoring, hand-history format, golden tests.
2. **Lab harness.** Node `worker_threads`, duplicate scheduler, metrics, SPRT, ratings, plus random and rule bots (levels 1–2).
3. **500 solver and benchmarks.** Then PIMC with the exact hard-constraint sampler (levels 3–4) and a first coach hint and EV table.
4. **Bidding.** Hand+kitty evaluator trained on lab labels; simulation bidding; misère evaluator; score-state utility table from self-play.
5. **Inference and polish.** Bid and play likelihoods via the bots' own policies with softmax noise; plausibility and information tie-breaks; level 5; partner-behavior regression suite.
6. **Coach v1.** Graded checker, review timeline, luck/skill split, fairness page, belief heatmap, retry.
7. **Calibration.** Personality calibration, human playtests, the "flag this play" pipeline.
8. **Later.** Puzzles, daily duplicate deal, skill profile, learned components, endgame αμ/EPIMC experiments, WebAssembly solver port if profiling demands.

---

## 6. Sources

**PIMC, ISMCTS, inference, search**

- Long, Sturtevant, Buro, Furtak (2010). Understanding the Success of PIMC Sampling in Game Tree Search. AAAI. https://ojs.aaai.org/index.php/AAAI/article/view/7562 (PDF: https://cdn.aaai.org/ojs/7562/7562-13-11092-1-2-20201228.pdf)
- Ginsberg (2001). GIB: Imperfect Information in a Computationally Challenging Game. JAIR 14. https://jair.org/index.php/jair/article/view/10279 ; https://arxiv.org/abs/1106.0669
- Ginsberg (2000). Monte Carlo Methods. Dr. Dobb's Journal. https://jacobfilipp.com/DrDobbs/articles/DDJ/2000/0004/0004c/0004c.htm
- Frank & Basin (1998). Search in games with incomplete information: a case study using bridge card play (via the Edinburgh listing). https://www.dai.ed.ac.uk/daidb/papers/documents/rp575.html
- Buro, Long, Furtak, Sturtevant (2009). Improving State Evaluation, Inference, and Search in Trick-Based Card Games (Kermit). IJCAI. https://cs.du.edu/~sturtevant/papers/skat.pdf ; https://www.ijcai.org/Abstract/09/236
- Solinas, Rebstock, Buro (2019). Improving Search with Supervised Learning in Trick-Based Card Games. https://arxiv.org/abs/1903.09604
- Rebstock, Solinas, Buro, Sturtevant (2019). Policy Based Inference in Trick-Taking Card Games. https://arxiv.org/abs/1905.10911
- Rebstock et al. (2019). Learning Policies from Human Data for Skat. https://arxiv.org/abs/1905.10907
- Rebstock, Solinas, Sturtevant, Buro (2024). Transformer Based Planning in the Observation Space with Applications to Trick Taking Card Games. https://arxiv.org/abs/2404.13150
- Long & Buro (2011). Real-Time Opponent Modeling in Trick-Taking Card Games. IJCAI. https://www.ijcai.org/Proceedings/11/Papers/110.pdf
- Furtak (2013). Symmetries and Search in Trick-Taking Card Games (PhD thesis; IIMC). https://era.library.ualberta.ca/items/df9f1055-df09-43ed-910b-38da4e9d0792
- Wisser (2015). An Expert-Level Card Playing Agent Based on a Variant of PIMC Sampling. IJCAI. https://www.ijcai.org/Proceedings/15/Papers/025.pdf
- Cazenave & Ventos (2019). The αμ Search Algorithm for the Game of Bridge. https://arxiv.org/abs/1911.07960
- Cazenave et al. (2021). Optimizing αμ (notes recursive Monte Carlo's cost). https://arxiv.org/abs/2101.12639
- Arjonilla, Saffidine, Cazenave (2024). Perfect Information Monte Carlo with Postponing Reasoning. https://arxiv.org/abs/2408.02380
- Cowling, Powley, Whitehouse (2012). Information Set Monte Carlo Tree Search. IEEE TCIAIG. https://eprints.whiterose.ac.uk/75048/
- Whitehouse (2014). Monte Carlo Tree Search for Games with Hidden Information and Uncertainty (PhD thesis). https://etheses.whiterose.ac.uk/id/eprint/8117/
- Whitehouse, Cowling, Powley, Rollason (2013). Integrating MCTS with Knowledge-Based Methods to Create Engaging Play in a Commercial Mobile Game. AIIDE. https://ojs.aaai.org/index.php/AIIDE/article/view/12679
- Devlin et al. (2016). Combining Gameplay Data with MCTS to Emulate Human Play. AIIDE. https://ojs.aaai.org/index.php/AIIDE/article/view/12858
- Baier et al. (2018). Emulating Human Play in a Leading Mobile Card Game. IEEE ToG. https://repository.falmouth.ac.uk/2872/
- Sievers & Helmert (2015). A Doppelkopf Player Based on UCT. KI. https://ai.dmi.unibas.ch/papers/sievers-helmert-ki2015.pdf
- Bethe (2021). Advances in Computer Bridge (PhD thesis, NYU). https://cs.nyu.edu/media/publications/bethe_pm_thesis_final.pdf
- Celli & Gatti (2018). Computational Results for Extensive-Form Adversarial Team Games. https://arxiv.org/abs/1711.06930

**Neural and game-theoretic systems**

- Ben bridge engine. https://github.com/lorserker/ben ; bidding training notes: https://github.com/lorserker/ben/tree/main/scripts/training/bidding
- About Ben on BBO. https://news.bridgebase.com/about-ben-on-bbo/ ; BBO news on Ben: https://news.bridgebase.com/?p=30111
- Lorand Dali on the new GIBBO. https://news.bridgebase.com/?p=87069
- Lorand Dali comments on Ben's sampling (Bridge Winners). https://bridgewinners.com/bwcomments/user/lorand-dali/?page=2
- Bridge AI: How Neural Networks Learn to Bid (Bridge Winners). https://bridgewinners.com/article/view/bridge-ai-how-neural-networks-learn-to-bid/
- IntoBridge review (Bridge Winners). https://bridgewinners.com/article/view/intobridge-review/
- Kita, Koyamada, Yamaguchi, Ishii (2024). A Simple, Solid, and Reproducible Baseline for Bridge Bidding AI. https://arxiv.org/abs/2406.10306
- NooK coverage: https://indiaai.gov.in/article/hybrid-ai-beats-eight-world-champions-at-bridge ; critique: https://www.greaterwrong.com/posts/yHxmJch8dJoH6dwwz/so-has-ai-conquered-bridge
- Sturtevant & White. Feature Construction for Reinforcement Learning in Hearts. https://webdocs.cs.ualberta.ca/~nathanst/papers/heartslearning.pdf
- Niklaus et al. (2020). Challenging Human Supremacy: MCTS and Deep Learning for Jass. https://zenodo.org/records/5769551 ; survey: https://arxiv.org/abs/1906.04439 ; University of Bern thesis topic page: https://www.digitale-nachhaltigkeit.unibe.ch/studies/bachelor_s__and_master_s_theses_at_inf/game_ais/can_the_best_jass_ais_beat_the_top_humans/index_eng.html
- Pugh (2020). Learning to Play Euchre With Model-Free RL. https://web.stanford.edu/class/aa228/reports/2020/final165.pdf
- Zha et al. (2021). DouZero. https://arxiv.org/abs/2106.06135
- Brown, Bakhtin, Lerer, Gong (2020). ReBeL. https://arxiv.org/abs/2007.13544
- Schmid et al. (2023). Student of Games. https://arxiv.org/abs/2112.03178
- Pluribus training cost (InfoQ). https://www.infoq.com/news/2019/07/AI-beat-poker-facebook-cmu
- Blüml, Czech, Kersting (2023). AlphaZe∗∗. https://doi.org/10.3389/frai.2023.1014561
- OpenSpiel available games. https://github.com/google-deepmind/open_spiel/blob/master/docs/games.md
- ONNX Runtime Web. https://www.npmjs.com/package/onnxruntime-web

**Double-dummy solvers and web deployment**

- DDS (Haglund & Hein; DDS3). https://github.com/dds-bridge/dds ; docs: https://github.com/dds-bridge/dds/tree/main/doc ; move ordering: https://raw.githubusercontent.com/dds-bridge/dds/main/doc/heuristic-sorting.md ; WebAssembly build: https://github.com/dds-bridge/dds/tree/main/web
- macroxue bridge-solver (benchmarks). https://github.com/macroxue/bridge-solver
- bridge-craftwork bridge-solver (native vs WebAssembly timings). https://github.com/bridge-craftwork/bridge-solver
- @bridge-tools/dd (DDS in WebAssembly, npm). https://www.npmjs.com/package/@bridge-tools/dd
- Graham Hazel (2023). Double-Dummy Analysis in a Web Browser. https://grahamhazel.com/blog/2023/02/06/double-dummy-analysis-in-a-web-browser/
- Bridge Solver Online. https://mirgo2.co.uk/bridgesolver/
- Lead Solver. https://lajollabridge.com/Software/Lead-Solver/Lead-Solver-About.htm
- koejon issue #86 (TypeScript PIMC with exact solver). https://github.com/Winandvdb/koejon/issues/86
- Wikipedia: Computer bridge. https://en.wikipedia.org/wiki/Computer_bridge
- MDN: SharedArrayBuffer security requirements. https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/SharedArrayBuffer

**Difficulty, personalities, evaluation**

- Stockfish UCI options (Skill Level, UCI_Elo). https://github.com/official-stockfish/Stockfish/wiki/UCI-&-Commands
- McIlroy-Young et al. (2020). Maia. https://arxiv.org/abs/2006.01855 ; https://maiachess.com/blog/introducing_maia
- Quantal response equilibrium. https://en.wikipedia.org/wiki/Quantal_response_equilibrium
- Holmgård et al. Automated Playtesting with Procedural Personas through MCTS with Evolved Heuristics. https://arxiv.org/abs/1802.06881
- Demediuk et al. (2017). MCTS-based algorithms for dynamic difficulty adjustment. https://opus.lib.uts.edu.au/handle/10453/127485
- The Value of Recall in Extensive-Form Games. https://arxiv.org/abs/2412.19659
- Burch et al. (2018). AIVAT. https://arxiv.org/abs/1612.06915
- Zinkevich & Littman. The AAAI Computer Poker Competition (duplicate matches). https://www.martin.zinkevich.org/publications/competition.pdf
- Sequential Probability Ratio Test (Chessprogramming wiki). https://www.chessprogramming.org/Sequential_Probability_Ratio_Test ; Stockfish 11 notes: https://stockfishchess.org/blog/2020/stockfish-11/
- Herbrich, Minka, Graepel (2006). TrueSkill. https://www.microsoft.com/en-us/research/publication/trueskilltm-a-bayesian-skill-rating-system/

**Coaching and explanation references**

- lichess accuracy. https://lichess.org/page/accuracy ; thresholds discussion: https://lichess.org/forum/general-chess-discussion/what-constitutes-an-inaccuracy ; Learn from your mistakes: https://lichess.org/blog/WFvLpiQAACMA8e9D/learn-from-your-mistakes ; puzzle database: https://database.lichess.org/ ; puzzle generator: https://github.com/ornicar/lichess-puzzler
- chess.com move classification. https://support.chess.com/en/articles/8572705-how-are-moves-classified-what-is-a-blunder-or-brilliant-move ; Game Review: https://support.chess.com/en/articles/8584089-how-does-game-review-work
- GTO Wizard. https://help.gtowizard.com/tips-and-tricks-to-study-analyzed-hands/ ; https://help.gtowizard.com/analyze-mode-guide/
- PokerSnowie review (Pokerfuse). https://pokerfuse.com/features/reviews/review-poker-snowie-promises-perfect-gameplay-training-with-new-gto-based-learning-software-01-10/
- eXtreme Gammon (PR). https://en.wikipedia.org/wiki/EXtreme_Gammon ; GNU Backgammon luck analysis: https://www.gnu.org/software/gnubg/manual/html_node/Luck-analysis.html
- Funbridge analysis features. https://funbridge.com/blog/?p=31217 ; challenge vs Argine: https://funbridge.com/blog/?p=12678
- BBO robot tournaments. https://online.bridgebase.com/v2/doc_about_robot_tourneys.html
- NeuralPlay Bridge. https://www.neuralplay.com/bridge.html
- Bridge Baron (App Store). https://apps.apple.com/us/app/-/id1465176932
- Jack 6 (retail listing). https://chess.co.uk/products/jack-6-bridge-program-for-players-at-any-level-pc-download-version
- Bridge Master description (2019). https://site.pianola.net/sheffield/news/20190413
- ACBL Learn to Play Bridge discussion (BBO forums). https://www.bridgebase.com/forums/topic/62906-acbl-software-learn-to-play-bridge/
- Whisthub: "The Kingen AI has been cheating". https://www.whisthub.com/blog/the-kingen-ai-has-been-cheating
- Cribbage JD fair shuffle page (fetch blocked; cited from search summary). https://cardsjd.com/cribbage/fair-shuffle/

**500 rules, strategy and products**

- Pagat: Five Hundred. https://www.pagat.com/euchre/500.html
- Wikipedia: 500 (card game). https://en.wikipedia.org/wiki/500_(card_game)
- Bryce Francis, A better way to play 500. https://play500.com/calling1.html ; https://play500.com/gameof500.html
- NeuralPlay Five Hundred: https://www.neuralplay.com/fivehundred.html ; App Store US: https://apps.apple.com/us/app/five-hundred-500-expert-ai/id1559298243 ; App Store AU: https://apps.apple.com/au/app/five-hundred-500-expert-ai/id1559298243 ; Google Play: https://play.google.com/store/apps/details?id=com.neuralplay.android.fivehundred ; APKCombo: https://apkcombo.com/five-hundred-500-expert-ai/com.neuralplay.android.fivehundred/ ; Chrome-Stats: https://chrome-stats.com/d/id1559298243 ; Google publisher story: https://www.google.ca/ads/publisher/stories/neuralplay
- rotundtapir 500: https://f-droid.org/en/packages/io.github.rotundtapir.fivehundred/ ; https://github.com/rotundtapir/500 ; https://rotundtapir.github.io/500/
- playboardgames.org 500. https://playboardgames.org/play/500
- PiecePals Five Hundred. https://piecepals.com/play/five-hundred
- Trickster 500. https://www.trickstercards.com/games/500/ ; Trickster app: https://apps.apple.com/app/id982267355
- Board Game Arena: 500. https://en.boardgamearena.com/gamepanel?game=fivehundred
- Ultimate 500: https://apps.apple.com/app/id641417505 ; Lite: https://apps.apple.com/app/id956406859
- Extreme 500. https://apps.apple.com/us/app/-/id775846912
- 500 Card Game (OEngines): https://apps.apple.com/us/app/-/id6480395820 ; https://play.google.com/store/apps/details?id=com.oengines.online500cards
- Five Hundred (500) (tencardgame): https://play.google.com/store/apps/details?id=com.tencardgame.ten_five_hundred ; https://www.appbrain.com/app/five-hundred-500/com.tencardgame.ten_five_hundred
- Five Hundred – Card Games (Topy Games). https://play.google.com/store/apps/details?id=fivehundred.free.online.offline.classic.card.game
- 500+plus. https://apps.apple.com/us/app/-/id1521924208
- Five Hundred (Oliver Giles). https://fivehundred.sourceforge.net/
- Other web/Android listings: https://www.cardgames.club/five-hundred/ ; https://500cardgame.wtf/ ; https://play.google.com/store/apps/details?id=com.atakala.fivehundred&hl=en ; https://play.google.com/store/apps/details?id=com.rhhtechnology.fivehundred&hl=en_US ; https://play.google.com/store/apps/details?id=com.five2.play.fivehundred&hl=en_AU
