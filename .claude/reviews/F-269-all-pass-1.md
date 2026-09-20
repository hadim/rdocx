# F-269, all, pass 1

**Reviewed**: the working tree on `work/f-269-claude`, 20 files, 3006
insertions and 55 deletions
**Verdict**: 3 defects, 1 smell, 4 nitpicks

Aspects reviewed: correctness, contract, panics, ooxml, tests, structure.

## Defects

### D1, vertical alignment moves the note area with the body band
`crates/rdocx-layout/src/paginator.rs:1602`

`finish_page` calls `place_page_notes` before `apply_vertical_alignment`, so a
centred or bottom-aligned page translates its own footnote separator and note
text downward with the body. Input: any section with `w:vAlign="bottom"` and a
footnote. What happens: the separator and the note are drawn below the bottom
margin instead of above it.

The same function also computes its unused measure as
`content_height - ink_bottom`, ignoring the note area `available_height`
already reserved. A bottom-aligned page therefore translates the body into the
band it made room for, overlapping the notes.

### D2, a split paragraph's continuation lines are never numbered
`crates/rdocx-layout/src/paginator.rs:2411`

`render_para_split` has two exits. The recursive exit re-enters itself and
records its lines, but the "remaining fits on the new page" exit renders
`remaining_lines` directly and does not call `record_numbered_lines`. Input:
any `w:lnNumType` section whose paragraph splits across a page or a column.
What happens: the continuation lines carry no number and every later line in
the section takes a number one too low for each lost line.

### D3, gutter resolution leaves stale column tracks behind
`crates/rdocx-layout/src/engine.rs:7334`

`section_page_geometry` adds the gutter to the inside margin and then
re-resolves the tracks against the shrunken text measure.
`resolve_column_tracks` bypasses and returns the geometry untouched when a
resolved width is not positive. Input: mirrored margins with a gutter large
enough to collapse an equal-width division. What happens: the geometry keeps
the tracks the first resolution produced, positioned against the ungutttered
margin, so the body lays out in the wrong place instead of falling back to one
column.

## Smells

### S1, `translate_element` and `translate_element_tree` are two answers to one question
`crates/rdocx-layout/src/paginator.rs:2255`

`translate_element` does not descend into `MarkedContent` or `Group`, and its
one existing caller, the shape text shift in `render_shape_text`, depends on
that. Vertical page alignment needs the nested form, so a second function now
sits beside it under a name that does not say which is which, and a later
caller can reach for the shallow one and silently leave nested content
behind.

## Nitpicks

- `crates/rdocx-oxml/src/document.rs:1020`, a retained `w:bidi`, `w:rtlGutter`,
  `w:docGrid` or `w:printerSettings` that a permissive producer placed before
  `w:textDirection` is written after it. The writer is correcting toward the
  `xsd:sequence`, which is what the slot table has always done across slots.
- `crates/rdocx-oxml/src/document.rs:1160`, modeled attributes are written in
  schema order with retained attributes first, so a producer that ordered
  `w:offsetFrom` before `w:zOrder` reads back with them swapped. This is the
  serialisation contract `CT_BorderEdge` already sets.
- `crates/rdocx/src/document.rs:2709`, `column_widths` maps an absent
  `w:col/@w:space` to zero, so a caller cannot tell an absent trailing space
  from an explicit zero. The XML round trip is unaffected.
- `crates/rdocx-layout/src/paginator.rs:775`, `LINE_NUMBER_FONT_SIZE` is fixed
  at nine points rather than read from the document default. A fixed size is
  what makes the rendering baseline reproducible, and the constant says so.

## Not found

- **correctness**, beyond D1 to D3. No other wrong logic, off-by-one,
  unhandled case or operator precedence error. The one-column bypass, the
  track advance, the mirrored swap on the displayed page number and the two
  page-border offset conventions were each traced by hand against their tests.
- **contract**. The diff implements the design plan and nothing more. No
  binding surface changed, so DOCX-036 keeps its `B` boundary.
- **panics**. No new `unwrap`, `expect`, slice index or unchecked arithmetic on
  parsed input. `count_by` is clamped to at least one before the modulus,
  `next_line_number` uses `saturating_add`, `finish_page_outright` uses
  `saturating_sub`, and every new parse retains the source text rather than
  erroring.
- **ooxml**. Child order is `xsd:sequence` across the whole slot table, reads
  are prefix tolerant, writes use the fixed `w:` prefix, and the six children
  that stay raw keep their bytes.
- **tests**. Every new test fails if its code is reverted. The column, vertical
  alignment and mirroring tests each assert a negative control, so none of them
  passes vacuously.
- **structure**. No new trait, generic parameter, `Box<dyn>`, feature flag,
  module or file.
