#!/bin/bash
S=/private/tmp/claude-501/-Users-hadim-Code-libs-rdocx/08057ee4-afc5-4751-aca1-2668752380de/scratchpad
cd /Users/hadim/Code/libs/rdocx/worktrees/403-commit
while [ -d $S/timing.lock ]; do sleep 10; done
RUST_MIN_STACK=8388608 cargo test -p rpptx --no-fail-fast > $S/pr2-rpptx-test2.log 2>&1; echo EXIT $? >> $S/pr2-rpptx-test2.log
while [ -d $S/timing.lock ]; do sleep 10; done
VIRTUAL_ENV=$S/venv403 PATH=$S/venv403/bin:$PATH maturin develop --uv -m crates/rpptx-py/Cargo.toml > $S/pr2-debug-build2.log 2>&1
$S/venv403/bin/python -m pytest crates/rpptx-py/tests -q -p no:cacheprovider > $S/pr2-pytest2.log 2>&1; echo EXIT $? >> $S/pr2-pytest2.log
