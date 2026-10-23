"""Compare two diff407 logs: base (main) and new (branch). XML may differ only by the w14 contract."""
import json
import sys

DECL = ' xmlns:w14="http://schemas.microsoft.com/office/word/2010/wordml"'
base = json.load(open(sys.argv[1]))
new = json.load(open(sys.argv[2]))
problems = 0
w14_diffs = 0
panics = 0


def walk(a, b, path):
    global problems, w14_diffs, panics
    if isinstance(b, list) and b and b[0] == "err" and len(b) > 1 and "Panic" in str(b[1]):
        panics += 1
        if panics <= 20:
            print("PANIC", path, str(b[2])[:300])
    if a == b:
        return
    if isinstance(a, str) and isinstance(b, str) and a.startswith("<") and b.startswith("<"):
        if a.replace(DECL, "", 1) == b and "w14:" not in b:
            w14_diffs += 1
            return
    if isinstance(a, list) and isinstance(b, list) and len(a) == len(b):
        for i, (x, y) in enumerate(zip(a, b)):
            walk(x, y, path + [i])
        return
    if isinstance(a, dict) and isinstance(b, dict) and a.keys() == b.keys():
        for k in a:
            walk(a[k], b[k], path + [k])
        return
    problems += 1
    if problems <= 30:
        print("DIFF", path)
        print("  base:", str(a)[:600])
        print("  new: ", str(b)[:600])


for seed in base:
    walk(base[seed], new.get(seed), [seed, base[seed].get("shape")])
print(f"seeds={len(base)} problems={problems} w14_contract_diffs={w14_diffs} panics={panics}")
