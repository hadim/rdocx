#!/bin/sh
# usage: run-fuzz.sh WORKER  (chunks of 20 seeds, 10 steps; waits while the timing lock exists)
S=/private/tmp/claude-501/-Users-hadim-Code-libs-rdocx/08057ee4-afc5-4751-aca1-2668752380de/scratchpad
cd $S/rv407
w=$1
for c in 0 1 2 3 4 5 6 7 8 9; do
  start=$(( (c*2 + w) * 20 + 100 ))
  end=$(( start + 20 ))
  while [ -d $S/timing.lock ]; do sleep 2; done
  PYTHONPATH=$S/rv407/debug $S/venv407/bin/python diff407.py $start $end 10 > d-$start.json 2>d-$start.err
  while [ -d $S/timing.lock ]; do sleep 2; done
  PYTHONPATH=$S/baseline $S/venv407/bin/python diff407.py $start $end 10 > b-$start.json 2>b-$start.err
  python3 cmp407.py b-$start.json d-$start.json > c-$start.txt
  tail -1 c-$start.txt
done
