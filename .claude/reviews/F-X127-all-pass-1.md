# F-X127, all aspects, pass 1

**Reviewed**: working diff, 4 files, 148 insertions and 5 deletions
**Verdict**: 1 defect, 0 smells, 0 nitpicks

## Defects

### D1, visible continuation formatting can be collapsed

`crates/rdocx-layout/src/paginator.rs:2390`

The trailing-break predicate checks only line items and forced break kinds. A
paragraph can still paint shading, borders, a visible revision bar, or a
drawing-clear offset on the continuation. The next paragraph's
`pageBreakBefore` would then be suppressed despite visible structure between
the two requests. Exclude those paragraph states and add a focused control.

## Smells

None.

## Nitpicks

None.

## Not found

No other correctness, contract, panic, OOXML, test, or structure findings.
