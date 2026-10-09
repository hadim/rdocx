# F-X208, Author PowerPoint footer fields themes and transitions

**Status**: draft
**Sprint**: S91
**Size**: L
**Depends on**: F-X197

Technical design is not approved for implementation. Issue311 is
owned here. Issue312 theme editing remains F-X209 and shared one-shot CLI
commands remain F-X204. F-X197 is the approved colour-format prerequisite.
Do not reinterpret PR315 caller-font metadata, font ownership or rendering
claims as footer evidence. Its intake evidence remains unchanged.

## Problem

- Issue311 evidence is `.claude/scratch/S91-intake-issues.json`, updated
  2026-10-09T18:42:46Z. Acceptance requires the default template and a branded
  potx, one Python call and one CLI call producing slide numbers plus footer
  with title-slide suppression, correct PDF output and native PowerPoint
  opening without repair. It also requires date modes, placeholder format,
  theme reading and the complete requested common transition set.
- `crates/rpptx/src/lib.rs:3253` creates slides. At 3272 it unconditionally
  excludes dt/ftr/sldNum and only copies type/index for other placeholders.
  A hf-aware route must clone sufficient existing placeholder geometry,
  properties and text formatting, preserve idx inheritance and allocate
  fresh slide-local shape IDs. Copying foreign relationship-bearing content
  requires owned dependency remapping or explicit atomic refusal.
- `crates/rpptx-layout/src/context.rs:286` and 291 derive independent master
  and layout latent policies. At 3670 an inherited placeholder is emitted
  only if its source hf permits it, it is occupied, and no deeper matching
  placeholder already wins. This is the actual present renderer behavior,
  not a confirmed PowerPoint rule. HLD06:1101 presently requires these latent
  placeholders to be inherited and never cloned. This section must change
  only after actual native comparison.
- `crates/rpptx-layout/src/context.rs:3815` defines occupancy from nonempty
  plain text. An uncached slidenum latent field may be filtered here before
  `crates/rpptx-render/src/text.rs:156` can substitute page_number. That
  renderer already substitutes slidenum regardless of cached text, so a
  duplicate field evaluator is unnecessary. Inspect inherited masking and
  duplicate-selection occupancy together with the visibility fix.
- `crates/rpptx-oxml/src/presentation.rs:152` does not type firstSlideNum.
  Root attributes remain opaque. Add presence-sensitive typed handling at the
  model boundary, and pass the displayed number separately from physical
  page identity to static and timeline render paths. Zero is a supported
  input. Do not change physical page numbering, selection, or output naming.
- `crates/rpptx-oxml/src/slide_parts.rs:201` CT_Slide has no p:hf. Layout and
  master roots have it. A per-slide public header/footer state is derived
  from slide-owned placeholders and fields. Do not serialize an invalid
  slide-root p:hf. Native capture must establish which legal master/layout
  settings the dialog writes for deck-wide and per-slide operations.
- `crates/oxml-drawing/src/text/paragraph.rs:1359` already models a:fld,
  GUID/type/properties/optional text and opaque children. Add checked field
  construction in this existing module and facade/Python paragraph routes.
  Allocate GUIDs uniquely and deterministically under the existing identity
  convention. Preserve run formatting and paragraph schema child order.
- `crates/rpptx-oxml/src/timing.rs:1175` has typed Cut/Fade/Wipe/Push/Zoom/Morph
  but Split/Cover/Uncover are Other. CT_SlideTransition at1198 retains raw XML,
  and to_xml at1316 returns raw_xml, so changing public fields alone will NOT
  persist transition edits. Existing set_speed and morph editing demonstrate
  retained-XML patching. Author canonical supported transitions with checked
  properties, and preserve unrelated children and extensions when editing.

## Mandatory native oracle before renderer changes

Microsoft PowerPoint was located through read-only CUA getApp. It is at its
Home/open-recent chooser, with no deck opened by this task. No menu, document
or UI mutation was performed. Pin the app version/build before capture.

Prepare finite synthetic decks outside tracked source and record hashes and
full package provenance. Use footer/date/number strings unique by owning
master/layout/slide and identical geometry so native PDF text and raster
visibility distinguish ownership. Use a normal and a title layout.

1. No slide-owned latent placeholders, occupied master placeholders only.
   Compare absent hf, empty hf and each explicit enabled/disabled flag.
2. Occupied layout placeholders only with the same hf presence/flag matrix.
   Include conflicting master/layout flags and layout showMasterSp false.
3. Both master and layout placeholders with distinguishable strings and
   matching versus distinct idx. Include empty and uncached-field variants.
