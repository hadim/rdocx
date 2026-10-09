# F-X202, Complete Word tables pictures and internal links

**Status**: draft
**Sprint**: S91
**Size**: L
**Depends on**: F-X194, F-X198

This canonical draft is not approved for implementation.
Both prerequisites remain unfinished. Do not claim or start this story until
their reviewed integration and the decisions below are complete. Sources and
citations were checked against canonical head
`360b5b1cf77d19e7fe5c4cc26152dd55a00d12c1` and the frozen X198 worker.

## Problem

Issue305 requests complete everyday tables, pictures and internal links, plus
synchronized scoped text-box edits. Python exposes checked merge primitives
and clone_row but lacks the requested empty-row, nested-table and table
formatting surface. Native `Document::clone_table_row` deliberately copies
content at `crates/rdocx/src/document.rs:19793`. Native table columns and whole
merge-region split have no corresponding public operations.

`Document::set_picture_size` at `crates/rdocx/src/document.rs:20672` deliberately
resizes every main-document picture sharing the relationship. The Python test
`crates/rdocx-py/tests/test_core.py:877` explicitly pins this behavior. The issue
demands one physical picture. `Paragraph::add_hyperlink_with_tooltip` at
`crates/rdocx/src/paragraph.rs:1104` sets anchor=None. F-X198's approved scoped
refusal safely prevents divergence but does not fulfill Issue305's required
successful edit of both Choice and Fallback copies.

## Spec reference

- `docs/hld/14-development-backlog.md`, F-X202 and complete Issue305 criteria.
- `docs/hld/10-bindings-spec.md`, Python API shape, The invalidation problem,
  handled loudly, and Native Word facade stability, specifically table
  topology, picture ownership, hyperlinks and checked scoped replacement.
- `docs/hld/08-rendering-spec.md`, Tables, Autofit, and The renderer's input,
  specifically Word table geometry and drawing placement.
- `docs/hld/04-opc-and-packaging.md`, The package, deterministic saves and
  invalid text refusal.
- `docs/hld/12-testing-strategy.md`, Test taxonomy, Binding tests and The Word
  render fidelity gate.
- `docs/hld/01-glossary.md`, Units, with existing truncating conversions.

## Approach

Use existing native and binding files. Inventory every advertised operation
against the actual claimed Base before tests or implementation. Reuse X194
signed removable indentation and X198 namespace-qualified physical-owner
discovery. Do not duplicate either prerequisite's implementation or intake.

### Tables

Proposed Python surface, final spelling subject to the open contract choices:

```python
Table.indent: int | None  # EMU, negative supported
Table.autofit: bool
Table.style: str | None
Table.first_row: bool
Table.last_row: bool
Table.first_column: bool
Table.last_column: bool
Table.banded_rows: bool
Table.banded_columns: bool
Table.add_row() -> Row
Table.insert_column(index: int, *, before: bool = True) -> None
Table.delete_column(index: int) -> None
Cell.add_table(rows: int, cols: int) -> Table
Cell.split() -> None
```

Native checked methods in existing document.rs take existing checked table or
cell physical location and logical grid index, rather than an unvalidated
caller-owned mutable cell detached from its table. New column operations and
split return Result and stage the document, validate topology, reopen and
publish once. Proposed shape:

```rust
pub fn insert_table_column(&mut self, table_index: usize,
    index: usize, before: bool) -> Result<()>;
pub fn delete_table_column(&mut self, table_index: usize,
    index: usize) -> Result<()>;
pub fn split_table_cell(&mut self, table_index: usize,
    row: usize, logical_column: usize) -> Result<()>;
pub fn add_empty_table_row(&mut self, table_index: usize) -> Result<usize>;
```

Confirm nested-table indexing against existing document-wide table inventory.
Python resolves ContentPath and checks revision before translating location.
Every topology edit bumps revision and returns fresh Row/Table handles where
promised. Held rows/cells become stale after structural mutation.

Native table formatting already exists at `crates/rdocx/src/table.rs:966`,
`:971`, `:998`, `:1021` and `:1028`. Bind these operations, including inversions
for noHBand/noVBand, without creating a forwarding native abstraction. Style
assignment must resolve a real table style display name or explicit style ID,
check type and refuse ambiguous names. Current Python table.rs:384 simply
writes the supplied string as an ID. Do not turn an unknown name into a guessed
slug or use a paragraph-style name resolver without type validation.

Empty add_row preserves last-row row and cell properties, widths and logical
grid occupancy, but creates required empty paragraphs instead of copying
text, relationships, nested tables or document-wide identities. Decide the
vertical-merge continuation rule from a pinned Word/python-docx control.
`clone_table_row` remains content cloning. Native `remove_table_row` at
document.rs:19846 already restarts continuing merges and preserves boundaries.

