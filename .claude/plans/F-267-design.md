# F-267, Complete table style and conditional formatting authoring

**Status**: completed
**Sprint**: S74
**Size**: L
**Depends on**: F-246, F-257, F-258

Series order is settled. F-267 completes before F-268a starts, enforced as a
dependency-prefix checkpoint, so nothing here is designed around a shared-wave
merge with the advanced table work.

## Problem

`docs/hld/02-scope-and-non-goals.md:241` records DOCX-034, table styles and
conditional formatting, as `partial` with `Remove` at `N` and F-267 as its
owner. The gap is in three layers.

The model drops two of the five conditional property layers.
`CT_TblStylePr` at `crates/rdocx-oxml/src/styles.rs:83` carries only `region`,
`paragraph_properties`, `table_properties` and `cell_properties`. ECMA-376
`CT_TblStylePr` also carries `w:rPr` and `w:trPr`, and
`parse_conditional_style_properties` at `crates/rdocx-oxml/src/styles.rs:1066`
never projects them, so they survive only inside `raw_xml` and through the
rank-ordered preservation in `serialize_conditional_table_style` at
`crates/rdocx-oxml/src/styles.rs:1098`. `CT_Style` at
`crates/rdocx-oxml/src/styles.rs:45` has the same hole for a table style's own
base `w:trPr` and `w:tcPr`, which `style_child_rank` sends to `extra_xml` at
ranks 23 and 24 (`crates/rdocx-oxml/src/styles.rs:805`). Band sizes are schema
slots in `tbl_pr_raw_boundary` at `crates/rdocx-oxml/src/table.rs:764` but are
absent from `CT_TblPr` at `crates/rdocx-oxml/src/table.rs:403`, so
`w:tblStyleRowBandSize` and `w:tblStyleColBandSize` are preserve-only.

Resolution is incomplete and mis-ordered.
`resolve_table_style_cell` at `crates/rdocx-layout/src/table.rs:950` produces
only paragraph properties, borders and shading. Nothing carries a run-property
layer, and `resolve_run_properties` at
`crates/rdocx-layout/src/style_resolver.rs:289` has no table-style step at all,
so a `firstRow` region holding `w:rPr` with `w:b` renders unbolded. The
conditional merge at `crates/rdocx-layout/src/table.rs:1005` runs inside the
per-style `basedOn` iteration, so a derived style's `wholeTable` overrides a
base style's `firstRow` instead of the chain being flattened per region first.
`applicable_table_regions` at `crates/rdocx-layout/src/table.rs:1063` pushes
the horizontal band before the vertical band, and since later regions overwrite
earlier ones, the vertical band wins where Word gives the horizontal band the
higher priority. The same function bands with `row.is_multiple_of(2)` and
`column.is_multiple_of(2)` at `crates/rdocx-layout/src/table.rs:1096` and
`:1104`, which ignores band size entirely and does not skip the header row or
first column before counting bands.

The authoring surface is thin and leaks.
`StyleBuilder::conditional_table_style` at `crates/rdocx/src/style.rs:391`
takes a `&str` region and three raw OXML options, with no run or row layer and
no way to remove one region. `clear_conditional_table_styles` at
`crates/rdocx/src/style.rs:384` is all or nothing, which is why DOCX-034
`Remove` is `N`. `Style::conditional_table_styles` at
`crates/rdocx/src/style.rs:98` returns `&[CT_TblStylePr]`, a type
`crates/rdocx/src/lib.rs` does not re-export, so a caller cannot name the
return value. Region validity is a string list checked late in
`validate_style_graph` at `crates/rdocx/src/style.rs:465`. `Table::set_look` at
`crates/rdocx/src/table.rs:679` writes `val: None`, discarding the legacy
`w:tblLook` bitmask that `Table::look` at `crates/rdocx/src/table.rs:1799`
still reads as a fallback, and there is no way to remove a look.

