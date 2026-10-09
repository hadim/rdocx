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

The authenticated Word continuation/restart/multilevel oracle is complete.
The remaining decision is how the requested parity supersedes the explicit
independent-numId rendering contract, including bounded missing controls,
actual fixture deltas and the F-283 dependency. Technical signatures, vertical
mode tri-state behavior, document defaults, tabs and checked list helpers
are resolved below. The plan remains draft until final contract review.

## Resolved technical decisions

## Oracle provenance

The requested /private/tmp/s77-fx141-venv/bin/python executed Python 3.12.13, but its sys.path omitted that venv's site-packages and import docx failed. Used the already installed /private/tmp/S91-F-X198-python/bin/python instead, asserted docx.__version__ == '1.2.0', inspected its exact installed source and executed 24 super/subscript transition cases. No substitute Word oracle was used.

Pinned oracle source root for citations below: /private/tmp/S91-F-X198-python/lib/python3.12/site-packages/docx/. Native repository citations refer to the source checkpoint above.

### Decision 1: vertical modes and direct properties

Match python-docx Font.superscript and Font.subscript names and Optional[bool] facts. Oracle text/font.py:346 and :364 delegate to oxml/text/font.py:241 and :263.

| Assignment | Existing direct mode | Resulting direct w:vertAlign |
|---|---|---|
| superscript=True | any | superscript |
| subscript=True | any | subscript |
| superscript=False | superscript | absent |
| superscript=False | subscript, baseline, absent | unchanged |
| subscript=False | subscript | absent |
| subscript=False | superscript, baseline, absent | unchanged |
| either=None | any | absent |

Getters return None if vertAlign absent, otherwise a boolean comparison with the corresponding token. Thus direct baseline yields False for both. Never report effective inherited formatting as a direct value. The executed 24-case matrix confirmed every row above. Preserve repository read/clear no-op discipline: unlike python-docx's get_or_add_rPr setter, clearing absent direct state must not insert empty nodes.

Existing native Run::set_superscript and set_subscript at run.rs:887/:898 support only True. Add minimal Option<bool> value setters with the matrix above. RunRef::vert_align at :1362 supplies getter facts. Preserve unrelated raised/lowered position rather than conflating w:position with the requested w:vertAlign.

Add inheritance-aware Optional booleans for small_caps, all_caps, double_strike and hidden, using existing toggle pattern. Existing bool-only methods at run.rs:854, :865, :876 and :942 need Option value counterparts. rtl already has set_rtl_value :985. None clears, False writes explicit false for independent toggles.

Exact proposed additional Font names: character_spacing, language, language_east_asia, language_bidi, name_east_asia, name_complex_script, rtl. Length/integer EMU character spacing returns Length|None. Check signed i32 twips before conversion and use integer division toward zero. Native Run::set_character_spacing :909 currently narrows through as_twips and lacks clear. add a checked optional operation. Script names use set_slot_font at :685, languages use :727/:1013/:1021. retain slot theme clearing only for the changed slot.

Correction to issue narration: native Run::set_font_value at :666 sets all four slots, and existing Python style font_name writes all four at rdocx-py/src/document.rs:1003. Do not silently change those established general-name semantics. The new slot properties allow independently changing East Asian/complex-script names.

### Decision 2: document defaults

Use Document.default_font_name -> str|None and Document.default_font_size -> Length|None, both writable with None clearing their own direct facts. No python-docx public document-default equivalent exists, so these are repository naming decisions consistent with existing style font_name/font_size and Font.size. Do not expose CT_RPr in Python.

Add four concrete native operations in existing document.rs:

- default_font_name(&self) -> Option<&str>
- set_default_font_name_value(&mut self, value: Option<&str>) -> Result<()>
- default_font_size(&self) -> Option<Length>
- set_default_font_size_value(&mut self, value: Option<Length>) -> Result<()>

