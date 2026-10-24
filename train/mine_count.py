"""Look for what the bidding count misses.

Refits the count (`rules_of_thumb.py`: the hand features and the auction) on the solver data, then
tries a long list of candidate features one at a time, each added to the full fit, so a candidate's
value is what it adds beyond everything the count already has. Candidates are ranked by how much
they reduce the count's error over all decisions (their value times how often they come up). The
top ones are rules of thumb worth checking.

    train/.venv/Scripts/python train/mine_count.py [--data train/data/bid,train/data/bid_selfplay]
"""

import argparse
import json
import pathlib
import sys

import numpy as np

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
import rules_of_thumb as rt  # noqa: E402

ROOT = pathlib.Path(__file__).resolve().parent.parent


def load(prefixes):
    xs, ys = [], []
    for prefix in prefixes:
        prefix = pathlib.Path(prefix)
        meta = json.loads(prefix.with_suffix(".json").read_text())
        xs.append(np.fromfile(prefix.with_suffix(".x"), dtype="<f4").reshape(meta["rows"], meta["features"]).astype(float))
        ys.append(np.fromfile(prefix.with_suffix(".y"), dtype=np.uint8).reshape(meta["rows"], meta["labels"]))
    return np.concatenate(xs), np.concatenate(ys)


def suit_candidates(x, t):
    """Candidate features when suit t is trumps."""
    joker = x[:, 42]
    right = x[:, rt.idx(t, 11)]
    left = x[:, rt.idx(rt.SAME_COLOR[t], 11)]
    ace, king, queen = (x[:, rt.idx(t, r)] for r in (14, 13, 12))
    ten, nine = x[:, rt.idx(t, 10)], x[:, rt.idx(t, 9)]
    trumps = rt.suit_cards(x, t).sum(1) + left + joker
    top = joker + right + left + ace
    side = [s for s in range(4) if s != t]
    cards = {s: rt.suit_cards(x, s) for s in side}
    length = {s: cards[s].sum(1) - (x[:, rt.idx(s, 11)] if s == rt.SAME_COLOR[t] else 0) for s in side}
    has = lambda s, r: x[:, rt.idx(s, r)] * (0 if (r == 11 and s == rt.SAME_COLOR[t]) else 1)
    side_aces = sum(has(s, 14) for s in side)
    voids = sum((length[s] == 0).astype(float) for s in side)
    c = {}
    for k in range(2, 10):
        c[f"Exactly {k} trumps"] = (trumps == k).astype(float)
    for k in range(5):
        c[f"{k} of the top four trumps (Joker, bowers, ace)"] = (top == k).astype(float)
    c["Both bowers"] = right * left
    c["The Joker and both bowers"] = joker * right * left
    c["Joker, without either bower"] = joker * (1 - right) * (1 - left)
    c["Right bower without the Joker or left bower"] = right * (1 - joker) * (1 - left)
    c["Ten of trumps"] = ten
    c["Nine of trumps"] = nine
    c["Ace-king of trumps"] = ace * king
    c["Void, with 6+ trumps"] = voids * (trumps >= 6)
    c["Void, with 4 or fewer trumps"] = voids * (trumps <= 4)
    c["Side ace, with 4 or fewer trumps"] = side_aces * (trumps <= 4)
    c["Side ace, with 7+ trumps"] = side_aces * (trumps >= 7)
    c["Two or more side voids"] = (voids >= 2).astype(float)
    c["Side A-K"] = sum(has(s, 14) * has(s, 13) for s in side)
    c["Side A-K-Q"] = sum(has(s, 14) * has(s, 13) * has(s, 12) for s in side)
    c["Side K-Q without the ace"] = sum((1 - has(s, 14)) * has(s, 13) * has(s, 12) for s in side)
    c["Side singleton ace"] = sum(has(s, 14) * (length[s] == 1) for s in side)
    c["Side singleton king"] = sum(has(s, 13) * (length[s] == 1) for s in side)
    c["Side queen (any)"] = sum(has(s, 12) for s in side)
    c["Side jack (not a bower)"] = sum(has(s, 11) for s in side if s != rt.SAME_COLOR[t])
    c["Side ten"] = sum(has(s, 10) for s in side)
    c["Side suit of 4"] = sum((length[s] == 4).astype(float) for s in side)
    c["Side suit of 5+"] = sum((length[s] >= 5).astype(float) for s in side)
    c["Side suit of 4+ headed by the ace"] = sum(has(s, 14) * (length[s] >= 4) for s in side)
    c["Side doubleton with no honour"] = sum(((length[s] == 2) & (has(s, 14) + has(s, 13) + has(s, 12) == 0)).astype(float) for s in side)
    c["Left bower's suit: void (besides the bower)"] = (length[rt.SAME_COLOR[t]] == 0).astype(float)
    c["Left bower's suit: ace"] = has(rt.SAME_COLOR[t], 14)
    # The auction.
    pos = [x[:, rt.POS + k] for k in range(4)]
    for k, name in enumerate(["First to speak", "Second to speak", "Third to speak"]):
        c[name] = pos[k]
    mention = lambda b, s: x[:, b + 2 + s]
    six_any = lambda b: x[:, b + 7:b + 12].sum(1)
    passed = lambda b: x[:, b + 1]
    c["Left-hand opponent mentioned this suit"] = mention(rt.LEFT, t)
    c["Right-hand opponent mentioned this suit"] = mention(rt.RIGHT, t)
    c["Both opponents passed"] = passed(rt.LEFT) * passed(rt.RIGHT)
    c["Partner passed, with 6+ trumps"] = passed(rt.PARTNER) * (trumps >= 6)
    c["Partner passed, with the Joker"] = passed(rt.PARTNER) * joker
    c["Partner mentioned this suit, with 5+ trumps"] = mention(rt.PARTNER, t) * (trumps >= 5)
    c["Partner mentioned this suit, holding both bowers"] = mention(rt.PARTNER, t) * right * left
    nobody6 = (six_any(rt.LEFT) + six_any(rt.PARTNER) + six_any(rt.RIGHT) == 0).astype(float)
    called = lambda b: 1 - x[:, b]  # slot 0: no call yet
    everyone_called = called(rt.LEFT) * called(rt.PARTNER) * called(rt.RIGHT)
    c["Nobody showed the Joker, without it yourself, after three calls"] = nobody6 * everyone_called * (1 - joker)
    c["Nobody showed the Joker, without it yourself, after two calls"] = nobody6 * (pos[2]) * (1 - joker)
    return c, trumps


