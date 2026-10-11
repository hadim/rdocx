# F-X216, Import complete slides and merge decks with source formatting

**Status**: approved
**Sprint**: S91
**Size**: L
**Depends on**: F-X206, F-X215
**Issues**: 347, 348, 349
**Test gate**: regression
**Implementation**: not claimed, no code or execution

## Problem

`crates/rpptx/src/lib.rs:3021` validates one source slide and layout, preflights relationships, clones the destination and commits after copy. Its default explicitly uses the destination theme. `preflight_slide_import` at5867 refuses comments, and `preflight_import_scope` at5904 refuses internal chart/OLE/other-slide relationships. `copy_slide_from_in_place` at3208 separately refuses modern comments without new IDs. Dropping these checks without transferring the full package graph would create broken decks.

Current Python `slides.import_slide` at rpptx-py/slide.rs605 publishes native mutation before its legacy revision bump and returned-object allocation at664. New merge factories must use the completed F206 transactional owner engine. `rpptx-cli/main.rs:19` has no requested slide import command. Existing source includes notes/media/SmartArt copying and comment author structures at lib.rs615 and2303 that must be reused once.

## Spec reference

- docs/hld/04-opc-and-packaging.md, The package, Relationship types and owned graph transfer.
- docs/hld/06-presentationml-model.md, Relationship remapping, Adding a slide and source order/z-order.
- docs/hld/08-rendering-spec.md, Why Group is the whole design and inherited layout/transform output.
- docs/hld/09-charts-spec.md, Why a chart needs three parts.
- docs/hld/10-bindings-spec.md, Python API shape and checked owner publication.
- docs/hld/12-testing-strategy.md, The deck corpus and Binding tests.
- docs/hld/14-development-backlog.md, F-X216 at7571, exact size/dependencies/regression gate.

## Approach

Retain the existing native import_slide signature/default behavior. Add concrete options and batch entrypoints in existing rpptx/lib.rs, with no new module/file: `ImportSlideOptions { layout_index: Option<usize>, insert_index: Option<usize>, keep_source_formatting: bool, keep_sections: bool }`, and `import_slides(source, indexes: &[usize], options: &ImportSlideOptions, existing_slide_map: &[(usize, usize)]) -> Result<(Vec<usize>, Vec<String>)>`. Empty selection is an exact no-op. Missing selection in the append-all facade selects source order. Explicit selection preserves caller order and rejects repeated indexes before publication to avoid ambiguous jump mapping. Indices and insertion positions are checked before narrowing. A single-slide options method delegates to the same real batch operation, not a separate transfer engine. Returned indices are immediate native results, not mutable index aliases in Python.

Preflight the entire requested source graph before mutation. Allocate all destination slide part names/IDs and the source-to-destination slide map up front. Validate supplied existing map against the actual source and destination presentations, including duplicates/conflicts. Copy charts, embedded workbooks, colour/style parts, OLE embedded bytes and previews, comments/authors, notes and existing media/SmartArt using an explicit visited part map. Resolve internal targets through the owning OPC part and content type, not string concatenation. Preserve external relationships and reject unsupported internal types with exact type and owner path before commit. Each recognized graph part is processed once, cycles use visited identities, and opaque reachable XML receives only the namespace-owned relationship remapping required for new scope.

Comments require deterministic collision-free new author/comment/reply/thread identifiers and all matching parent/person/author references. Merge authors by complete supported semantic identity, not display name alone. Distinct equal-name authors stay distinct. Preserve timestamps, text, extension payload and comment order. Validate collaboration model after staging. Classic and modern comments have separate exact identities and tests. No human choice is needed to replace colliding IDs safely.

Map slide jumps to another imported slide or explicit existing_slide_map target. Unmapped jumps release only actual owned hyperlink action and unused relationship scope, retain click sounds/shared references as required, and append a diagnostic naming source slide/owner/target. Notes' normal back-link maps to the new slide, while genuine cross-slide jumps follow the same mapping. Foreign r:id/action lookalikes remain byte-identical. Reuse existing qualified relmap, release_hyperlinks and relationship pruning. Never blanket-delete relationships still referenced by another retained owner.

