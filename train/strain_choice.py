"""When to bid no trumps, and when length matters more than strength.

Uses the bidding datasets (each row: one bidder's hand, labelled with double-dummy tricks for the
bidder's side in every strain). Results are on the real-play scale (half a trick below
double-dummy) and scored at the 7 level with the house values and defenders' 10 per trick.

    train/.venv/Scripts/python train/strain_choice.py [--data train/data/bid,train/data/bid_selfplay]
"""

import argparse
import json
import pathlib
import sys

import numpy as np

ROOT = pathlib.Path(__file__).resolve().parent.parent
BASE = {0: 0, 1: 10, 2: 20, 3: 31}
LOW = {0: 5, 1: 5, 2: 4, 3: 4}
SAME_COLOR = {0: 1, 1: 0, 2: 3, 3: 2}
SUIT = ["♠", "♣", "♦", "♥"]
VALUE7 = [140, 160, 180, 200, 220]


def idx(suit, rank):
    return BASE[suit] + rank - LOW[suit]


def suit_cards(x, suit):
    lo, n = BASE[suit], 14 - LOW[suit] + 1
    return x[:, lo:lo + n]


def load(prefixes):
    xs, ys = [], []
    for prefix in prefixes:
        p = pathlib.Path(prefix)
        meta = json.loads(p.with_suffix(".json").read_text())
        xs.append(np.fromfile(p.with_suffix(".x"), dtype="<f4").reshape(meta["rows"], meta["features"]))
        ys.append(np.fromfile(p.with_suffix(".y"), dtype=np.uint8).reshape(meta["rows"], meta["labels"]))
    return np.concatenate(xs).astype(float), np.concatenate(ys)


def expected_points(tricks, value, level=7):
    """Expected points for the bidder's side, half real play at t and half at t-1 (the play gap)."""
    def u(t):
        made = t >= level
        return np.where(made, value, -value) - 10 * (10 - t)
    return 0.5 * u(tricks) + 0.5 * u(tricks - 1)


def trumps_in(x, t):
    return suit_cards(x, t).sum(1) + x[:, idx(SAME_COLOR[t], 11)] + x[:, 42]


def table(title, rows, cols, cell):
    print(f"\n{title}\n")
    print("| | " + " | ".join(c for c, _ in cols) + " |")
    print("|---|" + "---|" * len(cols))
    for rname, rmask in rows:
        cells = []
        for _, cmask in cols:
            m = rmask & cmask
            cells.append(cell(m) if m.sum() >= 80 else "—")
        print(f"| {rname} | " + " | ".join(cells) + " |")


