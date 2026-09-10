# F-253, all, pass 8

**Reviewed**: complete uncommitted working-tree diff on
`work/f-253-codex`, 5 files and 2,469 changed lines, comprising 2,456
insertions and 13 deletions
**Verdict**: 3 defects, 0 smells, 0 nitpicks

## Defects

### D1, complex fields are exposed as individual marker elements
`crates/rdocx/src/document.rs:3557`
`crates/rdocx/src/document.rs:3981`
`crates/rdocx-oxml/src/text.rs:3891`

The item scanner classifies every `w:fldChar` start, separator, and end marker
as an independent `Field` item. Each marker contains no `w:t`, so its text is
`None` and mutation returns `NotTextBearing`. The existing typed paragraph
model instead projects the balanced marker sequence as one complex `Field`
with a cached result. A normal complex PAGE, REF, or form field is therefore
reported as three noneditable field fragments rather than one addressable
field. This does not provide the field traversal and generic mutation promised
by the story contract.

### D2, the transparency allowlists still cross typed preservation boundaries
`crates/rdocx/src/document.rs:3273`
`crates/rdocx/src/document.rs:3289`
`crates/rdocx/src/document.rs:3566`
`crates/rdocx-oxml/src/text.rs:3806`

Opacity is decided by a flat Word local-name allowlist and by treating every
element in six drawing or VML namespaces as transparent. It does not check the
parent-child grammar that decides whether the typed paragraph parser models an
element or captures it as raw XML. For example, an unexpected known-name Word
wrapper such as `w:tbl` inside `w:p`, or an unknown DrawingML wrapper containing
`w:t` or `w:txbxContent`, is preserved by the typed paragraph owner but remains
transparent to these scans. Its text or nested owner is consequently projected
and can be mutated despite belonging to a raw subtree. Pass 7 closes unknown
Word wrappers only. Known names in invalid schema positions and unknown names
in the selected foreign namespaces still require a grammar-aware boundary.

### D3, related story sources do not retain relationship order
`crates/rdocx/src/document.rs:7642`
`crates/rdocx/src/document.rs:7697`

The public method promises stable document and relationship order, but source
discovery loops over Footnote, Endnote, and Comment kinds first and scans the
relationship list separately for each kind. If a comments relationship
precedes a footnotes relationship, `stories()` still returns every footnote
owner before every comment owner. The integration fixture adds those
relationships in the implementation's imposed kind order, so it cannot catch
this reversal. Discovery must either retain actual relationship order or state
and test a different approved ordering contract.

## Smells

None. Count: 0.

## Nitpicks

None. Count: 0.

## Not found

- **Pass 7 unknown Word wrappers**: unrecognized Word local names make their
  complete subtrees opaque. D2 is the remaining context-sensitive boundary.
- **Pass 7 invalid note ids**: missing, empty, malformed, and nonpositive note
  ids filter the complete note subtree and do not leak nested owners.
- **Pass 7 empty text nodes**: explicit and self-closing empty `w:t` elements
  remain addressable and expand in place while retaining their run owner.
- **Earlier remediation**: nested owner exclusion, entity and CDATA text,
  whitespace preservation, complete owner fingerprints, escaped separator
  values, self-closing paragraphs, unrelated opening-tag bytes, namespace
  fail-closed behavior, identifier provenance, and optional-part ownership
  provenance remain intact.
- **Panics**: no production `unwrap`, `expect`, unchecked indexing, slicing, or
  non-wrapping arithmetic was added on untrusted story input.
- **Structure**: no new trait, generic parameter, feature flag, crate, module,
  forwarding wrapper, or dynamically dispatched concrete dependency was
  introduced.
