#!/bin/bash
# Profile each call of calls.txt at PARAGRAPHS paragraphs, taking the timing lock for each run only.
S=/private/tmp/claude-501/-Users-hadim-Code-libs-rdocx/08057ee4-afc5-4751-aca1-2668752380de/scratchpad
D=$S/docxplan
COUNT=${1:-100}
SECONDS_EACH=${2:-8}
mkdir -p $D/samples
while IFS=$'\t' read -r name class; do
  [ -z "$name" ] && continue
  out=$D/samples/$name.$COUNT.txt
  [ -s "$out" ] && continue
  until mkdir $S/timing.lock 2>/dev/null; do sleep 2; done
  while pgrep -x rustc >/dev/null; do sleep 2; done
  $S/venvdocx/bin/python $D/profdocx.py "$name" $COUNT $SECONDS_EACH "$out" 2>&1 | tail -3
  rmdir $S/timing.lock
  sleep 1
done < $D/calls.txt
echo "profile-all done"
