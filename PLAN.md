# 500 — Game & Trainer: Project Plan

*Status: Phases 0–6 done (rules engine, bots, simulator, web game, expert AI and lab, coach, neural bidding, curriculum) · 2026-10-18*

## 1. Goals

1. **A complete, polished game of 500 (Five Hundred)** played in the browser: you and an AI partner against two AI opponents, using the American rules your table plays.
2. **A trainer that takes a casual player to expert level.** It gives hints that explain *why*, flags mistakes as you make them, reviews each hand afterwards, and teaches through lessons and drills.
3. **AI players with distinct, recognizable play styles**, from a forgetful beginner to an expert, plus a neural network that taught itself through self-play.
4. **A strategy lab that works out what actually wins.** It plays millions of simulated hands to test the advice in the strategy research and to calibrate bidding. It also trains the neural AI and mines it for lessons.

**Why the lab matters:** almost nothing has been published about strategy for these exact rules. Most written strategy assumes Australian rules (multi-round auction, misère, "you can't win on defense"). Your rules differ on all three, and they add calls that signal your partner (mentioning a suit, and 6 for the Joker). So the lab and the neural AI will derive much of the best play, and the lessons will be built from what they find.

## 2. Decisions so far

| Topic | Decision |
|---|---|
| Game | Trick-taking 500, **American rules** (section 3) |
| Platform | **Web app**, solo vs AI, installable and playable offline, no server |
| Architecture | **Rust core** shared by the game (WebAssembly), the lab (native) and training (Python module) |
| AI ambition | **Research + simulation lab + self-learning neural AI** |
| Your level | **Casual**: you know the rules and want to sharpen bidding, counting and defense |
| Your PC | i7-1065G7 (4 cores/8 threads), 16 GB RAM, no NVIDIA GPU. Rust 1.98 + VS Build Tools, Node 24 and Python 3.12 are installed and working |

Research collected in this session (with sources) lives in `docs/research/`:
- `01-rules-and-variants.md`: full rules spec, more than 40 variant toggles, edge cases, presets.
- `02-strategy-playbook.md`: expert technique, probability facts, 30 testable hypotheses.
- `03-ai-and-trainer-landscape.md`: AI methods, competing apps, coaching design.

## 3. Ruleset: "US Classic" (our default)

The engine supports every toggle in `01-rules-and-variants.md` §5, but the AIs are tuned and trained for this ruleset.

**Table and cards**
- 4 players in two partnerships (partners sit opposite each other).
- 43-card pack: ♥♦ 4–A, ♠♣ 5–A, plus one Joker. Each player gets 10 cards and 3 go to the kitty (the "widow").

**Card ranking**
- In a suit contract, trumps rank Joker > right bower (J of trumps) > left bower (J of the same color) > A K Q 10 9 … The left bower *is* a trump: it is not a card of its printed suit.
- In no trumps, the Joker is a suit of its own and the only trump.

**Bidding: once around, with partner signals**
- Bidding starts left of the dealer, and each player gets **exactly one** call. The dealer speaks last.
- The four kinds of call:

  | Call | Meaning | Counts as a bid? |
  |---|---|---|
  | **Pass** | — | — |
  | **Mention a suit** | "This is my strongest suit (typically I hold a bower), but I can't bid." | No |
  | **6 of a suit** | "I hold the Joker, and this is my best suit." | No, and it can never become the contract |
  | **Bid 7 or more** | A real contract in a suit or no trumps | Yes |

- **Mentions rank like bids: ♠ < ♣ < ♦ < ♥ < NT.** Each mention must outrank the previous one, so two players can never mention the same suit. For example, after a diamonds mention the next player may mention only hearts or NT.
- Whatever has been mentioned, a player may still call 6 of **any** suit or bid 7+.
- **Mentions are allowed only until someone calls 6 or bids 7+.** After that, each later player can only pass or bid higher. After a 6, the next real bid must be at least 7.
- A real bid must beat the previous real bid by Avondale value (7♠ = 140 … 10NT = 520).
- **No nullo.**

**When nobody bids 7+** (every call was a pass, mention or 6):
- The hand is thrown in and **the same dealer redeals**, up to twice.
- On the third deal, if no real bid has been made by the time it is the dealer's turn, **the dealer must bid** at least 7.

**Kitty and play**
- The winning bidder takes the kitty, discards 3 cards face down, and leads to the first trick.
- Players must follow suit by *effective* suit, which means bowers count as trumps.

**The Joker in no trumps**
- It may be played only when its holder is void in the suit led, and playing it is never compulsory.
- It may be led; the leader then names **any suit**, and the others must follow it.
- The contractor cannot give it a suit in advance.

**Scoring**

