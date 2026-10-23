#!/bin/bash
# Usage: timed403.sh <command...>; runs the command under the shared timing lock.
S=/private/tmp/claude-501/-Users-hadim-Code-libs-rdocx/08057ee4-afc5-4751-aca1-2668752380de/scratchpad
until mkdir "$S/timing.lock" 2>/dev/null; do sleep 5; done
trap 'rmdir "$S/timing.lock"' EXIT
while pgrep -x rustc >/dev/null; do sleep 5; done
"$@"
