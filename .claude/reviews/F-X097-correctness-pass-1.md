# F-X097, correctness, pass 1

**Reviewed**: working tree, 3 files, 250 insertions and 22 deletions
**Verdict**: 2 defects, 0 smells, 0 nitpicks

## Defects

### D1, the synthetic Word prefix can collide with a producer binding

`crates/rdocx/src/comparison.rs:1363`

`story_document` always assigns `rdocxcmp` to the Word namespace and then
silently omits an inherited `xmlns:rdocxcmp` binding at line 1365. If an input
story legitimately binds that prefix to another namespace and uses it inside
the captured story content, the synthetic document changes the expanded names
before parsing. Comparison can then reject valid input or interpret foreign
elements as Word elements. The wrapper prefix must be selected without
colliding with any inherited binding, while the producer binding remains in
scope for the raw content.

### D2, the required regression does not test accept or reject

`crates/rdocx/tests/regression_test.rs:16518`

The drawing half of the named gate stops after save and reopen. The complex
field half at line 16530 likewise checks the staged XML but never calls
`accept_all` or `reject_all`. The design contract requires both ancestor-scoped
drawings and the changed complex-field paragraph to survive compare, save,
reopen, accept, and reject. A defect in either revision finalization path would
therefore leave this required gate green.

## Smells

None.

## Nitpicks

None.

## Not found

No additional findings in complex-field run projection, drawing raw-subtree
retention, malformed-input atomicity, panic safety, schema child order, public
API shape, or structural rules.
