#!/bin/zsh
S=/private/tmp/claude-501/-Users-hadim-Code-libs-rdocx/08057ee4-afc5-4751-aca1-2668752380de/scratchpad
cd $S/review
PY=$S/venv403/bin/python
for seed in $(seq $1 $2); do
  $PY diffops2.py $S/baseline-rpptx $seed $3 b2-$seed.json
  $PY diffops2.py $S/review/debug $seed $3 r2-$seed.json
  $PY - $seed <<'PYEOF'
import json, sys
seed = sys.argv[1]
a = json.load(open(f"b2-{seed}.json")); b = json.load(open(f"r2-{seed}.json"))
for x, y in zip(a, b):
    if x != y:
        print(f"seed {seed} DIFF at step {x['step']} {x['op']} base={x['outcome']!r} branch={y['outcome']!r} bytes {x['bytes']} {y['bytes']}")
        for i, (p, q) in enumerate(zip(x["probes"], y["probes"])):
            if p != q:
                print("   probe", i, p, q)
        break
else:
    print(f"seed {seed} identical {len(a)} steps, ok {sum(x['outcome']=='ok' for x in a)}")
PYEOF
done
