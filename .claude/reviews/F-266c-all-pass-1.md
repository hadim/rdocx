# F-266c, all, pass 1

**Reviewed**: the uncommitted working tree on `work/f-266c-claude`, 20 tracked
files, 2009 insertions and 78 deletions.
**Verdict**: 5 defects, 11 smells, 6 nitpicks

Aspects reviewed: correctness, contract, panics, ooxml, tests, structure.

## Defects

### D1, a rotated cell breaks its lines at a different width than the box it is painted into
`crates/rdocx-layout/src/table.rs:690`, against `crates/rdocx-layout/src/paginator.rs:4299`

Layout picks the wrap measure as `declared_height - cell_margin_top -
cell_margin_bottom` when the row declares a height, and `content_width`
otherwise. The paginator paints into a transposed box whose usable width is
`box_width - cell.margin_left - cell.margin_right`, where `box_width` is
`cell.merged_height`, the resolved row height. Three things disagree. The wrong
margin pair is subtracted, top and bottom at layout against left and right at
paint. `declared_height` is the authored `w:trHeight` while the painted height
is the resolved one, which is larger for `hRule="atLeast"` or when another cell
grows the row. For an auto-height row the lines were broken at the cell's
horizontal content width, an unrelated number. A `tbRl` cell in a row with
`w:trHeight` wraps early or overflows, and a justified paragraph in a rotated
cell stretches to `line.available_width`, which is the layout measure rather
than the painted box.

### D2, autofit sizes a rotated cell's column from its text length
`crates/rdocx-layout/src/table.rs:1283`

`autofit_column_widths` measures every cell horizontally. `doc_grid` was
threaded into it, the rotation was not. A rotated cell's column should be sized
by the stacked height of its transposed content, because its line length
belongs to the row. A table with `w:tblLayout="autofit"` holding a `tbRl` cell
gives that cell a column as wide as its text is long.

### D3, caller-width measurement applies the section grid to a related story
`crates/rdocx-layout/src/engine.rs:1686`, against
`crates/rdocx-layout/src/engine.rs:8335`

`measure_content` reads `input.document.body.sect_pr.doc_grid` whatever
`related_story_scope` says, while `layout_header_footer_variant_uncached`
deliberately passes `None`. `docs/hld/08-rendering-spec.md` states that
measurement loads what whole-document layout loads. Measuring a header
paragraph in a gridded section returns a grid-snapped height for a header that
is laid out ungridded, which is exactly what a caller sizing a frame from the
measurement relies on.

### D4, a multi-column vertical section transposes and rotates about different centres
`crates/rdocx-layout/src/paginator.rs:1091`, against
`crates/rdocx-layout/src/paginator.rs:1780`

`transpose_body_band` transposes `self.geometry`, which `apply_active_track`
has already narrowed to the active column track, so the transposition is about
the track's centre and `self.content_height` becomes the track width.
`apply_body_rotation` rotates about the page band's centre. The two coincide
for a single-column section and diverge for a section carrying both
`w:cols w:num="2"` and a vertical `w:textDirection`, so the painted content
lands away from its track. `self.tracks` is also left populated while
`self.geometry.columns` is cleared, so `advance_column_track` keeps walking
tracks whose flow height is now the track width.

### D5, a change bar and an anchored drawing inside a rotated cell are rotated with it
`crates/rdocx-layout/src/paginator.rs:4355`, with the rotation at
`crates/rdocx-layout/src/paginator.rs:4392`

`render_change_bar` draws at the page margin out of the page geometry, and
pushes into `elements` inside the cell's block loop, after
`content_element_start`. The rotation closure splits off from that index and
wraps everything, so a tracked-change paragraph in a `tbRl` cell puts its
change bar at an arbitrary page position. `place_cell_anchored` lands in the
same bucket.

## Smells

### S1, the `w:vert` and `w:vertCompress` render path has no test
`crates/rdocx-layout/src/engine.rs:8940`

About sixty lines, including the rotation and scale composition and the
`baseline: None` group placement, are exercised by nothing. The golden fixture
uses `combine` only and `east_asian_layout_reopens_as_modeled_state` is a round
trip. Reverting the whole `vert` branch to `Ok(false)` leaves the suite green.

### S2, `w:eastAsianLayout` silently shadows `w:em`
`crates/rdocx-layout/src/engine.rs:6285`, with the fixture at
`crates/rdocx/tests/integration_test.rs:19873`

The new branch runs before the emphasis branch and continues, so a run carrying
both loses its marks. F-266b's `emphasis_marks_reopen_as_modeled_state` builds
exactly such runs. It asserts XML only, so it passes, but the precedence is
live in the repository and is stated nowhere but a code comment.

### S3, a vertical section records body fragments in the transposed frame
`crates/rdocx-layout/src/paginator.rs:1091`, consumed at
`crates/rdocx-layout/src/paginator.rs:806`

`record_body_fragment` uses the body band's margins, which after the
transposition can be negative. `WordBodyLayoutFragment` drives comment anchor
rectangles and float keep-out bands outside layout, so those consumers see a
frame the painted page does not use.

### S4, line numbers mix the transposed and untransposed frames
`crates/rdocx-layout/src/paginator.rs:1707`

`draw_line_numbers` composes a `track_x` recorded against the transposed band
with `self.page_geometry.margin_top`, an untransposed margin, and a transposed
baseline.

