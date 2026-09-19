# F-266c, Character grid and vertical text

**Status**: completed
**Sprint**: S74
**Size**: L
**Depends on**: F-266a, F-269

## Problem

The last two behaviours DOCX-033 names are the East Asian character grid and
vertical text. One is unmodelled, the other is authorable but unrendered.

**`w:docGrid` and `w:eastAsianLayout` are unmodelled.** `w:docGrid` exists in
the workspace only as a `w:sectPr` child-ordering slot at
`crates/rdocx-oxml/src/document.rs:825` and as a numbering-level ordering entry
at `crates/rdocx-oxml/src/numbering.rs:173`. A round-trip test at
`crates/rdocx/tests/integration_test.rs:611` proves the element survives as raw
XML, which is exactly the point: it survives and does nothing.
`w:eastAsianLayout` is thinner still, a bare raw schema slot number at
`crates/rdocx-oxml/src/properties.rs:2796`. Neither has a typed field, a public
setter or a layout effect, so a Word document authored on a character grid
reflows to Latin metrics the moment it is laid out here.

**Vertical text is authorable and unrendered.** `CellTextDirection` at
`crates/rdocx/src/table.rs:102` covers all six `w:tcPr/w:textDirection` values,
with a setter at `table.rs:1523` and a reader at `table.rs:2146`, and
`crates/rdocx-oxml/src/table.rs:1418` parses the element. The round trip is
proven at `crates/rdocx/tests/integration_test.rs:7880` to `integration_test.rs:8072`.
None of that reaches geometry. `crates/rdocx-layout/src/table.rs` never consults
the value, and `oxml-layout` has no writing-mode concept at all: a grep for
vertical across `crates/rdocx-layout/src/` returns only vertical merge and
vertical anchor, both unrelated. A cell set to `tbRl` lays out and paints
horizontally today.

The transposed-box and `Group` rotation approach that
`docs/hld/08-rendering-spec.md` specifies under "**Vertical text**" exists, but
it is the DrawingML shape path in `rpptx-render`. Word tables never enter it.

## Spec reference

- `docs/hld/08-rendering-spec.md`, the "**Vertical text**" paragraph in "Text in
  a shape", for the transposed same-centre box, the `Group` wrapper, the 90 and
  -90 degree rotations and the upright-stacking fallback diagnostics.
- `docs/hld/08-rendering-spec.md`, "Tables", and its "Caller-width Word block
  measurement" subsection, for how cell content is measured before placement.
- `docs/hld/02-scope-and-non-goals.md`, "Modern DOCX capability matrix", the
  DOCX-033 row, and the shape-path rows recording that `eaVert` and
  `mongolianVert` upright stacking fall back to rotated text.
- `docs/hld/04-opc-and-packaging.md`, the namespace-aware modeled part and
  schema-ordered serialization sections, for prefix-tolerant reading, fixed
  prefix writing and `xsd:sequence` child order in `w:sectPr` and `w:rPr`.
- `docs/hld/12-testing-strategy.md`, "The golden-PNG gate" and the two in-code
  golden paragraphs that follow it, for the recorded-digest pattern with no
  committed fixture.
- `docs/hld/14-development-backlog.md`, "F-266, International and vertical
  typography (L)".

## Prerequisites delivered by earlier stories

F-266a and F-269 both complete before this story starts.

From **F-266a** this story consumes `TextScript::Hangul` and `TextScript::Kana`,
the slot-aware `resolve_font_family` with `WordFontSlot`, the three bundled
subset faces, and the shared deterministic golden fixture module holding
`mixed_script_page_matches_the_pinned_geometry_and_reading_order`.

From **F-269** this story consumes the authoring and preservation of
`w:sectPr/w:textDirection`. F-269 owns section properties, so it owns that
element's typed field, setter, reader and round trip. **This story owns only its
render projection.** The two must not overlap in the diff.

`crates/rdocx-oxml/src/properties.rs` is split into separate paragraph-property
and run-property modules before wave 1. The `w:eastAsianLayout` field lands in
**the run property module**.

