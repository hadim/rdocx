# F-X211, Python script compatibility and safe XML access

**Status**: approved
**Sprint**: S91
**Size**: L
**Depends on**: F-X200 through F-X210 for final corpus acceptance, refined source barriers below

## Problem

Issue316 introduces a measurable first-draft compatibility contract, validated
writable element XML, stable owners and actionable errors. Current Word
test_python_docx_parity.py:60 holds17 documentation examples, while
PowerPoint test_documented_examples.py:34 explicitly re-fetches handles and
:701 holds7 pinned records. HLD10:160 through :174 documents their adaptation,
not a100-script unmodified correct-file metric. CI runs installed binding
tests with exact oracle versions at .github/workflows/ci.yml:220/:221.

PowerPoint Shape.xml is bytes-only read at shape.rs:568 and Word
StoryItem.xml is a snapshot read at document.rs:540. No replace_xml wrapper
was found on the requested owners. Global generations stale unchanged
ancestors, including table.rs:50 row/column restructure bumping at :66 and
notes_text at slide.rs:229. Word Hyperlink edit resolution at
document.rs:1545 uses a whole link-layout digest and fails after unrelated
add/remove. Stale errors currently name revisions and re-fetch paths but
not the mutation call. Provider fixes cover some examples, not this complete
cross-binding acceptance.

## Spec reference

- docs/hld/14-development-backlog.md, "F-X211, Complete Python script
  compatibility and safe XML access", full316 acceptance and providers.
- docs/hld/10-bindings-spec.md, "Python API shape", the17/7 compatibility
  gate, XML/preservation boundaries, and "The invalidation problem, handled
  loudly".
- docs/hld/12-testing-strategy.md, "Binding tests" and "What CI runs".
- docs/hld/02-scope-and-non-goals.md, public compatibility and current raw
  XML escape-hatch boundary.
- docs/hld/04-opc-and-packaging.md, actual owner relationships and preserved
  parts.
- docs/hld/06-presentationml-model.md, part owners, child sequence and
  retained source.
- docs/hld/03-architecture.md, allowed dependency graph and concrete ownership.
- docs/hld/08-rendering-spec.md, deterministic saved/rendered evidence.

## Approach

### Full acceptance and provider barriers

Preserve all316 requirements: about100 short idiomatic scripts spanning
303..313, saved-file assertions and per-area unmodified correct-file rates
in CI and final PR, existing17/7 examples as subset, declared actionable
divergences, complete element XML read/write/type validation, named supported
handle survival and invalidating operations, silent-failure audit and helpful
attribute/type errors.

Final corpus acceptance depends on integrated F-X200..X210. Corpus design
and native inventory can proceed read-only before all those stories finish.
Implementation barriers are actual ownership contracts:

- F-X194 and F-X206 must settle the common presentation text/layout/topology
  partition before XML adapters capture or invalidate handles.
- F-X202 must settle Word hyperlink and picture/physical-owner operations.
- F-X201 supplies checked section ownership and geometry validation.
- F-X209 supplies checked layout/master owners and editable native records.
- Other feature stories supply the exact APIs exercised by the frozen corpus.

Known fonts296, fragment-link305 and section304 fixes remain F-X195, F-X202
and F-X201. F-X194/F-X206 own already-requested cell/layout/title stability.
Do not reproduce their source fixes here. X211 owns residual Word hyperlink
and table-grid ancestor stability in both bindings unless the root explicitly
amends those provider plans to own that implementation. Notes ancestor
survival is assigned once alongside X206's title-text survival. X211 retains
the cross-story regression and acceptance evidence either way.

No new module, crate, dependency or feature flag is proposed. Existing test
entrypoints can hold the inline corpus rather than100 standalone files.
Root owns canonical plans, ledgers, baseline lease and final integration.

### Frozen first-draft corpus and rate

Extend existing Word test_python_docx_parity.py/test_core.py and PowerPoint
test_documented_examples.py with a frozen typed manifest of about100 short
scripts. Each record has stable ID, family, area, exact statements, expected
saved-file assertion and any explicitly deliberate divergence. Cover Word
formatting/lists/styles/tables/pictures/header/footer/notes/template/export,
PowerPoint slides/placeholders/shapes/tables/notes/fills/master/layout/theme/
fit/export, and the remaining303..313 operation families. Plan the per-area
allocation before measurements, not after observing failures.

