# F-253, all, pass 3

**Reviewed**: complete uncommitted working-tree diff on
`work/f-253-codex`, 5 files and 1,767 changed lines, comprising 1,761
insertions and 6 deletions
**Verdict**: 1 defect, 0 smells, 0 nitpicks

## Defects

### D1, escaped nonpositive note ids expose conventional separator records
`crates/rdocx/src/document.rs:3308`

The note-owner filter decodes and normalizes `w:type`, but still parses `w:id`
from its raw attribute bytes. An untyped conventional separator written as
`w:id="&#45;1"` therefore fails integer parsing, becomes an absent id, and is
exposed as an editable normal-note story. XML character references are valid in
attribute values, so the id must be decoded before applying the existing
nonpositive-id fallback. The escaped-type regression does not cover the same
legal lexical form for `w:id`.

## Smells

None. Count: 0.

## Nitpicks

None. Count: 0.

## Not found

- **Pass 1 D1**: text extraction and replacement continue to exclude nested
  table-cell and text-box owners.
- **Pass 1 D2**: ordinary text is XML-unescaped while CDATA remains literal.
- **Pass 1 D3**: text mutation maintains `xml:space` for boundary whitespace.
- **Pass 1 D4**: complete owner-byte fingerprints invalidate locations after
  text, attribute, and structural mutation.
- **Pass 1 D5**: explicit separator types and literal nonpositive conventional
  separator ids remain filtered, subject to D1 above.
- **Pass 1 D6**: every named story kind exercises wrong-owner, wrong-kind,
  bounds, stale, and successful mutation behavior.
- **Pass 1 D7**: the round-trip gate asserts exact owner and item order and
  retained nodes across every named story.
- **Pass 1 D8**: item views retain a document borrow and location, package XML
  stays borrowed, and typed-tree projection is deferred.
- **Pass 2 D1**: self-closing paragraphs are expanded into matching-prefix
  paragraph, run, and text elements before insertion.
- **Pass 2 D2**: escaped separator `w:type` values are decoded and filtered,
  subject to the remaining escaped-id path in D1.
- **Pass 2 D3**: changing `xml:space` copies unrelated opening-tag attribute
  bytes without reparsing and reserializing them.
- **Correctness**: no additional wrong logic, boundary error, or unsupported
  mutation was found.
- **Contract**: apart from D1, traversal, identity, staging, and additive public
  API behavior match the approved design.
- **Panics**: no production `unwrap`, `expect`, unchecked indexing, slicing, or
  non-wrapping arithmetic was added on untrusted input.
- **OOXML**: apart from D1, no namespace, schema-order, whitespace, or retained
  subtree defect was found.
- **Tests**: apart from the missing escaped-id case in D1, the named integration,
  round-trip, and regression gates exercise the approved behavior.
- **Structure**: no new trait, generic parameter, feature flag, crate, module,
  forwarding wrapper, or dynamically dispatched concrete dependency was
  introduced.