## Approach

### E. East Asian layout and the character grid

`w:eastAsianLayout` becomes a typed `CT_RPr` field in the run property module:

```rust
/// East Asian typography layout (eastAsianLayout).
pub east_asian_layout: Option<CT_EastAsianLayout>,

pub struct CT_EastAsianLayout {
    pub id: Option<i32>,
    pub combine: Option<bool>,
    pub combine_brackets: Option<ST_CombineBrackets>,
    pub vert: Option<bool>,
    pub vert_compress: Option<bool>,
}
```

`w:docGrid` becomes a typed `CT_SectPr` field in
`crates/rdocx-oxml/src/document.rs`:

```rust
/// Character grid for the section (docGrid).
pub doc_grid: Option<CT_DocGrid>,

pub struct CT_DocGrid {
    pub grid_type: Option<ST_DocGrid>,   // default | lines | linesAndChars | snapToChars
    pub line_pitch: Option<Twips>,
    pub char_space: Option<i32>,
}
```

Both are prefix-tolerant on read and write with the fixed `w:` prefix at their
existing schema slots, which the raw-retention tables already reserve at
`crates/rdocx-oxml/src/properties.rs:2796` and
`crates/rdocx-oxml/src/document.rs:825`.

The layout effect in `crates/rdocx-layout/src/engine.rs` is deliberately narrow.
`linePitch` participates in line advance and `charSpace` in per-character
advance **only** for the `lines`, `linesAndChars` and `snapToChars` grid types.
The `default` type keeps today's geometry with no new arithmetic on the path at
all, which is what makes the hash prediction below provable rather than hopeful.
`w:eastAsianLayout` `combine` lays its run's text into one base-character
advance with the requested brackets, and `vert` rotates that run within a
horizontal line. `vertCompress` narrows the rotated run to the base advance.

### F. Vertical text rendering

`w:tcPr/w:textDirection`, already authorable through
`crates/rdocx/src/table.rs:1523`, gains the render projection that
`docs/hld/08-rendering-spec.md` already specifies for shapes. Cell content is
laid out in a same-centre transposed box and the result is wrapped in a `Group`:

| Value | Projection |
|---|---|
| `lrTb` | Today's path, unchanged, no `Group` |
| `tbRl`, `tbRlV` | Transposed box rotated 90 degrees |
| `btLr`, `lrTbV` | Transposed box rotated -90 degrees |
| `tbLrV` | Transposed box rotated -90 degrees |

Upright stacked CJK is out of scope and stays visible, recording the diagnostic
`east Asian vertical text rendered as rotated vertical text`. That matches the
fallback already documented for the shape path at
`docs/hld/02-scope-and-non-goals.md:108`, so the product says one thing about
upright stacking rather than two.

Row height and the table's equal-height pass measure the transposed box, so a
vertical cell's content height drives the column width it needs and its height
comes from the row. `crates/rdocx-layout/src/table.rs` is where that measurement
change lands.

`w:sectPr/w:textDirection` takes the same projection at the section level, over
the typed field F-269 delivers. This story reads that field and never writes it.

### Files the diff touches

- The run property module, for `w:eastAsianLayout`
- `crates/rdocx-oxml/src/document.rs`, for `w:docGrid` on `CT_SectPr`
- `crates/rdocx-layout/src/engine.rs`
- `crates/rdocx-layout/src/table.rs`
- `crates/rdocx/src/run.rs`
- `crates/rdocx/src/document.rs`
- `crates/rdocx/tests/integration_test.rs`
- `crates/rdocx/tests/regression_test.rs`
- `docs/hld/02-scope-and-non-goals.md`, for the final DOCX-033 classification

No new crate, module or file.

## Rejected alternatives

- **Add a `writing_mode` concept to `oxml-layout`.** Rejected. The transposed
  box plus `Group` rotation already exists for the shape path and reuses the
  `Group` primitive, so no layout-engine concept is added and one approach
  covers both families.