Scripts run under isolated source-built installed bindings. They use rdocx or
rpptx imports with upstream idioms and no injected re-fetches, coercions,
aliases or preprocessing. Execution alone is insufficient. Reopen each
saved file and assert modeled structure and package invariants. Compare
supported public records against pinned python-docx1.2.0/python-pptx1.0.2.
Use deterministic rendering for visually meaningful assertions and preserve
actual native oracle provenance where a producer comparison is required.

Keep the existing17/7 gate, including its declared adaptations, as a named
subset. The metric also records whether the original upstream-style body
runs unmodified. An adapted pass is not an unmodified first-draft pass.
The existing held-row and title/subtitle adaptations become explicit baseline
cases. Do not quietly edit their source bodies to make the rate rise.

Report per area: attempted count, executed count, correct-file count,
deliberate divergence count, unexpected failure count and percentage of
attempted scripts that ran unmodified and yielded correct files. Deliberate
divergences remain in the denominator and do not count as successful runs.
Their expected actionable-error tests can pass independently. CI fails on
unexpected failure, missing assertion, duplicate/missing manifest identity or
unreviewed divergence. Retain fixed denominators and stable corpus digest.

Capture a real Base rate and final integrated post316/post303..313 rate with
source SHA, installed build identity, interpreter/oracle versions and corpus
digest. Put the observed final rates in the eventual PR. This proposal
contains no invented percentages or completed test claims. Existing CI's
installed binding jobs are the integration point, with no new workflow file.

### Element XML API and concrete owner matrix

Public signatures:

```python
paragraph.element.xml: str
paragraph.element.replace_xml(xml: str) -> None
```

The same two operations apply to every requested owner. Keep existing
Shape.xml and StoryItem.xml byte snapshot properties compatible. The new
element view is not an lxml proxy, mutable tree or promise of arbitrary
private python-docx attributes.

Issue316 literally lists ten generic names, not eleven. Cover all of them.
Paragraph/run/table/cell exist in both families, yielding fourteen concrete
family cases before shape variants:

| Family | Owner | Expected qualified root |
|---|---|---|
| Word | Paragraph | WordprocessingML p |
| Word | Run | WordprocessingML r |
| Word | Table | WordprocessingML tbl |
| Word | Cell | WordprocessingML tc |
| Word | Section properties | WordprocessingML sectPr |
| PowerPoint | Paragraph | DrawingML p |
| PowerPoint | Run | DrawingML r for regular Run handles |
| PowerPoint | Table | DrawingML tbl |
| PowerPoint | Cell | DrawingML tc |
| PowerPoint | Shape | Actual captured shape kind, such as PresentationML sp/pic/grpSp/cxnSp/graphicFrame |
| PowerPoint | Text body | Actual owner root, PresentationML txBody or DrawingML txBody |
| PowerPoint | Slide | PresentationML sld |
| PowerPoint | Layout | PresentationML sldLayout |
| PowerPoint | Master | PresentationML sldMaster |

Reject a regular run replacement with a field/br root. Any field-node escape
contract is an explicit additional owner, not hidden behind Run. Shape
replacement preserves the actual captured kind unless an independently
approved conversion operation permits changing it. Table/text-body nesting
uses the actual namespace of its owner, not a fixed prefix string.

The concrete element view owns a document/presentation reference, existing
owner/path information and captured generations. It performs qualified type
validation and atomic owner mutation, so it is not a forwarding-only wrapper.
Use existing ParagraphLocation, AcceptedRunPath, ContentLocation,
StoryRunPosition, ShapeMut/Ref and the X209 part-owner discriminator as
applicable. No generic unsafe replace-part method, trait, Box<dyn> or new
revision framework. A finite concrete discriminator for actual owner cases
may reduce reader cases if the final source layout needs it.

Read returns namespace-complete UTF-8 serialization for exactly the selected
owner, without mutation or generation changes. Standalone XML may use the
existing canonical writer prefixes. Preserve intentionally retained raw
subtrees and required inherited namespace closure. Do not return a fragment
whose prefixes only work in a removed parent.

