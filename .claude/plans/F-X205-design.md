# F-X205, Complete Python PowerPoint text fill and export parity

**Status**: draft
**Sprint**: S91
**Size**: L
**Depends on**: F-X194, F-X197
**Issue**: 308, captured in S91-intake-issues.json on 2026-10-09
**Prerequisites**: F-X194 and F-X197
**Implementation**: not claimed, no source changes or new gate evidence

## Problem

Issue308 reports Python omissions and two default-template defects. Its claim
that every omission already exists in native Rust needs qualification against
actual source. Some requested exports and writable metadata lack a native
facade, and notes PDF is already exposed in Python.

The complete issue inventory is:

- Fresh Presentation has no inherited template author, lastModifiedBy,
  description or 2013 timestamps. Created/modified dates are supplied at save
  or use deterministic values in deterministic mode.
- Fresh presentation, master and all11 layouts use Office 16:9 geometry,
  rather than a 16:9 page with 4:3 title and body rectangles.
- Cell.text_frame and its paragraphs, fonts and alignment, reused from X194.
- Font baseline for super/subscript, character spacing, language, East Asian
  and complex-script typefaces.
- Auto-number bullet scheme and start, bullet colour, size and font.
- Linear/path gradient fills with stops/angle, pattern and picture fills,
  solid-fill alpha transparency, and gradient/picture slide backgrounds.
- ColourFormat.theme_color, brightness and type with MSO_THEME_COLOR, plus
  nested font.color.rgb, reused from the amended X197 contract.
- Six table flags first_row, last_row, first_col, last_col, horz_banding and
  vert_banding, and the table style id.
- Presentation core, application and custom properties.
- Section reading and setting.
- Media insert with poster frame, listing, extraction, replacement and removal.
- Per-slide JPEG/TIFF, notes/handout PDF, PDF/A, ODP reading and writing.
- Save/reopen, visually meaningful render evidence, exact stubs, python-pptx
  names where available, installed strict mypy and stubtest.

Google Slides compatibility remains best effort. Explicit RGB solid alpha and
common numbering schemes are preferred. Linear gradients may import, while
path gradients and patterns may degrade. Do not turn that advisory into a CI
claim. Issue295 is satisfied once through X194, not implemented again here.

## Spec reference

- `docs/hld/10-bindings-spec.md`, Python API shape and checked handle invalidation.
- `docs/hld/02-scope-and-non-goals.md`, native and Python capability rows.
- `docs/hld/04-opc-and-packaging.md`, The package and owned relationships.
- `docs/hld/08-rendering-spec.md`, Text in a shape, Tables and deterministic output.
- `docs/hld/14-development-backlog.md`, F-X205 and the full linked issue acceptance.

## Approach

Extend existing rpptx-py text.rs font property read/update functions. Native
TextRunMut.properties/set_properties are lib.rs:9369 and :9379, and
paragraph defaults have default_run_properties_mut at :9332.
CT_TextCharacterProperties at oxml-drawing/src/text/paragraph.rs:610 exposes
spacing:617, baseline:618, latin:620, east_asian:621 and complex_script:622.
Baseline currently uses Option<String>, so public numeric conversion must
validate the DrawingML percentage representation rather than pass arbitrary
strings. Spacing uses TextPointValue. Preserve scalar unit truncation and
unknown property bytes. Language is not a typed field in this structure.
Imported lang is a retained raw attribute, so writable language requires a
small typed model extension with round-trip preservation tests. This is an
actual native gap, not an ordinary wrapper.

TextBullet at oxml-drawing/src/text/bullet.rs:24 owns colour, size, font and
choice. TextAutoNumber at :369 has scheme and optional u16 start_at.
Native TextParagraph.set_bullet exists at lib.rs:9353. Expose concrete
checked fields on the existing paragraph bullet interface, retaining the
current character/none reading and picture-bullet preservation boundary.
Replacing the bullet choice must not accidentally discard separately edited
colour, size or font. Follow existing field merge semantics deliberately.

Fill at oxml-drawing/src/fill.rs:88 already has NoFill, Solid, Gradient,
Pattern and Blip. Existing binding dml.rs current_fill/write_fill and
PyFillFormat are the extension points, with native ShapeMut.set_fill at
lib.rs:7891 and SlideMut.set_background at :7017. Picture fills require a
native package operation allocating the image part and relationship for the
actual owning slide. Do not author relationship XML in Python or use an image
relationship belonging to another slide. Reuse existing MediaStore and
native image insertion mechanics where safe. Replacement/removal must prune
only unreferenced relationships and media parts.

