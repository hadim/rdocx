# F-262, Corpus drawings, text boxes, and watermarks

**Status**: completed
**Sprint**: S73
**Size**: L
**Depends on**: F-252, F-255, F-260

## Problem

The facade can add basic inline and anchored images and fixed-default
watermarks, but it cannot author the complete M23 drawing set. Crop, full
anchor and wrapping controls, rotated text boxes, text direction,
section-scoped watermark variants, and modeled compatibility fallbacks remain
incomplete or accessible only below the public facade.

## Spec reference

- `docs/hld/03-architecture.md`, story and drawing relationship ownership.
- `docs/hld/04-opc-and-packaging.md`, drawing media and watermark package invariants.
- `docs/hld/05-drawingml-model.md`, WordprocessingDrawing and compatibility structure.
- `docs/hld/08-rendering-spec.md`, anchored drawings, text boxes, and watermarks.
- `docs/hld/10-bindings-spec.md`, native Word drawing and watermark authoring.
- `docs/hld/12-testing-strategy.md`, watermark golden and M23 conformance.
- `docs/hld/14-development-backlog.md`, F-262.

## Approach

Complete the native facade with concrete option values for inline and floating
image crop, size, position, relative anchor, wrap mode and distances, z-order,
and behind-text behavior. Add public text-box authoring that owns a typed story
body, rotation, text direction, and geometry. Extend watermark operations with
section and header-variant selection while retaining the existing defaults.
Build the required DrawingML primary branch and modeled AlternateContent or
VML fallback together in one staged operation, using story-local media
relationships and identifier allocation. Do not expose raw header XML.

## Rejected alternatives

- Accept arbitrary DrawingML or VML strings. The M23 generator must prove the
  public typed facade, and raw XML can violate namespace and relationship
  ownership silently.
- Put Word drawing behavior in `oxml-drawing`. Anchors, text boxes, and
  watermarks here are WordprocessingML constructs.
- Replace every existing watermark in a header. Only API-owned watermarks are
  eligible for replacement.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| differential | `m23_drawings_text_boxes_and_watermarks_match_word` | Every private-corpus construct matches reviewed geometry and compatibility structure. |
| round-trip | drawing option matrix | Crop, size, anchor, wrap, rotation, direction, and fallback branches reopen with exact owner relationships. |
| golden | section-aware watermark pages | Default, first, and even headers select the expected watermark before ordinary content. |
| failure | staged drawing invariants | Invalid geometry, story, resource, or exhausted identity publishes no package change. |

The **test gate** is the differential test named in the backlog.

## HLD impact

- `docs/hld/02-scope-and-non-goals.md`
- `docs/hld/03-architecture.md`
- `docs/hld/04-opc-and-packaging.md`
- `docs/hld/05-drawingml-model.md`
- `docs/hld/08-rendering-spec.md`
- `docs/hld/10-bindings-spec.md`
- `docs/hld/12-testing-strategy.md`
- `docs/hld/14-development-backlog.md`

## Risk routing

- Unit conversion. Preserve truncating conversions and attribute every drawing
  geometry delta to its reviewed option.
- Any parser or serialiser. Verify Word drawing, text-box, AlternateContent,
  and VML child order plus exact unsupported subtree preservation.
- Layout. Use deterministic fonts and images for every drawing and text-box
  render baseline.
- Public API of a published crate. State the additive pre-1.0 surface, inspect
  rustdoc and API changes, and run package dry-runs and size checks.
- External oracle comparison. Pin Word, LibreOffice, and Poppler and record the
  exact structure, DPI, and raster tolerances.

## Hash harness

Expected changes are limited to samples that opt into new drawing options.
Each delta must be reviewed as an intentional drawing behavior change.

## Implementation checklist

- [x] Add failing inline, anchor, crop, wrap, text-box, and watermark differentials.
- [x] Complete story-aware typed drawing and text-box authoring.
- [x] Complete section-aware watermark selection and modeled compatibility branches.
- [x] Validate and reopen the complete package before publishing any mutation.
- [x] Retire the completed DOCX-027 capability owner and its workflow expectation.
- [x] Run parser, layout, public API, oracle, hash harness, full verification, and microscope gates.

## Open questions

None.