Columns are logical tblGrid columns, not row cell-vector indices. Build one
bounded grid projection from current rows, gridSpan and vMerge. At insertion,
copy the selected neighbour's width and properties. Expand spans crossed by
the boundary or create a new independent cell according to the reviewed
native rule. At deletion, shrink intersecting spans, remove only a span that
loses its final column, and keep tblGrid plus cell widths consistent. Respect
gridBefore/gridAfter or explicitly refuse unsupported malformed topology before
publication. Refuse deletion of the final logical column.

Split operates on the complete selected horizontal and vertical merge region.
It removes gridSpan/vMerge and materializes the original logical grid cells.
Keep content once at the existing merge origin and preserve cell properties
using the reviewed rule. Do not duplicate comments, bookmarks, images, control
bindings or opaque XML. An unmerged cell is a defined no-op. Namespace aliases,
producer children, comment ownership and unrelated table boundaries survive.
Existing checked span setters at table.rs:1267 and Cell span setter:1876 are
not substitutes for this whole-region operation.

Nested Cell.add_table binds existing checked native Cell::add_table_checked at
table.rs:1975, with positive counts, bounded extents and mandatory terminal
paragraph policy. Do not call the infallible add_table at1930 with unchecked
Python input.

### Pictures

Expose the existing PictureOptions vocabulary at document.rs:504 and checked
authoring at17624, including description, title, decorative, crop, inline or
square/tight/top-and-bottom/behind/in-front placement, anchor relativity and
size. Reuse the native validators. Reject nonfinite positions and crop values,
invalid XML strings, conflicting inline/anchor inputs and wrong Python types
before mutable access. Through wrapping is not silently promised by this issue.

```python
Document.add_picture(..., options: PictureOptions | None = None) -> Picture
Run.add_picture(image: str | bytes, *, width: int, height: int,
    options: PictureOptions | None = None) -> Picture
Cell.add_picture(image: str | bytes, *, width: int, height: int,
    options: PictureOptions | None = None) -> Picture
Document.pictures() -> tuple[Picture, ...]
Document.set_picture_size(picture: Picture, width: int, height: int) -> None
```

Picture is a frozen snapshot of physical owner, size, description/title,
decorative state, content type and owned bytes, carrying checked document
identity, revision and qualified source identity for mutation. Inventory
specifies source order and story owner, distinguishes two drawings of one
relationship and admits supported picture owners without conflating charts,
SVG extension relationships, groups or alternate representations. Frozen
snapshots never become live by reinterpreting their stored relationship IDs.

Native Run::add_picture at run.rs:500 and Cell::add_picture at table.rs:1686
accept only existing relationship IDs. Package-owned import must allocate the
image relationship on the actual story part, stage qualified owner insertion
and publish atomically. Reuse checked add_picture_to_story/insert_picture_to_story
at document.rs:17472/17496. A stale run/cell refuses without adding media.

The proposed set_picture_size signature is an explicit pre-1.0 source and
behavior change. Rename the old relationship-wide native operation to an
explicit relationship-sized operation only if compatibility retention is
approved. Rewrite the existing multi-drawing test to pin both intentional
contracts, with reviewed release/API migration notes. Never silently map an
ambiguous relationship ID to the first drawing. Preserve relationship-aware
replace_image, which changes the shared media target intentionally.

### Links

```python
Paragraph.add_hyperlink(text: str, url: str, *, tooltip: str | None = None) -> Run
Paragraph.add_internal_hyperlink(text: str, bookmark: str,
    *, tooltip: str | None = None) -> Run
Document.add_heading_hyperlink(paragraph: Paragraph, heading: Paragraph,
    text: str, *, tooltip: str | None = None) -> Run
```

For add_hyperlink(..., '#name'), dispatch to the same internal-anchor operation
instead of writing an external relationship. Actual URL fragments remain
external URLs. Native authoring writes w:anchor and no external relationship.
Heading links create a unique valid bookmark start/end around the checked
heading only when none exists, reusing a suitable existing bookmark. Stage
source link, destination bookmark, IDs and relationships together. Refuse
missing targets, invalid names, stale handles and unsupported cross-owner
boundaries atomically. Preserve tooltip and formatting with native HyperlinkSpan
at paragraph.rs:1104, without text-only replacement or marker stripping.

### Synchronized scoped alternate text boxes