### Native write operations, validation and atomicity

Binding wrappers call checked native owner operations, not a Python package
editor. The native parser for each owner is inventoried before implementation.
Provisional concrete Word facade signatures for design review are:

```rust
Document::replace_paragraph_xml(&mut self, location: &ContentLocation, xml: &str) -> Result<()>;
Document::replace_run_xml(&mut self, position: &StoryRunPosition, xml: &str) -> Result<()>;
Document::replace_table_xml(&mut self, location: &ContentLocation, xml: &str) -> Result<()>;
Document::replace_cell_xml(&mut self, table: &ContentLocation, row: usize, cell: usize, xml: &str) -> Result<()>;
Document::replace_section_properties_xml(&mut self, section: usize, xml: &str) -> Result<()>;
```

Their final location meaning must match actual physical owner semantics,
including accepted nested controls, related stories, merged cells and run
identity. Do not adopt these provisional signatures if ContentLocation
cannot faithfully resolve the requested target. Existing types are reused
where valid, with concrete native gaps recorded rather than fabricated APIs.

PowerPoint uses equivalent concrete checked operations in existing lib.rs:
replace_shape_xml, replace_text_body_xml and explicit cell/table/paragraph/
run paths, plus replace_slide_xml, replace_layout_xml and replace_master_xml.
Finalize signatures against X209's actual part-owner API. Slide-relative
index parameters cannot safely masquerade as master/layout ownership.
Borrowed ShapeMut alone lacks package relationship scope, so whole-node
replacement stays in the owner-aware Presentation operation.

Every replacement must validate:

- Exactly one complete root with expected namespace URI/local name and no
  trailing root or forbidden XML characters.
- Prefix aliases and default namespace equivalence, with wrong foreign or
  rebound same-local-name root refused.
- Modeled element invariants, valid known attributes and schema child order.
  Do not claim a complete XSD validator from an existing permissive parser.
- Existing owner identity, relationship target/type/scope, IDs and applicable
  comments/fields/sections/table grids. A dangling or wrong-owner r:id fails.
- Existing physical-alternate-copy guarantees from X198/X202. Raw writes are
  not a shortcut around unsupported scoped ownership.

Parse, mutate a staged candidate, validate affected owner/package, serialize
and reopen before commit. Failure preserves native state, XML/parts,
relationships and all generations. Success applies one mutation and advances
only the generations the approved stable-handle contract requires.
Identical replacement no-op semantics are explicit and tested. Unsupported
known invariant validation must produce a named refusal, not a false success.

Unknown target XML remains preserved where the model already permits it.
Exact unrelated opaque bytes remain unchanged. Intentionally replacing a
target does not require preserving its old target XML. Cross-part relationship
creation/import is a separate checked operation, not implicit in replace_xml.

### Stable owner and invalidation contract

Keep the approved provider partitions and extend only residual cases.
PowerPoint Table remains the same shape owner after a row/column edit, while
old indexed Row/Column/Cell and paragraph/run/font descendants can become
unsafe. Word tables have the analogous owner/descendant distinction.
Use checked owner identities or concrete topology counters that actually
express this distinction. Do not re-fetch a table by an old index after its
shape owner was removed or replaced.

Word Hyperlink snapshots retain read facts, while live edit resolution must
identify the original physical hyperlink after unrelated add/remove. Current
document.rs:1545 uses index plus story_link_layout digest at :666. Value
equality, URL or relationship ID alone cannot distinguish identical links.
Design a checked physical selector or native identity preserving unrelated
links. Removed/replaced owner must fail clearly rather than resolve another
identical link. Preserve constructed snapshot ambiguity refusal.

SlideRef.id is lib.rs:7674 and ShapeRef.non_visual_id is :9636. Check actual
non-reuse, uniqueness and owner kind before choosing stable-ID resolution.
Master/layout owners come from X209. No unbounded ID registry or universal
stable-handle system is proposed. True removed/reordered unsafe descendants
retain loud checks, and raw writes use the same generations.

