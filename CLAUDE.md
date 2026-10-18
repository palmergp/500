# 500 card game & trainer

- `PLAN.md` is the roadmap; `docs/rules/house-rules.md` is the authoritative rules spec (rule IDs H1…, cited by tests).
- Rust workspace: `crates/core` (rules engine, dependency-free), `crates/ai` (bots), `crates/lab` (CLI).
- Bots and hints must only use `Game::observe(seat)`, never `Game::hand` of another seat.

## Commands

- `cargo test --workspace` — all tests; `cargo clippy --workspace --all-targets` should stay warning-free.
- `cargo run --release -p fh-lab -- play` — play in the terminal (add `--watch` to let a bot play your seat).
- `cargo run --release -p fh-lab -- sim --games 20000 --ns steady --ew bold` — bot-vs-bot statistics.