Extend X198 physical containment and qualified MC ownership into a complete
source-splice transaction. Identify every supported Choice/Fallback counterpart
of the selected semantic paragraph or descendant cell, not merely every box
sharing text. Validate branch topology, mapped owner and selection boundaries
before mutation. Logical found/expected count belongs to the selected view,
while each physical counterpart must independently match the mapped edit.
Count mismatch and zero-match semantics remain as in X198.

Stage edits against original qualified ranges and apply ranges in descending
offset order. Preserve each representation's run formatting, comments, fields,
namespace closure and opaque children. Publish only after every copy and
companion closure passes validation/reopen. Refuse unsupported asymmetric
topology or ambiguous owner mapping without changing any bytes. Do not remove
X198's guard and splice only Choice. Genuine public paired-copy success is
required by this story, alongside existing refusal and count controls.

## Rejected alternatives

- Empty add_row implemented as clone_row, which copies document content.
- Column indices interpreted as vector cell indices, which breaks gridSpan.
- split clearing one cell's flags without restoring the vertical merge region.
- Relationship ID treated as picture identity, which violates Issue305.
- '#' URLs written as external file relationships.
- Removing the scoped refusal before a complete paired transaction exists.
- A new module or generic owner wrapper, when existing document and ContentPath
  routes already identify the supported physical owners.

## Test plan

**Test gate**: regression.

| Category | Test | Asserts |
|---|---|---|
| regression | test_issue305_table_surface_on_installed_base | Every missing requested Python method fails through fresh installed Base before implementation |
| regression | empty_row_and_nested_table_preserve_properties | Empty content, retained last-cell properties, nested table reopen and valid terminal paragraph |
| regression | table_look_style_and_layout_roundtrip | Six look flags, named style resolution, unknown/ambiguous/type mismatch, autofit/fixed and signed removable indent |
| regression | logical_column_edits_preserve_merged_grid | Before/after edges, horizontal and vertical spans, unequal widths, nested tables, final-column refusal, malformed topology rollback |
| regression | split_restores_entire_merge_region_once | Horizontal/vertical/rectangular merge, content once, owner IDs, unmerged no-op, opaque bytes |
| regression | picture_options_and_owned_run_cell_insertions | Every requested option/anchor, story-owned relationship, stale owner and invalid input atomicity |
| regression | resize_one_of_two_shared_media_pictures | Exactly one physical drawing changes, other extent/media unchanged, explicit legacy transition and stale snapshot refusal |
| regression | picture_inventory_preserves_physical_identity | Bytes/content type/size/alt, same relationship multiple owners, headers/cells/text boxes, deterministic order |
| regression | bookmark_and_heading_links_are_internal | w:anchor, no external relation, tooltips, unique heading bookmark, target collision and stale/invalid rollback |
| regression | scoped_paired_text_box_edits_publish_all_copies | Qualified Choice/Fallback success, multiple branches, descendant cell, unequal runs, no unselected changes |
| regression | paired_mapping_refuses_ambiguity_atomically | Asymmetric owners, opaque/foreign MC, aliases, count/no-op/stale rules, complete bytes and comments |
| round-trip | issue305_saved_documents_reopen_without_repair | Every item saved/reopened, native Word opening and render for geometry/links/pictures/merge topology |
| differential | issue305_word_and_python_docx_controls | Pinned native merge/split and picture rules, supported python-docx row/style/nested APIs |
| binding | issue305_typed_and_stale_handle_contract | Current wheel, strict mypy, stubtest, correct exceptions, fresh returned handles and held stale controls |

Reuse existing regression_test.rs, integration_test.rs only if the eventual
writer claim permits it, and current Python test_core/test_formatting_tables/
typing_smoke entrypoints. No new test binary. Existing references include
Python test_core.py:877, formatting-table tests:523/549/577, native integration
merge controls:8699 and row restart:11232, plus the integrated X194/X198 tests.
Pin Word and python-docx versions. Deterministic fonts are required for render
comparisons. Google Docs imports are best effort with documented direct-shading
and tight-wrap limitations, not inferred native acceptance.

## HLD impact

- `docs/hld/08-rendering-spec.md`
- `docs/hld/10-bindings-spec.md`
- `docs/hld/12-testing-strategy.md`
- `docs/hld/14-development-backlog.md`

Update Tables/Autofit and renderer input where accepted topology or drawing
behavior changes, precise native/Python API and invalidation contracts, the
complete verification matrix, and actual Issue305 scope. HLD07 is PowerPoint
inheritance and is not a Word table model document.

## Risk routing

- Parser/serializer, HLD04 The package and HLD06 Preservation strategy:
  namespace alias/foreign/rebound controls, schema-order assertions and exact
  raw producer subtree preservation through save/reopen.
