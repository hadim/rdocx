#!/bin/bash
# Split every sample of calls.txt: python split.py on its pymethod root.
S=/private/tmp/claude-501/-Users-hadim-Code-libs-rdocx/08057ee4-afc5-4751-aca1-2668752380de/scratchpad
D=$S/docxplan
COUNT=${1:-100}
while IFS=$'\t' read -r name class; do
  f=$D/samples/$name.$COUNT.txt
  [ -s "$f" ] || continue
  attr=${name#*.}
  root="${class}[0-9]+___pymethod_(set_set_)?${attr}__"
  echo "=== $name"
  python3 $D/split.py "$f" "$root" --top 5
done < $D/calls.txt