def nt_candidates(x):
    """Candidate features for no trumps."""
    joker = x[:, 42]
    has = lambda s, r: x[:, rt.idx(s, r)]
    length = {s: rt.suit_cards(x, s).sum(1) for s in range(4)}
    aces = sum(has(s, 14) for s in range(4))
    stopped = sum(((has(s, 14) == 1) | ((has(s, 13) == 1) & (length[s] >= 2)) | ((has(s, 12) == 1) & (length[s] >= 3))).astype(float) for s in range(4))
    c = {}
    for k in range(5):
        c[f"Exactly {k} aces"] = (aces == k).astype(float)
    c["Joker and 2+ aces"] = joker * (aces >= 2)
    c["Joker and no ace"] = joker * (aces == 0)
    c["Ace, with the Joker"] = aces * joker
    for k in range(5):
        c[f"{k} suits stopped (A, K-x or Q-x-x)"] = (stopped == k).astype(float)
    c["Side A-K"] = sum(has(s, 14) * has(s, 13) for s in range(4))
    c["A-Q in a suit"] = sum(has(s, 14) * has(s, 12) * (1 - has(s, 13)) for s in range(4))
    c["K-Q in a suit (no ace)"] = sum(has(s, 13) * has(s, 12) * (1 - has(s, 14)) for s in range(4))
    c["A-K-Q in a suit"] = sum(has(s, 14) * has(s, 13) * has(s, 12) for s in range(4))
    c["5+ suit headed by A-K"] = sum(has(s, 14) * has(s, 13) * (length[s] >= 5) for s in range(4))
    c["Long suit (5+) without the ace"] = sum((1 - has(s, 14)) * (length[s] >= 5) for s in range(4))
    c["Jack (any)"] = sum(has(s, 11) for s in range(4))
    c["Singleton"] = sum((length[s] == 1).astype(float) for s in range(4))
    c["Void"] = sum((length[s] == 0).astype(float) for s in range(4))
    c["Lone king"] = sum(has(s, 13) * (length[s] == 1) for s in range(4))
    pos = [x[:, rt.POS + k] for k in range(4)]
    for k, name in enumerate(["First to speak", "Second to speak", "Third to speak"]):
        c[name] = pos[k]
    return c


