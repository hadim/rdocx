"""Compact per-step comparison of guard407 logs: op outcome and save outcome under main and branch."""
import json
import sys

base = json.load(open(sys.argv[1]))
new = json.load(open(sys.argv[2]))
only = sys.argv[3:] if len(sys.argv) > 3 else None


def short(o):
    if o is None:
        return "-"
    if o[0] == "ok":
        v = o[1]
        return f"ok:{str(v)[:12]}"
    return f"ERR:{o[2][:70]}"


for key in base:
    if only and not any(key.startswith(k) for k in only):
        continue
    b, n = base[key], new[key]
    if "build" in b or "build" in n:
        print(key, "build", b.get("build"), n.get("build"))
        continue
    same_all = True
    lines = []
    bi, ni = b["initial"], n["initial"]
    for (op, bo, bs), (_, no, ns) in zip(b["steps"], n["steps"]):
        bsave, nsave = bs["bytes"], ns["bytes"]
        save_same = bsave == nsave
        texts_same = bs["texts"] == ns["texts"]
        op_same = bo == no
        flag = "" if (save_same and texts_same and op_same) else "  <<"
        if flag:
            same_all = False
        lines.append(f"  {op:20s} op base={short(bo):30s} new={short(no):30s} | save base={short(bsave)[:24]:24s} new={short(nsave)[:24]:24s}"
                     f" texts={'=' if texts_same else 'DIFF'}{flag}")
    print(f"{key}: {'SAME' if same_all else 'DIFFERS'}")
    if not same_all:
        print("\n".join(lines))
