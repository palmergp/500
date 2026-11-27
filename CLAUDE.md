# 500 card game & trainer

- `PLAN.md` is the roadmap; `docs/rules/house-rules.md` is the authoritative rules spec (rule IDs H1…, cited by tests).
- Rust workspace: `crates/core` (rules engine, dependency-free), `crates/ai` (bots and personas; `solver.rs` double-dummy solver, `belief.rs` hidden-card
  sampling, `search_bot.rs` PIMC expert, `coach.rs` grading and explanations), `crates/lab` (CLI),
  `crates/wasm` (browser bindings: game logic in `table.rs`/`view.rs`, natively testable),
  `crates/host` (`fh-host`, wifi play: serves `web/dist`, one table in `room.rs`, WebSocket JSON protocol).
- Views are built for a viewer seat (`view::Lens`): seat 0 is always the viewer and team 0 their side. Solo play is
  South's lens; the host builds one per person. The host never sends a save (it would reveal every hand).
- Solo play from a browser that loaded the app from a host: the host's computer does the engine's work. Each client
  (play, coach, tournament simulator) sends the worker's requests over `/engine` (`crates/wasm/src/service.rs`, one
  thread per connection), resyncing the game from its last save on a new connection and falling back to its own
  worker if the host can't be reached (`web/src/engine/index.ts`; `?local` forces the worker). That's the browser's
  own game, so saves may travel there; the shared table's room still never sends one.
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
  Kitty grades do the same (`discard_comparison`): 12 deals, then 60 fresh ones when the gap is unclear or small;
  a gap the deals can't confirm is a close call (good) within `CLOSE_POINTS`, at most an inaccuracy beyond.
  Their deals fit the auction (partner's bid read in, unlike the search's card play): closer to the truth (finding 58,
  `fh-lab kittyworlds`).
- Winning (H23–H24, games to 500 only): declarers who make their bid and reach 500 win; defensive points never win
  (no cap, they just keep counting); −500 or worse loses. `game_winner(rules, scores, declarers, made)`.
- Near the end of a game to 500 (either side within 160 of 500 or of −500) the coach plays for the game
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
- Records and bid stats (`web/src/records.ts`): high game, best hand (with its bid and who declared it), highest bid made, each bid you declared,
  and tournaments (won, podiums, average place, highest total; counted when the final reveal is done);
  counted once per game and hand, shown on Learn → Progress, with a popover banner when you set one.
- Achievements (`web/src/achievements.ts`, Learn → Achievements): measured mostly from what's already kept (records, drill
  scores, lessons, solved mistakes), so past play counts; moments the records don't keep (slams, sets, streaks, comebacks,
  clean games) are noted as they happen. Anything that changes progress calls `checkAchievements()`, which announces new
  ones with a `fh500:achievements` window event (App shows the banner); the start-up check is quiet.
- Top bar: on small screens (`COMPACT` in App.tsx, where its labels hide) Code, Settings, sound and music go in its
  "More" menu (`components/MoreMenu.tsx`; e2e helper `topBarControl`); speed and the rest stay in the bar.
- Hand codes (`web/src/handcode.ts`): the top bar's Code button (and Code in reviews and decision lists) names an
  exact moment of a solo game. With a host the code is short (`XXXX-XXXX`) and the host saves the game in
  `%LOCALAPPDATA%/500/hands/` (`POST /api/hands`); without one it's `500:` plus the game. **When the user gives a
  code, run `cargo run --release -p fh-lab -- show --code CODE`**: the deal, the auction and play, and the coach's
  view of that moment.
- Replay (solo): from a hand's review ("Replay this hand", per decision, per trick) the play engine loads the save
  cut at that action (`saveAt`); nothing in a replay is saved, graded or counted, and "Back to the game" reloads
  the real save. `HandView.firstAction` (from `Table::notes`) says where each deal starts in the log.
