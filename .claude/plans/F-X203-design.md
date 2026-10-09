# F-X203, Complete Python Word document operations and validation

**Status**: draft
**Sprint**: S91
**Size**: L
**Depends on**: F-X196, F-X201

This canonical draft cannot start implementation. Prerequisites are
unfinished and API decisions below remain open. Source citations were checked
against canonical head `360b5b1cf77d19e7fe5c4cc26152dd55a00d12c1`.

## Problem

Issue306 requests Python parity with existing native and CLI document
operations. Native exports and templates exist, but Python document.rs exposes
only core_properties at2102 among the requested property groups. Native
Document::text at `crates/rdocx/src/document.rs:19671` is main-body text and is
not CLI all-story text. CLI commands.rs:146 combines body plus readable stories,
with a separate malformed-story policy at230. CLI validation at2181 privately
assembles structural and advisory findings.

Native insert_document at document.rs:26198 is infallible and uses expect on
preflight. Checked import_fragment at18411 stages dependency closure with style
conflict policy. Exports ODT/RTF/EPUB return bytes plus loss diagnostics, not
bare bytes. A Python parity plan must not hide diagnostics, call a panic-based
insertion wrapper or treat existing body-only text as every-story output.

## Spec reference

- `docs/hld/14-development-backlog.md`, F-X203 and full Issue306 criteria.
- `docs/hld/10-bindings-spec.md`, Python API shape, The invalidation problem,
  handled loudly, Native Word facade stability and CLIs, including validation
  findings, accepted-view text, properties, fragment closure and templates.
- `docs/hld/03-architecture.md`, Three families, one workspace and Why these
  seams, for native validation ownership without a facade-to-CLI dependency.
- `docs/hld/08-rendering-spec.md`, Word OfficeMath layout, Word revision views
  and Word section geometry and page numbering.
- `docs/hld/12-testing-strategy.md`, Test taxonomy, Binding tests and The Word
  render fidelity gate.
- `docs/hld/04-opc-and-packaging.md`, The package and deterministic saves.

## Approach

Expose existing native capabilities through concrete checked binding objects
in existing files. Inventory all public Rust behavior and declared Python names
on the claimed Base, then write genuine installed-wheel missing-operation
regressions. Reuse integrated X196 visible paragraph validation and X201 section
story rules. Do not redefine their acceptance in a second implementation.

### Exports, text and counts

```python
Document.to_html() -> str
Document.to_markdown() -> str
Document.text() -> str
Document.text_report() -> TextReport
Document.to_odt() -> ExportResult
Document.to_rtf() -> ExportResult
Document.to_epub() -> ExportResult
Document.word_count() -> int
Document.character_count(*, include_spaces: bool = True) -> int
Document.page_count() -> int
```

ExportResult owns bytes and immutable typed diagnostic records. Reuse native
to_html/to_markdown at document.rs:28085/28097 and existing ODT/RTF/EPUB write
results at odt.rs:137, rtf.rs:85 and epub.rs:64. Do not discard the existing
path/message diagnostics from lossy converters. If naming is to_odt_bytes or
save_odt instead, settle the exact signatures once before stub publication.
Bare HTML/Markdown preserve existing native limitations and CommonMark subset,
not a new independent Python formatter.

Extract shared accepted-view text presentation from existing CLI into native
existing document.rs/lib.rs, with one owned TextReport containing rendered
plain text, scope and diagnostics. Its deterministic renderer matches CLI
body text, story ordering, separators and trailing newlines exactly. CLI
adopts the same function and retains stderr warning behavior. If another
story is malformed, retain body plus the same degraded main-only scope and
warning, instead of silently skipping a single owner or changing CLI policy.
Python text() returns the exact CLI stdout projection. text_report exposes
diagnostics separately so degraded extraction is inspectable. No subprocess,
temp-file CLI invocation or rdocx dependency on rdocx-cli.

Counts must have explicit scope. Existing word_count at document.rs:29025 uses
accepted main-body paragraphs, including table/control text, and whitespace
tokens. Proposed word_count preserves this native contract. Character count
uses Unicode scalar characters, with documented whitespace handling, across
the same accepted body projection. Do not count CLI presentation separators or
metadata story labels as content. All-story counts, if requested instead, need
explicit names and a shared native policy. Page count comes from fresh
deterministic layout, not cached app-properties pages. Reuse F-X195 caller-font
selection if caller font options are part of the final method contract.

### Templates and document assembly

