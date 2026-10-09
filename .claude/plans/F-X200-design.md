# F-X200, Complete Python Word formatting and list parity

**Status**: draft
**Sprint**: S91
**Size**: L
**Depends on**: F-X197
**Issue**: 303, captured in S91-intake-issues.json on 2026-10-09
**Prerequisite**: F-X197
**Implementation**: not claimed, no source changes or new gate evidence

## Problem

Issue303 requests installed Python parity for everyday Word formatting that
mostly exists in the native facade. The complete acceptance inventory is:

- Run.font superscript, subscript, small_caps, all_caps, character spacing,
  double_strike and hidden.
- Font.language, East Asian and complex-script font names, and rtl.
- Paragraph.paragraph_format.tab_stops with add, clear and ordered listing,
  including alignment and leader.
- Paragraph borders top, bottom, left, right and between, and shading.
- Paragraph outline level and bidi.
- add_style and set_style alignment, line spacing, underline, keep-with-next,
  keep-together, page-break-before, borders, shading and tab stops, extending
  existing font, size, bold, italic, colour, spacing and indent arguments.
- Document default font and size in w:docDefaults, without restyling Normal.
- add_bullet_list_item and add_numbered_list_item with level.
- List restart and start overrides through a new instance of one definition.
- Checklist glyph presets for unchecked and checked bullets.
- Both bindings' shared colour input and nested font.color format contract,
  delivered once by F-X197, including one conversion in oxml-py-support.
- Getter and setter save/reopen parity, meaningful render/PDF evidence,
  handwritten stubs, strict mypy and stubtest, python-docx names where they
  exist, and accurate HLD02 Python capability rows.

Google Docs import is best effort. Author w:vertAlign, real w:tabs, plain
w:numPr numbering and supported paragraph borders. RGB assignment writes a
literal colour. Do not invent a theme-only colour from RGB. Issues264 and281
remain excluded. The prerequisite may reuse their existing native work.

## Spec reference

- `docs/hld/10-bindings-spec.md`, Python API shape and checked handle invalidation.
- `docs/hld/02-scope-and-non-goals.md`, native and Python capability rows.
- `docs/hld/04-opc-and-packaging.md`, The package and owned relationships.
- `docs/hld/08-rendering-spec.md`, Text in a shape, Tables and deterministic output.
- `docs/hld/14-development-backlog.md`, F-X200 and the full linked issue acceptance.

## Approach

Extend FontSnapshot, FontUpdate and PyFont in
crates/rdocx-py/src/formatting.rs:62, :103 and :126. Preserve its existing
AcceptedRunPath and ParagraphLocation resolution through run_snapshot:178
and apply_run_update:205. Adding one run property must work in accepted
control, table and body runs, not just direct paragraph children.

Native Run setters already cover language at run.rs:727, script font slots
at :685, language slots at :1013 and :1021, rtl at :985, character spacing
at :909, super/subscript at :887 and :898, and caps, strike and hidden at
:854 through :942. RunRef has corresponding property projections for many
fields, including vert_align:1362 and slot_font:1378. Some builder-era
methods accept only bool or unconditionally set a value. Add the smallest
Option setter/getter pairs needed for direct inheritance-aware Python
properties. Do not turn None into false. Validate spacing range before
conversion and preserve canonical truncation toward zero.

Extend ParagraphSnapshot, ParagraphUpdate and PyParagraphFormat in the same
binding file at :378, :432 and :457. Native Paragraph already has
set_border_value at paragraph.rs:1992, set_shading_value:2052, tab_stop:2067,
set_tab_stop:2081, remove_tab_stop:2108, clear_tab_stops:2128 and
set_outline_level_value:2240. ParagraphRef exposes border:3112 and bidi:3185.
Use those checked paths. Any missing immutable projection or clear operation
is a concrete native gap to fill, not permission to bypass the facade.

Styles are configured in document.rs:2751 and :2871. Native StyleBuilder
paragraph_properties at style.rs:415 and run_properties:463 accept the
existing concrete CT_PPr and CT_RPr. Stage a single checked property update
per style operation, preserving imported unsupported attributes and children.
Do not add a generic property application framework or many forwarding
wrappers. Validate the entire input before creating or mutating the style.

NumberingLevelOverride already exists at native document.rs:29971 with
level, start and optional replacement. add_numbering_instance:22207 stages
and validates its override slice. Python document.rs:2977 currently always
passes an empty slice, so restart/start is an exposure gap. ListLevel and
existing PyListLevel supply checklist glyph configuration. The native
one-call list helpers exist at :21809 and :21868, but their allocation paths
contain expect and the level is not validated before allocation. Check the
requested level before invoking them or provide a checked native operation
if required for imported malformed graphs.

No public native Document default-font/default-size operation was found.
doc_defaults currently appears in style resolution, including style.rs:977
and :1011. Add a small checked native read/update projection over the
existing defaults model. It must preserve paragraph defaults, unrelated
run defaults and unknown XML. Normal and explicitly configured styles remain
unchanged. This item is not merely a binding wrapper.

## Proposed Python contracts to settle in design

Use inheritance-aware Optional properties for direct font booleans and
language/font-name slots. superscript and subscript share one w:vertAlign
source, so setting one true replaces the other. Setting one false must not
clear a different active vertical mode. None clears direct formatting.
Character spacing should accept documented Length and return Length,
without an undocumented point or EMU reinterpretation.

Expose a live tab_stops collection on paragraph_format. Follow the requested
python-docx spelling add_tab_stop(position, alignment, leader). Iteration
returns typed positions, alignment and leader, ordered as the actual source
requires. The draft does not prescribe a new file or generic collection.
A new concrete collection class inside existing formatting.rs is justified
by the named public API, but its exact mutation invalidation needs review.

