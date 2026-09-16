# F-259, Container measurement and equal-height layout

**Status**: completed
**Sprint**: S73
**Size**: M
**Depends on**: F-257, F-258

## Problem

The public facade can lay out a whole document, but it cannot measure a
supported paragraph or table at a caller-supplied width. Callers that need two
independent nested tables to share one final height must guess using rules that
can diverge from the production paginator and selected deterministic fonts.

## Spec reference

- `docs/hld/03-architecture.md`, shared layout ownership and deterministic font engines.
- `docs/hld/08-rendering-spec.md`, Word table measurement and paginator reuse.
- `docs/hld/10-bindings-spec.md`, native Word facade stability.
- `docs/hld/12-testing-strategy.md`, deterministic Word layout and M23 conformance.
- `docs/hld/14-development-backlog.md`, F-259.

## Approach

Add an owned `ContentMeasurement` result with height in points and ordered
layout diagnostics, plus a native `Document::measure_content` operation that
accepts an existing `ContentLocation`, a positive `Length` width, and render
options. Resolve the selected paragraph or table through the existing checked
location model, build the same layout input and deterministic font manager as
whole-document layout, and invoke the shared paragraph and table measurement
path without pagination or document mutation. Use the measured maximum when
the M23 generator sets paired row heights, then prove whole-document layout
produces the same geometry.

## Rejected alternatives

- Publish a second approximate text measurer. It would drift from the line
  breaker, fonts, table grid, and diagnostics used by pagination.
- Return only an integer twip height. Diagnostics and fractional point output
  are required to explain fallback and compare the actual layout result.
- Cache measurements in the document. This story is a pure query and does not
  need another invalidation surface.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| regression | `independent_nested_tables_measure_to_one_final_height` | Two caller-width measurements select one row height that matches whole-document layout. |
| unit | paragraph and table measurement | Width validation, fonts, wrapping, spans, margins, borders, and ordered diagnostics reuse production rules. |
| failure | measurement is pure | Invalid location or width returns an error without changing bytes or reusable layout state. |

The **test gate** is the regression test named in the backlog.

## HLD impact

- `docs/hld/03-architecture.md`
- `docs/hld/08-rendering-spec.md`
- `docs/hld/10-bindings-spec.md`
- `docs/hld/12-testing-strategy.md`
- `docs/hld/14-development-backlog.md`

## Risk routing

- Unit conversion. Preserve truncating constructors and compare exact point and
  twip boundaries used by the public result.
- Layout. Use deterministic font mode for every assertion and inspect any
  output or pagination delta explicitly.
- Public API of a published crate. State the additive owned result and method,
  run rustdoc and API inspection, and run publish dry-runs and size checks.

## Hash harness

Expected to be unchanged. Measurement is a pure opt-in query and the existing
samples do not use it.

## Implementation checklist

- [x] Add failing paragraph, table, nested-table, and invalid-location measurements.
- [x] Expose one owned measurement result and one concrete native entry point.
- [x] Reuse production input, fonts, layout rules, and diagnostics without mutation.
- [x] Prove measured equal height matches final whole-document geometry.
- [x] Run layout, public API, hash harness, full verification, and microscope gates.

## Open questions

None.
