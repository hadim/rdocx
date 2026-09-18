# F-X129, Tolerate unmatched notes placeholders

**Status**: completed
**Sprint**: S74
**Size**: S
**Depends on**: F-X118

## Problem

Issue 131 shows that Google Slides can emit a notes-slide placeholder index
absent from the notes master. `to_notes_pdf` rejects the whole export even
though the slide, notes text, and remaining notes placeholders are usable.

## Spec reference

- `docs/hld/06-presentationml-model.md`, placeholder inheritance and notes ownership.
- `docs/hld/08-rendering-spec.md`, notes-page composition and diagnostics.
- `docs/hld/12-testing-strategy.md`, source-built differential fixtures.
- `docs/hld/14-development-backlog.md`, "F-X129, Tolerate unmatched notes placeholders".

## Approach

Keep overlays whose keys match a notes-master placeholder. Skip each unmatched
notes-slide placeholder and emit one ordered render diagnostic. Preserve hard
errors for ambiguity, duplicate matching, invalid relationships, and a missing
slide-image placeholder. Non-placeholder notes shapes remain appended.

## Rejected alternatives

- Match only by placeholder type. Multiple placeholders of one type need indices.
- Add unmatched placeholders as ordinary shapes. Their geometry may be inherited and absent.
- Suppress all placeholder errors. Ambiguity remains unsafe.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| differential | `notes_pdf_skips_only_unmatched_slide_placeholder_overlays` | Google Slides index variant renders and matches pinned LibreOffice text and geometry. |
| regression | matched and ambiguous controls | Matched overlays apply, ambiguity and duplicates still fail. |
| integration | notes PDF report | One ordered diagnostic names the skipped placeholder. |

The **test gate** is the named differential test.

## HLD impact

- `docs/hld/06-presentationml-model.md`
- `docs/hld/08-rendering-spec.md`
- `docs/hld/12-testing-strategy.md`
- `docs/hld/14-development-backlog.md`

## Risk routing

- Placeholder resolution and rendering. Compare structure with pinned
  python-pptx and render with pinned LibreOffice in deterministic font mode.

## Hash harness

Expected unchanged. The checked samples do not render notes pages.

## Implementation checklist

- [x] Add the source-built unmatched index and matched controls.
- [x] Skip only unmatched overlays and report them in order.
- [x] Preserve ambiguity and required slide-image failures.
- [x] Run the impacted rpptx, render, differential, and hash gates.

## Open questions

None. Skipping is safer than rendering a placeholder with missing inherited geometry.
