# F-X104, Render DrawingML picture transparency

**Status**: completed
**Sprint**: S73
**Size**: M
**Depends on**: F-217

## Problem

`oxml-drawing::Blip` preserves every child as raw XML. The rpptx resolver
therefore never sees `a:alphaModFix`, and every picture becomes an opaque
`ResolvedImage` even when the package requests partial transparency.

## Spec reference

- `docs/hld/05-drawingml-model.md`, DrawingML fill and effect ownership.
- `docs/hld/08-rendering-spec.md`, shared group opacity and presentation output.
- `docs/hld/12-testing-strategy.md`, Presentation fidelity and raster comparison.
- `docs/hld/14-development-backlog.md`, F-X104.

## Approach

Model at most one namespace-aware `a:alphaModFix` child on `Blip` with a
validated 0 through 100000 amount and an ordered raw-child slot. Preserve
unsupported effects byte for byte. Add effective opacity to resolved images
and lower it through a backend-neutral group so SVG, PDF, raster, slide-layout,
master, background, and preview paths share the same result. Multiply with
existing enclosing and animation opacity rather than replacing it.

## Rejected alternatives

- Parse raw XML inside rpptx layout. DrawingML owns this vocabulary and both
  Word and PowerPoint consumers need one model.
- Preblend pixels against white. It breaks transparent backgrounds and nested
  opacity composition.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| differential | `picture_alpha_mod_fix_matches_presentation_renderers` | Slide and layout pictures at 30 percent match pinned LibreOffice, while the control remains opaque. |
| round-trip | modeled blip effects | Prefix aliases, schema order, duplicate handling, invalid amounts, and raw sibling preservation are exact. |
| regression | output backends | SVG, PDF, raster, backgrounds, previews, and animation multiply the same opacity. |

The **test gate** is the differential test named in the backlog.

## HLD impact

- `docs/hld/05-drawingml-model.md`
- `docs/hld/08-rendering-spec.md`
- `docs/hld/12-testing-strategy.md`
- `docs/hld/14-development-backlog.md`

## Risk routing

- Any parser or serialiser. Verify schema order, prefix-tolerant reads,
  fixed-prefix writes, malformed values, and exact raw-subtree preservation.
- Layout. Use deterministic fonts and declared raster thresholds for every
  output assertion.
- Public API of a published crate. State the additive pre-1.0 surface and run
  rustdoc, API inspection, package dry-runs, and archive-size checks.
- External oracle comparison. Pin LibreOffice and Poppler and record exact
  versions, commands, DPI, and tolerance.

## Hash harness

Expected to be unchanged unless a current presentation fixture contains
`alphaModFix`. Any affected entry is an intentional behavior change and must be
reviewed alone.

## Implementation checklist

- [x] Add failing slide, layout, and opaque-control renders.
- [x] Model and serialize `alphaModFix` with ordered raw siblings.
- [x] Propagate and compose resolved image opacity across every backend.
- [x] Run oracle, parser, layout, public API, hash harness, full verification, and microscope gates.

## Open questions

None.
