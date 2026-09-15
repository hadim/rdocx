# F-X102, Resolve header and footer pictures in their story scope

**Status**: completed
**Sprint**: S73
**Size**: M
**Depends on**: F-255

## Problem

`Document::build_layout_input` keys related-story images as
`{story_relationship_id}\0{local_image_relationship_id}`. Header and footer
paragraph layout later calls `MediaRegistry::id_for_relationship` with only the
local identifier, so the lookup selects the missing-media sentinel and the
render drops the picture. Body pictures work because their relationship IDs
are unscoped keys.

## Spec reference

- `docs/hld/03-architecture.md`, physical story ownership.
- `docs/hld/08-rendering-spec.md`, header and footer layout.
- `docs/hld/12-testing-strategy.md`, story-scoped render coverage.
- `docs/hld/14-development-backlog.md`, F-X102.

## Approach

Carry an optional relationship-scope prefix through paragraph and drawing
layout. Resolve body media with the unscoped ID and related-story media with
the exact `{story}\0{local}` key. Apply the same scope to inline and anchored
picture resolution and image diagnostics. Preserve the existing cache key and
source rebinding rules so cached headers cannot reuse another story's media.

## Rejected alternatives

- Insert duplicate unscoped image keys into `LayoutInput`. Colliding local IDs
  from two stories would silently select the wrong bytes.
- Special-case images only in the paginator. Relationship ownership belongs at
  drawing resolution, before positioned output exists.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| differential | `header_and_footer_pictures_render_from_story_relationships` | Header, footer, and body controls match the pinned raster within the stated tolerance. |
| regression | scoped relationship collisions | Identical local IDs in body, two headers, and a footer resolve only their own bytes. |
| regression | cache and diagnostics | Header cache reuse keeps scoped media and missing or external targets report once. |

The **test gate** is the differential test named in the backlog.

## HLD impact

- `docs/hld/03-architecture.md`
- `docs/hld/08-rendering-spec.md`
- `docs/hld/12-testing-strategy.md`
- `docs/hld/14-development-backlog.md`

## Risk routing

- Layout. Use deterministic fonts and images for every baseline and review any
  hash delta explicitly.
- External oracle comparison. Pin LibreOffice and Poppler, record DPI and the
  red-pixel or raster tolerance, and retain generated inputs in code only.

## Hash harness

The feature-showcase PDF byte, page-stream, and resource fingerprints change
because its existing 400 by 40 header logo is now rendered on page 11. The
page-one PNG and every other sample fingerprint remain unchanged.

## Implementation checklist

- [x] Add a failing scoped header and footer image regression.
- [x] Carry relationship scope through paragraph drawing resolution.
- [x] Cover inline, anchor, cache, collision, and diagnostic paths.
- [x] Run deterministic PNG and PDF comparison plus full verification and microscope.

## Open questions

None.