Border and shading methods should reuse current table/cell naming where
python-docx has no public counterpart. Their colour arguments use F-X197's
one shared validator. Preserve auto only where the existing native contract
supports it. Unset getters do not create empty property nodes.

Extend add_numbering_instance with explicit overrides and a convenience
start/level form only if it reduces actual caller work. Reject conflicting
forms rather than guessing. The final design must state its exact signature.
Checklist presets produce real bullet definitions using the supplied glyph,
not ordinary text characters preceding the paragraph.

## Shared ownership and exclusive paths

F-X197 owns colour extraction, RGBColor behaviour, colour formats, theme and
brightness semantics, their stubs and colour contract documentation. X200
only consumes the integrated shared implementation for its newly exposed
paragraph/style colours. It does not duplicate an extractor.

Proposed exclusive implementation paths are existing rdocx-py formatting.rs,
document.rs and current handwritten stubs/enums, plus native run.rs,
paragraph.rs, style.rs and document.rs only for demonstrated missing checked
operations. Existing Word test_formatting_tables.py, test_core.py and typing
entrypoints supply the regression locations. Other stories use native
document.rs, so ownership must be claimed and edits serialized or integrated
before this worker starts. No tracked file is created by this proposal.

## Rejected alternatives

- Duplicate prerequisite colour/table APIs instead of reusing their actual result.
- Treat the issue's claimed native parity as proof where source inspection shows gaps.
- Drop unknown source or stale-path checks to make the missing API appear usable.

## Test plan

**Test gate**: regression.


Start with an installed Base reproduction matrix. Each missing property,
helper and style argument must fail through the installed public API before
production changes. Distinguish AttributeError, rejected keyword and actual
incorrect output. Reuse F-X197 evidence for nested colours.

Suggested tests in existing entrypoints:

| Test | Evidence |
|---|---|
| test_word_extended_font_properties_round_trip | Every font property, true/false/None, mutually exclusive vertical modes and script slots |
| test_extended_font_properties_cover_accepted_nested_runs | Existing accepted body/control/table run paths and stale controls |
| test_paragraph_tabs_borders_shading_and_bidi_round_trip | All requested edges, tabs, alignment/leader and scalar getter facts |
| test_tab_stops_preserve_unknown_xml_and_fail_atomically | Foreign siblings, clearing, invalid position/enum and unchanged revision |
| test_extended_style_keywords_preserve_existing_style_graph | Create/update, inheritance, missing values and rejected input atomicity |
| test_document_defaults_do_not_restyle_normal | Defaults round trip and unchanged explicit style properties |
| test_one_call_lists_and_checklist_presets_round_trip | Level checks, glyphs, actual numPr and definition/instance ownership |
| test_numbering_start_overrides_reuse_one_definition | Correct lvlOverride/startOverride, save/reopen and clear/update controls |
| test_plain_numbering_instances_match_authenticated_word | Separate native oracle experiment before any continuation change |
| test_extended_python_word_formatting_reaches_render_and_pdf | Deterministic rendering of the visually meaningful requested features |

Extend tests already present at test_formatting_tables.py:88, :639, :793,
:904, :926 and :970. Native tests stay in existing run/paragraph/style
modules and existing integration or regression entrypoints. No new test
binary. Compare supported public facts with pinned python-docx, then inspect
package bytes for non-public properties. Save/reopen twice and assert exact
unsupported XML bytes remain unchanged. Render tests use deterministic fonts.
Add accurate stub types and installed strict mypy/stubtest evidence for new
properties, collections and arguments.

## Risk routing

Read HLD01 units and CLAUDE truncation rules for spacing/tabs. Read HLD04 and
HLD06 for parser/serializer ordering and prefix/raw preservation. Read HLD07
inheritance and resolution, HLD08 deterministic rendering, HLD10
bindings and the invalidation contract, and the pinned differential oracle
skill. Native published API changes earn archive dry-run, size and actual
measurement maintenance. Binding changes earn the documented WASM check and
workspace binding exclusions. Normal scoped verification and independent
microscope remain mandatory.

Expected default hash harness delta is none for wrapper exposure. Newly
authored requested properties change only the new source-built test outputs.
Do not move the baseline. If the authenticated continuation experiment proves
a native rendering defect, propose its separate labelled behaviour commit
with the exact expected delta and exclusive baseline ownership before fixing
it. Tests and claims may not treat an unconfirmed Word assertion as evidence.

## HLD impact

- docs/hld/02-scope-and-non-goals.md, Python capability rows.
- docs/hld/07-inheritance-and-resolution.md, defaults and list resolution contracts if expanded.
- docs/hld/10-bindings-spec.md, exact font/paragraph/style/default/list signatures and invalidation.
- docs/hld/14-development-backlog.md, resolved acceptance and retained boundaries.

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

1. Confirm Word plain-instance continuation with authenticated pinned Word,
   including restart overrides and multilevel interruption controls.
2. Finalize document-default public names and the minimal checked native
   projection. Do not expose raw CT_RPr in Python.
3. Finalize tab/border/shading getter and mutation types and exact style
   keyword names without proliferating unrelated types or enum files.
4. Confirm Python superscript/subscript false and None behaviour against
   pinned python-docx, and preserve direct versus inherited facts.
5. Decide whether a checked list-helper native counterpart is needed for
   malformed imported numbering, instead of allowing expect to surface.

No question above is automatically a user approval gate. Resolve routine
technical choices through evidence, then review the complete draft against
the issue before implementation.