With keep_source_formatting false, retain current destination-theme/name-match or explicit destination layout behavior. With true, transfer source slide layout/master/theme dependency closure and preserve inherited fonts, colours, placeholder geometry, notes and rendering. Deduplicate by complete semantic graph equality after resolving relationship identities. Ignore only part names and assigned IDs that are precisely remapped. Different theme/geometry/opaque metadata is not equal merely because layout/master names match. The first shared source layout imports exactly one layout/master/theme and later slides reuse it. Allocate presentation/master/layout relationship sets and schema ordered ID lists coherently. Repeated import reuses equivalent graphs without duplicating a master per slide.

When keep_sections is true, selected source slides preserve section identity/order as contiguous selected blocks, generating noncolliding destination section IDs. A noncontiguous selection does not reorder slides to recover original sections. Insert imported blocks at the requested destination position and retain destination sections outside that block. An unspecified section option leaves destination section behavior unchanged. Tests cover duplicate section names and collisions. Section/notes/header/comment/table style dependencies are copied only where actual source ownership requires them, without overwriting the destination global table-style definitions.

Stage one destination candidate, checked all source constraints/package validation and Python returned-object allocation before publication. F206 prepare/capture_prepared/commit owner mechanics provide exact public operation causes, stable unrelated handles and retirement only for changed collection topology. Source presentation and package remain unchanged. Same-deck import uses a stable snapshot. Batch failure on the last selected slide rolls back every earlier staged copy. An ordinary no-op never advances serial or creates orphan scopes.

Python extends `slides.import_slide(slide, layout=None, index=None, *, keep_source_formatting=False)` and adds `slides.import_slides(source, indexes=None, *, layout=None, index=None, keep_source_formatting=False, keep_sections=False, slide_map=None) -> tuple[tuple[Slide, ...], tuple[str, ...]]`. Add the directly needed `slides.import_slide_with_diagnostics(...) -> tuple[Slide, tuple[str, ...]]` companion with the same checked single-slide arguments when the old return shape cannot carry drop diagnostics. All new Slide objects are allocated from prepared native ownership before publication. Accepted native options cover sections and explicit jump map. Preserve old positional calls and old return type on the single-slide method. Stubs, exports, foreign input/type checks and causal labels are complete. No missing diagnostic channel is hidden in logging.

CLI exact required spelling: `rpptx slide import DEST SOURCE [--slides 1,3-5] [--at N] [--keep-source-formatting] -o OUT`. Existing commands/main.rs carry the implementation. CLI ranges and insertion positions are one-based, matching existing slide range conventions, with append as default and --at1 inserting before the first slide. Parse ranges, reject duplicates/out-of-range and never overwrite either input. Default selects every source slide. Diagnostics go to the existing structured/human channel. Save through the established atomic output path and validate before write.

Existing carriers: rpptx/lib.rs, rpptx-oxml comments.rs/presentation.rs/relmap.rs and only necessary existing namespace/shape/timing carriers, oxml-opc existing package and relationships carriers, rpptx-py slide.rs/presentation.rs existing exports/stubs, rpptx-cli main.rs/commands.rs, and existing inline/integration/Python tests. F215 owns chart read/replace semantics. This story copies its accepted graph without reauthoring chart data. F206 owns group/live-handle operations. No new product file/module/crate/dependency or generic copy framework.

## Rejected alternatives

