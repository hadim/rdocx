#!/bin/zsh
# Runs diffops.py on main (release) and the branch (debug) for many seeds and reports divergences.
S=/private/tmp/claude-501/-Users-hadim-Code-libs-rdocx/08057ee4-afc5-4751-aca1-2668752380de/scratchpad
cd $S/review
PY=$S/venv403/bin/python
for seed in $(seq $1 $2); do
  $PY diffops.py $S/baseline-rpptx $seed $3 base-$seed.json
  $PY diffops.py $S/review/debug $seed $3 br-$seed.json
  $PY - $seed <<'EOF'
import json, sys
from collections import Counter
seed = sys.argv[1]
a = json.load(open(f"base-{seed}.json")); b = json.load(open(f"br-{seed}.json"))
ops = Counter(x["op"] for x in a if x["outcome"] == "ok")
for x, y in zip(a, b):
    if x != y:
        print(f"seed {seed} DIFF at step {x['step']} {x['op']} base={x['outcome']!r} branch={y['outcome']!r} bytes {x['bytes']} {y['bytes']}")
        for i, (p, q) in enumerate(zip(x["probes"], y["probes"])):
            if p != q:
                print("   probe", i, p, q)
        break
else:
    print(f"seed {seed} identical {len(a)} steps, ok ops {sum(ops.values())}, pool {len(a[-1]['probes'])}")
EOF
done