Extend the existing concrete revision counters with bounded operation
provenance. The error names the real API call responsible for the relevant
generation transition and the exact recovery path. Current shared
StaleElementError only records kind/captured/current revision/recovery hint.
Do not substitute the latest unrelated call for the first causal invalidator
without explicitly defining that weaker supported message. Final design
must prove repeated mutations, no-op/failure cases and owner-specific causal
reporting. No generic history tracker or speculative event framework.

### Silent failures and actionable names

Inventory accepted Python arguments and CLI options, their native consumer,
unit/value bounds and actual saved output. Every detected ignored value or
modeled invalid OOXML gets a failing Base public test, cited source, owning
story and fixed-after evidence. Keep the known matrix explicit:

| Case | Implementation owner |
|---|---|
| Image output silently ignores --font-dir296 | F-X195 |
| add_hyperlink(text, "#name") becomes external305 | F-X202 |
| Section lacks valid page size304 | F-X201 |
| Wrong unit produces inconsistent tiny page304 | F-X201 |

Those fixes are reused, not duplicated. Do not invent minimum page thresholds
or change canonical length truncation to hide a wrong Python unit contract.
Raw escape writes validate the same modeled invariants as regular authoring.

For deliberate differently spelled upstream names, a bounded known-name
mapping returns AttributeError/TypeError or the existing documented exception
with attempted name, accepted forms and correct rdocx/rpptx call. Correct
already-authorized features such as font.color.rgb through X197 rather than
leave a divergence hint after implementation. No fuzzy matching, automatic
coercion, hidden retry, silent setter ignore or lxml emulation.

## Rejected alternatives

- Duplicate provider fonts/links/section/colour fixes. One source change and
  reusable evidence prevents redo and semantic conflicts.
- Inject refetches into first-draft scripts. It measures adaptation quality,
  not unmodified agent success.
- Remove divergences from denominator or count raised errors as success.
  That inflates the required correct-file rate.
- Use arbitrary package-part XML writes or XML in binding wrappers. It loses
  native owner, relationship, schema and atomicity guarantees.
- Disable stale checks or match stable handles by values. Removed/indexed
  owners can silently bind to another object.
- Add universal traits, dynamic registries or new modules. Concrete current
  owner types suffice, and absent capabilities must be designed explicitly.

## Test plan

**Test gate**: regression. Installed isolated bindings run the full corpus
and assert saved package structure. All requested element kinds round-trip
opaque content and refuse wrong types atomically. Handles survive supported
edits, unsupported invalidations name the triggering operation, and each
silent failure has a failing-before/fixed-after public control.

| Category | Test | Asserts |
|---|---|---|
| regression | first_draft_corpus_runs_unmodified_and_reports_per_area_rates | Stable exactly100-script denominator, saved-file facts and honest divergence counts |
| regression | existing_documentation_examples_remain_identified_subset | All17/7 IDs and declared old adaptations retained |
| regression | actionable_divergences_name_the_supported_call | Known attribute/type names, correct forms and no hidden mutation |
| regression | element_xml_covers_all_requested_family_owners | Fourteen concrete cases plus declared shape-kind variants and namespace closure |
| regression | replace_xml_refuses_wrong_qualified_roots_and_trailing_content | All roots, aliases/defaults, foreign/rebound controls and unchanged bytes/generations |
| regression | replace_xml_preserves_owner_relationships_and_opaque_content | Shared media, hyperlink IDs, comments/fields, foreign nodes and save/reopen twice |
| regression | raw_xml_cannot_bypass_physical_alternate_owner_safety | X198/X202 supported/refused cases, no unequal physical copies |
| regression | table_owner_survives_grid_edits_but_unsafe_descendants_fail | Both families, row/column add/remove/move, merged cells and removed-table refusal |
| regression | hyperlink_edit_handle_survives_unrelated_add_remove | Identical links, physical owner identity and removed-owner failure |
| regression | provider_layout_slide_and_cell_stability_is_integrated | X194/X206 layout, title, notes, cell text and topology acceptance |
| regression | stale_error_names_the_actual_invalidating_call | Both bindings, repeated edits, no-op/failure and exact recovery path |
| regression | accepted_option_audit_reuses_each_known_provider_failure | Four known cases, real consumer inventory and per-case failing/fixed evidence |
| integration | installed_binding_ci_prints_final_corpus_rate | Isolated source-built package, exact oracle/interpreter provenance and schema |

