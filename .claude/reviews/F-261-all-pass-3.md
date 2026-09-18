# F-261, all aspects, pass 3

**Reviewed**: working tree against `4e96f8348adcd9b27309d7403d5f50b357030b22`, 15 files, 1245 insertions and 69 deletions
**Verdict**: 0 defects, 0 smells, 1 nitpick

## Defects

None.

## Smells

None.

## Nitpicks

- `crates/rdocx/src/html.rs:1784`, the shared image-dimension parser still
  describes an invalid fragment dimension as an MHTML image error.

## Not found

No correctness, contract, panic, OOXML child-order, namespace-preservation,
unmodelled-content preservation, atomicity, test-strength, workflow-invariant,
or structural-rule findings remain. The final capability-matrix change matches
the native-only public boundary, and the workflow owner set now removes F-259
and F-261 only after their capability rows are complete.
