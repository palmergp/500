"""Do guarded kings add up, or does each extra one count for less?

Refits the bidding count (`rules_of_thumb.py`: the hand and the auction) on the solver data with the
guarded side kings (no ace in the suit) entered as "exactly 1, 2, 3 of them" instead of a count, so
each king gets its own value: the first, the second, the third. Kings beside their ace stay a count,
as in the count. Done for suit bids (the trump suit's kings aside) and for no trumps, and split by
the guard (one, or two or more), since the count already values those apart (finding 25).

    train/.venv/Scripts/python train/kings.py [--data train/data/bid,train/data/bid_selfplay]
"""

import argparse
import pathlib
import sys

import numpy as np

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
import mine_count as mc  # noqa: E402
import rules_of_thumb as rt  # noqa: E402

ROOT = pathlib.Path(__file__).resolve().parent.parent


def suit_kings(x, t):
    """Side kings when suit t is trumps: (beside the ace, guarded with one guard, with 2+ guards)."""
    side = [s for s in range(4) if s != t]
    ace_k, thin, thick = 0, 0, 0
    for s in side:
        length = rt.suit_cards(x, s).sum(1) - (x[:, rt.idx(s, 11)] if s == rt.SAME_COLOR[t] else 0)
        king, ace = x[:, rt.idx(s, 13)], x[:, rt.idx(s, 14)]
        ace_k = ace_k + king * ace
        thin = thin + king * (1 - ace) * (length == 2)
        thick = thick + king * (1 - ace) * (length >= 3)
    return ace_k, thin, thick


def nt_kings(x):
    ace_k, thin, thick = 0, 0, 0
    for s in range(4):
        length = rt.suit_cards(x, s).sum(1)
        king, ace = x[:, rt.idx(s, 13)], x[:, rt.idx(s, 14)]
        ace_k = ace_k + king * ace
        thin = thin + king * (1 - ace) * (length == 2)
        thick = thick + king * (1 - ace) * (length >= 3)
    return ace_k, thin, thick


def dummies(n, top, label):
    """Indicators for exactly 1, 2, ... (and `top` or more), leaving out any that never occur."""
    d = {f"{label}: exactly {k}" if k < top else f"{label}: {k} or more": (n == k).astype(float) if k < top else (n >= k).astype(float) for k in range(1, top + 1)}
    return {k: v for k, v in d.items() if v.sum() >= 20}


def show(title, base, extra, target):
    names, coef, se, r2, _ = rt.fit({**base, **extra}, target)
    print(f"### {title} (n = {len(target)}, R² = {r2:.3f})\n")
    print("| Holding | How often | Worth in all (tricks) | ± (95%) | This king adds |")
    print("|---|---|---|---|---|")
    prev = 0.0
    for k in extra:
        i = names.index(k) + 1
        print(f"| {k} | {extra[k].mean():.1%} | {coef[i]:+.2f} | {1.96 * se[i]:.2f} | {coef[i] - prev:+.2f} |")
        prev = coef[i]
    print()


def main():
    sys.stdout.reconfigure(encoding="utf-8")
    ap = argparse.ArgumentParser()
    ap.add_argument("--data", default=f"{ROOT / 'train/data/bid'},{ROOT / 'train/data/bid_selfplay'}")
    args = ap.parse_args()
    x, y = mc.load(args.data.split(","))
    print(f"{len(x)} bidding decisions\n")

    # Suit bids: every row in each suit it was solved in.
    feats, extras, target = [], [], []
    for t in range(4):
        ok = y[:, t] != 255
        f = {**rt.suit_features(x[ok], t), **rt.call_features(x[ok], t)}
        del f["Guarded side king"]
        ace_k, thin, thick = suit_kings(x[ok], t)
        f["Side king beside its ace"] = ace_k
        feats.append(f)
        extras.append((thin, thick))
        target.append(y[ok, t].astype(float) - rt.DD_BIAS)
    base = {k: np.concatenate([f[k] for f in feats]) for k in feats[0]}
    thin = np.concatenate([e[0] for e in extras])
    thick = np.concatenate([e[1] for e in extras])
    tricks = np.concatenate(target)

    print("## Suit bids\n")
    show("Guarded side kings (any guard), by how many", base, dummies(thin + thick, 3, "Guarded kings"), tricks)
    show("Kings with 2+ guards, by how many (one-guard kings counted at their own rate)", {**base, "King with one guard": thin}, dummies(thick, 3, "Kings with 2+ guards"), tricks)
    show("Kings with one guard, by how many (2+-guard kings counted at their own rate)", {**base, "King with 2+ guards": thick}, dummies(thin, 2, "Kings with one guard"), tricks)
    # For comparison: a count, the way the count has it.
    names, coef, se, r2, _ = rt.fit({**base, "King with one guard": thin, "King with 2+ guards": thick}, tricks)
    for k in ("King with one guard", "King with 2+ guards", "Side king beside its ace"):
        i = names.index(k) + 1
        print(f"- As a count (each king the same): {k} {coef[i]:+.2f} ± {1.96 * se[i]:.2f}")
    print()
    # Does the answer depend on trumps? Guarded kings count for more with fewer trumps?
    trumps = base["Each other trump (10 or lower)"] + sum(base[k] for k in ("Joker", "Right bower", "Left bower", "Ace of trumps", "King of trumps", "Queen of trumps"))
    for label, sel in (("4 or fewer trumps", trumps <= 4), ("5 trumps", trumps == 5), ("6+ trumps", trumps >= 6)):
        b = {k: v[sel] for k, v in base.items()}
        show(f"Guarded side kings, with {label}", b, dummies((thin + thick)[sel], 3, "Guarded kings"), tricks[sel])

    # No trumps.
    ok = y[:, 4] != 255
    f = {**rt.nt_features(x[ok]), **rt.call_features(x[ok], 4)}
    del f["Guarded king"]
    # "Unstopped suit" is defined by the aces and guarded kings themselves, so it would take a share of
    # their value; the count has no such term either.
    del f["Unstopped suit"]
    ace_k, thin, thick = nt_kings(x[ok])
    f["King beside its ace"] = ace_k
    nt = y[ok, 4].astype(float) - rt.DD_BIAS
    print("## No trumps\n")
    show("Guarded kings (no ace in the suit, any guard), by how many", f, dummies(thin + thick, 4, "Guarded kings"), nt)
    show("Kings with one guard, by how many", {**f, "King with 2+ guards": thick}, dummies(thin, 3, "Kings with one guard"), nt)
    show("Kings with 2+ guards, by how many", {**f, "King with one guard": thin}, dummies(thick, 3, "Kings with 2+ guards"), nt)
    names, coef, se, r2, _ = rt.fit({**f, "King with one guard": thin, "King with 2+ guards": thick}, nt)
    for k in ("King with one guard", "King with 2+ guards", "King beside its ace"):
        i = names.index(k) + 1
        print(f"- As a count: {k} {coef[i]:+.2f} ± {1.96 * se[i]:.2f}")
    print()
    # Aces, for comparison: do they add up?
    aces = f.pop("Ace")
    show("For comparison, aces in no trumps, by how many", {**f, "King with one guard": thin, "King with 2+ guards": thick}, dummies(aces, 4, "Aces"), nt)


if __name__ == "__main__":
    main()
