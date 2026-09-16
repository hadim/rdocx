# F-X109, Split text runs at Unicode character offsets

**Status**: completed
**Sprint**: S73
**Size**: M
**Depends on**: F-260, F-X106a

## Problem

`RunPosition` addresses only boundaries between top-level paragraph runs.
Comment anchors and formatting therefore cannot select part of a run, and the
facade exposes no checked primitive that creates such a boundary without
losing run properties or surrounding ordered content.

## Spec reference

- `docs/hld/03-architecture.md`, ordered run content and staged mutation.
- `docs/hld/04-opc-and-packaging.md`, run child order and range markers.
- `docs/hld/10-bindings-spec.md`, RunPosition and Python coordinate contracts.
- `docs/hld/12-testing-strategy.md`, comments, Unicode, and binding coverage.
- `docs/hld/14-development-backlog.md`, F-X109.

## Approach

Add `Document::split_run(body_index, run_index, character_offset) ->
Result<usize>` and the same Python method. Count Unicode scalar values across
the run's direct literal text children. Zero and end offsets are no-op
boundaries, while an interior offset creates a continuation. Walk the ordered
content once, split the selected text child when the boundary falls inside it,
and partition zero-width non-text children by their existing source position.
Clone run properties to the continuation and retain every raw child at its
corresponding boundary. Update hyperlink spans and direct marker coordinates.
Return the selected or new run index so callers can create exact existing
`RunRange` values without changing that public struct.

## Rejected alternatives

- Add a field to `RunPosition`. Existing struct literals would be a breaking
  source change and every range consumer would need new ambiguity rules.
- Use UTF-8 byte offsets. They are unsafe for Python strings and easy to place
  inside a code point.
- Split grapheme clusters only. Python indexing and the current native text
  APIs are based on Unicode scalar values, not locale-dependent graphemes.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| binding | `split_run_enables_exact_comment_ranges_without_losing_content` | Two splits create a word-level comment range in Rust and Python and reopen exactly. |
| regression | ordered mixed content | Text, tabs, breaks, fields, drawings, symbols, and hyperlinks stay in order with copied formatting. |
| failure | offset and ownership | Non-text runs, no-op endpoints, invalid body or run indices, and out-of-range offsets leave bytes unchanged. |

The **test gate** is the binding test named in the backlog.

## HLD impact

- `docs/hld/03-architecture.md`
- `docs/hld/04-opc-and-packaging.md`
- `docs/hld/10-bindings-spec.md`
- `docs/hld/12-testing-strategy.md`
- `docs/hld/14-development-backlog.md`

## Risk routing

- Any parser or serialiser. Verify run child order, range-marker coordinates,
  and exact raw-subtree round trip after a split.
- Public API of a published crate. State the additive API, run rustdoc, inspect
  the API diff, and run package dry-runs and size checks.
- WASM or PyO3 bindings. Run both WASM checks, pytest, strict mypy, stubtest,
  and clean abi3 installation.

## Hash harness

Expected to be unchanged because no sample invokes the split operation.

## Implementation checklist

- [x] Add failing ASCII, multibyte, hyperlink, field-adjacent, and mixed-content tests.
- [x] Implement checked staged splitting and coordinate repair.
- [x] Expose the returned boundary in Python.
- [x] Anchor and reopen an exact word-level comment.
- [x] Run serialization, binding, API, hash harness, full verification, and microscope gates.

## Open questions

None. The user approved Unicode scalar offsets, matching valid Python string
code points and Rust `char`, rather than locale-dependent grapheme clusters.
