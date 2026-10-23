"""Measure rules of thumb for bidding from the training data.

Fits, for suit contracts and for no trumps, a linear model of the tricks a bidder's side takes as
declarer (double-dummy, minus the measured half-trick play gap) on features a player can count at
the table. The coefficients read directly as "this is worth about N tricks". Then finds the count
at which bidding 7 and 8 starts to pay, from the house scoring and defenders' trick points.

    train/.venv/Scripts/python train/rules_of_thumb.py [--data train/data/bid]
"""

import argparse
import json
import pathlib
import sys

import numpy as np

ROOT = pathlib.Path(__file__).resolve().parent.parent
DD_BIAS = 0.5  # double-dummy runs about half a trick above real play (docs/lab/README.md, finding 2)

# Card indices follow the pack order in crates/ai/src/features.rs: spades 5-A, clubs 5-A,
# diamonds 4-A, hearts 4-A, Joker.
BASE = {0: 0, 1: 10, 2: 20, 3: 31}
LOW = {0: 5, 1: 5, 2: 4, 3: 4}
SUIT = ["♠", "♣", "♦", "♥"]
SAME_COLOR = {0: 1, 1: 0, 2: 3, 3: 2}
CALL = 32
POS, LEFT, PARTNER, RIGHT = 43, 47, 47 + CALL, 47 + 2 * CALL


def idx(suit, rank):
    return BASE[suit] + rank - LOW[suit]


def suit_cards(x, suit):
    lo, n = BASE[suit], 14 - LOW[suit] + 1
    return x[:, lo:lo + n]


def suit_features(x, t):
    """Features of each hand when suit t is trumps."""
    joker = x[:, 42]
    right = x[:, idx(t, 11)]
    left = x[:, idx(SAME_COLOR[t], 11)]
    ace, king, queen = (x[:, idx(t, r)] for r in (14, 13, 12))
    trumps = suit_cards(x, t).sum(1) + left + joker
    side = [s for s in range(4) if s != t]
    lengths = [suit_cards(x, s).sum(1) - (x[:, idx(s, 11)] if s == SAME_COLOR[t] else 0) for s in side]
    side_aces = sum(x[:, idx(s, 14)] for s in side)
    guarded_kings = sum(x[:, idx(s, 13)] * (lengths[i] >= 2) for i, s in enumerate(side))
    voids = sum((l == 0).astype(float) for l in lengths)
    singletons = sum((l == 1).astype(float) for l in lengths)
    small = trumps - joker - right - left - ace - king - queen
    return {
        "Joker": joker,
        "Right bower": right,
        "Left bower": left,
        "Ace of trumps": ace,
        "King of trumps": king,
        "Queen of trumps": queen,
        "Each other trump (10 or lower)": small,
        "Side ace": side_aces,
        "Guarded side king": guarded_kings,
        "Side void": voids,
        "Side singleton": singletons,
    }


def nt_features(x):
    aces = sum(x[:, idx(s, 14)] for s in range(4))
    kings = sum(x[:, idx(s, 13)] * (suit_cards(x, s).sum(1) >= 2) for s in range(4))
    queens = sum(x[:, idx(s, 12)] * (suit_cards(x, s).sum(1) >= 3) for s in range(4))
    unstopped = sum(
        ((x[:, idx(s, 14)] == 0) & ~((x[:, idx(s, 13)] == 1) & (suit_cards(x, s).sum(1) >= 2))).astype(float)
        for s in range(4)
    )
    long_cards = sum(np.maximum(suit_cards(x, s).sum(1) - 3, 0) * x[:, idx(s, 14)] for s in range(4))
    return {
        "Joker": x[:, 42],
        "Ace": aces,
        "Guarded king": kings,
        "Queen with 2+ guards": queens,
        "Unstopped suit": unstopped,
        "Long card in an ace-led suit": long_cards,
    }


