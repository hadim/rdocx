# F-X101, Honor run-level page breaks during pagination

**Status**: completed
**Sprint**: S73
**Size**: M
**Depends on**: F-260

## Problem

`crates/rdocx-layout/src/engine.rs` correctly lowers `w:br w:type="page"`
to `InlineItem::PageBreak`, but `crates/oxml-layout/src/line.rs` converts every
forced break into an ordinary zero-width line and retains only `is_last`.
`crates/rdocx-layout/src/paginator.rs` therefore sees no run-level page-break
boundary and keeps following text on the same page.

## Spec reference

- `docs/hld/02-scope-and-non-goals.md`, modern DOCX capability matrix.
- `docs/hld/08-rendering-spec.md`, Word line breaking and pagination.
- `docs/hld/12-testing-strategy.md`, deterministic Word render comparison.
- `docs/hld/14-development-backlog.md`, F-X101.

## Approach

Add a non-exhaustive `ForcedBreakKind` plus
`LayoutLine::forced_break_after: Option<ForcedBreakKind>`, preserve the marker
on the line ending at the forced boundary, and teach Word pagination to
split immediately after a page-marked line before applying ordinary fitting,
widow, or keep rules to the continuation. Keep line breaks as line-only and
keep column breaks distinguishable while the layout remains single-column.
Reuse the resulting page sequence for layout fragments, PDF, PNG, PAGE,
NUMPAGES, PAGEREF, and TOC targets.

## Rejected alternatives

- Infer a page break from `LayoutLine::is_last`. Paragraph-final lines and
  column breaks already use the same value.
- Rewrite the break as `pageBreakBefore` on the next paragraph. An inline break
  can have text on both sides within one paragraph.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| differential | `run_level_page_breaks_match_word_pagination` | Own-paragraph and inline page breaks produce the same two pages and page text as the pinned oracle. |
| regression | pagination consumers | PDF, PNG, body fragments, page fields, and TOC targets share the corrected page boundary. |
| unit | line forced-break classification | Line, page, and column breaks remain distinguishable after line breaking. |

The **test gate** is the differential test named in the backlog.

## HLD impact

- `docs/hld/02-scope-and-non-goals.md`
- `docs/hld/08-rendering-spec.md`
- `docs/hld/12-testing-strategy.md`
- `docs/hld/14-development-backlog.md`

## Risk routing

- Layout, pagination, and line breaking. Use deterministic fonts for every
  render assertion and declare any hash-harness delta before recording it.
- Public API of a published crate. Record the pre-1.0 struct-literal source
  break from the required line field, run rustdoc, inspect the API diff,
  publish dry-runs, and archive-size checks.
- External oracle comparison. Pin the oracle version and record the structural
  and raster commands and tolerances.

## Hash harness

Expected to be unchanged because no current sample contains a run-level page
break. Any delta requires separate review.

## Implementation checklist

- [x] Add failing own-paragraph and inline page-break regressions.
- [x] Preserve forced-break identity in line output.
- [x] Split Word paragraphs at page-marked lines during pagination.
- [x] Verify every pagination consumer and the focused oracle comparison.
- [x] Run the layout, public API, hash harness, full verification, and microscope gates.

## Open questions

None.
