# F-254, all, pass 4

**Reviewed**: exact current uncommitted working-tree implementation diff on
`work/f-254-codex`, excluding review artifacts, 5 files, 1,737 insertions and
14 deletions
**Verdict**: 1 defect, 0 smells, 0 nitpicks

## Defects

### D1, character references bypass direct-text rejection
`crates/rdocx/src/document.rs:4355`

The serialized block-control validator rejects direct `Text` and `CData`
events, but its catch-all arm accepts `Event::GeneralRef`. quick-xml 0.41 emits
numeric character references and the five predefined XML entity references as
`GeneralRef` events. The content-control parser preserves those events in raw
root slots or raw content entries. A control whose `w:sdtContent` directly
contains `&#65;`, or whose `w:sdt` root directly contains it, therefore passes
`ContentFragment::content_control` even though the reference resolves to
prohibited non-whitespace direct text. The scanner must resolve or
conservatively reject direct general references with the same depth test used
for text and CDATA.

## Smells

None.

## Nitpicks

None.

## Not found

Pass-3 D1 is otherwise closed. The serialized scanner sees modeled and
preserved root slots, rejects duplicate or out-of-order Word root children,
rejects raw direct Word row, cell, and run children, and recursively validates
direct nested block controls. Namespace resolution admits aliased Word markup,
honours scope shadowing, and leaves foreign raw subtrees opaque. Empty and
self-closing content containers, foreign raw children, whitespace-only direct
text, comments, processing instructions, and preserved alternate-content
subtrees remain serializable without loss.

No additional defects were found in canonical flattened destination anchors,
explicit end boundaries, self-closing owner expansion, body section-properties
placement, same-owner move arithmetic, sole-paragraph cell moves, namespace
closure, relationship ownership, identity freshening, balanced bookmark
preflight, staged atomicity, or reopen publication. No additional panic,
recursion, public API compatibility, OOXML child-order, test-sensitivity, or
repository structural-rule findings were found. The full `rdocx` regression
binary passed with 418 tests passed and 4 ignored.