Use existing native modules and integration/regression entrypoints. Python
tests stay in current test_core.py, test_formatting_tables.py,
test_python_docx_parity.py, test_documented_examples.py and current typing
entrypoints. Stubs describe element views and ownership. Extend existing
.github/workflows/ci.yml and existing test scripts only. No new tracked file
is required by this proposal. Run strict mypy/stubtest against installed
bindings and meaningful deterministic render/native comparisons.

No regression was executed by this design. Preserve Base failure, provider
evidence and final integrated corpus result as separate records. Root records
rates in the eventual PR, after genuine measured completion.

## HLD impact

- docs/hld/02-scope-and-non-goals.md, concrete XML escape and compatibility boundary.
- docs/hld/03-architecture.md, native owner/path/provenance responsibility if changed.
- docs/hld/04-opc-and-packaging.md, checked raw replacement and relationship guarantees.
- docs/hld/06-presentationml-model.md, actual raw owner/type mutation boundaries.
- docs/hld/10-bindings-spec.md, exact element API, stable handles, errors and corpus metric.
- docs/hld/12-testing-strategy.md, frozen corpus, rates, divergences and CI methodology.
- docs/hld/14-development-backlog.md, delivered316 acceptance and provider dependencies.

## Risk routing

- Parser/serializer. HLD04/06 mandatory, qualified root checks, correct child
  sequence and exact unrelated opaque-byte round trips.
- Published native APIs. HLD10/structural rules, explicit additive pre-1.0
  changes, affected-crate archive dry-run/size and actual measured carriers.
- PyO3. HLD10, installed source-built tests, accurate strict mypy/stubtest,
  workspace binding exclusions and WASM graph compilation.
- Dependency edges. HLD03, no family reverse edge, native-to-Python bridge or
  generic mutator depending on binding-only path types.
- Layout/rendering and unit validation. HLD01/08/CLAUDE, canonical truncation,
  deterministic fonts and actual evidence for saved/rendered claims.
- External oracle. Canonical differential-testing skill, exact pinned
  python-docx/python-pptx/native producer versions and provenance.
- New concrete view/error fields. Structural rules, existing files and real
  current consumers, no unused framework or extra module.

Normal scoped gate and independent zero-defect/zero-smell microscope remain
required, followed by root's final integrated corpus/sprint verification.
Read HLD15 packaging before actual archive maintenance, not stale size reuse.

## Hash harness

Expect no default delta from metric/error infrastructure or additive XML
operations. Requested raw writes intentionally change only explicitly edited
targets, with opaque sibling controls. Any native parser/default rendering or
silent-behaviour correction gets its own labelled reviewed expected delta
under its actual provider. Root retains exclusive baseline ownership. Never
change baselines or corpus denominators to manufacture a passing rate.

## Implementation checklist

- [ ] Reconcile provider ownership and final corpus/source capability barriers.
- [ ] Freeze complete corpus IDs, statements, assertions and per-area denominators.
- [ ] Measure actual isolated installed Base first-draft rate and missing API failures.
- [ ] Inventory each native XML owner/parser/namespace/relationship validator.
- [ ] Finalize concrete native/Python element signatures and atomic replacement.
- [ ] Implement only assigned residual stable-owner/provenance/error behaviour.
- [ ] Reuse provider silent-failure fixes with exact integrated public controls.
- [ ] Add existing-entrypoint tests, stubs, CI reporting and HLD contract updates.
- [ ] Run scoped/risk gates and independent microscope over actual final diff.
- [ ] Measure final integrated first-draft rate and record observed PR evidence.

## Open questions

Resolved by the exact fourteen-family XML matrix, bounded causal compatibility
contract, provider ownership and100-script allocation below. F-X206 owns
shared state and existing consumer migration after F-X194/F-X197. Providers
create their new handles on that one foundation, with no provider dependency
on F-X211. This story owns residual audits/XML and the final integrated corpus.
Allocation is frozen now. Actual original statement/assertion bodies and their
separate full corpus digest must be frozen in existing entrypoints before
Base measurement or production changes. Allocation digest is not a body
freeze, executed rate or successful-file evidence.