One further element arrives by handoff. `w:cnfStyle` on `w:pPr` is the
paragraph-level conditional selector. `CT_PPr` has no typed field for it, and
it is raw-preserved at schema slot 32 by `ppr_child_rank` in
`crates/rdocx-oxml/src/properties.rs:2572`. F-264 leaves it raw-preserved at
that slot deliberately and hands it to this story, so F-267 owns modelling it,
authoring it and feeding it into region selection alongside the row and cell
`w:cnfStyle` values that already work.

## Spec reference

- `docs/hld/02-scope-and-non-goals.md`, the DOCX capability table row DOCX-034,
  "table styles and conditional formatting".
- `docs/hld/08-rendering-spec.md`, "Word table styles resolve base-first
  through `basedOn`, then apply table-region properties in deterministic
  whole-table, band, edge, and corner priority" and the adjacent default table
  style and direct-overlay paragraphs.
- `docs/hld/04-opc-and-packaging.md`, "What transfers unmodified", for the
  unmodelled `w:tblStylePr` subtree and its schema slot ordering.
- `docs/hld/03-architecture.md`, "The dependency rule" and "Facade
  conventions", for the `rdocx-oxml` to `rdocx-layout` to `rdocx` split and for
  what the facade may return.
- `docs/hld/12-testing-strategy.md`, "Test taxonomy" for the differential
  category, "The Word render fidelity gate" for the pinned oracle identities,
  and "The private from-scratch DOCX conformance corpus" for the authoring
  evidence path.
- `docs/hld/10-bindings-spec.md`, native Word facade stability, for the
  additive public surface.
- `docs/hld/14-development-backlog.md`, F-267.

## Approach

Complete the model, correct and complete resolution, then expose a typed
authoring surface over both. Every step stays inside files that exist today.

**Model, `crates/rdocx-oxml`.** Add `run_properties: Option<CT_RPr>` and
`row_properties: Option<CT_TrPr>` to `CT_TblStylePr`, and
`table_row_properties: Option<CT_TrPr>` plus
`table_cell_properties: Option<CT_TcPr>` to `CT_Style`. Project them in
`parse_conditional_style_properties` and in the `CT_Style` child loop at
`crates/rdocx-oxml/src/styles.rs:255`, and emit them at their existing schema
ranks so the writer keeps the `pPr`, `rPr`, `tblPr`, `trPr`, `tcPr` sequence.
The byte-identical short circuit in `serialize_conditional_table_style` at
`crates/rdocx-oxml/src/styles.rs:1115` extends to compare the two new layers,
so an untouched region still round-trips as its original bytes. Add
`row_band_size: Option<u32>` and `column_band_size: Option<u32>` to `CT_TblPr`
at their existing `tbl_pr_raw_boundary` indices 4 and 5. Add
`cnf_style: Option<String>` to `CT_PPr`, parsed and written at its existing
schema slot 32 in `crates/rdocx-oxml/src/properties.rs:2572`, matching the
`cnf_style` fields `CT_TrPr` and `CT_TcPr` already carry at
`crates/rdocx-oxml/src/table.rs:942` and `crates/rdocx-oxml/src/table.rs:1286`.

`TableStyleRegion` lives in `crates/rdocx-oxml/src/styles.rs`, which is the
lower crate that both `crates/rdocx-layout` and `crates/rdocx` already depend
on, and is re-exported from `crates/rdocx`. A `w:tblStylePr` whose `w:type` is
not a recognised region is preserved through `raw_xml` and excluded from
resolution. A reopened Word file is never refused over an unrecognised region.

