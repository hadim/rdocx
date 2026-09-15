# S72 sprint review, pass 18

**Reviewed**: `sprint/s72` at `bb63d89b24bd` against merge base
`5fdbfc578adf2e5a1ceca0a3f0cee601ece44746`, 174 files, 33,860
insertions, 1,479 deletions, 35,339 changed lines, crates: `oxml-layout`,
`oxml-pdf`, `rdocx-cli`, `rdocx-layout`, `rdocx-oxml`, `rdocx-py`,
`rdocx`, `rpptx-py`, `rpptx`
**Review boundary**: scheduled final integration boundary after F-X092 and
F-X094f release preparation. The sprint-running session extends the configured
three-pass limit through pass 18 for the explicit reason `scheduled final
integration boundary`. Prior clean passes covered the earlier dependency
prefixes.
**Verdict**: 0 blocking, 1 should-fix, 3 previously recorded nice-to-have

## Blocking

None. Count: 0.

## Should-fix

### S72-R18-1, sequencing note claims a PDF baseline change that did not occur

`docs/sprints/CURRENT_SPRINT.md:74`

The sequencing note says F-X092 exclusively owns a declared PDF baseline
change. The approved plan instead requires the 49-entry hash harness to remain
unchanged at `.claude/plans/F-X092-design.md:69`, and the completed delivery
record confirms 49 of 49 unchanged at `docs/sprints/AS_BUILT.md:13496`.
Leaving the stale claim makes the sprint record contradict both its contract
and its verification evidence. Replace it with the actual sequencing reason,
which is ownership of the PDF writer and logical extraction boundary.

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

F-X092 keeps logical extraction planning inside the existing PDF writer. The
bounded span at `crates/oxml-pdf/src/writer.rs:1076` augments rich positioned
runs without reordering paint operations or opening a second document model.
The source-built gate at `crates/rdocx/src/document.rs:26131` proves exact
logical extraction for large Word and PowerPoint documents under pinned
Poppler 26.01.0 while preserving the pre-change raster digests.

F-X094f adds the release contract around the already reviewed Python bindings.
Its backlog contract explicitly leaves tagging, publication, external
comments, and Issue 76 closure behind fresh `/release py-v0.13.1` approval at
the reviewed SHA at `docs/hld/14-development-backlog.md:4872`. It does not
publish during sprint preparation.

The integrated sprint changes no Cargo manifest, adds no cross-family mutable
state, and keeps the lower-level package boundaries intact. Full workspace,
deterministic rendering, package, binding, typing, and supply-chain gates pass
at the completed implementation SHA.

## Milestone gate

The M23 end gate still does not hold. It requires all five private references
to be generated from a blank public facade, reopen without repair, meet the
package and deterministic visual requirements, reproduce identical bytes, and
report no unexplained preservation-only fallback at
`docs/hld/14-development-backlog.md:2214`. Seven planned M23 stories remain
pending outside S72, and F-X094f remains in progress until the separately
approved Python release completes at `docs/sprints/CURRENT_SPRINT.md:58`.

The completed S72 implementation satisfies its narrower pre-release contract.
The PDF gate at `docs/sprints/CURRENT_SPRINT.md:101` is covered by the exact
large-document extraction and raster test. The release preparation gate
validates fresh Python 3.9 and 3.12 wheels, strict typing, stub parity, and the
exact distribution contract without claiming that publication already
occurred.

## Not found

- **interaction**: zero findings. The PDF extraction span stays inside the
  existing writer and the Python release machinery consumes reviewed binding
  surfaces.
- **duplication**: zero findings. No second layout engine, PDF writer, package
  graph, document tree, or release inventory was added.
- **layering**: zero findings. No Cargo manifest changed in the sprint and no
  forbidden lower-to-facade dependency was introduced.
- **harness**: zero findings. All 49 entries match and no baseline moved.
- **gate**: zero findings for the completed pre-release contract. Publication
  remains deliberately gated by separate approval.
- **docs**: one should-fix finding. The sequencing note contradicts the
  approved unchanged-harness contract.
- **deps**: zero findings. No dependency declaration changed.
- **surface**: zero findings. Public additions are bounded by the approved
  issue-derived stories and their exact integration tests.

## Checks

- `cargo fmt --all --check`, passed.
- `cargo clippy --workspace --all-targets --all-features --exclude rdocx-py
  --exclude rpptx-py -- -D warnings`, passed.
- Changed-crate and full workspace tests passed with pinned LibreOffice,
  Poppler, and corpus tooling.
- `python3 scripts/hash_harness.py --check`, passed 49 of 49 with no baseline
  change.
- `python3 scripts/prose_check.py`, passed with 0 violations before this review
  file was added.
- `python3 scripts/sync_agent_skills.py --check`, passed with 26 skills in
  sync.
- `python3 -m unittest scripts.test_sprint_workflow`, passed 116 tests with 2
  skipped.
- `cargo test -p oxml-layout --no-default-features`, passed 102 tests and 3
  doctests.
- `cargo check --target wasm32-unknown-unknown -p rdocx-wasm -p rpptx-wasm`,
  passed.
- Rustdoc with warnings denied and all README doctests passed.
- The exact 22-crate publish dry run passed, with every archive below 10 MiB.
- `cargo deny check`, passed with the configured duplicate and unmatched-license
  warnings.
- Fresh `rdocx` and `rpptx` `cp39-abi3` wheels passed 53 binding tests on both
  Python 3.9 and 3.12, together with strict mypy and stubtest for all 11
  extension modules.
