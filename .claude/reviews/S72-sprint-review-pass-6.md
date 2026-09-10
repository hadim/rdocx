# S72 sprint review, pass 6

**Reviewed**: `sprint/s72` at
`e56ed6d81d4feb38e7f7b0c9f308c5337d5b26a5` against merge base
`5fdbfc578adf2e5a1ceca0a3f0cee601ece44746`, 77 files, 14,908
insertions, 302 deletions, 15,210 changed lines, crates: `oxml-layout`,
`rdocx-layout`, `rdocx-oxml`, `rdocx-py`, `rdocx`
**Review boundary**: scheduled dependency-prefix boundary after F-254. This is
the approved extension beyond configured pass 3.
**Verdict**: 1 blocking, 0 should-fix, 3 nice-to-have

## Blocking

### B1, the section-owner scan enters opaque foreign subtrees
`crates/rdocx/src/document.rs:4281`

Commit `e56ed6d8` closes the active direct-paragraph and block-control paths
from passes 4 and 5. The expanded-name scan finds a Word `w:p/w:pPr/w:sectPr`
at any depth, so an active paragraph below a direct block control or table is
rejected before insert, remove, clone, or move. The focused rollback tests for
the direct paragraph and block control pass, namespace aliases are recognized,
foreign-qualified lookalikes and escaped text are ignored, and ordinary
operations still pass.

The scanner does not track the typed admission boundary. It descends through
every foreign element and rejects a Word-qualified structural lookalike below
that preserved wrapper. For example,
`<x:raw><w:p><w:pPr><w:sectPr/></w:pPr></w:p></x:raw>` is one opaque raw subtree,
not an active section owner, but the conditions at lines 4281 through 4290
classify its descendant as active. A direct preserved fragment or block
content control containing that raw subtree can no longer be inserted,
removed, cloned, or moved even though the approved F-254 plan requires raw
nodes to remain movable and cloneable without parsing
(`.claude/plans/F-254-design.md:32`). The package contract likewise requires
foreign raw subtrees to stay opaque (`docs/hld/04-opc-and-packaging.md:105`).

The guard must follow the same modeled-versus-preserved boundaries as the
content grammar. It must recognize active Word section owners through aliases
and default namespace bindings, including active block-control and table
descendants, while skipping every descendant below a foreign or otherwise
opaque preserved root. A regression must place a Word-qualified
`p/pPr/sectPr` below such a foreign raw wrapper and prove insert, remove, clone,
and move retain normal behavior and exact raw bytes. Existing active-owner
cases must continue to reject atomically.

## Should-fix

None. Count: 0.

## Nice-to-have

### N1, the F-253 final review retains its recorded trailing blank line
`.claude/reviews/F-253-all-pass-21.md:46`

This review-only whitespace observation remains non-behavioral and safe to
leave recorded.

### N2, the pass-4 sprint review retains its trailing blank line
`.claude/reviews/S72-sprint-review-pass-4.md:125`

This is also review-only whitespace with no gate effect.

### N3, the pass-5 sprint review ends with an extra blank line
`.claude/reviews/S72-sprint-review-pass-5.md:120`

`git diff --check 5fdbfc578adf2e5a1ceca0a3f0cee601ece44746..HEAD`
reports all three review-file observations. None changes behavior.

## Milestone gate

The M23 end gate requires all five private references to be generated from a
blank public facade, reopen without repair, match required package semantics
and reviewed deterministic visual thresholds, reproduce identical DOCX bytes,
and report no unexplained preservation-only fallback
(`docs/hld/14-development-backlog.md:2214`). It does not yet hold. F-255,
F-252, and F-256 remain pending (`docs/sprints/CURRENT_SPRINT.md:41`).

The four completed story gates continue to hold individually. The full
`rdocx` regression binary passed 426 tests with 4 ignored. The direct-paragraph
and block-control section-owner regressions passed, as did
`content_control_fragment_accepts_aliases_and_foreign_raw_content`,
`cloned_preserved_content_gets_fresh_in_scope_identity_sets`, and F-250's
`removing_a_section_never_orphans_a_shared_story`. Those predicates prove the
active paths and ordinary foreign content covered by the remediation, but none
places Word-qualified section markup below an opaque foreign wrapper.

The dependency prefix is not ready to advance while B1 remains. F-255's F-253
and F-249 foundations are otherwise present. F-252's F-250 and F-253
foundations are present, but the sprint sequence correctly keeps it after
F-255 so rich header and footer content receives part-scoped relationships
(`docs/sprints/CURRENT_SPRINT.md:52`). F-256 remains unready until F-255 and
F-252 complete its F-246 through F-255 dependency range
(`docs/hld/14-development-backlog.md:2430`).

## Not found

- **interaction**: one blocking finding, B1. The active direct-paragraph,
  block-control, and table-descendant portions of the prior finding are closed.
  No other interaction finding was found among F-253, F-250, F-251, and F-254.
- **duplication**: zero findings. The remediation adds one shared private guard
  used by all four generic operations and no competing tree or mutation path.
- **layering**: zero findings. No Cargo manifest changed and no forbidden
  lower-to-facade dependency was introduced.
- **harness**: zero findings. No baseline changed and the remediation declares
  the expected unchanged 49-of-49 result.
- **gate**: zero additional findings. Every completed story's named predicate
  exists and passes. The unfinished M23 end gate is not claimed as complete.
- **docs**: zero findings. The HLD and delivery records match the four completed
  public surfaces and retain F-255, F-252, and F-256 as live owners.
- **deps**: zero findings. No dependency, feature declaration, crate, module,
  or source file was added. The run-state wave order retains F-255 before F-252
  and F-256.
- **surface**: zero findings. The remediation adds no public API. The integrated
  story, content, section, and geometry surfaces remain those approved by
  F-253, F-254, F-250, and F-251.

## Checks

- `cargo test -p rdocx --test regression_test`, passed 426 tests and ignored 4.
- Both `generic_content_operations_reject_section_owning_*_atomically`
  predicates passed.
- `content_control_fragment_accepts_aliases_and_foreign_raw_content`, passed.
- `cloned_preserved_content_gets_fresh_in_scope_identity_sets`, passed.
- `removing_a_section_never_orphans_a_shared_story`, passed.