Existing ColorFormat is dml.rs:412. X197 must own shared RGB extraction,
theme/type/brightness semantics, supported enum shims, font format sources
and preservation of unrelated transforms. Solid transparency is fill alpha,
not a second global colour conversion. Retain line and shadow-specific alpha
semantics and fore_color prerequisites. Unset getters do not solidify fills.

Native TableRef and TableMut have the six flags at lib.rs:8210 through :8250
and :8454 through :8527. Their existing Python table.rs handles can expose
the requested python-pptx aliases. CT_TableProperties already has style_id at
oxml-drawing/src/table.rs:56, but the current facade lacks a style id getter
or setter in the inspected TableRef/Mut methods. Add those minimal facade
operations and preserve unknown table property bytes and style references.
Changing banding/style formatting must not invalidate grid or cell handles.

Presentation.core_properties/core_properties_mut exist at lib.rs:1927 and
:1932. Application/custom property read/write facade operations were not
found in native rpptx. Their OPC models can be reused, but package ownership,
part creation, relationship/content-type management and staged atomicity
must first exist in the native facade. Do not expose internal package access
as Python's properties API. Use concrete snapshots/live properties consistent
with existing Word public contracts where they genuinely match.

Native sections/set_sections at :2479/:2484 and media lifecycle at :4217,
:4230, :4349, :4449 and :4475 supply checked native operations. Media insert
is owner-aware and must keep its signature, poster fallback and supported
formats explicit. Do not infer that a generic add_movie path supports every
codec or playback policy.

Native PDF/A, notes PDF and handout PDF exist at lib.rs:1292, :1302 and
:1318. Python presentation.rs:258 ALREADY exposes to_notes_pdf and :265
exposes render_all_notes. Reuse their evidence and strengthen the acceptance
tests where needed. ODP public methods exist in rpptx/src/odp.rs:71 and
:102, including typed loss/diagnostic reports. Python must retain those
reports instead of silently hiding lossy conversion. Native per-slide
JPEG/TIFF facade methods were not found, though PNG methods exist at
lib.rs:1177 and :1184. Choose a small native image encoding operation using
existing image support rather than presenting those items as wrapper-only.

## Fresh-template corrections

PyPresentation.new at presentation.rs:188 delegates directly to native
Presentation::new. Native lib.rs:818 returns from_bytes(DEFAULT_TEMPLATE).
Its file-level documentation says only slide size/type changed from the
python-pptx template. Therefore the bugs originate in native fresh-template
creation and the bundled template, not a binding-only constructor.

First build source-derived package assertions for existing template core.xml
and all master/layout placeholder rectangles. Compare supported geometry
with an authenticated Office 16:9 document, retaining named provenance and
scale units. A width-inside-slide check alone is too weak to prove Office
rectangles. Update only the fresh template or its fresh-constructor
normalization, preserving arbitrary imported metadata and geometry.

The final design must choose one explicit metadata policy. The issue permits
save-time timestamps or a deterministic constant. Avoid injecting current
time into to_bytes, which is currently the deterministic OPC writer at
lib.rs:969. A deterministic default is the smallest compatible approach,
subject to reviewing the fresh authoring API and existing metadata promise.
Fresh empty author fields versus absent fields must be stated and tested.
No overwrite of user-provided core properties at save.

This behavioural change must carry a separately labelled commit and exact
expected package/render delta. It may affect native, Python and WASM fresh
presentations. Coordinate exclusive hash baseline ownership with the root.
Do not hide it inside wrapper exposure or an asset regeneration.

## Ownership and files

X194 owns cell text frames. X197 owns shared conversion and all colour-format
and theme/brightness changes, including enum/stub migration. X206 owns alt
text, grouping, glued connectors, picture placeholders and general live-owner
handle contracts. X205 may expose new formatting only after their integrated
prerequisites and uses their revision validation unchanged.

Proposed implementation paths are existing rpptx-py text.rs, dml.rs,
table.rs, presentation.rs and slide.rs, their current handwritten stub and
enum shims, native rpptx/lib.rs and odp.rs as needed, and existing DrawingML
text/fill/table models for demonstrated gaps. The actual template asset is
crates/rpptx/assets/default.pptx. No new file or module is proposed. Claim
those paths serially with X206 and X197 because their semantic areas overlap.
Native PDF/export/model changes also need review with any active export story.

## Rejected alternatives

- Duplicate prerequisite colour/table APIs instead of reusing their actual result.
- Treat the issue's claimed native parity as proof where source inspection shows gaps.
- Drop unknown source or stale-path checks to make the missing API appear usable.

## Test plan

**Test gate**: regression.


Reproduce every missing operation against an installed Base. Record existing
to_notes_pdf as present and passing or actually defective, not a fabricated
missing-method failure. Source-build fresh documents and media controls.
Do not reuse a contributor output as the expected oracle.

