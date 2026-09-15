# F-251, all aspects, pass 2

**Reviewed**: complete working-tree source diff from `25226ae5be006391463e509525e629e20f060d5b`, 7 files, 1,128 insertions, 96 deletions, 1,224 changed lines, plus pass 1 review
**Verdict**: 6 defects, 0 smells, 0 nitpicks

## Pass 1 closure

- D1 is closed for quoted `>` and attribute-name text inside values. The new
  scanner identifies quoted attribute spans before editing.
- D2 is closed for repeated `w:pgNumType` elements. The first element remains
  typed and later elements replay after it in source order.
- D3 is closed for unchanged repeated header and footer vectors. Retained
  children now carry occurrence boundaries.
- D4 is closed for page size, gutter, column spacing, and header or footer
  distance. These paths check the full EMU value before narrowing.
- D5 is closed. The legacy document setters retain their historical unchecked,
  infallible behavior instead of discarding a checked-setter error.
- D6 is closed for the three-page subject. A pinned, ignored Word and Poppler
  regeneration route produces a normalized record, and the normal gate checks
  multi-digit displayed numbers separately from physical page identity.

## Defects

### D1, endnote pages lose the final section's displayed-number sequence
`crates/rdocx-layout/src/paginator.rs:1615`

Endnote pages are appended with `PageFrame::new`, which initializes
`displayed_page_number` from the physical page number. If the final body
section restarts at 27 and occupies physical page 3, its first endnote page is
physical page 4 but should continue the displayed sequence at 28. The current
frame reports 4, and late PAGE substitution uses that wrong value. This path
also participates in recorded restart pagination, so the error reaches both
fresh and restarted results.

### D2, the new checked section margin setter still permits narrowing wrap
`crates/rdocx/src/document.rs:1933`

Unlike the other new section geometry setters, `Section::set_margins` calls
`Length::as_twips()` directly and returns no `Result`. A value beyond the
`i32` twip range therefore wraps to an unrelated margin. Signed margins may be
valid, but their magnitude still needs pre-narrow range validation. The legacy
`Document::set_margins` can retain its explicit unchecked compatibility path.

### D3, adding a missing start attribute can bind it to a foreign namespace
`crates/rdocx-oxml/src/document.rs:825`

When retained `pgNumType` XML has no start attribute, the rewriter always
inserts `w:start`. A valid producer element can use `q` for the Word namespace
while locally rebinding `w` to a foreign namespace, for example
`<q:pgNumType xmlns:q=".../main" xmlns:w="urn:producer"/>`. Setting the start
then authors a foreign `w:start`, so reopen still has no Word page restart.
Insertion must select a prefix that is bound to the Word namespace in the
retained element or safely establish the fixed output binding.

### D4, occurrence counts cannot retain a boundary when an earlier reference is removed
`crates/rdocx-oxml/src/document.rs:559`

For source order `h1, retained, h2`, the retained child is anchored after
header occurrence 1. If a caller removes `h1` from the public `header_refs`
vector, `h2` becomes occurrence 1 and serialization emits `h2, retained`.
The retained child has crossed the surviving modeled child. The fallback for
unreached occurrences at `crates/rdocx-oxml/src/document.rs:569` handles tail
truncation and clearing, but cannot distinguish front removal or reordering.
Stable replay around mutable repeated vectors needs surviving-child identity,
not only an ordinal count.

### D5, oracle regeneration ignores unexpected pages after page three
`crates/rdocx/tests/integration_test.rs:776`

The regeneration command asks `pdfinfo` only for pages 1 through 3, then zips
those sizes with extracted PAGE values at
`crates/rdocx/tests/integration_test.rs:795`. If Word unexpectedly produces a
fourth page, the zip still yields the three expected records and the oracle
regeneration passes. The route must assert the PDF page count and equal vector
lengths before comparing normalized records.

### D6, oracle temporary files survive every failing regeneration path
`crates/rdocx/tests/integration_test.rs:729`

The ignored oracle test creates a unique directory in Word's container, but
removes it only after all commands, parsing, and assertions succeed at
`crates/rdocx/tests/integration_test.rs:808`. A Word failure, Poppler failure,
parse failure, or expected-record mismatch leaves the DOCX and PDF behind.
Earlier failed assertions already required manual cleanup according to the
progress record. Cleanup needs a guard that runs during unwinding as well as
on success.

## Smells

None.

## Nitpicks

None.

## Focused checks

The following focused tests passed:

- the two `page_number` filtered `rdocx-oxml` tests
- `mixed_orientation_sections_match_word_geometry_and_page_numbers`
- `rejected_section_geometry_is_atomic`
- `legacy_section_geometry_setters_preserve_infallible_compatibility`

The external GUI oracle was not rerun during this audit. Pass 2 inspected its
recorded successful route and code path.

## Not found

- **Correctness**: no additional body-page continuation, header PAGE
  substitution, header parity, physical page identity, or cache input defect
  was found.
- **Contract**: all requested M23 property accessors and ordinary checked
  setters are present, and format or chapter metadata remains outside the
  authored surface as required.
- **Panics**: no new production `unwrap`, `expect`, unchecked index, or
  arithmetic panic was found.
- **OOXML**: quoted values, entity text, existing namespace aliases, duplicate
  page-number elements, unchanged raw subtrees, fixed-prefix fresh authoring,
  and the ordinary schema sequence produced no additional finding.
- **Tests**: the normal three-page oracle is sensitive to mixed geometry,
  multi-digit displayed PAGE values, and physical PageFrame numbers. No
  additional test issue was found beyond D4 through D6.
- **Structure**: no unjustified trait, generic, dynamic dispatch, forwarding
  wrapper, feature flag, crate, module, or source file was added.
