# F-X209, Edit PowerPoint masters layouts and themes

**Status**: draft
**Sprint**: S91
**Size**: L
**Depends on**: F-X205, F-X206, F-X208

## Problem

Issue312 requests persisted part-owned master/layout/theme editing.
`crates/rpptx/src/lib.rs:631` retains layout records, while Presentation at
:607 has no editable master record collection. ShapesMut at :7193 addresses
one slide and group path. ShapeMut at :7822 already borrows the actual child
without a slide index, so do not add redundant ownership to that handle.
Layout and master shape authoring needs the correct part relationship owner.

## Spec reference

- `docs/hld/06-presentationml-model.md`, Public facade, Parts, shape tree,
  placeholders, preservation strategy and relationship remapping.
- `docs/hld/10-bindings-spec.md`, Python API shape and handle invalidation.
- `docs/hld/05-drawingml-model.md`, theme colours/fonts and format schemes.
- `docs/hld/08-rendering-spec.md`, Text in a shape and inherited rendering.
- `docs/hld/14-development-backlog.md`, F-X209 and full Issue312 acceptance.

## Approach

Extend existing concrete part records and shape authoring in lib.rs. A small
concrete owner discriminator addresses slide, layout or master collections.
Use the existing borrowed ShapeMut for in-place properties. Part-owned picture
insertion creates relationships on the actual owning part and content-hash
media deduplication keeps its current semantics. No parallel mirror model,
new trait, generic parameter, source module or crate.

Load and persist masters and themes with existing qualified OOXML models,
explicit dirty state and ordered writes. Add live Python slide_masters,
slide_master, SlideMaster and SlideLayout properties in existing modules.
Expose shapes, placeholders, background, show_master_shapes, text_styles,
layout name and usage. Deleting a used layout refuses before any part,
relationship, vector or handle mutation. Duplication remaps owning-part
relationships, media and shape IDs with existing copying rules.

Add solid/gradient/picture backgrounds, master title/body levels1 to5/other
font, size, colour and bullet properties, and theme colour/font mutation.
F-X205 owns common fill property wrappers and F-X208 owns theme read, so reuse
those actual integrated surfaces. Existing slide rendering must consume
persisted master/layout changes through its existing inheritance chain.

apply_theme imports colours, fonts and format schemes from pptx, potx or thmx,
with optional master/layout import. Stage the complete import and refuse
unresolved dependencies, malformed sources and unsafe relationship targets
atomically. Keep all existing slide content, opaque XML and unrelated parts.
Do not silently copy executable content with a theme. Exact selective import
and preservation policy needs resolution before approval.

## Rejected alternatives

- Repeat logos on each slide, which does not provide master editing.
- Add a slide index to ShapeMut, which already borrows its exact child.
- Rewrite all template XML when a specific part property changes.

## Test plan

**Test gate**: regression.

| Category | Test | Asserts |
|---|---|---|
| regression | master_and_layout_pictures_use_own_part_relationships | Master logo appears on every admitted slide, hidden title layout and alternate layout logo agree |
| round-trip | master_background_and_text_style_edits_follow_inheritance | Native PDF/PNG and saved PowerPoint reflect level properties and showMasterSp |
| regression | used_layout_delete_and_malformed_theme_import_are_atomic | Complete package and revisions unchanged on refusal |
| round-trip | theme_apply_preserves_existing_slide_content | Deck, potx and thmx sources apply colours/fonts/format scheme, optional master/layout import retains content |
| regression | live_master_and_layout_handles_keep_checked_identity | Related and unrelated edits have deliberate success-only invalidation |

Use existing native and Python test entrypoints. Run all three Issue312 scripts
from a fresh installed extension, strict mypy and stubtest, deterministic
native PDF/PNG, actual PowerPoint no-repair open and pinned LibreOffice
round-trip. Preserve exact unknown subtrees and schema order. Google Slides
import is best effort with explicit evidence limits.

## HLD impact

- `docs/hld/02-scope-and-non-goals.md`
- `docs/hld/05-drawingml-model.md`
- `docs/hld/06-presentationml-model.md`
- `docs/hld/08-rendering-spec.md`
- `docs/hld/10-bindings-spec.md`
- `docs/hld/12-testing-strategy.md`
- `docs/hld/14-development-backlog.md`

## Risk routing

Parser/serializer and package relationships require qualified reads, schema
sequence, atomic import and exact raw retention. Published API additions
require local-source packaging dry-run and actual affected archive inventory.
PyO3 requires installed isolated tests, typing/stubtest and both WASM checks.
Theme operations retain the deliberately pinned Word tint/shade convention.
Layout comparisons use deterministic fonts. Pin PowerPoint and LibreOffice
oracles and retain compared inputs/results. Audit dependency direction before
any cross-family use. No new source module, crate or dependency is approved.

## Hash harness

Existing immutable deck output should remain unchanged until callers edit
master/layout/theme properties. Measure changed authored output separately.
Any observed fixture delta requires an approved explicit revision before
baseline recording. No baseline movement authorized by this draft.

## Exclusive file claims

rpptx lib.rs, existing slide_parts/theme models where required, existing
rpptx-py layout/shape/dml/presentation/slide modules and stubs, existing native
and Python test entrypoints, and the HLD sections above. Source conflicts
serialize after X205, X206 and X208 completion. No worktree is claimed yet.

## Implementation checklist

- [ ] Resolve exact owner record and live handle design after prerequisites.
- [ ] Resolve selective theme import and preserve/refuse policy.
- [ ] Approve source claims, riders and expected output behavior.
- [ ] Reproduce missing public scripts before implementation.
- [ ] Implement and verify every Issue312 item with native oracle evidence.
- [ ] Obtain clean independent review and validated prepared handoff.

## Open questions

Determine the smallest part-owned record/accessor API that reuses completed
X205/X206/X208 without handle duplication. Specify apply_theme import scope,
relationship closure and existing-slide remapping for each source kind.
Prerequisites are unfinished, so this design remains draft and implementation
must wait for their reviewed completion. No product approval is inferred from
this intake record.