4. Add slide-owned placeholders with master/layout flags disabled, cached and
   uncached slidenum, and firstSlideNum absent, zero and a nondefault positive
   number across at least two slides.
5. Native new-slide action on templates with each relevant hf state. Inspect
   generated slide XML, copied fields, cached text, IDs and placeholder idx.
6. Native Header and Footer dialog Apply and Apply to All, with footer/date
   fixed/automatic/off and title suppression. Save and reopen normally, then
   export native PDF. Inspect legal master/layout flag locations, per-slide
   ownership, generated datetime type and title identification.

Freeze before/native-saved/reopened outputs and actual action receipts. Compare
native PDF text and page rasters with deterministic rpptx renders of the same
immutable deck. Opening alone is not rendering evidence. A save can rewrite
or materialize placeholders, so distinguish original versus native-saved
package behavior. No hf-only conclusion or renderer baseline delta is approved
until these controls establish the actual native rule.

## Spec reference

- `docs/hld/06-presentationml-model.md`, Adding a slide, Placeholders and Public facade.
- `docs/hld/08-rendering-spec.md`, Text in a shape and displayed field values.
- `docs/hld/10-bindings-spec.md`, Python API shape, Native PowerPoint text layout and CLIs.
- `docs/hld/02-scope-and-non-goals.md`, explicit transition-authoring boundary.
- `docs/hld/14-development-backlog.md`, F-X208 and Issue311 acceptance.

## Approach

Use one concrete header/footer options value with slide number bool, footer
optional text, date off/fixed/automatic-format and skip-title bool. Native
Presentation and checked SlideMut methods stage all selected slides and
required legal package edits, validate identities/fields and publish once.
The Python `prs.set_header_footer(...)` and slide.header_footer interface use
those same operations. Validate wrong types, unsupported date formats and
invalid XML text before mutable access. Public applied state must distinguish
absence from false, fixed from automatic and local override from inheritance.

Title suppression should use the native-confirmed layout/placeholder rule,
not slide index zero or a display-name string assumption. Repeated calls must
be idempotent, preserve unrelated slide-owned shapes and avoid duplicate
latent placeholders. Off removes only the identified owned footer field
placeholders, never an ordinary textbox with matching text. A template with
missing footer geometry needs an explicit documented synthesis or atomic
refusal contract, decided from the default/branded acceptance captures.

Python shape.is_placeholder and placeholder_format.type/idx should reuse
existing ShapeRef/placeholder collection identity and stale-handle rules.
PP_PLACEHOLDER follows existing python-pptx names and integer values.

Theme reading resolves master through presentation relationships, then theme
through that master's relationship, with multi-master tests. Expose name,
twelve resolved RGB scheme slots and major/minor latin/EA/CS fonts. Reuse the
amended F-X197 RGBColor/ColorFormat and theme vocabulary. Reserve owner identity
by master part, not a detached first-theme cache or slide index. X209 later
adds mutation and apply_theme, including theme write-back and lifecycle.
Do not create a second incompatible theme proxy that X209 must replace.

Transitions use none/fade/push/wipe/split/cover/uncover/cut/zoom. None removes
the selected transition, not all animation timing. Validate effect-specific
direction vocabularies, finite nonnegative durations and advance-after values,
with checked conversion to milliseconds. Preserve presence-sensitive click
advancement. Apply-to-all stages every selected slide and rolls back if any
refuses. Unknown existing effects remain preserved and readable with explicit
unsupported metadata. Add typed model/render support for the requested missing
effects rather than pretending Other is implemented. Static output should
remain unchanged except reviewed footer/number fixes.

Footer CLI owns `rpptx footer FILE --slide-number --footer TEXT --date
auto|TEXT|off --skip-title` plus established safe output selection and refusal
of input overwrite. All operations must be Python-visible. X208 also extends
existing inspect text/JSON with masters, layouts by master, theme colours/fonts
and hf presence/flags. Keep existing metadata and slide details compatible.
F-X204 owns other general CLI commands and consumes this surface.

## Rejected alternatives

- Write p:hf on the slide root, which has no such legal modeled element.
- Add another slidenum evaluator when the existing evaluator already works.
- Infer PowerPoint inheritance from LibreOffice behavior without native capture.

## Test plan

**Test gate**: regression.


Before implementation, fresh installed public Base regressions for every
missing Python operation and native source-built regression for each reported
rendering defect. Reuse existing test modules and entrypoints, no new binary.

