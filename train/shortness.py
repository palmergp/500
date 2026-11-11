"""What are short side suits worth: voids, singletons, doubletons, and their combinations?

Refits the bidding count (`rules_of_thumb.py`: the hand and the auction) on the solver data with the
side-suit shape entered as categories: every combination of voids and singletons (and, separately,
doubletons) gets its own value against a hand with no short side suit, everything else held equal.
Also split by trump length, since shortness pays only by trumping, and for no trumps.

    train/.venv/Scripts/python train/shortness.py [--data train/data/bid,train/data/bid_selfplay]
"""

import argparse
import pathlib
import sys

import numpy as np

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
import mine_count as mc  # noqa: E402
import rules_of_thumb as rt  # noqa: E402

ROOT = pathlib.Path(__file__).resolve().parent.parent
TOP = ("Joker", "Right bower", "Left bower", "Ace of trumps", "King of trumps", "Queen of trumps", "Each other trump (10 or lower)")


def side_lengths(x, t):
    """Lengths of the three side suits when suit t is trumps (the left bower counts as a trump)."""
    return [rt.suit_cards(x, s).sum(1) - (x[:, rt.idx(s, 11)] if s == rt.SAME_COLOR[t] else 0) for s in range(4) if s != t]


def shape_name(v, s):
    parts = []
    if v:
        parts.append(f"{v} void{'s' if v > 1 else ''}")
    if s:
        parts.append(f"{s} singleton{'s' if s > 1 else ''}")
    return " + ".join(parts) or "no void or singleton"


def show(title, base, cats, target, note=None, min_rows=40):
    """Fits the count with indicator columns `cats` (baseline: none of them) and prints each one."""
    cats = {k: v for k, v in cats.items() if v.sum() >= min_rows}
    names, coef, se, r2, _ = rt.fit({**base, **cats}, target)
    print(f"### {title} (n = {len(target)}, R² = {r2:.3f})\n")
    if note:
        print(note + "\n")
    print("| Shape | How often | Worth (tricks) | ± (95%) |")
    print("|---|---|---|---|")
    for k in cats:
        i = names.index(k) + 1
        print(f"| {k} | {cats[k].mean():.1%} | {coef[i]:+.2f} | {1.96 * se[i]:.2f} |")
    print()
    return {k: coef[names.index(k) + 1] for k in cats}