## Bounded causal-token refinement

The existing shared RevisionCounter in oxml-py-support/src/lib.rs:80 serves
Word document.rs:1508 and PowerPoint presentation.rs:154. ContentPath owns
only a numeric revision today. Word StoryItem at document.rs:487 bypasses
ContentPath and builds a stale error directly at document.rs:1619, so it
must be included. The full read-only consumer and publication inventory is
.claude/scratch/F-X211-causal-invalidation-analysis.md.

The minimal concrete candidate replaces each active counter value with an
Arc to a RevisionState containing its immutable revision and a OnceLock of
InvalidationCause. Captured handles retain the same Arc. A successful relevant
structural commit fills that retiring state's actual public operation and
transition revision once, then swaps a fresh active state. Retired states have
no next-state link and are owned only by live captured handles. This bounds
memory by live captured generations instead of total intervening mutations.
The cause is one-time publication, not a literally immutable future value.

A handle captured before A then B must retain A as its first cause. A handle
captured between them reports B. No current-operation lookup or history map
is allowed. Remove counter Copy and owner cloning. Validate bound token and
owner identity, not equal numeric counters from different documents. Preserve
ContentPath and StoryItem visible value equality by excluding hidden mutable
cause state. Native StoryItems can capture authority, while publicly constructed
snapshots cannot invent it from a supplied revision. Preserve their established
snapshot behavior and give factual unavailable-provenance guidance when needed.

Preflight successor allocation and overflow before native publication. Failed
candidate validation, expected-count mismatch and contractual no-ops retain
native bytes, token identity and cause. Pass real public entrypoint names
through shared helpers and aliases, including property setters. Existing
text-setter semantics are not changed merely because display text is equal.
Use the same relevant generation partitions as X194/X206/X202/X209. This
mechanism does not make a currently unstable owner survive by itself.

Where multiple owner tokens can invalidate one handle, accurate chronological
first-cause selection requires one existing document commit serial shared by
the affected transitions. Provider coordination must prove that serial before
claiming cross-scope first causality. Otherwise report the named owner-generation
cause without falsely promising chronological first. Actual provider partitions,
constructed-snapshot edit authority and stable physical selectors remain
approval barriers. Add existing-entrypoint tests for repeated edits, cloned
handles, Arc weak lifetime reclamation, owner survival, failure/no-op stability,
actual alias labels and cross-document/constructed snapshots. This refinement
settles the bounded mechanism candidate, not full F-X211 design approval.

## Final causal compatibility and common serial contract

The authenticated resolution memo is
/private/tmp/S91-F-X211-causal-finalization/resolution-memo.md. Its15 source
references match current canonical bytes. Preserve visible Story/StoryItem
constructors and field-based equality. Manual numeric construction remains
an explicit unbound current-target selector, never a captured owner token.
An unmatched numeric guard reports unavailable causal provenance, not an
invented first invalidating operation. Preserve existing missing-manual-story
error ordering. A manually constructed Item using a bound Story retains that
Story's originating-owner constraint without fabricating item generation.

Native captures and their clones carry hidden document/owner state tokens.
Reject foreign document identity before lookup or revision comparison. Validate
retired relevant tokens before resolving their current native location.
Visible equality excludes identity, token pointers and mutable cause state.
Removed/recreated physical owners cannot resurrect captured handles.
ContentPath::new stays const/unbound with no allocation. Genuine captures
use token-aware validation. Removing RevisionCounter Copy and adding hidden
Rust fields is an intentional pre-1.0 source change, not claimed compatibility.

Keep the current top-level revision as one monotonic invalidating-transaction
serial alongside partitioned scope generations. Capture's visible serial
is distinct from the local generation token. An unrelated owner's commit
must not make a still-live scope stale by comparing these different numbers.
The earliest relevant retired token is selected by minimum common committed
serial, never unrelated local generations or validator traversal order.
Same-transaction ties have one public label/serial and fixed scope ordering.

Each owner retains only its current Arc<RevisionState>. Live handles may
retain old states with one OnceLock first cause. No event history, next-state
pointer, revision lookup map or document ownership cycle. Preflight scalar
and local checked_add plus successor allocation before native publication.
Publish success and retire/swap exactly once without a fallible step afterward.
No-op/refusal retains bytes, all state identities, causes and serial values.