def report(title, base, extra, target, top=25):
    names, coef, se, r2, pred = rt.fit(base, target)
    resid = target - pred
    print(f"## {title} (n = {len(target)}, count's R² = {r2:.3f}, error {np.abs(resid).mean():.3f} tricks)\n")
    print("| Candidate | Worth (tricks) | ± (95%) | How often | Error removed per 1,000 decisions |")
    print("|---|---|---|---|---|")
    rows = []
    for name, feat in extra.items():
        if feat.std() == 0:
            continue
        n2, c2, s2, _, pred2 = rt.fit({**base, name: feat}, target)
        k = n2.index(name)
        gain = (resid ** 2).mean() - ((target - pred2) ** 2).mean()
        rows.append((gain, name, c2[k + 1], s2[k + 1], (feat != 0).mean()))
    rows.sort(reverse=True)
    for gain, name, c, s, often in rows[:top]:
        print(f"| {name} | {c:+.2f} | {1.96 * s:.2f} | {often:.1%} | {1000 * gain:.1f} |")
    print()
    # How the count's error runs with the count itself (is it too flat or too steep?).
    print("| The count's estimate | n | Actual less the count |\n|---|---|---|")
    for lo in np.arange(4.0, 11.0, 1.0):
        m = (pred >= lo) & (pred < lo + 1)
        if m.sum() >= 200:
            print(f"| {lo:.0f}–{lo + 1:.0f} | {m.sum()} | {resid[m].mean():+.2f} |")
    print()


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--data", default=f"{ROOT / 'train/data/bid'},{ROOT / 'train/data/bid_selfplay'}")
    args = ap.parse_args()
    x, y = load(args.data.split(","))
    sys.stdout.reconfigure(encoding="utf-8")
    base_rows, extra_rows, target = None, None, []
    for t in range(4):
        ok = y[:, t] != 255
        xs = x[ok]
        base = {**rt.suit_features(xs, t), **rt.call_features(xs, t)}
        extra, _ = suit_candidates(xs, t)
        base_rows = {k: [v] for k, v in base.items()} if base_rows is None else {k: base_rows[k] + [base[k]] for k in base_rows}
        extra_rows = {k: [v] for k, v in extra.items()} if extra_rows is None else {k: extra_rows[k] + [extra[k]] for k in extra_rows}
        target.append(y[ok, t].astype(float) - rt.DD_BIAS)
    base = {k: np.concatenate(v) for k, v in base_rows.items()}
    extra = {k: np.concatenate(v) for k, v in extra_rows.items()}
    report("Suit bids: what the count misses", base, extra, np.concatenate(target))
    ok = y[:, 4] != 255
    xs = x[ok]
    report("No trumps: what the count misses", {**rt.nt_features(xs), **rt.call_features(xs, 4)}, nt_candidates(xs), y[ok, 4].astype(float) - rt.DD_BIAS)


if __name__ == "__main__":
    main()