- Importing only chart caches leaves Edit Data broken.
- Deduplication by layout name or author display name loses source semantics.
- Commit-per-slide makes a failed merge partially observable.
- Copying all source package parts invents unused objects and unrelated presentation state.
- Keeping dangling jumps or refusing the entire batch for one unmapped jump contradicts explicit issue acceptance.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| regression | `fx216_import_full_relationship_graph_is_transactional` | Separate chart/workbook/style/colour, OLE/preview, classic/modern comments/authors and slide-jump cases save/reopen and package-validate, late unknown type yields no mutation |
| round-trip | `fx216_foreign_relationship_tokens_and_graph_sharing_preserved` | Alias/local rebound namespaces, raw unknown nodes/child order, shared media and sounds, part collisions and source package byte retention |
| regression | `fx216_batch_jumps_map_or_drop_with_diagnostics` | Forward/backward/imported/existing/self/notes jumps map exactly, absent targets release owned links only with explicit diagnostics, nonselected source slides remain unchanged |
| regression | `fx216_source_formatting_reuses_equal_layout_master_theme` | Different-theme import resolves source fonts/colours/placeholders and copies one shared layout/master/theme, repeated calls reuse equal graphs, unequal names/opaque payload are not collapsed |
| regression | `fx216_merge_sections_count_order_notes_and_media` | Two pinned corpus decks, all and selected imports, count/order/notes/media reuse, section collisions and noncontiguous selection, same-deck and empty selection |
| regression | `fx216_cli_slide_range_and_insertion_write_new_file` | Actual CLI --slides1,3-5 and --atN, output new, source hashes unchanged, error on invalid ranges, diagnostics and exact slide order |
| regression | `fx216_installed_returned_slide_owners_and_refusal_atomicity` | Current provider factories before publication, stable unrelated owners, retired changed collection descendants, foreign inputs/overflow/Py allocation failure and no-op preserve source/revisions |
| differential | `fx216_imported_native_source_formatting_matches` | Source versus imported deterministic geometry/fonts/colours/render, comparison metric fixed before measured output, wrong destination-theme negative differs |

Native PowerPoint opens each copied relationship kind and merged source-format deck without repair, normal save/reopen retains graph and comments/authors, actual source/imported slide renders are compared. Record exact application build and source/output hashes. Existing corpus manifests bind the actual two decks and their dependencies. Missing corpus/native work is uncredited, never a silent selfskip. Source-built fixtures supply all relationship kinds without new checked-in binary assets. Pin python-pptx1.0.2 only where its public model answers the comparison. Fresh binding wheel, exact installed nodes, strict typing/stubtest and CLI integration must execute against the reviewed current union. Native graphical evidence is not replaced by successful ZIP validation.

## HLD impact

- docs/hld/02-scope-and-non-goals.md
- docs/hld/04-opc-and-packaging.md
- docs/hld/06-presentationml-model.md
- docs/hld/08-rendering-spec.md
- docs/hld/10-bindings-spec.md
- docs/hld/12-testing-strategy.md
- docs/hld/14-development-backlog.md

## Risk routing

Parser/serializer04/06: owned relationship remap, content types, schema ID lists, lossless opaque capture and namespace controls. Public API10: stated additive option/batch/diagnostic surface, old import signature/default unchanged, verified affected archives and10MiB limits. Bindings10: fresh installed provenance, strict types/stubs and both WASM graphs. Layout08: deterministic fonts and actual source-format equivalence. External oracles: pinned native PowerPoint/python-pptx, source-built/native comparison with declared tolerances. Dependency03: no reverse low-level facade dependency and no new generic graph framework. No new feature flag, bundled asset or version change. All ordinary scoped gates remain mandatory.

## Hash harness

Existing defaults and samples expected unchanged. Source-format option and expanded accepted graph alter only the explicitly selected imported output. Any existing harness/corpus change must be separately attributed, declared and reviewed in a labelled behavior commit before baseline movement. A file move is not proposed.

## Implementation checklist

- [ ] Approve complete options/diagnostics/section selection contract and wait for F206/F215 done.
- [ ] Preserve genuine Base missing graph/formatting/merge controls with valid setup.
- [ ] Implement checked whole-graph transfer, IDs/authors/jumps and graph-based deduplication.
- [ ] Implement batch/native/Python/CLI atomic publication and all returned owners/stubs.
- [ ] Prove corpus source-format and native PowerPoint repair-free open/render/save/reopen.
- [ ] Complete scoped current-source gates, HLD and independent zero-finding review.

## Open questions

No blocking product ambiguity. Defaults remain destination theme, append, no sections unless requested. Modern comment ID remapping is mandatory, not optional. Stable existing-slide mapping is explicit input and validated. F206 and F215 are unfinished in the current registered backlog, so draft design-all-first is permitted but implementation must not start before actual dependency completion.

## Batch approval, 2026-10-11

The rolling intake instruction authorizes the full linked issue acceptance.
ROOT reviewed the source, dependencies, test gate, risk riders and HLD list.
The stated defaults are adopted. No blocking product question remains.
Implementation waits for completed dependencies and an exclusive resource wave.
