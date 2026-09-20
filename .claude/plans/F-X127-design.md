# F-X127, Collapse adjacent page break requests

**Status**: completed
**Sprint**: S74
**Size**: S
**Depends on**: F-X101

## Problem

Issue 129 shows an empty page when a paragraph ending in a run page break is
immediately followed by a paragraph with `pageBreakBefore`. Each request alone
correctly starts one new page.

## Spec reference

- `docs/hld/08-rendering-spec.md`, deterministic pagination and forced breaks.
- `docs/hld/12-testing-strategy.md`, deterministic render and oracle rules.
- `docs/hld/14-development-backlog.md`, "F-X127, Collapse adjacent page break requests".

## Approach

Record that the most recent page transition came from a trailing run-level
page break at the previous block boundary. Suppress only the next block's
`pageBreakBefore` when no visible content or structural transition intervened.
Clear the state on content, tables, sections, columns, and other break kinds.

## Rejected alternatives

- Suppress every page break on an empty page. Some explicit blank pages are intentional.
- Remove `pageBreakBefore` during parsing. The property must round-trip unchanged.
- Treat all forced breaks alike. Column and section semantics differ.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| differential | `adjacent_run_and_paragraph_page_breaks_share_one_transition` | Combined case matches pinned LibreOffice page count and text. |
| regression | individual and separated breaks | Each single break remains two pages and intervening content preserves two transitions. |
| golden | deterministic page manifest | No blank middle page and unchanged page geometry. |

The **test gate** is the named differential test.

## HLD impact

- `docs/hld/08-rendering-spec.md`
- `docs/hld/12-testing-strategy.md`
- `docs/hld/14-development-backlog.md`

## Risk routing

- Layout and pagination. Use deterministic fonts and the pinned LibreOffice
  26.2.5.2 oracle with exact page count and normalized text.

## Hash harness

Expected unchanged. Existing samples do not contain adjacent break requests.

## Implementation checklist

- [x] Add combined, single, and separation-boundary fixtures.
- [x] Track and consume only an adjacent run page transition.
- [x] Record the pinned LibreOffice result.
- [x] Run paginator, render, differential, hash, and scoped verification gates.

## Open questions

None. The reporter supplied matching LibreOffice results on two platforms.
