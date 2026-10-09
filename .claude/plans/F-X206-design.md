# F-X206, Complete PowerPoint shape authoring and live handles

**Status**: draft
**Sprint**: S91
**Size**: L
**Depends on**: F-X194, F-X197
**Issue**: 309, captured in S91-intake-issues.json on 2026-10-09
**Prerequisites**: F-X194 and F-X197
**Implementation**: not claimed, no source changes or new gate evidence

## Problem

Issue309 requires shape operations missing in native Rust as well as Python,
plus two concrete ancestor-handle defects. Complete acceptance is:

- Shape, picture, group and table alt_text and alt_title in p:cNvPr, plus
  the a16:decorative extension through shape.decorative.
- shape.flip_h and flip_v through DrawingML transform flags.
- shapes.group(existing_shapes) and group.ungroup, preserving slide positions
  through nested coordinate systems, rotation and flips.
- connector.begin_connect(shape, site) and end_connect, storing stCxn/endCxn
  and moving endpoints to preset-geometry connection sites.
- placeholder.insert_picture(path), cropping the picture to fill the existing
  placeholder rectangle.
- cell.vertical_anchor through a:tcPr anchor.
- Internal run slide hyperlinks with ppaction://hlinksldjump.
- Run highlight and small caps through a:highlight and cap="small".
- Native align/distribute helpers for horizontal/vertical positions relative
  to the slide or selection, usable by Python, CLI and WASM consumers.
- Native placeholder names matching PowerPoint, such as Title1 and Content
  Placeholder2, rather than generic PlaceholderN.
- Slide layout handles remain live after add_slide.
- Slide handles remain live after replacing the title TextFrame.text, while
  unsafe descendant paths still fail loudly.
- Every item saves/reopens, has a typed stub, and saved files open in
  PowerPoint without repair. Flips, groups, connections and crop require
  actual render evidence.

Google Slides import remains best effort. Descr, flips, groups, glued
connections, picture placeholder crops and cell anchor should use standard
OOXML. Highlight is text background, not a paragraph or shape fill. Title
and decorative import differences are documented without claiming CI support.
Issues264 and281 remain excluded.

## Spec reference

- `docs/hld/10-bindings-spec.md`, Python API shape and checked handle invalidation.
- `docs/hld/02-scope-and-non-goals.md`, native and Python capability rows.
- `docs/hld/04-opc-and-packaging.md`, The package and owned relationships.
- `docs/hld/08-rendering-spec.md`, Text in a shape, Tables and deterministic output.
- `docs/hld/14-development-backlog.md`, F-X206 and the full linked issue acceptance.

## Approach

Native ShapeRef and ShapeMut are rpptx/src/lib.rs:7805 and :7822. ShapeMut
already routes set_position:7855, set_size:7861, set_rotation:7870,
set_name:7876, set_fill:7891, set_line:7897 and set_crop:7912 through
concrete typed child kinds. AlternateContent is read-only for existing
mutations. Retain that boundary and atomic unsupported errors for new ones.

Add concrete alt text/title/decorative getters and checked native setters
over existing shape/picture/group/graphic-frame non-visual property handling.
Do not parse every cNvPr operation in the Python wrapper. Existing model
files are rpptx-oxml/src/shape_tree.rs, picture.rs, graphic_frame.rs and
connector.rs. Imported unknown extensions and child positions must retain
exact bytes. Decorative must update its actual namespaced extension without
discarding unrelated extLst entries or duplicating its URI. Foreign and
rebound a16 prefixes remain preservation controls, not decorative flags.

CT_Transform2D already has flip_horizontal and flip_vertical in
oxml-drawing/src/xfrm.rs:107 and :108. Add native facade accessors preserving
the other transform fields. Boolean false versus missing flags must be
documented. Support each actual transform-bearing shape kind, including
groups and graphic frames, with checked unsupported errors.

Existing ShapesMut at native lib.rs:7193 and native group creation at :7241
are the insertion points for group/ungroup. Existing group child mutation
at :7839/:7847 and coordinate helpers at :7480 through :7585 provide current
mechanics. They do not provide an operation to group an existing selection.
Stage the whole selection mutation, validate same owner, duplicate selection,
ancestor/descendant collisions, actual child kind, transform bounds and IDs,
then commit once. Preserve z-order, relationships, IDs and unsupported raw
nodes. Ungroup converts children into the parent coordinate space rather
than copying offsets or dropping rotation. Do not silently flatten opaque
AlternateContent children or approximate unsupported transforms.

