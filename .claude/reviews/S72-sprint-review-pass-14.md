# S72 sprint review, pass 14

**Reviewed**: `sprint/s72` at `06699b1c8131` against merge base
`5fdbfc578adf2e5a1ceca0a3f0cee601ece44746`, 124 files, 27,610
insertions, 2,668 deletions, 30,278 changed lines, crates: `oxml-layout`,
`rdocx-layout`, `rdocx-oxml`, `rdocx-py`, `rdocx-cli`, `rdocx`
**Review boundary**: scheduled dependency-prefix boundary after F-256. The
sprint-running session extends the configured three-pass limit through pass 14
because prior clean passes covered earlier dependency prefixes.
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

The new fragment surface is the concrete package-authoritative owner required
to cross documents. `DocumentFragment` and its non-exhaustive conflict policy
are declared at `crates/rdocx/src/document.rs:404`, range capture validates one
nonempty main-body owner at `crates/rdocx/src/document.rs:476`, and import
stages the complete candidate at `crates/rdocx/src/document.rs:11584`.
This does not relax F-254 same-owner movement. The ordinary content path still
rejects section-owning or relationship-unsafe fragments before publication at
`crates/rdocx/src/document.rs:11570`.

F-X090, F-252, F-X093, and F-256 preserve compatible drawing ownership. The
source accepts producer drawing identities per physical part, while imported
drawings receive fresh package-global identities. Comparison and related-story
staging keep exact wrappers rather than reconstructing their typed projection.
The repeated dependency-rich import test exercises pictures and charts at
`crates/rdocx/tests/regression_test.rs:26748`, while the malformed closure unit
test exercises the private relationship parser at
`crates/rdocx/src/document.rs:31443`.

F-X091 and F-X093 preserve namespace authority through the same physical
package boundary used by F-256. Selected raw body and comment XML therefore
keeps producer syntax while mapped expanded-name attributes change. The
conflict matrix covers equivalent reuse and rename behavior at
`crates/rdocx/tests/regression_test.rs:26905`, and atomic unsupported closure
coverage begins at `crates/rdocx/tests/regression_test.rs:27027`.

Delivery records agree. DOCX-021 is complete and ownerless at
`docs/hld/02-scope-and-non-goals.md:228`, both live sprint trackers mark F-256
done at `docs/sprints/CURRENT_SPRINT.md:52` and
`docs/sprints/BACKLOG.md:471`, and the workflow completion assertion removes
256 from the incomplete-owner set at `scripts/test_sprint_workflow.py:8311`.

## Milestone gate

The M23 end gate requires all five private references to be generated from a
blank public facade, reopen without repair, match package semantics and
reviewed deterministic visual thresholds, reproduce identical DOCX bytes, and
report no unexplained preservation-only fallback
(`docs/hld/14-development-backlog.md:2214`). It does not yet hold. The current
sprint still has F-X092 and F-X094a through F-X094f pending at
`docs/sprints/CURRENT_SPRINT.md:53`.

The completed prefix does satisfy its declared fragment gate. The dependency
rich repeated import, deterministic conflict-policy, malformed-closure, and
atomic unsupported-dependency tests all pass. The full workspace gate passes
at the reviewed state, the hash harness matches all 49 entries, the exact
22-crate dry run succeeds, and no archive exceeds 10 MiB. This does not claim
the unfinished issue wave or M23 end gate.

## Not found

- **interaction**: zero findings. Producer-part drawing scope, physical package
  authority, related-story lifecycle, comparison staging, and cross-document
  remapping retain distinct responsibilities and share one staged commit rule.
- **duplication**: zero findings. F-256 generalizes the established rich-merge
  dependency importer rather than introducing a second package-closure engine.
- **layering**: zero findings. No Cargo manifest changed, and no forbidden
  lower-to-facade dependency was introduced.
- **harness**: zero findings. No baseline changed and all 49 entries match.
- **gate**: zero findings for the completed prefix. The full sprint and M23 end
  gates remain explicitly unfinished.
- **docs**: zero findings. DOCX-021, all nine planned HLD files, the delivery
  ledgers, and the completion-owner assertion describe the current boundary.
- **deps**: zero findings. No dependency declaration changed.
- **surface**: zero findings. `DocumentFragment` and
  `FragmentConflictPolicy` are the approved additive pre-1.0 native Rust
  surface. Python, WASM, and CLI bindings remain deliberately unchanged here.

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
  skipped after the completed-owner assertion was aligned.
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