def main():
    sys.stdout.reconfigure(encoding="utf-8")
    ap = argparse.ArgumentParser()
    ap.add_argument("--data", default=f"{ROOT / 'train/data/bid'},{ROOT / 'train/data/bid_selfplay'}")
    args = ap.parse_args()
    x, y = load(args.data.split(","))
    ok = (y != 255).all(1)
    x, y = x[ok], y[ok].astype(float)
    n = len(x)
    print(f"{n} bidding hands (all five strains solved)")

    # The suit a player would choose: most trumps (counting the Joker and bowers), then most cards.
    trumps = np.stack([trumps_in(x, t) for t in range(4)], 1)
    natural = np.stack([suit_cards(x, t).sum(1) for t in range(4)], 1)
    choice = np.argmax(trumps * 100 + natural, 1)
    rows_i = np.arange(n)
    suit_tricks = y[rows_i, choice]
    suit_value = np.array(VALUE7)[choice]
    nt_tricks = y[:, 4]
    suit_ev = expected_points(suit_tricks, suit_value)
    nt_ev = expected_points(nt_tricks, 220)
    nt_better = nt_ev > suit_ev

    longest = natural.max(1)
    joker = x[:, 42] == 1
    aces = sum(x[:, idx(s, 14)] for s in range(4))
    kings = sum(x[:, idx(s, 13)] for s in range(4))
    stopped = [
        (x[:, idx(s, 14)] == 1) | ((x[:, idx(s, 13)] == 1) & (suit_cards(x, s).sum(1) >= 2)) | ((x[:, idx(s, 12)] == 1) & (suit_cards(x, s).sum(1) >= 3))
        for s in range(4)
    ]
    stoppers = sum(st.astype(int) for st in stopped)
    best_trumps = trumps[rows_i, choice]
    singleton_or_void = (natural.min(1) <= 1)

    def pct_nt(m):
        return f"{nt_better[m].mean():.0%} ({nt_tricks[m].mean() - suit_tricks[m].mean():+.1f})"

    print("\nCells: how often no trumps scores better than your longest trump suit at the 7 level,")
    print("and (in brackets) NT tricks minus suit tricks on average. '—' = too few hands.")

    table(
        "## By your longest suit and the Joker",
        [(f"Longest suit {L}", longest == L) for L in [3, 4, 5]] + [("Longest suit 6+", longest >= 6)],
        [("No Joker", ~joker), ("Joker", joker)],
        pct_nt,
    )
    table(
        "## By aces and stoppers (suits with A, Kx or Qxx)",
        [(f"{a} aces", aces == a) for a in [0, 1, 2]] + [("3+ aces", aces >= 3)],
        [(f"{s} stoppers", stoppers == s) for s in [1, 2, 3]] + [("4 stoppers", stoppers == 4)],
        pct_nt,
    )
    table(
        "## By shape and the Joker",
        [("No singleton or void", ~singleton_or_void), ("A singleton or void", singleton_or_void)],
        [("No Joker", ~joker), ("Joker", joker)],
        pct_nt,
    )

    # NT wins overall?
    print(f"\nOverall NT beats the longest suit on {nt_better.mean():.0%} of hands.")
    good_nt = joker & (aces >= 2) & (longest <= 4)
    print(f"Joker + 2 aces + no suit longer than 4: NT better on {nt_better[good_nt].mean():.0%} (n={good_nt.sum()}).")
    good_nt2 = (aces >= 3) & (stoppers >= 3) & ~singleton_or_void
    print(f"3+ aces, 3+ stoppers, no singleton: NT better on {nt_better[good_nt2].mean():.0%} (n={good_nt2.sum()}).")
    long_suit = longest >= 6
    print(f"A 6+ card suit: NT better on {nt_better[long_suit].mean():.0%} (n={long_suit.sum()}).")

    # Length vs strength in suit contracts: tricks by trump count and top honours.
    top3 = np.stack([x[:, 42] + x[:, idx(t, 11)] + x[:, idx(SAME_COLOR[t], 11)] for t in range(4)], 1)
    all_rows = np.repeat(np.arange(n), 4)
    all_t = np.tile(np.arange(4), n)
    t_count = trumps[all_rows, all_t]
    t_top = top3[all_rows, all_t]
    t_tricks = y[all_rows, all_t] - 0.5
    print("\n## Suit contracts: average tricks for your side, by trumps held and top trumps (Joker, right and left bower)\n")
    print("| Trumps | none of the top 3 | 1 | 2 | all 3 |")
    print("|---|---|---|---|---|")
    for k in range(3, 9):
        cells = []
        for h in range(4):
            m = (t_count == k) & (t_top == h)
            cells.append(f"{t_tricks[m].mean():.1f}" if m.sum() >= 80 else "—")
        print(f"| {k} | " + " | ".join(cells) + " |")

    # NT: long suit length vs whether it's headed by A / A K.
    ls = np.argmax(natural, 1)
    head_a = x[rows_i, [idx(s, 14) for s in ls]] == 1
    head_k = x[rows_i, [idx(s, 13) for s in ls]] == 1
    nt_real = nt_tricks - 0.5
    print("\n## No trumps: average tricks, by your longest suit's length and what heads it\n")
    print("| Longest suit | no ace | ace, no king | ace and king |")
    print("|---|---|---|---|")
    for L in [4, 5, 6, 7]:
        sel = (longest == L) if L < 7 else (longest >= 7)
        cells = []
        for m in [sel & ~head_a, sel & head_a & ~head_k, sel & head_a & head_k]:
            cells.append(f"{nt_real[m].mean():.1f}" if m.sum() >= 80 else "—")
        print(f"| {L}{'+' if L == 7 else ''} | " + " | ".join(cells) + " |")

    # Extra trump length vs an extra honour, in tricks.
    print("\n## Suit contracts: what one more card is worth, by what it is (from the table above's slopes)")
    for h in range(3):
        a = t_tricks[(t_count == 5) & (t_top == h)].mean()
        b = t_tricks[(t_count == 6) & (t_top == h)].mean()
        c = t_tricks[(t_count == 5) & (t_top == h + 1)].mean()
        print(f"  holding 5 trumps with {h} of the top 3: one more small trump {b - a:+.2f}, swapping a small trump for a top one {c - a:+.2f}")


if __name__ == "__main__":
    main()