def call_features(x, strain):
    """What the others said, relative to the strain being considered."""
    six = lambda block: x[:, block + 7:block + 12].sum(1)
    six_here = lambda block: x[:, block + 7 + strain]
    feats = {
        # A 6-call names the caller's best strain: in this one it says far more (finding 27).
        "Partner called 6 in this strain": six_here(PARTNER),
        "Partner called 6 in another strain": six(PARTNER) - six_here(PARTNER),
        # (In a suit bid, a 6-call in the left bower's suit counts for about twice another: finding 31.)
        "Partner mentioned this strain": x[:, PARTNER + 2 + strain],
        "Partner bid this strain": sum(x[:, PARTNER + 12 + lvl * 5 + strain] for lvl in range(4)),
        "Partner passed": x[:, PARTNER + 1],
        "An opponent called 6 in this strain": six_here(LEFT) + six_here(RIGHT),
        "An opponent called 6 in another strain": six(LEFT) + six(RIGHT) - six_here(LEFT) - six_here(RIGHT),
        "An opponent mentioned this strain": x[:, LEFT + 2 + strain] + x[:, RIGHT + 2 + strain],
        "An opponent bid this strain": sum(x[:, b + 12 + lvl * 5 + strain] for b in (LEFT, RIGHT) for lvl in range(4)),
        "Speaking last (dealer)": x[:, POS + 3],
    }
    return feats


def fit(features: dict, target: np.ndarray):
    names = list(features)
    a = np.column_stack([np.ones(len(target))] + [features[n] for n in names])
    coef, *_ = np.linalg.lstsq(a, target, rcond=None)
    resid = target - a @ coef
    sigma2 = resid @ resid / (len(target) - a.shape[1])
    se = np.sqrt(np.diag(sigma2 * np.linalg.pinv(a.T @ a)))
    r2 = 1 - resid.var() / target.var()
    return names, coef, se, r2, a @ coef


def breakeven(value: int):
    """Make-probability where bidding beats passing, if defenders take about 3 tricks either way."""
    return 0.5 + 15 / value


def made(tricks, level):
    """How often a bid of `level` is made in real play. `tricks` are solver tricks less DD_BIAS;
    real play runs half a trick below the solver on average, so a deal the solver makes with
    exactly the bid counts as half made (real play takes the solver's count or one fewer, equally
    often). Counting it as a failure, as an earlier version did, charged a whole trick."""
    dd = tricks + DD_BIAS
    return ((dd >= level + 1) + 0.5 * (dd == level)).mean()


def threshold(estimate, tricks, level, value):
    """Smallest estimate (in 0.25 steps) from which the contract makes often enough to pay."""
    need = breakeven(value)
    for e in np.arange(4.0, 10.01, 0.25):
        sel = (estimate >= e - 0.25) & (estimate < e + 0.25)
        if sel.sum() >= 40 and made(tricks[sel], level) >= need:
            return e, made(tricks[sel], level), sel.sum()
    return None


def step_up(estimate, tricks, t, value8):
    """Points gained on average by bidding 9 instead of 8 in suit `t`, by estimate. Going up adds
    100 when 9 is made; taking exactly 8 turns +value8 into -(value8 + 100); fewer costs 100."""
    rows = []
    for e in np.arange(8.0, 11.01, 0.5):
        sel = (estimate >= e - 0.25) & (estimate < e + 0.25)
        if sel.sum() < 30:
            continue
        dd = tricks[sel] + DD_BIAS
        gain = 0.0
        for real in (dd, dd - 1):  # equally likely, as in made()
            exactly8 = real == 8
            gain += 0.5 * np.where(real >= 9, 100, np.where(exactly8, -(2 * value8 + 100), -100)).mean()
        rows.append((e, made(tricks[sel], 9), gain, sel.sum()))
    return rows


