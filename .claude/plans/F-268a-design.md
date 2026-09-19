# F-268a, Advanced table authoring and geometry

**Status**: completed
**Sprint**: S74
**Size**: L
**Depends on**: F-267

## Problem

The DOCX-035 row in `docs/hld/02-scope-and-non-goals.md:242` is classified
`unsupported`. Half of that classification is an authoring and geometry gap
that has nothing to do with floating.

Six table children and four row children are absent from the model and survive
only as ordered raw slots:

- `w:tblpPr`, `w:tblOverlap`, `w:bidiVisual`, `w:tblCellSpacing`,
  `w:tblCaption` and `w:tblDescription` are absent from `CT_TblPr`
  (`crates/rdocx-oxml/src/table.rs:400`) and preserved as slots 1, 2, 3, 8, 15
  and 16 of `tbl_pr_raw_boundary` (`:758`). The facade proves it by counting
  `<w:bidiVisual/>` as an unmodeled property at
  `crates/rdocx/src/table.rs:2272`.
- `w:wBefore`, `w:wAfter`, row-level `w:tblCellSpacing` and `w:hidden` are
  absent from `CT_TrPr` (`crates/rdocx-oxml/src/table.rs:923`) and preserved as
  slots 4, 5, 9 and 11 of `tr_pr_raw_boundary` (`:1209`).

Two more are modeled and then ignored:

- `w:gridBefore` and `w:gridAfter` are modeled at
  `crates/rdocx-oxml/src/table.rs:933` and read back at
  `crates/rdocx/src/table.rs:1902`, and no line of `crates/rdocx-layout/`
  mentions either. A row with a leading omission still paints its first cell at
  grid column zero.
- `w:tblLayout` is an opaque `Option<String>` at
  `crates/rdocx-oxml/src/table.rs:414`, copied through style resolution at
  `crates/rdocx-layout/src/table.rs:675` and never read by
  `compute_column_widths` (`:716`). Column widths come from the declared grid
  scaled to the declared `w:tblW`, with no content measurement, so `autofit`
  and `fixed` produce identical geometry.

There is also a specification claim with no Word implementation behind it.
`docs/hld/08-rendering-spec.md:602` states that right-to-left tables reverse
visual column placement without changing logical cell ownership. That sentence
is satisfied only by `crates/rpptx-render/src/lib.rs:585` for the DrawingML
`a:tbl`. No WordprocessingML path reverses anything.

## Spec reference

- `docs/hld/02-scope-and-non-goals.md`, "Modern DOCX capability matrix", the
  DOCX-035 row for floating, bidirectional, autofit and advanced table layout.
- `docs/hld/08-rendering-spec.md`, "Tables", for grid derivation, merge groups,
  physical border segments and resolved cell content boxes.
- `docs/hld/08-rendering-spec.md`, "Caller-width Word block measurement", for
  the measurement path autofit reuses.
- `docs/hld/08-rendering-spec.md`, "Autofit", which today specifies only the
  PresentationML shape ladder and needs a Word table-layout counterpart.
- `docs/hld/10-bindings-spec.md`, "Native Word facade stability", for additive
  concrete setters and readers on the existing handles.
- `docs/hld/12-testing-strategy.md`, "Test taxonomy", for the golden category.
- `docs/hld/12-testing-strategy.md`, "The golden-PNG gate", for deterministic
  font mode, the pinned Poppler 26.01.0 rasteriser and the reviewed manifest.
- `docs/hld/14-development-backlog.md`, "F-268, Floating and advanced table
  layout (L)", and the F-268a child entry the split creates.

## Approach

Three layers. Floating placement is F-268b and is not designed here.

### 1. Model the missing grammar, `crates/rdocx-oxml/src/table.rs`

```rust
/// `w:tblpPr`, the floating table position.
pub struct CT_TblPPr {
    pub left_from_text: Option<Twips>,
    pub right_from_text: Option<Twips>,
    pub top_from_text: Option<Twips>,
    pub bottom_from_text: Option<Twips>,
    pub horz_anchor: Option<ST_TblAnchor>,
    pub vert_anchor: Option<ST_TblAnchor>,
    pub tbl_p_x: Option<Twips>,
    pub tbl_p_x_spec: Option<AnchorAlignH>,
    pub tbl_p_y: Option<Twips>,
    pub tbl_p_y_spec: Option<ST_YAlign>,
}

pub enum ST_TblAnchor { Margin, Page, Text }
pub enum ST_YAlign { Inline, Top, Center, Bottom, Inside, Outside }
pub enum ST_TblOverlap { Never, Overlap }
```

