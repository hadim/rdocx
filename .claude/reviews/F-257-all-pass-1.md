# F-257, all aspects, pass 1

**Reviewed**: working-tree diff, 5 files, 775 changed lines
**Verdict**: 2 defects, 0 smells, 0 nitpicks

## Defects

### D1, val-only table looks report the wrong regions
`crates/rdocx/src/table.rs:1141`

`TableRef::look` reads only the six optional boolean attributes and ignores the
legacy `w:val` bitmask. Word writers can emit the bitmask without the expanded
attributes, and the low-level model deliberately preserves both forms. A
`w:tblLook w:val="04A0"` value therefore reports the wrong first-row,
first-column, and banding state through the new typed facade even though the
layout resolver already falls back to the mask.

### D2, the gate leaves two checked branches unproved
`crates/rdocx/tests/integration_test.rs:6634`

The named gate exercises fixed layout and one-cell-per-column grids only. It
does not reopen `TableLayout::AutoFit`, and it does not prove that complete-grid
replacement accepts a valid spanning row or rejects incomplete row coverage
and total-width overflow without mutation. Those cases are explicit parts of
the design contract, so regressions in those branches would still leave the
gate green.

## Smells

None.

## Nitpicks

None.

## Not found

No additional correctness, contract, panic, OOXML ordering, raw-preservation,
test-gate, or structural findings.
