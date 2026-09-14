# S72 sprint review, pass 15

**Reviewed**: `sprint/s72` at `a657e404611e` against merge base
`5fdbfc578adf2e5a1ceca0a3f0cee601ece44746`, 132 files, 30,038
insertions, 2,741 deletions, 32,779 changed lines, crates: `oxml-layout`,
`rdocx-layout`, `rdocx-oxml`, `rdocx-py`, `rdocx-cli`, `rdocx`
**Review boundary**: scheduled dependency-prefix boundary after F-X094a and
F-X094b. The sprint-running session extends the configured three-pass limit
through pass 15 for the explicit reason `scheduled dependency-prefix boundary`.
Prior clean passes covered the earlier dependency prefixes.
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

F-X094a and F-X094b extend one existing Clap command graph rather than adding
a parallel executable or dispatch layer. The collaboration commands enter the
same dispatch at `crates/rdocx-cli/src/main.rs:401`, while structured layout
enters at `crates/rdocx-cli/src/main.rs:348`. Existing command syntax remains
additive.

All new document mutations share one complete-document publication boundary.
Comment addition begins from a reopened `Document` at
`crates/rdocx-cli/src/commands.rs:595`, revision resolution validates its
selector before mutation at `crates/rdocx-cli/src/commands.rs:706`, and the
common publisher serializes before staged publication at
`crates/rdocx-cli/src/commands.rs:866`. Guarded replacement checks its exact
count before calling that publisher at `crates/rdocx-cli/src/commands.rs:942`.
The two CLI stories therefore do not introduce competing output semantics.

The F-X094b layout sidecar does not alter the shared positioned output used by
rendering. `WordLayoutResult` owns the private fragment vectors and exposes
them through one indexed accessor at `crates/rdocx-layout/src/lib.rs:57`.
Fresh and reusable-engine provenance paths both take the result-local sidecar
at `crates/rdocx-layout/src/lib.rs:141` and
`crates/rdocx-layout/src/lib.rs:163`. The warm restart equality test at
`crates/rdocx-layout/src/engine.rs:18170` proves cached pagination returns the
same fragments as fresh pagination.

Delivery records agree. The sprint table marks both issue slices complete and
ownerless at `docs/sprints/CURRENT_SPRINT.md:53`, the backlog carries the same
state at `docs/sprints/BACKLOG.md:636`, and the integrated summaries are
recorded at `docs/sprints/AS_BUILT.md:13260` and
`docs/sprints/AS_BUILT.md:13298`.

## Milestone gate

The M23 end gate requires all five private references to be generated from a
blank public facade, reopen without repair, match package semantics and
reviewed deterministic visual thresholds, reproduce identical DOCX bytes, and
report no unexplained preservation-only fallback
(`docs/hld/14-development-backlog.md:2214`). It does not yet hold. F-X094c
through F-X094f and F-X092 remain pending at
`docs/sprints/CURRENT_SPRINT.md:55`.

The completed prefix satisfies its narrower gates. The exact compiled-binary
collaboration contract is exercised at
`crates/rdocx-cli/tests/integration.rs:691`. Structured text, layout, and
guarded replacement are exercised at
`crates/rdocx-cli/tests/integration.rs:1120`. Page-spanning and nontext body
fragments are independently exercised at
`crates/rdocx-layout/src/engine.rs:18091` and
`crates/rdocx-layout/src/engine.rs:18123`. The full workspace gate passes at
the reviewed state, the hash harness matches all 49 entries, the exact 22-crate
dry run succeeds, and no archive exceeds 10 MiB. This does not claim the
unfinished issue wave or M23 end gate.

## Not found

- **interaction**: zero findings. Collaboration, structured inspection, and
  guarded replacement share one command graph and one atomic publisher, while
  the layout sidecar remains result-local.
- **duplication**: zero findings. The new commands reuse the native document
  facades and the existing staged output utility.
- **layering**: zero findings. No Cargo manifest changed, and no forbidden
  lower-to-facade dependency was introduced.
- **harness**: zero findings. No baseline changed and all 49 entries match.
- **gate**: zero findings for the completed prefix. The full sprint and M23 end
  gates remain explicitly unfinished.
- **docs**: zero findings. The four planned HLD files, CLI and layout README
  contracts, and shared delivery records describe the implemented boundary.
- **deps**: zero findings. No dependency declaration changed.
- **surface**: zero findings. `WordBodyLayoutFragment` and its result accessor
  are the approved additive pre-1.0 native Rust surface. The schema-1 CLI
  records match the two issue-slice contracts.

## Checks

- `cargo fmt --all --check`, passed.
- `cargo clippy --workspace --all-targets --all-features --exclude rdocx-py
  --exclude rpptx-py -- -D warnings`, passed.
- `cargo test --workspace --all-features --exclude rdocx-py --exclude
  rpptx-py`, passed with pinned LibreOffice and PPTX corpus paths.
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
