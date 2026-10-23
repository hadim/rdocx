#!/bin/bash
S=/private/tmp/claude-501/-Users-hadim-Code-libs-rdocx/08057ee4-afc5-4751-aca1-2668752380de/scratchpad
cd /Users/hadim/Code/libs/rdocx/worktrees/403-commit
wait_lock() { while [ -d $S/timing.lock ]; do sleep 10; done; }
G=$S/gates
mkdir -p $G
wait_lock; RUST_MIN_STACK=8388608 cargo test -p rpptx -p rpptx-oxml -p rpptx-cli -p oxml-opc --no-fail-fast > $G/crates.log 2>&1; echo EXIT $? >> $G/crates.log
wait_lock; pixi exec -s poppler -- env RUST_MIN_STACK=8388608 cargo test -p rpptx --test integration -- notes_pages_follow_master handouts_follow_master notes_pdf_skips notes_and_handout_export > $G/rpptx-poppler.log 2>&1; echo EXIT $? >> $G/rpptx-poppler.log
wait_lock; DYLD_LIBRARY_PATH=/Users/hadim/.local/share/uv/python/cpython-3.13.11-macos-aarch64-none/lib RUST_MIN_STACK=8388608 cargo test -p rpptx-py > $G/rpptx-py.log 2>&1; echo EXIT $? >> $G/rpptx-py.log
wait_lock; RUST_MIN_STACK=8388608 cargo test -p oxml-sml -p oxml-chart --no-fail-fast > $G/opc-users.log 2>&1; echo EXIT $? >> $G/opc-users.log
wait_lock; pixi exec -s poppler -- env RUST_MIN_STACK=8388608 cargo test -p rdocx --no-fail-fast > $G/rdocx.log 2>&1; echo EXIT $? >> $G/rdocx.log
wait_lock; cargo check --target wasm32-unknown-unknown -p rdocx-wasm -p rpptx-wasm --tests > $G/wasm.log 2>&1; echo EXIT $? >> $G/wasm.log
wait_lock; cargo clippy --workspace --all-targets --all-features -- -D warnings > $G/clippy.log 2>&1; echo EXIT $? >> $G/clippy.log
cargo fmt --all --check > $G/fmt.log 2>&1; echo EXIT $? >> $G/fmt.log
python3 scripts/prose_check.py > $G/prose.log 2>&1; echo EXIT $? >> $G/prose.log
python3 scripts/sync_agent_skills.py --check > $G/sync.log 2>&1; echo EXIT $? >> $G/sync.log
wait_lock; python3 scripts/hash_harness.py --check > $G/hash.log 2>&1; echo EXIT $? >> $G/hash.log
wait_lock; VIRTUAL_ENV=$S/venv403 PATH=$S/venv403/bin:$PATH maturin develop --uv -m crates/rpptx-py/Cargo.toml > $G/debug-build.log 2>&1
$S/venv403/bin/python -m pytest crates/rpptx-py/tests -q -p no:cacheprovider > $G/pytest.log 2>&1; echo EXIT $? >> $G/pytest.log
echo ALLDONE > $G/done
