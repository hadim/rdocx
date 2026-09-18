# F-X126, all aspects, pass 1

**Reviewed**: working diff, 4 files, 265 insertions and 11 deletions
**Verdict**: 0 defects, 1 smell, 0 nitpicks

## Defects

None.

## Smells

### S1, The gate does not prove exact drawing markup retention

`crates/rdocx/tests/regression_test.rs:18561`

The design contract requires compared package save and reopen to retain raw
drawing markup. The new assertions check required namespace declarations,
relationship identifiers, payload bytes, visible text, and `docPr` identity,
but never compare the complete drawing fragment with the corresponding source
fragment. A regression that drops or rewrites an unrelated retained drawing
child could still pass this gate. Capture the three source drawing fragments
for each expected view and require the resolved fragments to match exactly.

## Nitpicks

None.

## Not found

Correctness, contract scope outside S1, panic safety, OOXML namespace closure,
schema ordering, changed-drawing sensitivity, and structure produced no other
findings. The model-only closure keeps package serialization on the existing
root-ownership path, and the changed `docPr` control prevents drawing equality
from being weakened globally.