```python
Document.render_template(data: Mapping[str, JsonValue]) -> int
Document.insert_document(other: Document, *, at: int,
    policy: FragmentConflictPolicy | None = None) -> None
Document.import_fragment(destination: ContentLocation,
    fragment: DocumentFragment, *, policy: FragmentConflictPolicy | None = None) -> None
FragmentConflictPolicy.reuse_equivalent() -> FragmentConflictPolicy
FragmentConflictPolicy.rename_all() -> FragmentConflictPolicy
FragmentConflictPolicy.with_style_reuse(reuse: bool) -> FragmentConflictPolicy
FragmentConflictPolicy.with_numbering_reuse(reuse: bool) -> FragmentConflictPolicy
FragmentConflictPolicy.with_related_part_reuse(reuse: bool) -> FragmentConflictPolicy
```

Native FragmentConflictPolicy at document.rs:1036 is a concrete non-exhaustive
struct, not an enum. Its constructors at1044/1053 reuse equivalent supported
dependencies or allocate fresh identities, with independent style/numbering/
related-leaf toggles. None means the native default reuse_equivalent policy.
Do not invent a 'fail' variant or a narrower style-only policy that discards
numbering and related-part decisions. JsonValue is a recursive supported
JSON scalar/list/string-keyed-object type alias in the existing stub.
DocumentFragment and ContentLocation reuse existing native owned fragment and
checked story boundary semantics. Bind only required constructors/export/read
metadata, with concrete package identity and frozen source ownership.

render_template at document.rs:27201 already delegates to the staged native
engine. Convert Python JSON-compatible scalars/arrays/objects explicitly.
Check bool before integer, bound integers to the native JSON representation,
reject nonfinite floats, bytes, cyclic containers and unsupported objects.
Never call arbitrary str() or attribute getters. Conversion completes before
mutable access. Scalar tags crossing runs, nested for/if, missing paths and
native atomic errors retain native behavior and exception mapping.

Implement insert_document via the checked fragment/dependency closure route,
or a minimal checked native try_insert_document in existing document.rs that
reuses that route. Do not bind the legacy infallible expect wrapper at26198.
Validate at against the documented direct body index rather than silently
clamping Python mistakes. Stage imported styles/numbering/media/comments/notes,
IDs and section ownership, with the existing equivalent-reuse/rename policy.
Preserve source Document and opaque dependency bytes. Structural success bumps
revision once and invalidates old destination handles. Failure bumps nothing.

### Properties and settings

```python
Document.custom_properties: CustomProperties
CustomProperties.get(name: str) -> CustomPropertyValue | None
CustomProperties.set(name: str, value: CustomPropertyValue) -> None
CustomProperties.delete(name: str) -> bool
Document.app_properties: AppProperties
Document.settings: Settings
Settings.track_revisions: bool | None
```

Native custom_properties/set/remove at document.rs:25950/25965/26011 and
application getters/setters at25911/25919 already stage checked mutations.
Reuse `oxml-core/src/custom_properties.rs:22` typed CustomPropertyValue and
actual AppProperties fields. Define string, bool, integer, floating and date
mapping exactly, preserving property IDs/types on updates. Reject bool-as-int,
out-of-range integers, nonfinite floats, invalid dates and invalid names before
mutation. Missing delete is a defined no-op. Date timezone/serialization choice
must match the native supported value contract.

AppProperties exposes company, manager, template and supported native count
fields without conflating cached producer counts with calculated counts.
Preserve unsupported fields and producer XML. Settings is document-owned and
read-only retrieval does not create a settings part or increment revision.
Native track_revisions getter/set/remove at document.rs:24956/24961 supplies
presence-sensitive None/false/true. Assignment and removal use staged native
settings mutation, preserving schema position and opaque settings children.
This sets the tracking flag only. F-291 owns tracked-edit authoring.

### Validation parity

Move the existing CLI structural/style/numbering/comment/metadata advisory
assembly into a shared native routine in existing files. Proposed result:

```rust
pub struct ValidationFinding {
    pub severity: ValidationSeverity,
    pub message: String,
}
pub struct ValidationReport {
    pub findings: Vec<ValidationFinding>,
}
pub fn validate_document_bytes(bytes: &[u8]) -> Result<ValidationReport>;
```

Exact message/order and validity projection match the CLI. Byte-level checking
is needed to diagnose a package that cannot produce a Document. Avoid assuming
a bound open Document can reproduce invalid unopened input. Python exposes
Document.validate() over an immutable serialized snapshot and a bytes/path
validation entrypoint if needed for full CLI malformed-package parity. Snapshot
validation must not change source, public revision or held-handle usability.
CLI formatting and exit status stay local to commands.rs, consuming the report.
Malformed XML findings retain named part errors and open failure, not an opaque
panic or swallowed scan error. Reuse X196's final opaque-aware visible-paragraph
logic and X201 checked section findings, not its earlier unreconciled draft.

### Equations and content controls

