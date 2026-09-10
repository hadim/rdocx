# F-254, all, pass 3

**Reviewed**: exact current uncommitted working-tree diff on
`work/f-254-codex`, 5 files, 1,534 insertions and 14 deletions
**Verdict**: 1 defect, 0 smells, 0 nitpicks

## Defects

### D1, block control validation ignores preserved root slots and raw content
`crates/rdocx/src/document.rs:372`

The new validator walks only `CT_Sdt::content` and accepts every `RawXml`
entry. It cannot see preserved root slots. The public standalone parser stores
a second `w:sdtContent` and other unmodeled root children as private raw slots
(`crates/rdocx-oxml/src/content_control.rs:684`). A caller can therefore parse
a control with one valid block content container followed by a second content
container containing a run. `ContentFragment::content_control` accepts it, and
serialization emits the duplicate container and invalid block grammar into the
destination story. A block-context parse can likewise retain an inadmissible
known child as `SdtContent::RawXml`, which this validator accepts without
examining its expanded name (`crates/rdocx-oxml/src/content_control.rs:1008`).
The new constructor regression covers typed paragraph, table, row, cell, run,
and nested-control values only (`crates/rdocx/tests/regression_test.rs:589`). It
does not cover either preserved representation. The constructor must validate
the complete serialized control grammar, including raw and root-slot content,
before presenting it as a block fragment.

## Smells

None.

## Nitpicks

None.

## Not found

The pass-2 boundary defect is closed. `ContentLocation::end` is distinct through
the private `is_end` field, and derived equality and hashing include that field.
`ContentLocation::new` retains its existing signature and always creates a
canonical non-end location. Actual anchors validate the flattened index, item
kind, direct-child status, owner fingerprint, and the body section-properties
exclusion without falling back to a second index space. Explicit end locations
work for ordinary, aliased, and self-closing package-backed owners and remain
before body section properties.

No additional defects were found in same-owner move arithmetic, self and
adjacent moves, sole-paragraph cell moves, cell raw-boundary stability, clone
identity allocation, balanced bookmark nesting, namespace closure and
shadowing, relationship scope and dangling-reference rejection, dirty-note
persistence, staged atomicity and reopen failure, panic safety, public API
source compatibility, test sensitivity outside the gap above, or repository
structural rules. `StoryError` remains unchanged, so downstream exhaustive
matches are not broken by a new variant.
