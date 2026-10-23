#!/bin/sh
# Run a timed command under the shared timing protocol.
until mkdir /private/tmp/claude-501/-Users-hadim-Code-libs-rdocx/08057ee4-afc5-4751-aca1-2668752380de/scratchpad/timing.lock 2>/dev/null; do sleep 2; done
until ! pgrep -x rustc >/dev/null; do sleep 2; done
"$@"; status=$?
rmdir /private/tmp/claude-501/-Users-hadim-Code-libs-rdocx/08057ee4-afc5-4751-aca1-2668752380de/scratchpad/timing.lock
exit $status
