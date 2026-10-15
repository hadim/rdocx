# F-X246, Honor chart legend position and overlay during rendering

**Status**: approved
**Sprint**: S93
**Size**: L
**Depends on**: none

## Problem

Chart rendering fixes plot bounds before legend measurement
(`crates/oxml-chart/src/lib.rs:1156`). render_legend checks only legend presence
and paints fixed upper-right rows inside the plot at3054. CT_Legend retains
all children opaquely at7261. Issue383's default/bottom/left repros therefore
cannot honor position or non-overlay placement in either document family.
The issue also requests Python legend controls in both families.

## Spec reference

- `docs/hld/14-development-backlog.md`, "F-X246, Honor chart legend position and overlay during rendering".
- `docs/hld/09-charts-spec.md`, chart legend model, package ownership and shared rendering.
- `docs/hld/08-rendering-spec.md`, chart group geometry and cross-format rendering.
- `docs/hld/10-bindings-spec.md`, Word chart authoring and PowerPoint live Chart ownership.
- `docs/hld/12-testing-strategy.md`, "Test taxonomy" and "The SVG page golden gate", cross-format chart evidence.
- `docs/hld/15-build-and-toolchain.md`, "Deterministic rendering" and "Packaging".
- `docs/hld/04-opc-and-packaging.md`, package preservation and qualified relationships.
- `docs/hld/06-presentationml-model.md`, "Parts", namespace tolerance and schema sequence.

## Approach

Capture the exact Issue383 six-package script. Model/project only owned legendPos
and overlay children using expanded names while preserving unknown children,
attributes and unmodified producer bytes. Carry actual ancestor namespace
bindings into legend projection and honor child-local shadowing, rejecting
malformed or duplicate owned scalar children atomically. Never infer chart
ownership from local names or an invented c binding. Test foreign same-local-name
children, inherited producer prefixes, exact no-op bytes and shadowed prefixes.
Default position right, overlay false.
Support right, left, top, bottom and top-right. Measure real shaped labels and
keys before selecting plot bounds. Non-overlay legends reserve exterior bands,
with bounded layout for long labels and small charts. Overlay true retains the
full plot and intentionally paints within it. Absent/hidden legends reserve no
band. Reuse the shared renderer so both formats consume one geometry decision.

The user explicitly approved full Python API inclusion for both families
on2026-10-15. Use existing files and avoid a new Word Chart proxy:

```python
doc.chart_legend(paragraph)  # dict with visible, position, overlay
doc.set_chart_legend(paragraph, *, visible=None, position=None, overlay=None)
chart.has_legend           # PowerPoint existing Chart, read/write bool
chart.legend_position      # read/write: r, l, t, b, tr
chart.legend_overlay       # read/write bool
```

The Word paragraph is the live same-document direct-body handle returned by
add_chart, containing exactly one owned chart. Unknown, stale, foreign, absent
or ambiguous owners refuse atomically. Effective getters use right/false for
missing properties, and visible=false when no legend exists. None mutation
arguments mean leave unchanged. Empty mutation is a no-op. Setting position or
overlay alone preserves visibility. Hiding removes only the owned legend and
showing creates a default legend when absent. Position/overlay-only mutation
of an absent legend is a documented no-op and does not persist hidden settings.
A combined visible=True mutation creates the selected settings. Hiding deliberately
removes the complete legend subtree and discards its settings. Preservation of
unknown legend children applies to edits while the legend is present. Repeated
values must not invalidate
held owners or rewrite resources unnecessarily. Document exact hidden-state
property behavior and test it before publication.

Use concrete shared LegendPosition and a three-field LegendSettings value in
existing oxml-chart source, with native Word paragraph-index and PowerPoint
slide/shape-id getters/setters consumed by bindings. Reuse actual qualified
chart ownership and copy-on-write rather than mutating a shared part in place.
PPT chart_owner and replace_chart_data at rpptx/src/lib.rs:11998,12083 provide
the ownership precedent. Legend edits preserve workbooks and unrelated chart
style/relationships. Python validates input types and prepares owner/revision
publication before committing the complete native candidate. Update existing
stubs and strict typing controls. Additive APIs retain current add_chart results.

