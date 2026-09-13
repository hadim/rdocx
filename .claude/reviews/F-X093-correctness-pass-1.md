# F-X093, correctness, pass 1

**Reviewed**: working-tree diff, 6 tracked files, 358 inserted lines and 69 deleted lines
**Verdict**: 1 defect, 0 smells, 0 nitpicks

## Defects

### D1, changed drawing owners still use lossy typed serialization

`crates/rdocx/src/comparison.rs:1617`

Exact source bytes are reused only when the complete aligned body item is
equal. If text changes in the same paragraph, table, or content control as an
otherwise unchanged drawing, the code enters `compare_body_content` and later
serializes the drawing-bearing run from the typed model at
`crates/rdocx/src/comparison.rs:2081`. A drawing whose `wp` binding is owned by
its original `w:drawing` wrapper can therefore lose that binding and fail with
the same missing `wp:docPr/@id` error this feature is meant to close. The gate
at `crates/rdocx/tests/regression_test.rs:16358` puts each drawing in a separate
unchanged paragraph, so it does not exercise this path.

## Smells

None.

## Nitpicks

None.

## Not found

No additional contract, panic, OOXML schema-order, test, or structure findings.
