# F-X102, all aspects, pass 1

**Reviewed**: working-tree diff, 12 tracked files, 272 inserted lines and 48 deleted lines
**Verdict**: 0 defects, 1 smell, 0 nitpicks

## Defects

None.

## Smells

### S1, The scoped registry helper unnecessarily expands the public API

`crates/rdocx-layout/src/input.rs:87`

Only the layout engine and crate-local tests consume `scoped_to_part`. Leaving
it public adds an unsupported external construction path that the design does
not require. Restrict it to the crate so the feature changes rendering without
growing the public contract.

## Nitpicks

None.

## Not found

Correctness, panic safety, OOXML schema order, namespace preservation, test
gate coverage, cache identity, and structural indirection produced no other
findings.
