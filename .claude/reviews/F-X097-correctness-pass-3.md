# F-X097, correctness, pass 3

**Reviewed**: remediated working tree, 4 files, 445 insertions and 25 deletions
**Verdict**: 1 defect, 0 smells, 0 nitpicks

## Defects

### D1, changing one sibling field rewrites the wrong shared source region

`crates/rdocx-oxml/src/text.rs:4209`

For fields that share one physical run, the serializer selects the sole changed
field and passes its complete shared source to `write_field`. The complex-field
rewriter starts at the first top-level field in that source. If the second
sibling is changed, it therefore writes the second field's instruction and
result into the first sibling and clears or retains the wrong later content.
The new grouping path must either combine changes with source-aware field
boundaries or fail closed whenever any field in a shared physical owner has
changed. It must not serialize a plausible but corrupted field sequence.

## Smells

None.

## Nitpicks

None.

## Not found

The pass 2 unchanged same-run sibling comparison failure is remediated. No
additional findings in ordinary multi-run fields, physical span accounting,
inherited namespace handling, drawing raw-subtree retention, accept and reject
postconditions, panic safety, schema child order, or structural rules.
