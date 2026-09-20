# F-268b, Floating table placement and wrap

**Status**: completed
**Sprint**: S74
**Size**: M
**Depends on**: F-268a

## Problem

F-268a models, authors and round-trips `w:tblpPr` and `w:tblOverlap`, and
nothing reads them. A table that declares a floating position still renders in
the flow, which leaves the DOCX-035 row in
`docs/hld/02-scope-and-non-goals.md:242` at `partial`.

The paginator has no way to express a floating table. Its table arm at
`crates/rdocx-layout/src/paginator.rs:574` places every table at
`margin_left + table_indent` and advances `cursor_y` row by row. It never
consults `page_wraps`, and no code path can make a table an obstacle for the
text around it.

The mechanism it needs already exists and is exercised by floating drawings:

- `PlacedWrap` and its keep-out band at
  `crates/rdocx-layout/src/paginator.rs:33`.
- `place_anchored` at `:1028`, which resolves a rect, pushes a `PlacedWrap` and
  records a paragraph-relative anchor into `resolved_out`.
- `resolve_anchor_h` at `:1929` and `resolve_anchor_v` at `:1970`.
- `lookahead_wraps` at `:984`, which lets a drawing anchored to a later
  paragraph push earlier text aside.
- `has_paragraph_relative_wrap` at `:394` and the two-pass `ResolvedWraps`
  fixed point at `:463`.
- `reflow_around_wraps` at `:1993`, called from `paginate_paragraph` before
  anything is measured.

`AnchoredDrawing` at `crates/rdocx-layout/src/block.rs:22` carries the
positioning fields those functions consume. A floating table is the same
obstacle with different content.

The vocabulary maps one for one, which is what makes this a re-use rather than
a new mechanism. `w:tblpPr` `horzAnchor` `margin`, `page` and `text` map onto
`ST_RelativeFromH::{Margin, Page, Column}` at
`crates/rdocx-oxml/src/drawing.rs:25`. `vertAnchor` maps onto
`ST_RelativeFromV::{Margin, Page, Paragraph}` at `:67`. `tblpXSpec` is already
`AnchorAlignH` at `:203` and `ST_YAlign` minus `Inline` is `AnchorAlignV` at
`:238`.

## Spec reference

- `docs/hld/02-scope-and-non-goals.md`, "Modern DOCX capability matrix", the
  DOCX-035 row for floating, bidirectional, autofit and advanced table layout.
- `docs/hld/08-rendering-spec.md`, "Tables", for table origin, row placement,
  repeating header rows and physical border segments.
- `docs/hld/12-testing-strategy.md`, "Test taxonomy", for the golden category.
- `docs/hld/12-testing-strategy.md`, "The golden-PNG gate", for deterministic
  font mode, the pinned Poppler 26.01.0 rasteriser and the reviewed manifest.
- `docs/hld/14-development-backlog.md`, "F-268, Floating and advanced table
  layout (L)", and the F-268b child entry the split creates.

## Approach

### 1. Lower the position, `crates/rdocx-layout/src/table.rs` and `block.rs`

`TableBlock` at `crates/rdocx-layout/src/block.rs:110` gains
`pub floating: Option<FloatingTable>`:

```rust
pub struct FloatingTable {
    pub rel_h: ST_RelativeFromH,
    pub off_h: f64,
    pub align_h: Option<AnchorAlignH>,
    pub rel_v: ST_RelativeFromV,
    pub off_v: f64,
    pub align_v: Option<AnchorAlignV>,
    pub dist_top: f64,
    pub dist_bottom: f64,
    pub dist_left: f64,
    pub dist_right: f64,
    pub overlap_allowed: bool,
}
```

The field set is deliberately the positioning half of `AnchoredDrawing`, so
`resolve_anchor_h` and `resolve_anchor_v` take it without a signature change.
`layout_table_inner` at `crates/rdocx-layout/src/table.rs:271` maps the
`CT_TblPPr` that F-268a modeled onto it. `tblpYSpec="inline"`, or an absent
`w:tblpPr`, leaves `floating` as `None` and the table stays in the flow.

A floating table keeps its own `table_indent` out of the picture. Its origin
comes entirely from the resolved anchor, which is what `w:tblpPr` means.

### 2. Place it, `crates/rdocx-layout/src/paginator.rs`

The table arm at `:574` branches on `table.floating`:

- `None` keeps today's row loop byte for byte.
- `Some(f)` calls a new `Pager::place_floating_table`, which resolves the rect
  through `resolve_anchor_h` and `resolve_anchor_v`, renders every row at that
  origin with the existing `render_table_row`, pushes one
  `PlacedWrap { wrap: WrapType::Square, .. }` carrying the four from-text
  distances, and **does not** advance `cursor_y`.

```rust
impl Pager<'_> {
    fn place_floating_table(
        &mut self,
        table: &TableView<'_>,
        floating: &FloatingTable,
        body_index: Option<usize>,
        block_idx: usize,
        para_top: f64,
    );
}
```

