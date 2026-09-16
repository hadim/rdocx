# F-259, all aspects, pass 1

**Reviewed**: working tree, 12 files, 378 additions and 14 deletions
**Verdict**: 1 defect, 0 smells, 0 nitpicks

## Defects

### D1, ordered diagnostics are not tested

`crates/rdocx/tests/integration_test.rs:7658`

The approved unit contract requires ordered layout diagnostics to reuse the
production rules. Both current assertions require the diagnostic lists to be
empty or equal, so an implementation that always discards diagnostics passes.
Add a public measurement fixture that produces at least two distinct production
diagnostics and assert their exact order.

## Smells

None.

## Nitpicks

None.

## Not found

No additional findings in correctness, contract, panics, OOXML ownership,
tests, or structure.
