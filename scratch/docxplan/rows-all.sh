#!/bin/bash
# PR shares of every growing call: rows-all.sh DIR COUNT > out
S=/private/tmp/claude-501/-Users-hadim-Code-libs-rdocx/08057ee4-afc5-4751-aca1-2668752380de/scratchpad
D=$S/docxplan
DIR=$1
COUNT=$2
while read -r name; do
  cls=${name%%.*}
  case $cls in
    FootnoteEndnote) class=PyNote ;;
    *) class=Py$cls ;;
  esac
  attr=${name#*.}
  f=$D/$DIR/$name.$COUNT.txt
  [ -s "$f" ] || { echo "MISSING $name"; continue; }
  python3 $D/split.py "$f" "${class}[0-9]+___pymethod_(set_set_)?${attr}__" --pr "$name" | grep '^PR'
done < $D/growing135.txt