- Tournament mode (`web/src/tournament.ts`, `components/Tournament.tsx`): 12 players (you and 11 bots with random
  names; personas by the kind chosen at the start: Random, Expert (`EXPERT_STYLES`) or Chaos (all The Gambler)), 3 tables, 8 rounds of a 4-hand game; a side's final score goes to both partners' totals, and
  partners never repeat (circle-method schedule). You play one table (names via `SaveData.names`); the other two are
  played out by `simulateGame` in a third worker (`Simulator`). Saved in local storage; standings after each round,
  then (straight after your last game, waiting for the other tables if need be) a staged reveal of 3rd, 2nd, 1st with the High Game and High Hand, and every player's score in every round
  (`GameGrid`: tinted cells, the round's best starred, each game's hands and bids in its tooltip, a player's games listed when chosen).
  After the reveal, Your recap (`components/TournamentRecap.tsx`, `recap()`; also from New tournament later): from
  `Tournament.mine`, your game each round (save, hands, grades). The coach grades whatever it didn't during play in
  the background (`gradeGame`, coach op `decisions`), a grade at a time, pausing while you play a game it grades live.
  `Tournament.assisted` turns true on a hint, an undo, Finish hand, or a helper setting on (`assistHelpers`) in one of
  its games; a win without it is the Unaided achievement. The round-results dialog can't be closed: its only way on is
  the next round. Quit tournament is offered any time before the results (`quitTournament` clears the saved game only
  if it's the tournament's), and the unaided note's Turn off all help switches every helper off (`turnOffHelp`; the
  story's chapter list has it too).
- Hand review with grading off: opening a finished hand's review grades your decisions in it after the fact
  (`gradeHandAfter`, quietly: no grade panel, not counted as help), leaving out plays Finish hand made (`autoActed`).
  Trick by trick (`dd_trace`) names the plays that would have kept par under any card that cost a trick (`better`).
- Story mode (`web/src/career/`, called career in the code): the story "The Kitty" lives in `story.ts` (characters with the persona that plays
  them, ten chapters: opening scene, level, win and lose scenes, lines after each hand); progress in `career.ts`
  (local storage). A level is a game against story characters (solo save, `careerGame` in App.tsx, "Continue the
  story" at its end) or a tournament with a story cast (`Tournament.career`, judged by place after the final reveal).
  Scenes play in `components/Cutscene.tsx` (a full-screen `<dialog>`; `{name}` is the player, who is never given
  pronouns); characters are drawn by `components/Portrait.tsx` from their `Look`; the chapter list is
  `components/Career.tsx`. Scene sound is `storyAudio.ts` (synthesized like `audio.ts`, whose music rests meanwhile):
  each place has a mood and ambience (`PLACE_SOUND`), steps `{ music }` and `{ sfx }` change them, title cards sting;
  the score follows `Settings.storyMusic`, the rest `Settings.sound`. Test: `e2e/career.spec.ts`.
  Finished chapters offer their opening and ending scenes again (`watchScene`). A chapter beaten with no help (no
  hint, undo or Finish hand, helpers off: `career.playing.assisted`, or a story tournament's `assisted`) is kept in
  `CareerState.unaided` and marked 🙌; achievements' Story group has one per chapter and Clean hands (all ten unaided).
- A finished solo game stays saved (a reload shows its result) but is marked over (`savedGame.over`), so the main
  menu doesn't offer Continue; the game-over result has Main menu and New game.
- "Finish hand" (solo, card play) lets The Natural play your cards at a tenth of the pace (`Table::auto_act`,
  worker op `autoAct`); those plays aren't graded. The top bar's speed button cycles slow/normal/fast.
- Persona "any" picks a random persona when the seat is filled (`resolve_persona` in `table.rs`); the save
  stores the pick. Short games: `Rules::hand_limit` (H25), stored in the save as `hands`.
- Lessons live in `web/src/learn/lessons.ts`; their numbers come from `docs/rules-of-thumb.md`, so keep them in step.
  `docs/bidding-count.md` lists every value in the coach's count (`crates/ai/src/thumb.rs`): keep it in step too.
  Drills (bidding trainer, play puzzles) are generated by `crates/wasm/src/coach.rs` in the coach worker.
- Full-hand puzzles (Learn → Full hands, `web/src/learn/handPuzzles.ts`): a hard bid of South's to play out from the
  kitty against The Oracle. `fh-lab handpuzzles` writes `web/src/learn/handPuzzles.json`, keeping a deal only if the
  solver makes the bid, the expert (own cards only) makes it on three runs, and the steady bot goes down. The bots are
  seeded by the deal, so a puzzle plays the same every time; **after changing how bots play, regenerate them** (the
  check was made against the old play). At the table they run in puzzle mode (`puzzleRef` in App.tsx): nothing saved
  or counted, grades kept until the hand is over, and Leave goes back to the game that was on the table.
- Counting quiz (`components/CountingQuiz.tsx`): trumps unseen, cards unseen in a suit, who's shown out, the
  highest card of a suit not yet played; from the third trick, never on the last, as often as `Settings.quizRate`.
- Bots follow the house discard signal (first throw-off from the strongest side suit; `crates/ai/src/signals.rs`).
- Of touching cards, bots (and the coach) play the boss while partner is still to play to the trick: the ace from
  A-K, so a short partner can read it (`cardplay::show_winner`, applied in every bot's card choice).
- Search bots and The Natural never lead the Joker as the declaring side in the first three tricks of a no-trump
  hand while holding other cards (`cardplay::early_joker_lead`, findings 54 and 56); the coach grades that lead as
  costing the bid's chance. From trick 4 the search decides (tricks 4–6 came out even, 7+ the Joker lead pays).
- Rule bots stop drawing trumps once both opponents have shown out (`cardplay::opponents_may_hold_trumps`).
- The rule bots' bidding estimate (Steady, Rock, Gambler, Rookie, and The Counter's and The Shark's bidding) counts a
  suit with fewer than four trumps 0.9 a trump short (`rule_bot::short_trumps`, finding 59, `fh-lab strainpick`).
- `fh-lab show --code C --at N` shows any seat's decision at action N (your own: the coach's grade and reasons; its
  bot, The Oracle's ranking, the solver's values; `--discards "6S 8S 4H"` grades a kitty on five seeds and 400
  deals); `fh-lab whatif --code C --at N --a X --b Y` plays two choices out over that seat's possible deals.
- Early in a suit bid (tricks 1–3), the top trump and a side ace lead come out even when played out (finding 55,
  `fh-lab acelead`): the coach grades either as good and offers the other in its hint (`cardplay::ace_or_top_trump`).
- Offline: `web/public/sw.js` precaches the build (from `dist/precache.json`, written by `vite.config.ts`; Vite's
  manifest misses the worker's WebAssembly file). The build appends its own hash to the `CACHE` name, so each release
  installs fresh; bump the `v2` part only when changing caching behaviour. It only registers in production builds
  (and only over HTTPS or on localhost, so not from a host's wifi address).
- iPhone: `publish.cmd` (→ `scripts/publish.ps1`) builds and uploads `web/dist` to Cloudflare Pages
  (`https://<project>.pages.dev`, wrangler pinned) for Add to Home Screen; `web/public/_headers` sets its caching.
  Full screen it runs below a solid black status bar (not black-translucent: iOS 26 blurs content drawn under
  it; changing the tag needs the app re-added to the Home Screen); the layout pads by the safe-area insets
  (`e2e/iphone.spec.ts`), and
  short screens (a phone on its side, `max-height: 500px`) put the other three players in a row across the top.
- Calls and bids render through `components/BidText.tsx` (♥ and ♦ red, the four-color deck's colors too; lighter on
  the felt). The bid being played is the gold-edged `.bid-tile` plate in `InfoPanel` (bid, whose, trumps, tricks along its foot);
  pressing it opens everyone's calls for that hand (narrow panels shrink its "Bids" label to an arrow, a container query).
  `Settings.pauseAfterBidding` (solo only, not puzzles or replays): the finished auction waits in the middle
  (`auctionPause` in App.tsx) until Continue, and the bots wait with it.
- Touch screens have no hover: `components/TouchTips.tsx` shows any `title` on press-and-hold (and on a tap for plain
  labels), so put extra information in `title`, never in hover-only handlers.
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
