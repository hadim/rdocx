# F-261, all aspects, pass 2

**Reviewed**: working tree against `373d5a701615bef07a2970be2d6709388899e624`, 12 files, 1241 insertions and 67 deletions
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
unmodelled-content preservation, atomicity, test-strength, or structural-rule
findings remain. Pass 1 D1 is closed by the explicit story-kind gate. Pass 1
D2 is closed by routing main-document cells through their containing direct
typed body item. Pass 1 D3 is closed by the pinned Word, LibreOffice, and
Poppler verifier plus the production render assertion in the named gate. The
format-specific projection names from the pass 1 nitpick are now shared HTML
names.
