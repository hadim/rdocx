# F-253, all, pass 14

**Reviewed**: complete uncommitted working-tree diff on
`work/f-253-codex`, 8 files and 4,381 changed lines, comprising 4,333
insertions and 48 deletions
**Verdict**: 3 defects, 0 smells, 0 nitpicks

## Defects

### D1, byte-equal fields can cross a typed preservation boundary
`crates/rdocx/src/document.rs:4366`
`crates/rdocx/src/document.rs:4371`

Typed admission does not map each scanned field to the typed field at the same
source position. It retains a candidate when its bytes occur anywhere in any
typed field source from the enclosing paragraph. A paragraph containing one
valid direct complex field and one byte-identical sequence inside a boundary
that the typed paragraph parser keeps opaque therefore admits both scanner
items. The opaque copy becomes visible and mutable solely because the direct
copy has the same bytes. Admission must preserve source identity or positions,
not test byte membership across the paragraph.

### D2, complex-field XML overwrites an inherited foreign `w` binding
`crates/rdocx/src/document.rs:3703`
`crates/rdocx/src/document.rs:3705`
`crates/rdocx/src/document.rs:3707`

The owned projection always declares `w` as WordprocessingML and deliberately
discards the actual in-scope `w` binding. A valid field can use `q` for
WordprocessingML while inheriting `w="urn:producer"` for a retained attribute
or raw descendant. `StoryItemRef::xml()` then rebinds that retained `w:` name
to WordprocessingML, so the promised materialized namespace scope is not
equivalent to the source. The wrapper needs a collision-free Word prefix while
retaining every used in-scope binding.

### D3, valid same-run fields alias one full-run scan range
`crates/rdocx/src/document.rs:4211`
`crates/rdocx/src/document.rs:4212`
`crates/rdocx/src/document.rs:4281`
`crates/rdocx-oxml/src/text.rs:7801`

Each complex item records the complete enclosing begin run through the
complete enclosing end run. When two sibling fields, or an outer and nested
field, place multiple markers in the same run, their story items therefore
receive the same scan range even though the typed parser explicitly accepts
same-run complex markers. Text projection scans every cached result in that
range and mutation can replace text belonging to the other field. Field items
need exact marker-bounded content spans rather than run-bounded spans.

## Smells

None. Count: 0.

## Nitpicks

None. Count: 0.

## Not found

- **Pass 13 D1**: separator detection now checks depth one, so a nested
  separator does not suppress insertion for a separator-free outer field.
- **Pass 13 D2**: separator insertion under a default Word namespace now adds
  a nonempty Word prefix for `fldCharType`, and the focused reopen test reaches
  the typed field again.
- **Pass 13 D3 outside D2**: the public `Cow` documentation matches ordinary
  body, comment, and package-backed subtree behavior, and the focused test
  distinguishes owned typed sources from borrowed ancestor-dependent slices.
- **Owner identity and atomicity**: no additional relationship-order,
  fingerprint, bounds, kind, stale-resolution, staged-commit, or lifecycle
  defect was found.
- **Retained XML outside the findings**: modeled child grammar, opaque wrapper
  handling, unrelated text-tag attributes, and exact source splicing remain
  protected.
- **Panics and structure**: no added production panic, unchecked index,
  non-wrapping arithmetic, trait, generic parameter, feature flag, crate,
  module, forwarding wrapper, or unnecessary dynamic dispatch was found.
