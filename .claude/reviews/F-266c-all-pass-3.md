# F-266c, all, pass 3

**Reviewed**: the uncommitted working tree on `work/f-266c-claude` after the
pass 2 remediation, 20 tracked files, 2616 insertions and 107 deletions.
**Verdict**: 1 defect, 1 smell, 2 nitpicks

Aspects reviewed: correctness, contract, panics, ooxml, tests, structure.

Every pass 2 finding was re-read against the current tree.

- **ND1** is closed. `crates/rdocx-layout/src/table.rs:1323` measures a rotated
  cell once at the wide trial and returns it for both bounds, so the collapse
  at `table.rs:1341` is a no-op. The new regression test proves autofit really
  engages: the same table resolves its first column to 36 points under
  `autofit` and 239 under `fixed`.
- **ND2 and ND4** are closed. `crates/rdocx-layout/src/engine.rs:8884` is one
  em again, with the reason stated, and the vertical run test discriminates:
  33.0000 plain, 15.9280 rotated, 11.0000 compressed.
- **ND3** is closed. `crates/rdocx-layout/src/paginator.rs:4397` keeps the
  previous values exactly on the non-rotated arm and builds the cell's upright
  row band on the rotated one.
- Every pass 2 nitpick is closed, and the interim `debug_assert_eq!` is gone
  with it. `crates/rdocx-layout/src/table.rs:694` calls the shared
  `cell_rotation`, and `cell_direction` is still live for the upright-stacking
  diagnostic.

## Defects

### D1, an auto-height rotated cell wraps on the stacking axis and paints outside its column
`crates/rdocx-layout/src/table.rs:702`

The fallback arm uses the column's horizontal content width as the
line-length measure. For a rotated cell the column width is the stacking axis
and the line-length axis is the row height, which the declared-height arm
gets right. The text wraps into many short lines, the stack becomes far taller
than the column is wide, and nothing constrains it. Measured with a long
sentence in a `tbRl` cell, the content paints from 11.33 to 94.42 points
against a column running 72.00 to 108.07, so it starts sixty points left of its
own column and outside the page margin. The declared narrow grid behaves the
same way and only a declared `w:trHeight` is correct. This is not introduced by
the ND1 fix, but that fix makes it reachable by default, because autofit now
deliberately chooses a narrow column, and
`autofit_does_not_size_a_vertical_column_from_its_text_length` asserts exactly
that column and so locks the symptom in.

## Smells

### S1, a new `PageGeometry` clone on the non-rotated hot path
`crates/rdocx-layout/src/paginator.rs:4409`

`cell_geometry` is already built per paragraph, and the unified furniture
tuple clones it again for every paragraph of every cell of every table, purely
to make the two arms agree on a type. `PageGeometry` owns a page border frame
and a line numbering, so the clone allocates for any bordered section.

## Nitpicks

- `crates/rdocx-layout/src/paginator.rs:4405`, a rotated cell with several
  paragraphs draws several identical change bars over the same row band, now
  that the band no longer varies per paragraph.
- `crates/rdocx/tests/integration_test.rs:20902`, the vertical run test still
  hard-codes the line start at 72 points.

## Contradiction carried forward

The DOCX-033 row is honest about `w:kinsoku`, `w:wordWrap` and
`w:topLinePunct`, whose default behaviour the UAX#14 breaker already provides.
It over-claims for `w:overflowPunct`, `w:autoSpaceDE` and `w:autoSpaceDN`,
whose ECMA defaults are on and whose hanging punctuation and inter-script
spacing are not implemented. The wording reached in pass 2 only said nothing
happens when an author turns one off, which sidesteps the unimplemented
default.

## Not found

- **test vacuity**. Every `.all()` is guarded by a length assertion, and the
  reversed containment in the row-height closure is guarded against the
  empty-string trap.
- **`painted_origin_x` first-match semantics**. The walk is depth-first
  document order, the sentinel occurs once, and a miss panics rather than
  passing silently.
- **borrow and ownership** of the conditional furniture binding and the
  upright band tuple.
- **panics, ooxml child order, structure and cache correctness**, all
  unchanged from pass 2 and still correct.
- **gates**. Formatting, clippy over the workspace, the hash harness at 49 of
  49 and the prose rules are all clean at this tree.