- Units, HLD01 Units and CLAUDE deliberate truncation: negative indentation,
  lengths, image crop/anchor input bounds and unchanged conversion semantics.
- Layout, HLD08 Tables/Autofit: deterministic PDF/PNG and native geometry
  controls, intentional output changes declared before baseline updates.
- Public API, HLD10 and structural rules: explicit pre-1.0 resize migration,
  no new source file/module/trait, verified patched workspace publish dry-run
  with --allow-dirty while scoped and all archives below10MiB.
- PyO3, HLD10: fresh installed runtime and strict typing/stubtest, both WASM
  target checks, exclude binding Rust test binaries at workspace gate.
- External oracle: differential-testing skill, pinned versions and immutable
  source/native-saved/reopened receipts. Native conclusions require actual
  captures, not code-generated expected layouts.
- Archive measurement carriers: if required by exact source inventory gate,
  rerecord only actual affected tuples/README rows under the Cargo lease,
  preserving all inventory, provenance and tolerances. HLD15 Packaging applies.

## Hash harness

Additive methods should leave existing49 cases unchanged. A resize contract
transition, paired-copy success or topology/render behavior may affect cases
that explicitly exercise those operations. Enumerate actual affected entries
and independently review expected deltas before any baseline movement. Never
guess that existing snapshots or relationships are new per-picture identities.
Only the designated exclusive sprint story may write the harness baseline.

## Exclusive file claims and integration

Proposed native scope: existing document.rs, table.rs, paragraph.rs, run.rs and
only necessary existing oxml table/drawing serializers. Python scope: existing
document.rs/table.rs/paragraph.rs/run.rs, lib registration, _rdocx.pyi and
current test entrypoints. Confirm exact source scope after topology/oracle work.
Serialize document.rs and binding/stub overlaps with X200/X201/X203 and HLD
semantic integration. X204 consumes these APIs and owns general CLI commands.
Workers never write sprint totals. Keep X194/X198 integrated evidence as the
dependency prefix, and update this draft's citations at the actual claim Base.

## Implementation checklist

- [ ] Integrate reviewed X194/X198 and settle explicit API transition decisions.
- [ ] Pin native topology/row/picture controls and define unsupported admissions.
- [ ] Freeze full API/stub/source inventory and prove genuine public Base failures.
- [ ] Implement checked tables, picture identity/options, links and paired edits.
- [ ] Prove all criteria through reopen, render and full atomic/opaque controls.
- [ ] Run declared scoped, binding, WASM, package and hash riders.
- [ ] Complete independent zero-defect, zero-smell review and declared HLD work.
- [ ] Prepare validated worker handoff without sprint-ledger writes.

## Open questions

1. Approve the explicit set_picture_size target/signature change and migration
   of the current relationship-wide behavior. Decide whether that legacy
   operation remains under an explicit name, rather than an ambiguous overload.
2. Resolve logical-column insertion across a span and vertical merged empty-row
   continuation from native controls. Define split content/formatting and
   gridBefore/gridAfter admission before implementing the full projection.
3. Choose the final public picture snapshot selector, inventory scope and
   paired picture representation support without silently inventing identity.
4. Define heading-link placement/return handle and compatible internal dispatch
   for '#name'. Exact native target/bookmark admission is technical work.
5. Define the complete paired owner/paragraph correspondence supported by scoped
   edits. Asymmetric topology needs an explicit refusal boundary, while genuine
   native paired success remains mandatory.
6. Split implementation into dependency-safe children if the full native plus
   Python/oracle contract exceeds the approved L estimate. No criterion can be
   dropped merely to complete a smaller patch.

## PR317 synchronization reuse assessment

Immutable document.rs head3d056f46e97e9cc1caf2969b5096eeb6775a5fa0 offers
useful namespace-closed fragment edits, each physical replacement count
equal to the logical count at27195/27210, descending splice offsets at
27212/27215 and prepare/reopen before publication at27216/27229. Reuse
those mechanics with the existing F-X198 qualified owner router.

Do not adopt local txbxContent routing at8275/8292 or unqualified
AlternateContent/branch admission at8323/8335 and8419/8433. Opaque earlier
Choice content must not suppress a later qualified modeled owner. Paired
cell, nested topology and whole-box equality checks at27101/27130 and
8363/8370 currently precede expected-count/noop checks at27180/27191.
Preserve the existing proven count/noop/stale ordering before synchronized
preflight. Genuine success and unsupported atomic refusal both require
qualified multiple-choice/grouped/identical/foreign/opaque controls,
asymmetric box counts, nested cells and unselected-paragraph divergence.
Contributor claimed regression tests have not been independently verified
by this static source assessment. Do not claim local success from them.