| Tricks | ♠ | ♣ | ♦ | ♥ | NT |
|---|---|---|---|---|---|
| 7 | 140 | 160 | 180 | 200 | 220 |
| 8 | 240 | 260 | 280 | 300 | 320 |
| 9 | 340 | 360 | 380 | 400 | 420 |
| 10 | 440 | 460 | 480 | 500 | 520 |

- A made contract scores its value, with no points for extra tricks. A failed contract loses its value.
- Mentions and 6-calls never score.
- Defenders score 10 per trick, always.
- **Slam:** winning all 10 tricks scores 250, or the bid's value if that is higher (8♣ and above).

**Winning**
- **First to 500 wins, defensive points included.**
- If both sides cross 500 on the same hand, the bidder's side counts first and wins.
- **There is no losing floor:** a team can go as far negative as it likes.

**My assumptions (tell me if any are wrong):**
- The engine allows any call, but all AIs use mentions and 6-calls honestly. Bluffing a signal would mislead your own partner too.
- If a mention or 6-call is the only call before everyone passes, it counts as an all-pass, so the hand is redealt.

## 4. Architecture

```
500/
├─ crates/                 Rust workspace: the single source of truth
│  ├─ core/                cards (bitboards), rules + variant config, game state machine,
│  │                       per-seat observations, NN input encoding, seeded RNG, game logs
│  ├─ ai/                  heuristic bots, personas, double-dummy solver, PIMC search,
│  │                       bidding/kitty evaluators, inference (belief over hidden cards)
│  ├─ lab/                 CLI: duplicate tournaments, experiments, training-data generation (all cores)
│  ├─ wasm/                wasm-bindgen API → npm package used by web/
│  └─ py/                  PyO3/maturin module → `import five_hundred` in train/
├─ web/                    Vite + React + TypeScript game & trainer (PWA)
│                          AI runs in Web Workers (WASM search + onnxruntime-web for neural nets)
├─ train/                  Python + PyTorch: distillation, self-play RL, evaluation, ONNX export, notebooks
└─ docs/                   research/, rules/, lab/ (experiment reports), lessons/ (curriculum content)
```

**Principles**
- **Bots can't cheat by construction.** Every AI and the hint engine receive only a per-seat observation; hidden cards never cross that boundary. Deals are seeded and reproducible, and the review screen can show exactly what each AI knew.
- **One implementation of the rules and of the NN inputs.** Python training and browser inference call the same Rust encoder, so they can never drift apart.
- **Deterministic and replayable.** Every game is a seed plus an action log. This lets us replay bugs, play the same deals twice to cancel out luck ("duplicate"), and build puzzles.
- **Contract-aware scoring.** Search maximizes expected *game points* or win probability, not tricks.

**Stack:** Rust (rand_chacha, serde, rayon, wasm-bindgen, pyo3, proptest, criterion, clap) · React + Zustand + Comlink + onnxruntime-web + vite-plugin-pwa + IndexedDB · Vitest + Playwright · Python (uv, PyTorch CPU, numpy, onnx, Jupyter).

**Tools to install:** `rustup target add wasm32-unknown-unknown`, `wasm-pack`, `maturin`, and `uv`. Put the project under git (`git init`).

## 5. The AI players

All bots share one engine and differ by **parameters**: bid aggression, partner and kitty credit, risk appetite, search budget, memory, inference, signaling honesty, score awareness and noise. Following the research, AI partners are always competent. Bad partners are the most common complaint about existing 500 apps.

| Persona | Style | Engine | What it teaches you |
|---|---|---|---|
| **Rookie** | Forgets cards, overvalues aces, bids by gut | Rule bot + noise | Warm-up, and spotting classic mistakes |
| **The Rock** | Bids only sure things, plays safe | Heuristic + light search | How to punish timid bidding |
| **The Gambler** | Bids a trick high, loves no trumps, swingy | Search, high risk appetite | When to let them overreach and set them |
| **The Counter** | Perfect memory, plays strictly by the odds | PIMC + inference | Counting and inference done right |
| **The Shark** | Expert, plus false cards, score pressure and late-game bidding to block you | PIMC + inference + deception | Not trusting signals; playing to the score |
| **The Oracle** | Taught itself by self-play; sometimes unconventional | Neural net (+ search) | The frontier, and the source of new lessons |

There is also a difficulty slider within each style, set by search budget and noise. You choose your partner as well as your opponents.

Every persona makes and reads mentions and 6-calls; the expert bots also use them to infer where the hidden cards are. One more persona is planned: a style built around your own rules of thumb, once you share them.

