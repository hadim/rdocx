# F-X094b, Structured CLI text and layout plus guarded replacement

**Status**: approved
**Sprint**: S72
**Size**: L
**Depends on**: F-X032, F-X037, F-X047

## Problem

CLI text output is flat, replacement has no expected-count guard, and no
command exposes `WordLayoutResult`. Existing text spans do not provide a real
top-level body-item extent for empty tables, images, or page-spanning blocks.
GitHub Issue 76 asks for addressable text and geometry automation.

## Spec reference

- `docs/hld/08-rendering-spec.md`, Word layout and source provenance.
- `docs/hld/10-bindings-spec.md`, CLI and native facade contracts.
- `docs/hld/12-testing-strategy.md`, layout and CLI integration gates.
- `docs/hld/14-development-backlog.md`, F-X094b.

## Approach

Add public additive `WordBodyLayoutFragment` records and a body-fragment
sidecar on `WordLayoutResult`. Carry a private top-level body owner through
existing layout blocks and record actual placed or split bounds. Do not change
`PositionedElement`. Add `text --json`, `layout --json`, and
`replace --expect N`. Text uses a top-level body index plus typed nested path,
style, numbering, and nullable direct run formatting. Layout returns one-based
physical and displayed pages plus point-space fragments. Every body item is
present, with an empty fragment list only for preservation-only unlaid content.

## Rejected alternatives

- Derive boxes from glyph baselines. That drops images and empty tables.
- Change shared positioned elements. A private sidecar avoids renderer churn.
- Use a flattened integer path. Nested ownership becomes ambiguous.
- Serialize before checking `--expect`. Mismatch must publish nothing.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| integration | `cli_structured_text_layout_and_guarded_replace_preserve_exact_contracts` | Nested paths, formatting, real fragments, and no output on count mismatch. |
| layout | `layout_json_keeps_page_spanning_body_elements_as_multiple_fragments` | One body item reports ordered fragments on each physical and displayed page. |
| layout | `empty_table_and_image_blocks_keep_real_geometry` | Nontext and empty blocks receive measured extents. |
| integration | `text_json_preserves_nested_paths_styles_numbering_and_run_formatting` | Exact schema-1 source order and nullable direct formatting. |

The test gate is `cli_structured_text_layout_and_guarded_replace_preserve_exact_contracts`.

## HLD impact

- `docs/hld/08-rendering-spec.md`
- `docs/hld/10-bindings-spec.md`
- `docs/hld/12-testing-strategy.md`
- `docs/hld/14-development-backlog.md`

## Risk routing

- Layout, pagination, line breaking, or shaping. Use deterministic fonts and
  prove positioned output and the hash baseline do not move.
- Public API of published crates. Record additive pre-1.0 impact, run patched
  package dry-runs, and enforce archive sizes.

## Hash harness

Expected unchanged across all 49 entries because the sidecar does not alter
positioned or rendered output.

## Implementation checklist

- [ ] Add the body-fragment record and accessor.
- [ ] Carry body ownership and record split or empty extents.
- [ ] Add exact structured text and layout JSON.
- [ ] Add guarded run-aware replacement.
- [ ] Document units, paths, pages, and unmatched behavior.
- [ ] Run sensitivity tests, the full gate, and unchanged hashes.
- [ ] Update exactly the listed HLD files.

## Open questions

None.