- **Implement upright stacked vertical CJK.** Rejected. It is already a
  documented fallback for the shape path at
  `docs/hld/02-scope-and-non-goals.md:108`, and matching it keeps one answer.
- **Apply `w:docGrid` arithmetic to the `default` grid type as a no-op
  multiply.** Rejected. A no-op that runs is still a floating-point path on
  every existing line, which is precisely how a silent hash delta is created.
  The `default` type takes the existing branch untouched.
- **Own `w:sectPr/w:textDirection` authoring here.** Rejected in the S74 round.
  F-269 owns section properties, and two stories writing the same element is the
  semantic conflict `.claude/WORKFLOW.md` says never to resolve automatically.
- **Model `w:docGrid` on the layout input rather than on `CT_SectPr`.**
  Rejected. It is a section property in the schema, and putting it elsewhere
  would break the round trip that already preserves it.
- **Defer the character grid and ship only vertical text.** Rejected. Both are
  section and run level East Asian metrics that change line advance, and they
  share the measurement code they would otherwise each duplicate.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| golden | `grid_and_vertical_page_matches_the_pinned_geometry_and_reading_order` | **The test gate.** The shared deterministic fixture gains one page carrying a `linesAndChars` gridded Japanese section, a table with `tbRl` and `btLr` cells alongside an `lrTb` control, a `combine` run and a Latin control, built through the public facade with `FontManager::new_deterministic`. Its own digest is recorded as a constant. The F-266a and F-266b digests are asserted unmoved in the same module |
| round-trip | `doc_grid_reopens_on_its_section` | Grid type, `linePitch` and `charSpace` author, save, reopen and return typed through the public section surface, in the correct `w:sectPr` sequence position, with unrelated producer XML byte for byte |
| round-trip | `east_asian_layout_reopens_as_modeled_state` | `w:id`, `w:combine`, `w:combineBrackets`, `w:vert` and `w:vertCompress` round-trip typed at the correct `EG_RPrBase` position, prefix-aliased on read and fixed `w:` on write |
| round-trip | `an_unmodelled_doc_grid_attribute_survives_a_noop_save` | The raw-carrier path keeps a producer attribute the typed model does not own |
| regression | `the_default_doc_grid_type_does_not_change_line_advance` | Named as the failure it prevents. A `default` grid produces geometry identical to no grid at all |
| regression | `a_horizontal_cell_beside_a_vertical_cell_keeps_its_geometry` | Adding a rotated neighbour does not move an `lrTb` cell |
| regression | `vertical_cell_text_is_extracted_in_logical_order` | Rotation changes painting only. `ActualText`, SVG text and round-trip XML stay logical |
| integration | `every_supported_cell_text_direction_renders_at_its_rotation` | All six `w:tcPr/w:textDirection` values project to the table above, and upright stacking records its diagnostic |
| integration | `a_vertical_cell_drives_the_row_height_from_its_transposed_box` | Measurement uses the transposed box, so row height and the equal-height pass are correct |
| integration | `a_gridded_section_snaps_line_advance_to_its_line_pitch` | `lines`, `linesAndChars` and `snapToChars` each change advance in their own documented way |
| integration | `section_text_direction_renders_over_the_property_f269_delivers` | The section-level projection reads F-269's typed field and writes nothing |

The **test gate** is the golden
`grid_and_vertical_page_matches_the_pinned_geometry_and_reading_order`.

Every rendering assertion uses `FontManager::new_deterministic`. No baseline is
recorded against system fonts. No binary fixture is added. New tests join the
existing `crates/rdocx/tests/integration_test.rs` and `regression_test.rs`
entrypoints, and crate-local units join the existing in-file `#[cfg(test)]`
modules.

## HLD impact

- `docs/hld/02-scope-and-non-goals.md`
- `docs/hld/08-rendering-spec.md`
- `docs/hld/04-opc-and-packaging.md`
- `docs/hld/12-testing-strategy.md`
- `docs/hld/14-development-backlog.md`

This is the last child to close, so it carries the final DOCX-033 row. The row
must say honestly that shipped deterministic font coverage for Hebrew, Korean
and Japanese is the fixture repertoire only, not the full families, and that
upright stacked vertical CJK falls back to rotated text.