A float whose rect does not fit the remaining page moves whole to the next
page, which is what Word does and what the row loop cannot express. A floating
table never repeats header rows, because it never crosses a page boundary.

Body fragments are recorded for the float at its resolved rect, so field
resolution and bookmark pagination keep seeing it.

### 3. Integrate it with the wrap machinery

`has_paragraph_relative_wrap` at `:394` and `lookahead_wraps` at `:984` are
extended to see a floating table block, not only `para.anchored`. This is what
makes a paragraph that precedes the table in the body get pushed aside by it,
and what makes the two-pass fixed point converge for a `text`-anchored table
whose vertical position depends on where its own anchor paragraph landed.

The `ResolvedWraps` key is `(block_idx, anchor_idx)`. A floating table block
uses anchor index 0, which cannot collide, because a table block carries no
`anchored` vector.

`reflow_around_wraps` needs no change. It already consumes `page_wraps` without
caring what produced a rect.

### 4. Resolve floats against each other

`w:tblOverlap` is carried on `FloatingTable::overlap_allowed`. When two floats
on one page would intersect and either forbids overlap, the later one in body
order is pushed below the earlier one's keep-out band. Resolution is
single-page, because that is the scope the reviewed geometry exercises.
Facing-page and section-scoped resolution is out of scope and recorded in the
parent plan.

### Deliberately out of scope

- **A non-floating table flowing beside a float.** The reciprocal half. The
  table arm would have to narrow columns per row inside a keep-out band, which
  is a re-layout rather than a re-position. Named follow-up in the parent plan.
- **`w:cantSplit` as a distinction**, for the same reason as F-268a. The row
  loop at `:584` already keeps every row atomic.
- **Any public API.** This story adds no facade surface. F-268a owns all of it.

## Rejected alternatives

- Add `AnchoredContent::Table(TableBlock)` and hang the float off the following
  paragraph. It moves table rendering out of the one place that owns it and
  duplicates `render_table_row` inside `anchored_elements` at
  `crates/rdocx-layout/src/paginator.rs:1351`.
- Add a third `LayoutBlock` variant for a floating table. Every match on
  `LayoutBlock` at `crates/rdocx-layout/src/block.rs:103` and every
  `LayoutBlockLike` implementer would grow a case for something that is still a
  table.
- Give `FloatingTable` its own anchor enums rather than the drawing ones.
  `resolve_anchor_h` and `resolve_anchor_v` already take
  `ST_RelativeFromH` and `ST_RelativeFromV`, and a parallel set would mean two
  resolvers to keep in agreement.
- Let a floating table split across pages like an inline one. Word moves it
  whole, and splitting it would need a second anchor resolution on the
  continuation page with no source fact to resolve against.
- Run the wrap fixed point to convergence rather than the existing two passes.
  The second pass already reflows earlier text, and a third would make
  pagination cost depend on float density for no reviewed geometry difference.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| golden | `floating_tables_match_reviewed_word_page_geometry_and_pagination` | One in-code document holding a margin-anchored float, a page-anchored float and a text-anchored float renders in deterministic font mode to a pinned page count, pinned float origins and pinned wrapped line boxes in the text beside each one. |
| regression | `a_floating_table_that_does_not_fit_moves_whole_to_the_next_page` | The float is never split, never repeats a header row and never overlaps the bottom margin. |
| regression | `a_text_anchored_float_converges_in_the_existing_two_passes` | The second pass reflows the paragraphs before the anchor and the resolved rect is stable between passes. |
| regression | `a_paragraph_before_a_floating_table_is_pushed_aside_by_it` | `lookahead_wraps` sees the float, and the preceding paragraph's line boxes narrow. |
| regression | `two_floats_that_forbid_overlap_stack_instead_of_intersecting` | The later float in body order sits below the earlier one's keep-out band, and two floats that both allow overlap are left intersecting. |
| unit | `tblp_pr_anchors_map_onto_the_drawing_anchor_frames` | `margin`, `page` and `text` map onto the documented `ST_RelativeFromH` and `ST_RelativeFromV` variants, and `tblpYSpec="inline"` yields no float. |
| unit | `a_floating_table_takes_its_origin_from_the_anchor_not_the_indent` | `table_indent` does not contribute to a floating table's resolved x. |
| regression | `a_document_with_no_floating_table_still_paginates_in_one_pass` | `has_paragraph_relative_wrap` stays false when the only tables are inline. Named guard for hash-harness fact 2. |

The **test gate** is the golden test
`floating_tables_match_reviewed_word_page_geometry_and_pagination`.

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
- `docs/hld/12-testing-strategy.md`
- `docs/hld/14-development-backlog.md`

`docs/hld/08-rendering-spec.md` gains the floating table paragraphs in
"Tables": anchor frame resolution, the keep-out band, whole-table movement
across a page boundary, the absence of header repetition on a float, and
single-page float-against-float resolution.

`docs/hld/02-scope-and-non-goals.md` moves the DOCX-035 row at `:242` from
`partial` to `complete`, which is what closes the F-268 parent.

`docs/hld/10-bindings-spec.md` is deliberately absent. This story adds no
public API.

