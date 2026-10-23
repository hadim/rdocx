"""Which growing calls each cumulative PR set should turn constant: python accept.py pr25.txt pr100.txt

A PR removes the whole-document work of its buckets (cost 0 after it); the call passes the scaling test when what
remains satisfies 4N <= 1.5 * N + 5 ms. Totals are the N and 4N times of the #415 run (scale407/final/scaling.txt)."""
import itertools
import re
import sys

S = "/private/tmp/claude-501/-Users-hadim-Code-libs-rdocx/08057ee4-afc5-4751-aca1-2668752380de/scratchpad"
GROUPS = ("P1", "P2", "P3", "P4", "C", "E")


def load(path):
    rows = {}
    for line in open(path):
        if line.startswith("PR "):
            _, name, total, *shares = line.split()
            rows[name] = dict(zip(GROUPS, (float(x) / 100 for x in shares)))
    return rows


def main():
    small, large = load(sys.argv[1]), load(sys.argv[2])
    totals = {}
    for line in open(f"{S}/scale407/final/scaling.txt"):
        m = re.search(r"AssertionError: (\S+): ([\d.]+) ms at 100 paragraphs, ([\d.]+) ms at 25", line)
        if m:
            totals[m.group(1)] = (float(m.group(3)), float(m.group(2)))
    names = [n for n in totals if n in small and n in large]

    def remaining(name, fixed):
        n, n4 = totals[name]
        keep = [g for g in GROUPS if g not in fixed]
        return sum(small[name][g] for g in keep) * n, sum(large[name][g] for g in keep) * n4

    def passes(name, fixed):
        rn, r4 = remaining(name, fixed)
        return r4 <= 1.5 * rn + 5

    if "--table" in sys.argv:
        print(f"{'call':44s} {'N':>6s} {'4N':>7s} |" + "".join(f" {g}:N/4N ms    " for g in GROUPS[:4]))
        for name in sorted(names, key=lambda n: -totals[n][1]):
            n, n4 = totals[name]
            cells = "".join(f" {small[name][g] * n:5.1f}/{large[name][g] * n4:6.1f}" for g in GROUPS[:4])
            print(f"{name:44s} {n:6.1f} {n4:7.1f} |{cells}")
        return
    order = ["P1", "P2", "P3", "P4"]
    done = set()
    for k in range(1, 5):
        fixed = set(order[:k])
        now = {n for n in names if passes(n, fixed)}
        new = sorted(now - done)
        print(f"after {'+'.join(order[:k])}: {len(now)} of {len(names)} constant, {len(new)} new")
        print("   " + ", ".join(new))
        done = now
    print("never constant with P1-P4:", sorted(set(names) - done))
    print("\nsingle PRs and pairs:")
    for r in range(1, 4):
        for combo in itertools.combinations(order, r):
            print(f"   {'+'.join(combo):12s} {sum(passes(n, set(combo)) for n in names)}")
    print("\nresidual 4N ms after all four (largest):")
    for name in sorted(names, key=lambda n: -remaining(n, set(order))[1])[:8]:
        rn, r4 = remaining(name, set(order))
        print(f"   {name:44s} N {rn:5.1f}  4N {r4:5.1f}")


main()
