# F-X109, all aspects, pass 1

**Reviewed**: working tree against `HEAD`, 13 files, 578 lines added and 55
lines removed
**Verdict**: 2 defects, 0 smells, 0 nitpicks

## Defects

### D1, alternate-content drawings make an otherwise valid split fail

`crates/rdocx-oxml/src/text.rs:958`

The approved contract requires every raw child to remain at its source
boundary, and the regression plan explicitly includes drawings. A run with
literal text and a preserved `mc:AlternateContent` drawing instead returns
`AlternateContentDrawing` before any split. Word commonly uses these preserved
compatibility blocks for shapes, so this leaves a normal mixed-content run
outside the promised checked primitive rather than moving its raw block and
layout-only projection together.

### D2, the mixed-content ordering contract is not covered

`crates/rdocx-oxml/src/text.rs:11058`

The tests cover plain text with a bookmark, a hyperlink with raw children, and
a field-only run. None splits one run containing the planned combination of
literal text, tab, break, field, drawing, symbol, and preserved raw content.
The new zero-width offset rules and the movement of each child across the
boundary could regress independently while every current test remains green.

## Smells

None.

## Nitpicks

None.

## Not found

No additional correctness, contract, panic, OOXML, test, or structural
findings beyond the defects above.
