# S72 sprint review, pass 3

**Reviewed**: `sprint/s72` at
`cb13f3b1951b40223195ea98ebb378b5316ed7d3` against merge base
`5fdbfc578adf2e5a1ceca0a3f0cee601ece44746`, 68 files, 11,948 insertions,
296 deletions, 12,244 changed lines, crates: `oxml-layout`, `rdocx-layout`,
`rdocx-oxml`, `rdocx-py`, `rdocx`
**Verdict**: 0 blocking, 0 should-fix, 1 previously recorded nice-to-have

## Blocking

None. Count: 0.

## Should-fix

None. Count: 0.

## Nice-to-have

### N1, the pass-2 trailing blank-line observation remains unchanged
`.claude/reviews/F-253-all-pass-21.md:46`

This is the same nice-to-have recorded in pass 2, not a new actionable
finding. The integrated range still ends the file with an extra newline-only
line, so
`git diff --check 5fdbfc578adf2e5a1ceca0a3f0cee601ece44746..HEAD`
reports `new blank line at EOF`. It has no behavioral, documentation, harness,
or gate effect and remains safe to leave recorded.

## Milestone gate

The M23 end gate requires all five private references to be generated from a
blank public facade, reopen without repair, match required package semantics
and reviewed deterministic visual thresholds, reproduce identical DOCX bytes,
and report no unexplained preservation-only fallback
(`docs/hld/14-development-backlog.md:2214`). The end gate does not yet hold at
this dependency-prefix checkpoint because F-254, F-255, F-252, and F-256 remain
pending (`docs/sprints/CURRENT_SPRINT.md:41`).

All three completed prefix gates hold. F-253 applies one resolver and mutation
across body, cell, text-box, header, footer, footnote, endnote, and comment
owners (`crates/rdocx/tests/integration_test.rs:102`). F-250 exercises ordered
section mutation with independent story references
(`crates/rdocx/tests/integration_test.rs:915`) and observes retained and pruned
owners through F-253's `stories()` traversal after section removal
(`crates/rdocx/tests/integration_test.rs:1224`). Both interaction tests passed
at the reviewed HEAD.

F-251 builds its three-section Word oracle through F-250's `insert_section` and
`section_mut` APIs (`crates/rdocx/tests/integration_test.rs:640`), then proves
typed geometry after reopen, deterministic physical geometry, independent
physical and displayed page numbers, and exact Word records
(`crates/rdocx/tests/integration_test.rs:685`). Its preservation gate covers
unsupported child order and retained M24 page-number state
(`crates/rdocx/tests/integration_test.rs:422`), while its invalid-input gate
checks byte atomicity (`crates/rdocx/tests/integration_test.rs:521`). The
continued endnote-page regression also verifies physical identity and PAGE
substitution after a restart (`crates/rdocx-layout/src/engine.rs:13380`). All of
these focused checks passed at the reviewed HEAD.

The delivery record ties full verification to F-253, F-250, and F-251's
integrated feature SHAs and records an unchanged 49-of-49 harness for each
(`docs/sprints/AS_BUILT.md:12823`, `docs/sprints/AS_BUILT.md:12869`,
`docs/sprints/AS_BUILT.md:12916`). Their adjoining hash records are unchanged
(`docs/sprints/AS_BUILT.md:12826`, `docs/sprints/AS_BUILT.md:12872`,
`docs/sprints/AS_BUILT.md:12919`). A current harness check also matched all 49
entries.

## Not found

- **interaction**: F-251 geometry is authored through the F-250 section facade,
  while F-250 removal exposes surviving ownership through the F-253 story
  facade. The direct combined gates at
  `crates/rdocx/tests/integration_test.rs:640` and
  `crates/rdocx/tests/integration_test.rs:1224` passed. No joint mismatch among
  F-250, F-251, and F-253 was found.
- **duplication**: the integrated architecture retains one facade-owned story
  model and one ordered view over existing section owners
  (`docs/hld/03-architecture.md:1015`). The section iterator reads the existing
  paragraph and final-body owners directly
  (`crates/rdocx/src/document.rs:12790`). No competing story tree, section tree,
  geometry model, location resolver, or staged structural commit path was
  added.
- **layering**: no Cargo manifest changed in the reviewed range. The typed
  section grammar remains in `rdocx-oxml`
  (`crates/rdocx-oxml/src/document.rs:55`), the facade consumes it through
  concrete borrowed handles (`crates/rdocx/src/document.rs:1777`), and layout
  consumes the displayed number through `PageFrame`
  (`crates/oxml-layout/src/output.rs:400`). No forbidden dependency direction
  was introduced.
- **harness**: no baseline file changed. Every completed delivery record says
  49 of 49 unchanged (`docs/sprints/AS_BUILT.md:12826`,
  `docs/sprints/AS_BUILT.md:12872`, `docs/sprints/AS_BUILT.md:12919`), and the
  current harness reproduced that result.
- **gate**: completed DOCX-015, DOCX-016, and DOCX-018 rows have implementation
  evidence and no owner, while the adjacent incomplete rows retain F-252,
  F-254, F-255, and F-256 (`docs/hld/02-scope-and-non-goals.md:222`). The
  focused modern-DOCX owner-invariant test passed.
- **docs**: architecture assigns story traversal and section geometry to the
  same facade without a second tree (`docs/hld/03-architecture.md:1015`). The
  rendering specification distinguishes physical and displayed numbering and
  carries restarts through endnotes (`docs/hld/08-rendering-spec.md:957`). The
  testing strategy names the differential, preservation, and atomicity gates
  exercised in this pass (`docs/hld/12-testing-strategy.md:1422`). No
  contradiction or missing integrated HLD update was found.
- **deps**: no dependency declaration, feature declaration, crate, or module
  changed in the reviewed range.
- **surface**: the exported story and section values are the planned native API
  (`crates/rdocx/src/lib.rs:53`). The binding contract explicitly covers the
  story, section, typed `CT_SectPr`, and `PageFrame` additions while withholding
  unplanned Python, WASM, and CLI entry points
  (`docs/hld/10-bindings-spec.md:320`). No public API without a story was found.

## Checks

- `cargo fmt --all --check`, passed.
- `cargo check -p rdocx --all-targets`, passed.
- `cargo test -p rdocx --test integration_test mixed_orientation_sections_match_word_geometry_and_page_numbers -- --exact`,
  passed.
- `cargo test -p rdocx --test integration_test section_geometry_round_trips_with_unsupported_children_in_order -- --exact`,
  passed.
- `cargo test -p rdocx --test integration_test rejected_section_geometry_is_atomic -- --exact`,
  passed.
- `cargo test -p rdocx --test integration_test ordered_section_mutations_preserve_independent_story_references -- --exact`,
  passed.
- `cargo test -p rdocx --test integration_test removing_a_section_never_orphans_a_shared_story -- --exact`,
  passed.
- `cargo test -p rdocx --test integration_test one_generic_mutation_edits_the_same_shape_in_every_story -- --exact`,
  passed.
- `cargo test -p rdocx-layout endnote_pages_continue_restarted_display_numbers_and_page_fields`,
  passed.
- `cargo test -p rdocx-layout section_page_number_start_uses_the_typed_section_property`,
  passed.
- `python3 scripts/hash_harness.py --check`, passed 49 of 49.
- `python3 -m unittest scripts.test_sprint_workflow.SprintWorkflowTests.test_every_incomplete_modern_docx_row_has_one_live_owner`,
  passed.
- `python3 scripts/prose_check.py`, passed.
- `python3 scripts/sync_agent_skills.py --check`, passed with 26 skills in sync.