def main():
    sys.stdout.reconfigure(encoding="utf-8")
    ap = argparse.ArgumentParser()
    ap.add_argument("--data", default=f"{ROOT / 'train/data/bid'},{ROOT / 'train/data/bid_selfplay'}")
    args = ap.parse_args()
    x, y = mc.load(args.data.split(","))
    print(f"{len(x)} bidding decisions\n")

    feats, lens, target = [], [], []
    for t in range(4):
        ok = y[:, t] != 255
        f = {**rt.suit_features(x[ok], t), **rt.call_features(x[ok], t)}
        # The shape goes in as categories instead.
        del f["Side void"], f["Side singleton"]
        # A singleton ace is worth less than an ace (rule 1): keep that apart from the shape.
        f["Singleton side ace"] = sum(x[ok][:, rt.idx(s, 14)] * (l == 1) for s, l in zip([s for s in range(4) if s != t], side_lengths(x[ok], t)))
        feats.append(f)
        lens.append(np.column_stack(side_lengths(x[ok], t)))
        target.append(y[ok, t].astype(float) - rt.DD_BIAS)
    base = {k: np.concatenate([f[k] for f in feats]) for k in feats[0]}
    lengths = np.concatenate(lens)
    tricks = np.concatenate(target)
    trumps = sum(base[k] for k in TOP)
    # Shape and trump length go together (six trumps leave four side cards, so something is short),
    # and the count's trumps add up linearly: give each trump length its own value too, so a long
    # trump suit's own worth doesn't land on the shapes that come with it.
    for k in range(3, 9):
        base[f"Exactly {k} trumps"] = (trumps == k).astype(float)
    voids = (lengths == 0).sum(1)
    singles = (lengths == 1).sum(1)
    doubles = (lengths == 2).sum(1)

    print("## Suit bids\n")
    cats = {shape_name(v, s): ((voids == v) & (singles == s)).astype(float) for v in range(3) for s in range(4) if (v, s) != (0, 0) and v + s <= 3}
    worth = show(
        "Every mix of voids and singletons",
        base,
        cats,
        tricks,
        "Against a hand with no void or singleton in a side suit.",
    )
    one_v, one_s = worth.get("1 void", 0), worth.get("1 singleton", 0)
    print("Adding up, or more than the parts?\n")
    print("| Shape | Measured | Sum of its parts (1 void, 1 singleton each) | Extra |")
    print("|---|---|---|---|")
    for (v, s), name in [((v, s), shape_name(v, s)) for v in range(3) for s in range(4) if v + s >= 2 and v + s <= 3]:
        if name in worth:
            parts = v * one_v + s * one_s
            print(f"| {name} | {worth[name]:+.2f} | {parts:+.2f} | {worth[name] - parts:+.2f} |")
    print()

    # By trump length: shortness pays by trumping. With five or more trumps a hand can't avoid a short
    # side suit, so within each length every shape is measured against one void (which all lengths have).
    allshapes = {shape_name(v, s): ((voids == v) & (singles == s)).astype(float) for v in range(4) for s in range(4) if v + s <= 3}
    print("By trump length, each shape against a hand with just one void (and the side suits otherwise 2+):\n")
    print("| Shape | " + " | ".join(["3 trumps", "4 trumps", "5 trumps", "6 trumps", "7+ trumps"]) + " |")
    print("|---|---|---|---|---|---|")
    bins = [trumps == 3, trumps == 4, trumps == 5, trumps == 6, trumps >= 7]
    results = []
    for sel in bins:
        b = {k: v[sel] for k, v in base.items() if not k.startswith("Exactly")}
        c = {k: v[sel] for k, v in allshapes.items() if k != "1 void" and v[sel].sum() >= 40}
        names, coef, se, _, _ = rt.fit({**b, **c}, tricks[sel])
        results.append({k: (coef[names.index(k) + 1], 1.96 * se[names.index(k) + 1], c[k].mean()) for k in c})
    for name in allshapes:
        if name == "1 void":
            continue
        cells = [f"{r[name][0]:+.2f} ± {r[name][1]:.2f} ({r[name][2]:.0%})" if name in r else "—" for r in results]
        if any(c != "—" for c in cells):
            print(f"| {name} | " + " | ".join(cells) + " |")
    print("\n(± is the 95% range; in brackets, how often the shape comes up at that trump length.)\n")

    # Singleton plus doubletons: one short suit and another shortish.
    cats2 = {}
    for s in range(3):
        for d in range(4):
            if (s, d) == (0, 0) or s + d > 3:
                continue
            name = " + ".join(p for p in [f"{s} singleton{'s' if s > 1 else ''}" if s else "", f"{d} doubleton{'s' if d > 1 else ''}" if d else ""] if p)
            cats2[name] = ((voids == 0) & (singles == s) & (doubles == d)).astype(float)
    show(
        "Singletons and doubletons (hands with no void)",
        base,
        {**cats2, "any void": (voids > 0).astype(float)},
        tricks,
        "Against a hand with no side suit shorter than three.",
    )

    # No trumps: shortness is a weakness there (nothing to trump with), if anything.
    ok = y[:, 4] != 255
    f = {**rt.nt_features(x[ok]), **rt.call_features(x[ok], 4)}
    del f["Unstopped suit"]
    lens_nt = np.column_stack([rt.suit_cards(x[ok], s).sum(1) for s in range(4)])
    v, s = (lens_nt == 0).sum(1), (lens_nt == 1).sum(1)
    nt_cats = {shape_name(a, b): ((v == a) & (s == b)).astype(float) for a in range(3) for b in range(3) if (a, b) != (0, 0)}
    print("## No trumps\n")
    show("Voids and singletons (any suit)", f, nt_cats, y[ok, 4].astype(float) - rt.DD_BIAS)


if __name__ == "__main__":
    main()
