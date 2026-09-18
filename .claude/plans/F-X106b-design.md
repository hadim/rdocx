# F-X106b, Expose paragraph and run formatting mutations in Python

**Status**: completed
**Sprint**: S73
**Size**: M
**Depends on**: F-X106a

## Problem

Python Paragraph objects cannot read or write style and numbering, Run objects
cannot read or write character style, and Font objects omit coherent highlight
mutation even though each operation exists in the native facade. The current
contributor implementation reads Word highlight names but accepts only RGB hex
on write and emits `w:shd`, which changes shading rather than highlight.

## Spec reference

- `docs/hld/10-bindings-spec.md`, python-docx-compatible object surface.
- `docs/hld/12-testing-strategy.md`, Python runtime and typing gates.
- `docs/hld/14-development-backlog.md`, F-X106b.

## Approach

Add nullable paragraph `style` and `numbering` properties with typed setters,
nullable run `style_id`, and nullable Font `highlight` using the existing Word
`ST_HighlightColor` names in both directions. Keep run shading as a separate
property rather than treating a hex fill as highlight. Call the established
native readers and setters directly. Formatting changes preserve all non-text
run children and do not stale handles whose structural coordinates remain
valid.

## Rejected alternatives

- Accept raw style XML or numeric highlight values. Existing typed IDs and
  enumerations already express the supported native contract.
- Replace the paragraph or run after each setter. Direct formatting mutation
  preserves handle identity and unrelated ordered content.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| binding | `python_paragraph_and_run_formatting_matches_native_facades` | Paragraph style, numbering, run style, named highlight, and separate shading read and write like the native facade after reopen. |
| regression | mixed run content | Formatting setters preserve tabs, breaks, fields, symbols, drawings, and text order. |
| typing | installed mypy and stubtest | Nullable IDs, numbering values, and highlight enumeration match runtime descriptors. |

The **test gate** is the binding test named in the backlog.

## HLD impact

- `docs/hld/10-bindings-spec.md`
- `docs/hld/12-testing-strategy.md`
- `docs/hld/14-development-backlog.md`

## Risk routing

- WASM or PyO3 bindings. Run both WASM checks, the mixed Python package,
  pytest, strict mypy, stubtest, and clean abi3 wheel installation.

## Hash harness

Expected to be unchanged because no sample is generated through these Python
setters.

## Implementation checklist

- [x] Add runtime and typing failures for all four property groups.
- [x] Bind paragraph style and numbering.
- [x] Bind run style, named highlight, and separate shading semantics.
- [x] Prove mixed content and handle identity remain intact.
- [x] Run binding, WASM, hash harness, full verification, and microscope gates.

## Open questions

None.
