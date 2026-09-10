# S72 sprint review, pass 4

**Reviewed**: `sprint/s72` at
`ea3c743863205c8c5c27a185a67b6c0d29529174` against merge base
`5fdbfc578adf2e5a1ceca0a3f0cee601ece44746`, 75 files, 14,484
insertions, 301 deletions, 14,785 changed lines, crates: `oxml-layout`,
`rdocx-layout`, `rdocx-oxml`, `rdocx-py`, `rdocx`
**Review boundary**: scheduled dependency-prefix boundary after F-254. This
pass is the explicit approved extension beyond the configured three-pass
default.
**Verdict**: 1 blocking, 0 should-fix, 1 previously recorded nice-to-have

## Blocking

### B1, generic paragraph removal bypasses section ownership cleanup
`crates/rdocx/src/document.rs:10235`

`remove_content_at` accepts every direct body paragraph, validates only its
fragment relationships, drains the paragraph, and reopens the package. A body
paragraph may own a non-final `w:sectPr`. Removing that paragraph through this
F-254 path bypasses F-250's `remove_section` behavior at
`crates/rdocx/src/document.rs:13788`, which materializes effective inherited
header and footer references and prunes only unreachable facade-owned stories.
For example, when the following final section inherits a header from the
removed boundary, the generic removal drops the only reference and leaves the
authored header part orphaned. Package reopen accepts that structurally valid
orphan, so staged publication does not reject the result.

This recreates the body-removal hazard that F-250 was designed to close
(`.claude/plans/F-250-design.md:14`) and violates the sprint requirement that
ordered removal never orphan a related story
(`docs/sprints/CURRENT_SPRINT.md:60`). Before the next dependency wave, generic
removal must either apply the F-250 section inheritance and ownership rules or
reject section-owning paragraphs atomically. The same boundary must be checked
for generic clone and move behavior, and a cross-feature regression must prove
that removing or relocating a section-owning paragraph cannot bypass the
section facade's graph invariants.

## Should-fix

None. Count: 0.

## Nice-to-have

### N1, the previously recorded trailing blank line remains
`.claude/reviews/F-253-all-pass-21.md:46`

This is the same non-behavioral observation recorded in passes 2 and 3. The
integrated range still ends the file with an extra newline-only line, so `git
diff --check 5fdbfc578adf2e5a1ceca0a3f0cee601ece44746..HEAD` reports `new
blank line at EOF`. It does not affect a gate and remains safe to leave
recorded.

## Milestone gate

The M23 end gate requires all five private references to be generated from a
blank public facade, reopen without repair, match required package semantics
and reviewed deterministic visual thresholds, reproduce identical DOCX bytes,
and report no unexplained preservation-only fallback
(`docs/hld/14-development-backlog.md:2214`). It does not yet hold. F-255,
F-252, and F-256 remain pending (`docs/sprints/CURRENT_SPRINT.md:41`).

The four completed story gates hold individually at the reviewed SHA. F-253's
common story mutation gate covers body, cell, text-box, header, footer, note,
and comment owners (`crates/rdocx/tests/integration_test.rs:102`). F-250's
round-trip gate proves ordered section mutation and exact story relationships
(`crates/rdocx/tests/integration_test.rs:915`), while its shared-story removal
gate begins at `crates/rdocx/tests/integration_test.rs:1224`. F-251's Word
differential gate begins at `crates/rdocx/tests/integration_test.rs:685`, and
its preservation and atomicity predicates begin at lines 422 and 521. F-254's
interleaved content-operation gate begins at
`crates/rdocx/tests/regression_test.rs:79`, with same-owner movement at line
135 and atomic rejection at line 215. These focused gates and the complete
regression binary passed in this review.

The integrated dependency prefix is not ready to advance while B1 remains.
F-255's declared F-253 and F-249 foundations are otherwise present. F-252's
section and story foundations are present, but the sprint sequence places it
after F-255 so rich header and footer content receives part-scoped
relationships (`docs/sprints/CURRENT_SPRINT.md:52`). F-256 remains correctly
unready until F-255 and F-252 complete its full F-246 through F-255 dependency
range (`docs/hld/14-development-backlog.md:2430`).

## Not found

- **interaction**: one blocking finding, B1. No additional interaction finding
  was found among F-253 traversal, F-250 section ownership, F-251 geometry and
  pagination, and F-254 content locations.
- **duplication**: zero findings. F-254 extends the F-253 source and location
  resolver and the existing staged document boundary. It adds no second story
  tree, section tree, package model, or competing mutation facade.
- **layering**: zero findings. No Cargo manifest changed. The typed Word grammar
  remains in `rdocx-oxml`, the facade remains in `rdocx`, and layout consumes
  section numbering without a forbidden lower-to-facade dependency.
- **harness**: zero findings. No baseline file changed, every completed
  AS_BUILT entry declares 49 of 49 unchanged, and the current harness reproduced
  all 49 entries.
- **gate**: zero additional findings. Every completed story's named predicate
  exists and passed. B1 is a missing cross-feature invariant rather than a
  false claim that the unfinished milestone gate holds.
- **docs**: zero findings. The completed capability rows, architecture,
  package, binding, testing, risk, rendering, and backlog text match the
  integrated public surfaces and retain F-255, F-252, and F-256 as live owners.
- **deps**: zero findings. No dependency, feature declaration, crate, module,
  or source file was added without a named consumer. The run-state wave order
  retains F-255 before F-252 and F-256.
- **surface**: zero findings. `StoryId`, `ContentLocation`, `ContentFragment`,
  the generic content operations, concrete section handles, checked geometry,
  and displayed page numbering are the additive native Rust APIs called for by
  F-250, F-251, F-253, and F-254. No unplanned Python, WASM, or CLI surface was
  added.

## Checks

- `cargo test -p rdocx --test regression_test`, passed 424 tests and ignored 4.
- The exact F-253, F-250, and F-251 integration predicates cited above passed.
- `cargo test -p rdocx-layout endnote_pages_continue_restarted_display_numbers_and_page_fields`,
  passed.
- `cargo check -p rdocx --all-targets`, passed.
- `cargo fmt --all --check`, passed.
- `python3 scripts/hash_harness.py --check`, passed 49 of 49.
- `python3 -m unittest scripts.test_sprint_workflow.SprintWorkflowTests.test_every_incomplete_modern_docx_row_has_one_live_owner`,
  passed.
- `python3 scripts/sync_agent_skills.py --check`, passed with 26 skills in sync.