The existing pivot_shift documentation at lib.rs:7556 explicitly states that
PowerPoint flips then rotates, while the renderer rotates then flips. They
differ for groups that are both rotated and flipped. This earns an actual
combined-transform oracle and an explicit decision on rendering semantics.
No assumption that grouping is an output-neutral wrapper is justified.
Reject unsupported decomposition atomically or implement and review the
actual affine transformation needed by the issue. Existing unit constructors
still truncate. Geometry algorithms must document any independently required
rounding and may not change the constructors to get a desired picture.

Connectors already model start_connection and end_connection at
rpptx-oxml/src/connector.rs:39 and :40. CT_Connection serialization respects
schema order and retained connector property bytes. Public glue and endpoint
site resolution are missing. Add native owner-aware operations using actual
shape IDs and preset geometry connection sites. Validate the site index and
geometry before editing connector or target. Define the behaviour when the
target later moves, changes geometry, is removed or is regrouped. A stored
stCxn alone does not satisfy the endpoint requirement.

Picture placeholder filling must replace the selected placeholder tree child
with a picture carrying its placeholder identity and metadata, allocate its
image part/relationship through the native owner, and crop using image aspect
ratio and inherited placeholder rectangle. Existing ShapeMut.set_crop:7912
and Presentation image operations :3358/:3489 are reusable. A generic picture
insert followed by deleting the placeholder is not equivalent without
preserving order, identity, relationships and inherited geometry atomically.

Cell anchor is a:tcPr anchor. X194 TextFrame.vertical_anchor belongs to
a:bodyPr, and they are different sources. Native TextFrame.set_vertical_anchor
at lib.rs:9079 cannot substitute for the cell property. Add checked
TableCellRef/Mut cell-anchor accessors on the existing typed cell properties,
then expose them in current rpptx-py/table.rs. Formatting changes preserve
cell/grid handles and nested text topology.

Presentation.set_run_hyperlink at native lib.rs:3967 handles external address
relationships. Existing shape_target_slide:3742 and set_shape_hyperlink:3787
provide internal-link mechanics to inspect and reuse. Extend the native run
contract for a checked target slide and action, respecting owner slide and
relationship pruning. Bind it through the existing text.rs Hyperlink class.
Retain X194's explicit cell-run hyperlink refusal until a real cell native
owner path is separately approved. Test refusal before relationship mutation.

CT_TextCharacterProperties at oxml-drawing/src/text/paragraph.rs:610 has
all_caps Option<bool>, but no typed small_caps or highlight. The existing
test character_all_caps_round_trips_while_small_caps_stays_unmodelled at
:2285 intentionally preserves cap="small" raw. Promote only the requested
model fields, retain existing public all_caps behaviour, update raw ownership
without duplicate cap attributes, and preserve unsupported siblings in their
ordered slots. Then update native resolution/rendering where necessary.
Python-only setters over raw attributes would leave native consumers behind.

Align/distribute belong in existing native ShapesMut/Presentation operations,
with Python wrappers consuming checked same-owner selections. Use slide
coordinates, actual transformed bounds and an explicit reference rectangle.
Validate spacing/selection cardinality before writes. Final design must
state whether visual rotated bounds or unrotated geometry defines alignment,
and prove that choice against PowerPoint. Do not add unrelated CLI commands
or WASM exports without an approved signature scope.

Placeholder naming belongs in native add_slide/template-derived placeholder
creation, not only in Python Shape.name. Preserve explicit imported names.
Source-build all supported layout kinds and compare with pinned PowerPoint
names and numbering rules, including duplicate shape IDs/name collisions.

## Concrete handle partition proposal

The current source has one RevisionCounter in PyPresentation at
rpptx-py/presentation.rs:151. PySlideLayout and its collection at slide.rs:25
and :67 store ContentPath and validate the same global revision as all
structural descendants. add_slide bumps it at slide.rs:586. TextFrame.text
at text.rs:223 and Paragraph.text at :556 also bump it. This is why title
text invalidates Slide, and adding a slide invalidates an unchanged layout.
Removing revision checks globally would let indexed handles silently bind to
different objects and is rejected.

Preferred smallest direction for design review:

1. Keep the existing structural revision for indexed shape/slide topology.
   Successful shape insertion/removal/move/grouping and slide removal/reorder
   retain loud stale-path behaviour. Formatting does not bump it.
2. Generalize the proposed X194 cell-text generation into one concrete text
   topology RevisionCounter in PyPresentation and one optional text revision
   field on existing ContentPath. The current approved X194 field is named
   text_revision with PyPresentation.text_revisions, reconciled in F-X194
   before either story is claimed. F-X194 initially captures Some for cell
   text only. This story adds actual shape-text consumers to the same fields,
   without a second counter or another field migration.
