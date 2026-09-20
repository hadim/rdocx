# F-270, Complete settings and web settings authoring

**Status**: completed
**Sprint**: S74
**Size**: L
**Depends on**: F-244

## Problem

`crates/rdocx-oxml/src/settings.rs` models ten `w:settings` children and nothing
else. `CT_Settings` at `crates/rdocx-oxml/src/settings.rs:186` carries
`document_protection`, `document_variables`, `compatibility_settings`,
`default_tab_stop`, `character_spacing_control`, `theme_font_language`,
`automatic_hyphenation`, `even_and_odd_headers`, `update_fields`,
`math_properties` and `source_xml`. Everything else in the part stays in
`source_xml` and is replayed verbatim by `to_xml` at
`crates/rdocx-oxml/src/settings.rs:775`. The story's named families are absent.
There is no `w:proofState`, no `w:mailMerge`, and no typed `w:compat` toggle
option. `crates/rdocx-oxml/src/settings.rs:1160` already enumerates 96 top-level
children in schema order, which proves the writer knows the sequence but models
almost none of it.

Removal is inconsistent. `set_automatic_hyphenation`
(`crates/rdocx-oxml/src/settings.rs:721`), `set_even_and_odd_headers`
(`:737`) and `set_math_properties` (`:709`) have no paired remover, and
`DocumentProtection` is a read-only projection with no setter at all
(`:491`). The facade mirrors that shape, so a caller can add a setting it
cannot then take away (`crates/rdocx/src/document.rs:18316` through
`:18588`).

There is no web settings support anywhere in the workspace. A repository-wide
case-insensitive search for `webSettings` returns nothing, and
`crates/oxml-opc/src/relationship.rs:17` has no `WEB_SETTINGS` relationship
constant, so the part is not even resolved, let alone modeled. The paragraph
side of the same gap is `w:divId`, whose schema slot already exists at
`crates/rdocx-oxml/src/properties.rs:2571` as a raw-preserved position with no
typed field, because the element it points at lives in the unmodeled web
settings part.

Nothing reports what a settings part contains that the model failed to own.
`from_xml` silently drops a duplicated or malformed modeled child by resetting
its value to `None` after counting occurrences
(`crates/rdocx-oxml/src/settings.rs:450` through `:473`), so a caller cannot
distinguish "absent" from "present but not owned". The story's acceptance
contract requires exactly that distinction.

Finally, `w:defaultTabStop` is modeled but never reaches layout.
`crates/oxml-layout/src/line.rs:1447` hard-codes `let default_interval = 36.0;`
and `crates/rdocx-layout/src/input.rs:155` has no default tab field, which is
why DOCX-007 carries Layout `P` and Render `P` with F-270 as its owner.

## Spec reference

- `docs/hld/02-scope-and-non-goals.md`, "Modern DOCX capability matrix", rows
  DOCX-007 and DOCX-037 and the legend paragraph above the table that defines
  `Y`, `P`, `PV`, `N` and `B` and requires an owner for every `partial` row.
- `docs/hld/03-architecture.md`, "What stays put", the settings owner paragraph
  at line 506 that fixes parsed producer bytes as the sole serialization source,
  and the OfficeMath settings paragraph at line 487.
- `docs/hld/03-architecture.md`, "Facade conventions", the staged settings
  mutation boundary at line 1112.
- `docs/hld/04-opc-and-packaging.md`, "What transfers unmodified", the settings
  paragraphs at lines 520 to 556 covering relationship-resolved targets, the
  bounded authoring surface, schema-position insertion, and the rule that an
  ordinary save never creates a part, relationship or content-type override the
  document did not already have.
- `docs/hld/04-opc-and-packaging.md`, "Relationship types" and "Part naming",
  for the new web settings relationship and its collision-safe part allocation.
- `docs/hld/10-bindings-spec.md`, "Native Word facade stability", for additive
  pre-1.0 Rust surface and the rule that Python, WASM and CLI do not gain entry
  points implicitly.
