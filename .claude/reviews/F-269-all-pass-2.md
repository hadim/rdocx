# F-269, all, pass 2

**Reviewed**: the remediated working tree on `work/f-269-claude`, 20 files,
3112 insertions and 57 deletions
**Verdict**: 0 defects, 0 smells, 4 nitpicks

Aspects reviewed: correctness, contract, panics, ooxml, tests, structure.

## Pass 1 findings, re-checked

- **D1, vertical alignment moved the note area with the body band.** Fixed.
  `crates/rdocx-layout/src/paginator.rs:1600` now draws the line numbers,
  translates the band and only then places the notes, and
  `apply_vertical_alignment` at `:1479` subtracts `reserved_height()` before
  computing the unused measure. Notes stay anchored to the bottom margin and a
  bottom-aligned body stops above the band it reserved.
- **D2, a split paragraph's continuation lines were never numbered.** Fixed.
  `crates/rdocx-layout/src/paginator.rs:2424` records the continuation lines on
  the direct exit of `render_para_split`, matching the recursive exit. The new
  assertion in
  `line_numbering_count_by_and_restart_place_margin_numbers` walks the whole
  continuous sequence across every page boundary and requires each number to
  be exactly `countBy` above the last, which fails if any placed line goes
  uncounted.
- **D3, gutter resolution left stale column tracks behind.** Fixed.
  `crates/rdocx-layout/src/engine.rs:7334` clears the tracks and the separator
  flag before re-resolving against the gutter-adjusted measure, so a bypass in
  the second resolution falls back to one column instead of keeping the first
  resolution's answer.
- **S1, two element-translation functions under one name.** Resolved.
  `crates/rdocx-layout/src/paginator.rs:2265` is `translate_element_tree` and
  `:2282` is `translate_element_shallow`. Neither name can be reached for by
  accident, the doc comment on the recursive one names the single shallow
  caller and says why they are apart, and no existing caller's behaviour
  changed. Merging them was measured and rejected: it would alter
  `render_shape_text` for nested content, which is a behaviour change outside
  this story with no test asking for it.

## Defects

None.

## Smells

None.

## Nitpicks

Unchanged from pass 1, and recorded rather than fixed.

- `crates/rdocx-oxml/src/document.rs:1020`, a retained `w:bidi`, `w:rtlGutter`,
  `w:docGrid` or `w:printerSettings` that a permissive producer placed before
  `w:textDirection` is written after it. The writer is correcting toward the
  `xsd:sequence`, which is what the slot table has always done across slots.
- `crates/rdocx-oxml/src/document.rs:1160`, modeled attributes are written in
  schema order with retained attributes first, so a producer that ordered
  `w:offsetFrom` before `w:zOrder` reads back with them swapped. This is the
  serialisation contract `CT_BorderEdge` already sets and
  `docs/hld/04-opc-and-packaging.md` now records for section children too.
- `crates/rdocx/src/document.rs:2709`, `column_widths` maps an absent
  `w:col/@w:space` to zero, so a caller cannot tell an absent trailing space
  from an explicit zero. The XML round trip is unaffected.
- `crates/rdocx-layout/src/paginator.rs:775`, `LINE_NUMBER_FONT_SIZE` is fixed
  at nine points rather than read from the document default. A fixed size is
  what makes the rendering baseline reproducible, and the constant says so.

## Not found

- **correctness**. No wrong logic, off-by-one, unhandled case or operator
  precedence error. Re-traced by hand: the one-column bypass in
  `resolve_column_tracks`, the track advance and the outright page finish, the
  mirrored swap keyed on the displayed rather than the physical page number,
  the two `w:offsetFrom` conventions, and the `w:display` and `w:zOrder`
  selection, which now has its own test.
- **contract**. The diff implements the design plan and nothing more. Three
  deliberate deviations are recorded in the handoff. No binding surface
  changed, so DOCX-036 keeps its `B` boundary, and `SectionInfo` is untouched.
- **panics**. No new `unwrap`, `expect`, slice index or unchecked arithmetic on
  parsed input outside test code. `count_by` is clamped to at least one before
  the modulus, `next_line_number` uses `saturating_add`,
  `finish_page_outright` uses `saturating_sub`, and every new parse retains the
  source text rather than erroring, so no previously loadable document becomes
  a parse failure.
- **ooxml**. Child order is `xsd:sequence` across the whole slot table, reads
  are prefix tolerant and writes use the fixed `w:` prefix, and the six
  children that stay raw keep their bytes at their slots.
  `unmodeled_section_children_stay_byte_exact` and
  `variable_width_columns_parse_and_write_in_schema_order` assert both, the
  second through an aliased prefix.
- **tests**. Every new test fails if its code is reverted. The column,
  vertical alignment, mirroring and single-column tests each assert a negative
  control, and the differential compares against LibreOffice rather than a
  recorded expectation of our own.
- **structure**. No new trait, generic parameter, `Box<dyn>`, feature flag,
  module or file. `PageGeometry` loses `Copy` because it now owns a track list,
  which is a breaking change recorded in the handoff.
- **hash harness**. 49 of 49 entries match, before and after remediation.
