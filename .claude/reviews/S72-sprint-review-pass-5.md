# S72 sprint review, pass 5

**Reviewed**: `sprint/s72` at
`3b00e4475ef9420a29f9926c93c50bbd19054cf3` against merge base
`5fdbfc578adf2e5a1ceca0a3f0cee601ece44746`, 76 files, 14,694
insertions, 302 deletions, 14,996 changed lines, crates: `oxml-layout`,
`rdocx-layout`, `rdocx-oxml`, `rdocx-py`, `rdocx`
**Review boundary**: scheduled dependency-prefix boundary after F-254. This is
the approved extension beyond configured pass 3.
**Verdict**: 1 blocking, 0 should-fix, 2 nice-to-have

## Blocking

### B1, block content controls still bypass the section-owner guard
`crates/rdocx/src/document.rs:4267`

Commit `3b00e447` closes the pass-4 path for a direct paragraph fragment. The
new guard rejects insert, remove, clone, and move when the fragment itself is a
paragraph with typed section properties. It returns immediately for every
other direct fragment kind, including `ContentFragment::content_control`.

A body-level block content control is an approved direct owner child and can
contain a typed paragraph whose `w:pPr/w:sectPr` owns a header or footer
relationship. The existing source-built case at
`crates/rdocx/tests/regression_test.rs:12190` proves that this exact typed
shape and its relationship are admitted. Generic removal of the direct
`w:sdt`, or insertion, clone, or movement of the same fragment, therefore
passes the new guard and still changes a section-bearing subtree without
F-250's inheritance materialization and graph cleanup. Removal can drop the
last live reference and leave the authored story part orphaned.

The pass-4 interaction finding is not fully closed. The guard must inspect the
complete admitted direct fragment for section-owning paragraphs, including
block content-control descendants, or reject every direct fragment kind that
can contain one. A mutation-sensitive regression must exercise the block
content-control path and prove byte-identical rollback for insert, remove,
clone, and move before this dependency prefix advances.

## Should-fix

None. Count: 0.

## Nice-to-have

### N1, the F-253 final review retains its recorded trailing blank line
`.claude/reviews/F-253-all-pass-21.md:46`

This is the same non-behavioral observation recorded since pass 2. It does not
affect a gate and remains safe to leave recorded.

### N2, the pass-4 sprint review also ends with an extra blank line
`.claude/reviews/S72-sprint-review-pass-4.md:125`

`git diff --check 5fdbfc578adf2e5a1ceca0a3f0cee601ece44746..HEAD`
reports a new blank line at EOF for this file as well. It is review-only
whitespace with no behavioral or gate effect.

## Milestone gate

The M23 end gate requires all five private references to be generated from a
blank public facade, reopen without repair, match required package semantics
and reviewed deterministic visual thresholds, reproduce identical DOCX bytes,
and report no unexplained preservation-only fallback
(`docs/hld/14-development-backlog.md:2214`). It does not yet hold. F-255,
F-252, and F-256 remain pending (`docs/sprints/CURRENT_SPRINT.md:41`).

The four completed story gates continue to hold individually. The full
`rdocx` regression binary passed 425 tests with 4 ignored. The focused F-254
remediation predicate
`generic_content_operations_reject_section_owning_paragraphs_atomically`
passed, and F-250's `removing_a_section_never_orphans_a_shared_story` also
passed. Those tests prove the direct-paragraph remediation and dedicated
section path, but neither exercises a direct block content control containing
the same section-owning paragraph. The existing nested relationship case at
`crates/rdocx/tests/regression_test.rs:12170` confirms that such a fragment is
part of the modeled package surface.

The dependency prefix is not ready to advance while B1 remains. F-255's F-253
and F-249 foundations are otherwise present. F-252's F-250 and F-253
foundations are present, but the sprint sequence correctly keeps it after
F-255 so rich header and footer content receives part-scoped relationships
(`docs/sprints/CURRENT_SPRINT.md:52`). F-256 remains unready until F-255 and
F-252 complete its F-246 through F-255 dependency range
(`docs/hld/14-development-backlog.md:2430`).

## Not found

- **interaction**: one blocking finding, B1. The direct-paragraph portion of
  pass-4 B1 is closed, and no other interaction finding was found among F-253,
  F-250, F-251, and F-254.
- **duplication**: zero findings. The remediation adds one private guard and
  calls it from the four generic operations. It adds no second story tree,
  section tree, package model, or competing mutation path.
- **layering**: zero findings. No Cargo manifest changed and no forbidden
  lower-to-facade dependency was introduced.
- **harness**: zero findings. No baseline file changed, the remediation commit
  declares an unchanged harness, and the current harness matched all 49
  entries.
- **gate**: zero additional findings. Every completed story's named predicate
  exists and passes. The unfinished M23 end gate is not claimed as complete.
- **docs**: zero findings. The HLD and delivery records still match the four
  completed public surfaces and retain F-255, F-252, and F-256 as live owners.
- **deps**: zero findings. No dependency, feature declaration, crate, module,
  or source file was added. The run-state wave order retains F-255 before F-252
  and F-256.
- **surface**: zero findings. The remediation adds no public API. The integrated
  story, content, section, and geometry surfaces remain those approved by
  F-253, F-254, F-250, and F-251.

## Checks

- `cargo test -p rdocx --test regression_test`, passed 425 tests and ignored 4.
- `cargo test -p rdocx --test regression_test generic_content_operations_reject_section_owning_paragraphs_atomically -- --exact`,
  passed.
- `cargo test -p rdocx --test regression_test sectioned_mail_merge_scans_header_references_in_block_content_controls -- --exact`,
  passed.
- `cargo test -p rdocx --test integration_test removing_a_section_never_orphans_a_shared_story -- --exact`,
  passed.
- `python3 scripts/hash_harness.py --check`, passed 49 of 49.