Existing-entrypoint tests cover bound/manual equality, foreign equal-valued
capture refusal, manual selectors, bound Story propagation, alias labels,
clone/removal/reuse, reversed multi-scope validator order, overflow-before-edit,
late failure and weak-state reclamation after thousands of uncaptured edits.
These settle the design's causal question, with provider implementation and
actual frozen-corpus acceptance still required once, not already passed.

## Approved XML scope and exact corpus allocation

X211 element matrix is exactly the plan's ten generic names/fourteen family cases. Keep paragraph.element.xml:str and element.replace_xml(xml:str)->None, namespace-complete serialization, concrete checked native ContentLocation/StoryRunPosition or true part-owner routes, no unsafe generic package write or lxml promise. Exactly one matching qualified root, modeled invariant/schema order and owned relationship validation, inherited namespace closure, staged serialize/reopen and same-token atomicity. Raw writes retain X198/X202 paired-owner barriers. Regular Run refuses fld/br roots. Declared limits reuse current package/text parsing caps, reject DTD/entity/network resolution and malformed root/trailing content before mutable access. Not a complete XSD validator. Identical fully equivalent owner replacement is no-op. replacing owner topology retires precisely its unsafe descendants. Existing manual snapshots remain compatible.

## Fixed allocation and evidence boundaries

Selected exact100-script allocation: Word49 and PowerPoint51. Existing examples24 (Word17/PPT7), XML14 (Word5/PPT9), lifetime16 (Word7/PPT9), provider acceptance46 (Word20/PPT26). Shape XML case includes sp/pic/grpSp/cxnSp/graphicFrame variants within one script, not extra denominator identities. Each XML script bundles positive qualified/default/alias closure and save/reopen with wrong-root/foreign/late-failure atomic checks. Lifetime scripts include wrong-owner and retired-handle actionable failures alongside required survival. These nested controls do not increase script count.

Canonical allocation JSON is UTF-8 compact json.dumps(records,sort_keys=True,separators=(",",":")), each record has id/area/case, with area equal to the uppercase second component of id (BASE/XML/LIFE/AREA). Independently derived SHA256 f6adc336ed82bcfa2a3c034e91513ef58543fb322e4de259c1a6d6d2010ebcbf. This digest freezes only allocation identities below. No full statements/assertion-body corpus digest or Base/final rate currently exists. Freeze actual original bodies, setup and saved-file assertions in existing test entrypoints before Base measurement. Retain original17/7 bodies and named adaptations. No injected re-fetches/coercions. Successful adapted gate remains separate from unmodified correct-file metric. Every attempt stays denominator, deliberate divergence counts as non-success, unexpected failure/missing identity/assertion/unreviewed divergence fails CI. Finite deliberate refusal cases have independent actionable-error tests, not fake successful-file credit.

Saved-file assertions must reopen through both binding/native relevant paths and verify semantic contents plus qualified owner/relationship/grid identities. visible contracts add deterministic rendering. XML cases verify exact unrelated opaque bytes and qualified schema facts, exports verify format/page/selection/diagnostic counts, lifetimes verify both bytes and generations. Source/build/interpreter/oracle versions and final body digest accompany Base/final rates. Final corpus runs once on integrated200..210, not provider worker copies. These unrun gates do not block design approval.

### Exact allocation IDs