**Engine tiers**
1. **Rule bots:** hand-written heuristics. Fast and predictable.
2. **PIMC expert:** sample deals that fit what the seat knows, solve each with a purpose-built 500 double-dummy solver (alpha-beta, transposition table, equivalent-card pruning), and average the results. This is the approach behind the bridge program GIB and the Skat program Kermit. The research found that **inference** (weighting the samples by the bids and plays so far) and score-based evaluation matter more than fancier search.
   - *Built in Phase 3* (`crates/ai`).
   - Search wins at card play: +9.4 points per hand over Steady.
   - Simulated bidding with a handful of samples lost 27 points per hand, so the expert bids by a lab-tuned rule for now. A learned hand evaluator in Phase 5 should fix that.
   - Details in `docs/lab/README.md`.
3. **Neural:** see section 7.

## 6. The trainer (coaching features)

- **Hint button:** the best action, with 1–3 plain-language reasons drawn from the search statistics. Example: *"Lead the Joker: the opponents hold 4 trumps, and drawing two rounds leaves them one or fewer in 76% of deals."*
- **Live coach:** grades each decision as an inaccuracy, mistake or blunder by expected-point loss, with an optional take-back in practice mode. Decisions are graded only on what you could have known, never on hindsight.
- **Post-hand review:** a trick-by-trick replay with all hands shown, key moments flagged, the best possible result shown alongside the actual one, and a bidding review showing what your hand was really worth. It separates **luck from skill** by replaying your cards with the expert in your seat.
- **Training wheels for a casual player:** bowers highlighted and sorted into the trump suit, a trump counter showing how many trumps are still out, and a card tracker. Each can be switched off as you improve.
- **Lessons:** rules refresher → hand evaluation → single-round bidding and seat position → signaling with mentions and 6-calls (when to signal, and what you give away to the opponents) → kitty discards → declarer play → defense → no trumps and the Joker → counting and inference → playing to the score (where defenders can win). Each lesson has examples and a quiz, and its claims are checked in the lab.
- **Drills:** a bidding trainer (rate hands, get feedback, repeat the hand types you misjudge), play and defense puzzles, and "replay this deal".
- **Progress dashboard:** bidding accuracy, contracts made, average point loss per hand, mistakes by category, and your rating against each persona.

## 7. Strategy lab and neural AI

**The lab** is a Rust CLI that uses all CPU cores.
- **Duplicate tournaments:** each deal is played twice with the seats swapped, compared with paired statistics, and tracked on an Elo ladder. A new bot is accepted only when it beats the old one with statistical significance.
- **Experiments:** calibrate how many tricks a hand is worth with partner and kitty, and test the 30 hypotheses from the playbook.
- **New questions for our rules:**
  - How seat position matters when bidding goes around once (the dealer bids last).
  - How bidding and sacrificing change when defenders can go out.
  - What a mention or a 6-call is worth to your partner, compared with what it tells the opponents.
  - How you should bid when your partner has signaled.
  - What the dealer should do when forced to bid on the third deal.
- **Output:** findings are written up in `docs/lab/` and become lessons.

**Neural pipeline.** It is sized for a CPU-only laptop and starts small.

*As built in Phase 5:* the lab showed bidding is the expert's weak spot, so the network starts there.

1. **Data:** `fh-lab dataset` records each bidder's view and labels it with the solver's result for every strain. The inputs are defined once, in `crates/ai/src/features.rs`.
2. **Training:** `train/train_bid.py` (PyTorch, CPU) learns a trick distribution per strain.
3. **Running the model:** a small multilayer network runs in plain Rust (`crates/ai/src/nn.rs`) with the weights embedded. It behaves the same natively and in the browser, so ONNX isn't needed.
4. **The Oracle:** neural bidding plus The Counter's search play, tested in duplicate.
5. **Self-play refinement:** once the Oracle bids differently, partner's calls mean different things. So the data is regenerated with the Oracle bidding and the model retrained.
6. **Rules of thumb:** `train/rules_of_thumb.py` turns the same data into counting rules (`docs/rules-of-thumb.md`).

*Later steps:*

- **Distillation:** the PIMC expert labels millions of decisions, and small networks (about 1–5M parameters) learn to imitate it. This gives fast, decent policies cheaply.
- **Self-play reinforcement learning:** Deep Monte Carlo (the method behind the DouZero card-game AI) or PPO, starting from the distilled networks and evaluated by duplicate matches against the PIMC expert.
- **Hybrid play:** the neural network guides the search and supplies inference. This is likely the strongest player overall.
- **Lesson mining:** find where the network disagrees with conventional wisdom, confirm the difference in the lab, and teach it.