## Risk routing

Three rows of `.claude/skills/risk-routing.md` match.

1. **Layout, pagination, line breaking, text shaping.** Read
   `docs/hld/08-rendering-spec.md`. Extra checks: every baseline uses
   `FontManager::new_deterministic`. Both the character grid and the transposed
   box change line advance and cell measurement, so the paginator and the table
   equal-height pass are exercised deliberately. The F-266a and F-266b recorded
   digests are asserted unmoved.

2. **Any parser or serialiser.** Read `docs/hld/04-opc-and-packaging.md` and
   `docs/hld/06-presentationml-model.md`. Extra checks: `w:docGrid` and
   `w:eastAsianLayout` are prefix-tolerant on read and fixed `w:` prefix on
   write, each lands in its correct `xsd:sequence` position in `w:sectPr` and
   `EG_RPrBase`, and each carries a round-trip test proving `capture_element`
   preserved its unmodelled subtree byte for byte. The existing raw-XML
   round trip at `crates/rdocx/tests/integration_test.rs:611` must keep passing
   against the newly typed `w:docGrid`.

3. **Public API of a published crate.** Read `docs/hld/10-bindings-spec.md` and
   the `CLAUDE.md` structural rules. Extra checks: `CT_DocGrid`,
   `CT_EastAsianLayout`, `ST_DocGrid` and `ST_CombineBrackets` are additive to
   `rdocx-oxml`, and the `CT_RPr` and `CT_SectPr` fields are additive to
   non-`#[non_exhaustive]` structs, which is stated. Run
   `cargo publish --dry-run` for `rdocx-oxml`, `rdocx-layout` and `rdocx`, with
   the archive-size assertion.

Not matched: unit conversion, theme colour tint and shade, bundled fonts, crate
dependency graph, feature flags, file moves, new trait or generic, external
oracle, release scripting. The golden is a recorded geometry digest and needs no
rasteriser.

## Hash harness

**Expected unchanged. All 49 entries and all 7 golden-PNG entries.**

The seven `samples/` documents generated by
`crates/rdocx/examples/generate_all_samples.rs` are Latin, and this story adds
nothing to them. Taking the change groups in turn:

- **`w:eastAsianLayout`.** Absent-by-default. `CT_RPr::east_asian_layout`
  defaults to `None`, and the combine and vert branches are entered only when
  the element exists. No sample carries it.
- **`w:docGrid`.** Present in one sample's raw XML round-trip fixture at
  `crates/rdocx/tests/integration_test.rs:611`, which is a test string and not a
  generated sample. None of the seven generated documents sets a grid. Where a
  grid is absent, or where its type is `default`, the advance calculation takes
  the existing branch with no new arithmetic, which is why the `default` type
  was deliberately kept off the new path and is pinned by
  `the_default_doc_grid_type_does_not_change_line_advance`.
- **Vertical text.** `lrTb` is the default and keeps today's path with no
  `Group` wrapper. Every sample table cell is `lrTb`, so no sample enters the
  transposed-box path. The measurement change in
  `crates/rdocx-layout/src/table.rs` is guarded on a non-`lrTb` direction.
- **Section text direction.** Read-only here, over F-269's typed field, and no
  sample sets it.

This story bundles no font and touches no `oxml-layout` file, so the
coverage-fallback risk that made F-266a's font commit sensitive does not exist
here.

This story does not claim the sprint's exclusive baseline re-record, and it
asserts both sibling golden digests unmoved rather than re-recording them.

## Implementation checklist

- [x] Confirm F-266a and F-269 are `done`, that the shared golden fixture module
      and both recorded digests exist, and that F-269 delivered the typed
      `w:sectPr/w:textDirection` field this story only reads.
- [x] Confirm the run-property module split has landed and add
      `CT_RPr::east_asian_layout` there.
