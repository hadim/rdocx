# F-268b, all, pass 2

**Reviewed**: the same working tree after the pass 1 defect and smell were
fixed, 10 files and 1079 insertions against 22 deletions. Re-read every new
function in `paginator.rs`, `table.rs` and `engine.rs`, and both new test
modules.
**Verdict**: 0 defects, 0 smells, 5 nitpicks

## Defects

None.

D1 from pass 1 is closed. `settle_float` at
`crates/rdocx-layout/src/paginator.rs:535` is now the one implementation of the
`w:tblOverlap` rule, and both `Pager::place_floating_table` at
`crates/rdocx-layout/src/paginator.rs:1455` and `Pager::lookahead_wraps` at
`crates/rdocx-layout/src/paginator.rs:1315` run it against the same ordered
list. `two_floats_that_forbid_overlap_stack_instead_of_intersecting` pins the
second float at `(1, 82.0, 119.74, 100.0, 39.74)` when either float forbids the
overlap and at `(1, 82.0, 82.0, 100.0, 39.74)` when both allow it, and the
first float keeps its anchor in both.

## Smells

None.

S1 from pass 1 is closed. `crates/rdocx-layout/src/table.rs:168` declares
`floating_table` as the single decision, and
`crates/rdocx-layout/src/engine.rs:7160` calls it instead of restating the
`tblpYSpec="inline"` rule.

## Nitpicks

Carried unchanged from pass 1, each a recorded boundary rather than an
oversight.

- `crates/rdocx-layout/src/paginator.rs:1330`, a float that moves to the next
  page after settling is not re-offered to the look-ahead unless it is measured
  from its own block. Recorded in `docs/hld/08-rendering-spec.md` and in the
  code.
- `crates/rdocx-layout/src/engine.rs:7160`, direct table properties only, so a
  float declared by a table style is placed but does not buy the second pass.
- `crates/rdocx-layout/src/paginator.rs:1941`, `page_floats` clears with
  `page_wraps` at a page end and not at a column-track advance.
- `crates/rdocx-layout/src/paginator.rs:1321`, the look-ahead's flow estimate
  still counts a float's height.
- `crates/rdocx-layout/src/table.rs:228`, a nested table's lowered float is
  never read by the cell renderer.

## Re-checked in this pass

- **correctness**, the ordering inside `place_floating_table`. The rect is
  settled, then tested against the bottom of the body, then the page is
  finished and the rect is resolved and settled again on the fresh page. The
  wrap and the float record are pushed after that, so they land on the page the
  float actually occupies, and `record_body_fragment` and `render_table_row`
  both read `self.page_number` after the move. One attempt, so it terminates.
- **correctness**, `settle_float` terminates. It sweeps at most once per already
  placed float and each move strictly increases `y`, and `rects_intersect` uses
  strict inequalities so two floats stacked exactly band to band are settled.
- **correctness**, the `continue` in the table arm at
  `crates/rdocx-layout/src/paginator.rs:770` is equivalent to falling through.
  Nothing follows the table arm in the block loop.
- **correctness**, `has_paragraph_relative_wrap` at
  `crates/rdocx-layout/src/paginator.rs:490` returns early for every table
  block. That is sound because `block.paragraph()` is `None` for a table, so
  the arm it skips could only have returned false.
- **panics**, unchanged from pass 1. No `unwrap`, no `expect`, no indexing and
  no slicing in the new layout code.
- **tests**, the gate and all four regressions fail against the three layout
  sources stashed. `a_floating_table_that_does_not_fit_moves_whole_to_the_next_page`
  first proves the same six-row table splits when it is inline, so the single
  fragment it then asserts is about the float and not about the table.
- **contract**, the deviations recorded in pass 1 are unchanged and are listed
  there.

## Not found

- **ooxml**: no parser, no serialiser, no schema order and no prefix in this
  diff.
- **structure**: no new trait, generic parameter, `Box<dyn>`, feature flag,
  crate, module or file.
- **public API**: none added, which is what the plan required.
