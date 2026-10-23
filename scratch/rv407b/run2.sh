#!/bin/sh
# usage: run2.sh START END STEPS TAG
S=/private/tmp/claude-501/-Users-hadim-Code-libs-rdocx/08057ee4-afc5-4751-aca1-2668752380de/scratchpad
cd $S/rv407b
while [ -d $S/timing.lock ]; do sleep 2; done
PYTHONPATH=$S/rv407b/debug $S/venv407/bin/python fz2.py $1 $2 $3 > n-$4.json 2> n-$4.err
while [ -d $S/timing.lock ]; do sleep 2; done
PYTHONPATH=$S/baseline $S/venv407/bin/python fz2.py $1 $2 $3 > b-$4.json 2> b-$4.err
python3 $S/rv407/cmp407.py b-$4.json n-$4.json > c-$4.txt
tail -1 c-$4.txt