Independent native Word and PowerPoint exports qualify geometry and paint,
with actual build identities and deterministic product fonts. Use the existing
chart oracle/golden carriers and text receipts. Define case-specific measured
bounds before comparison. Do not substitute self-generated golden output for
the differential gate. No new crate, module, product file, trait, generic,
feature or dependency is proposed.

## Rejected alternatives

- Renderer-only closure, since the user explicitly includes both Python APIs.
- Fixed legend width, assuming every label fits58points.
- Rewriting the entire opaque legend or shared chart/workbook graph.
- A new Word Chart handle hierarchy for three controls on an existing paragraph.

## Test plan

The **test gate** is **differential**. Extend existing oxml-chart unit tests,
both native integration entrypoints and existing Python test files.

| Category | Test | Asserts |
|---|---|---|
| regression | `fx246_legend_position_reserves_measured_plot_space` | All five positions with absent/false overlay have measured non-overlapping plot and legend rectangles |
| regression | `fx246_overlay_and_absent_legends_keep_the_correct_plot` | True overlay, absent and hidden legends retain appropriate plot geometry |
| regression | `fx246_legend_labels_fit_bounded_chart_geometry` | Long labels, constrained bounds and multiple series remain finite, bounded and legible or explicitly refused |
| round-trip | `fx246_legend_mutation_preserves_opaque_source_and_order` | Prefix aliases, unknown children, schema order and unrelated chart/workbook bytes survive getters and edits |
| integration | both Python legend APIs | Defaults, every position, type errors, no-op, stale/foreign/ambiguous ownership, shared charts, atomic refusal and save/reopen |
| differential | exact default/bottom/left packages in both formats | Three distinct deterministic renders, independent plot/legend bounds, no non-overlay overlap and pinned native Office comparison |

Retain existing raster thresholds. For new cropped legend placement comparisons,
use150DPI,1.0point geometry tolerance and the existing0.99 block-luminance SSIM
contract where equivalent fonts permit it. A proved native/product font mismatch
must be diagnosed and recorded, never hidden by threshold relaxation. Include
a negative control moving the legend into the plot so actual overlap checks fail.

Run changed-crate tests/checks/clippy, current installed packages, strict typing
and stubs, both WASM graphs, publication dry-runs/size, scoped verification and
independent zero-finding microscope.

## HLD impact

- `docs/hld/09-charts-spec.md`
- `docs/hld/08-rendering-spec.md`
- `docs/hld/10-bindings-spec.md`
- `docs/hld/12-testing-strategy.md`
- `docs/hld/15-build-and-toolchain.md`

## Risk routing

- Parser/serializer: expanded-name reads, fixed schema-ordered writes and exact
  preserved opaque payload tests for all unowned legend children.
- Layout/shaping: deterministic fonts, measured text extents, finite bounded
  geometry and separately attributed intentional chart raster changes.
- Public APIs: additive pre-1.0 types and methods, denied-warning rustdoc,
  verified publish dry-runs and actual archive size assertions below10MiB.
- PyO3/WASM: both installed families, genuine owner publication/atomicity,
  strict typing and stub agreement, both target checks. Final workspace tests
  exclude both extension crates.
- External oracle: pin actual native Word/PowerPoint, rasterizer, source and
  output hashes, DPI and geometry/raster outcomes. Preserve old evidence.

## Hash harness

Intentional changes are limited to chart-bearing entries with visible legends
whose existing placement differs from the requested/default non-overlay
geometry. Enumerate and review affected outputs against independent measurements
before recording. Hidden/absent legends and unrelated entries remain unchanged.
This story exclusively owns chart baseline updates in its wave.

## Implementation checklist

- [ ] Capture exact source, claim and establish both-format Base failures.
- [ ] Implement preserved legend properties and measured shared geometry.
- [ ] Implement both approved native/Python surfaces with atomic ownership.
- [ ] Qualify exact repro, full position/overlay matrix and independent native paint.
- [ ] Review output deltas and pass all riders, scoped verify and microscope.
- [ ] Prepare structured handoff with no worker push or sprint ledger edits.

## Open questions

Resolved: user includes visibility, position and overlay APIs for both Word and
PowerPoint in this story. No Python request is deferred. The concrete paragraph
and existing Chart contracts above are the approved implementation boundary.