Read the explicit ASCII slot and sz from self.styles.doc_defaults.rpr. General font assignment follows the existing four-slot general setter semantics, clearing corresponding theme references, preserving font hint and all nonfont fields. Size assignment writes normal and complex-script sizes as the current general font size setter does. Validate schema-representable nonnegative half-points before any mutation. Absent getters and absent None clears create nothing. Stage updates using existing Document candidate/style-part reservation, commit only after success, and invalidate layout without bumping paragraph topology. Normal and named styles are not restyled.

Concrete preservation rider: CT_DocDefaults at rdocx-oxml/src/styles.rs:564 only holds rpr/ppr. Its parser skips unknown docDefaults children at :593 and rPrDefault wrapper children at :623. Therefore a setter over the current model cannot honestly promise preservation of unknown wrapper source. Extend that existing model/parser/writer minimally to retain source attributes and unknown child slots in those exact wrappers, reusing existing raw/order facilities. This is a demonstrated existing-model gap, not a new generic XML mutator or file. Add its existing styles.rs tests to the exclusive path/risk list. Native CT_RPr/CT_PPr unknown fields must also remain untouched. Authenticate malformed namespace controls and empty/default clearing serialization rather than claim preservation from fields alone.

### Decision 3: paragraph tabs, borders, shading and style keywords

#### Tabs

Use paragraph_format.tab_stops as a live collection over the existing paragraph ContentPath/AcceptedRunPath resolution, implemented in existing formatting.rs. Read access creates no pPr or tabs. Exact requested public operations:

- add_tab_stop(position: Length|int, alignment=WD_TAB_ALIGNMENT.LEFT, leader=WD_TAB_LEADER.SPACES) -> TabStop
- clear_all() -> None
- len, iteration, integer indexing including negative indices, del collection[index]
- Returned TabStop exposes position: Length, alignment: WD_TAB_ALIGNMENT, leader: WD_TAB_LEADER as read-only owned facts.

The last choice deliberately limits the projection to the issue's add/clear/list acceptance. It does not claim python-docx's additional writable TabStop.position/alignment/leader API. A frozen three-field snapshot in existing formatting.rs has concrete consumers: add result, iteration and indexed inspection. Consequently no live indexed child can silently alias another tab after reorder, no tab revision framework is needed, and collection operations leave parent paragraph identity live. Document this subset in stubs/HLD. If mutable individual tab setters are added to scope later, design their identity/invalidation explicitly first.

Oracle text/tabstops.py:53 defines add signature and permits negative positions. :67 defines clear_all. :31 indexing. :20 deletion. :43 iteration uses source XML order. Oracle oxml/text/parfmt.py:383 inserts new tabs before the first greater position, preserving equal-position insertion order. Oracle TabStop.leader at text/tabstops.py:105 accepts None as SPACES. Model absent leader reads SPACES, preserving source absence on a read. Native TabStopRef::leader at paragraph.rs:642 conflates missing and unsupported tokens, so add a concrete source projection or complete its modeled variants before applying the default. Add/None leader writes the repository equivalent of the oracle default. Add WD_TAB_ALIGNMENT/WD_TAB_LEADER to existing enum/text.py, not new files.

Native paragraph.rs:1645/:1670 currently append, while :2081 rewrites in place without reordering. Provide a checked native insertion path for this named Python add consumer that inserts by position and validates signed twips before narrowing. Retain imported source order and unknown alignment/leader tokens instead of sorting the entire existing list. Extend existing native TabAlignment/TabLeader projections to expose the already modeled Bar/Clear/Num and Heavy/MiddleDot variants rather than misreport them as absent. The pinned oracle also exports LIST=list, END=end and START=start. Existing low-level ST_TabJc at rdocx-oxml/src/shared.rs:432 rejects list and normalizes start/end to left/right at :446/:448. Add concrete List/Start/End variants to that existing enum/parser/writer and the facade to expose and preserve the exact requested oracle vocabulary. No new enum file is needed. Any genuinely unsupported token must remain preserved and reads must report an explicit unsupported-value error rather than guess. Use native removal :2108 and clear :2128. Source-owned snapshots remain readable after later mutations, while the live collection validates its parent path on every operation.

