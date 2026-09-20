# F-268, Floating and advanced table layout

**Status**: completed
**Sprint**: S74
**Size**: L
**Depends on**: F-257, F-258, F-259

## The split

F-268 is split into two children under the sub-ID rule in
`.claude/WORKFLOW.md`, "Sub-IDs when a story splits". The draft parent plan
found four implementation layers with a clean seam after the third. The S74
consolidated design round confirmed the split.

The seam is deliberate and shippable. The first child models, authors and
round-trips every property the story names, including `w:tblpPr`, and makes
autofit, bidirectional order and row-grid offsets real in layout. It leaves
`w:tblpPr` as a modeled and authored fact that layout reads but does not act
on, so a floating table still renders in the flow. The second child closes
that gap by teaching the paginator to place it.

| Child | Title | Size | Depends on | Test gate |
|---|---|---|---|---|
| F-268a | Advanced table authoring and geometry | L | F-267 | golden, `fixed_autofit_and_nested_table_geometry_matches_reviewed_word_pages` |
| F-268b | Floating table placement and wrap | M | F-268a | golden, `floating_tables_match_reviewed_word_page_geometry_and_pagination` |

- `.claude/plans/F-268a-design.md` carries the `CT_TblPr` and `CT_TrPr`
  grammar, the public authoring and reader surface, content-driven autofit,
  `w:bidiVisual` visual column reversal, and the `w:gridBefore`, `w:wBefore`,
  `w:gridAfter`, `w:wAfter` and `w:tblCellSpacing` row-grid offsets.
- `.claude/plans/F-268b-design.md` carries `FloatingTable` lowering,
  `Pager::place_floating_table`, the `lookahead_wraps` and `ResolvedWraps`
  integration, and single-page float-against-float resolution for
  `w:tblOverlap`.

## Sequencing

Settled in series, not in parallel. F-267 completes before F-268a starts, and
F-268a completes before F-268b starts. Both edges are enforced as
dependency-prefix checkpoints, so neither child designs around a shared-wave
merge.

## Closure

The parent closes only when both children close. Each child carries its own
design plan, its own AS_BUILT entry and its own named test gate. Update
`docs/hld/14-development-backlog.md` and `docs/sprints/BACKLOG.md` with both
children when the split lands.

The DOCX-035 capability row in `docs/hld/02-scope-and-non-goals.md:242` moves
from `unsupported` to `partial` at F-268a and from `partial` to `complete` at
F-268b. It is not `complete` until the parent closes.

## Out of scope for both children

Recorded here once so neither child restates it as an obligation.

- **Adopting the literal ECMA autofit default**, which is autofit whenever
  `w:tblLayout` is absent regardless of `w:tblW`. F-268a requires an `auto` or
  absent `w:tblW` as well. Widening the predicate moves rendered sample
  geometry and needs the exclusive harness baseline, so it is a named future
  story rather than a deferred obligation inside this one.
- **A non-floating table flowing beside a float.** The reciprocal half of
  F-268b. The table arm never narrows columns inside a keep-out band, and
  teaching it to is a re-layout rather than a re-position. Named follow-up.
- **`w:cantSplit` as a meaningful distinction.** It needs row splitting for the
  default case, which needs the exclusive harness baseline, and no S74 story
  holds it. Both children keep `cantSplit` a round-trip and reader fact.
- **Facing-page and section-scoped float resolution.** F-268b resolves floats
  against each other within one page, because that is the scope the reviewed
  geometry exercises.
- **Binding parity.** `crates/rdocx-py/src/table.rs` and
  `crates/rdocx-wasm/src/lib.rs` carry a python-docx parity subset. F-X130
  documents the public surface separately.

## Open questions

None. The seven questions in the draft parent were answered in the S74
consolidated design round and are recorded in the children.
