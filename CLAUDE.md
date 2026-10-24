# 500 card game & trainer

- `PLAN.md` is the roadmap; `docs/rules/house-rules.md` is the authoritative rules spec (rule IDs H1…, cited by tests).
- Rust workspace: `crates/core` (rules engine, dependency-free), `crates/ai` (bots and personas; `solver.rs` double-dummy solver, `belief.rs` hidden-card
  sampling, `search_bot.rs` PIMC expert, `coach.rs` grading and explanations), `crates/lab` (CLI),
  `crates/wasm` (browser bindings: game logic in `table.rs`/`view.rs`, natively testable),
  `crates/host` (`fh-host`, wifi play: serves `web/dist`, one table in `room.rs`, WebSocket JSON protocol).
- Views are built for a viewer seat (`view::Lens`): seat 0 is always the viewer and team 0 their side. Solo play is
  South's lens; the host builds one per person. The host never sends a save (it would reveal every hand).
- `web/`: Vite + React + TypeScript app. `web/src/engine/types.ts` mirrors `crates/wasm/src/view.rs` — keep them in sync.
- Bots and hints must only use `Game::observe(seat)`, never `Game::hand` of another seat. The coach's advice must
  also never depend on who the other players are (their personas): it reads every player's calls and cards the
  same way. Tests: `advice_depends_only_on_what_you_can_see` (fh-ai), `advice_ignores_who_the_other_players_are` (fh-wasm),
  and for every bot persona `bots_decide_only_from_what_they_can_see` (fh-ai tests/behavior.rs).
- Say "bid" to players, never "contract" (the user's table's word); code may keep `contract`.
- The table must never make the page scroll (`web/e2e/layout.spec.ts`): panels go in the side rail
  (wide screens) or the dock above the middle, and only a panel itself may scroll.

## Commands

- `cargo test --workspace` — all Rust tests; `cargo clippy --workspace --all-targets` should stay warning-free.
- `cargo run --release -p fh-lab -- play` — play in the terminal (add `--watch` to let a bot play your seat).
- `cargo run --release -p fh-lab -- sim --games 20000 --ns steady --ew gambler` — bot-vs-bot statistics.
- `fh-lab duel --a counter --b steady --deals 200` (duplicate match), `fh-lab calibrate`, `fh-lab bench` — see `docs/lab/README.md`,
  where lab findings are recorded.
- Search bots budget work in solver nodes, not time (no clock in wasm; keeps runs reproducible).
- Long lab runs (hours) go in `scripts/lab-queue.sh` (`bash scripts/lab-queue.sh`): one after another, each saved as it
  finishes, so after a restart rerunning it skips what's done. Datasets are generated in chunks for the same reason.
- `fh-lab coach --seed N --south rookie` prints the coach's grades and reasons for a hand: use it to check explanations.
- The coach's card-play analysis takes a closer look (up to 64 more deals) when its best play isn't clearly ahead of
  the next different play, and reports plays it still can't separate as close calls (`Analysis::close`), so a near
  tie never reads as a firm answer. Its significance test weights deals as the averages do (effective sample size).
- Near the end of a game to 500 (either side within 160 of it) the coach plays for the game
  (`plays_for_the_game`, `Units::WinChance`): values are tenths of a percent of the chance to win, and grades
  carry `units: "win"`. Four-hand games are always played for points (the user's rule: no race to 500, so no
  sacrifices). Displayed make chances go through the measured real-play curves (`real_play_chance`).
- Neural bidding: `fh-lab dataset` writes training data (inputs defined in `crates/ai/src/features.rs`), then
  `train/.venv/Scripts/python train/train_bid.py` writes `crates/ai/models/bid.fhnn`, embedded at build time.
  `train/rules_of_thumb.py` measures the counting rules in `docs/rules-of-thumb.md`.
- Neural card play: `fh-lab playdata` (the Oracle's search values for every legal card, inputs in
  `features::play_features`, cards in role-based slots `features::play_slot`), then
  `train/.venv/Scripts/python train/train_play.py` writes `crates/ai/models/play.fhnn`; `neural_play.rs` plays with it
  (persona id `neural`). Changing the inputs means regenerating the data and retraining.
- Records and bid stats (`web/src/records.ts`): high game, best hand, highest bid made, and each bid you declared;
  counted once per game and hand, shown on Learn → Progress, with a popover banner when you set one.
- Hand codes (`web/src/handcode.ts`): the top bar's Code button (and Code in reviews and decision lists) names an
  exact moment of a solo game. With a host the code is short (`XXXX-XXXX`) and the host saves the game in
  `%LOCALAPPDATA%/500/hands/` (`POST /api/hands`); without one it's `500:` plus the game. **When the user gives a
  code, run `cargo run --release -p fh-lab -- show --code CODE`**: the deal, the auction and play, and the coach's
  view of that moment.
- Replay (solo): from a hand's review ("Replay this hand", per decision, per trick) the play engine loads the save
  cut at that action (`saveAt`); nothing in a replay is saved, graded or counted, and "Back to the game" reloads
  the real save. `HandView.firstAction` (from `Table::notes`) says where each deal starts in the log.
- "Finish hand" (solo, card play) lets The Natural play your cards at a tenth of the pace (`Table::auto_act`,
  worker op `autoAct`); those plays aren't graded. The top bar's speed button cycles slow/normal/fast.
- Persona "any" picks a random persona when the seat is filled (`resolve_persona` in `table.rs`); the save
  stores the pick. Short games: `Rules::hand_limit` (H25), stored in the save as `hands`.
- Lessons live in `web/src/learn/lessons.ts`; their numbers come from `docs/rules-of-thumb.md`, so keep them in step.
  `docs/bidding-count.md` lists every value in the coach's count (`crates/ai/src/thumb.rs`): keep it in step too.
  Drills (bidding trainer, play puzzles) are generated by `crates/wasm/src/coach.rs` in the coach worker.
- Bots follow the house discard signal (first throw-off from the strongest side suit; `crates/ai/src/signals.rs`).
- Offline: `web/public/sw.js` precaches the build (from `dist/precache.json`, written by `vite.config.ts`; Vite's
  manifest misses the worker's WebAssembly file); bump its `CACHE` name when
  changing caching behaviour. It only registers in production builds.
- The web app runs two engine workers: one plays, one coaches (rebuilding positions from the save), so grading never delays play.
- In `web/`: `npm run dev` (builds the WASM engine, then serves the app), `npm run build`,
  `npm run e2e` (Playwright, after `npm run build`; screenshots land in `web/test-results/`; the `lan` project
  starts `fh-host` on port 5599).
- `make-app.cmd` (→ `scripts/make-app.ps1`) builds `app/500.exe`: fh-host with `--features embed` (web/dist built
  in via `include_dir`, icon from `crates/host/assets/500.ico`), in `target/app` so it doesn't clash with play.cmd's
  build. That exe keeps its table in `%LOCALAPPDATA%/500/table.json`; solo games live in the browser's storage.
- `play.cmd` (→ `scripts/play.ps1`) is the one-step start: rebuilds `web/dist` if any web or engine source is newer,
  builds and runs `fh-host` (which opens the browser; `--no-browser` to stop it). Keep its source list in step with what the web build depends on.
- Wifi play: `cargo run --release -p fh-host` after `npm run build`. The app detects a host via `/api/info` and
  switches to `web/src/net.ts` (lobby, remote actions and coach); `web/src/components/Lobby.tsx` is the seat picker.
  Messages are in `crates/host/src/room.rs` (`ClientMessage`, `LobbyView`, `Snapshot`) — keep `net.ts` in step.
