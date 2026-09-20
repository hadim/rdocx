# F-X126, Preserve drawings through comparison acceptance

**Status**: completed
**Sprint**: S74
**Size**: M
**Depends on**: F-X097, F-X102

## Problem

Issue 128 reports a text-only comparison that reaches the acceptance
postcondition with unequal drawing projections. Body and header pictures are
unchanged, self-comparison succeeds, and simpler picture cases do not expose
the mismatch.

## Spec reference

- `docs/hld/04-opc-and-packaging.md`, story-scoped package ownership.
- `docs/hld/08-rendering-spec.md`, body and related-story drawing semantics.
- `docs/hld/12-testing-strategy.md`, source-built comparison regressions.
- `docs/hld/14-development-backlog.md`, "F-X126, Preserve drawings through comparison acceptance".

## Approach

Build a source fixture with body, header, and footer drawings whose inline or
anchor XML depends on local namespace scope. Apply multiple text edits across
those stories. Compare normalization at every postcondition boundary, then
retain scoped drawing markup symmetrically wherever the failing projection
differs. Keep changed-drawing sensitivity explicit.

## Rejected alternatives

- Remove drawings from normalized equality. That would hide real loss.
- Disable acceptance validation. It is a transactional safety condition.
- Depend on the reporter's private files. The regression must be source-built.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| regression | `text_only_comparison_with_body_header_and_footer_drawings_accepts_exactly` | Multi-story text edits preserve drawings and pass both postconditions. |
| round-trip | compared package save and reopen | Local namespaces, relationships, and raw drawing markup survive. |
| regression | changed drawing control | A real drawing change remains visible to acceptance comparison. |

The **test gate** is the named regression.

## HLD impact

- `docs/hld/04-opc-and-packaging.md`
- `docs/hld/08-rendering-spec.md`
- `docs/hld/12-testing-strategy.md`
- `docs/hld/14-development-backlog.md`

## Risk routing

- Parser, serializer, and rendering input. Prove namespace closure, story
  relationship scope, deterministic fonts, and unchanged visual output.

## Hash harness

Expected unchanged. Comparison is not used by sample generation.

## Implementation checklist

- [x] Reproduce the mismatch with source-built multi-story drawings.
- [x] Identify the first asymmetric normalized projection.
- [x] Retain drawing scope symmetrically without masking real changes.
- [x] Run the impacted comparison, story, clippy, formatting, prose, skill
  drift, and hash gates requested for this issue-only pass.

## Open questions

The private pair is unavailable. The reporter offered candidate testing, but
completion requires a source-built reproduction and does not depend on it.
