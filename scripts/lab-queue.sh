#!/usr/bin/env bash
# The lab's long runs, one after another, resumable: a job whose output already exists is skipped,
# and each writes to a temporary file first, so a restart loses only the job in progress.
# Run from the repo root: bash scripts/lab-queue.sh
set -u
B=./target/fh-lab-queue.exe
mkdir -p target/lab
run() { # name, then the fh-lab arguments
  local out="target/lab/$1.txt"; shift
  if [ -s "$out" ]; then echo "skip $out"; return; fi
  echo "start $out $(date +%H:%M)"
  if "$B" "$@" > "$out.tmp" 2>&1; then mv "$out.tmp" "$out"; echo "done $out $(date +%H:%M)"; else echo "FAILED $out"; fi
}
# 1. Fresh bidding data under today's conventions, in chunks.
for k in 0 1 2 3 4 5; do
  if [ -s "train/data/bid_v3_$k.json" ]; then echo "skip chunk $k"; continue; fi
  echo "start chunk $k $(date +%H:%M)"
  "$B" dataset --deals 5000 --seed $((5000000 + k * 5000)) --bidder oracle --out "train/data/bid_v3_$k" > "target/lab/dataset-v3-$k.txt" 2>&1 && echo "done chunk $k $(date +%H:%M)"
done
# 2. The studies.
run kittysearch kittysearch --deals 3000
run hintcheck hintcheck --deals 600
run plays plays --deals 2000
run duel-guide duel --a guide --b oracle --deals 1000 --seed 3200000
run duel-guide-noauction duel --a guide --b guide-noauction --deals 1000 --seed 3300000
run duel-guide-nine duel --a guide --b guide-nine --deals 1000 --seed 3400000
run duel-guide-overcalls duel --a guide --b guide-overcalls --deals 1000 --seed 3500000
run duel-guide-nosignals duel --a guide --b guide-nosignals --deals 1000 --seed 3600000
# 3. Follow-ups: the retrained network in play, bolder guides, fairer tests.
export FH_ALT_BID_MODEL=train/data/bid_v3_mixed.fhnn
run duel-altnet duel --a oracle-altnet --b oracle --deals 2000 --seed 3700000
run hintcheck-closer hintcheck --deals 600
run plays-v2 plays --deals 2000
run duel-guide-bold duel --a guide-bold --b guide --deals 1000 --seed 3800000
run duel-guide-bolder duel --a guide-bolder --b guide --deals 1000 --seed 3900000
run duel-guide-boldest duel --a guide-boldest --b guide --deals 1000 --seed 4000000
echo "queue finished $(date +%H:%M)"
