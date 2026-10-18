"""Train the neural bidding evaluator.

Input:  train/data/bid.{x,y,json} from `fh-lab dataset` (features are defined in
        crates/ai/src/features.rs; labels are double-dummy tricks per strain).
Output: crates/ai/models/bid.fhnn, read by crates/ai/src/nn.rs.

The network sees one bidder's hand, position and the calls so far, and predicts, for each strain,
a probability for each trick count 0-10 the bidder's side would take as declarer.

    train/.venv/Scripts/python train/train_bid.py [--data train/data/bid] [--epochs 40]
"""

import argparse
import json
import pathlib
import struct
import sys
import time

import numpy as np
import torch
from torch import nn

ROOT = pathlib.Path(__file__).resolve().parent.parent
STRAINS = ["S", "C", "D", "H", "NT"]


def load(prefix: pathlib.Path):
    meta = json.loads(prefix.with_suffix(".json").read_text())
    x = np.fromfile(prefix.with_suffix(".x"), dtype="<f4").reshape(meta["rows"], meta["features"])
    y = np.fromfile(prefix.with_suffix(".y"), dtype=np.uint8).reshape(meta["rows"], meta["labels"])
    return x, y


class BidNet(nn.Module):
    def __init__(self, features: int, hidden: int = 256):
        super().__init__()
        self.body = nn.Sequential(
            nn.Linear(features, hidden), nn.ReLU(),
            nn.Linear(hidden, hidden), nn.ReLU(),
            nn.Linear(hidden, 5 * 11),
        )

    def forward(self, x):
        return self.body(x).view(-1, 5, 11)


def loss_and_stats(model, x, y):
    """Mean cross-entropy over solved strains, and mean absolute error of the expected tricks."""
    logits = model(x)
    mask = y != 255
    target = torch.where(mask, y, torch.zeros_like(y)).long()
    ce = nn.functional.cross_entropy(logits.permute(0, 2, 1), target, reduction="none")
    loss = (ce * mask).sum() / mask.sum()
    expected = (logits.softmax(-1) * torch.arange(11.0)).sum(-1)
    mae = ((expected - target).abs() * mask).sum() / mask.sum()
    return loss, mae


def export(model: BidNet, path: pathlib.Path):
    layers = [m for m in model.body if isinstance(m, nn.Linear)]
    out = bytearray(b"FHNN")
    out += struct.pack("<I", len(layers))
    for layer in layers:
        w = layer.weight.detach().cpu().numpy().astype("<f4")
        b = layer.bias.detach().cpu().numpy().astype("<f4")
        out += struct.pack("<II", w.shape[1], w.shape[0])
        out += w.tobytes() + b.tobytes()
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_bytes(bytes(out))


def main():
    sys.stdout.reconfigure(encoding="utf-8")
    ap = argparse.ArgumentParser()
    ap.add_argument("--data", default=str(ROOT / "train/data/bid"), help="one or more dataset prefixes, comma-separated")
    ap.add_argument("--out", default=str(ROOT / "crates/ai/models/bid.fhnn"))
    ap.add_argument("--epochs", type=int, default=40)
    ap.add_argument("--hidden", type=int, default=256)
    ap.add_argument("--seed", type=int, default=1)
    args = ap.parse_args()

    torch.manual_seed(args.seed)
    # Hold out the last 10% of each dataset (rows come grouped by deal, so deals stay intact).
    parts = [load(pathlib.Path(d)) for d in args.data.split(",")]
    train_parts, val_parts = [], []
    for x, y in parts:
        split = int(len(x) * 0.9) // 4 * 4
        train_parts.append((x[:split], y[:split]))
        val_parts.append((x[split:], y[split:]))
    xt = torch.tensor(np.concatenate([p[0] for p in train_parts]))
    yt = torch.tensor(np.concatenate([p[1] for p in train_parts]))
    xv = torch.tensor(np.concatenate([p[0] for p in val_parts]))
    yv = torch.tensor(np.concatenate([p[1] for p in val_parts]))
    x = xt.numpy()
    print(f"{len(xt) + len(xv)} rows from {len(parts)} dataset(s), {x.shape[1]} features; unsolved labels: {(yt == 255).float().mean():.2%}")

    # Baselines on the validation set: one guess per strain, and the training mean per strain.
    solved = yv != 255
    mean = torch.stack([yt[:, s][yt[:, s] != 255].float().mean() for s in range(5)])
    base_mae = ((yv.float() - mean).abs() * solved).sum() / solved.sum()
    print(f"baseline (strain average) validation MAE: {base_mae:.3f} tricks")

    model = BidNet(x.shape[1], args.hidden)
    opt = torch.optim.AdamW(model.parameters(), lr=1e-3, weight_decay=1e-4)
    sched = torch.optim.lr_scheduler.CosineAnnealingLR(opt, args.epochs)
    best = (float("inf"), None)
    for epoch in range(args.epochs):
        start = time.time()
        model.train()
        perm = torch.randperm(len(xt))
        for i in range(0, len(xt), 512):
            idx = perm[i:i + 512]
            loss, _ = loss_and_stats(model, xt[idx], yt[idx])
            opt.zero_grad()
            loss.backward()
            opt.step()
        sched.step()
        model.eval()
        with torch.no_grad():
            vloss, vmae = loss_and_stats(model, xv, yv)
        if vloss.item() < best[0]:
            best = (vloss.item(), {k: v.clone() for k, v in model.state_dict().items()})
        print(f"epoch {epoch + 1:2d}  val loss {vloss:.4f}  val MAE {vmae:.3f}  ({time.time() - start:.1f}s)")

    model.load_state_dict(best[1])
    model.eval()
    with torch.no_grad():
        vloss, vmae = loss_and_stats(model, xv, yv)
        expected = (model(xv).softmax(-1) * torch.arange(11.0)).sum(-1)
    print(f"best: val loss {vloss:.4f}, MAE {vmae:.3f} tricks (strain-average baseline {base_mae:.3f})")
    for s, name in enumerate(STRAINS):
        m = solved[:, s]
        err = (expected[m, s] - yv[m, s].float())
        print(f"  {name:>2}: bias {err.mean():+.3f}  MAE {err.abs().mean():.3f}")
    export(model, pathlib.Path(args.out))
    print(f"wrote {args.out} ({pathlib.Path(args.out).stat().st_size // 1024} KB)")


if __name__ == "__main__":
    main()
