# F-X090, correctness, pass 1

**Reviewed**: working-tree implementation, 4 files, 142 added lines and 8 removed lines
**Verdict**: 1 defect, 0 smells, 0 nitpicks

## Defects

### D1, repeated round trips do not assert that the reused identity survives

`crates/rdocx/tests/regression_test.rs:921`

The round-trip regression checks only that each part contains one `wp:docPr`.
It would still pass if serialization renumbered the body and header drawings to
different values, which is the preservation behavior the test is meant to
lock down. Extract the normalized value from both parts on every iteration and
assert that both remain equal to the original reused producer identity.

## Smells

None.

## Nitpicks

None.

## Not found

No additional correctness, contract, panic, OOXML, structure, or allocator
defects were found. The implementation checks producer uniqueness within one
physical part, merges valid values into package-wide authored occupancy, keeps
expanded-name filtering, and retains same-part alias rejection.
