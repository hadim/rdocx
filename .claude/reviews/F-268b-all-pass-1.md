# F-268b, all, pass 1

**Reviewed**: the uncommitted working tree on `work/f-268b-claude`, 10 files and
1079 insertions against 22 deletions. Three layout sources, two test
entrypoints, four HLD files and one workflow guard.
**Verdict**: 1 defect, 1 smell, 5 nitpicks

## Defects

### D1, the look-ahead offered a float at its unresolved anchor
`crates/rdocx-layout/src/paginator.rs:1315`

`lookahead_wraps` built each float's rect from `floating_wrap` alone, while
`place_floating_table` then dropped the same float below any float it may not
overlap. Two floats under `w:tblOverlap w:val="never"` therefore made the text
above them reserve a band for the second float at its anchor, which is not
where the second float lands. The recorded geometry was a left edge of 300
points where the float actually occupied the band ending at 191.

Fixed in this pass by extracting `settle_float` and running the same resolution
in the look-ahead, seeded from `page_floats` and extended as the look-ahead
walks forward. The regression
`two_floats_that_forbid_overlap_stack_instead_of_intersecting` pins both the
stacked and the intersecting case.

## Smells

### S1, the float predicate was written twice
`crates/rdocx-layout/src/engine.rs:7160`

`document_has_wrapping_drawing` had its own copy of "a `w:tblpPr` whose
`tblpYSpec` is not `inline` floats this table", and `table::floating_table` had
the same rule. Two places to look for one decision, and a future change to one
would silently disagree with the other: the engine would route the document
through one pass while layout still placed a float.

Fixed in this pass. `floating_table` is now `pub(crate)` and the engine asks it,
so the rule has one home.

## Nitpicks

- `crates/rdocx-layout/src/paginator.rs:1330`, the look-ahead cannot anticipate
  a float that moves to the next page after settling, because only a
  block-measured float is re-offered across the two passes. Recorded as a
  boundary in `docs/hld/08-rendering-spec.md` and in the code, rather than paid
  for with a second pass on every float document.
- `crates/rdocx-layout/src/engine.rs:7160`, the engine reads direct table
  properties, so a float declared only by a table style is placed but does not
  buy the document its second pass. The same function already accepts the
  identical miss for a drawing inside a nested table, and resolving the style
  chain in a cheap pre-pass would cost every table on every layout.
- `crates/rdocx-layout/src/paginator.rs:1941`, `page_floats` is cleared with
  `page_wraps` when a page ends and not when the body moves to the next column
  track, which is exactly the existing behaviour of `page_wraps`.
- `crates/rdocx-layout/src/paginator.rs:1321`, the look-ahead still adds a
  float's content height to its running flow estimate even though a float does
  not occupy the flow. It only decides when the look-ahead stops walking, and
  leaving it alone keeps a document with no float on the arithmetic it has
  today.
- `crates/rdocx-layout/src/table.rs:228`, a nested table that declares
  `w:tblpPr` lowers a `FloatingTable` that the cell renderer never reads, so it
  renders in the flow. That is what Word does with a nested table, and the
  lowered fact stays available to a later story.

## Deviations from the design plan

Recorded here because the plan is the contract.

- `FloatingTable` lives in `crates/rdocx-layout/src/table.rs`, beside
  `TableBlock`, not in `block.rs`. The plan cited `block.rs:110` for
  `TableBlock`, which is stale: `TableBlock` is declared in `table.rs`.
- `TableBlock::floating` is `Option<Box<FloatingTable>>`, not
  `Option<FloatingTable>`. The sprint's boxing rule for a new composite member
  applies, and `TableBlock` nests inside itself through `CellBlock`.
  `size_of::<Document>()` is 27192 before and after, and `TableBlock` grows 552
  to 560.
- `document_has_wrapping_drawing` in `engine.rs` had to see a floating table.
  The plan did not name it. Without it the engine routes a float document onto
  the single-pass restart path and the two-pass integration the plan specifies
  never runs.
- The plan lists four new free helpers as one method. `place_floating_table` is
  the one new `Pager` method, and `floating_wrap`, `settle_float`,
  `rects_intersect` and `is_paragraph_relative_float` are free functions beside
  `resolve_anchor_h`. Each has a second caller today: the look-ahead.
- `a_document_with_no_floating_table_still_paginates_in_one_pass` lives in the
  existing `paginator.rs` test module rather than the `rdocx` entrypoints,
  because `has_paragraph_relative_wrap` is private to `rdocx-layout` and the
  pass count is not observable from outside it. Making the guard real beat
  placing it where the plan sketched it.

## Not found

- **panics**: no `unwrap`, no `expect`, no indexing and no slicing in the new
  layout code. A float with no rows resolves a zero-height rect, which
  `rects_intersect` reports as intersecting nothing rather than panicking.
- **ooxml**: this story adds no parser and no serialiser, and touches no
  schema child order or prefix. F-268a owns all of it.
- **contract, scope**: no public facade surface is added, which is what the
  plan required. `FloatingTable` is public only because `rdocx_layout::table`
  is, which is how `TableBlock` is already exposed.
- **structure**: no new trait, no new generic parameter, no `Box<dyn>`, no new
  feature flag, no new crate, module or file.
- **tests, would they fail if reverted**: checked by stashing the three layout
  sources. The gate and all four regressions fail. The gate fails on the first
  float origin, `[(1, 72.0, 151.48, 100.0, 39.74)]` against the pinned
  `[(1, 72.0, 72.0, 100.0, 39.74)]`, which is the float back in the flow.
- **harness**: 49 of 49 hash entries match, and 7 of 7 golden pixel digests
  match against the pinned Poppler 26.01.0.
