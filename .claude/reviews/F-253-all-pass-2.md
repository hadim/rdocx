# F-253, all, pass 2

**Reviewed**: complete uncommitted working-tree diff on
`work/f-253-codex`, 5 files and 1,526 changed lines, comprising 1,520
insertions and 6 deletions
**Verdict**: 3 defects, 0 smells, 0 nitpicks

## Defects

### D1, self-closing paragraphs cannot receive text
`crates/rdocx/src/document.rs:3753`

`scan_story_items` exposes `<w:p/>` as a paragraph, but the empty-text path
requires an explicit closing-tag position before it can insert a run. A caller
that resolves such a paragraph and calls `set_story_text` receives the generic
"paragraph story item has no closing element" error instead of editing the
supported paragraph. Empty self-closing paragraphs are common in note and
header content, and the separator regression fixture itself contains this
shape without attempting to mutate it.

### D2, escaped note types can still expose separator records
`crates/rdocx/src/document.rs:3297`

The note-owner filter compares the raw attribute bytes directly with
`separator`, `continuationSeparator`, and `continuationNotice`. XML character
references are legal in attribute values, so a value such as
`w:type="separ&#97;tor"` resolves to `separator` but misses this comparison and
is exposed as an editable note story. The same scan already parses XML and
must compare the unescaped value, as the existing normal-note scanner does.

### D3, text mutation rewrites unrelated attributes on the first text node
`crates/rdocx/src/document.rs:3622`

`story_text_start` reconstructs the complete opening tag with `BytesStart` in
order to update `xml:space`. That canonicalizes every retained attribute's
lexical form, including quote style, spacing, and entity spelling, even when
the attribute is foreign and unrelated to the requested text mutation. This
violates the repository rule that unmodelled XML is preserved verbatim. The
patch should change only the `xml:space` attribute while copying all unrelated
opening-tag bytes exactly.

## Smells

None. Count: 0.

## Nitpicks

None. Count: 0.

## Not found

- **Pass 1 D1**: paragraph and table text scans now exclude nested table-cell
  and text-box owners.
- **Pass 1 D2**: text projection now unescapes ordinary text and leaves CDATA
  content literal.
- **Pass 1 D3**: boundary-space values now add `xml:space="preserve"`, subject
  to the unrelated-attribute preservation defect in D3 above.
- **Pass 1 D4**: the owner fingerprint now covers the complete owner bytes, so
  text and attribute mutations invalidate prior locations.
- **Pass 1 D5**: literal separator types and conventional nonpositive untyped
  separator records are filtered, subject to the escaped-value defect in D2
  above.
- **Pass 1 D6**: every named story kind now exercises wrong-owner, wrong-kind,
  bounds, stale, and successful mutation behavior.
- **Pass 1 D7**: the round-trip test now asserts exact owner and item order and
  retained nodes in every named story.
- **Pass 1 D8**: item views now retain only a document borrow and location.
  Package-backed XML is borrowed and typed-tree projection is deferred.
- **Panics**: no production `unwrap`, `expect`, unchecked index, or
  non-wrapping arithmetic was added on untrusted input.
- **OOXML child order and prefixes**: apart from D2 and D3, no fixed-prefix read
  assumption or schema-order violation was found.
- **Structure**: no new trait, generic parameter, feature flag, crate, module,
  forwarding wrapper, or dynamically dispatched concrete dependency was
  introduced.
