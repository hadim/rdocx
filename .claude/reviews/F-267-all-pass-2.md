# F-267, all, pass 2

**Reviewed**: the uncommitted working tree on `work/f-267-claude` after the
pass 1 remediation, 22 files, 2309 insertions and 208 deletions. Aspects:
correctness, contract, panics, ooxml, tests, structure.
**Verdict**: 0 defects, 0 smells, 2 nitpicks

## Pass 1 findings, resolved

- **D1**, the style update merge dropping an authored band size, is fixed at
  `crates/rdocx/src/document.rs:9553`. `merge_table_style_properties` now
  carries `row_band_size` and `column_band_size` with the same
  authored-overrides-existing rule as every other scalar member.
  `conditional_region_run_and_row_layers_survive_reopen` covers it, and the
  coverage was checked against the unfixed merge: it reports
  `Some((Some(2), Some(3)))` where the story requires `Some((Some(5), Some(3)))`.
- **S1**, the unconditional capture on the `w:cnfStyle` parse path, is fixed at
  `crates/rdocx-oxml/src/paragraph_properties.rs:823`. The capture now happens
  only when `w:val` is absent or the element carries an attribute beyond
  `w:val`, which is the only case that needs a carrier.
- **S2**, the stale `find_map` comment, is fixed at
  `crates/rdocx-layout/src/table.rs:1015`.
- The unused `PartialEq` derive from the pass 1 nitpicks is removed at
  `crates/rdocx-oxml/src/styles.rs:1189`.

## Defects

None.

## Smells

None.

## Nitpicks

- `crates/rdocx-oxml/src/styles.rs:1332`, a structurally empty `<w:rPr/>` or
  `<w:trPr/>` inside a `w:tblStylePr` is now skipped by
  `preserved_conditional_style_children` rather than carried as a preserved
  child, because both are modeled layers. It is only reachable when some other
  layer of the same region changed, since an untouched region returns its
  original bytes. `w:pPr`, `w:tblPr` and `w:tcPr` have behaved this way since
  they were modeled, so the two new layers match the family.
- `crates/rdocx/src/table.rs:242`, `TableLook::to_mask` writes four uppercase
  hex digits because that is the form Word writes. A producer that wrote a
  lowercase or wider mask reads back identically, since `Table::look` parses
  with `u16::from_str_radix`, but a byte comparison against such a producer's
  original `w:val` would differ after `set_look`. Only a caller that changed
  the look reaches it.

## Not found

- **correctness**: the region computation was re-derived against the plan.
  `applicable_table_regions` builds the selected regions then sorts by the enum
  order, so precedence is a property of the type rather than of push order.
  The `basedOn` chain flattens per region before the next region starts.
  Banding divides by the resolved band size after subtracting the header row or
  first column with `checked_sub`, so the header cell falls into no band. Band
  sizes resolve from the already style-resolved `CT_TblPr` that
  `resolve_base_table_properties` produces, which `overlay_table_properties`
  now carries them through. Nothing found.
- **panics**: no new `unwrap`, `expect`, slicing or indexing on parsed input in
  the shipped crates. `max(1)` before every band division, `checked_sub` for
  both offsets, and a checked band-size setter that rejects zero before it
  mutates. Nothing found.
- **ooxml**: the five `w:tblStylePr` layers write in schema order with the
  preserved children interleaved at their ranks, the two band sizes write at
  `w:tblPr` slots 4 and 5, the style's own `w:trPr` and `w:tcPr` write at ranks
  23 and 24 between the base `w:tblPr` and the first region, and `w:cnfStyle`
  writes at `w:pPr` slot 32. Reads are prefix-tolerant, writes use the fixed
  `w:` prefix, an unrecognised `w:type` serialises from `raw_xml` unchanged and
  is excluded from both resolution and the duplicate-region check, and the
  per-region attribute form of `CT_Cnf` survives through the existing modeled
  attribute carrier. `untouched_conditional_regions_serialize_byte_for_byte`
  and `an_unrecognised_conditional_region_no_longer_risks_refusing_the_file`
  prove both preservation paths. Nothing found.
- **contract**: every checklist item in `.claude/plans/F-267-design.md` is
  implemented. Two recorded deviations, both narrowing: the cell margins,
  vertical alignment and text direction mentioned once in the plan's Approach
  prose are not added to `ResolvedTableCellStyle`, because no checklist item
  and no test in the plan covers them and they would be an unverified layout
  change, and `StyleBuilder` gained base `table_row_properties` and
  `table_cell_properties` setters that the Approach did not spell out, without
  which the newly modeled base layers would be read-only and the DOCX-034
  `Create` and `Mutate` columns could not be claimed. Both are in the handoff.
  Nothing found.
- **tests**: the gate is reversible. Removing the single
  `regions.sort_unstable()` that makes the enum order the resolution order
  makes `every_conditional_table_region_matches_word` report
  `["A10000", "B1F1B1", "B1F1B1", "A20000", "C1C1F1", "E1E1F1", ...]` against
  the required `[..., "C1C1F1", "D1F1D1", ...]`, which is exactly the vertical
  band winning where the horizontal band must. The three named precedence
  regressions were each written before their fix and observed failing. The
  differential's oracle half found a genuine divergence rather than agreeing by
  construction: LibreOffice 26.2.5.2 paints the vertical band where Word and
  this workspace paint the horizontal band, and both sides of that divergence
  are asserted per rule 5 of `.claude/skills/differential-testing.md`. Nothing
  found.
- **structure**: no new trait, generic parameter, crate, module or file, and no
  new feature flag. The refused `table_style` module stayed refused. The added
  public surface is the enum, the typed read side, the region layers, the
  per-region removal, the base row and cell layers, `clear_look`, the band-size
  accessors and the paragraph selector, all named by the plan or required by
  the matrix columns it closes. Nothing found.