- W-BASE-01: documents.opening-a-document
- W-BASE-02: documents.really-opening-a-document
- W-BASE-03: quickstart.opening-a-document
- W-BASE-04: quickstart.adding-a-paragraph
- W-BASE-05: quickstart.adding-a-table
- W-BASE-06: quickstart.table-cell-text
- W-BASE-07: quickstart.held-row-two-cell-assignment
- W-BASE-08: quickstart.table-iteration
- W-BASE-09: quickstart.table-style
- W-BASE-10: quickstart.adding-runs
- W-BASE-11: text.horizontal-alignment
- W-BASE-12: text.indentation
- W-BASE-13: text.line-spacing
- W-BASE-14: text.paragraph-spacing
- W-BASE-15: text.pagination-properties
- W-BASE-16: text.font-name-and-size
- W-BASE-17: text.font-tristate-and-underline
- P-BASE-01: hello
- P-BASE-02: bullet
- P-BASE-03: textbox
- P-BASE-04: picture
- P-BASE-05: shapes
- P-BASE-06: table
- P-BASE-07: text-extraction
- W-XML-01: paragraph
- W-XML-02: run
- W-XML-03: table
- W-XML-04: cell
- W-XML-05: section-properties
- P-XML-01: paragraph
- P-XML-02: run
- P-XML-03: table
- P-XML-04: cell
- P-XML-05: shape-kinds
- P-XML-06: text-body
- P-XML-07: slide
- P-XML-08: layout
- P-XML-09: master
- W-LIFE-01: table-held-row-after-cell-text
- W-LIFE-02: table-ancestor-after-grid-edit
- W-LIFE-03: hyperlink-after-unrelated-link-edit
- W-LIFE-04: removed-duplicate-link-no-rebind
- W-LIFE-05: manual-vs-captured-story-item
- W-LIFE-06: first-cause-multiple-edits
- W-LIFE-07: noop-failure-overflow-and-weak-retirement
- P-LIFE-01: title-subtitle-ancestor-survival
- P-LIFE-02: notes-ancestor-survival
- P-LIFE-03: layout-after-add-slide
- P-LIFE-04: master-theme-proxy-survival
- P-LIFE-05: table-after-row-column-edit
- P-LIFE-06: removed-shape-no-id-rebind
- P-LIFE-07: gradient-stop-duplicate-insert-delete
- P-LIFE-08: cross-scope-first-cause-and-foreign-owner
- P-LIFE-09: noop-failure-overflow-and-weak-retirement
- W-AREA-01: style-create-delete-kind
- W-AREA-02: style-cascade-and-language
- W-AREA-03: lists-numbering-restart
- W-AREA-04: paragraph-keep-and-tabs
- W-AREA-05: sections-orientation-margins
- W-AREA-06: header-footer-distinct-stories
- W-AREA-07: footnote-endnote-scope
- W-AREA-08: table-indent-look-style
- W-AREA-09: table-grid-merge-split
- W-AREA-10: picture-add-resize-inventory
- W-AREA-11: bookmark-heading-fragment-link
- W-AREA-12: external-link-retained-format
- W-AREA-13: content-control-edit
- W-AREA-14: math-expression-loss-report
- W-AREA-15: template-nested-conditions
- W-AREA-16: checked-validation-diagnostics
- W-AREA-17: core-app-custom-properties
- W-AREA-18: scope-text-and-format-export
- W-AREA-19: caller-font-fit-pdf-render
- W-AREA-20: paired-textbox-physical-replacement
- P-AREA-01: fresh-metadata-all-layout-geometry
- P-AREA-02: font-baseline-spacing-scripts
- P-AREA-03: numbered-bullet-details
- P-AREA-04: gradient-live-stops
- P-AREA-05: pattern-and-path-fill-pixels
- P-AREA-06: picture-fill-owner-pruning
- P-AREA-07: solid-theme-alpha-brightness
- P-AREA-08: cell-text-anchor-banding-style
- P-AREA-09: core-app-custom-sections
- P-AREA-10: movie-poster-lifecycle
- P-AREA-11: jpeg-tiff-multipage-selection
- P-AREA-12: notes-blank-default-handout
- P-AREA-13: pdfa-and-odp-loss-reports
- P-AREA-14: alt-title-decorative-qualification
- P-AREA-15: flip-rotate-native-order
- P-AREA-16: group-ungroup-id-zorder
- P-AREA-17: nested-nonuniform-affine-refusal
- P-AREA-18: glue-move-delete-regroup
- P-AREA-19: issue323-connector-scaled-group
- P-AREA-20: picture-placeholder-identity-crop
- P-AREA-21: run-internal-link-highlight-smallcaps
- P-AREA-22: align-visual-selection-slide
- P-AREA-23: distribute-gap-ties-cardinality
- P-AREA-24: footer-title-date-first-zero
- P-AREA-25: master-layout-theme-apply
- P-AREA-26: all-transitions-retained-sound
