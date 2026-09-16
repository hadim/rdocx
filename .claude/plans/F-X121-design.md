# F-X121, Adopt PR 123 authored line-chart portability

**Status**: completed
**Sprint**: S73
**Size**: S
**Depends on**: F-X087

## Problem

The authored ChartML path in `crates/oxml-chart/src/lib.rs` emits axis-title
text without an explicit layout or overlay decision. It also omits false
chart-level marker and smoothing values for line plots. Viewers may therefore
position a title over labels, invent per-point markers, or smooth the authored
line. Kevin Brown's PR 123 supplies a focused correction at exact head
`1375b6342548e79ec17faa99ac76a57e4a1c5e9b`.

## Spec reference

- `docs/hld/09-charts-spec.md`, "The ChartML model" and "Authoring API".
- `docs/hld/12-testing-strategy.md`, the portable authored-chart gate.
- `docs/hld/14-development-backlog.md`, F-X121.
- GitHub PR 123 at exact head `1375b6342548e79ec17faa99ac76a57e4a1c5e9b`.

## Approach

Preserve the contributor's focused implementation. Seed a newly authored
`CT_Title` ordered-raw boundary after `c:tx` with `c:layout` followed by
`c:overlay val="0"`. Seed the existing line `PlotMarkup` marker and smooth
lexical state so the typed writer emits `c:marker val="0"` and
`c:smooth val="0"` in their existing schema positions. Do not change the
public API, parser model, workbook generation, palette handling, or any other
chart family.

Harden the contributor regression around exact direct-child order, false
values, non-line exclusion, and parse and rewrite stability. Retain PR 123 and
Kevin Brown in the release contribution inventory and result comments.

## Rejected alternatives

- Merge the draft pull request directly. Sprint work lands through the feature
  lifecycle, and the contributor patch still needs the sprint's schema-order
  and round-trip gates.
- Change `Plot::line` defaults to true or expose another public option. The
  request is portable serialization of the existing false defaults, not new
  chart behavior.
- Model title layout and overlay as new public fields. The authored constants
  need no new abstraction, while ordered raw children already preserve their
  schema position.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| regression | `authored_charts_emit_portable_viewer_defaults` | Each authored axis title emits `c:tx`, `c:layout`, then false `c:overlay`, and an authored line emits false marker then false smooth before its axis ids. |
| round-trip | authored line parse and rewrite | The portable defaults survive `CT_ChartSpace` parse and serialization without changing workbook references or palette markup. |
| regression | non-line authored charts | Bar, pie, doughnut, area, scatter, and radar output do not acquire line-only marker or smooth defaults. |
| differential | pinned Pages authored-chart oracle | The generated line chart opens and exports without title overlap or invented line markers at the repository-pinned Pages version. |

The **test gate** is the regression test named in the backlog.

## HLD impact

- `docs/hld/09-charts-spec.md`
- `docs/hld/12-testing-strategy.md`
- `docs/hld/14-development-backlog.md`

## Risk routing

- Any parser or serializer. Prove exact title and line-plot child order,
  prefix-correct output, false lexical values, and parse and rewrite stability.
- Rendering, PDF, SVG, or raster output. Run the focused authored-chart gate
  and confirm that all 49 output-stability entries remain unchanged.
- External oracle or corpus comparison. Use the repository-pinned Pages
  Creator Studio 15.1.1 build 7044.0.273 protocol from
  `docs/hld/09-charts-spec.md`. Treat the contributor's Pages 14.5 observation
  as provenance, not replacement oracle evidence.

## Hash harness

Expected to be unchanged because no checked-in sample authors a chart. Any
delta blocks completion until separately explained.

## Implementation checklist

- [x] Reconcile PR 123's exact head, review state, comments, and CI immediately before implementation.
- [x] Apply the contributor title layout and false overlay output in schema order.
- [x] Apply explicit false marker and smoothing output only to authored line plots.
- [x] Harden direct-child order, exclusion, parse and rewrite, workbook, and palette assertions.
- [x] Run focused oxml-chart, clippy, hash harness, external-oracle rider, microscope, and full sprint verification gates.
- [x] Record Kevin Brown and PR 123 in the release contribution inventory and queue a human result comment for the verified integrated result.

## Open questions

None. The user explicitly asked to include PR 123 in S73 and has already
approved the sprint work.
