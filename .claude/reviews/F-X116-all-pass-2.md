# F-X116, all, pass 2

**Reviewed**: remediated working-tree diff, 15 files, 1,455 changed lines
**Verdict**: 1 defect, 1 smell, 0 nitpicks

## Defects

### D1, StoryItem text includes move-source text that Paragraph text excludes

`crates/rdocx/src/document.rs:7571`

The story text scanner admits every modeled `w:t` without tracking whether it
is inside an accepted or rejected revision wrapper. A valid `w:moveFrom` uses
ordinary `w:t`, so `StoryItem.text` includes moved-away text while the paragraph
accepted walker excludes the complete wrapper. A nested accepted revision under
a rejected outer revision is also exposed. This violates the single accepted
view promised by the design.

## Smells

### S1, the accepted-view regression covers deletion text but not move-source text

`crates/rdocx-py/tests/test_core.py:233`

The fixture uses standard `w:delText`, which the story scanner ignores even
without understanding revision visibility. Add a `w:moveFrom` containing
ordinary `w:t` so the gate proves that StoryItem text and Paragraph text reject
the same complete revision subtree.

## Nitpicks

None.

## Not found

The pass 1 serialization and atomicity findings are fixed. No additional
contract, panic, OOXML child-order, namespace, performance, or structure
findings.
