# F-269, Complete section page semantics

**Status**: completed
**Sprint**: S74
**Size**: L
**Depends on**: F-250, F-251

## Problem

`CT_SectPr` models thirteen of the twenty-two `w:sectPr` children. The struct at
`crates/rdocx-oxml/src/document.rs:162` carries page size, margins, gutter,
header and footer distance, orientation, break type, columns, page-number
restart, title page, and the header and footer references. Everything else falls
into `extra_xml` with a schema-slot anchor. `CT_SectPr::raw_child_schema_slot`
at `crates/rdocx-oxml/src/document.rs:816` is the complete list of what is
preserved and not modeled: `w:footnotePr` and `w:endnotePr` at slot 2,
`w:paperSrc`, `w:pgBorders` and `w:lnNumType` at slot 5, `w:formProt`,
`w:vAlign` and `w:noEndnote` at slot 7, and `w:textDirection`, `w:bidi`,
`w:rtlGutter`, `w:docGrid` and `w:printerSettings` at slot 8. None of those has
a public getter or setter on `SectionRef` at `crates/rdocx/src/document.rs:2654`
or on `Section` at `crates/rdocx/src/document.rs:2741`.

Columns are worse than absent, and closing that defect is the centre of this
story. They are modeled in XML through `CT_Columns` at
`crates/rdocx-oxml/src/document.rs:44`, they round-trip, and
`Section::set_columns` at `crates/rdocx/src/document.rs:2849` authors them, but
`sect_pr_to_geometry` at `crates/rdocx-layout/src/engine.rs:7194` never reads
the field. `PageGeometry` at `crates/rdocx-layout/src/paginator.rs:60` has eight
f64 fields and no column state, so `content_width()` is always the full text
measure and every section lays out as one column. The public
`Section::set_columns` surface therefore writes valid XML that this workspace
renders incorrectly. `docs/hld/08-rendering-spec.md` already claims columns
reach pagination, which makes this spec drift rather than an omission.

Mirrored margins and book fold are not `w:sectPr` children at all. They live in
`w:settings` and today appear only as ordering entries in the `ORDER` table at
`crates/rdocx-oxml/src/settings.rs:1175` and `:1210`, with no typed model.

## Spec reference

- `docs/hld/02-scope-and-non-goals.md`, "Modern DOCX capability matrix", row
  DOCX-036, which is `partial` with this F-ID as its owner, and the adjacent
  DOCX-030 and DOCX-032 through DOCX-037 rows that bound what this story must
  not absorb.
- `docs/hld/03-architecture.md`, "Facade conventions", for the mutable `Foo<'a>`
  and read-only `FooRef<'a>` borrow-handle idiom the new accessors follow.
- `docs/hld/04-opc-and-packaging.md`, "What transfers unmodified", for the
  section children that stay byte-preserved after this story.
- `docs/hld/08-rendering-spec.md`, "Word section geometry and page numbering",
  which names columns as reaching pagination and is the section this story
  makes true.
- `docs/hld/12-testing-strategy.md`, "Test taxonomy" for the `differential`
  gate, and "The Word corpus" for the pinned-record and regeneration pattern.
- `docs/hld/14-development-backlog.md`, "F-269, Complete section page
  semantics".

## Approach

Two separable halves. Properties that must change layout or rendering, and
properties that must only survive a round trip.

### Layout-affecting and render-affecting

**Variable-width columns and separators.** `CT_Columns` already parses `w:num`,
`w:space`, `w:equalWidth`, `w:sep` and the `w:col` children. The work is in
layout. Resolve one track list per section before pagination:

```rust
/// One resolved column track in points, left to right.
pub struct ColumnTrack { pub x: f64, pub width: f64 }
```

