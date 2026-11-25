# 500

The card game 500 (Five Hundred), with a coach that teaches you to play it well. You and a bot partner play two bot
opponents in the browser, or friends join from their phones over home wifi. Under the hood are a rules engine,
AI players of several styles, and a strategy lab that plays millions of hands to work out what actually wins.

It plays the **American rules**: one round of bidding, no misère (nullo), and the defenders can win. Partners signal
each other in the auction by *mentioning* a suit or calling *6* to show the Joker. The full ruleset is in
[docs/rules/house-rules.md](docs/rules/house-rules.md).

## What's in it

- **Solo play** against eight AI players, from a forgetful Rookie and a reckless Gambler to experts: The Counter
  (solves dozens of deals the hidden cards could make), The Oracle (bids with a neural network) and The Natural (a network that
  learned card play).
- **A coach** that gives hints with reasons, grades every bid, kitty and card you play, and lets you undo a mistake.
  Each hand has a review: trick by trick, your graded decisions, and a replay from any point.
- **Learn:** lessons on bidding, play and counting (their numbers come from the lab), a bidding trainer, play
  puzzles, full-hand puzzles against The Oracle, your own past mistakes to replay, a counting quiz, records and
  achievements.
- **Tournaments:** 12 players at 3 tables over 8 rounds, with a new partner every round, a final reveal and a recap of
  how you played.
- **Story mode:** *The Kitty*, a ten-chapter crime story about an underground 500 tournament, told in cutscenes
  around games and tournaments.
- **Wifi play:** one computer hosts and everyone else joins from a phone or laptop on the same wifi.
  See [docs/playing-over-wifi.md](docs/playing-over-wifi.md).
- **Works offline,** and can be added to an iPhone's Home Screen.

## Play it (Windows)

You need [Rust](https://rustup.rs) and [Node.js](https://nodejs.org).

```
play.cmd
```

Double-click `play.cmd` (or run `play` in a terminal in this folder). It builds whatever has changed, starts the
game and opens it in your browser. The first run takes a few minutes; after that it starts in seconds. Friends on the
same wifi can join using the address and QR code it shows.

Two other scripts:

- **`make-app.cmd`** builds `app/500.exe`: the whole game in one file. Copy it to any Windows PC and double-click it
  there; nothing else needs installing.
- **`publish.cmd`** uploads the web app to Cloudflare Pages (a free account), so it can be opened on an iPhone and
  added to the Home Screen. The first run asks you to sign in.

## How it's built

| Part | What it is |
|---|---|
| `crates/core` | The rules engine: dealing, the auction, play and scoring, with no dependencies. Its tests cite the rule IDs in the house rules. |
| `crates/ai` | The AI players and the coach: a double-dummy solver, sampling of hidden cards, the search player, rule-based players, the neural networks, and the coach's grading and explanations. |
| `crates/wasm` | The engine compiled to WebAssembly for the browser. |
| `crates/host` | `fh-host`: serves the app and runs the shared table for wifi play. |
| `crates/lab` | `fh-lab`: the command-line lab for simulations, experiments, training data and puzzle generation. |
| `web/` | The app: Vite, React and TypeScript. |
| `train/` | Python scripts that train the bidding and card-play networks. |

For development, in `web/`:

```
npm run dev      # build the WebAssembly engine and serve the app
npm run build    # production build
npm run e2e      # browser tests (Playwright), after a build
```

and from the root:

```
cargo test --workspace
cargo clippy --workspace --all-targets
cargo run --release -p fh-lab -- play                                   # play in the terminal
cargo run --release -p fh-lab -- sim --games 20000 --ns steady --ew gambler
cargo run --release -p fh-lab -- show --code CODE                       # a hand code from the app's Code button
```

## The lab

Little has been published about strategy for these rules, so most of the advice here was measured.
[docs/lab/README.md](docs/lab/README.md) records more than fifty experiments: what to bid with a given hand, when to
lead trumps, what the Joker is worth, whether a mention helps your partner, and how the AI players compare in
duplicate matches. The findings feed the coach and the lessons. The rules of thumb that came out of them are in
[docs/rules-of-thumb.md](docs/rules-of-thumb.md), and [PLAN.md](PLAN.md) has the roadmap.

## License

MIT: see [LICENSE](LICENSE).