**Expectations**
- The research found that neural nets work well as *components* trained on our own lab data. Full self-play systems at research scale (ReBeL, Student of Games) need data-center compute.
- Success means the Oracle beats the PIMC expert in duplicate play. If it doesn't, the networks still pay off by speeding up search and improving inference.
- Larger runs can move to free Kaggle or Colab GPUs, or a rented GPU at roughly $0.25–1 per hour. That decision comes at the start of Phase 5.

## 8. Roadmap

Each phase ends with something usable.

| Phase | Deliverable | Done when |
|---|---|---|
| **0. Foundations** ✅ | git repo, Cargo workspace, rules spec in `docs/rules/house-rules.md`. The web and Python scaffolds move to Phases 2 and 5, when they're first needed | `cargo test` runs |
| **1. Rules engine** ✅ | Rust core for the house rules, plus the toggles we use (mentions, 6-calls, redeals, slam, defensive cap); a rule bot in three styles; terminal play; `fh-lab sim` | Every house rule has a test; 1M bot games (9.1M deals, 353M actions) with every invariant holding after every action. The full regional variant catalogue is deferred until someone needs it |
| **2. Playable web game (MVP)** ✅ | React table over the WASM engine (`crates/wasm`, `web/`); Rookie, Rock, Gambler and Steady; save and resume; score pad; hand review; bower badges. The rule bots are instant, so moving the AI to Web Workers waits for Phase 3 | Playwright plays complete hands at desktop and phone sizes (`npm run e2e`) |
| **3. Expert AI + lab** ✅ | Double-dummy solver, PIMC card play with inference, lab-tuned bidding, heuristic kitty discards (searched discards tested: no gain); Counter and Shark; `fh-lab duel`, `calibrate` and `bench`; engine in a Web Worker; findings in `docs/lab/README.md` | Expert beats the rule bots in duplicate (Counter +29 points per hand over Steady, 95% CI +16 to +42); every expert move ≤ 1.1 s in the browser on this laptop |
| **4. Coach** ✅ | Hint button with reasons; live grades (best → blunder by expected points, capped when the sampled deals cannot separate two plays) with Undo; review tabs for your decisions, trick-by-trick hindsight (all cards visible) and luck vs skill (replay with The Counter in your seat); trump counter and card tracker; coach in its own worker | Every decision in a hand can be graded and explained (`crates/ai/src/coach.rs`, `web/e2e/coach.spec.ts`) |
| **5. Neural AI** ✅ (first stage) | Neural bidding evaluator (`fh-lab dataset` → `train/train_bid.py` → `crates/ai/src/nn.rs`), one self-play refinement round, The Oracle (now the default partner), coach bids graded by the evaluator, `docs/rules-of-thumb.md` from solver data and expert habits. Play-policy distillation and self-play RL are left for a later stage | The Oracle beats The Counter: +12.4 points per hand on 400 duplicate deals, +8 to +12 across three matches |
| **6. Curriculum** ✅ | Learn screen: 9 lessons with quizzes (`web/src/learn/lessons.ts`), bidding trainer, play puzzles (positions with one clearly best play), progress dashboard; counting aids (who is out of which suit, cards out per suit) and an in-game counting quiz; the discard-signal convention in every bot; opening-lead analysis (`fh-lab leads`) behind the play tips | The full beginner-to-expert path is playable; `web/e2e/learn.spec.ts` covers lessons, drills, quiz and progress |
| **7. Polish & extras** | Animations and sound, accessibility, offline install, 3/5/6-player variants; online play optional | — |

Phase 5's data generation can run in the background while Phase 4 is built.

## 9. Quality and testing

- **Rules:** a unit test per rule and toggle, including every edge case in `01-rules-and-variants.md` §6 (left bower following, Joker leads in no trumps, simultaneous 500). Also property tests and golden game transcripts, with the WASM and Python bindings checked against native results.
- **AI:**
  - Zero illegal moves across millions of games.
  - Regression tournaments before any bot change ships.
  - Performance budgets per persona.
  - Deterministic tie-breaks, so equal plays never look random or silly.
- **Web:** Vitest for logic and Playwright playing complete games end to end.

## 10. Risks

| Risk | Mitigation |
|---|---|
| Rule ambiguity between tables | Every variant is a toggle; your table's rules are written down in Phase 0 |
| Search too slow in the browser or on phones | WASM + Web Workers, time budgets, fewer samples on slow devices |
| Neural training limited by CPU-only hardware | Small networks, distillation first, cloud GPU only if needed; the game never depends on the neural net |
| A search or neural AI can't explain itself | Reasons generated from search statistics and hand features; neural suggestions cross-checked by search |
| Scope creep | Each phase ships something usable; online play and other player counts are deferred |
| Users suspect the AI cheats | Seat views enforced by the engine, reproducible deals, and "what the AI knew" in the review |