`PageGeometry` gains `columns: Vec<ColumnTrack>` and `column_separator: bool`.
`sect_pr_to_geometry` resolves equal-width tracks from `w:num` and `w:space`,
or explicit tracks from the `w:col` list when `w:equalWidth` is `0` or the list
is non-empty. **The single-column case must bypass the track arithmetic and keep
the exact current `content_width()` expression**, not evaluate a neutral form of
the new one. Reassociated f64 arithmetic is how a no-op refactor moves a PNG
hash. The paginator fills track `n` before track `n+1`, advances to the next
page when the last track overflows, and draws a vertical rule at each inter-track
midpoint when `w:sep` is set, reusing `PositionedElement::Line`.

Fill is unbalanced left to right. **Word's column balancing at a continuous
section break is out of scope for M24** and is a named follow-up. Recording it
here pre-explains the one differential finding a balancing fixture would
produce, so a reviewer does not read it as a defect.

**Page borders.** New `CT_PageBorders` in
`crates/rdocx-oxml/src/document.rs`, at slot 5 between `w:pgMar` and
`w:lnNumType`:

```rust
pub struct CT_PageBorders {
    pub display: Option<ST_PageBorderDisplay>,   // allPages | firstPage | notFirstPage
    pub offset_from: Option<ST_PageBorderOffset>, // page | text
    pub z_order: Option<ST_PageBorderZOrder>,     // front | back
    pub top: Option<CT_BorderEdge>,
    pub left: Option<CT_BorderEdge>,
    pub bottom: Option<CT_BorderEdge>,
    pub right: Option<CT_BorderEdge>,
    pub raw_xml: Option<Vec<u8>>,
}
```

`CT_BorderEdge` at `crates/rdocx-oxml/src/borders.rs:36` reads only `val`, `sz`,
`space` and `color` today, so modeling `w:pgBorders` with it would convert a
byte-preserved subtree into a lossy one. **F-264 owns that fix and delivers it in
wave 1**, adding ordered attribute retention for `w:shadow`, `w:frame`,
`w:themeColor`, `w:themeTint` and `w:themeShade`. This story consumes the
retaining `CT_BorderEdge` and adds no wrapper of its own, so `borders.rs` is not
in this diff.

Rendering reuses `render_border_edges` at
`crates/rdocx-layout/src/paginator.rs:3808`, which already takes `CT_BorderEdge`
values and emits dashed and solid `Line` elements. The rectangle is the page box
inset by `w:pgBorders` offsets for `offsetFrom="page"`, or the margin rectangle
for `offsetFrom="text"`. `display` selects which physical pages of the section
receive the draw.

**Line numbering.** New `CT_LineNumber` at slot 5:

```rust
pub struct CT_LineNumber {
    pub count_by: Option<u32>,
    pub start: Option<u32>,
    pub distance: Option<Twips>,
    pub restart: Option<ST_LineNumberRestart>, // newPage | newSection | continuous
}
```

The paginator counts body lines per column track, and at each `count_by`
multiple emits a right-aligned number `distance` to the left of the track,
shaped through the deterministic `FontManager`. **Decision: line numbers are a
page artifact and are excluded from the PDF reading order**, wrapped in
`PositionedElement::MarkedContent { structure: None }`. They are furniture
rather than content, and a screen reader should not read them into the prose.

**Vertical page alignment.** `w:vAlign` at slot 7, typed as `ST_VerticalJc`.
That enum at `crates/rdocx-oxml/src/table.rs:1238` gains the ECMA-376 `Both`
variant, which it is currently missing, and becomes `#[non_exhaustive]` so a
later addition is not a second breaking change. The semver impact is a breaking
change to `rdocx-oxml`, absorbed now because the workspace is at an unreleased
0.14.0 with 0.13.1 the latest on crates.io.

After a page's blocks are placed, `Center` and `Bottom` translate the body band
by the unused vertical measure or half of it. `Top` is the current behaviour and
must take the untouched code path. **`Both` preserves its source value, lays out
as `Top`, and emits a diagnostic**, matching how unsupported values are handled
elsewhere in this workspace. True vertical distribution is a named follow-up,
not a gap.