```python
Paragraph.add_equation(source: str, *, format: Literal['latex', 'mathml']) -> Equation
Paragraph.equations() -> tuple[Equation, ...]
Equation.to_latex() -> MathConversionResult
Equation.to_mathml() -> MathConversionResult
Document.content_controls() -> tuple[ContentControl, ...]
Document.set_content_control_value_by_tag(tag: str, value: str) -> int
Document.set_content_control_value_by_alias(alias: str, value: str) -> int
```

Native equations are paragraph-owned, paragraph.rs:960/981. Conversion already
exists at math.rs:115/137/182/200 and returns MathConversionResult diagnostics
for bounded LaTex/MathML support. Preserve those diagnostics, validate before
owner mutation and use native OfficeMath serialization/schema order. Do not
claim arbitrary TeX execution or invent an unrelated body insertion contract.
Equation snapshots and source owner identity follow existing handle rules.

Native content_control.rs:69 inventory is typed main-body controls, with
recursive accepted ownership. Expose tag/alias/type/text with frozen immutable
records and document that scope. The by-tag/by-alias setters at95/102 already
stage display/custom XML binding updates. Reuse exactly, including duplicate
selection and unsupported binding refusal. Do not reinterpret list results as
mutable detached XML handles. No content-control creation, dropdown authoring
or tracked-change creation is included. If all-story inventory is required,
that native expansion must be explicitly designed and tested, not assumed.

## Rejected alternatives

- Document.text wrapper advertised as all-story text when it is body-only.
- Running the CLI from Python or adding a facade-to-CLI dependency.
- Discarding export/math loss diagnostics to offer a convenient bare value.
- Binding the expect-based native insert_document wrapper directly.
- Calling arbitrary Python str() during template data conversion.
- Cached producer pages called fresh layout page count.
- Detached property/settings/control snapshots pretending to be mutable owners.
- A new source file/module for a forwarding binding abstraction.

## Test plan

**Test gate**: regression.

| Category | Test | Asserts |
|---|---|---|
| regression | test_issue306_missing_operations_on_installed_base | Genuine missing-method failures for every advertised operation before production edits |
| regression | python_markdown_html_match_native_cli | Sample corpus accepted content, headings/lists/tables/links/bold/italic, exact MD CLI bytes |
| regression | python_all_story_text_matches_cli | Body tables/controls, headers/footers/notes/comments/text boxes, ordering/markers/newlines, accepted revisions |
| regression | malformed_story_text_keeps_cli_degraded_scope | Same retained body, scope and diagnostic, no silent loss or source mutation |
| regression | python_exports_retain_loss_diagnostics | ODT/RTF/EPUB bytes open with native converter, unsupported source reports path/message, deterministic output |
| regression | counts_have_explicit_content_and_layout_scope | Unicode scalars, spaces, native word tokens, cached-vs-calculated pages, deterministic layout |
| regression | python_template_json_conversion_and_native_blocks | Split-run scalars, nested for/if, invalid/cyclic/unsupported values and native failure byte equality |
| regression | checked_document_fragment_import_closes_dependencies | Styles/policies, images/numbering/notes/comments/custom XML, source unchanged, stale handles and atomic failure |
| regression | typed_custom_and_app_properties_roundtrip | Bool/int/date/string/float distinctions, IDs, delete no-op, invalid bounds and opaque producer fields |
| regression | python_validation_matches_cli_findings | Complete finding ordering/message/severity and verdict, X196 pictures/opaque controls, X201 sections, malformed unopened packages |
| regression | equation_conversion_and_insertion_preserve_diagnostics | Supported LaTex/MathML, unsupported constructs, owner stale/invalid rollback, OMML reopen/render |
| regression | content_control_inventory_and_binding_are_checked | Tag/alias/type/text, duplicate selectors, custom XML update, unsupported binding byte equality |
| regression | track_revisions_is_presence_sensitive | None/false/true, readonly no part creation, schema placement and opaque settings preservation |
| binding | issue306_runtime_stubs_and_revision_contract | Fresh current worker wheel, neutral CWD, strict mypy/stubtest, correct types/errors and live/stale ownership |
| round-trip | issue306_all_mutations_save_and_reopen | Every criterion survives native reopen, source bytes/closure and appropriate deterministic render |

Reuse existing Python test_core/typing_smoke and native regression entrypoints.
Reference native fragment conflict regression_test.rs:37569, atomic incomplete
closure:37691, full-story dependency graph:46119 and owner-pair matrix:46683.
Reuse template paired text boxes:21373 and control boundary refusal:21992,
content-control binding controls:20160 through20356 and existing property tests.
Rebase citations before claiming. No new test binary or binary fixture.

