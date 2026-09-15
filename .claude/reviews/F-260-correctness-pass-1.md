# F-260, correctness, pass 1

**Reviewed**: working diff, 5 files, 263 insertions and 7 deletions
**Verdict**: 1 defect, 0 smells, 0 nitpicks

## Defects

### D1, authored field display loses the logical run formatting

`crates/rdocx-oxml/src/text.rs:4342`

The mixed-field serializer writes the field through the ordinary field path
without passing the owning run properties. Text before and after the field is
written in physical runs that clone those properties, but the authored field
result run has no `w:rPr`. A caller that formats one logical mixed run therefore
gets different formatting on the cached field display after save and reopen.

## Smells

None.

## Nitpicks

None.

## Not found

No additional defects were found in contract scope, panic safety, OOXML child
order, namespace handling, unmodelled subtree preservation, tests, or
structure.