**Mirrored margins.** `w:mirrorMargins` and `w:gutterAtTop` are `w:settings`
children, and **F-270 owns their typed accessors and delivers them in wave 1**.
This story consumes that surface and adds no accessor of its own, which is what
keeps two stories out of one file. When mirroring is active, an even displayed
page swaps `margin_left` and `margin_right` and places the gutter on the inside
edge. The swap is applied inside `PageGeometry` resolution per page, so nothing
downstream learns a new concept, and it follows the displayed page number rather
than the physical one.

### Round-trip only, no layout effect

**Paper source.** New `CT_PaperSource { first: Option<u32>, other: Option<u32> }`
at slot 5. It selects a printer tray and has no on-page consequence. Pagination
must be byte-identical with and without it, and a regression asserts that.

**Book fold.** `w:bookFoldPrinting`, `w:bookFoldPrintingSheets` and
`w:bookFoldRevPrinting` are `w:settings` children whose typed accessors **F-270
owns and delivers in wave 1**. This story consumes them. **Decision: round trip
plus an explicit non-goal statement is the complete answer.** Book fold is a
print-time sheet imposition, not a page layout, and Word leaves the document's
page count and page geometry alone, so this workspace re-imposes nothing. That
is a deliberate boundary and `docs/hld/02-scope-and-non-goals.md` records it as
one.

**Section footnote and endnote configuration.** `w:footnotePr` and `w:endnotePr`
at slot 2 gain a shared typed model carrying `w:pos`, `w:numFmt`, `w:numStart`
and `w:numRestart`, plus retained raw bytes for anything else. The values become
public and survive a round trip. **Their effect on marker text, placement and
restart is F-274**, which depends on this story. F-269 delivers the model and
the authoring surface and changes no note rendering.

**Section text direction.** **This story owns `w:sectPr/w:textDirection`
authoring and preservation. F-266c owns its render projection**, and F-269
completes before F-266c starts. So `w:textDirection` gains a typed value at slot
8 with a public getter and setter and an exact round trip, and changes no
rendering here. That split is what lets the authoring surface close in M24
without waiting on the vertical typography work.

### What deliberately stays raw

`w:formProt`, `w:noEndnote`, `w:bidi`, `w:rtlGutter`, `w:docGrid` and
`w:printerSettings` keep their existing slot anchors and raw capture. `w:bidi`
and `w:rtlGutter` are the bidirectional family of DOCX-033 and belong to F-266.

### Facade

`SectionRef` gains read accessors and `Section` gains matching setters, one pair
per property family, following the existing borrow-handle pattern and the
`checked_*_section_twips` validators at `crates/rdocx/src/document.rs:2937`.
`section_has_layout` at `crates/rdocx/src/document.rs:21986` gains the
layout-affecting new fields. `has_unmodeled_section_properties` needs no change,
because the newly modeled children leave `extra_xml` by construction.

**DOCX-036 keeps its `B` binding boundary.** The frozen
`#[pyo3(signature = ...)]` constructor for `SectionInfo` at
`crates/rdocx-py/src/document.rs:598` does not change, and neither WASM nor the
CLI gains surface. Bindings are out of scope for S74.

## Rejected alternatives

- A new `crates/rdocx-oxml/src/section.rs` module. `CT_SectPr`, its slot table
  and its raw-position machinery are one contract, and splitting them puts it in
  two files. A new module also needs an explicit ask.
- A separate section-only vertical alignment enum. `ST_VerticalJc` is one XSD
  type, and two Rust enums for it increases the places a reader must look. The
  variant addition is the smaller cost.
- A retaining wrapper around `CT_BorderEdge` owned by this story. F-264 owns the
  containing grammar and delivers the retention first, so a second retention
  path would be two answers to one question.
- Duplicate typed settings accessors for mirrored margins and book fold. F-270
  owns `w:settings`, and two stories writing one file is the merge conflict the
  wave ordering exists to prevent.
- Recording a new Word-authored pinned oracle record set. See the test plan.
  Word GUI automation is unavailable on this machine, so a plan that depended on
  it would be a plan that cannot be executed.
- `PageGeometry { columns: Option<Vec<Column>> }` resolved lazily in the
  paginator. Resolving once up front keeps the single-column path textually
  identical and keeps the harness still.