`CT_TblPr` gains `float_position: Option<CT_TblPPr>`,
`overlap: Option<ST_TblOverlap>`, `bidi_visual: Option<bool>`,
`cell_spacing: Option<CT_TblWidth>`, `caption: Option<String>` and
`description: Option<String>`. Each name stops landing in `extra_xml` and
starts landing in a typed field. The write order is exactly the slot order
`tbl_pr_raw_boundary` already encodes, so the `xsd:sequence` contract is
unchanged.

`CT_TrPr` gains `width_before: Option<CT_TblWidth>`,
`width_after: Option<CT_TblWidth>`, `cell_spacing: Option<CT_TblWidth>` and
`hidden: Option<bool>` at slots 4, 5, 9 and 11.

`tblpXSpec` reuses `AnchorAlignH` from `crates/rdocx-oxml/src/drawing.rs:203`,
whose five variants are exactly `left`, `center`, `right`, `inside` and
`outside`. `ST_YAlign` is a separate type rather than `AnchorAlignV` because
`tblpYSpec` adds `inline`, and folding that into an `Option<AnchorAlignV>` plus
a boolean would make the contradictory state representable.

`w:tblpPr` is modeled and authored here and consumed by layout in F-268b. That
intermediate state is deliberate. Until F-268b lands, a floating table
round-trips exactly and renders in the flow, which is what it does today.

### 2. Author it, `crates/rdocx/src/table.rs`

```rust
pub struct TableFloatPosition {
    pub horizontal_anchor: TableAnchor,
    pub vertical_anchor: TableAnchor,
    pub horizontal: TableFloatX,
    pub vertical: TableFloatY,
    pub distance_from_text: TableTextDistance,
}
pub enum TableAnchor { Margin, Page, Text }
pub enum TableFloatX { Align(HorizontalAlign), Offset(Length) }
pub enum TableFloatY { Inline, Align(VerticalAlign), Offset(Length) }
pub struct TableTextDistance { pub top: Length, pub right: Length,
                               pub bottom: Length, pub left: Length }
pub enum TableOverlap { Never, Allow }

impl Table<'_> {
    pub fn set_float_position(&mut self, p: Option<TableFloatPosition>) -> Result<()>;
    pub fn set_overlap(&mut self, overlap: Option<TableOverlap>);
    pub fn set_bidi_visual(&mut self, value: Option<bool>);
    pub fn set_cell_spacing(&mut self, spacing: Option<Length>) -> Result<()>;
    pub fn set_caption(&mut self, caption: Option<&str>) -> Result<()>;
    pub fn set_description(&mut self, description: Option<&str>) -> Result<()>;
}
impl Row<'_> {
    pub fn set_width_before(&mut self, width: Option<TableWidth>) -> Result<()>;
    pub fn set_width_after(&mut self, width: Option<TableWidth>) -> Result<()>;
    pub fn set_cell_spacing(&mut self, spacing: Option<Length>) -> Result<()>;
    pub fn set_hidden(&mut self, value: Option<bool>);
}
```

`TableRef` and `RowRef` gain the matching readers. Every checked setter
validates before publication, matching the F-257 rule that an invalid value
leaves the document bytes unchanged. `has_unmodeled_properties` narrows at
`crates/rdocx/src/table.rs:1652` and `:2019` by exactly the six table names and
four row names now owned.

`set_width_mode` already covers `TableWidth::{Auto, Fixed, Percentage}` and
`set_layout` already writes `w:tblLayout`. This story makes those two settings
mean something in layout rather than adding new width surface.

### 3. Lower it, `crates/rdocx-layout/src/table.rs` and `block.rs`

`TableBlock` (`crates/rdocx-layout/src/block.rs:110`) gains
`pub bidi_visual: bool`. `TableRow` (`:141`) gains `pub offset_left: f64`, the
resolved sum of the `w:gridBefore` columns and `w:wBefore`.

When `bidi_visual` is set, `layout_table_inner`
(`crates/rdocx-layout/src/table.rs:271`) reverses `col_widths` and the visual
cell order inside each row **after** merge resolution, leaving `TableSemantics`
in logical order so the structure tree, body fragments and accessibility
ownership stay logical. Reversal happens once at lowering rather than inside
`render_table_row`, which keeps direction out of the painting path for every
table. Border segment conflict resolution runs on the reversed physical
segments, which is the visual result Word produces.

Autofit is a new sibling of `compute_column_widths` (`:716`):

