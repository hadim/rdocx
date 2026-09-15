# S72 sprint review, pass 16

**Reviewed**: `sprint/s72` at `1f4bc7b3d371` against merge base
`5fdbfc578adf2e5a1ceca0a3f0cee601ece44746`, 142 files, 31,151
insertions, 2,752 deletions, 33,903 changed lines, crates: `oxml-layout`,
`rdocx-layout`, `rdocx-oxml`, `rdocx-py`, `rdocx-cli`, `rdocx`
**Review boundary**: scheduled dependency-prefix boundary after F-X094c. The
sprint-running session extends the configured three-pass limit through pass 16
for the explicit reason `scheduled dependency-prefix boundary`. Prior clean
passes covered the earlier dependency prefixes.
**Verdict**: 0 blocking, 0 should-fix, 3 previously recorded nice-to-have

## Blocking

None. Count: 0.

## Should-fix

None. Count: 0.

## Nice-to-have

### N1, the F-253 final review retains its recorded trailing blank line
`.claude/reviews/F-253-all-pass-21.md:46`

This review-only whitespace observation remains nonbehavioral and safe to
leave recorded.

### N2, the pass-4 sprint review retains its trailing blank line
`.claude/reviews/S72-sprint-review-pass-4.md:125`

This review-only whitespace observation remains nonbehavioral and safe to
leave recorded.

### N3, the pass-5 sprint review retains its trailing blank line
`.claude/reviews/S72-sprint-review-pass-5.md:120`

This review-only whitespace observation remains nonbehavioral and safe to
leave recorded.

## Interaction audit

F-X094c projects the existing native comparison, comment, deterministic
layout, and TOC operations through the established Python `Document`. It does
not introduce a parallel document model. All eight value records are registered
beside the existing binding types at `crates/rdocx-py/src/lib.rs:88`, exported
through the package at `crates/rdocx-py/python/rdocx/__init__.py:28`, and
described by matching installed stubs at
`crates/rdocx-py/python/rdocx/_rdocx.pyi:164`.

The mutation boundary composes with the sprint's native staging work.
Comparison performs the native staged operation while detached and advances
the binding revision only when serialized package state changes at
`crates/rdocx-py/src/document.rs:364`. Comment addition, reply, resolution, and
removal advance revisions only after their native calls succeed at
`crates/rdocx-py/src/document.rs:407`. TOC rebuilding applies the same
serialized-state rule at `crates/rdocx-py/src/document.rs:507`. The installed
binding gate proves successful invalidation, failed-operation atomicity,
save-reopen behavior, typed layout, and no-op stability at
`crates/rdocx-py/tests/test_core.py:110`.

Long-running work uses the existing PyO3 detachment pattern. Layout snapshots
come from the native deterministic layout and its result-local body fragment
sidecar at `crates/rdocx-py/src/document.rs:463`. The dedicated scheduler gates
exercise serialization, comparison, layout, and TOC calls at
`crates/rdocx-py/tests/test_rendering_threads.py:271`. This preserves the
F-X094b layout ownership boundary and adds no alternate geometry computation.

Delivery records agree. The sprint and backlog mark F-X094c complete and
ownerless at `docs/sprints/CURRENT_SPRINT.md:55` and
`docs/sprints/BACKLOG.md:638`. Its integrated behavior, wheel evidence, and
unchanged harness are recorded at `docs/sprints/AS_BUILT.md:13338`.

## Milestone gate

The M23 end gate requires all five private references to be generated from a
blank public facade, reopen without repair, match package semantics and
reviewed deterministic visual thresholds, reproduce identical DOCX bytes, and
report no unexplained preservation-only fallback
(`docs/hld/14-development-backlog.md:2214`). It does not yet hold. F-X094d,
F-X094e, F-X094f, and F-X092 remain pending at
`docs/sprints/CURRENT_SPRINT.md:56`. Issue 76 also remains open until the
remaining Python surfaces and the separately approved release path complete.

The completed prefix satisfies its narrower binding gate. The HLD requires
frozen snapshots, GIL release, one revision advance after successful mutation,
and no candidate or revision advance after native failure at
`docs/hld/10-bindings-spec.md:214`. The installed binding test covers that
contract at `crates/rdocx-py/tests/test_core.py:110`, and the dedicated thread
tests cover all four detached operations at
`crates/rdocx-py/tests/test_rendering_threads.py:271`. The exact cp39-abi3 wheel
passed 41 tests, strict mypy, and stubtest. The full workspace gate passes at
the reviewed state, the hash harness matches all 49 entries, the exact 22-crate
dry run succeeds, and no archive exceeds 10 MiB. This does not claim the
unfinished issue wave or M23 end gate.

## Not found

- **interaction**: zero findings. The Python projection uses the existing
  native staged mutations, deterministic layout sidecar, and revision counter.
- **duplication**: zero findings. No second comparison, comment, layout, TOC,
  or package mutation implementation was added.
- **layering**: zero findings. No Cargo manifest changed, and no forbidden
  lower-to-facade dependency was introduced.
- **harness**: zero findings. No baseline changed and all 49 entries match.
- **gate**: zero findings for the completed prefix. The full sprint, Issue 76,
  and M23 end gates remain explicitly unfinished.
- **docs**: zero findings. All four HLD files named by the design plan and the
  binding README describe the implemented boundary.
- **deps**: zero findings. No dependency declaration changed.
- **surface**: zero findings. The eight frozen records and eight `Document`
  members are the bounded surface named by F-X094c, with future all-story work
  explicitly excluded at `docs/hld/14-development-backlog.md:4813`.

## Checks

- `cargo fmt --all --check`, passed.
- `cargo clippy --workspace --all-targets --all-features --exclude rdocx-py
  --exclude rpptx-py -- -D warnings`, passed.
- `cargo test --workspace --all-features --exclude rdocx-py --exclude
  rpptx-py`, passed with pinned LibreOffice, Poppler, and corpus tooling.
- `python3 scripts/hash_harness.py --check`, passed 49 of 49.
- `python3 scripts/prose_check.py`, passed with 0 violations before this review
  file was added.
- `python3 scripts/sync_agent_skills.py --check`, passed with 26 skills in
  sync.
- `python3 -m unittest scripts.test_sprint_workflow`, passed 115 tests with 2
  skipped.
- `cargo test -p oxml-layout --no-default-features`, passed 102 tests and 3
  doctests.
- `cargo check --target wasm32-unknown-unknown -p rdocx-wasm -p rpptx-wasm`,
  passed.
- `RUSTDOCFLAGS='-D warnings' cargo doc --workspace --no-deps`, passed.
- `python3 scripts/readme_doctests.py`, passed 27 README and 22 publishable
  package inventories.
- `cargo publish --workspace --dry-run` with the 22 reviewed local patches,
  passed, and every archive remained below 10 MiB.
- `cargo deny check`, passed with the recorded duplicate and unmatched-license
  warnings.
- The clean installed `cp39-abi3` wheel passed 41 binding tests, strict mypy,
  and stubtest for all six extension modules.
