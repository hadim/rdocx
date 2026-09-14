# F-256, correctness, pass 2

**Reviewed**: working-tree diff, 7 implementation files, 1,322 inserted lines and 56 deleted lines
**Verdict**: 3 defects, 0 smells, 0 nitpicks

## Defects

### D1, Style-carried numbering is removed from the selected closure

`crates/rdocx/src/field.rs:1812`

Numbering pruning discovers `w:numId` values only from body XML. A selected
paragraph whose selected style supplies its numbering has no body `w:numId`, so
the referenced numbering instance and abstract definition are removed before
the selected style is merged. Dependency discovery must include numbering
references from the retained style closure.

### D2, Selected comments do not bring their part-local relationship closure

`crates/rdocx/src/field.rs:1668`

Relationship discovery scans only the selected main-body XML. The comment
import clones selected comment paragraphs, including retained or typed drawing
and hyperlink references, but it neither discovers nor remaps relationships
owned by the source comments part. A producer comment containing relationship
bearing content therefore reopens with a missing or unrelated relationship.

### D3, The field assertion does not prove that bookmark references were remapped

`crates/rdocx/tests/regression_test.rs:26773`

The dependency-rich test counts `w:fldSimple` opening tags but never inspects
their instructions or resolves them against the two freshly allocated bookmark
names. The test remains green if both imported fields retain the source name or
point at the same imported bookmark, which does not establish the approved
field-reference contract.

## Smells

None.

## Nitpicks

None.

## Not found

No additional package-authoritative XML, comment-thread identity, conflict
policy, atomicity, panic, schema-order, namespace, structure, or test-isolation
findings were found.