- Modeling `w:docGrid`, `w:bidi` and `w:rtlGutter` here. They are DOCX-033 and
  belong to F-266. `w:textDirection` is the deliberate exception, because this
  story owns its authoring and preservation while F-266c owns its render
  projection.
- Rendering `w:sectPr/w:textDirection` here. F-266c owns the projection and
  starts after this story completes, so building it twice would be the waste the
  split exists to avoid.
- Implementing note restart and marker policy here. F-274 owns it and depends on
  this story.
- Re-imposing pages for book fold. It is a sheet arrangement at print time, and
  Word's own page count and page geometry do not change.
- A `SectionProperty` trait over the new children. No second implementer exists
  today.

## Test plan

### Evidence policy

The gate lands on evidence reproducible on this machine. **Word GUI automation
is not available here, so no new Word-authored pinned record set is recorded.**
The differential gate therefore runs against source-built structural evidence
plus the installed LibreOffice 26.2.5.2 and `pdftoppm` 26.01.0 raster path,
which are already pinned at `scripts/docx_ssim_harness.py:38` and
`crates/rdocx/tests/regression_test.rs:20152`.

Two consequences a reviewer should read as deliberate rather than as oversight:

- **F-251's existing pins are left exactly as they are.**
  `crates/rdocx/tests/integration_test.rs:57` pins `pdftotext` and `pdfinfo`
  26.09.0 while the pinned oracle path carries Poppler 26.01.0. Nothing fails
  today, because the only two tests that invoke those binaries are `#[ignore]`d
  regeneration helpers. Re-pinning them would invalidate F-251's recorded
  evidence with no human able to re-capture it, which is strictly worse than a
  stale pin on a test that does not run.
- **Word confirmation becomes a tracked human action, not a blocker.** It lands
  as a mandatory `#[ignore]` capture test plus a named follow-up entry in
  `docs/hld/14-development-backlog.md`, which is the convention
  `docs/hld/12-testing-strategy.md` already uses for GUI evidence that cannot be
  automated.

### The tests

| Category | Test | Asserts |
|---|---|---|
| differential | `section_page_semantics_match_pinned_libreoffice_render` | **The test gate.** Columns, a page border, line numbers, vertical alignment and mirrored margins, source-built, converted by pinned LibreOffice 26.2.5.2 and rasterised by pinned `pdftoppm` 26.01.0, compared at a stated DPI and SSIM floor. Deterministic font mode on our side |
| differential | `capture_f269_word_section_evidence` (`#[ignore]`) | The mandatory human-action capture path, asserting the Word build before it records anything. It is never part of the automated gate |
| round-trip | `every_section_property_survives_noop_save` | Every new child returns through the public `Section` surface with exact values, source attribute order and `xsd:sequence` child order |
| round-trip | `unmodeled_section_children_stay_byte_exact` | `w:formProt`, `w:noEndnote`, `w:bidi`, `w:rtlGutter`, `w:textDirection`, `w:docGrid` and `w:printerSettings` remain byte-identical at their slots beside the newly modeled siblings |
| unit | `variable_width_columns_parse_and_write_in_schema_order` | `w:equalWidth="0"` with explicit `w:col` tracks parses, writes and reparses identically, prefix-tolerant on read |
| unit | `section_vertical_alignment_retains_its_source_value` | `w:vAlign` round-trips every ECMA value including `both`, without collapsing to `top` |
| unit | `section_text_direction_round_trips_without_render_effect` | `w:sectPr/w:textDirection` is authorable and exact, and positioned elements are unchanged, which is the F-266c boundary |
| unit | `vertical_alignment_both_lays_out_as_top_with_a_diagnostic` | The value is preserved, the page matches `Top`, and exactly one diagnostic is emitted |
| unit | `page_border_offset_and_display_attributes_round_trip` | `offsetFrom`, `display` and `zOrder` survive, and unmodeled edge attributes are not dropped |
| integration | `variable_width_columns_place_text_in_resolved_tracks` | Text fills track 0 before track 1, the separator rule is drawn only when `w:sep` is set, deterministic font mode |
| integration | `line_numbering_count_by_and_restart_place_margin_numbers` | Numbers appear at the `countBy` interval at the `distance` offset, `newPage` and `continuous` differ, and the numbers carry no structure id |
| integration | `mirrored_margins_swap_inside_and_outside_on_even_pages` | Even displayed pages mirror, odd pages do not, and the swap follows the displayed number rather than the physical one |
| regression | `authored_columns_no_longer_lay_out_as_a_single_full_width_column` | The named drift this story closes. `Section::set_columns` followed by layout produces resolved tracks, not one full-measure column, so the defect at `crates/rdocx-layout/src/engine.rs:7194` cannot return |
| regression | `paper_source_and_book_fold_settings_do_not_change_page_geometry` | Page count, page sizes and every positioned element are identical with and without them |
| regression | `a_single_column_section_keeps_its_exact_content_width` | The one-column path produces the same f64 measure as before the column work |
| golden | `python3 scripts/hash_harness.py --check` | 49 of 49 entries unchanged |