def main():
    sys.stdout.reconfigure(encoding="utf-8")
    ap = argparse.ArgumentParser()
    ap.add_argument("--data", default=str(ROOT / "train/data/bid"))
    args = ap.parse_args()
    prefix = pathlib.Path(args.data)
    meta = json.loads(prefix.with_suffix(".json").read_text())
    x = np.fromfile(prefix.with_suffix(".x"), dtype="<f4").reshape(meta["rows"], meta["features"]).astype(float)
    y = np.fromfile(prefix.with_suffix(".y"), dtype=np.uint8).reshape(meta["rows"], meta["labels"])
    print(f"{len(x)} bidding decisions from {meta['deals']} deals\n")

    # Suit contracts: pool all four suits (each row × each trump suit).
    rows = {k: [] for k in list(suit_features(x[:1], 0)) + list(call_features(x[:1], 0))}
    target = []
    for t in range(4):
        ok = y[:, t] != 255
        f = {**suit_features(x[ok], t), **call_features(x[ok], t)}
        for k in rows:
            rows[k].append(f[k])
        target.append(y[ok, t].astype(float) - DD_BIAS)
    features = {k: np.concatenate(v) for k, v in rows.items()}
    tricks = np.concatenate(target)
    suit_of = np.concatenate([np.full((y[:, t] != 255).sum(), t) for t in range(4)])
    names, coef, se, r2, estimate = fit(features, tricks)
    print("## Suit contracts: tricks your side takes as declarer (real-play scale)\n")
    print(f"Starting point (an empty hand, no information): {coef[0]:.2f} tricks.  R² = {r2:.2f}\n")
    print("| Feature | Tricks | ± |")
    print("|---|---|---|")
    for n, c, s in zip(names, coef[1:], se[1:]):
        print(f"| {n} | {c:+.2f} | {1.96 * s:.2f} |")

    print("\nWhen does bidding pay? (house scoring; defenders' 10 per trick included)\n")
    print("| Contract | Make-chance needed | Estimate from which it pays | Made at that estimate |")
    print("|---|---|---|---|")
    for t, value7 in [(0, 140), (1, 160), (2, 180), (3, 200)]:
        for level, value in [(7, value7), (8, value7 + 100)]:
            r = threshold(estimate, tricks, level, value)
            label = f"{level}{SUIT[t]} ({value})"
            if r:
                print(f"| {label} | {breakeven(value):.0%} | {r[0]:.2f} | {r[1]:.0%} (n={r[2]}) |")
            else:
                print(f"| {label} | {breakeven(value):.0%} | — | — |")

    print("\nGoing from 8 to 9 (no one bidding against you): points gained by bidding 9\n")
    print("| Count | " + " | ".join(f"9{s}: made, gain" for s in SUIT) + " |")
    print("|---|---|---|---|---|")
    per_suit = [dict((r[0], r) for r in step_up(estimate[suit_of == t], tricks[suit_of == t], t, 240 + 20 * t)) for t in range(4)]
    for e in sorted(set().union(*per_suit)):
        cells = [f"{d[e][1]:.0%}, {d[e][2]:+.0f}" if e in d else "—" for d in per_suit]
        print(f"| {e:.1f} | " + " | ".join(cells) + " |")

    # No trumps.
    ok = y[:, 4] != 255
    f = {**nt_features(x[ok]), **call_features(x[ok], 4)}
    nt = y[ok, 4].astype(float) - DD_BIAS
    names, coef, se, r2, estimate = fit(f, nt)
    print("\n## No trumps\n")
    print(f"Starting point: {coef[0]:.2f} tricks.  R² = {r2:.2f}\n")
    print("| Feature | Tricks | ± |")
    print("|---|---|---|")
    for n, c, s in zip(names, coef[1:], se[1:]):
        print(f"| {n} | {c:+.2f} | {1.96 * s:.2f} |")
    for level, value in [(7, 220), (8, 320)]:
        r = threshold(estimate, nt, level, value)
        if r:
            print(f"\n{level}NT ({value}) pays from an estimate of about {r[0]:.2f} (made {r[1]:.0%} there, n={r[2]}).")

    # How much do partner and the kitty add, on average? Compare the bidder's own-hand estimate.
    print("\n## Averages\n")
    best = np.where(y[:, :4] == 255, 0, y[:, :4]).max(1) - DD_BIAS
    print(f"Best suit, average tricks for the bidder's side: {best.mean():.2f}")


if __name__ == "__main__":
    main()
