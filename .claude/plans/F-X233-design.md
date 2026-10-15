# F-X233, Repair paragraph border geometry

**Status**: approved
**Sprint**: S93
**Size**: M
**Depends on**: none

## Problem

Issue366 has two screenshots and a15tp description, with no source DOCX, code
or unit definition. The renderer already divides sz by8
(`crates/rdocx-layout/src/paginator.rs:5803`). Border geometry uses paragraph
content bounds at3889 and independently offsets edges by space around5862.
Width, clearance, clipping and corner joins need separate measurements before
selecting a repair.

## Spec reference

- `docs/hld/14-development-backlog.md`, "F-X233, Repair paragraph border geometry".
- `docs/hld/08-rendering-spec.md`, paragraph paint and "Word section geometry and page numbering", shared border-edge rendering.
- `docs/hld/12-testing-strategy.md`, "Test taxonomy", render geometry and native raster comparison contracts.
- `docs/hld/15-build-and-toolchain.md`, "Deterministic rendering".
- `docs/hld/01-glossary.md`, units and OOXML eighth-point border size.

## Approach

The user authorized an explicit diagnostic interpretation on2026-10-15: start
with15points and take a screenshot, then try raw15 if needed. Source-build a
15point candidate with w:sz="120", save/reopen and independently inspect its
actual XML. Capture deterministic product and pinned native renders and compare
the relevant crops to the issue screenshots. Retain a distinct w:sz="15"
candidate when the first interpretation does not explain the report. Both
remain inferred candidates, not the unavailable original source document.

Measure stroke thickness, w:space clearance, outside extents, clipping and join
continuity separately. Vary only one property at a time. Diagnose before changing
code. Repair the proved geometry defect in existing paginator.rs/paint carriers,
preserving unit constructors, exact schema/source representation and unrelated
paragraph placement. Include single/double styles, asymmetric edges, adjacent
paragraphs, indentation and page fragments. If both candidates already match
native paint, record that outcome and the missing-original limitation rather
than inventing a conversion fix.

No new public surface, module, dependency, product file or raster threshold is
proposed. Tests use the existing paginator.rs unit module and a scratch public facade
save/reopen consumer. No facade dev-dependency is added to the layout crate.
The Word integration entrypoint remains exclusively with F-X246 in the parallel
wave. Native evidence records actual current Word build and Poppler identity
independently from historical pins. Start after F-X232 source integration.
F-X233 alone holds the shared hash and pixel baseline lease in this wave.

## Rejected alternatives

- Multiplying or dividing border size before reproducing the defect.
- Calling either inferred candidate the exact original input.
- Loosening raster thresholds or fixing all border styles speculatively.

## Test plan

The **test gate** is **regression**.

| Category | Test | Asserts |
|---|---|---|
| regression | `fx233_fifteen_point_border_has_correct_paint_geometry` | Saved sz120, space, measured stroke width, clearance, extents and joins agree with independently measured geometry |
| regression | `fx233_raw_fifteen_border_size_keeps_eighth_point_units` | Distinct sz15 control preserves1.875point interpretation and makes unit differences explicit |
| regression | border fragment/corner controls | Adjacent, indented, asymmetric and split paragraphs do not clip or detach the selected border edges |
| round-trip | saved border input inspection | Size, space, style and opaque siblings survive reopen without source changes from rendering |
| differential | inferred input native paint comparisons | Pinned native raster and geometry qualify the diagnosed repair under unchanged thresholds, with interpretation limitation retained |

Run focused layout checks/tests, deterministic captures and independent crop
measurement, scoped verify and independent microscope. A missing original
prevents claiming exact-original reproduction even when candidate gates pass.

## HLD impact

- `docs/hld/08-rendering-spec.md`
- `docs/hld/12-testing-strategy.md`

## Risk routing

- Layout/pagination: deterministic font mode, independent geometry and explicit
  baseline attribution. Do not change unrelated paragraph layout.
- Units: preserve truncating constructors and eighth-point size semantics.
- External oracle: pin actual native producer, source XML, output digests,
  rasterizer and DPI. Preserve every existing tolerance.

## Hash harness

Any intentional change must be limited to fixtures exercising the diagnosed
border behavior, with independent expected width/clearance/join evidence.
Unrelated outputs remain unchanged. No baseline change until the diagnosis
and exact affected entries have been reviewed.

## Implementation checklist

- [ ] Claim after all sprint designs are approved.
- [ ] Build15point candidate, inspect saved XML and capture screenshot.
- [ ] Compare raw15 candidate if necessary and record interpretation limits.
- [ ] Diagnose and repair only the independently proved defect.
- [ ] Pass scoped checks, independent microscope and structured handoff.

## Open questions

Resolved by user steering: try15points first and raw15 second if needed.
The original document remains unavailable, which is a recorded evidence limit,
not an unanswered product choice.
