# F-267, all, pass 1

**Reviewed**: the uncommitted working tree on `work/f-267-claude`, 22 files,
2309 insertions and 208 deletions. Aspects: correctness, contract, panics,
ooxml, tests, structure.
**Verdict**: 1 defect, 2 smells, 4 nitpicks

## Defects

### D1, the style update merge drops an authored table-style band size
`crates/rdocx/src/document.rs:9553`

`merge_table_style_properties` copies every modeled `CT_TblPr` member from the
authored side onto the existing side. The two members this story adds,
`row_band_size` and `column_band_size`, had no branch there.

`existing.cloned()` seeds the merge, so an existing band size survived by
accident and masked the gap. The reachable case is an update that sets a band
size on a table style:
`set_style(StyleBuilder::table(id, name).table_properties(CT_TblPr { row_band_size: Some(5), .. }))`
published the existing value, or none, instead of 5. That is silent loss of an
authored value through the documented mutation path, which is exactly what
DOCX-034's `Mutate` column claims works.

The same class of gap was checked in `merge_conditional_table_style` for the
two new region layers and in `overlay_table_properties` for layout, and both
carry the new members correctly. Only the style-level table properties were
missed.

## Smells

### S1, an unconditional allocation on the hottest new parse path
`crates/rdocx-oxml/src/paragraph_properties.rs:823`

The `w:cnfStyle` parse captured the source element and rebuilt its namespace
bindings for every occurrence, then discarded the result whenever the element
carried only `w:val`. Word writes `w:cnfStyle` on every paragraph inside a
styled table, so the common shape is exactly the one that paid for a capture it
never used. The capture belongs behind the test that decides whether an
attribute carrier is needed at all.

### S2, a comment that describes code the rewrite removed
`crates/rdocx-layout/src/table.rs:1015`

The `basedOn` chain comment said the collection order exists so `find_map`
below answers with the nearest definition. The rewrite moved band-size
resolution onto the already-resolved `CT_TblPr` and deleted that `find_map`, so
the comment pointed a reader at code that is not there. A stale reason is worse
than no reason, because it is the thing a later reader trusts.

## Nitpicks

- `crates/rdocx-oxml/src/table.rs:578`, `w:tblStyleRowBandSize` and
  `w:tblStyleColBandSize` parse through `val.parse()?`, so a producer value
  that is not an unsigned integer now fails the whole file where it was
  previously preserved raw. This matches `w:divId` and `w:outlineLvl` in the
  same family, which is the convention of the files being edited, and Word
  writes only small positive integers. Recorded rather than changed.
- `crates/rdocx-layout/src/table.rs:965`, `cell_conditional_selectors` collects
  `w:cnfStyle` from the cell's direct paragraphs only. A paragraph nested inside
  a content control in the same cell does not contribute a selector. Word writes
  the selector on every paragraph of the cell, so the direct ones answer the
  question in practice.
- `crates/rdocx/src/style.rs:187`, `CLEAR_TABLE_CELL_PROPERTIES` takes bit 15,
  which is the last bit of the `u16` at `crates/rdocx/src/style.rs:168`. The
  next clear flag has to widen the field. Not this story's problem, worth
  knowing.
- `crates/rdocx-oxml/src/styles.rs:1190`, `ConditionalStyleLayers` derived
  `PartialEq` that nothing used, because the byte-identical short circuit
  compares the five members one at a time.

## Not found

- **panics**: no new `unwrap`, `expect`, slicing or indexing on parsed input in
  the shipped crates. `applicable_table_regions` uses `checked_sub` for the
  header offset and clamps the band size with `max(1)` before dividing, so the
  banding arithmetic cannot divide by zero or underflow. The band-size setters
  reject zero before mutating. Nothing found.
- **ooxml**: the `w:tblStylePr` writer emits `pPr`, `rPr`, `tblPr`, `trPr`,
  `tcPr` in schema order with preserved children interleaved at their ranks,
  `w:tblStyleRowBandSize` and `w:tblStyleColBandSize` land at `w:tblPr` slots 4
  and 5 between `w:bidiVisual` and `w:tblW`, and `w:cnfStyle` lands at `w:pPr`
  slot 32 between `w:divId` and `w:rPr`. Reads stay prefix-tolerant through
  `is_word_element` and writes use the fixed `w:` prefix. An unrecognised
  `w:type` serialises from `raw_xml` unchanged at
  `crates/rdocx-oxml/src/styles.rs:1264`. Nothing found.
- **contract**: the diff implements the plan's checklist. Two departures are
  recorded as deviations rather than findings, both narrowing rather than
  widening scope, and both listed in the handoff. Nothing found.
- **structure**: no new trait, generic parameter, crate, module or file. The
  resolution rewrite stayed in `crates/rdocx-layout/src/table.rs` as the S74
  consolidated round required, split into `resolve_table_style_cell`,
  `apply_conditional_region`, `cell_conditional_selectors` and
  `applicable_table_regions` so each reads on its own. `TableStyleRegion` is an
  enum in an existing file that removes thirteen string comparisons and makes
  an invalid region unrepresentable. Nothing found.
- **tests**: the gate fails against reverted behaviour, which is recorded in
  pass 2. Nothing found.