- `docs/hld/12-testing-strategy.md`, "Test taxonomy", the `round-trip` row.
- `docs/hld/12-testing-strategy.md`, "The private from-scratch DOCX conformance
  corpus", the feature-level property gate paragraph and the settings mutation
  fixtures paragraph at lines 1874 to 1895.
- `docs/hld/12-testing-strategy.md`, "The hash harness" and "The golden-PNG
  gate", for the parts that are baselined and the deterministic font rule.
- `docs/hld/01-glossary.md`, "Units", for `Twips` and the truncating
  constructors.
- `docs/hld/14-development-backlog.md`, "F-270, Complete settings and web
  settings authoring".

## Approach

### The closed supported set

"Supported" becomes a checkable constant rather than a judgement. One
`const SUPPORTED_SETTINGS: &[&str]` in `settings.rs` names, in schema order,
every top-level `w:settings` child the typed model owns after this story:

```
view, zoom, removePersonalInformation, removeDateAndTime, mirrorMargins,
gutterAtTop, proofState, linkStyles, mailMerge, trackRevisions,
doNotTrackMoves, doNotTrackFormatting, documentProtection, defaultTabStop,
autoHyphenation, consecutiveHyphenLimit, hyphenationZone, doNotHyphenateCaps,
defaultTableStyle, evenAndOddHeaders, bookFoldRevPrinting, bookFoldPrinting,
bookFoldPrintingSheets, characterSpacingControl, updateFields, compat,
docVars, mathPr, themeFontLang, decimalSymbol, listSeparator
```

That is 31 names. Nested supported names are `compat/compatSetting`, every
`CompatibilityOption` local name under `compat`, `docVars/docVar`, and the
thirteen bounded `mailMerge` children listed below.

Everything else stays preservation-only and is never a diagnostic. That
boundary is named explicitly so a reviewer can check it: `attachedTemplate`,
`rsids`, `clrSchemeMapping`, `shapeDefaults`, `hdrShapeDefaults`,
`attachedSchema`, `smartTagType`, `schemaLibrary`, `captions`,
`readModeInkLockDown`, `stylePaneFormatFilter`, `stylePaneSortMethod`,
`documentType`, `revisionView`, `autoFormatOverride`, the `styleLock` pair, the
drawing-grid family, the print and save families, the XML-handling family, and
every `w14:` or `w15:` MCE extension. `footnotePr` and `endnotePr` are
preservation-only here because F-269 owns section note configuration and F-274
owns note behaviour.

`attachedTemplate` is preservation-only because it carries an `r:id` and
relationship rebasing for a settings child has no owner yet. That is recorded as
a named follow-up in `## Open questions` rather than absorbed here. `rsids` is
preservation-only for the reason F-X128 established, which is that revision
session identities are retained rather than authored.

### Cross-story deliverables this sprint depends on

Two items are named deliverables rather than incidentals, because another S74
story consumes them and F-270 runs in wave 1 and completes first.

- **`w:mirrorMargins`, `w:gutterAtTop` and the book-fold trio**
  (`w:bookFoldRevPrinting`, `w:bookFoldPrinting`, `w:bookFoldPrintingSheets`).
  F-269 consumes these typed accessors for its mirrored-margin and book-fold
  work and adds none of its own. Building them here removes a duplicate-accessor
  conflict inside a single file. `bookFoldPrintingSheets` is a signed integer
  sheet count, the other four are on-off toggles.
- **`w:divId`.** F-264 types the `CT_PPr` field at slot 31 and ships its
  paragraph facade accessor, because `w:divId` is a `w:pPr` child and F-264 is
  the only wave 1 story that edits `CT_PPr`. This story owns the other half,
  `CT_WebSettings` and the read-only `div_ids()` projection that says whether a
  reference resolves. Splitting it this way keeps two wave 1 workers out of one
  struct.