- [x] Add `CT_EastAsianLayout` and `ST_CombineBrackets` with the parser and
      serialiser at the `w:eastAsianLayout` schema slot. **Deviation**: F-265
      had already landed `CT_EastAsianLayout` with `combine_brackets` as a
      `String`, so this story consumed it rather than replacing that public
      field with an `ST_CombineBrackets` nobody had asked for.
- [x] Add `CT_DocGrid` and `ST_DocGrid` on `CT_SectPr` in
      `crates/rdocx-oxml/src/document.rs` at the `w:docGrid` slot, keeping the
      existing raw round-trip test green.
- [x] Add the facade setters and readers in `crates/rdocx/src/run.rs` and
      `crates/rdocx/src/document.rs`. **Deviation**: `run.rs` already carried
      `set_east_asian_layout_value` and `east_asian_layout` from F-265, so only
      the section grid accessors were new.
- [x] Apply `linePitch` and `charSpace` to advance for `lines`,
      `linesAndChars` and `snapToChars` only, leaving `default` on the existing
      branch.
- [x] Apply `combine`, `combineBrackets`, `vert` and `vertCompress` to run
      advance and rotation.
- [x] Project `w:tcPr/w:textDirection` into a transposed same-centre box wrapped
      in a `Group`, with the rotations in the table above and the
      upright-stacking diagnostic.
- [x] Make row height and the equal-height pass measure the transposed box in
      `crates/rdocx-layout/src/table.rs`.
- [x] Project `w:sectPr/w:textDirection` the same way, reading F-269's field and
      writing nothing.
- [x] Add every test in `## Test plan` to the existing entrypoints.
- [x] Record this story's golden digest, with its reason, in the test and in
      `docs/hld/12-testing-strategy.md`, and assert both sibling digests unmoved.
- [x] Update DOCX-033 in `docs/hld/02-scope-and-non-goals.md` to its final
      classification, stating the fixture-repertoire font coverage and the
      upright-stacking fallback honestly.
- [x] Run `/verify`, plus `cargo test -p oxml-layout --no-default-features`,
      `cargo check --target wasm32-unknown-unknown -p rdocx-wasm -p rpptx-wasm`,
      and `cargo publish --dry-run` with the archive-size assertion.
      **Deviation**: `cargo publish --dry-run` verifies for `rdocx-oxml`, which
      packages 32 files at 2.3 MiB and 358 KiB compressed, and fails for
      `rdocx-layout` and `rdocx` because they resolve `rdocx-oxml` from the
      registry and 0.14.0 is not published yet. That failure predates this
      story and is the known worker-worktree artifact.

## Deviations

- The seven East Asian paragraph toggles F-264 left raw-preserved and named
  for this story, `w:kinsoku`, `w:wordWrap`, `w:overflowPunct`,
  `w:topLinePunct`, `w:autoSpaceDE`, `w:autoSpaceDN` and `w:snapToGrid`, are
  typed on `CT_PPr` with a public paragraph surface, which the approved plan
  did not list. They are what DOCX-033's `Create` column turns on, and this
  story is the last child, so leaving them raw would have left the row
  unclosable. `w:snapToGrid` gets its render projection here as the gate on
  the character grid. The other six are authorable with the render position
  the row's evidence cell states.
- Files touched beyond `## Files the diff touches`:
  `crates/rdocx-layout/src/convert.rs`, `notes.rs` and `paginator.rs` for the
  line advance, the note story exclusion and the render projections,
  `crates/rdocx-oxml/src/paragraph_properties.rs` and
  `crates/rdocx/src/paragraph.rs` for the toggles, and
  `scripts/test_sprint_workflow.py` for the DOCX-033 owner exclusion the
  closing row requires.
- Headers, footers and note text are laid out off the character grid. The plan
  did not discuss them. The narrowing is recorded in
  `docs/hld/08-rendering-spec.md`.
- A vertical section drops its column tracks and records a diagnostic, because
  transposing each track about its own centre while the page rotates about one
  is a wrong picture rather than a missing one. Recorded in the rendering spec
  and in the DOCX-033 evidence cell.

## Open questions

None. Resolved in the S74 consolidated design round.