3. Shape-backed and cell-backed TextFrame paragraph/run/font consumers
   capture that optional generation directly and inherit it into descendants.
   Text replacement, paragraph/run insertion and cell replacement/merge/split
   bump it only after success. Structural revision validation remains first.
   Shape/Slide/Table/Cell identities without text generation remain live
   across text-only topology changes. Conservative invalidation of all
   presentation text descendants is explicit and acceptable only if tested
   and documented. No per-cell map or generic revision policy framework.
4. Give the actual layout handle kind a separate concrete layout-inventory
   generation in PyPresentation. PySlideLayout/Collection check that captured
   generation and native bounds, rather than unrelated shape or text changes.
   Existing-layout add_slide does not change it. Any operation that replaces,
   removes, reorders or imports the layout inventory updates it after success.
   Audit all present native layout mutation paths before claiming this.

The text counter serves concrete shape and cell text consumers today. The
layout counter serves actual SlideLayout and SlideLayoutCollection consumers
today. Both use existing RevisionCounter, not a new trait, map, builder or
dynamic framework. The partition is an explicit contract migration and must
retain meaningful stale checks and exact recovery messages.

Checked stable-owner IDs are a possible narrower alternative if the layout
inventory never reorders. The current native facade exposes SlideRef.id at lib.rs:7674 and
ShapeRef.non_visual_id at lib.rs:9636, but no indexed stable-ID resolver or layout owner-part
projection was found. Do not invent an existing API or blindly resolve reused
IDs. Any stable-owner approach first needs minimal native lookup, owner-kind
checks, uniqueness/non-reuse reasoning and removed-owner tests. A stable
slide handle is not required to fix title-only invalidation because the index
is unchanged. Prefer the counter partition over unrelated stable-ID machinery
unless a concrete accepted operation requires handle survival across reorder.

Alternative retaining X194's cell-only field would need a second shape-text
field/counter and duplicate inheritance logic. That increases cases readers
must consider. Do not choose it merely because the earlier draft existed.
The field names are reconciled with F-X194. Complete the mutation and
constructor audit before implementing the additional shape partition.

## Ownership and affected paths

X194 owns cell text traversal and the initial cell topology invalidation
contract. X197 owns shared colour validation and colour formats, used for
highlight input without another extractor. X205 owns the fresh 16:9 template
and metadata, so placeholder names and inherited geometry must be reconciled
with its authenticated template update. X206 owns shape mutation and the
ancestor-handle partition after explicit plan reconciliation.

Likely exclusive files are native rpptx/lib.rs, current rpptx-oxml shape,
picture, connector and graphic-frame files, oxml-drawing/xfrm.rs and current
text model/resolution files only where required, existing rpptx-py
presentation.rs, slide.rs, shape.rs, text.rs and table.rs, shared
oxml-py-support/src/lib.rs for the coordinated ContentPath field, current
stub/enum files and current tests. These overlap prerequisites, so claim
after integration or agree serialized ownership. No new file is prescribed.

## Rejected alternatives

- Duplicate prerequisite colour/table APIs instead of reusing their actual result.
- Treat the issue's claimed native parity as proof where source inspection shows gaps.
- Drop unknown source or stale-path checks to make the missing API appear usable.

## Test plan

**Test gate**: regression.


Reproduce each requested operation through installed Python on the claimed
Base, and use native public source-built regressions for operations missing
in Rust. Preserve two exact current stale-handle failures as baseline
evidence before changing revision semantics.

| Test | Required evidence |
|---|---|
| test_shape_accessibility_and_flips_round_trip | All named kinds, true/false/None policy and foreign extension preservation |
| group_existing_shapes_preserves_transformed_slide_positions | Rotated/flipped/nested geometry, z-order, IDs and exact opaque nodes |
| ungroup_preserves_combined_transform_and_relationship_ownership | Inverse transform, invalid/opaque atomic refusals and native render oracle |
| glued_connector_endpoints_follow_preset_connection_sites | Endpoint facts, invalid sites, move/delete/regroup behaviour |
| test_picture_placeholder_fill_keeps_placeholder_and_crops_to_fill | Landscape/portrait images, inherited geometry, package ownership and rendering |
| test_cell_anchor_is_distinct_from_text_frame_anchor | Actual tcPr/bodyPr values, save/reopen and unchanged text handles |
| test_internal_run_slide_hyperlinks_prune_owner_relationships | Same/cross-owner targets, deletion and no-orphan rejected cell links |
| test_run_highlight_and_small_caps_preserve_other_caps_metadata | Typed promotion, all/none/small, unrelated raw attributes/children and render |
| align_and_distribute_use_checked_transformed_selections | Every direction/reference, cardinality, bounds and atomic refusal |
| test_native_placeholder_names_match_powerpoint | All supported layouts, duplicate names and imported-name preservation |
| test_layout_handles_survive_add_slide_but_detect_layout_change | Collection/item identity, actual inventory mutation and exact stale recovery |
| test_slide_survives_title_text_while_descendants_stale | Slide/Shape live, old paragraph/run/font stale, fresh paths usable |
| test_text_topology_generations_cover_shape_and_cell_consumers | Coordinated X194 replacement/merge/split/add operations, no-op and failure |
| test_true_structural_mutations_still_reject_old_index_paths | Remove/reorder/group boundaries and no silently rebound handles |