### S5, the grid snap trusts its caller for a positive pitch and has no tolerance
`crates/rdocx-layout/src/convert.rs:144`

`restore_word_line_heights` is `pub(crate)`. A future caller passing
`Some(0.0)` yields an infinite row count and a `NaN` height that spreads
silently through pagination. Separately the ceiling has no tolerance, so a line
whose height is a few ulps above an exact multiple of the pitch takes a whole
extra grid row.

### S6, the grid character space misses several shaping sites in the same funnel
`crates/rdocx-layout/src/engine.rs:6697`,
`crates/rdocx-layout/src/engine.rs:6733`,
`crates/rdocx-layout/src/engine.rs:6014`

`w:sym`, `w:noBreakHyphen` and the numbering marker shape without the grid
character space. They never applied `w:spacing` either, so the gap is not new,
but `docs/hld/08-rendering-spec.md` now promises the space is added to every
character advance, which those sites do not do.

### S7, a modelled `w:docGrid` retains a foreign attribute without proving its binding
`crates/rdocx-oxml/src/document.rs:1311`

`parse_doc_grid` keeps an unmodelled attribute by its source spelling and never
receives `owner_bindings`. The new round-trip test declares `xmlns:x` on the
element itself, so nothing covers an attribute bound on an ancestor.

### S8, the column-width half of the plan's transposed-box sentence is unimplemented
`crates/rdocx-layout/src/table.rs:713`

The plan says a vertical cell's content height drives the column width it
needs. Only the row-height half landed.

### S9, a combined or rotated run drops its provenance and its link
`crates/rdocx-layout/src/engine.rs:8870`

The glyph run builder hardcodes `source`, `field_kind`, `field_source` and
`note` to `None`, and `InlineItem::Group` carries no hyperlink, so a
hyperlinked run with `w:eastAsianLayout` emits no link annotation and a
combined run contributes no source span. This matches the F-266b emphasis
precedent, which is why it is a smell, but the story adds a second element
class with the same hole and says nothing about it.

### S10, the row-height test asserts only that the row grew
`crates/rdocx/tests/integration_test.rs:20994`

`a_vertical_cell_drives_the_row_height_from_its_transposed_box` asserts
`vertical > horizontal + 1.0`, which any implementation that makes a rotated
cell taller by any amount satisfies. It never checks the height against the
transposed measure its name claims.

### S11, seven previously byte-preserved toggles now canonicalise, untested
`crates/rdocx-oxml/src/paragraph_properties.rs:1328`

`w:kinsoku w:val="true"` now saves as `w:kinsoku`. That is the established
behaviour for the toggles F-264 modelled, but these seven round-tripped
verbatim before this diff and nothing pins the new spelling.

## Nitpicks

- `crates/rdocx-layout/src/engine.rs:2773`, `#[allow(clippy::too_many_arguments)]` written twice.
- `crates/rdocx-oxml/src/document.rs:1920`, two adjacent `if ordered_raw` blocks.
- `crates/rdocx-oxml/src/document.rs:1896`, the legacy `!ordered_raw` branch writes raw before the typed `w:docGrid`.
- `crates/rdocx-layout/src/table.rs:345`, `text_direction_rotation` returns `None` for `lrTb` and for a misspelled producer value alike, with no diagnostic between them.
- `crates/rdocx-layout/src/engine.rs:9492`, `w:charSpace` is added to Latin runs as well as East Asian ones.
- `crates/rdocx-layout/src/engine.rs:6269`, `annotation_base` is built for every text run whether either branch is taken or not.
- `crates/rdocx-layout/src/engine.rs:7829`, the vertical-column diagnostic uses a bare push while the upright-stacking one dedupes.

## Not found

- **panics**. No new `unwrap`, `expect`, indexing or slicing on untrusted
  input. `parse_doc_grid` retains an attribute when `str::parse` fails rather
  than propagating. A negative or zero `w:linePitch` is filtered before it
  reaches the arithmetic. Both new divisions are guarded by a positive check on
  their divisor.
- **ooxml child order**. Both new sequences are correct against ECMA-376.
  `w:docGrid` writes between the slot `(8, 1)` raw children and the new slot
  `(8, 2)` `w:printerSettings`, and the seven `CT_PPr` toggles write at 12 to
  17 and 20 between `w:suppressAutoHyphens`, `w:bidi`, `w:adjustRightInd` and
  `w:spacing`. Prefix-tolerant read and fixed `w:` write hold for both, and an
  unmodelled subtree is captured rather than dropped, including a `w:docGrid`
  that carries children.
- **structure**. No new trait, generic parameter, `Box<dyn>`, forwarding
  wrapper, feature flag, crate, module or file. `ST_DocGrid` and `CT_DocGrid`
  are additive, and `TableCell::rotation` and `PageGeometry::body_rotation` are
  additive public fields on structs constructed by literal inside the workspace
  only.
- **cache correctness**. `doc_grid` is compared on both cache keys and stored
  on both entries, and every call site passes the value it keys on. No path
  builds a block under one grid and reads it under another.
- **float association on the horizontal path**. The `None` arm of the
  transposed box reproduces the original expressions exactly and every
  downstream expression keeps its association, so the horizontal path is bit
  identical. The hash harness holding at 49 of 49 agrees.
- **module placement**. Both new test modules are the last top-level `mod` in
  their file.
