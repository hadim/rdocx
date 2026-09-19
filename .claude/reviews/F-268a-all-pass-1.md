# F-268a, all, pass 1

**Reviewed**: `git diff` on `work/f-268a-claude`, 13 files, 2324 insertions and
72 deletions, against `.claude/plans/F-268a-design.md` and the parent split in
`.claude/plans/F-268-design.md`.
**Verdict**: 2 defects, 2 smells, 4 nitpicks

## Defects

### D1, free-text `w:val` attributes are escaped on write and not unescaped on read
`crates/rdocx-oxml/src/table.rs:813`

`w:tblCaption` and `w:tblDescription` are the first free-text `w:val`
attributes in this family. The writer escapes through quick-xml's attribute
constructor, but `get_word_val_attr` returns the attribute exactly as written,
which is correct for a token or an identifier and wrong for prose. Authoring
the caption `Totals & targets` and reopening the document returns
`Totals &amp; targets`, so the round trip is not idempotent and a second save
double escapes. A caption holding `<` has the same shape.

### D2, the autofit maximum double counts the cell margin
`crates/rdocx-layout/src/table.rs:1102`

The maximum was computed as `max_content.max(minimum) + horizontal_margin`
where `minimum` already carried the margin, so a cell whose natural width was
below its longest unbreakable run received the margin twice. It inflated the
narrow column of an autofit table by exactly the horizontal cell margin.

## Smells

### S1, autofit measurement recurses into nested autofit
`crates/rdocx-layout/src/table.rs:1072`

Measuring a cell at two trial widths lays out any table nested in it twice, and
each of those tables autofits in turn. With the production pass that follows,
the cost of a table nested `n` deep is three to the `n`. Word writes
`<w:tblW w:w="0" w:type="auto"/>` for ordinary nested tables, which satisfies
the engagement predicate, and the parser admits nesting to
`MAX_RECOGNIZED_TABLE_NESTING`, which is 32. A hand-built file is a denial of
service and an ordinary four-level corpus table is a visible slowdown.

### S2, autofit assigns cells to grid columns from the direct row properties
`crates/rdocx-layout/src/table.rs:1044`

The measurement pass read `w:gridBefore` from the row's own properties while
the production pass reads the style-resolved value. A table style that declares
`w:gridBefore` would have measured one set of columns and painted another.

## Nitpicks

- `crates/rdocx-oxml/src/table.rs:799`, a duplicate `w:tblpPr` keeps the last
  one and drops the first, which matches how `w:tblW` and `w:jc` already behave
  in this parser but is worth knowing.
- `crates/rdocx-oxml/src/table.rs:422`, the three new enums are not
  `#[non_exhaustive]`. Each is a closed ECMA vocabulary and `AnchorAlignH`
  beside them is closed too, so this matches the file.
- `crates/rdocx-layout/src/table.rs:456`, `omitted_after` is computed for every
  row and consumed only by a bidirectional one.
- `crates/rdocx/src/table.rs:510`, a `w:tblpPr` carrying neither an offset nor
  an alignment spec reads back as a zero offset rather than as absent, which is
  Word's default and is stated in the function's doc comment.

## Remediation

All four findings were fixed before this record closed, and the gate list was
re-run clean afterwards.

- D1: `unescaped_word_val_attr` at `crates/rdocx-oxml/src/table.rs:407` reads
  the two free-text values and puts the entities back, keeping a malformed
  entity verbatim rather than failing the parse.
  `table_and_row_advanced_properties_survive_reopen` now authors a caption with
  `&` and a description with `<` and `"`, asserts the escaped bytes in the
  saved part, and asserts the unescaped values on reopen.
- D2: the two measurements each add the margin once.
- S1: `declared_nested_grid_width` at `crates/rdocx-layout/src/table.rs:1159`
  gives a cell that holds a nested table that table's declared grid instead of
  a measured width, which makes autofit linear in nesting depth. The production
  pass still lays the nested table out for real, and
  `docs/hld/08-rendering-spec.md` records the rule.
- S2: the measurement pass calls `resolve_row_properties`.

## Not found

- **Contract.** Every checklist item in the design plan is implemented. The six
  `w:tblPr` children land at raw slots 1, 2, 3, 8, 15 and 16 and the four
  `w:trPr` children at slots 4, 5, 9 and 11, with the `xsd:sequence` order
  preserved on write and proved by
  `table_and_row_advanced_properties_survive_reopen`. The four deliberate
  deviations are recorded in `.claude/scratch/F-268a-progress.md` and none
  changes the observable contract. Nothing outside the plan was built, and
  floating placement, row splitting and binding parity were all left alone.
- **OOXML.** Read is prefix tolerant, proved by
  `tbl_ppr_attribute_matrix_is_prefix_tolerant_and_writes_a_fixed_prefix` under
  an aliased prefix. Write uses a fixed `w:` prefix. `capture_element` still
  preserves every unmodelled sibling at its slot, proved by the three reworked
  `unmodelled_*` tests, which now use a producer extension and `w:divId`
  because the names they used are modeled. An unrecognised anchor, alignment or
  overlap value reads as absent rather than inventing a position.
- **Panics.** No new `unwrap`, `expect` or slicing on parsed input. The
  autofit indices are bounded by `column_count`, `grid_before` and `grid_after`
  are clamped to the grid, and the bidirectional index is
  `num_cells - 1 - visual_idx` over `0..num_cells`. `i32::try_from` guards both
  twip converters.
- **Tests.** The gate
  `fixed_autofit_and_nested_table_geometry_matches_reviewed_word_pages` fails
  against reverted code, because the autofit table's pinned 20.34 and 242.28
  point columns are the declared 144 point grid without this change. The two
  harness guards are named for the facts they protect. Every geometry
  assertion runs in deterministic font mode.
- **Structure.** No new trait, generic parameter, `Box<dyn>`, wrapper, feature
  flag, crate, module or file. Three new enums and one struct in
  `rdocx-oxml`, six public types in the facade, two fields and four private
  functions in `rdocx-layout`. `CT_TblPPr` is boxed on `CT_TblPr`, so
  `size_of::<Document>()` is 27192 before and after.
- **Harness.** Unchanged at 49 of 49 entries. All four facts the plan rests on
  were checked against the generator and the paginator rather than assumed.
