# 500 card game & trainer

- `PLAN.md` is the roadmap; `docs/rules/house-rules.md` is the authoritative rules spec (rule IDs H1…, cited by tests).
- Rust workspace: `crates/core` (rules engine, dependency-free), `crates/ai` (bots and personas), `crates/lab` (CLI),
  `crates/wasm` (browser bindings: game logic in `table.rs`/`view.rs`, natively testable).
- `web/`: Vite + React + TypeScript app. `web/src/engine/types.ts` mirrors `crates/wasm/src/view.rs` — keep them in sync.
- Bots and hints must only use `Game::observe(seat)`, never `Game::hand` of another seat.

## Commands

- `cargo test --workspace` — all Rust tests; `cargo clippy --workspace --all-targets` should stay warning-free.
- `cargo run --release -p fh-lab -- play` — play in the terminal (add `--watch` to let a bot play your seat).
- `cargo run --release -p fh-lab -- sim --games 20000 --ns steady --ew gambler` — bot-vs-bot statistics.
- In `web/`: `npm run dev` (builds the WASM engine, then serves the app), `npm run build`,
  `npm run e2e` (Playwright, after `npm run build`; screenshots land in `web/test-results/`).
