# F-X092, Preserve logical reading order in generated PDFs

**Status**: completed
**Sprint**: S72
**Size**: L
**Depends on**: F-255

## Problem

The shared PDF writer emits rich text as run-sized objects and resets `Tm` for
each multilingual glyph in `crates/oxml-pdf/src/writer.rs`. GitHub Issue 74
reproduces 99 percent one-or-two-word extraction lines on a 51-page document,
with the same failure in PowerPoint. Rich `/ActualText` receives degenerate
geometry, and visual paint traversal can differ from logical source order.

## Spec reference

- `docs/hld/03-architecture.md`, the shared PDF backend seam.
- `docs/hld/08-rendering-spec.md`, logical text and `ActualText` ownership.
- `docs/hld/12-testing-strategy.md`, deterministic render and extractor gates.
- `docs/hld/14-development-backlog.md`, F-X092.

## Approach

Emit one multilingual run as one positioned text object with one initial `Tm`
and exact relative glyph positioning. Keep one run-wide `ActualText`. Where
adjacent styled or bidirectional runs still fragment extraction, build a
private PDF-local same-line plan from semantic ownership, transformed baseline,
source spans, and logical indices. Attach the complete logical line to the
first painted run and empty replacement text to later painted runs without
reordering paint or adding an invisible second text layer. Duplicate or gapped
logical indices, mixed ownership, and baseline changes prevent coalescing.

## Rejected alternatives

- Add an invisible extraction layer. It can duplicate selection and drift from
  painted content.
- Reorder paint operators. That changes overlap and z-order semantics.
- Coalesce a whole page. That destroys paragraph, table-cell, and semantic
  boundaries.
- Add a public layout line identifier. Existing spans and geometry are enough.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| unit | `multilingual_run_uses_one_positioned_text_object` | One text object, one initial matrix, relative placement, one logical span, full-run extent. |
| unit | `same_line_actual_text_uses_logical_source_order_without_repainting` | Logical extraction changes while glyph and nontext paint order does not. |
| regression | `large_word_and_presentation_pdfs_preserve_logical_reading_order` | Pinned Poppler extracts every source-built substantial line once and in order from large DOCX and PPTX PDFs. |
| regression | `logical_line_coalescing_respects_owner_and_baseline_boundaries` | Different lines, owners, cells, and ambiguous runs are not merged. |
| compatibility | `handouts_follow_master_metadata_and_all_six_audience_layouts` | The existing handout date is extracted as one complete logical line rather than three run fragments. |
| golden | deterministic raster and stream matrix | Raster and geometry remain identical, only declared text operators and PDF bytes move. |

The test gate is `large_word_and_presentation_pdfs_preserve_logical_reading_order`.

## HLD impact

- `docs/hld/03-architecture.md`
- `docs/hld/08-rendering-spec.md`
- `docs/hld/12-testing-strategy.md`
- `docs/hld/14-development-backlog.md`

## Risk routing

- Layout, pagination, line breaking, or shaping. Use deterministic font mode
  and prove page geometry and raster bytes do not move.
- External oracle comparison. Pin Poppler 26.01.0 and record its version.

## Hash harness

The 49-entry hash harness remains unchanged. Its seven documents use the legacy
Latin `GlyphRun` path, while F-X092 changes only `MultilingualGlyphRun` PDF
emission. Any harness delta blocks completion. The source-built Word and
PowerPoint gate exercises the changed rich path with pinned Poppler 26.01.0.

## Implementation checklist

- [x] Add the source-built large Word and PowerPoint reproduction first.
- [x] Emit rich runs with run-wide extraction geometry.
- [x] Add bounded same-line logical spans without repainting.
- [x] Validate logical extraction with pinned Poppler 26.01.0.
- [x] Prove pre-change rich PDF rasters and the 49-entry harness remain unchanged.
- [x] Review the unchanged hash harness result.
- [x] Run both facade and CLI PDF paths plus the complete non-fast gate.
- [x] Update exactly the listed HLD files.

## Open questions

None. The audit established both geometry and logical-order defects.
