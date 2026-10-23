"""Inclusive sample counts per function from a macOS `sample` call graph: python summarize_sample.py FILE [TOP] [FILTER]."""
import re
import sys
from collections import defaultdict

LINE = re.compile(r"^(?P<indent>[\s+!:|]*?)(?P<count>\d+) (?P<name>.+?)\s+\(in (?P<lib>[^)]+)\)")


def main():
    path = sys.argv[1]
    top = int(sys.argv[2]) if len(sys.argv) > 2 else 40
    pattern = re.compile(sys.argv[3]) if len(sys.argv) > 3 else None
    text = open(path, errors="replace").read()
    graph = text.split("Call graph:", 1)[1].split("Total number in stack", 1)[0]
    inclusive = defaultdict(int)
    stack = []  # (depth, name)
    total = 0
    for line in graph.splitlines():
        match = LINE.match(line)
        if not match:
            continue
        depth = len(match.group("indent"))
        count = int(match.group("count"))
        name = re.sub(r"::h[0-9a-f]{16}$", "", match.group("name").strip())
        name = re.sub(r"\s+\+ \d+.*$", "", name)
        while stack and stack[-1][0] >= depth:
            stack.pop()
        if not stack:
            total += count
        if all(other != name for _, other in stack):
            inclusive[name] += count
        stack.append((depth, name))
    rows = sorted(inclusive.items(), key=lambda item: -item[1])
    if pattern:
        rows = [row for row in rows if pattern.search(row[0])]
    print(f"total samples {total}")
    for name, count in rows[:top]:
        print(f"{100 * count / max(total, 1):5.1f}%  {count:6d}  {name[:150]}")


main()