Every new test is a module added to `crates/rdocx/tests/integration_test.rs` or
`crates/rdocx/tests/regression_test.rs`. No new file under a `tests/` directory,
and no binary fixture.

## HLD impact

- `docs/hld/02-scope-and-non-goals.md`
- `docs/hld/04-opc-and-packaging.md`
- `docs/hld/08-rendering-spec.md`
- `docs/hld/12-testing-strategy.md`
- `docs/hld/14-development-backlog.md`

## Risk routing

Matched rows from `.claude/skills/risk-routing.md`:

- **Unit conversion, `Twips`, `Emu`, `Points`, `Inches`.** `w:col/@w:w`,
  `w:col/@w:space` and `w:lnNumType/@w:distance` are new twip-valued inputs, and
  `w:pgBorders` offsets are in points. Constructors truncate with `as i64` and
  that stays. Extra check: the harness delta is declared below, and the
  single-column regression pins the unchanged measure.
- **Layout, pagination, line breaking, text shaping.** Column flow, page
  borders, line numbers, vertical alignment and mirrored margins all move
  geometry. Extra check: deterministic font mode for every baseline in this
  story, and no baseline is re-recorded incidentally.
- **Any parser or serialiser.** New `w:sectPr` children and new settings
  accessors. Extra checks: `xsd:sequence` order on write across the full slot
  table, prefix-tolerant on read with a fixed prefix on write, and a round-trip
  test proving `capture_element` still preserves the remaining unmodelled
  subtrees byte for byte.
- **Public API of a published crate.** `rdocx-oxml` and `rdocx` gain public
  types and accessors, and `ST_VerticalJc` gains the `Both` variant plus
  `#[non_exhaustive]`, which is a breaking change to `rdocx-oxml`. Extra checks:
  semver impact stated in the completion commit, `cargo publish --dry-run`, and
  the `.crate` size assertion.
- **An external oracle comparison.** The test gate is `differential`. Extra
  check: LibreOffice 26.2.5.2 and `pdftoppm` 26.01.0 are asserted in the test
  source before the comparison runs, per
  `.claude/skills/differential-testing.md`. The unavailable Word oracle is a
  recorded human action, never a silent skip.

Not matched: theme colour, crate dependency graph, bundled fonts, feature flags,
release scripting, and file moves. The WASM and PyO3 row is not matched because
DOCX-036 keeps its `B` binding boundary and no binding surface changes.

## Hash harness

**Expected unchanged. All 49 entries.**

`scripts/hash_harness.py` hashes seven samples times three OOXML parts plus one
PNG plus three PDF fingerprints. The reasoning per layer:

- **`word/document.xml`, `word/styles.xml`, `word/numbering.xml`.** The seven
  samples are generated by `crates/rdocx/examples/generate_all_samples.rs`. It
  calls `set_page_size`, `set_margins`, `set_different_first_page`,
  `section_break` and `section_landscape`, and nothing else section-related. It
  never calls `set_columns`, so `CT_SectPr::columns` stays `None`, and no sample
  authors `w:pgBorders`, `w:lnNumType`, `w:vAlign`, `w:paperSrc`, section
  `w:footnotePr` or `w:endnotePr`. Every new `CT_SectPr` field is `Option` and
  initialised to `None` in `empty`, `default_letter` and `default_a4`, and the
  writer emits nothing for `None`. `word/settings.xml` is not hashed at all, so
  the mirrored-margin and book-fold settings work cannot reach a hash even if
  the part changed.
- **The page-one PNG and the three PDF entries.** These move only if geometry
  or element order moves. The one real hazard is the column refactor of
  `sect_pr_to_geometry` at `crates/rdocx-layout/src/engine.rs:7194`. If the
  single-column case is routed through generalised track arithmetic, reassociated
  f64 operations can shift a glyph position by a float ulp and move every PNG and
  PDF hash for all seven samples with no visible change. **The implementation
  must bypass the track arithmetic when the resolved column count is one, not
  neutralise it**, and `a_single_column_section_keeps_its_exact_content_width`
  is the assertion that catches a violation before the harness does. The same
  rule applies to the vertical-alignment translation, which must not run for
  `Top`.

One cross-story note. F-264's retention change to `CT_BorderEdge` and F-270's
settings accessors land before this story, and any harness movement they cause
is theirs to declare. This story measures against the baseline as it stands
after those two integrate, not against the baseline at the sprint's start.

If the harness does move, stop. Do not re-record. The baseline is exclusive to
one story per sprint wave and this story does not hold it.

## Implementation checklist

- [x] Confirm F-264 and F-270 are integrated first. This story consumes the
      retaining `CT_BorderEdge` from F-264 and the typed settings accessors from
      F-270, and starts before either is a plan for a merge conflict.
- [x] Add `CT_PageBorders`, `CT_LineNumber`, `CT_PaperSource` and the shared
      note-properties type to `crates/rdocx-oxml/src/document.rs`, with raw
      retention for attributes they do not type.
- [x] Add the `Both` variant and `#[non_exhaustive]` to `ST_VerticalJc` at
      `crates/rdocx-oxml/src/table.rs:1238`, and fix every match arm the change
      surfaces.
- [x] Add the matching `CT_SectPr` fields, including typed `w:textDirection`,
      and remove those locals from `raw_child_schema_slot` at
      `crates/rdocx-oxml/src/document.rs:816`.
- [x] Parse and write each new child at its existing slot, keeping the slot
      boundaries and `write_raw_position` calls correct for the children that
      stay raw.
- [x] Extend `SectionRef` and `Section` in `crates/rdocx/src/document.rs` with
      one read and write pair per family, reusing the `checked_*_section_twips`
      validators.
- [x] Update `section_has_layout` at `crates/rdocx/src/document.rs:21986`.
- [x] Add `columns` and `column_separator` to `PageGeometry`, resolve tracks in
      `sect_pr_to_geometry`, and keep the one-column path textually unchanged.
- [x] Implement column flow, the separator rule, page borders, line numbers,
      vertical alignment and mirrored margins in
      `crates/rdocx-layout/src/paginator.rs`, consuming F-270's settings
      accessors for the mirroring toggle.
- [x] Account for the new owned allocations in the layout cache capacity
      calculation at `crates/rdocx-layout/src/engine.rs:4582`.
- [x] Add the test modules to the two existing integration entrypoints, assert
      LibreOffice 26.2.5.2 and `pdftoppm` 26.01.0 in source, and leave F-251's
      26.09.0 pins untouched.
- [x] Record the three named follow-ups in
      `docs/hld/14-development-backlog.md`: Word GUI human-action confirmation
      for section page semantics, true vertical distribution for
      `w:vAlign="both"`, and column balancing at a continuous section break.
- [x] Run `cargo test -p rdocx-oxml`, `-p rdocx-layout`, `-p rdocx`, then
      `/verify`, and confirm the harness reports 49 of 49 unchanged.

## Open questions

None. Resolved in the S74 consolidated design round.