Required matrix includes default/branded potx, multiple masters, title/normal
layouts, missing/empty/conflicting hf, uncached/cached field values, index-zero
displayed numbering, repeated calls, per-slide overrides and deck-wide atomic
failure. Save/reopen must retain qualified names, raw producer children,
relationships, GUID uniqueness and placeholder idx. Compare field text and
PDF/PNG output under deterministic fonts with pinned PowerPoint captures.
Transitions require every effect/direction, boundaries, malformed input,
all-slide rollback, retained unrelated timing and supported timeline samples.
Python stubs, strict mypy and stubtest cover all types and held/stale handles.
CLI help/reference, footer and inspect compatibility tests use existing
integration entrypoint. Google Slides import is best effort, with honest
notes on automatic dates and hf-only compatibility, not an inferred CI pass.

## HLD impact

- `docs/hld/02-scope-and-non-goals.md`
- `docs/hld/05-drawingml-model.md`
- `docs/hld/06-presentationml-model.md`
- `docs/hld/08-rendering-spec.md`
- `docs/hld/10-bindings-spec.md`
- `docs/hld/12-testing-strategy.md`
- `docs/hld/14-development-backlog.md`

HLD sections: 02 specific transition-authoring non-goal, 05 DrawingML field
and theme API, 06 facade/footer rules and transition persistence, 08 displayed
number/latent visibility and timeline effects, 10 Rust/Python/CLI surfaces,
12 exact oracle matrix and 14 complete story contract. Review each section
against the final planned diff before approving this list.

## Risk routing

Matched rows: parser/serializer HLD04+06, raw subtree byte preservation and schema
ordering. Rendering HLD08 deterministic fonts and explicitly declared harness
delta after native proof. Public API HLD10 semver, full local patched package
dry run and archive limits. PyO3 and both WASM targets. Units HLD01 existing
truncation conventions for transition milliseconds, no incidental rounding.
External oracle skill requires pinned PowerPoint version/build and truthful
capture receipts. No new crate, module, source file or speculative abstraction.

## Hash harness

Existing static transition output should remain unchanged. Footer occupancy
and displayed-number fixes can change deterministic output only after native
proof and explicit reviewed delta declaration. No baseline move is approved
by this draft.

## Implementation checklist

- [ ] Capture and pin native PowerPoint footer/new-slide behavior.
- [ ] Resolve legal owner state, title suppression and theme proxy contracts.
- [ ] Approve exact source claims, primitives and expected output changes.
- [ ] Reproduce public defects on the claimed Base before implementation.
- [ ] Implement full Issue311 native/Python/CLI acceptance and existing tests.
- [ ] Run scoped verification and native oracle/risk riders.
- [ ] Obtain clean independent review and prepare validated handoff.

## Open questions

Native hf-only/new-slide and dialog persistence rules are genuine remaining
evidence gaps, not user preference questions. Resolve with captures first.
Then specify title suppression and missing-placeholder synthesis, legal flag
owner updates, displayed-number plumbing and supported automatic date cache
context. Synchronize X209 theme-owner/proxy design before source claims and
X204 CLI command ownership before shared main.rs/commands.rs writes.

## Actual native design evidence

PowerPoint16.113.3 build16.113.26092714 produced the controlled blank
16:9 baseline and authenticated Apply/ApplyAll captures in
/private/tmp/S91-X208-native-footer-controls, with input/output hashes and
owner facts. Apply on the title slide with suppression false creates three
slide-owned placeholders. Fixed dates are ordinary text, slide numbers are
a:fld type slidenum. Setting start0 writes presentation firstSlideNum=0 and
the visible first-slide field0. A newly added slide after Apply only has no
footer/date/number placeholders.

ApplyAll with automatic date, footer and numbers plus title suppression
removes the title slide placeholders, writes all three on the ordinary slide,
updates master/layout placeholder content and stores legal master hf hdr=0.
Neither slide has a root p:hf. Automatic date uses a:fld type datetime1 in
this native locale control. Normal same-path reopen/Save preserves all14
slide/layout/master XML parts byte-for-byte and firstSlideNum0, with no
repair dialog. The ordinary slide visibly shows date, footer and number1.

A source-built comparison removes only the three ordinary-slide owned
placeholders, retaining exact master/layout inputs. Native normal open shows
none of the three footer fields and Save does not materialize them. This
contradicts unconditional latent inheritance for this concrete control.
Use explicit owned-placeholder authoring and a separately reviewed renderer
correction, with intentional hash deltas declared before implementation.
This evidence is one blank-theme control, not the complete hf matrix,
branded potx or native PDF acceptance. Keep those remaining gates.
