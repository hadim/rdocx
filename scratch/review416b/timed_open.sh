#!/bin/bash
S=/private/tmp/claude-501/-Users-hadim-Code-libs-rdocx/08057ee4-afc5-4751-aca1-2668752380de/scratchpad
R=$S/review416b
until mkdir $S/timing.lock 2>/dev/null; do sleep 2; done
while pgrep -x rustc >/dev/null; do sleep 2; done
{
  uptime
  for round in 1 2; do
    for build in rel403-pr2 rel416; do
      $S/review403-venv/bin/python -I -c "import sys; sys.path.insert(0, '$S/$build'); sys.argv=['x','$R/deck200.pptx','15','$build']; exec(open('$R/open_time.py').read())"
    done
  done
  uptime
} > $R/open_time.txt 2>&1
rmdir $S/timing.lock
echo DONE >> $R/open_time.txt
