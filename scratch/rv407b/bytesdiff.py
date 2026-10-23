"""Print the reopened document.xml difference of one guard407 case and step under main and branch."""
import difflib
import json
import re
import sys

base = json.load(open(sys.argv[1]))
new = json.load(open(sys.argv[2]))
key, step = sys.argv[3], int(sys.argv[4])
b = base[key]["steps"][step][2]["reopen"][1]
n = new[key]["steps"][step][2]["reopen"][1]


def split(x):
    return re.sub(r"(<w:(?:p|tbl|sdt|tr|tc)[ >])", r"\n\1", x).splitlines()


for line in difflib.unified_diff(split(b), split(n), "main", "branch", lineterm="", n=0):
    print(line[:400])