`SUPPORTED_SETTINGS` is a strict subsequence of the existing 96-name order
table, and a unit test proves that mechanically.

### Collapse the three order tables into one

`setting_follows` (`:1159`), `setting_follows_auto_hyphenation` (`:1501`) and
`setting_follows_math_properties` (`:1352`) each hand-maintain a copy of the
same sequence. Adding members to three copies is how they drift. They collapse
into one `const SETTINGS_ORDER: &[&[u8]]` and one
`fn setting_follows(local: &[u8], element, prefixes) -> bool`, with the math
case expressed as `setting_follows(b"mathPr", ...)`. This reduces the number of
places a reader must look, which is the structural test.

### New typed members

`w:proofState`, one empty element with `w:spelling` and `w:grammar`:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProofState { Clean, Dirty }

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct DocumentProofState {
    pub spelling: Option<ProofState>,
    pub grammar: Option<ProofState>,
}
```

`w:compat` toggle options, one closed enum instead of roughly eighty fields,
because every `CT_Compat` child except `w:compatSetting` is a `CT_OnOff` of
identical shape:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum CompatibilityOption { /* the complete closed CT_Compat on-off child set */ }

impl CompatibilityOption {
    pub fn local_name(self) -> &'static str;
    fn parse(local: &str) -> Option<Self>;
}
```

The enum covers the **complete** closed `CT_Compat` on-off child set, roughly
eighty variants, not a selected subset. A partial set would make "no unmodeled
supported children" untrue for any Word-authored part, and that sentence is the
story's acceptance contract.

`w:mailMerge`, a bounded thirteen-child subset with the relationship-bearing and
complex children left preservation-only:

```rust
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct MailMerge {
    pub main_document_type: Option<MailMergeDocumentType>,
    pub link_to_query: Option<bool>,
    pub data_type: Option<String>,
    pub connect_string: Option<String>,
    pub query: Option<String>,
    pub do_not_suppress_blank_lines: Option<bool>,
    pub destination: Option<MailMergeDestination>,
    pub address_field_name: Option<String>,
    pub mail_subject: Option<String>,
    pub mail_as_attachment: Option<bool>,
    pub view_merged_data: Option<bool>,
    pub active_record: Option<i32>,
    pub check_errors: Option<i32>,
}
```

`w:dataSource`, `w:headerSource` and `w:odso` are preservation-only. The first
two carry `r:id` values whose rebasing belongs to the relationship layer, and
`w:odso` is a nested tree no story asked for.

The remaining top-level names in `SUPPORTED_SETTINGS` are on-off toggles
(`removePersonalInformation`, `removeDateAndTime`, `mirrorMargins`,
`gutterAtTop`, `linkStyles`, `trackRevisions`, `doNotTrackMoves`,
`doNotTrackFormatting`, `doNotHyphenateCaps`, `bookFoldRevPrinting`,
`bookFoldPrinting`), bounded enums (`view`, `zoom` with its `w:percent`
attribute), `Twips` values (`hyphenationZone`), integer counts
(`consecutiveHyphenLimit`, `bookFoldPrintingSheets`), or string values
(`defaultTableStyle`, `decimalSymbol`, `listSeparator`).

### `w:divId` and the web settings division reference

`CT_PPr` gains `pub div_id: Option<u32>` written at the reserved slot 31, so the
element stops being raw-preserved and becomes typed at the same schema position.
The paragraph facade gains a getter, setter and remover. `w:divs` itself stays
preservation-only for authoring, but `CT_WebSettings` exposes a read-only
`pub fn div_ids(&self) -> Vec<u32>` projected from the retained `w:divs`
subtree, so a caller can check that a `w:divId` resolves to a division the part
actually declares. That makes the reference checkable without opening
`w:divs` to authoring, which no story asked for.

### Typed removal

Every typed member gains a paired remover that returns the previous value:

```rust
pub fn remove_automatic_hyphenation(&mut self) -> Result<Option<bool>>;
pub fn remove_even_and_odd_headers(&mut self) -> Result<Option<bool>>;
pub fn remove_math_properties(&mut self) -> Result<Option<MathProperties>>;
pub fn set_document_protection(&mut self, value: DocumentProtection) -> Result<()>;
pub fn remove_document_protection(&mut self) -> Result<Option<DocumentProtection>>;
pub fn remove_proof_state(&mut self) -> Result<Option<DocumentProofState>>;
pub fn remove_mail_merge(&mut self) -> Result<Option<MailMerge>>;
pub fn remove_compatibility_option(&mut self, option: CompatibilityOption) -> Result<Option<bool>>;
```

plus the same shape for each new scalar member. Removal writes an empty
replacement through the existing `rewrite_scalar` path, which already deletes
the modeled occurrence and leaves every neighbouring byte alone. Removal is
never allowed to reinterpret an unowned occurrence, so it returns
`OxmlError::InvalidValue` when the member is duplicated or malformed, exactly
as `set_update_fields` already does at `:754`.

`set_document_protection` accepts a caller-supplied `hash`, `salt` and
`spin_count` verbatim and derives nothing. No cryptographic randomness is
introduced, which keeps DOCX-002's determinism note untouched.

**Password derivation is an explicit non-goal, not a gap.** This surface records
protection metadata the caller already holds. It never computes a hash from a
password, never chooses a salt, and never picks a spin count. A reviewer should
read the absence of those as deliberate. Nothing in DOCX-007 or DOCX-037 asks
for them, and inventing them would put fresh cryptographic randomness into a
determinism-gated path.

