# F-X097, correctness, pass 4

**Reviewed**: remediated working tree, 4 files, 459 insertions and 25 deletions
**Verdict**: 1 defect, 0 smells, 0 nitpicks

## Defects

### D1, the shared-owner guard rejects every parsed complex-field mutation

`crates/rdocx-oxml/src/text.rs:4204`

Every parsed complex field has a source owner, including the ordinary case in
which one modeled field alone owns its physical run sequence. `owner_fields`
therefore always contains at least the current field. If that field changes,
the new nonempty check rejects serialization even when the owner is not shared.
Existing supported edits to a normal complex field's instruction, cached
result, dirty state, or legacy form data can no longer save. The fail-closed
guard must apply only when the physical owner has more than one modeled field.

## Smells

None.

## Nitpicks

None.

## Not found

The pass 3 unsafe selection of one changed sibling is otherwise removed. No
additional findings in unchanged shared-owner serialization, comparison span
accounting, inherited namespace handling, drawing preservation, tests, panic
safety, schema order, or structural rules.