Suggested tests in current test_documented_examples.py and native entrypoints:

| Test | Evidence |
|---|---|
| test_fresh_presentation_has_fresh_core_properties | Exact package metadata, user mutation and imported controls |
| test_fresh_master_and_all_layouts_use_office_widescreen_geometry | Authenticated rectangles, 16:9 slide and save/reopen |
| test_extended_run_properties_round_trip_through_python_pptx | Baseline, spacing, language and all script typefaces |
| test_numbered_bullet_attributes_preserve_unedited_choices | Scheme/start/colour/font/size, clearing and unknown controls |
| test_gradient_pattern_picture_and_alpha_fills_round_trip | Shape/slide owners, all fill choices and real rendered evidence |
| test_picture_fill_relationship_lifecycle_is_owner_scoped | Replacement/removal, shared image, cross-slide and no-orphan failures |
| test_table_style_and_six_banding_flags_round_trip | Native properties and correct conditional rendered regions |
| test_presentation_properties_and_sections_round_trip | All supported data types, invalid input and source preservation |
| test_media_insert_extract_replace_remove_preserves_poster_ownership | Actual lifecycle, parts/relationships and malformed atomicity |
| test_python_export_formats_use_native_deterministic_results | JPEG/TIFF, existing notes, handout, PDF/A and ODP reports |
| test_new_python_parity_getters_are_read_only | No property creation, transform changes or revision bump |

Reuse existing Python tests at :798, :1016 and :3870 for notes, :1494 for
bullets, :1627 for text parity, :2328 for fills/lines and :2412 for tables.
Use existing typing_smoke.py and _rpptx.pyi for installed strict typing and
stubtest. Native tests stay in lib.rs modules and current integration or
regression files. No new test binary. Add prefix-alias/default namespace,
foreign/rebound namespace, malformed bounds, exact raw sibling and twice
save/reopen controls for each promoted model field.

Visually meaningful tests render with deterministic fonts and use pinned
PowerPoint or LibreOffice evidence appropriately. A package round trip alone
does not establish render coverage. Existing notes and other exports may
reuse prerequisite regression evidence, but all acceptance items still need
an explicit capability/evidence mapping. PowerPoint open-without-repair is a
native application check, not a claim from python-pptx parsing.

## Risk routing

Matched rows: unit conversion, theme/colour transforms, parser/serializer,
layout/rendering, published native APIs, PyO3 bindings, dependency edges,
external oracles and packaged assets. Read HLD01, HLD03, HLD04, HLD05,
HLD06, HLD07, HLD08, HLD10 and HLD15, plus pinned differential testing.
Do not change the legacy Word tint implementation. Respect schema child
order, exact unsupported subtree preservation, real archive measurement and
size checks, WASM compilation and documented workspace binding exclusions.
No new feature flag is justified. Normal scoped gate and independent
microscope remain required.

Wrapper-only additions expect an unchanged default hash harness. The fresh
template correction is an intentional default behaviour delta and needs an
approved expected package/render change, a separate labelled commit and
exclusive baseline ownership. No automatic re-recording is authorized by
this proposal. Reconcile integrated archives from actual bytes, not tuples
copied from another worker.

## HLD impact

- docs/hld/02-scope-and-non-goals.md, accurate Python parity/export rows.
- docs/hld/05-drawingml-model.md, language and any new typed fill/colour contracts.
- docs/hld/06-presentationml-model.md, fresh metadata/template, table style and property contracts.
- docs/hld/07-inheritance-and-resolution.md, template geometry/default resolution changes if needed.
- docs/hld/08-rendering-spec.md, deterministic export contracts and actual supported output.
- docs/hld/10-bindings-spec.md, exact public signatures, return reports and handle behaviour.
- docs/hld/14-development-backlog.md, complete acceptance and prerequisites.
- docs/hld/15-build-and-toolchain.md, actual template provenance and packaged asset changes.

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

1. Choose exact Python bullet/fill/property/media/export signatures and enum
   values, using python-pptx names where present and concrete current types.
2. Select deterministic fresh metadata values without changing imported or
   user-written fields, and obtain authenticated Office geometry provenance.
3. Confirm missing native application/custom properties and JPEG/TIFF API
   inventory, then define the smallest checked facade operations.
4. Define ODP read/write return types that retain diagnostics and loss facts.
5. Clarify solid transparency ownership versus shared colour alpha transforms
   with X197, including theme brightness editing and preserved metadata.
6. Determine which new native render semantics, rather than mere exposure,
   are required after source-built regressions for language and gradients.

Resolve these through design and source evidence. Do not silently reduce the
issue acceptance or request approval for already authorized nested colours.