#### Borders and shading

Follow existing table/cell names where python-docx has no equivalent:

- paragraph_format.set_border(edge, style, *, size, color)
- paragraph_format.border(edge) -> tuple[str, int|None, RGBColor|str|None]|None
- paragraph_format.clear_border(edge), clear_borders()
- paragraph_format.shading -> RGBColor|'auto'|None, writable RGBColor|str|None
- paragraph_format.outline_level -> int|None, writable checked 0..9|None
- paragraph_format.rtl -> bool|None

Edges: top, bottom, left, right, between, bar, using actual ParagraphBorderEdge slots. size is eighths of a point, matching current table/cell spelling at rdocx-py/src/table.rs:454/:1200. Colour uses integrated F-X197 shared extraction and surface-specific auto handling. Border tuple comes from native ParagraphBorderRef :229, including style/size/color. do not silently replace unsupported existing style values. Preserve space and foreign border attributes when changing the named fields. Clear removes one edge, and last-edge clear removes pBdr, using paragraph.rs:1992.

Shading is the direct fill projection at ParagraphRef::shading_fill :2960. A fill edit preserves imported pattern, foreground and unknown shading attributes. Existing Paragraph::set_shading_value :2052 replaces the entire shading struct, so add a narrow native fill-value update rather than reuse that destructive operation. None removes direct shading, absent clear is no-op. Pattern editing is not required by this acceptance and no speculative shading class is needed.

### Styles

Extend both existing add_style/set_style keyword sets with alignment, line_spacing, underline, keep_with_next, keep_together, page_break_before, borders, shading, tab_stops. Existing font/spacing/indent names remain unchanged. Exact input shapes:

- alignment: WD_ALIGN_PARAGRAPH|None
- line_spacing: Length|float|None, using the existing ParagraphFormat spacing interpretation
- underline: bool|WD_UNDERLINE|None, using existing Font underline conversion
- keep_with_next, keep_together, page_break_before: bool|None
- borders: Mapping[str, tuple[str,int,RGBColor|str]]|None
- shading: RGBColor|str|None
- tab_stops: Sequence[tuple[Length|int,WD_TAB_ALIGNMENT,WD_TAB_LEADER|None]]|None

Retain established keyword omission semantics: None for these style convenience kwargs means no supplied override, False is explicit false. Empty borders/tab_stops are explicit clear-all inputs. Do not introduce a public UNSET object merely to change old None behavior. If explicit clearing of every scalar style property becomes required, use an explicit checked clear operation as a separately documented addition rather than overloading omission.

Validate the whole input before style creation/update. Copy existing relevant properties, apply only named supplied changes, then submit one existing StyleBuilder update. Native style.rs:415 and :463 already accept concrete property structs. current Python StyleFormatting document.rs:981 constructs new structs, so replacing properties blindly would drop imported state. Preserve unrelated children, attributes, schema order, aliases and run/paragraph defaults. Character styles reject supplied paragraph-only inputs before graph mutation. No new style-property framework is needed.

### Decision 4: explicit numbering restart and checked list helpers

Use the smallest required public signature:

add_numbering_instance(definition_id: int, *, start: int|None=None, level: int=0) -> int

When start supplied, construct one existing native NumberingLevelOverride::new(level).start(start), then call the existing staged Document::add_numbering_instance at document.rs:22207. When start omitted, pass the existing empty override slice and do not change continuation behavior. Reject nonzero level without start as meaningless, invalid level outside 0..8, negative or unrepresentable start (zero remains valid in the existing native u32 contract), missing definition and malformed numbering graph before source commit. No new public override type, parallel overrides/start forms or convenience framework is needed for Issue303's restart/start acceptance. Full replacement-level editing belongs the separately scoped numbering inspection work.

