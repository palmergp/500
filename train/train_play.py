"""Train the neural card-play network from `fh-lab playdata` output.

Each row is a card-play decision: what the player could see (PLAY_FEATURES floats, see
crates/ai/src/features.rs) and the search's value in points for each card slot (NaN where the card
can't be played). The network learns a distribution over slots: the target is a softmax of the
values over the legal cards, with temperature TEMPERATURE points, so the best card dominates and
near-equal cards share the credit.

Writes crates/ai/models/play.fhnn (the FHNN format read by crates/ai/src/nn.rs).

    train/.venv/Scripts/python train/train_play.py [--data train/data/play] [--epochs 10]
"""

import argparse
import json
import pathlib
import struct
import sys

import numpy as np
import torch
from torch import nn

ROOT = pathlib.Path(__file__).resolve().parent.parent
TEMPERATURE = 10.0


def load(prefix: pathlib.Path):
    meta = json.loads(prefix.with_suffix(".json").read_text())
    x = np.fromfile(prefix.with_suffix(".x"), dtype="<f4").reshape(meta["rows"], meta["features"])
    y = np.fromfile(prefix.with_suffix(".y"), dtype="<f4").reshape(meta["rows"], meta["slots"])
    return x, y, meta


class PlayNet(nn.Module):
    def __init__(self, features: int, slots: int, hidden: int = 512, dropout: float = 0.0):
        super().__init__()
        # Dropout only trains; the exported network (Linear layers only) runs without it.
        self.layers = nn.Sequential(
            nn.Linear(features, hidden),
            nn.ReLU(),
            nn.Dropout(dropout),
            nn.Linear(hidden, hidden // 2),
            nn.ReLU(),
            nn.Dropout(dropout),
            nn.Linear(hidden // 2, slots),
        )

    def forward(self, x):
        return self.layers(x)


def targets(y: torch.Tensor):
    legal = ~torch.isnan(y)
    values = torch.where(legal, y, torch.full_like(y, -1e9))
    best = values.max(-1, keepdim=True).values
    soft = torch.softmax((values - best) / TEMPERATURE, -1)
    return soft, legal, torch.where(legal, y, torch.zeros_like(y))


def evaluate(model, x, y, batch=8192):
    """Agreement with the search's best card, and the average points the network's pick gives up."""
    agree = regret = 0.0
    with torch.no_grad():
        for i in range(0, len(x), batch):
            xb, yb = x[i : i + batch], y[i : i + batch]
            _, legal, values = targets(yb)
            logits = model(xb).masked_fill(~legal, -1e9)
            pick = logits.argmax(-1)
            best = torch.where(legal, values, torch.full_like(values, -1e9)).max(-1)
            agree += (pick == best.indices).float().sum().item()
            regret += (best.values - values.gather(1, pick[:, None]).squeeze(1)).sum().item()
    return agree / len(x), regret / len(x)


def export(model: PlayNet, path: pathlib.Path, half: bool = True):
    """FHNN (f32) or, with `half`, FHNH (16-bit floats: half the size, no measurable loss)."""
    layers = [m for m in model.layers if isinstance(m, nn.Linear)]
    dtype = "<f2" if half else "<f4"
    out = bytearray(b"FHNH" if half else b"FHNN")
    out += struct.pack("<I", len(layers))
    for layer in layers:
        w = layer.weight.detach().numpy().astype(dtype)
        b = layer.bias.detach().numpy().astype(dtype)
        out += struct.pack("<II", w.shape[1], w.shape[0])
        out += w.tobytes()
        out += b.tobytes()
    path.write_bytes(bytes(out))


def main():
    sys.stdout.reconfigure(encoding="utf-8")
    ap = argparse.ArgumentParser()
    ap.add_argument("--data", default=str(ROOT / "train/data/play"))
    ap.add_argument("--epochs", type=int, default=10)
    ap.add_argument("--hidden", type=int, default=512)
    ap.add_argument("--dropout", type=float, default=0.2)
    ap.add_argument("--out", default=str(ROOT / "crates/ai/models/play.fhnn"))
    ap.add_argument("--seed", type=int, default=1)
    args = ap.parse_args()
    torch.manual_seed(args.seed)

    x, y, meta = load(pathlib.Path(args.data))
    print(f"{len(x)} decisions from {meta['deals']} deals, {meta['features']} inputs, {meta['slots']} slots")
    # Rows come grouped by deal, so the last 5% are (nearly all) deals the network never sees.
    split = int(len(x) * 0.95)
    xt, yt = torch.tensor(x[:split]), torch.tensor(y[:split])
    xv, yv = torch.tensor(x[split:]), torch.tensor(y[split:])

    model = PlayNet(meta["features"], meta["slots"], args.hidden, args.dropout)
    opt = torch.optim.AdamW(model.parameters(), lr=1e-3, weight_decay=1e-4)
    sched = torch.optim.lr_scheduler.CosineAnnealingLR(opt, args.epochs)
    batch = 1024
    best_regret, best_state = float("inf"), None
    for epoch in range(args.epochs):
        model.train()
        perm = torch.randperm(len(xt))
        total = 0.0
        for i in range(0, len(xt), batch):
            idx = perm[i : i + batch]
            soft, legal, _ = targets(yt[idx])
            logits = model(xt[idx]).masked_fill(~legal, -1e9)
            loss = -(soft * torch.log_softmax(logits, -1)).sum(-1).mean()
            opt.zero_grad()
            loss.backward()
            opt.step()
            total += loss.item() * len(idx)
        sched.step()
        model.eval()
        agree, regret = evaluate(model, xv, yv)
        print(f"epoch {epoch + 1}: loss {total / len(xt):.3f}; held-out: picks the search's best card {agree:.1%}, gives up {regret:.2f} points a decision")
        if regret < best_regret:
            # Keep the pass that plays best on held-out deals (later passes start to overfit).
            best_regret, best_state = regret, {k: v.clone() for k, v in model.state_dict().items()}
    model.load_state_dict(best_state)
    print(f"kept the best pass: gives up {best_regret:.2f} points a decision")

    # A baseline: always the first legal card in slot order (the strongest trump, or the ace).
    _, legal, values = targets(yv)
    first = legal.float().argmax(-1)
    best = torch.where(legal, values, torch.full_like(values, -1e9)).max(-1)
    naive = (best.values - values.gather(1, first[:, None]).squeeze(1)).mean().item()
    print(f"baseline (highest legal card): gives up {naive:.2f} points a decision")
    # The exported weights are 16-bit: check that the rounding costs nothing.
    import copy
    rounded = copy.deepcopy(model)
    with torch.no_grad():
        for param in rounded.parameters():
            param.copy_(param.half().float())
    rounded.eval()
    print(f"with 16-bit weights: gives up {evaluate(rounded, xv, yv)[1]:.2f} points a decision")
    export(model, pathlib.Path(args.out))
    print(f"wrote {args.out}")


if __name__ == "__main__":
    main()
