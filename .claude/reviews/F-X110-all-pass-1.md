# F-X110, all aspects, pass 1

**Reviewed**: complete working-tree diff, 14 tracked files, 311 inserted lines and 9 deleted lines
**Verdict**: 1 defect, 0 smells, 0 nitpicks

## Defects

### D1, Removing an absent setting is not a no-op

`crates/rdocx-oxml/src/settings.rs:753`

`set_update_fields(None)` always enters the scalar rewriter when there is no
modeled occurrence. For an existing aliased settings root, that path adds the
fixed `w` namespace declaration even though there is no selected child to
remove. At the facade boundary, a minimal package with no settings part also
enters relationship and part allocation before it learns that the requested
state is already absent. That no-op can therefore fail under relationship
identifier exhaustion. Return unchanged before rewriting when the source has
zero occurrences, and return unchanged before staging when the document has no
settings owner.

## Smells

None.

## Nitpicks

None.

## Not found

No other correctness, contract, panic, OOXML, test-gate, or structure finding
was found. One valid toggle follows the shared on-off vocabulary and schema
slot. Duplicate and malformed forms remain source-owned and reject mutation.
The native and Python surfaces are additive, concrete, staged, and covered by
runtime and typing tests.