Expose add_bullet_list_item(text, level=0) and add_numbered_list_item(text, level=0), returning the new Paragraph at the current revision. A checked native counterpart IS required, not just a Python level check. Native document.rs:21809/:21868 call fallible reserve_numbering_bundle and reserve_numbering_ids through expect. reserve_numbering_bundle at :21713 observes an imported package graph and can fail. Level validation cannot prevent those failures.

Add try_add_bullet_list_item and try_add_numbered_list_item returning Result<Paragraph<'_>> in existing document.rs. Preserve successful existing output. Convert existing staged expect paths to propagated errors and validate 0..8 before allocation. The existing infallible helpers may delegate preserving their historical signature, but Python consumes the checked operations. No unwind catcher, duplicated unchecked path or speculative trait is justified. Malformed imported graph/relationship and exhausted IDs must leave bytes, graph and revisions unchanged.

Checklist preset uses existing PyListLevel: ListLevel.checklist(*, checked=False) returns ListLevel(format='bullet', text='☑' if checked else '☐'). Use actual bullet definitions and numbering references. No ordinary-text prefix, new checklist type or mandatory unrelated helper. Deterministic glyph coverage and actual native PDF/raster output remain required.

### Tests and remaining oracle barrier

Add the executed super/sub transition matrix to existing test_formatting_tables.py, including explicit baseline, direct absent/inherited facts and None-no-node controls. Add tabs sorting/equal positions/negative positions/index errors/None leader/read-only source, full paragraph slot paths, style atomic validation/preservation, document-default no-restyle/foreign-wrapper preservation, numbering startOverride XML and malformed imported helper atomicity controls. Existing entrypoints only, installed runtime and strict mypy/stubtest required. Font and paragraph edits preserve parent text topology, existing structural edits remain loudly stale. Consume F-X197 once for all new colours.

The authenticated Word evidence below resolves the original plain-instance, interruption and multilevel experiment. Keep current production behavior unchanged until the explicit existing compatibility contract, additional affected constructs and measured output policy are reconciled. Any intentional renderer behavior change requires a labeled commit and reviewed hash delta.

Existing rdocx-oxml/src/styles.rs is an additional exclusive source owner
for minimal default-wrapper preservation. Run parser/serializer namespace
and raw child-order riders on that repair. No new module is proposed.

## Authenticated Word numbering result

Native Word 16.113.2, build 16.113.26092012, displayed and exported all
21 labeled numbering controls consistently before and after normal reopening.
The corrected input puts all abstract definitions before concrete instances,
uses distinct independent definition identities, and opened without repair.
Plain fresh instances sharing one abstract definition printed 1,2,3,4.
Explicit level-zero startOverride1 printed 1,2,1,2. The same instance
continued after an unnumbered interruption. Multilevel controls printed
1 / 1.1 / 1.2 / 2 / 2.1, then a plain fresh instance printed 3 / 3.1,
and an explicit root restart printed 1 / 1.1. Both exported PDFs preserve
the same 21-number map. Native display and pinned Poppler 26.01.0 extraction
are authoritative here, since accessibility list numerals differed.

Evidence and hashes are in the existing ignored
.claude/scratch/F-X200-word-numbering-oracle.md and the local
S91-F-X200-word-numbering/audit-receipt.json. The preliminary malformed
interleaved-definition fixture is excluded from acceptance. These controls
do not establish returning to old instances after overrides, style-linked
lists or every lvlRestart variant. Add controls if a change touches them.

The current independent-numId rendering is an intentional existing contract
in HLD12 and HLD08, with an explicit style-resolver test. This evidence
therefore requires a deliberate compatibility-contract decision and a
separate labeled behavior commit, rather than treating current behavior as
an unexplained defect. Final approval must reconcile that decision, actual
affected fixture keys and F-283 resolved-counter dependency before claiming
X200. No implementation or baseline update is authorized by this draft.