**Resolution, `crates/rdocx-layout`.** Replace the string region list with an
ordered enum so precedence is a property of the type rather than of push order:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum TableStyleRegion {
    WholeTable,
    Band1Vert,
    Band2Vert,
    Band1Horz,
    Band2Horz,
    FirstCol,
    LastCol,
    FirstRow,
    LastRow,
    NwCell,
    NeCell,
    SwCell,
    SeCell,
}
```

The declaration order is Word's increasing-priority order, which fixes the
horizontal against vertical band inversion. Restructure
`resolve_table_style_cell` to flatten the `basedOn` chain per region first and
then apply regions in ascending enum order, so a base style's `firstRow` still
beats a derived style's `wholeTable`. Compute band membership from the resolved
band sizes, defaulting to 1, after removing the leading header row when
`firstRow` applies and the leading column when `firstCol` applies. Widen
`ResolvedTableCellStyle` with `run_properties: Option<CT_RPr>`, cell margins,
vertical alignment and text direction, thread the run layer through
`layout_cell_content` at `crates/rdocx-layout/src/table.rs:774` and
`layout_paragraph_with_source_in_table` at
`crates/rdocx-layout/src/engine.rs:5653`, and add a table-style parameter to
`resolve_run_properties` that merges after document defaults and before the
paragraph style, matching the paragraph-property ordering already in
`resolve_paragraph_properties_in_table` at
`crates/rdocx-layout/src/style_resolver.rs:251`. Every other
`resolve_run_properties` call site passes `None`, which is exactly today's
behaviour outside a styled table.

**Facade, `crates/rdocx`.** Re-export `TableStyleRegion` from
`crates/rdocx/src/lib.rs` and take it in place of `&str` in
`StyleBuilder::conditional_table_style`, which gains the run and row layers.
Add `remove_conditional_table_style(region)` to close the DOCX-034 `Remove`
column, and a typed read side that returns the resolved layers instead of
`&[CT_TblStylePr]`. Make `Table::set_look` write both the six booleans and the
equivalent legacy `w:val` bitmask so the two forms never disagree, add
`Table::clear_look`, and add checked band-size setters. Region validity becomes
unrepresentable at the type level, and `validate_style_graph` at
`crates/rdocx/src/style.rs:465` keeps only the duplicate-region and
style-type checks. Extend the update merge at
`crates/rdocx/src/document.rs:9157` and `merge_conditional_table_style` at
`crates/rdocx/src/document.rs:9182` to cover the two new layers with the same
merge-or-replace policy the existing layers use. Add a paragraph-level
conditional selector setter and getter over the new `CT_PPr` field, reusing the
existing `TableConditionalFormatting` type at `crates/rdocx/src/table.rs:143`
so a paragraph, a row and a cell all select regions through one shape, and feed
the paragraph value into the `cnf` closure at
`crates/rdocx-layout/src/table.rs:1067` beside the row and cell values.

Replacing the `&str` region parameter is a breaking change to a published
crate. The family is pre-1.0 at an unreleased 0.14.0 and the string form has no
valid value the enum cannot express, so the typed form replaces it outright
rather than being added beside it. The semver impact is a breaking minor bump
within the 0.x series, which is the release this change lands in.

The `CT_Style` literals at `crates/rdocx/src/epub.rs:1462`,
`crates/rdocx/src/style.rs:153`, `crates/rdocx/src/style.rs:185`,
`crates/rdocx/src/style.rs:727` and
`crates/rdocx-layout/src/style_resolver.rs:778` gain the new fields as `None`.

## Rejected alternatives

- Keep `region` as a `String` and validate late. The type already has thirteen
  valid values and a precedence order, and an enum is what makes the ordering
  fix checkable rather than a comment.
- Model the two new layers as raw bytes. A raw layer cannot be merged during an
  update or read back through the facade, which is what DOCX-034 measures.
- Resolve run properties by synthesising a character style per region. That
  puts a fabricated style into `styles.xml` and changes the saved package to
  make the renderer easier.
- Correct the band-priority inversion in a separate story. It is not separable
  from the band-size work, since both rewrite the same region computation.
- Add a new `table_style` module for the resolution rewrite. Asked and refused
  in the S74 consolidated round. The rewrite stays in
  `crates/rdocx-layout/src/table.rs`. If the function genuinely becomes
  unreadable that is a microscope finding raised at review time, not a
  design-time assumption.
- Expose `CT_TblStylePr` more widely instead of a typed read side. F-257
  rejected exposing `CT_TblPr` from the facade for the same reason, which is
  that it makes callers responsible for schema order.
- Keep `StyleBuilder::conditional_table_style(&str)` beside the typed form.
  Two spellings of one parameter is a second place to look for no gain, and
  every string the old form accepted is expressible as a variant.
- Reject a `w:tblStylePr` carrying an unrecognised `w:type` on read. That
  refuses a file Word wrote, which is never the right trade for a region the
  renderer can simply skip.
- Apply the conditional `w:trPr` at layout in this story. It is modelled and
  round-tripped here, and its layout application belongs to F-268a, which owns
  advanced row-grid behaviour.
- Capture the Word-authored reference through the Word GUI. Word GUI capture is
  not available on this machine, so the reference is source-built XML pinned in
  the test instead. The no-repair confirmation becomes a tracked human action,
  not a gate this machine cannot run.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| differential | `every_conditional_table_region_matches_word` | All thirteen regions authored from scratch resolve and render to the pinned Word-authored reference, structurally over the parsed tree and by raster comparison against the pinned oracle. |
| regression | `a_vertical_band_no_longer_overrides_the_horizontal_band_it_should_lose_to` | Region precedence follows Word's increasing-priority order, so `band1Horz` and `band2Horz` beat `band1Vert` and `band2Vert` on the same cell. |
| regression | `a_derived_whole_table_region_no_longer_beats_a_base_style_first_row` | The `basedOn` chain flattens per region before region precedence applies, so a base style's `firstRow` still wins. |
| regression | `banding_no_longer_ignores_band_size_and_the_header_row_offset` | Row and column banding counts by the resolved band size and starts after the header row or first column when those regions apply. |
| unit | `conditional_run_properties_reach_resolved_runs` | A `firstRow` `w:rPr` renders on the header row and nowhere else. |
| unit | `paragraph_cnf_style_selects_conditional_regions` | A `w:cnfStyle` on `w:pPr` selects regions beside the row and cell selectors. |
| round-trip | `conditional_region_run_and_row_layers_survive_reopen` | Authored `w:rPr` and `w:trPr` in every region reopen as modeled content in the correct schema sequence. |
| round-trip | `paragraph_conditional_selector_survives_reopen` | An authored paragraph `w:cnfStyle` reopens as modeled content at schema slot 32 with unrelated producer XML intact. |
| round-trip | `untouched_conditional_regions_serialize_byte_for_byte` | An unmodified region keeps its original bytes, including unmodelled children and foreign attributes. |
| regression | `an_unrecognised_conditional_region_no_longer_risks_refusing_the_file` | An unknown `w:tblStylePr` type reopens, serialises unchanged from `raw_xml`, and contributes nothing to resolution. |
| regression | `a_table_look_change_no_longer_drops_the_legacy_bitmask` | `set_look` writes the `w:val` mask and the booleans together. |
| regression | `removing_one_conditional_region_leaves_its_siblings` | Per-region removal does not behave like `clear_conditional_table_styles`. |
| failure | `invalid_band_size_leaves_document_bytes_unchanged` | A rejected band size publishes no partial mutation. |

The **test gate** is the differential test
`every_conditional_table_region_matches_word`, per the backlog entry.

The gate lands on evidence this machine can reproduce, which is the settled
sprint evidence policy. The structural side asserts against a pinned
Word-authored `w:tblStylePr` reference tree built as source XML inside the
test. The raster side runs the already-installed pinned oracle. Oracle pins,
recorded per `.claude/skills/differential-testing.md` rule 1 and already fixed
by `scripts/docx_ssim_harness.py:38`: LibreOffice Writer 26.2.5.2 build
`cd7284b4cbbfeb507e630c1aac019f4157393acb` and pdftoppm 26.01.0, at 150 dpi
with the existing 0.95 SSIM and 0.80 coverage targets. Deterministic font mode
is mandatory on our side. The pinned tools resolve from
`/private/tmp/rdocx-s73-bin` first on `PATH`.

**Word GUI capture is not available on this machine and is not a blocker for
this story.** python-docx resolves no conditional formatting, so it cannot
answer the resolution question and is not used as the oracle here. Confirming
that Word itself opens the authored table without offering to repair it is a
tracked human action rather than an automated gate. It is recorded as a
follow-up with a backlog home so it is owned rather than remembered, and F-267
does not wait on it.

No new test binary. Tests are added as modules to
`crates/rdocx/tests/integration_test.rs` and
`crates/rdocx/tests/regression_test.rs`, alongside the existing conditional
coverage at `crates/rdocx/tests/integration_test.rs:9840`, and to the inline
module at `crates/rdocx-layout/src/table.rs:1174`. No binary fixture files. The
Word reference is source-built XML in the test.

## HLD impact

- `docs/hld/02-scope-and-non-goals.md`
- `docs/hld/04-opc-and-packaging.md`
- `docs/hld/08-rendering-spec.md`
- `docs/hld/10-bindings-spec.md`
- `docs/hld/12-testing-strategy.md`
- `docs/hld/14-development-backlog.md`

## Risk routing

Matched rows of `.claude/skills/risk-routing.md`:

- **Any parser or serialiser.** `crates/rdocx-oxml/src/styles.rs`,
  `crates/rdocx-oxml/src/table.rs` and `crates/rdocx-oxml/src/properties.rs`
  change. Extra checks: assert the `pPr`, `rPr`, `tblPr`, `trPr`, `tcPr` write
  sequence inside `w:tblStylePr`, the `tblStyleRowBandSize` and
  `tblStyleColBandSize` slots inside `w:tblPr`, and `w:cnfStyle` at slot 32
  inside `w:pPr`, keep reads prefix-tolerant and writes fixed-prefix, and prove
  with a round-trip test that `capture_element` still preserves an unmodelled
  subtree and a foreign region attribute byte for byte. An unrecognised
  `w:tblStylePr` type must serialise back from `raw_xml` unchanged.
- **Layout, pagination, line breaking, text shaping.** The region computation
  and the new run layer change rendered output. Extra checks: deterministic
  font mode for every baseline and every raster comparison, and any re-recorded
  baseline is a deliberate labelled step, never incidental.
- **Public API of a published crate.** `crates/rdocx` gains
  `TableStyleRegion`, the new builder layers, per-region removal, the typed
  read side, `clear_look` and the band-size setters, and
  `StyleBuilder::conditional_table_style` changes its region parameter type,
  and the paragraph facade gains a conditional-selector setter and getter.
  Semver impact: breaking within the 0.x series, landing in the unreleased
  0.14.0. The only removal is the `&str` region parameter, which every caller
  can express as a `TableStyleRegion` variant. Extra checks: run
  `cargo publish --dry-run` and the `.crate` size assertion, and add no surface
  this story did not ask for.
- **A new trait, generic parameter, crate, module or file.**
  `TableStyleRegion` is an enum in an existing file and introduces no trait, no
  generic parameter and no new file. **No new crate, module or file is
  created.** The one candidate, a `table_style` module in `crates/rdocx-layout`
  for the resolution rewrite, was asked and refused in the S74 consolidated
  round. Readability of the resulting function is a microscope concern at
  review time.
- **An external oracle comparison.** Pins recorded above and taken from
  `scripts/docx_ssim_harness.py`, not from a comment. Word GUI capture is
  unavailable on this machine, so the Word no-repair confirmation is a tracked
  human action with a backlog home rather than a gate, and the automated gate
  rests on the source-built reference tree plus the pinned raster oracle.

Not matched: unit conversion (band sizes are counts, not lengths, and no
`Twips` or `Emu` constructor changes), theme colour, crate dependency graph,
bundled fonts, WASM or PyO3 bindings, a new feature flag, release scripting, a
pure file move.

## Hash harness

**Expected unchanged.** The resolution changes can only reach output through a
table style, and no sample has one. Verified directly against the generated
`samples/` set: all seven of `feature_showcase`, `proposal`, `quote`,
`invoice`, `report`, `letter` and `contract` contain zero
`w:style w:type="table"` elements and zero `w:tblStylePr` elements in
`word/styles.xml`. `crates/rdocx/examples/generate_all_samples.rs` never calls
`Table::set_style`, and the default styles part written by
`crates/rdocx/src/document.rs` defines no table style, so the
`styles.get_default(StyleType::Table)` fallback at
`crates/rdocx-layout/src/table.rs:965` returns `None` and
`resolve_table_style_cell` returns its default for every sample cell. All 49
baseline entries, the three OOXML part hashes plus the PNG plus the three PDF
entries for each of the seven samples, are therefore expected to hold.

The two ways this prediction could fail, both of which must be checked before
any baseline is touched:

1. Adding typed fields to `CT_TblPr` moves `tblStyleRowBandSize` and
   `tblStyleColBandSize` out of `extra_xml`. If a sample ever carried one the
   `word/styles.xml` or `word/document.xml` hash could move on serialisation
   order alone. The grep above shows none do.
2. Threading a table-style run layer into `resolve_run_properties` changes the
   resolved run for cells in a styled table. With no table style present the
   new layer is `None` and the merge is a no-op, so the PNG and PDF hashes hold.

If either check turns up a moved entry, stop and attribute it before
proceeding. An unexplained delta blocks the merge.

## Implementation checklist

- [x] Add the failing differential first, then the three failing precedence
      regressions named as the sentences in the test plan.
- [x] Model `w:rPr` and `w:trPr` on `CT_TblStylePr` and base `w:trPr` and
      `w:tcPr` on `CT_Style`, with schema-ordered writes and the extended
      byte-identical short circuit.
- [x] Model `tblStyleRowBandSize` and `tblStyleColBandSize` on `CT_TblPr` at
      their existing schema slots.
- [x] Model `w:cnfStyle` on `CT_PPr` at schema slot 32, the element F-264 hands
      over raw-preserved, and feed it into region selection beside the row and
      cell selectors.
- [x] Introduce `TableStyleRegion` in `crates/rdocx-oxml/src/styles.rs` with
      Word's priority ordering, re-export it from `crates/rdocx`, and replace
      the string region list in model, resolution and facade.
- [x] Preserve an unrecognised region through `raw_xml` and exclude it from
      resolution, never refusing the file.
- [x] Fix the horizontal against vertical band inversion in
      `applicable_table_regions`.
- [x] Flatten the `basedOn` chain per region before applying region precedence,
      so a derived `wholeTable` no longer beats a base `firstRow`.
- [x] Compute banding from band size with the header-row and first-column
      offsets.
- [x] Widen `ResolvedTableCellStyle` and thread the run layer through
      `layout_cell_content`, `layout_paragraph_with_source_in_table` and
      `resolve_run_properties`, passing `None` at every non-table call site.
- [x] Model and round-trip the conditional `w:trPr` without applying it at
      layout, which belongs to F-268a.
- [x] Complete the facade: typed region layers, per-region removal, typed read
      side, the paragraph conditional selector, `set_look` writing both forms,
      `clear_look`, checked band sizes.
- [x] Extend the style update merge for the two new conditional layers.
- [x] Update the five `CT_Style` literal construction sites.
- [x] Record the Word no-repair confirmation as a tracked human-action
      follow-up in `docs/hld/14-development-backlog.md`.
- [x] Run the impacted oxml, layout, facade, round-trip, differential, public
      API dry-run, hash harness, prose, full verify and microscope gates.

## Open questions

None. Resolved in the S74 consolidated design round.