Existing Python test_documented_examples.py contains staleness cases at :215,
:314 and :359, add_run at :1553, fill/line :2328, table :2412, picture crop
:3078, shape/slide reorder :3137/:3322 and shape slide links :4049. Update
only the expected contract cases, retaining old unsafe-path controls. Use
current typing_smoke.py and handwritten stub for strict mypy/stubtest.
Native tests stay in existing lib.rs/model modules and the current integration
entrypoint. No new integration binary.

Round-trip twice, with alternate/default namespace aliases, rebound and
foreign names, unknown extensions and exact unsupported bytes. Atomic
failures retain package bytes, relationships and both topology generations.
Getter reads do not create properties or advance revisions. Renderer claims
need deterministic fonts and actual native PowerPoint evidence for combined
rotation/flip, glue routing and placeholder fill. PowerPoint open-without-
repair must be separately observed, not inferred from library parsing.

## Risk routing

Matched rows are unit conversion, parser/serializer, rendering/layout,
published native API, PyO3, dependency edges, colour/theme and external
oracles. Read HLD01/03/04/05/06/07/08/10 and HLD15 packaging, plus the
canonical differential-testing skill. Preserve canonical truncation, child
order, exact raw XML, retained colour transforms and the Word tint routine.
Earned riders include actual published archive dry-run/size/measurements,
WASM compilation, binding exclusions, pinned deterministic render evidence,
normal scoped verification and independent microscope.

Expected wrapper/handle-only default harness delta is none. Placeholder-name
defaults, promoted native render semantics and a combined group transform
correction can intentionally change output. Each actual default change needs
its own labelled reviewed commit and exact delta. Root owns the exclusive
baseline, coordinated with X205's template change. No unexplained delta may
be waived as a harmless binding change.

## HLD impact

- docs/hld/02-scope-and-non-goals.md, actual native/Python capability rows.
- docs/hld/03-architecture.md, concrete shared path/counter ownership if changed.
- docs/hld/05-drawingml-model.md, caps/highlight, transforms and connector sites.
- docs/hld/06-presentationml-model.md, accessibility, group/placeholder/link ownership.
- docs/hld/07-inheritance-and-resolution.md, geometry and placeholder resolution if changed.
- docs/hld/08-rendering-spec.md, actual transform/highlight support and deterministic deltas.
- docs/hld/10-bindings-spec.md, exact methods, public types and revised invalidation contract.
- docs/hld/14-development-backlog.md, complete delivered acceptance and retained boundaries.

## Hash harness

Wrapper exposure should preserve existing defaults. Actual native semantic
changes require measured declared deltas and separate labelled commits before
any exclusive baseline update. The risk discussion above identifies those
possible changes. No baseline movement or implementation is approved by this draft.

## Implementation checklist

- [ ] Resolve technical contracts and actual native gaps against prerequisites.
- [ ] Approve complete issue acceptance, source claims and measured output policy.
- [ ] Claim only after actual prerequisites and file conflicts clear.
- [ ] Prove installed public Base regressions before production changes.
- [ ] Implement every requested capability, stubs and existing-entrypoint tests.
- [ ] Run scoped gates and exact applicable oracle/risk riders.
- [ ] Obtain independent zero-defect, zero-smell review and prepare handoff.

## Open questions

1. Reconcile the one shared text generation and layout generation with X194's
   approved cell-only design. Audit every mutation and descendant constructor.
2. Confirm native layout inventory mutation paths and whether concrete
   generations suffice without stable-owner lookup. No removal of checks.
3. Establish PowerPoint's combined rotated/flipped group result, then choose
   correct affine implementation or explicit unsupported atomic boundaries.
4. Define connection-site routing/lifecycle after move, regroup and removal,
   including preset guides and unsupported custom geometries.
5. Set exact native align/distribute selection/reference signatures and the
   visual-bound versus geometry-bound contract.
6. Choose typed caps/highlight ownership while retaining all_caps compatibility
   and exact imported small-cap/raw metadata.
7. Reconcile placeholder naming and fill identity with X205's fresh template
   geometry update and native PowerPoint provenance.

These require reviewed technical decisions, not invented user approval gates.
This draft does not authorize implementation or baseline changes.
