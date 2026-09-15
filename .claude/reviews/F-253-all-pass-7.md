# F-253, all, pass 7

**Reviewed**: complete uncommitted working-tree diff on
`work/f-253-codex`, 5 files and 2,144 changed lines, comprising 2,131
insertions and 13 deletions
**Verdict**: 3 defects, 0 smells, 0 nitpicks

## Defects

### D1, preserved Word-namespace wrappers remain editable
`crates/rdocx/src/document.rs:3610`
`crates/rdocx/src/document.rs:3797`
`crates/rdocx-oxml/src/text.rs:3806`

The pass 6 remediation makes foreign preserved wrappers opaque, but both text
scans still treat every WordprocessingML element as transparent. The typed
paragraph parser captures an unrecognized Word child as one raw subtree. A
paragraph containing `<w:producer><w:t>retained</w:t></w:producer>` therefore
owns that complete child through `extra_xml`, while `StoryItemRef::text`
projects its text and `set_story_text` rewrites it. The same gap permits a
table cell or text box nested under an unrecognized Word wrapper to be promoted
into a separate story. The lexical scans need an explicit allowlist of modeled
transparent Word containers, or equivalent typed ownership information, so an
unknown same-namespace subtree remains opaque.

### D2, notes with absent or malformed ids are exposed as normal stories
`crates/rdocx/src/document.rs:3316`
`crates/rdocx/src/document.rs:3349`
`crates/rdocx-oxml/src/footnotes.rs:114`

`editable_note_owner` accepts an absent or unparsable `w:id` because both
states become `None` and `is_none_or` returns true. The existing typed note
model instead defaults that identity to zero and classifies an untyped
nonpositive record as a separator. An imported `<w:footnote>` with no id, an
empty id, or a nonnumeric id is consequently offered for mutation as a real
story even though the document's established note projection treats it as
separator infrastructure. Rejection or filtering must agree with the typed
owner and remain stable across dirty and reopened states.

### D3, empty text elements are not recognized as text-bearing
`crates/rdocx/src/document.rs:3824`
`crates/rdocx/src/document.rs:3873`

The mutation scan records only text, CDATA, and general-reference events as
replacement ranges. A `w:t` with no character event, whether `<w:t/>` or
`<w:t></w:t>`, leaves `text_ranges` empty. A paragraph mutation then appends a
new run instead of preserving the existing run formatting, while a field or
content-control item returns `NotTextBearing` even though it contains a valid
editable text node. The scan must distinguish an existing empty `w:t` from the
absence of a text node and replace or expand that element in place.

## Smells

None. Count: 0.

## Nitpicks

None. Count: 0.

## Not found

- **Pass 6 namespace-shadow gate**: body story mutation now reparses through
  `CT_Document` and flushes through the existing namespace replay and
  fail-closed path before commit.
- **Pass 6 foreign preserved boundaries**: markup-compatibility and other
  nontransparent foreign wrappers remain opaque. D1 is the remaining
  same-namespace sibling of that defect.
- **Earlier remediation**: nested owner exclusion, entity and CDATA text,
  whitespace preservation, complete owner fingerprints, escaped separator
  values, self-closing paragraphs, unrelated opening-tag bytes, identifier
  provenance, and facade-owned optional-part provenance remain intact.
- **Panics**: no production `unwrap`, `expect`, unchecked indexing, slicing,
  or non-wrapping arithmetic was added on untrusted story input.
- **Structure**: no new trait, generic parameter, feature flag, crate, module,
  forwarding wrapper, or dynamically dispatched concrete dependency was
  introduced.