```rust
fn autofit_column_widths(
    tbl: &CT_Tbl,
    available_width: f64,
    styles: &CT_Styles,
    input: &LayoutInput,
    media: &MediaRegistry,
    fm: &mut FontManager,
    num_state: &mut NumberingState,
    diagnostics: &mut Vec<Diagnostic>,
    path: &[usize],
) -> Result<Option<Vec<f64>>>
```

It returns `None`, and `layout_table_inner` falls through to the existing
`compute_column_widths`, unless **both** hold: the effective `w:tblLayout` is
autofit or absent, **and** the effective `w:tblW` type is `auto` or absent.
When it engages it measures each cell twice through the existing cell content
path, once at a wide trial width for the maximum content width and once at a
minimal trial width for the minimum, distributes the available width by the
minimum-plus-proportional-slack rule, and clamps the result to
`available_width`. Per-cell `w:tcW` participates as a preferred width between
the measured minimum and maximum.

The second condition is narrower than the literal ECMA default, which makes
autofit apply whenever `w:tblLayout` is absent. The narrowing is the settled
S74 decision and it is load-bearing twice. It keeps the seven samples
immovable, and it keeps private Word corpus tables that carry
`w:tblW w:w="0" w:type="auto"` out of the new path, so this sprint does not
absorb an unreviewed corpus geometry delta. Widening the predicate is a named
future story, recorded in the parent plan, not a deferred obligation here.

`w:tblCellSpacing` adds its resolved gap between adjacent cell content boxes
and halves it at the table edge, which is Word's model. `w:hidden` is a
round-trip and reader fact only.

`resolve_base_table_properties` (`:612`) extends to carry layout mode, width
type, `bidi_visual` and the float position through table-style inheritance, so
a style-declared value resolves base-first exactly like the properties already
there.

### Deliberately out of scope

- **Floating placement.** F-268b. This story models and authors `w:tblpPr` and
  `w:tblOverlap` and leaves `crates/rdocx-layout/src/paginator.rs` untouched.
- **`w:cantSplit` as a distinction.** The row loop at
  `crates/rdocx-layout/src/paginator.rs:584` already treats every row as
  atomic, which is `cantSplit` behaviour for every row. Making the flag mean
  something requires implementing row splitting for the default case, which is
  the opposite change and would move sample geometry.
- **Binding parity.** `crates/rdocx-py/src/table.rs` and
  `crates/rdocx-wasm/src/lib.rs` expose a python-docx parity subset. F-X130
  documents the public surface separately.

## Rejected alternatives

- Reuse `ST_RelativeFromH` and `ST_RelativeFromV` directly as the parsed
  `w:tblpPr` anchor types. Five of their eight variants cannot be written by
  `w:tblpPr`, so the parse type would admit states the schema forbids.
- Fold `tblpYSpec="inline"` into an `Option<AnchorAlignV>` plus a boolean. It
  makes an inline spec that also carries an alignment representable.
- Reverse columns inside `render_table_row` for `w:bidiVisual`. It puts
  direction in the hot painting path for every table rather than once at
  lowering.
- Reverse `TableSemantics` alongside the visual order. Logical cell ownership
  is what the structure tree and accessibility contract are built on, and
  `docs/hld/08-rendering-spec.md:602` says ownership does not change.
- Make autofit the literal ECMA default. It moves six of the seven samples in
  a story that does not hold the exclusive harness baseline.
- Binary-search a continuous column scale for autofit. The measured minimum and
  maximum content widths give the answer directly, and a search would make
  table layout cost depend on content.
- Measure autofit content with a second approximate measurer rather than the
  production cell path. It would drift from the line breaker, fonts, margins
  and diagnostics that pagination uses, which is the same reason F-259 rejected
  it.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| golden | `fixed_autofit_and_nested_table_geometry_matches_reviewed_word_pages` | One in-code document holding a fixed-grid table, an auto-width autofit table and a nested table renders in deterministic font mode to a pinned page count, pinned per-row origins and pinned column widths. |