## Risk routing

Matched rows from `.claude/skills/risk-routing.md`:

- **Layout, pagination, line breaking, text shaping.** The primary row. Extra
  check: every golden and geometry assertion runs in deterministic font mode,
  baseline movement is a labelled commit with a stated delta, and
  `python3 scripts/golden_png_harness.py --check` runs against the pinned
  Poppler 26.01.0 oracle. The two-pass fixed point gets its own regression, so
  a convergence change is visible rather than inferred.
- **Unit conversion, `Twips`, `Points`.** `tblpX`, `tblpY` and the four
  from-text distances are authored in twips and resolved in points. Extra
  check: constructors keep truncating with `as i64`, and the resolved rect is
  asserted at exact point boundaries.
- **A new trait, generic parameter, crate, module or file.** No new crate,
  module or file. One new struct `FloatingTable` and one new field on
  `TableBlock` in `crates/rdocx-layout/src/block.rs`, and one new method on
  `Pager`. No new trait and no new generic parameter. `FloatingTable` exists
  because `TableBlock` needs the positioning half of `AnchoredDrawing` without
  its content half, and both halves are consumed by `resolve_anchor_h` and
  `resolve_anchor_v` today.

Not matched: parsers and serialisers, which F-268a owns and this story does not
touch. Not matched: public API of a published crate, because this story adds
none. Not matched: theme colour, crate dependency graph, bundled fonts and
assets, WASM and PyO3 bindings, feature flags, external oracle comparison
beyond the already pinned rasteriser, release scripting, file moves.

## Hash harness

**Expected unchanged.** `scripts/hash_baseline.json` stays at its current 49
entries and `scripts/golden_pixel_manifest.json` is unmoved.

Six of the seven samples in `scripts/hash_harness.py:33` contain tables.
Counting `add_table` calls in `crates/rdocx/examples/generate_all_samples.rs`:
`feature_showcase` 8, `invoice` 4, `quote` 3, `report` 3, `contract` 2,
`proposal` 2, `letter` 0. Two checkable facts say none of them moves. The other
four belong to F-268a.

1. **No sample floats.** No call to `set_float_position` exists in
   `crates/rdocx/examples/generate_all_samples.rs`, and the generator injects
   no raw `w:tblpPr`. `TableBlock::floating` is `None` for every sample table,
   and the `None` arm of the branch is today's row loop unchanged.
2. **The wrap extensions stay inert.** `has_paragraph_relative_wrap` at
   `crates/rdocx-layout/src/paginator.rs:394` gates the second pagination pass.
   Extending it to see a floating table cannot make it true for a document that
   has none, so every sample still paginates in one pass, and
   `lookahead_wraps` contributes no new rect. The regression test
   `a_document_with_no_floating_table_still_paginates_in_one_pass` is the guard
   for exactly this, and it is the one genuinely new failure mode this story
   introduces into a path every sample uses.

If either fact is falsified during implementation, that is a stop under the
escalation trigger in `.claude/WORKFLOW.md`, not a re-record. The baseline is
exclusive to one story per sprint wave and F-268b does not hold it. The same
rule applies to the private Word corpus gate, whose tables may carry
`w:tblpPr`. A corpus geometry change there is the intended effect of this
story, so it is reviewed and explained rather than re-recorded blind.

## Implementation checklist

- [x] Add failing coverage for margin, page and text anchored floats, for a
      float that does not fit, and for two floats that forbid overlap.
- [x] Add `FloatingTable` and `TableBlock::floating`, and map the `CT_TblPPr`
      that F-268a modeled onto the drawing anchor frames.
- [x] Add `Pager::place_floating_table`, rendering rows at the resolved rect
      without advancing `cursor_y` and recording body fragments.
- [x] Push one `PlacedWrap` per float and record a text-anchored float into
      `resolved_out`.
- [x] Extend `has_paragraph_relative_wrap` and `lookahead_wraps` to see a
      floating table block.
- [x] Move a float whole to the next page when its rect does not fit.
- [x] Resolve float against float on one page for
      `w:tblOverlap w:val="never"`.
- [x] Record the golden geometry in deterministic font mode and pin it.
- [x] Run the impacted layout, facade, golden, clippy, prose, hash and
      golden-PNG gates and confirm the 49 entries and the pixel manifest are
      unmoved.
- [x] Review and explain any private Word corpus geometry change rather than
      re-recording it.

**Worker note.** `scripts/docx_authoring_conformance.py --private-required` and
`scripts/docx_ssim_harness.py --check` cannot run from a worker worktree,
because `corpus/` is gitignored and does not exist there. The integrator runs
both over the integrated result. `scripts/golden_png_harness.py --check` cannot
run there either, because the pinned `pdftoppm` wrapper bind mounts only the
canonical repository path. The seven digests were proved instead by staging the
generated sample PDFs under `/private/tmp` and calling the harness's own
`decode_png` and `compare_pixels` against `scripts/golden_pixel_manifest.json`,
with the pinned Poppler 26.01.0. All seven match.

## Open questions

None. Resolved in the S74 consolidated design round.