Pinned native Word opening/rendering and existing LibreOffice/export oracles
validate only the planned exact cases. Google Docs custom-property loss is
documented, and supported OMML/control/Markdown import is best effort. Sample
corpus Python MD, text and validation parity is required, not optional.

## HLD impact

- `docs/hld/03-architecture.md`
- `docs/hld/08-rendering-spec.md`
- `docs/hld/10-bindings-spec.md`
- `docs/hld/12-testing-strategy.md`
- `docs/hld/14-development-backlog.md`

Update native ownership of shared validation/text, equation/count semantics if
needed, current Python APIs and invalidation, complete parity/diagnostic tests,
and actual delivered story scope. No fictional Word HLD07 document exists.

## Risk routing

- Parser/serializer: HLD04 The package and HLD06 Preservation strategy, exact
  opaque subtree/namespace/sequence controls for properties/settings/equations
  and dependency imports, malformed source refusal without partial mutation.
- Architecture: HLD03 dependency graph, native shared validation has no CLI
  dependency and neutral oxml crates gain no facade dependency.
- Public API: HLD10 exact new types/signatures, pre-1.0 source impact declared,
  no new file/module/trait, verified locally patched workspace dry-run packaging
  with --allow-dirty for scoped work and all archives below10MiB.
- PyO3: fresh installed full declared suite, strict mypy/stubtest and both WASM
  checks. Binding crates remain excluded from Rust workspace test binaries.
- Layout: HLD08 page-count/equation controls use deterministic fonts. Reuse
  existing page geometry and F-X195 caller-font selection, no system-font
  baseline or cached producer count substituted for actual layout.
- External oracle: differential-testing skill, pinned Word/LibreOffice/converter
  versions and exact bounded input/output receipts. Corpus parity compares the
  same immutable package through native CLI and current Python extension.
- Archive carriers, if changed source inventory requires them: HLD15 Packaging
  and release command, actual local affected records only, same tolerances and
  inventories, no publication or manifest/version changes.

## Hash harness

Expected49 existing deterministic outputs unchanged for binding exposure and
shared validation/text refactoring. Keep existing native template/export/import
semantics. If a discovered native defect requires behavior changes, record its
separate approved expected delta and exact affected entries before proceeding.
Neither parity wrappers nor diagnostic projection justify an unexplained hash
delta or a second baseline writer.

## Exclusive file claims and integration

Proposed native existing document.rs/lib.rs plus only necessary existing
content_control/math/export modules. Shared validation and text extraction
touch existing rdocx-cli commands.rs and its integration entrypoint, after
X196 integration and remediation. Python existing document.rs/paragraph.rs/
lib.rs, _rdocx.pyi and current test/typing entrypoints. Final exact claim list
follows public API decisions, with no new source files. Serialize document.rs,
binding registration/stub and HLD overlaps with X200/X201/X202. X204 consumes
these document operations and owns broader one-shot commands. Workers never
write sprint totals. Fresh dependency-prefix source is the test Base.

## Implementation checklist

- [ ] Integrate reviewed X196/X201 and resolve final count/text/validation scope.
- [ ] Freeze typed signatures, diagnostics and exact source ownership.
- [ ] Prove each missing installed Base operation and corpus parity failure.
- [ ] Share native CLI validation/text without changing its report semantics.
- [ ] Bind exports/templates/checked assembly/properties/equations/controls/settings.
- [ ] Prove source preservation, revision and malformed atomic controls.
- [ ] Run complete sample-corpus parity and declared runtime/typing/oracle gates.
- [ ] Run scoped hash, package, repository-policy, WASM and risk riders.
- [ ] Finish independent zero-defect, zero-smell review, HLD and valid handoff.

## Open questions

1. Final count scope and character definition need one explicit contract.
   Proposed native word/body scope and Unicode scalar character count differ
   from all-story text presentation. Do not count CLI labels as content.
2. Approve the immutable TextReport and export/math diagnostic result names,
   plus the Python bytes/path validation entrypoint for malformed unopened
   packages. Document.validate alone cannot represent every failed-open input.
3. Final fragment constructors and insertion bounds must match native owned
   closure and FragmentConflictPolicy methods, with a checked panic-free
   entrypoint. No new conflict policy is assumed.
4. Decide exact timezone/value conversion for typed custom dates and whether
   app-property writes use a live proxy or full typed snapshot replacement.
5. Equation insertion remains paragraph-owned. Confirm the convenience surface
   and supported conversions from existing native diagnostics.
6. Native content-control inventory currently means typed main body. If every
   story is requested, design that native expansion explicitly before approval.
7. Resolve shared document.rs/CLI/Python file ownership with X200/X201/X202/X204.
   Split into dependent children if necessary without omitting Issue306 items.