| round-trip | `table_and_row_advanced_properties_survive_reopen` | Every new table and row property reopens with the same typed value, in `tbl_pr_raw_boundary` and `tr_pr_raw_boundary` slot order, with unrelated producer XML preserved byte for byte. |
| unit | `tbl_ppr_attribute_matrix_is_prefix_tolerant_and_writes_a_fixed_prefix` | All ten `w:tblpPr` attributes parse under an alias prefix and write as `w:`. An unrecognised value falls back without inventing a position. |
| unit | `autofit_engages_only_for_an_auto_width_autofit_table` | The engagement predicate is false for a `dxa` width, false for an explicit `fixed` layout, and true only for an `auto` or absent width with autofit or absent layout. |
| unit | `autofit_distributes_available_width_between_measured_minima_and_maxima` | Column widths respect measured minimum and maximum content widths and sum to the available width. |
| regression | `a_bidi_visual_table_reverses_columns_without_changing_logical_cell_order` | Painted cell origins reverse while `TableSemantics` rows, cells and structure identifiers stay logical. |
| regression | `grid_before_and_width_before_offset_the_row_without_moving_the_table` | A leading omission shifts only that row's first cell. The table origin, table width and other rows are unchanged. |
| regression | `an_authored_dxa_table_width_never_enters_the_autofit_path` | A table built by `Document::add_table` produces the same column widths before and after this story. Named guard for hash-harness fact 1. |
| regression | `a_corpus_shaped_auto_width_table_without_explicit_autofit_keeps_its_grid` | A table carrying `w:tblW w:w="0" w:type="auto"` with an absent `w:tblLayout` and a complete grid still enters the autofit path only under the settled predicate, and its resolved widths are recorded. |
| failure | `checked_table_and_row_setters_reject_invalid_values` | Out-of-range twips, an out-of-range percentage and an empty caption or description leave the document bytes unchanged. |

The **test gate** is the golden test
`fixed_autofit_and_nested_table_geometry_matches_reviewed_word_pages`.

Every test is a module added to the existing
`crates/rdocx/tests/integration_test.rs` and
`crates/rdocx/tests/regression_test.rs` entrypoints. No new test binary and no
binary fixture.

Evidence is source-built structural assertion plus the already-installed
deterministic raster path on `/private/tmp/rdocx-s73-bin`, which is Poppler
26.01.0 and LibreOffice 26.2.5.2. Nothing in this plan needs Word GUI
automation. Anything that would is a tracked follow-up, not a blocker.

## HLD impact

- `docs/hld/02-scope-and-non-goals.md`
- `docs/hld/08-rendering-spec.md`
- `docs/hld/10-bindings-spec.md`
- `docs/hld/12-testing-strategy.md`
- `docs/hld/14-development-backlog.md`

`docs/hld/08-rendering-spec.md` carries two edits. The "Autofit" section gains
a Word table-layout counterpart to the PresentationML shape ladder. The
sentence at `:602`, that right-to-left tables reverse visual column placement
without changing logical cell ownership, is currently satisfied only by
`crates/rpptx-render/src/lib.rs:585` for the DrawingML `a:tbl`. This story is
what makes it true for WordprocessingML, and the section says so explicitly
rather than leaving one sentence covering two grammars.

`docs/hld/02-scope-and-non-goals.md` moves the DOCX-035 row at `:242` from
`unsupported` to `partial` with a `boundary:F-268b` marker. It is not
`complete` until F-268b lands.

## Risk routing

Matched rows from `.claude/skills/risk-routing.md`:

- **Unit conversion, `Twips`, `Points`.** `tblpX`, `tblpY`, the four from-text
  distances, `wBefore`, `wAfter` and `tblCellSpacing` are authored in twips and
  consumed in points. Extra check: constructors keep truncating with `as i64`,
  assert the exact point and twip boundary values on both sides of every new
  conversion, and declare the harness result below.
- **Layout, pagination, line breaking, text shaping.** Extra check: every
  golden and geometry assertion runs in deterministic font mode. Any baseline
  movement is a labelled commit with a stated delta, never incidental.
  `python3 scripts/golden_png_harness.py --check` runs against the pinned
  Poppler 26.01.0 oracle.
- **Any parser or serialiser.** Extra check: `w:tblPr` slots 1, 2, 3, 8, 15 and
  16 and `w:trPr` slots 4, 5, 9 and 11 keep their `xsd:sequence` positions on
  write. Read is prefix tolerant, write is a fixed `w:` prefix. A round-trip
  test proves `capture_element` still preserves every subtree this story does
  not model.
- **Public API of a published crate.** Extra check: the additions are purely
  additive, so the semver impact is minor. Run `cargo publish --dry-run` and
  the `.crate` size assertion for `rdocx-oxml` and `rdocx`. No surface beyond
  the ten setters, their readers and the seven public types.
