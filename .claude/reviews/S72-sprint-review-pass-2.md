# S72 sprint review, pass 2

**Reviewed**: `sprint/s72` at `69b0125aa743` against
`5fdbfc578adf2e5a1ceca0a3f0cee601ece44746`, 54 files, 9,437 changed lines,
crates: `rdocx-oxml`, `rdocx`, `rdocx-py`
**Verdict**: 0 blocking, 0 should-fix, 1 nice-to-have

## Blocking

None. Count: 0.

## Should-fix

None. Count: 0.

## Nice-to-have

### N1, the final F-253 review has an extra blank line at EOF
`.claude/reviews/F-253-all-pass-21.md:46`

The integrated sprint range ends this file with two newline-only lines, so
`git diff --check 5fdbfc578adf2e5a1ceca0a3f0cee601ece44746..HEAD`
reports `new blank line at EOF`. This does not affect behavior or any required
gate, but removing the extra blank line would keep the sprint delta
whitespace-clean.

## Milestone gate

The M23 end gate requires all five private references to be generated from a
blank public facade, reopen without repair, match required package semantics
and reviewed deterministic visual thresholds, reproduce identical DOCX bytes,
and report no unexplained preservation-only fallback
(`docs/hld/14-development-backlog.md:2214`). The end gate does not yet hold at
this dependency-prefix checkpoint. F-251, F-254, F-255, F-252, and F-256 remain
pending (`docs/sprints/CURRENT_SPRINT.md:40`).

Both completed prefix gates hold. F-253's integration gate applies one generic
resolver and mutation across body, cell, text-box, header, footer, footnote,
endnote, and comment owners with uniform error assertions
(`crates/rdocx/tests/integration_test.rs:94`). Its round-trip owner-order test
begins at `crates/rdocx/tests/integration_test.rs:186`, and its byte-atomic
invalid-location regression begins at
`crates/rdocx/tests/regression_test.rs:1687`.

F-250's round-trip gate begins at
`crates/rdocx/tests/integration_test.rs:415` and proves the three surviving
section owners, orientation order, retained raw XML, and exact story
relationships after reopen. The cross-feature regression at
`crates/rdocx/tests/integration_test.rs:724` removes section owners and observes
the surviving and pruned header owners through F-253's public `stories()` model.
The delivery record ties full verification to each integrated feature SHA and
records unchanged 49-of-49 harness results
(`docs/sprints/AS_BUILT.md:12819`, `docs/sprints/AS_BUILT.md:12865`). A current
hash-harness check also reported all 49 entries unchanged.

## Not found

- **interaction**: section insertion and removal retain or prune story owners
  through the same relationship graph that `stories()` discovers. The direct
  shared-owner assertions at `crates/rdocx/tests/integration_test.rs:737` and
  `crates/rdocx/tests/integration_test.rs:754` produced no cross-feature
  mismatch.
- **duplication**: F-253 keeps one story source resolver at
  `crates/rdocx/src/document.rs:8945`, while F-250's ordered section iterator at
  `crates/rdocx/src/document.rs:12533` owns only section ordering. No competing
  story tree, section tree, location resolver, or staged commit path was added.
- **layering**: no manifest changed in the sprint range. `rdocx-oxml` gained
  grammar admission helpers consumed upward by `rdocx`, with no dependency on
  an `rdocx-*` or `rpptx-*` facade crate.
- **harness**: no baseline file changed. Both completion records declare the
  unchanged result (`docs/sprints/AS_BUILT.md:12826`,
  `docs/sprints/AS_BUILT.md:12872`), and the current harness reproduced it.
- **gate**: the completed DOCX-015 and DOCX-018 rows have implementation
  evidence and no owner, while the adjacent incomplete rows retain live owners
  (`docs/hld/02-scope-and-non-goals.md:222`). The focused owner-invariant test
  passed.
- **docs**: the combined architecture describes one facade-owned story model
  followed by one ordered section view over existing owners
  (`docs/hld/03-architecture.md:1015`). The package, binding, testing, risk, and
  backlog sections match the integrated mechanisms and public surface.
- **deps**: no dependency, feature declaration, crate, or module changed in the
  sprint range.
- **surface**: the exported story values and operations match the native API at
  `docs/hld/10-bindings-spec.md:320`. The exported section handles and
  operations match the additive native API at
  `docs/hld/10-bindings-spec.md:333`. Python receives only exhaustive native
  error classification, not an unplanned story or section API.

## Checks

- `cargo fmt --all --check`, passed.
- `cargo check -p rdocx --all-targets`, passed.
- `cargo test -p rdocx --test integration_test`, passed 214 tests. Two
  pre-sprint tests could not run without the pinned LibreOffice executable, and
  three external-oracle tests were ignored as declared.
- `cargo test -p rdocx --test regression_test invalid_story_locations_are_atomic -- --exact`,
  passed.
- `cargo test -p rdocx-py`, passed two tests and its doc-test target.
- `python3 scripts/hash_harness.py --check`, passed 49 of 49.
- `python3 -m unittest scripts.test_sprint_workflow.SprintWorkflowTests.test_every_incomplete_modern_docx_row_has_one_live_owner`, passed.
- `python3 scripts/prose_check.py`, passed.
- `python3 scripts/sync_agent_skills.py --check`, passed with 26 skills in sync.
- The library portion of `cargo test -p rdocx` passed 447 tests and ignored six,
  then stopped at one pre-sprint rasterizer-version assertion because this shell
  does not expose the pinned rasterizer command.
