# F-266c, all, pass 6

**Reviewed**: the uncommitted working tree on `work/f-266c-claude` after the
pass 5 remediation, 20 tracked files. A confirmation pass over the pass 5
findings, then a sweep of the whole diff rather than of the last edit.
**Verdict**: 1 defect, 0 smells, 2 nitpicks

Aspects reviewed: correctness, contract, panics, ooxml, tests, structure.

Pass 5's smell is closed, and so is its second nitpick.

- **S1** is closed. `crates/rdocx/tests/regression_test.rs:32164` takes a
  direction and `regression_test.rs:32227` pins the horizontal side at two
  bars. Pass 5's reasoning was wrong about which mutation discriminates: the
  guard is doubly redundant, so dropping `!rotated ||` alone changes nothing.
  Dropping the setter's `rotated &&` reproduces the pass 4 bug and fails the
  plain-then-revised assertion, and dropping both fails the new horizontal
  one. The assertion is load bearing against the larger mutation.
- The grammar slip in the base advance comment is closed at
  `crates/rdocx-layout/src/engine.rs:8881`.

## Defects

### D1, a gridded paragraph that reflows around a float leaves the grid
`crates/rdocx-layout/src/paginator.rs:2776`, with the assignment at
`crates/rdocx-layout/src/paginator.rs:2871`

`reflow_around_wraps` re-breaks lines through the generic breaker and assigns
the result, and never re-applies the Word line height restore, so the grid snap
is dropped for every paragraph the second pagination pass touches.
`ParagraphReflow` carries only the items and the break parameters, and
`LineBreakParams` has no grid, so the pitch is not reachable there. Measured on
a `lines` grid with a 720 twip pitch and four body paragraphs, the advances are
44.0 throughout with no float and 43.61, 31.74 and 44.0 with one floating
table, and 31.74 is a natural line height rather than a multiple of the pitch.
This is not a pre-existing hole widened: an exact `w:lineRule` survives the
reflow because it is carried on the break parameters, and the grid has no such
representation. Any gridded section containing a floating table or a wrapping
drawing hits it, and `docs/hld/08-rendering-spec.md` now claims the snap
without qualification.

## Smells

None.

## Nitpicks

- `crates/rdocx-oxml/src/document.rs:801`, a second `w:docGrid` is retained at
  the slot that writes before the typed element, so two of them swap on save.
  The typing is first-wins and correct, and the input is schema-invalid, but
  `w:paperSrc` sends its duplicate to the slot after the typed element and this
  could match.
- `docs/hld/08-rendering-spec.md:720` is 108 columns where the surrounding new
  prose wraps at 70 to 80. The rewrap in pass 5 fixed one line and made
  another.

## Not found

- **the horizontal cell and ungridded paragraph paths**. Derived from the code
  rather than from the implementer's claim. With no rotation the transposed
  box reproduces the cell origin, width and painted height, and every
  downstream expression keeps its association. The upright band is never
  built, the furniture vector is provably empty, the rotation group is skipped
  and the change bar guard short-circuits. The one behavioural delta is the
  shaping guard moving from a present `w:spacing` to a non-zero extra, which
  skips a loop that would add zero. The harness holds at 49 of 49 over seven
  generated samples containing nested tables, and both sibling digests are
  asserted in the gate.
- **the seven `CT_PPr` toggles**. All seven appear exactly once in each of the
  ten wiring points, and the slot constants are a dense collision-free
  sequence with the new seven at 12 to 17 and 20, matching ECMA order.
- **the `doc_grid` cache key**. No bypass. Both production call sites pass the
  value they key on, the only `None` is a unit test, and the restart-eligible
  single-section path and the second wrap pass operate on already-laid-out
  blocks without re-entering either cache.
- **`w:docGrid` round trips**. A section carrying `w:bidi`, `w:rtlGutter`, a
  typed grid and `w:printerSettings` is byte exact, a grid carrying a child is
  preserved verbatim with its binding, and a grid written before `w:bidi` is
  normalised into schema order, which is what `w:vAlign` already gets.
- **module placement and test vacuity**, unchanged from pass 5.
- **gates**. Formatting, workspace clippy, the hash harness at 49 of 49 and the
  prose rules are clean, and every F-266c test passes.