- **A new trait, generic parameter, crate, module or file.** No new crate,
  module or file. Three new enums and one new struct in
  `crates/rdocx-oxml/src/table.rs`, and six new public types in
  `crates/rdocx/src/table.rs`. No new trait and no new generic parameter.
  `ST_YAlign` exists because `tblpYSpec` carries a sixth value `AnchorAlignV`
  does not, and reusing `AnchorAlignH` for `tblpXSpec` is what keeps the count
  at three rather than four.

Not matched: theme colour, crate dependency graph, bundled fonts and assets,
WASM and PyO3 bindings, feature flags, external oracle comparison beyond the
already pinned rasteriser, release scripting, file moves.

## Hash harness

**Expected unchanged.** `scripts/hash_baseline.json` stays at its current 49
entries and `scripts/golden_pixel_manifest.json` is unmoved.

Six of the seven samples in `scripts/hash_harness.py:33` contain tables.
Counting `add_table` calls in `crates/rdocx/examples/generate_all_samples.rs`:
`feature_showcase` 8, `invoice` 4, `quote` 3, `report` 3, `contract` 2,
`proposal` 2, `letter` 0. Those six are the ones that could move. Four
checkable facts say they do not. The fifth belongs to F-268b.

1. **Autofit cannot engage.** `Document::add_table` writes
   `CT_TblWidth::dxa(col_width * cols)` unconditionally at
   `crates/rdocx/src/document.rs:14333`, and every sample table is built
   through it. The settled engagement predicate requires an `auto` or absent
   `w:tblW` type, so no sample table reaches `autofit_column_widths`. The
   regression test `an_authored_dxa_table_width_never_enters_the_autofit_path`
   is the guard for exactly this.
2. **No sample is bidirectional.** The generator contains no `w:bidiVisual` and
   no facade call can have set it before this story. `bidi_visual` defaults to
   false, which is today's ordering.
3. **No sample omits grid columns or spaces cells.** The generator never calls
   `set_row_grid_omissions`, and `w:wBefore`, `w:wAfter` and
   `w:tblCellSpacing` do not appear. `TableRow::offset_left` is 0.0 for every
   sample row.
4. **The one `cantSplit` in the samples is inert.**
   `crates/rdocx/examples/generate_all_samples.rs:498` sets it on a
   `feature_showcase` row, and the paginator at
   `crates/rdocx-layout/src/paginator.rs:584` already refuses to split any row.
   Row splitting is out of scope for both children, so that authored flag keeps
   producing the geometry it produces today.

Modeling `w:tblpPr` and `w:tblOverlap` changes no geometry in this story,
because nothing reads them until F-268b.

If any of those four is falsified during implementation, that is a stop under
the escalation trigger in `.claude/WORKFLOW.md`, not a re-record. The baseline
is exclusive to one story per sprint wave and F-268a does not hold it. The same
rule applies to the private Word corpus gate. The narrow autofit predicate
exists so the corpus does not move, and if it moves anyway that is a finding,
not a re-record.

## Implementation checklist

- [x] Add failing round-trip coverage for `w:tblpPr`, `w:tblOverlap`,
      `w:bidiVisual`, `w:tblCellSpacing`, `w:tblCaption`, `w:tblDescription`,
      `w:wBefore`, `w:wAfter`, row `w:tblCellSpacing` and row `w:hidden`.
- [x] Model those children on `CT_TblPr` and `CT_TrPr` at their existing raw
      slots, prefix tolerant on read and a fixed `w:` prefix on write.
- [x] Prove `capture_element` still preserves every unmodeled sibling.
- [x] Add the checked `Table` and `Row` setters and the matching `TableRef` and
      `RowRef` readers, and narrow `has_unmodeled_properties` at
      `crates/rdocx/src/table.rs:1652` and `:2019`.
- [x] Add `TableBlock::bidi_visual` and `TableRow::offset_left`, and lower
      `w:bidiVisual`, `w:gridBefore`, `w:wBefore`, `w:gridAfter`, `w:wAfter`
      and `w:tblCellSpacing` into them.
- [x] Extend `resolve_base_table_properties` to inherit layout mode, width
      type, bidirectional order and float position base-first.
- [x] Add `autofit_column_widths` with the two-condition engagement predicate
      and the minimum-plus-proportional-slack distribution.
- [x] Record the golden geometry in deterministic font mode and pin it.
- [x] Run the impacted oxml, layout, facade, golden, clippy, prose, hash and
      golden-PNG gates and confirm the 49 entries and the pixel manifest are
      unmoved.
- [x] Confirm the private Word corpus gate is unmoved, and stop rather than
      re-record if it is not.

## Open questions

None. Resolved in the S74 consolidated design round.