### Diagnostics

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingsDiagnosticReason {
    /// More than one occurrence, so no single occurrence owns the value.
    Duplicated,
    /// One occurrence whose attributes did not parse to a typed value.
    Malformed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SettingsDiagnostic {
    /// The path from the part root, such as `["compat", "compatSetting"]`.
    pub path: Vec<&'static str>,
    pub occurrences: usize,
    pub reason: SettingsDiagnosticReason,
}

impl CT_Settings {
    pub fn diagnostics(&self) -> &[SettingsDiagnostic];
}
```

Diagnostics are computed once during `from_xml` and stored, so the accessor
borrows rather than re-parsing. A name outside the closed supported set is never
a diagnostic, which is what makes "no unmodeled supported children" decidable
rather than open-ended. A package authored entirely through the public API can
only produce well-formed single occurrences, so its diagnostic list is empty by
construction, and that is the acceptance assertion.

`CT_WebSettings` exposes the same two types.

### Web settings

The new module was explicitly approved in the S74 consolidated design round.
`crates/rdocx-oxml/src/web_settings.rs` owns `CT_WebSettings` with the
same source-preserving architecture as `CT_Settings`: `from_xml(&[u8])`,
`to_xml(&self)`, a retained `source_xml`, prefix-tolerant reads, fixed `w:`
writes, and schema-position insertion against a `WEB_SETTINGS_ORDER` table.

Supported children: `encoding`, `optimizeForBrowser`, `relyOnVML`, `allowPNG`,
`doNotRelyOnCSS`, `doNotSaveAsSingleFile`, `doNotOrganizeInFolder`,
`doNotUseLongFileNames`, `pixelsPerInch`, `targetScreenSz`,
`saveSmartTagsAsXml`. `w:frameset` and `w:divs` are preservation-only, because
`w:frameset` carries relationship targets and `w:divs` is a nested tree no story
asked for.

`crates/oxml-opc/src/relationship.rs` gains one constant in the existing
`rel_types` module:

```rust
pub const WEB_SETTINGS: &str =
    "http://schemas.openxmlformats.org/officeDocument/2006/relationships/webSettings";
```

The content type stays a private const in `crates/rdocx/src/document.rs`
alongside `SETTINGS_CONTENT_TYPE` at `:4594`.

### Facade wiring

`Document` gains `web_settings`, `web_settings_part_name` and
`web_settings_owned` beside the three settings fields at
`crates/rdocx/src/document.rs:4507`, resolved through `resolve_part` at `:11272`,
written back at `:11751`, carried through `clone_for_staging` at `:10017` and
through the reopen path at `:11635`. A
`web_settings_mutation_candidate` and a `prune_empty_owned_web_settings` mirror
`settings_mutation_candidate` at `:19048` and `prune_empty_owned_settings` at
`:19075`.

`new_word_compatible_package` at `:4657` is **not** changed. A fresh package
gains no web settings part, which keeps the packaging rule at
`docs/hld/04-opc-and-packaging.md:527` and leaves the exact expected member set
in `scripts/docx_authoring_conformance.py:350` untouched.

Public facade additions are the getter, setter and remover for each new typed
member, plus `settings_diagnostics()` and `web_settings_diagnostics()`. Python,
WASM and CLI gain nothing, which is what the `B` columns on DOCX-007 and
DOCX-037 already record.

### Default tab stop reaches layout

`w:defaultTabStop` closes the Layout and Render `P` on DOCX-007.
`oxml-layout::LineBreakParams` gains `pub default_tab_interval_pt: f64` with a
`Default` of `36.0`, the exact literal it replaces at
`crates/oxml-layout/src/line.rs:1447`. `resolve_tab_width` and
`inline_to_line_item` take it as a parameter.
`rdocx-layout::LayoutInput` gains `pub default_tab_stop: Option<Twips>`, and the
`rdocx-layout` conversion sets `default_tab_interval_pt` from it, falling back
to `36.0` when absent. The `rdocx` facade fills `LayoutInput::default_tab_stop`
from `CT_Settings::default_tab_stop()`. All eight exhaustive `LineBreakParams`
literals, including the two in `crates/rpptx-render/src/text.rs:792` and the two
in `crates/rdocx-layout/src/convert.rs`, set `36.0` explicitly, which is not a
behaviour change.

Semver impact of the new public field: adding it to `LineBreakParams` is
breaking for any exhaustive struct literal outside this workspace, so
`oxml-layout` takes a pre-1.0 minor bump. Every other addition in this story is
purely additive. The explicit `36.0` fallback is what keeps the samples from
moving, so it is not an optional detail.

## Rejected alternatives

- One struct field per `w:compat` toggle. Roughly eighty near-identical
  `Option<bool>` fields for one closed uniform family. The enum plus an ordered
  list carries the same information in one place.
- Model `w:mailMerge` as a single replaceable subtree. Replacing the whole
  element would drop `w:odso` and the relationship-bearing sources, which
  violates the preserve-unmodelled rule.
- Define "supported" as everything in the 96-name order table. That makes the
  acceptance contract unbounded and turns every future Word child into a
  regression.
- Put `CT_WebSettings` in `settings.rs`. That file is already 1991 lines and
  would then own two unrelated part roots, which defeats answering "what does
  this do?" from one file.
- Add a `SettingsChild` trait over the typed members. There is no second
  implementer that needs polymorphism today, and a match in the rewrite helper
  is fewer places to look.
- Model a diagnostic reason for "supported name we chose not to model". The
  closed set is defined as exactly what the model owns, so that state cannot
  exist.
- Emit a web settings part in fresh packages. It would change the conformance
  part inventory and break the rule that an ordinary save adds nothing.
- Open `w:divs` to authoring so `w:divId` can create its target. No story asks
  for HTML division authoring. A read-only `div_ids()` projection makes the
  reference checkable at a fraction of the surface.
- Leave `w:mirrorMargins`, `w:gutterAtTop` and the book-fold trio to F-269. Both
  stories would then add accessors to `crates/rdocx-oxml/src/settings.rs`, which
  is the one file a duplicate-accessor conflict would be worst in.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| round-trip | `public_authored_settings_package_reports_no_unmodeled_supported_children` | The **test gate**. Author every supported settings and web settings member through the public `Document` surface, save, reopen. Every typed value returns, `settings_diagnostics()` and `web_settings_diagnostics()` are both empty, and foreign `x:ext` subtrees placed before, inside `w:compat`, inside `w:mailMerge`, after the last child, and inside `w:webSettings` are byte identical to their input. |
| round-trip | `settings_children_serialize_in_schema_sequence_order` | Author members in reverse schema order, then assert the serialized top-level child order equals `SUPPORTED_SETTINGS` filtered to the emitted names. Same assertion inside `w:compat`, `w:mailMerge` and `w:webSettings`. This is the `xsd:sequence` proof. |
| unit | `supported_settings_is_a_strict_subsequence_of_the_order_table` | `SUPPORTED_SETTINGS` has no duplicates and appears in `SETTINGS_ORDER` in the same relative order. Guards the collapsed order table. |
| unit | `every_supported_name_is_projected_by_from_xml` | A fixture carrying exactly one well-formed instance of each supported name projects a typed value for each and reports zero diagnostics. Closes the gap between the constant and the parser. |
| unit | `duplicate_and_malformed_supported_children_report_diagnostics` | Duplicated and malformed occurrences of each supported name produce one `Duplicated` or `Malformed` entry with the right path and occurrence count, and stay byte identical on save. |
| unit | `unsupported_settings_children_are_never_diagnostics` | `rsids`, `clrSchemeMapping`, `shapeDefaults`, `w15:docId` and a foreign-namespace lookalike produce no diagnostic and survive byte for byte. |
| regression | `removing_a_setting_does_not_disturb_neighbouring_producer_bytes` | Every remover deletes only its own occurrence. Named as the failure it prevents. |
| regression | `removing_an_ambiguous_setting_fails_without_changing_the_document` | Removal of a duplicated or malformed member returns an error and leaves package bytes identical. |
| regression | `prefix_aliases_and_foreign_lookalikes_do_not_shadow_a_settings_child` | Reads accept in-scope Word aliases, writes use fixed `w:`, and a same-local-name element in a foreign namespace is never taken as the modeled child. |
| integration | `web_settings_part_is_created_on_demand_and_pruned_when_empty` | A document without a web settings relationship gains no part on an ordinary save, gains a collision-safe part and relationship on the first authored value, and loses the part, relationship and content-type override when the last value is removed. |
| integration | `fresh_package_profiles_gain_no_web_settings_part` | The four Word-compatible profiles keep their exact existing member inventory. |
| unit | `document_default_tab_stop_drives_implicit_tab_positions` | A paragraph with no explicit tab stops resolves implicit stops at the document `w:defaultTabStop` interval, and an absent setting reproduces the 36.0 point fallback exactly. Deterministic font mode. |
| round-trip | `web_settings_divisions_are_reported_for_reference_checking` | `CT_WebSettings::div_ids()` reports every declared `w:div` from a producer web settings part, the retained `w:divs` subtree stays byte identical through save and reopen, and an empty or absent `w:divs` reports no divisions. This asserts only this story's half. F-264 owns the `w:divId` field and its round-trip, and the two halves meet at integration rather than inside either worker. |
| round-trip | `mirror_margins_gutter_at_top_and_book_fold_round_trip` | The five accessors F-269 consumes read, write, remove and survive save and reopen, with `bookFoldPrintingSheets` keeping its integer value and all five landing at their schema positions between `saveFormsData` and `hideSpellingErrors` and between `evenAndOddHeaders` and `characterSpacingControl`. |
| unit | `document_protection_authoring_records_caller_metadata_verbatim` | `set_document_protection` stores the caller's hash, salt and spin count unchanged and derives nothing, and two identical calls produce byte-identical output. Pins the non-goal. |
| unit | `compatibility_option_covers_the_complete_compat_on_off_set` | Every `CompatibilityOption` variant maps to a distinct local name, the set matches the closed `CT_Compat` on-off child list, and a fixture carrying all of them reports zero diagnostics. |

Add these as modules inside `crates/rdocx/tests/integration_test.rs` and
`crates/rdocx/tests/regression_test.rs`. No new test binary, no binary fixture.
Every fixture is constructed in source.

## HLD impact

- `docs/hld/02-scope-and-non-goals.md`
- `docs/hld/03-architecture.md`
- `docs/hld/04-opc-and-packaging.md`
- `docs/hld/12-testing-strategy.md`
- `docs/hld/14-development-backlog.md`

## Risk routing

- **Any parser or serialiser.** Read `docs/hld/04-opc-and-packaging.md` and
  `docs/hld/06-presentationml-model.md` before editing. Extra checks:
  `xsd:sequence` child order on write proved by
  `settings_children_serialize_in_schema_sequence_order` at four levels,
  prefix-tolerant read and fixed `w:` write proved by
  `prefix_aliases_and_foreign_lookalikes_do_not_shadow_a_settings_child`, and a
  round-trip test proving the retained subtree is preserved byte for byte at
  every insertion boundary.
- **Public API of a published crate.** Read `docs/hld/10-bindings-spec.md`,
  "Native Word facade stability", and the `CLAUDE.md` structural rules. Extra
  checks: state the semver impact, which is a pre-1.0 minor bump that is
  breaking for exhaustive `LineBreakParams` literals and additive everywhere
  else. Run `cargo publish --dry-run` for `oxml-opc`, `oxml-layout`,
  `rdocx-oxml`, `rdocx-layout` and `rdocx`, and keep the `.crate` size
  assertion. Add no surface no story asked for, which is why the bindings gain
  nothing.
- **A new trait, generic parameter, crate, module or file.** Read the
  `CLAUDE.md` structural rules. One new module,
  `crates/rdocx-oxml/src/web_settings.rs`, which the user explicitly approved in
  the S74 consolidated design round. No new trait, no new generic parameter, no
  new feature flag, no new crate.
- **Layout, pagination, line breaking, text shaping.** Read
  `docs/hld/08-rendering-spec.md`. Extra checks: the default tab interval test
  runs in deterministic font mode, and the golden-PNG and SVG page golden gates
  are re-run. Their fixtures are constructed in source and set no
  `w:defaultTabStop`, so no golden baseline is re-recorded. If one moves, stop
  and explain it rather than re-recording.
- **Unit conversion, `Twips`.** Read `docs/hld/01-glossary.md`, "Units", and the
  `CLAUDE.md` "Things that are deliberately wrong" entry. Extra check: new
  `Twips`-valued members (`hyphenationZone`, and `defaultTabStop` reaching
  layout) keep the existing truncating `as i64` constructors and the existing
  non-negative validation at `crates/rdocx-oxml/src/settings.rs:623`. No
  rounding change.

## Hash harness

**Expected unchanged, all 49 entries.**

`scripts/hash_harness.py:42` sets `OOXML_PARTS = ("word/document.xml",
"word/styles.xml", "word/numbering.xml")`. `word/settings.xml` is not a
recorded part, and neither is any web settings part, so no settings byte can
reach an XML entry. `SAMPLES` at `:32` is the seven named samples and
`EXPECTED_ENTRY_COUNT` at `:47` is `7 * (3 + 1 + 3) = 49`.

The remaining four entries per sample are the page-one PNG and the three PDF
fingerprints, which are rendered output. Only two changes in this story could
reach them:

1. The default tab interval. The fallback is `36.0`, byte-for-byte the literal
   it replaces at `crates/oxml-layout/src/line.rs:1447`, and
   `crates/rdocx/examples/generate_all_samples.rs` never calls
   `set_default_tab_stop`, so every sample takes the fallback.
2. Newly emitted default children. `CT_Settings::new()` stays empty and
   `new_word_compatible_package` is unchanged, so a fresh package still writes
   `<w:settings xmlns:w="..."/>` with no children. The one settings call any
   sample makes is `set_auto_hyphenation(true)` at
   `crates/rdocx/examples/generate_all_samples.rs:118`, whose modeled value and
   layout effect are untouched.

If any of the 49 entries moves, that is an unexplained delta and the work stops.

## Implementation checklist

- [x] Collapse `setting_follows`, `setting_follows_auto_hyphenation` and
      `setting_follows_math_properties` into one `SETTINGS_ORDER` table and one
      predicate, with no behaviour change and the existing tests green.
- [x] Add `SUPPORTED_SETTINGS` and the subsequence unit test.
- [x] Add `SettingsDiagnostic`, `SettingsDiagnosticReason` and diagnostic
      capture during `from_xml`, covering the ten members that exist today.
- [x] Add `w:proofState`, its enums, setter and remover.
- [x] Add `CompatibilityOption`, its ordered projection, setter and remover
      inside `w:compat`, preserving `w:compatSetting` and unmodelled siblings.
- [x] Add `MailMerge` with in-group schema ordering, preserving `w:dataSource`,
      `w:headerSource` and `w:odso`.
- [x] Add the remaining top-level scalar members in `SUPPORTED_SETTINGS`.
- [x] Deliver the `w:mirrorMargins`, `w:gutterAtTop`, `w:bookFoldRevPrinting`,
      `w:bookFoldPrinting` and `w:bookFoldPrintingSheets` accessors that F-269
      consumes. These are contract deliverables, not incidentals.
- [x] Add `CT_WebSettings::div_ids()` over the retained `w:divs` subtree, so a
      caller can tell whether a `w:divId` resolves. F-264 types the `CT_PPr`
      field and its facade accessor, because it owns `CT_PPr` in wave 1.
- [x] Add the missing removers and `set_document_protection`, and extend
      `CT_Settings::is_empty` to every new member.
- [x] Add `crates/rdocx-oxml/src/web_settings.rs`, register it in
      `crates/rdocx-oxml/src/lib.rs`, and add `rel_types::WEB_SETTINGS`.
- [x] Wire web settings through the facade: fields, resolution, save, clone,
      reopen, mutation candidate, prune. Leave fresh profiles unchanged.
- [x] Add the facade getters, setters, removers and the two diagnostic
      accessors.
- [x] Thread `default_tab_interval_pt` through `oxml-layout` and
      `LayoutInput::default_tab_stop` through `rdocx-layout` and the facade.
- [x] Add every test in the test plan to the existing integration and
      regression entrypoints.
- [x] Update the five HLD files, including flipping DOCX-007 and DOCX-037 to
      `complete` with implementation and test evidence.
- [x] Run `/verify`, then the golden-PNG and SVG page golden gates, then
      `cargo publish --dry-run` for the five affected crates. `oxml-opc`,
      `oxml-layout` and `rdocx-oxml` package and verify clean. `rdocx-layout`
      and `rdocx` cannot verify their tarballs until the 0.14.0 family is
      published, because the dry run resolves their workspace dependencies from
      crates.io. That condition predates this feature and is a release-time
      item.

## Open questions

None. Resolved in the S74 consolidated design round.

The seven questions this plan carried at draft were answered as follows. The new
`crates/rdocx-oxml/src/web_settings.rs` module is approved. The
`w:defaultTabStop` layout wiring is in scope, with the explicit `36.0` fallback
that keeps the samples fixed. The `LineBreakParams` public field is accepted as
a pre-1.0 breaking change, fixed in place at all eight exhaustive literals.
Document protection authoring records caller-supplied metadata verbatim, and
password derivation is a stated non-goal rather than a gap. `CompatibilityOption`
models the complete closed `CT_Compat` on-off set. `w:attachedTemplate` and
`w:rsids` stay preservation-only.

One named follow-up, deliberately not absorbed here: relationship rebasing for a
settings child has no owner, which is why `w:attachedTemplate` stays
preservation-only. That needs an F-ID before any relationship-bearing settings
child can be authored.
