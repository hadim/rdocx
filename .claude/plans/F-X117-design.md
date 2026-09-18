# F-X117, Render transparent and large raster pictures safely

**Status**: completed
**Sprint**: S73
**Size**: M
**Depends on**: F-X104

## Problem

The raster backend passes straight-alpha RGBA bytes to tiny-skia's
premultiplied pixmap representation, so a fully transparent white pixel paints
white over content below it. Presentation rendering also silently drops images
whose decoded buffer exceeds 16 MiB, including ordinary 4K and 300 dpi assets.

## Spec reference

- `docs/hld/08-rendering-spec.md`, image composition and raster outputs.
- `docs/hld/12-testing-strategy.md`, pixel and bounded-resource gates.
- `docs/hld/13-risks-and-open-questions.md`, decoded-media resource limits.
- `docs/hld/14-development-backlog.md`, F-X117.

## Approach

Premultiply decoded RGBA channels exactly once before constructing a tiny-skia
pixmap. Raise the presentation preview decode ceiling to a reviewed bound that
covers the reported 4000 by 1500 and 2100 by 2100 cases while retaining checked
dimension, multiplication, and allocation limits. Images within the bound must
render in PDF and every raster format. Images beyond it must produce a stable
render diagnostic or error and visible fallback, never disappear silently.

## Rejected alternatives

- Ignore RGB values when alpha is zero only. Every partial alpha value also
  requires premultiplication.
- Remove decoded-size limits. Untrusted packages still need a finite allocation
  boundary.
- Keep silent omission above the limit. Missing page content is not an
  acceptable degradation path.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| raster | `straight_alpha_images_composite_with_premultiplied_pixels` | Transparent white and black storage produce identical navy pixels in PNG, JPEG, and TIFF output. |
| regression | `large_pictures_render_or_report_the_decode_limit` | Reported 22.9 MiB and 16.8 MiB pictures render in PDF and raster output, while an over-limit image reports one stable failure. |
| security | decode arithmetic | Malformed and overflowing dimensions fail before allocation without panic. |

The **test gate** is the raster test named in the backlog.

## HLD impact

- `docs/hld/08-rendering-spec.md`
- `docs/hld/12-testing-strategy.md`
- `docs/hld/13-risks-and-open-questions.md`
- `docs/hld/14-development-backlog.md`

## Risk routing

- Layout or rendering. Use deterministic media bytes, sampled pixels, and PDF
  raster comparison through the pinned Poppler build.
- Security-sensitive input and resource bounds. Check every dimension and byte
  multiplication, enforce the reviewed ceiling, and fuzz malformed headers.

## Hash harness

Expected to be unchanged because current samples contain neither the invalid
straight-alpha pixels nor images above the old ceiling.

## Implementation checklist

- [x] Add straight-alpha and reported large-image failures.
- [x] Premultiply RGBA bytes at the tiny-skia ownership boundary.
- [x] Raise and document the bounded decoded-image ceiling.
- [x] Make every over-limit path diagnostic and visibly non-silent.
- [x] Run raster, PDF, security, hash harness, full verification, and microscope gates.

## Open questions

None.
