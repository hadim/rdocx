# F-X097, correctness, pass 2

**Reviewed**: remediated working tree, 3 files, 334 insertions and 23 deletions
**Verdict**: 1 defect, 0 smells, 0 nitpicks

## Defects

### D1, sibling complex fields in one physical run cannot be correlated

`crates/rdocx/src/comparison.rs:5478`

The projection counts every complex field's complete retained source and then
advances the physical-run cursor by that count. The paragraph parser can
project two sibling complex fields from one physical `w:r` into two modeled
runs, with both fields retaining that same physical source. That admitted case
is established by `crates/rdocx-oxml/src/text.rs:2603` and its existing
same-run sibling regression. The first modeled field consumes the one physical
run here, then the second field fails the bounds check at line 5482. Comparing
a paragraph that contains same-run sibling complex fields therefore still
returns `comparison could not correlate paragraph run owners`. The ownership
projection must group modeled fields that share one physical source rather
than charging the same source span to each field independently.

## Smells

None.

## Nitpicks

None.

## Not found

The pass 1 synthetic-prefix collision and accept and reject coverage defects
are remediated. No additional findings in inherited namespace handling,
drawing raw-subtree retention, ordinary multi-run complex fields,
malformed-input atomicity, panic safety, schema child order, public API shape,
or structural rules.
