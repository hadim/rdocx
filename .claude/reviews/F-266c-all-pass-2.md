# F-266c, all, pass 2

**Reviewed**: the uncommitted working tree on `work/f-266c-claude` after the
pass 1 remediation, 20 tracked files, 2477 insertions and 100 deletions.
**Verdict**: 3 defects, 2 smells, 6 nitpicks

Aspects reviewed: correctness, contract, panics, ooxml, tests, structure.

Every pass 1 finding was re-read against the current tree. All five defects and
all eleven smells are closed, and the three nitpicks are closed. What follows
is new, over the code the remediation itself added.

## Defects

### ND1, autofit inverts the minimum and the maximum for a rotated cell
`crates/rdocx-layout/src/table.rs:1288`

`AUTOFIT_MIN_TRIAL_WIDTH` is one point and `AUTOFIT_MAX_TRIAL_WIDTH` is ten
thousand, and for a horizontal cell the narrow trial yields the smallest
content width. The rotated branch returns the stacked height instead, which
moves the other way: the one-point trial puts one word on every line, so the
stack is as tall as it can be. The minimum becomes the huge value, the maximum
the small one, and `maximum.max(minimum)` collapses both to the huge one. When
the total exceeds the measure every column in the table is rescaled. Nothing
exercises it, because every rotation test builds its table through `add_table`,
which writes a declared grid and returns before autofit engages.

### ND2, a leading space collapses a combined cell and a compressed advance
`crates/rdocx-layout/src/engine.rs:8879`

`base_advance` is `shaped.advances.first()`, so a run whose text begins with a
space, reachable through `w:t` with `xml:space="preserve"` and
`w:eastAsianLayout w:combine="1"`, takes the space advance as its whole cell.
The run is then scaled into about a quarter of its size and is illegible. The
positive filter does not catch it, because a space has a positive advance. The
same value drives `w:vertCompress`.

### ND3, a rotated cell's change bar and anchored drawings take the transposed frame
`crates/rdocx-layout/src/paginator.rs:4401`

The furniture now escapes the rotation group, which was the point of the pass 1
fix, but it is still handed `content_y`, derived from the transposed box's top,
and `paragraph.content_height()`, the stacked height that maps to the cell's
width on the page. `render_change_bar` reads that as a page-space y, so the bar
lands on the wrong page row with the wrong length. The right frame for both is
the cell's own upright row band.

## Smells

### ND4, `base_advance` is the visually first glyph and was chosen under test pressure
`crates/rdocx-layout/src/engine.rs:8879`

`shape_text` returns glyphs in visual order, so for a right-to-left run the
first advance is the logically last character, and `w:combine` is settable on
any run through the public facade. ECMA's two-lines-in-one fits a run into one
character cell, which for East Asian text is one em, the value this was before
the remediation. It changed so that `w:vertCompress` would measurably differ
from `w:vert` in the new test, because for Carlito the ascent and descent sum
to exactly one em and the spec rule is a no-op there. That is production
semantics following a test rather than the specification.

### ND5, the DOCX-033 row claims a rendered family while its evidence names six unrendered properties
`docs/hld/02-scope-and-non-goals.md:240`

The row is `complete` with every fidelity column `Y`, and its last evidence
clause says six East Asian toggles have no break-opportunity or inter-script
spacing projection. The matrix's own prose definition of complete is about
authorability, which the row satisfies, and the guard checks only that the
render column is `Y` or `NA`, so nothing mechanical catches the tension. The
guard itself is not weakened and `266` is added to the exclusion set in exactly
the shape F-267, F-268, F-269 and F-270 used. This is a judgement the
integrator should ratify rather than inherit silently.

## Nitpicks

- `crates/rdocx-layout/src/engine.rs:1688`, the `then_some(()).and_then(...)` chain reads worse than a plain `if`.
- `crates/rdocx-layout/src/table.rs:1288` duplicates the rotation resolution the cell loop already does.
- `crates/rdocx-layout/src/convert.rs:103`, the tolerance comment claims a few ulps for what is an absolute tolerance on a ratio.
- `crates/rdocx-layout/src/table.rs:727`, a rotated vertical-merge continuation now contributes its left and right margins to the row where it contributed its top and bottom ones before, because the measured width of no blocks is zero.
- `docs/hld/08-rendering-spec.md:688` does not say whether a `w:ruby` annotation line takes the grid, and `measure_annotation_line` does not apply it.
- `crates/rdocx/tests/integration_test.rs`, the vertical run test hard-codes the line start at 72 points.

## Not found

- **test vacuity**, the pass 1 trap. `painted_origin_x` panics when its text is
  absent, every `transforms_for` call site now asserts a length, and
  `the_default_doc_grid_type_does_not_change_line_advance` carries its own
  inequality guard against a vacuous comparison. None of the new assertions is
  vacuous.
- **borrow and ownership** of the conditional furniture binding and of the
  `write_doc_grid` closure. The closure captures the section and the integer
  buffer and is called at most once per branch, and the legacy branch now
  writes the modeled child before the raw dump, which is the house order.
- **panics**. No new `unwrap`, `expect`, indexing or slicing on untrusted
  input. Every new division is guarded by a positive check on its divisor.
- **ooxml child order**. Unchanged from pass 1 and still correct.
- **structure**. No new trait, generic parameter, `Box<dyn>`, forwarding
  wrapper, feature flag, crate, module or file.
- **cache correctness**. `doc_grid` is on both keys and compared at every read,
  and caller-width measurement bypasses the cache entirely.
