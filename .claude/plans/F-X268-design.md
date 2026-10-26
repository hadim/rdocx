# F-X268, Bound Word owner inventory and unchanged package staging

**Status**: approved
**Amendment note**: Codex root approved the refresh4 mechanism 8 contract on 2026-10-25 after the user answered Q24. The baseline is `d1f42516eb3747442fe4304a6331eab946dc9a0e`. The scope approved through refresh3 stays approved. F-X268 stays unclaimed. F-290 keeps its existing approved implementation and review. No gate pass is claimed.

**Current implementation readiness**: In progress, owner codex on work/f-x268-codex. All eight mechanisms are implemented. The unclaimed and not-started statements in the dated approval snapshots describe those earlier checkpoints. Local combined native, Python and release fallback checks pass. Whole-feature pass 1 identified D1 context-transition runtime coverage and D2 case-equivalent physical-part fallback. Both are addressed in one remediation batch. The two actual native binding regressions and ten affected installed Python cases pass. Independent whole-feature pass 2 is clean with zero defects, smells and nitpicks. Scoped local completion is ready for the final sprint acceptance integration checkpoint. Final release performance, memory, Word, hash and full-union riders remain pending.
**Sprint**: S95.4
**Size**: L
**Depends on**: F-X266, F-X267
**Estimate**: 12 to 21 developer-days for the combined scope, as a provisional Root planning judgment after the historical 10 to 17 plus a provisional 2 to 4 for this increment. Reassess it after decomposition. It is not a measurement, not a capacity promise, and not a new promise. Actual work is unknown. Do not add it to the historical seven-story figure of 27 to 41 developer-days. The 10 to 17 figure stays historical for the scope before PR422. The earlier 7 to 12 figure stays historical for the scope approved before the 419 and 420 amendment.
**Amendment**: Codex root approved the 2026-10-24 refresh2 extension. That approval stays. Codex root approved the 2026-10-24 refresh3 extension after reading the full corrected contract and the four-file delta. The canonical HEAD at that approval is `d6cefc239ab7de0d4dae63e13d730da558d3a3a5`, plus this working draft. This record is that Root review. It is not a self-review. The prior approved scope stays approved. Codex root approved the refresh4 extension on 2026-10-25 at baseline `d1f42516eb3747442fe4304a6331eab946dc9a0e`. No implementation is underway. No gate pass is claimed.

## Problem

Direct body insertion and later package staging still rebuild more of a Word document than the changed owner, and a retained namespace owner can leave a body insertion unsavable only after the model has already changed. The current native methods are infallible and mutate before they return:

- `Document::add_paragraph` at `crates/rdocx/src/document.rs:26608`
- `Document::add_table` at `crates/rdocx/src/document.rs:27239`
- `Document::insert_paragraph` at `crates/rdocx/src/document.rs:27272`
- `Document::insert_table` at `crates/rdocx/src/document.rs:27292`

Python publication already goes through `prepare_word_revision`, `publish_api_candidate`, `publish_owner_candidate`, and `publish_form_candidate` in `crates/rdocx-py/src/document.rs`. Those paths are the binding consumers of the inventory. They are not a new public API.

The cache and piece facts below are copied from the pinned PR417 and PR418 trees. They are explained here and have not been executed on this branch.

## Spec reference

- `docs/hld/14-development-backlog.md`, F-X268. The test gate is regression.
- `docs/hld/03-architecture.md`, Owned publication and causal paths.
- `docs/hld/04-opc-and-packaging.md`, The package, and Checked Word operation source ownership.
- `docs/hld/10-bindings-spec.md`, Python Word document operations.
- `docs/hld/12-testing-strategy.md`, Binding tests, and Word document-operation parity controls.
- `docs/hld/02-scope-and-non-goals.md` is unaffected. This story does not edit it and it is not an HLD impact.

## Approach

One story, eight serial mechanisms. The first four stay as approved. The fifth and sixth are the approved 2026-10-24 refresh2 extension. The seventh is the approved 2026-10-24 refresh3 extension. The eighth is the approved 2026-10-25 refresh4 extension. It is the same F-X268 story. It is not a ninth story and it adds no stable public API, dependency, or module. The Q19 and Q21 behavior is its own labelled commit. Performance and memory steps are separate commits. No stack is reimported. The reviewed F-X266 and F-X267 source stays. F-288, F-289, and F-290 keep their approved contracts. Their fact-changing publishers are named in the consumer map. The F-289 and F-290 publication proofs and related-owner proofs stay. They are not regressed by the seventh or eighth mechanism.

Captured increment boundaries, not the stacked diffs against main:

- Old PR415 `0fbf3e96709198dbd01b9e810181109d106d9f68` to new PR415 `2839e22c14f96aeaeab5245581b5681509774559` is a diverged endpoint tree of 9 paths: `README.md`, `crates/rdocx-oxml/README.md`, `crates/rdocx-oxml/src/document.rs`, `crates/rdocx-py/src/document.rs`, `crates/rdocx-py/tests/test_core.py`, `crates/rdocx/src/document.rs`, `docs/hld/03-architecture.md`, `docs/hld/10-bindings-spec.md`, `scripts/readme_doctests.py`.
- PR417 `bdf49f0005d1d15078b76b5da83c5f3e2b8cce12` has six own commits after the new PR415 head. Its own tree is 14 paths: `README.md`, `crates/rdocx-oxml/README.md`, `crates/rdocx-oxml/src/borders.rs`, `crates/rdocx-oxml/src/text.rs`, `crates/rdocx-py/src/document.rs`, `crates/rdocx-py/src/lib.rs`, `crates/rdocx-py/src/paragraph.rs`, `crates/rdocx-py/src/run.rs`, `crates/rdocx-py/src/table.rs`, `crates/rdocx-py/tests/test_core.py`, `crates/rdocx/src/document.rs`, `crates/rdocx/src/lib.rs`, `docs/hld/03-architecture.md`, `scripts/readme_doctests.py`.
- PR418 `dba496fcd8b4d482995a23ab81d27ad2bfcbf5e7` has six own commits after merge `0912a40e19950b223745ef83e8164266b4c36b66`. The merge parents are the PR417 head and PR416 head `768f8c109bd43fb9c1f6ecdfe2d913382bcc0368`. Its own tree is 8 paths: `README.md`, `crates/rdocx-py/src/document.rs`, `crates/rdocx-py/src/lib.rs`, `crates/rdocx-py/tests/test_core.py`, `crates/rdocx/src/document.rs`, `crates/rdocx/src/lib.rs`, `docs/hld/03-architecture.md`, `scripts/readme_doctests.py`.

Own increments only, still candidates and not accepted code:

- PR419 `9916bcdc01bc4acc065ae45afa8bd8ae07a2d30c` follows PR418 `dba496fcd8b4d482995a23ab81d27ad2bfcbf5e7`. It has 3 own commits and 7 paths. The own diff SHA-256 is `0de97b8eb0df312584bb86976676eb8fc66d7be694dd96a99830b0647836cc53`.
- PR420 `37851676af3e82a0500c4743ef86da417950d2c3` follows that PR419 head. It has 5 own commits and 11 paths. The own diff SHA-256 is `5b94cfeb7854c8d86a5f76c1d6a32424be8faa6d6af1b565af6eeb6fc28da728`.
- PR421 `ab038a21bb88a38d46c8f640587c3a20b57f6770` follows that PR420 head. It has 7 own commits and 10 paths. The own diff SHA-256 is `50c390ff8038d69b95d19510122ab51d2fd3c37b30f5b5d79a89dc610f805e35`. The file is `/private/tmp/s95.4-refresh3/pr-421-own-increment.diff`. The stacked diff against main is not this increment. The 10 paths are `README.md`, `crates/rdocx-oxml/README.md`, `crates/rdocx-oxml/src/borders.rs`, `crates/rdocx-oxml/src/document.rs`, `crates/rdocx-oxml/src/text.rs`, `crates/rdocx-py/src/document.rs`, `crates/rdocx-py/tests/test_core.py`, `crates/rdocx/src/document.rs`, `docs/hld/03-architecture.md`, and `scripts/readme_doctests.py`. `content_control.rs`, `field.rs`, and `settings.rs` are not in those 10 paths.
- PR422 `cf7bf55edfab1e0617f6fe35810355cb02126018` follows that PR421 head on a linear first-parent chain of 5 own commits. The compare is ahead by 5 and behind by 0. The merge base is the PR421 head. The own diff is 60325 bytes and its SHA-256 is `ee0a8ce239fafea714c7bc1a49601fab4f2c864c3931b8ff4c98c3fb9dce189c`. The file is `/private/tmp/s95.4-refresh4/pr-422-own-increment.diff`. The stacked diff against main is not this increment. The 5 paths are `README.md`, `crates/rdocx-py/tests/test_core.py`, `crates/rdocx/src/document.rs`, `docs/hld/03-architecture.md`, and `scripts/readme_doctests.py`. Comments and reviews are 0. PR422 stays an unmerged candidate and is not accepted code.

Actions run `40092396584` on the PR419 head is donor evidence only. Word fidelity job `120867623753` failed while downloading LibreOffice 26.2.5.2, with TLS `CERTIFICATE_VERIFY_FAILED`, before any fidelity comparison. The evidence file was absent. CI gate job `120878395382` failed because that Word job failed. The hash harness on that same donor run succeeded. No Word comparison was performed. A later green PR420 check is not a PR419 pass. The PR419 head did not move. A later Actions run `40103891571` concluded success, including Word fidelity job `120901503513` and CI gate job `120914750403`. That later run stays donor evidence. It is not local acceptance. PR421 checks were incomplete at cutoff 2026-10-24T17:02:28Z. No PR421 acceptance is inferred. At cutoff 2026-10-24T23:05:39Z the PR422 head has 19 completed donor check runs, with success or skip conclusions, and the donor CI gate succeeded. The legacy combined status endpoint still records `pending`. That field is recorded literally. It is not a claim that CI is incomplete and it is not acceptance. Donor CI is not our pass. Final fidelity on the own union stays mandatory. No product hash or render defect is inferred from those logs.

Donor timings and the donor 68 MiB claim are unverified. Reported 57 and 18 call timings are not copied. They are not added to the historical Q23 set of 73 flat calls, the current Q24 set of 79 flat calls, or the 5 linear calls. README package text, an M5 Max figure, and an allow-dirty note are not copied. README and crate README numbers in those trees are not copied. The donor figure is not accepted.

### 1. Qualified body splice and namespace replay

Amend the integrated F-X266 qualified body splice so an unrouted paragraph facade position still uses the checked route. Nested namespace replay is scoped to the exact current XML, model, and namespace-owner facts. A proof is not reused after an edit. Compatible positive owners stay. Do not adopt the donor's incidental redundant saved-namespace byte delta. Existing package bytes stay unchanged except for the requested edit.

### 2. Owner pieces

Keep one piece inventory per body block and per related physical part. Reuse on equality needs serialization and fact provenance. Semantic `PartialEq` alone is not proof. Cover the existing field source for form, span, derived parent cache, default, current, and typed nested cache, the tab source occurrence, and bookmark prefix facts, through the public and native publishers that change them. Reconstruct the same owners, parents, routes, projections, read facts, order, scope births, generations, and retired set as the checked full old inventory. Grammar is the exact qualified physical grammar, including independent text box ownership. Before any partial publish, an unsupported shape, a duplicate physical part, an invalid relationship, or a context change uses the old full rebuild. An error does not shift semantics.

### 3. Opened package staging

Share the opened package, the parsed main part, the retained owners, and the check record with staged clones. Reopen resets that baseline. Reuse only checked unchanged main bytes plus model serialization, and only the relationship, section, drawing, namespace, media, related-part, signature, and embedded facts that were checked. `OpcPackage::reread` keeps its current `trusted_baseline` precondition. Public part maps are not assumed immutable. A save-equivalent compare requires exact bytes and the same errors in the same order. A new or in-memory document falls back to the old ZIP compare. `render_template`, `compare`, and `rebuild_toc` keep the old `to_bytes` error order. A related part that changed is still checked when the main part is unchanged. A failed cache check is not success. A scope-preserving property edit, callback, or direct candidate assignment invalidates the facts it changes even when the generation is unchanged. An opaque change is not ignored. Legacy form defaults, current values, placeholders, custom XML, protection, permissions, and mail-merge stores and settings compose. Dead owners are not revived.

### 4. Binding publication

Python publication uses the prepared inventory. It allocates every result object and revision token before commit. A no-op and a refusal keep the old bytes, revisions, and live handles. Conservative publication still retires prior scopes. Append keeps only eligible owners. An owned old snapshot stays old. A returned handle uses the prepared proved route or the old fallback. It does not start a new whole-document walk.

### 5. Eligible in-memory reopen

`reopen_prepared_staged_with_limits` may reuse OPC reread only for a document opened from bytes, with `PackageReadLimits::UNBOUNDED`, and with trusted provenance. A new or in-memory document means there is no trusted opened-byte baseline, so that commit keeps the checked ZIP path. The same document may qualify after it is legitimately saved and then reopened from those bytes. A Flat OPC first commit, a limited reopen, a failure hook, and any unproved context keep the checked ZIP path. A previously authored document is not excluded forever. The carried main parse, nested namespace owners, and canonicalization record, including a cached error, move only when the main bytes and the original contextual dependencies are exact. Relationship, identifier, section, and related-part facts stay checked. A per-part identifier definition cache may replay only when duplicate ordering and errors match, including a retyped part. Exact bytes alone are not enough when relationship or type context changed. Both reopen routes carry the mechanism 6 scans only when frame, context, and bytes are proved exact. A debug whole-path comparison is internal agreement. It is not a substitute for release regressions.

### 6. Incremental direct-body addressing

`to_xml_with_body_spans` must reproduce the exact existing `to_xml` bytes, the direct-body order, and the head, tail, and final section context. A block source, owner, item, cell, or text box fact is reused only when the bytes match and the namespace, relationship, owner grammar, and context match. A root or frame change, and every changed fact, is invalidated. Duplicates match position and occurrence. There is no ordinal alias after prefix, suffix, or middle alignment, or after a reorder. Preserve the F-289 corrected accepted physical providers, per-part physical ordinals, and independent story scope for fields and controls. Preserve default, current, glyph, font, placeholder, and protection facts, and F-290 metadata. Do not overwrite `content_control.rs` from the donor. The existing direct id lookup and `collect_control_ids` may use an index while they preserve uniqueness, errors, and physical order. `direct_body_item_location` must equal the full story snapshot lookup for zero, one, and many items, nested controlled content, tables, cells, text boxes, and moved, removed, or repeated blocks, before and after both reopen routes. An unsupported independent block scan takes the checked full scan. An error is never an empty success. Whole-main hash and serialization stay separately counted and disclosed. A debug whole-path comparison is internal agreement. It is not a substitute for release regressions.

### 7. Main-part body-child staging

This is the approved 2026-10-24 refresh3 extension. It extends mechanisms 5 and 6. It does not replace them. The private design candidate is `MainPartBlocks` inside the existing `crates/rdocx/src/document.rs`. It is not a new file, module, or dependency. Its fields are the head bytes, the root range inside that head, the tail bytes, the kept block scans, an optional flushed part, a frame `OnceLock`, and nested namespace owners. A comment in the candidate says a child equal to a kept child that holds no field serializes to its kept bytes, and a child that holds a field is read again. That rule is the contract. `PartialEq` on a field-containing block is not reuse proof.

`CT_Document::body_frame_xml` is `#[doc(hidden)]` and returns `Result<(Vec<u8>, Vec<u8>)>`. The pair is the bytes written before the first body child, through the body start, and the bytes written after the last body child. That helper runs before the full writer declares `w14` on the root. The stored head used by `MainPartBlocks` then applies the same conditional `w14` declaration the full writer applies. `CT_Document::body_item_xml(&self, item: &BodyContent) -> Result<Vec<u8>>` is `#[doc(hidden)]`. `BodyContent::RawXml` is a passthrough without model admission, and the helper does not reject raw XML. The helper matches the whole writer on admission, bytes, error, and binding context. The item need not belong to the body. It writes the item under `root_binding_scope`. Parent binding context is exact. Inherited body bindings, unknown source bytes, root bindings, and frame bindings stay. A changed root invalidates the kept frame and the kept blocks. An alternate Word prefix on the document root is such a change. A fragment spliced into that part uses the checked whole-part read.

`BodyItemOwners` gains `holds_field: bool` as its 10th field on the final callable contract. The detector is a conservative byte-word scan for `fldSimple` and `fldChar`. It covers prefixed aliases and the default namespace because it matches those words and does not require one prefix. False positives are allowed. False negatives are not allowed. A positive result refuses reuse of that child by value. The implementation serializes or reads the changed child, or the field-bearing child, anew. Other proved blocks may still be reused. The checked whole-part fallback runs only when the child proof, the frame proof, or the context proof is unavailable. A positive field result does not by itself require that whole-part fallback for every field-bearing block. Field source, run span, nested cache, tab occurrence, and bookmark prefix publishers already named in the consumer map still invalidate the facts they change. Physical duplicates keep occurrence, order, and context. There is no ordinal alias.

Every fast path has a checked fallback. The fallback returns the same bytes and the same errors in the same order as the full writer and the full reopen. Release proof of that equality is mandatory. A debug whole-path comparison is internal agreement. It is not that release proof. An unproved child, an unproved frame, an unproved context, a failed parse, unknown source that the block path cannot prove, or a binding mismatch uses the whole-part fallback before publication. A field-bearing block whose child, frame, and context are proved is serialized or read anew, and the other proved blocks stay reusable. That case does not require the whole-part fallback. An error is never an empty success.

`take_staging_work` changes from the retired `[usize; 4]` counter to `[usize; 6]` under `debug_assertions` and `#[doc(hidden)]`. The four-slot array is not the callable counter. Python `_staging_work` returns the same six-slot array. It is not a stable export. The order is parsed bytes, scanned bytes, canonicalized bytes, ZIP bytes written, serialized children, then children read. The first four slots are byte counts. The last two slots are child counts. A child count is not a byte count. Setup calls sit outside the counter window. Cold work on the first changed edits is measured separately and is not amortized into the warm window. The warm window starts after two changed edits and then records two distinct edits.

### 8. Scoped replacement proof and child canonicalization

This is the approved 2026-10-25 refresh4 extension. It is the eighth serial mechanism on the same F-X268 story. Implementation is in progress as recorded above. It does not add a ninth story, a stable public API, a dependency, or a module. Mechanisms 1 through 7 stay as approved.

Scoped replacement proof uses physical body children. Picture canonicalization and relationship renames use those children and the tail. The old whole paths stay the authoritative fallback.

Count, order, context, root, and span proofs are exact before a fast path is accepted. In-bounds disjoint edits do not panic on a `Vec` splice. Sentinel collision and physical selection use a double parse and a write normalization that matches the current whole algorithm. The fast path is not a weaker acceptance than that whole path.

A body block that contains a field, and a block that does not round-trip, is normalized and read anew. `PartialEq` alone is not reuse. Other proved blocks may still be reused. Preserve F-288 default, current, and provenance. Preserve F-289 physical form owners and handles. Preserve F-290 metadata and the bound store.

Release equality and release error order are required. A debug whole oracle is internal agreement only. There is no blanket hypothetical OOXML validator. Unknown foreign attributes, entities, head, tail, or an unaffected child must raise where the whole path raises, and they must raise before publication. A repeated call and a refusal leave bytes and live handles unchanged. Any unproved child, frame, or context chooses the whole fallback with the same errors, the same order, and the same bytes.

The private helpers below are captured from the PR422 own patch. They are not a public cross-crate contract change. No forwarding wrapper is copied only to destructure a tuple. `direct_story_content_items` calls `direct_story_content_items_in` because the caller already holds the scanned items.

```rust
fn canonical_retained_main_part(
    retained: &[u8],
    remap: &HashMap<String, String>,
    modeled_drawing_ids: &[u32],
    nested_image_relationship_ids: &HashSet<String>,
    canonical_drawing_ids: &[(u32, u32)],
) -> Result<Option<(Vec<u8>, CT_Document)>>;

fn relationship_remap_edits(
    xml: &[u8],
    remap: &HashMap<String, String>,
    inherited_bindings: &[(String, String)],
) -> Result<Vec<(usize, usize, Vec<u8>)>>;

fn xml_decodes(xml: &[u8]) -> bool;
fn scoped_proof_marker(original: &[u8], restored: &[u8]) -> Result<String>;
fn scoped_proof_sentinel(selected_closed: &[u8], marker: &str) -> Result<String>;
fn direct_story_content_items_in(
    xml: &[u8],
    owner: &StoryOwnerSpan,
    story_items: Vec<StoryItemSpan>,
) -> Result<Vec<StoryItemSpan>>;
```

On the existing `MainPartBlocks` type, the captured methods are `edited(&self, edits: &[(usize, usize, Vec<u8>)]) -> Option<Self>`, `bytes(&self) -> Vec<u8>`, `relationship_remap_edits(&self, remap: &HashMap<String, String>, bindings: &[(String, String)]) -> Option<Vec<(usize, usize, Vec<u8>)>>`, and `canonical_retained(...) -> Option<Option<(Vec<u8>, Self, CT_Document)>>` with the same four arguments as `canonical_retained_main_part` after `self`. `BodyBlockScan::decodes(&self, root: &[u8]) -> bool` and `MainStoryLayout::owner_kinds(&self) -> Vec<StoryKind>` stay private. `Document::scoped_main_source_by_blocks` returns `bool` and `Document::prove_whole_scoped_main_source` stays the whole `Result<()>` path. A `false` fast result means the children do not tell, and the whole proof decides.

`BlockStagingReads` gains `decodes: OnceLock<bool>`. The stored fact is whether `xml_decodes` reads the bytes of `alone(root)`. A false result means those bytes are not a fast proof. Replacing the block builds a new scan, so a changed fact does not keep the old `OnceLock`. A false error is invalidated when the bytes or the root context that produced it change.

Q22 applies to this new fact and to the current state. The measured points stay clone, reopen, and drop, with no accumulating history. A whole-model clone, hash, or copy stays a disclosed linear cost. There is no universal O(1) claim.

### Public Rust contract

Q21 adds four stable Rust methods. There are no optional parameters and no new units. There are no Python `try_` names, no WASM methods, and no CLI methods.

```rust
impl Document {
    pub fn try_add_paragraph(&mut self, text: &str) -> Result<Paragraph<'_>>;
    pub fn try_insert_paragraph(&mut self, index: usize, text: &str) -> Result<Paragraph<'_>>;
    pub fn try_add_table(&mut self, rows: usize, cols: usize) -> Result<Table<'_>>;
    pub fn try_insert_table(
        &mut self,
        index: usize,
        rows: usize,
        cols: usize,
    ) -> Result<Table<'_>>;
}
```

Preflight order on the four `try_` methods, all before mutation: the insert index range first, then paragraph XML characters, then the retained namespace guard. An index greater than `content_count` returns `Error::Other`. It does not panic. Text is checked with the existing `oxml_core::xml::reject_non_xml_characters`, and the native OXML error is retained. Existing Python primitive, type, and index checks still run before that guard. The native error hierarchy is unchanged. The new early namespace error is intentional only on the affected checked paths. Zero rows and zero columns stay accepted, as the existing table builders accept them. Other row and column semantics stay as they are. There is no new resource budget. When namespace replay or a shadowed serializer binding would make saving fail, the method returns the existing native `Error::Other` with the same underlying diagnostic, before any model, layout, or revision change. The bound check runs before the edit. There is no panic, no silent normalization, and no side effect on that failure.

The four existing infallible methods keep their signatures and their legacy behavior. Q21 exempts them from Q19. They are not described as checked. The four new methods are the checked Rust API. Python `add_paragraph`, `insert_paragraph`, `add_table`, `insert_table`, `Paragraph.insert_paragraph_before`, `add_heading`, and `add_page_break` use the checked path. Their public signatures, error family, primitive checks, and index validation stay. The early atomic refusal is the labelled behavior commit. It is not the performance commit.

### Hidden native helpers

The internal shared validation method is `#[doc(hidden)] pub fn preflight_checked_paragraph(&self, index: Option<usize>, text: &str) -> Result<()>`. Its actual consumers are the four checked Rust methods and Python paragraph character-error ordering. Python checks the native policy before its existing splice or fallback publisher. This adds no stable Python export and does not replace the source-preserving route.

Adopt a donor helper only where the Python binding or `Document` is an actual consumer. The declarations in the PR417 and PR418 block below are methods, not free functions. The later refresh2 block adds the free function `take_reopen_work`. None of these is a stable Python, WASM, or CLI export. `BodyItemContext` is the whole fact source for piece reuse, and a caller must hold that proof. No forwarding wrapper, trait, generic, module, crate, feature flag, or dependency is added. No new trait or generic is planned. The repository rule that needs two implementers or two instantiations applies to traits and generics, not to this method. `CT_Body::paragraph_count_before` is the concrete count. `Document::paragraph_count_before_content` is one call chain into that method, not a second independent consumer. A public cross-crate count helper needs the actual `Document` consumer. It does not need an arbitrary second consumer.

Optional eligibility `None` chooses the checked full path. A helper `Err` may take that full path only where the full path returns the same checked result, error, and error order the old path would have returned before publication. A full-path `Err` propagates through the existing native and Python mapping. Invalid ownership or invalid XML never becomes empty facts or success. `unsavable_body_edit` returning `None` is not a general proof that the document is savable.

Captured from PR417 `crates/rdocx/src/document.rs` as nine fields. That nine-field capture stays historical. The struct below is the final refined callable contract. It keeps those nine fields and adds `pub holds_field: bool` as the tenth field. The structs are public doc(hidden) cross-crate Rust types. The functions are methods:

```rust
#[doc(hidden)]
#[allow(clippy::type_complexity)]
pub struct BodyItemOwners {
    pub snapshots: Vec<StoryItemSnapshot>,
    pub cells: Vec<(ContentLocation, usize, usize, StoryId)>,
    pub paragraphs: Vec<(ContentLocation, Vec<u8>)>,
    pub links: Vec<(ContentLocation, LinkInfo)>,
    pub cell_stories: usize,
    pub text_boxes: usize,
    pub paragraph_routes: Vec<Option<Vec<usize>>>,
    pub tables: usize,
    pub cell_paragraphs: Option<(HashMap<(usize, usize, usize), Vec<ContentLocation>>, bool)>,
    pub holds_field: bool,
}

#[doc(hidden)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BodyItemContext {
    part_name: String,
    root: [Vec<(String, String)>; 5],
    plain_local_edit_blocked: bool,
}

impl BodyItemContext {
    pub fn part_name(&self) -> &str;
}

impl Document {
#[doc(hidden)]
pub fn body_item_owners(&self, item: &BodyContent) -> Result<BodyItemOwners>;
#[doc(hidden)]
pub fn body_item_context(&self) -> BodyItemContext;
#[doc(hidden)]
pub fn body_children(&self) -> &[BodyContent];
#[doc(hidden)]
pub fn story_part_sources(&self) -> Result<Vec<(StoryKind, String, Cow<'_, [u8]>)>>;
#[doc(hidden)]
pub fn story_part_owner_snapshots(
    &self,
    kind: StoryKind,
    part_name: &str,
    xml: &[u8],
) -> Result<(
    Vec<StoryItemSnapshot>,
    Vec<(ContentLocation, usize, usize, StoryId)>,
    Vec<(ContentLocation, Vec<u8>)>,
    Vec<(ContentLocation, LinkInfo)>,
)>;
#[doc(hidden)]
pub fn story_hyperlink_relationships(&self, part_name: &str) -> HashSet<&str>;
#[doc(hidden)]
pub fn unsavable_body_edit(&self) -> Option<&str>;
#[doc(hidden)]
pub fn paragraph_count_before_content(&self, slot: usize) -> usize;
#[doc(hidden)]
pub fn saves_as(&self, other: &Document) -> Result<bool>;
}
```

Captured from the new PR415 tree at `sources-old415-vs-new415/crates/rdocx-oxml/src/document.rs`. The impl starts at line 3397 and the method at line 3423:

```rust
impl CT_Body {
    #[doc(hidden)]
    pub fn paragraph_count_before(&self, index: usize) -> usize;
}
```

`cell_paragraphs: None` is failed cell routing. It selects the checked full path. It is not an empty success.

`saves_as` is the PR418 method now declared on `impl Document`. It may use `OpcPackage::reread` only while the current trusted-baseline precondition still guarantees the same bytes, the same error, and the same error order as `to_bytes`. Otherwise it uses the ZIP compare. The donor comment that a reread is automatically byte-identical is not adopted as proof.

The refresh2 amendment adds these hidden cross-crate contracts. They are not stable Python, WASM, or CLI exports. No new trait, generic, module, file, or dependency is added.

```rust
impl CT_Document {
    #[doc(hidden)]
    pub fn to_xml_with_body_spans(
        &self,
    ) -> Result<(Vec<u8>, std::ops::Range<usize>, Vec<std::ops::Range<usize>>)>;
}

impl Document {
    #[doc(hidden)]
    pub fn direct_body_item_location(&self, slot: usize) -> Result<Option<ContentLocation>>;

    #[cfg(debug_assertions)]
    #[doc(hidden)]
    pub fn main_body_block_scans() -> usize;
}

#[cfg(debug_assertions)]
#[doc(hidden)]
pub fn take_reopen_work() -> [usize; 3];
```

The approved refresh3 extension adds these hidden contracts. They are not stable Python, WASM, or CLI exports. `MainPartBlocks` stays private to `document.rs`. No new trait, generic, module, file, or dependency is added.

```rust
impl CT_Document {
    #[doc(hidden)]
    pub fn body_frame_xml(&self) -> Result<(Vec<u8>, Vec<u8>)>;
    #[doc(hidden)]
    pub fn body_item_xml(&self, item: &BodyContent) -> Result<Vec<u8>>;
}

#[cfg(debug_assertions)]
#[doc(hidden)]
pub fn take_staging_work() -> [usize; 6];
```

`holds_field` is the tenth field of the final `BodyItemOwners` declaration above. It is not a free-standing field.

`body_frame_xml` and `body_item_xml` are consumed by the actual native staging and reopen callers in `Document`. `holds_field` is read by the piece inventory and by block reuse. `take_staging_work` feeds Python `_staging_work`. The array order is fixed above.

`to_xml_with_body_spans` is consumed by the actual native `Document` story-source and direct-slot callers. `direct_body_item_location` is consumed by the actual Python document and table callers. `None` means absence only. Failed fast eligibility uses the checked full lookup. A genuine `Err` is preserved. `take_reopen_work` is a free function, exported from the native `lib.rs` for the Python `_reopen_work` counter. The array order, taken from the assessed counter body before any local test, is ZIP package writes, then main document part parses, then identifier-part bytes scanned. The implementation counts those actual sites. `main_body_block_scans` is the debug count of main-part body children scanned rather than reused, and it feeds Python `_main_body_block_scans`. Debug hooks are not stable exports and are not release exports. A release installed full test may guard a helper only for a counter case. Mandatory debug proof stays. There is no broad skip.

## Consumer map

| Rule | Entry points and outputs | Shared provider and caller policy | Lifecycle stages compared |
|---|---|---|---|
| Checked insert | `try_add_paragraph`, `try_insert_paragraph`, `try_add_table`, `try_insert_table`. Python `add_paragraph`, `insert_paragraph`, `add_table`, `insert_table`, `insert_paragraph_before`, `add_heading`, `add_page_break` | Q21. Old four infallible methods stay legacy. Binding keeps its signatures | Evaluate the guard, then update, then serialize. Reopen the saved package |
| Namespace replay | `unsavable_body_edit` against current XML, model, and owners | Proof dies when the owner, local namespace, text, or typed field changes | Before mutation and again after a fact change |
| Piece inventory | `body_item_owners`, `body_item_context`, `body_children`, story part helpers | Binding publication is the consumer. Context equality is caller proof | Read, project, serialize, reopen |
| Staging | `saves_as` and the opened baseline shared with a staged clone | OPC `trusted_baseline` stays. Related parts stay checked | Save, error order, reopen, drop |
| In-memory reopen | `reopen_prepared_staged_with_limits` and `take_reopen_work` | UNBOUNDED opened-from-bytes trusted provenance only. Every other context keeps the ZIP path. Identifier replay keeps duplicate order and errors | ZIP path, in-memory path, limited reopen, both routes after mechanism 6 |
| Direct-body address | `to_xml_with_body_spans`, `direct_body_item_location`, `main_body_block_scans` | Exact `to_xml` bytes. `None` is absence. Fast-path failure uses the full lookup. `Err` stays `Err` | Full snapshot, both reopen routes, cache off, invalidated context |
| Main-part body blocks | `body_frame_xml`, `body_item_xml`, private `MainPartBlocks`, `BodyItemOwners.holds_field`, `take_staging_work` | Checked fallback matches full bytes and error order. No field-block reuse by `PartialEq`. Changed root invalidates. F-289 and F-290 publishers stay | Warm N and 4N counts, cold work separate, cache off, alternate root, save and reopen |
| Form and settings facts | Direct `cached_result`, `set_cached_runs`, annotation cache, `cached_field_mut`, refresh, attachment, and `set_legacy_form_field_value`. F-289 `validate_form`, `fill_form`, `reset_form`, `protect_form`, `set_document_protection`, `remove_document_protection`, and `add_story_permission_range`. F-290 `set_mail_merge_package`, `remove_mail_merge_package`, and `prepare_mail_merge_data` | Approved callables only. No second form model. Property callbacks and direct candidate assignment invalidate changed facts even when the generation is unchanged. Private field source form and equality that ignores serialization facts use proved invalidation or the full fallback. They must not publish a different truth. Dead owners stay dead | Held value, fresh value, native value, save, and reopen |

Imported metadata and an absent cache fact stay absent. A missing fact is not filled with a default by the cache.

## Behavior contracts and evidence

| Surface or placement | Exact behavior and boundary examples | Primary source identity and location | Local discriminator or pending evidence |
|---|---|---|---|
| Index above `content_count` | `Error::Other` before mutation. No panic | Current `insert_paragraph` panics. Q21 replaces that only on the `try_` methods | `checked_body_insertions_refuse_unsavable_namespace_owners_atomically` |
| Non-XML text | Existing `oxml_core::xml::reject_non_xml_characters`. Native OXML error. No mutation | Existing character check | Same native test |
| Zero rows or columns | Accepted, matching current table builders | `Document::add_table` | Same native test |
| Namespace or shadowed prefix | Existing `Error::Other` and the same diagnostic, before model, layout, or revision change | Q19 refined by Q21 | Native test and `test_checked_body_insertions_preserve_owners_on_namespace_refusal` |
| Old four methods | Signatures and legacy behavior unchanged, including a later save failure | Q21 exemption | Explicit legacy case in the native test |
| Package bytes | Unchanged except the requested edit. No redundant saved xmlns delta | Integrated F-X266 fragment rule | `word_owner_piece_sources_match_the_full_physical_inventory` |
| `saves_as` | Exact bytes and exact error order, or ZIP fallback | `OpcPackage::reread` trusted baseline | `unchanged_main_staging_preserves_save_bytes_and_error_order` |
| Work counters | Real walk, parse, and serialization counts. Not a constant entry counter | F-X266 linear owner bound retained | `unchanged_main_staging_reuse_counts_actual_work` |
| Body-child staging | Same bytes and same error order as the full path. `holds_field` blocks value reuse. Alternate root reads the part whole | PR421 candidate only. Not accepted code | `staging_reads_only_the_main_part_blocks_an_edit_changes`, `test_staging_writes_and_reads_only_the_main_part_blocks_an_edit_changes`, `test_a_fragment_imports_into_a_main_part_whose_root_has_another_word_prefix` |

## Dependency readiness

F-X266 source `9c6ef377971182a73b2e3cef07ccedde579d2f37` is integrated at `9ee48bf0902d976de016553b1dd0c79fed856612`. F-X267 source `70573c91030be7073d29e3451383060eb207207a` is integrated at `d0f17a6ac7ef675d0ed2a86cea641ce21d65bfdd`. Both are reviewed and in progress. Neither is done. Approval of those plans is not final acceptance.

The local prefix is approved contract. Canonical `0d8dfb68396e33ddfcaa0d9400c30c006e12d2b7` added `## F-X268 local provider checkpoint` to `.claude/plans/F-X266-design.md` and `.claude/plans/F-X267-design.md`. Codex root approved both checkpoints on 2026-10-24 for use by F-X268. Actual prefix completion has not been performed. Neither story is done. The checkpoints do not weaken the final performance, hash, or Word riders.

The prefix names every local scoped row already in those approved sections. Outcomes and commands come from the existing progress records. The approved contract does not invent a combined pass. No gate pass is claimed here.

F-X266 test plan, all 9 local scoped rows: `direct_routes_match_full_body_bytes`, `append_keeps_eligible_owners`, `conservative_retires_all_scopes`, `failed_splice_publishes_nothing`, `fragment_omits_unused_w14`, `owner_scope_precedence_unchanged`, `held_fresh_reopen_match_old_semantics`, `body_work_counters_follow_owners`, `missing_run_path_refuses`.

F-X267 test plan, all 13 local scoped rows: `remove_part_returns_shared_bytes`, `reread_matches_write_and_read`, `reread_matches_shared_zip_name_limits`, `hand_built_baseline_uses_fallback`, `untouched_slide_bytes_stay`, `fit_text_ignores_unrelated_broken_slide`, `fit_text_at_selects_size_valid_in_distinct_shared_contexts`, `footer_proof_is_part_local`, `parsed_baseline_tracks_real_edits`, `fx211b_native_opaque_root_edit_retains_invalidated_signature`, `embedded_replace_counts_commit_zip_write`, `unnormalized_opaque_part_counts_reread_zip_fallback`, `commit_counters_follow_real_work`.

The prefix also consumes the actual scoped checks, clean Root reviews, and the current applicable artifact proof. An old artifact is not reused as a fresh qualifier. Rerun only the inputs a later change invalidates, including a checked build and import where binding evidence is affected. No full oracle rider runs at the prefix. Word, the full suite, the hash harness, the final release 79-call and 252-call runs, the F-X267 248-call run, and the memory rider stay pending. Before any F-X268 claim, verify that the F-X266 and F-X267 source heads above are actual ancestors, that this prefix is actually done, and that binding source proof exists where the binding is affected. `AS_BUILT.md` and `SPRINT_TRACKER.md` are written once, after the full union. A formal consumer claim waits for the actual done checkpoint.

The next paragraph is the state before the 2026-10-25 local prefix. It is dated historical for that earlier readiness.

The current F-289 reviewed local integration is `cbcb0c9f07f392d541362f08bd6e6af6311f3417`. Root whole-feature pass 2 is clean, with 0 defects, 0 smells, and 0 nitpicks. The fresh local checks are native 91 and Python 13 on artifact `130f13ce533f8a78858b09a7113febc33198ebfdf778682e1196d85683906105`. Those checks are local only. Final Word 16.114 A7 and the full gate stay pending. F-289 is not done. That checkpoint is not delivery credit and it is not a formal dependency change. F-X268 stays unclaimed. F-X268 does not start while the shared leases are held by F-289 and then F-290.

## Local prefix readiness 2026-10-25

Root approved genuine local provider completion for F-X266 and F-X267 on canonical `79047bf193bcd9dfb229a0b37a93d979dc0997a7`. This record is that Root review. It is not a self-review. Both providers are done for that local prefix. Neither is final-accepted. Plan status for F-X266, F-X267, and F-X268 stays approved. The sentences above that say the prefix has not been performed stay the earlier contract text.

Source head `9c6ef377971182a73b2e3cef07ccedde579d2f37` and integration `9ee48bf0902d976de016553b1dd0c79fed856612` are ancestors of that HEAD. Source head `70573c91030be7073d29e3451383060eb207207a` and integration `d0f17a6ac7ef675d0ed2a86cea641ce21d65bfdd` are ancestors of that HEAD. Current artifacts are `/private/tmp/s95.4-fx266-verified-_rdocx.abi3.so` sha256 `ae3879224176459797da677f26058757a5aa0592817921004c985ded1d8d17d6` and `/private/tmp/s95.4-fx267-local-debug-_rpptx.abi3.so` sha256 `9fab4e917aef5ee74457ab95e85ffddaa262698fc7d9cbeaa1e88cc9cd0e56ca`. The `9fab4e91` file is a fresh development build and import proof only. It is not the native 13 and it is not a release build. The old `1fe91d2d` archive is not the current qualifier. The seven fresh F-X267 runs fill missing dedicated GREEN logs. They are not a claim that a later change invalidated those rows. The F-X266 linear-proof paragraph that still says the counter proof stays pending is the earlier contract state. The current local counter GREEN is `body_work_counters_follow_owners` in the 9 pytest rows. The signature row passed 1 with `--features digital-signatures`. The earlier 0-test filter is not a pass. The preferred retained PowerPoint proof for `parsed_baseline_tracks_real_edits` and the other names in the post-counter block is `.claude/scratch/F-X267-progress.md`.

F-290 is integrated at `79047bf193bcd9dfb229a0b37a93d979dc0997a7`. Root whole-feature pass 3 is clean at `.claude/reviews/F-290-whole-feature-pass-3.md`. Public local checks are consumed. F-290 stays in progress and is not done. F-289 stays in progress and is not done. Shared leases are released. This readiness record does not claim F-X268. Final debts stay pending: the 79 flat gates, the 252-call catalogue, the 5 linear calls, the original 407 gate, the PowerPoint 248-call run, the original 10, 40, and 80 timings, the M21 digest, memory, Word, the hash harness, the full suite, strict mypy, stubtest, WASM, and packaging. `AS_BUILT.md` and `SPRINT_TRACKER.md` stay unchanged.

## Acceptance criteria

| ID | Input or setup | Expected observation or refusal | Consumers | Local test | Final gate or pending evidence |
|---|---|---|---|---|---|
| A1 | Q19 and Q21. Four new Rust methods and every named Python delegate. An old document whose save already fails. A valid document. Bad index, non-XML text, zero columns | Refusal is atomic. Model, bytes, and handles stay at the old unsavable state. The old four methods stay legacy. Valid bytes match the checked source. Errors keep the existing family | Native `Document` and Python document, paragraph, heading, and page break | `checked_body_insertions_refuse_unsavable_namespace_owners_atomically`, `test_checked_body_insertions_preserve_owners_on_namespace_refusal` | Local behavior commit. Not the performance commit |
| A2 | New PR415 unrouted facade position and nested namespace proof. Then an owner, local namespace, text, or typed field edit | First proof matches the checked source. The stale proof is not reused. Accepted aliases and unknown bytes stay. No incidental namespace byte change | Facade and fragment writer | `word_owner_piece_sources_match_the_full_physical_inventory` | Local regression |
| A3 | Piece inventory against the full inventory for body, nested cell, text box, comment, note, shared header, duplicate part, invalid relationship, identical blocks, and a direct returned owner. The publishers named in the consumer map, including direct candidate assignment and property callbacks. Direct-body span bytes and block reuse | Equal owners, parents, routes, projections, read facts, order, births, generations, and retired set. Span bytes match existing `to_xml`. Unsupported shape falls back before publish. Held, fresh, native, save, and reopen agree. Ignored serialization facts do not count as proof | Binding and native readers | `word_owner_piece_sources_match_the_full_physical_inventory`, `word_owner_pieces_follow_splices_and_fact_changing_publishers`, `body_spans_match_existing_to_xml_on_current_fixtures`, `main_body_block_reuse_preserves_physical_owners_and_publication`, `test_body_block_reuse_preserves_forms_settings_and_live_owners`, `fx268_changed_existing_context_uses_full_inventory_before_publication`, `fx268_case_equivalent_story_parts_use_the_full_inventory`, `test_word_piece_shared_header_identity_uses_opc_part_names` | Local regression on the integrated ancestor that contains those publishers |
| A4 | Splices at head, middle, and tail, plus a missing item. A later edit. No-op and refusal | Route and later edit stay inside the work bound. Conservative births and retirement match the old path. Dead owners stay dead. No-op and refusal preserve bytes, revisions, and handles | Python publication | `word_owner_pieces_follow_splices_and_fact_changing_publishers`, `test_word_owner_pieces_follow_edits_splices_and_related_owners` | Local regression |
| A5 | Opened cache and `saves_as` for new, opened, clone, source-edited, relationship, media, signature, and embedded packages. Main unchanged with a changed related part. Eligible in-memory reopen and a document with no trusted opened-byte baseline | Exact bytes and native error order, or the ZIP fallback. Related changes are checked. Identifier order matches. A warm edit does not reparse an unchanged main part. A document qualifies for in-memory reread only after a trusted opened-byte baseline exists | OPC and binding | `unchanged_main_staging_preserves_save_bytes_and_error_order`, `unchanged_main_staging_reuse_counts_actual_work`, `test_unchanged_main_staging_keeps_forms_and_settings_authoritative`, `in_memory_reopen_matches_zip_for_new_flat_limited_and_hook_paths`, `in_memory_reopen_keeps_related_part_errors_and_drops_stale_main_checks`, `identifier_scan_replays_exact_bytes_categories_and_old_error_order`, `parsed_none_reopen_grows_zip_parse_and_identifier_bytes`, `test_header_story_text_reopen_counts_the_changed_header_part`. Native read-limit coverage is in the first of those four native tests. Python evidence uses the existing `Document.set_story_text` API only. Forms and settings evidence stays the planned `test_unchanged_main_staging_keeps_forms_and_settings_authoritative` | Local regression |
| A6 | The 79 targeted calls and the full 252-call catalogue on a fresh release install | The 79 meet the Q20 flat bound. The 252 results and residuals are reported without xfail hiding. The original 407 gate still holds for its three calls. The four extra PR419 test-map keys are not silently added | Installed Python 3.12 | Named in the performance section. Not a worker run | Final union |
| A7 | Q18 and Q22 memory workload | Actual per-layer and combined RSS samples at the named points. No summed RSS fabrication. No 68 MiB pass line and no hard cap | Fresh release process | Named in the memory section | Final union |
| A8 | Installed Python, strict mypy, stubtest, rustdoc, publish archive, WASM, hash, and the prior seven stories | Prior Word, SSIM, risk, and hash gates stay mandatory and stay unrun here. Q19 compares the previous save failure, model, and handles. Two failing `to_bytes` calls are not atomicity proof. Final own-union fidelity is mandatory. The PR419 donor log is not that pass | Final union | None in the worker | Final union |
| A9 | Q23 linear calls at 250 and 1000 paragraphs | Each call meets `large <= 5 * small + 5 ms`. Each relevant pass count meets `large <= 5 * small`, with additive allowance 0. Setup counts stay outside that pass. Whole-main hash and serialization are reported separately | Installed Python 3.12 | `body_lookup_work_is_linear_at_250_and_1000` for counts. Timings only in the final union | Final union |
| A10 | Fourteen public calls at 20 and 80 paragraphs, and native table row, section, list, paragraph scoped replace, cell scoped replace, and story picture edits at 50 and 200 paragraphs. Setup outside the counter window. Warm samples are two distinct edits after two changed edits. Cold work is a separate measurement | All six staging slots are compared as one `[usize; 6]` counter. The retired four-slot counter is not this contract. The first four slots are byte counts and are zero, except `Document.import_fragment`, whose bounded source-document byte work stays and is equal at both sizes. The last two slots are child counts, serialized children then children read. Those child counts are equal across the two sizes. They are not byte counts and they are not required to be zero. Reopen ZIP writes are 0. Main-part parses are 0, except the fragment call, where that slot is 1. Identifier-byte absolute delta is at most 16. Native warm staging has the first four byte slots at zero and the first two reopen slots at zero, with the same warm counts at both sizes. A cache-off mutant for the scope proof, the canonicalization, or the rename does actual work that scales. A missing helper is a compile failure, not that RED. These are counted-work obligations. They are not new timing gates | Python document and native `Document` | `test_staging_writes_and_reads_only_the_main_part_blocks_an_edit_changes`, `staging_reads_only_the_main_part_blocks_an_edit_changes`, `test_a_fragment_imports_into_a_main_part_whose_root_has_another_word_prefix` | Local counted-work regression. Approved with mechanism 8. Independent of the 79 timing gates |
| A11 | Release fallback and error-order proof. Aliases, the default namespace, a shadowed namespace, `w14`, opaque attributes, entities, tail, a field, a tab, bookmarks, and a duplicate physical position. Valid positive controls stay | The fast path matches the current whole algorithm on bytes, errors, and error order, or it takes that whole path. Unknown foreign attributes, entities, head, tail, and an unaffected child raise where the whole path raises, before publication. Repeated calls and refusals leave bytes and live handles unchanged. An unproved child, frame, or context uses the whole fallback | Native `Document` and Python document | Named with the mechanism 8 group. Not a timing promise | Local regression. Approved with mechanism 8. Not a timing promise |

The original 65 targeted calls are 62 catalogue keys plus `Run.xml`, `Table.xml`, and `Cell.xml`. The pinned 252 does not cover those three reads. The refresh2 amendment adds 8 calls. That historical Q23 flat set is 73 unique calls, which is 70 catalogue keys plus the same 3 XML reads. Q24, answered 2026-10-25, adds 6 calls. The current flat set is 79 unique calls, which is 76 catalogue keys plus the same 3 XML reads.

Targeted calls from PR415, 13:

`Document.insert_paragraph`, `Document.remove_content`, `Paragraph.insert_paragraph_before`, `Document.remove_content_range`, `Document.add_paragraph`, `Document.insert_table`, `Document.add_table`, `Document.add_heading`, `Document.add_page_break`, `Paragraph.xml`, `Run.xml`, `Table.xml`, `Cell.xml`.

Targeted table, row, and cell setters from PR417, 26:

`Table.indent`, `Table.alignment`, `Table.width`, `Table.set_borders`, `Table.set_border`, `Table.set_cell_margins`, `Table.grid_widths`, `Table.set_column_width`, `Table.set_cell_grid_span`, `Table.set_cell_vertical_merge`, `Table.autofit`, `Table.first_row`, `Table.last_row`, `Table.first_column`, `Table.last_column`, `Table.banded_rows`, `Table.banded_columns`, `Row.height`, `Row.height_rule`, `Row.cant_split`, `Row.is_header`, `Cell.width`, `Cell.vertical_alignment`, `Cell.shading`, `Cell.set_border`, `Cell.set_margins`.

Other PR417 calls, 16:

`Cell.add_paragraph`, `Paragraph.text`, `Paragraph.add_run`, `Paragraph.add_hyperlink`, `Paragraph.add_equation`, `Run.remove`, `Run.add_field`, `Document.split_run`, `Document.add_comment`, `Document.reply_to`, `Document.resolve_comment`, `Document.add_chart`, `Document.set_text_watermark`, `Document.set_image_watermark`, `Document.set_note_policy`, `Settings.odd_and_even_pages_header_footer`.

PR418 calls, 10:

`Document.set_font`, `Document.embed_font`, `Document.remove_embedded_font`, `Document.remove_font`, `Document.embed_true_type_fonts`, `Document.save_subset_fonts`, `Document.embed_system_fonts`, `Document.add_numbering_definition`, `Document.add_numbering_instance`, `Document.link_style_to_numbering`.

That original list is 65 unique calls. `Run.xml`, `Table.xml`, and `Cell.xml` are the three additional required read timings. They are not catalogue keys.

The 8 added flat calls, none of them in the original 65, are:

`Document.replace_image`, `Document.replace_image_for_story`, `HeaderFooter.add_paragraph`, `HeaderFooter.add_table`, `SectionHandle.set_text_watermark`, `SectionHandle.set_image_watermark`, `SectionHandle.remove_watermark`, `Cell.add_table`.

The historical 73 and the current 79 use the Q20 bound. The 100-paragraph time is at most 1.5 times the 25-paragraph time plus 5 ms, under the existing sample rules. The four extra keys in the PR419 test map are `FootnoteEndnote.add_paragraph`, `FootnoteEndnote.add_table`, `Document.add_custom_xml_store`, and `Document.set_chart_legend`. They are not added to the 79 and they are not silently gated. Under the historical Q23 set, `Run.add_picture`, `Cell.add_picture`, and `Cell.merge` were 252 residuals. Q24 moves `Cell.add_picture` into the 79. The current residuals of that trio are `Run.add_picture` and `Cell.merge`.

The six Q24 additions, none of them in the historical 73, are `Paragraph.replace_text`, `Cell.replace_text`, `Document.replace_text_at`, `Document.add_picture`, `Cell.add_picture`, and `Document.add_hyperlink_to_story`. All 5 A9 linear calls stay at 250 and 1000 paragraphs, including `Cell.replace_text` and `Document.add_picture`, which are also in this flat set. The 252-call catalogue, the original 407 gate, and the F-X267 248-call run stay unchanged.

### A10 counted work

The nine calls named in the PR421 own diff stay. The approved refresh4 extension adds five calls on producer-retained XML: `Paragraph.replace_text`, `Cell.replace_text`, `Document.replace_text_at`, `Document.add_picture`, and `Cell.add_picture`. The counted-work set is those 14 calls. A10 is independent of the Q24 timing gate. `Document.add_hyperlink_to_story` is one of the 79 flat calls and is not one of the 14. Python sizes stay 20 and 80 paragraphs. The producer keeps the XML. Setup stays outside the window. Warm samples are two distinct edits after two changed edits. Native sizes stay 50 and 200 paragraphs, and the existing native group adds paragraph scoped replace, cell scoped replace, and story picture. The six counter units stay the same. Bounds stay the same, including the fragment byte exception and the reopen identifier bound. A real cache-off mutant for the scope proof, the canonicalization, or the rename must expose growing actual work. A missing helper is a compile failure, not that RED. A10 does not add a timing promise of its own. Donor timings are not accepted.

### A11 release fallback

A11 is approved with mechanism 8. It proves release fallback and error order for aliases, the default namespace, a shadowed namespace, `w14`, opaque attributes, entities, tail, a field, a tab, bookmarks, and a duplicate physical position. Valid positive controls stay. The fast path matches the current whole algorithm, or the whole path runs with the same errors and the same order. Debug agreement is not this release proof.

### A9 linear calls

The five Q23 calls are `Cell.replace_text`, `Document.set_picture_size`, `Document.add_picture`, `Document.move_content_control`, and `Document.move_comment`. Use the same pinned builder, adapted to 250 and 1000 paragraphs, with the same six formatted runs and the same target geometry between sizes. Fresh target, call setup, and open stay outside the clock. The full public lookup, validation, and publication are inside the clock. Alternate 3 fresh changed targets at the small size and the large size. Retain every raw sample. The minimum follows `costs_ms`. The cold first sample stays visible. Warm statistics stay separate. There is no compilation concurrency. For each call, the large time is at most 5 times the small time plus 5 ms. Count the actual visited nodes, blocks, identifier operations, and lookup operations at 250 and at 1000. Pin that counted work before measurement. For each relevant pass, the large count is at most 5 times the small count, and the additive allowance is 0. Count setup separately, outside the measured pass. Retain the raw counts for each phase. A constant entry counter is not proof. Report whole-main hash and serialization separately. If the first cold scan is itself linear, disclose that. A semantic mutant or a cache-off build must expose the count regression. Absence of a new helper is a compile failure, not that RED.

### Post-pin public surface audit

The immutable 252-call catalogue stays. Its timed coverage checker predates F-288, F-289, and F-290. The final union public stub inventory enumerates every added or changed method and maps each new read-only, factory, global, and local method to its F-ID, its A-ID, and its timing disposition. Do not relabel a local method `WHOLE_DOCUMENT` only so the old checker passes. A transparent qualification driver may adapt only the post-pin coverage classification, with the new methods and the reasons named. It preserves all 252 old keys and all 79 required timings. It does not skip or drop broadly. A new local method that is not in the 79 stays outside the Q20 timing promise unless the original authoring plan already owns a performance criterion. Correctness, runtime, and stub tests for those methods stay mandatory. Report the expanded inventory count separately. Do not claim that 252 covers it.

The full catalogue is 252, which is 250 timed calls plus 2 reads: 216 literal timed calls, 15 CoreProperties calls, 13 Section calls, 6 App calls, and 2 reads. Of those, 145 were the old growing set and 107 were the old non-growing set. Run all 252 without hiding status behind xfail. Report each slow residual. Do not copy a donor pass count. Existing non-growing calls that are not in the 79 keep the original bound of 2.5 times N plus 5 ms. Each of the 79 uses the Q20 bound: the 100-paragraph time is at most 1.5 times the 25-paragraph time plus 5 ms.

The final consumer is a fresh release install of Python 3.12 from the actual union source head, with the artifact SHA, interpreter, and config recorded. The external builders use 25 and 100 paragraphs, six formatted runs, a picture or table every 10th paragraph, two sections, a header, comments, and a control, for the qualified targets in the pin `4646db50ed783ddf4a2220f4b95a15ad7a225944`. Sampling alternates N and 4N, up to 3 fresh changed targets, and keeps the minimum the way `costs_ms` does. Two samples may stop early only when both fastest samples are above 50 ms. Setup, open, and build stay outside the clock. Every raw sample is retained. There is no concurrent compilation. The first sample is cold and stays visible. Warm statistics are separate. A cold walk and a fallback cost are disclosed. A warm minimum is not a constant-time claim. The original Issue 407 gate stays: mean of 3, 100 versus 1000 paragraphs, six verbose run-property runs, `large <= 3 * max(small, 1 ms)`, plus a separate fresh median of 7, for both cold and warm, on the original F-X266 methods. A native benchmark does not replace the Python public lookup, validation, and publication.

Local counters sit at the real walk, parse, and serialization sites. Separate the owner-projection phase from the total public call. On a qualified case, 20 versus 80 or 50 versus 200 unrelated body blocks of equal piece count, the piece owner projection reads at most 2 pieces, and at most 1 piece after a qualified top splice or tail removal. That phase does not serialize an unrelated body block. Underlying staging may still serialize the current typed main part as the exact byte proof. Count those whole-main serialization bytes and that time separately. Do not hide them inside a zero reparse or zero canonicalization count. After the first unchanged main check, an eligible warm edit does 0 main reparses, 0 main canonicalizations, and 0 ZIP compare writes. That is not zero serialization for the whole call and it is not an O(1) whole call. The total public Python time covers lookup, validation, publication, and serialization. The existing F-X266 qualified direct-read and splice bound of no unrelated serialization stays unchanged. Misses and visited metadata are counted, so an O(N) metadata walk is disclosed and is not called O(1). Registry owner work keeps the existing F-X266 linear bound and is not quadratic. The cold proof body walk is counted separately. Native counters cover the current main parse, owner scan, canonicalization, and ZIP writes. Related-part checks are still counted. A constant entry counter is not proof. A build with the cache forced off, or a mutant that skips invalidation, must fail these asserts. Absence of a new helper is a compile failure, not that semantic RED. The debug full-path oracle is internal agreement, not external parity. Debug counters and release guards do not skip the production check.

Memory, final Q18 and Q22 rider only. The workload is 20000 paragraphs, six runs each, with fixed text and run formatting. Measure and report the actual main XML length. The donor 6 MiB claim is unverified. Use 3 paired alternating fresh processes. Record the median current RSS and report each peak, with the raw samples and the aggregation named, after open, after the first font edit, after 100 distinct warm edits, after clone and drop, after reopen and drop, and after the document drop. Pair a pre-cache commit baseline with the final release, sharing the inventory and otherwise equal. Record the baseline source, build, artifact, allocator, and process boundary. The shared cache may retain the opened baseline, one parsed model, and one serialized copy, and the current changed parts, plus bounded projection facts for each live piece. It does not retain a history per edit. N edits and 4N edits must not retain one baseline per edit. Clone ownership and reopen release stay safe. There is no 68 MiB pass claim and no hard budget the user did not choose. The donor 68 MiB figure is not accepted. Measure authored and new-document fallback, and the unsavable namespace, duplicate part, signed, and embedded fallback times, separately. Correctness is mandatory on those paths.

Q22 measures each cache layer on otherwise equal release builds, and it measures the combined footprint the same way. The layers are the PR418 parsed model and serialized copy, the PR419 reopen and identifier caches, the PR420 body bytes and scans, and the refresh3 `MainPartBlocks`, `BodyBlockScan`, frame, parsed, namespace, relationship, drawing, numbering, and identifier caches. Each of those added layers is measured independently and in the combined footprint. The points stay current state, N versus 4N, clone, reopen, and drop. There is no per-edit history. Use true equivalent pre-layer and final variants. Retain the original fixture, statistics, raw RSS, peaks, clone, reopen, drop, and the N versus 4N history test. Do not add RSS deltas together and call that sum the combined footprint. Q22 already covers this caching. The approved refresh4 extension adds the `decodes` fact to the same policy. The points stay current state, clone, reopen, and drop. There is no accumulating history. Whole-model clone, hash, and copy stay disclosed linear costs. There is no universal O(1) claim. Q22 is not reopened. Q24 is a timing-gate choice, not a memory-policy choice, and it is answered.

## Test matrix

| Case | Modes or state combined | Expected difference or agreement | Wrong implementation rejected |
|---|---|---|---|
| C1 | Zero, one, and many body blocks. Direct and nested. Alias and shadowed namespace | Guard and inventory match the full path | A stale proof after a text edit fails |
| C2 | Current versus default form value. Implicit and explicit protection | Facts change even when the generation does not | Semantic equality used as the only proof fails |
| C3 | New document and reopened package. Partial write and whole write | Bytes and error order agree, or the ZIP fallback runs | A main-unchanged skip that ignores a related part fails |
| C4 | No-op, refusal, and conservative splice | Handles and retired scopes match the old publication | A refusal that mutates, or a dead owner that returns, fails |
| C5 | Opened UNBOUNDED package, new document, limited reopen, and both reopen routes | In-memory reuse matches ZIP bytes, errors, and identifier order only on the eligible path | A limited reopen or a changed relationship context that skips the ZIP path fails |
| C6 | Direct slot zero, one, and many, nested controls, tables, cells, text boxes, and moved or repeated blocks | Span bytes match `to_xml`. Slot lookup matches the full snapshot before and after both reopen routes | An ordinal alias, an empty success, or a donor overwrite of form providers fails |
| C7 | Field-bearing and field-free blocks. Default namespace and an aliased root. Duplicate occurrence order. Cache on and cache off | Kept bytes match the full writer. A field block is not reused by `PartialEq`. Counts match A10. F-289 and F-290 owners stay | A false-negative field reuse, a changed-root hit, or a fallback that drops an error fails |
| C8 | Aliases, default and shadowed namespaces, `w14`, opaque attributes, entities, tail, field, tab, bookmarks, and a duplicate physical position. Valid positive controls. Cache on and cache off | A11 bytes, errors, and error order match the current whole path. Refusal and a repeated call leave bytes and live handles unchanged | A fast acceptance weaker than the whole path, a dropped error, or a changed refusal byte fails |

Omitted permutations are the full cross product of those axes. Each axis above, including C7 and C8, has one discriminating case. Same-consumer round trip does not replace the native, Python, stored, layout, and reopen comparison named in A3 and A5.

Native groups go in the existing `crates/rdocx/tests/integration_test.rs`:

- `checked_body_insertions_refuse_unsavable_namespace_owners_atomically`
- `word_owner_piece_sources_match_the_full_physical_inventory`
- `word_owner_pieces_follow_splices_and_fact_changing_publishers`
- `unchanged_main_staging_preserves_save_bytes_and_error_order`
- `unchanged_main_staging_reuse_counts_actual_work`
- `in_memory_reopen_matches_zip_for_new_flat_limited_and_hook_paths`
- `in_memory_reopen_keeps_related_part_errors_and_drops_stale_main_checks`
- `identifier_scan_replays_exact_bytes_categories_and_old_error_order`
- `parsed_none_reopen_grows_zip_parse_and_identifier_bytes`
- `body_spans_match_existing_to_xml_on_current_fixtures`
- `main_body_layout_raw_blocks_match_the_old_scanner`
- `main_body_layout_whole_walk_and_fallback_counts`
- `main_body_layout_mixed_owners_follow_old_scans`
- `main_body_layout_serialized_facts_and_frame_use_scanner_bytes`
- `main_body_layout_reopen_reuses_unchanged_children`
- `main_body_layout_multiple_body_owners_use_full_reader`
- `main_body_layout_coverage_steps_scale_with_children`
- `main_body_block_reuse_preserves_physical_owners_and_publication`
- `body_lookup_work_is_linear_at_250_and_1000`
- `staging_reads_only_the_main_part_blocks_an_edit_changes`

Python groups go in the existing `crates/rdocx-py/tests/test_core.py`:

- `test_checked_body_insertions_preserve_owners_on_namespace_refusal`
- `test_word_owner_pieces_follow_edits_splices_and_related_owners`
- `test_unchanged_main_staging_keeps_forms_and_settings_authoritative`
- `test_unchanged_main_staging_counts_actual_work`
- `test_header_story_text_reopen_counts_the_changed_header_part`
- `test_fx268_body_locations_keep_physical_slots`
- `test_body_block_reuse_preserves_forms_settings_and_live_owners`
- `test_staging_writes_and_reads_only_the_main_part_blocks_an_edit_changes`
- `test_a_fragment_imports_into_a_main_part_whose_root_has_another_word_prefix`

The implemented mechanism 6 span case is inline in `crates/rdocx-oxml/src/document.rs`. The implemented address cases are inline in `crates/rdocx/src/document.rs`. `test_fx268_body_locations_keep_physical_slots` is that binding node. It does not replace `test_body_block_reuse_preserves_forms_settings_and_live_owners`, `body_lookup_work_is_linear_at_250_and_1000`, or the A3 and A5 forms, settings, provenance and composition criteria. Those broader composition checks are separate from address coverage. `main_body_block_reuse_preserves_physical_owners_and_publication` is the implemented broader A3 composition proof. The seven address cases do not replace it. The last two staging-counter slots are populated by mechanism 7. The broader native and Python A3/A5 compositions and A10 controls are implemented. Their current execution and remaining acceptance are recorded in the progress notes. The original five native groups and four Python groups stay. A9 timings run only in the final union. The new named tests cover a forced fallback, a cache-off build, an invalidated context, and an actual RED on the old provenance. Inline unit helpers already in `document.rs` stay allowed.

`crates/rdocx-py/tests/typing_smoke.py` changes only if a public Python signature changes. None is expected. The native test calls all four new `try_` methods. Each local batch names one RED case on the old unsavable mutation, one GREEN case on a valid document, and one negative control. Worker verification is scoped `cargo fmt` and clippy on the changed crates. It is not the full suite, Word, the hash harness, or the performance rider.

The Q19 RED captures the model, the bytes, and the live handles at the point where the old save already fails. It does not treat two failing `to_bytes` results as proof that nothing changed.

## Rejected alternatives

- Importing PR415, PR417, PR418, PR419, PR420, or PR421 as a wholesale merge.
- Trusting semantic `PartialEq` as the only cache proof.
- Dropping the full-rebuild fallback in order to keep a fast path.
- Making the old infallible Rust methods panic or return `Result`. Q21 rejects both.
- Claiming the whole 252-call catalogue is constant, or copying a donor benchmark as credit.
- Treating the 14 A10 counted-work calls as a substitute for the 79 flat gates.
- Leaving the six Q24 calls measured only after the user required all 79 flat-cost gates.
- Reusing a field-bearing block because `PartialEq` returned true.
- Persisting a second shadow form model beside the integrated F-288, F-289, and F-290 facts.

## Test plan

| Timing | Category | Test | Asserts |
|---|---|---|---|
| local scoped | regression | The original five native groups and four Python groups | Old-source semantic RED on the unsavable mutation. Fixed-source GREEN. Negative control on cache-off or a skipped invalidation |
| local scoped | regression | The four added native groups and two added Python groups | Forced fallback, cache off, invalidated context, and actual RED provenance. A9 timings stay out of this row |
| local scoped | regression | A10 native group and the two A10 Python groups | Equal six-slot counts. The first four slots are byte counts and are zero, with the fragment byte exception. The last two slots are child counts and stay equal. They are not byte counts. Reopen bounds, cache-off scaling, and alternate-root whole read stay. The approved set adds the five Python calls and the three native cases. A10 is not a timing promise |
| local scoped | regression | A11 and C8 | Release fallback, error order, and the named context cases. Approved with mechanism 8 |
| local scoped | regression | Four `try_` methods | Index, text, zero-column, and namespace results. Old four methods unchanged |
| final sprint | performance | Release Python 3.12, the 79 calls, the five A9 calls, the 252 catalogue, and the original 407 gate | A6 and A9 bounds and an individual residual report |
| final sprint | memory | The 20000-paragraph RSS rider | A7 samples. No invented cap |
| final sprint | risk rider | `/verify --full`, hash, rustdoc, WASM, publish dry-run, installed stubs, and the prior seven Word and SSIM gates | A8. Unrun until the union |

The test gate from HLD 14 is regression. Local regressions are the readiness evidence. The performance, memory, Word, hash, and full-suite riders stay pending until final acceptance. No worker runs those riders.

## HLD impact

- `docs/hld/03-architecture.md`, Owned publication and causal paths
- `docs/hld/04-opc-and-packaging.md`, The package, and Checked Word operation source ownership
- `docs/hld/10-bindings-spec.md`, Python Word document operations
- `docs/hld/12-testing-strategy.md`, Binding tests, and Word document-operation parity controls
- `docs/hld/14-development-backlog.md`, the F-X268 definition

`docs/hld/02-scope-and-non-goals.md` is unaffected. No edit. It is not in the impact list.

Implementation leases, existing files only:

- `crates/rdocx/src/document.rs` and `crates/rdocx/src/lib.rs`
- `crates/rdocx-oxml/src/document.rs`, `crates/rdocx-oxml/src/text.rs`, and `crates/rdocx-oxml/src/borders.rs` only where a fact helper needs them
- `crates/rdocx-py/src/document.rs`, `crates/rdocx-py/src/lib.rs`, `crates/rdocx-py/src/paragraph.rs`, `crates/rdocx-py/src/run.rs`, and `crates/rdocx-py/src/table.rs`
- `crates/rdocx/tests/integration_test.rs` and inline units already in `document.rs`
- `crates/rdocx-py/tests/test_core.py`
- `crates/rdocx/src/content_control.rs`, only the existing indexed element lookup and `collect_control_ids`, composed with the final F-289 provider
- `crates/rdocx-oxml/README.md`, only when an actual final inventory needs it

No new source file, module, or test binary. README, the rdocx crate README, and `scripts/readme_doctests.py` change only for an actual final archive inventory, on the same pattern as a reviewed measurement, and not by copying donor figures. Stable Python is unchanged, so no new stub export is added.

## Risk routing

Matched rows of `.claude/skills/risk-routing.md`, each added to the existing final union and not run per worker:

- Public API of a published crate. The four `try_` methods are stable Rust. State the semver note. `cargo publish --dry-run` and the `.crate` size assertion stay under 10 MiB.
- WASM or PyO3 bindings. `cargo check --target wasm32-unknown-unknown`. Workspace tests keep `--exclude rdocx-py --exclude rpptx-py`.
- Any parser or serialiser. Namespace replay and saved bytes stay exact apart from the requested edit.
- Cached read facts or projections. Every fact-changing publisher is in A3, including an unchanged generation.
- Performance budget or bounded-edit optimization. A6, A9, and the local counters.
- Shared semantic behavior with multiple consumers. Native, Python, stored package, layout invalidation, and reopen.
- Public API accepting raw XML bytes. Opaque bytes stay. The new refusal is only the namespace-owner save failure, not a general XML validator.
- Grammar, scope, compatibility mode or empty collection. The C1 through C8 matrix.
- Contributed PR or new issue acceptance. The PR415, PR417, PR418, PR419, PR420, PR421, and PR422 heads are pinned and are not accepted code. The PR419 Actions logs are donor evidence only. PR421 CI was incomplete at the refresh3 cutoff. PR422 donor checks are not our pass. The legacy combined status `pending` is a literal record and is not a claim that CI is incomplete.
- Compatibility, numeric conversion or collection ordering. Owner order, error order, and the retained 407 gate.

Rustdoc warnings are denied. Installed stub tests run in the union. F-288 A6 stays the native Word gate in that plan: Word-created originals and library-created files for all three kinds, with stored-property equality separated from Word-open formatting and calculation effects. It is unchanged and it is not an SSIM gate. F-289 A7 stays equal page count, exact text and checkbox semantics, per-page SSIM at least 0.95 at 144 DPI, matched fonts, deterministic drawing, and documented margin alignment. It is unrun. Other original story riders stay unchanged. No new Word oracle and no performance comparison against Word is invented. F-X267 M21 behavior stays on that story and is not attributed to F-X268.

## Hash harness

Expected unchanged for existing fixtures touched by F-X268. Do not re-record a baseline to hide a byte change. The Q19 commit may make a previously unsavable insertion fail earlier, and only on the new checked Rust methods and the Python checked path. That commit is labelled and reviewed on its own. No new render byte change is claimed. The F-X267 M21 digest stays that story's declared behavior.

## Implementation checklist

- [x] Add the four `try_` methods without changing the old four signatures.
- [x] Teach the named Python delegates to refuse before mutation.
- [x] Keep the qualified splice and namespace proof on current facts.
- [x] Add the hidden piece and staging helpers with the captured signatures.
- [x] Count real walk, parse, and serialization work.
- [x] Land the behavior commit separately from the performance commit.
- [x] Leave the 79-call, A9, 252-call, memory, Word, and hash riders for the final union.
- [x] Add the eligible in-memory reopen and the direct-body span helpers with the captured signatures.
- [x] Keep `content_control.rs` to the indexed lookup and `collect_control_ids`, composed with the final F-289 provider.
- [x] Add the hidden frame and item helpers, `holds_field`, and the six-slot staging counter.
- [x] Add the mechanism 8 helpers with the captured signatures, the A10 additions, and A11. The contract is approved. Local execution and independent review apply as recorded in the current readiness and progress notes. Final sprint acceptance remains pending.

## Open questions

| ID | Affected F-IDs and A-IDs | Decision and recommendation | Status and answer/source |
|---|---|---|---|
| Q18 | F-X268 A5 and A7 | Allow caching with measured memory use. Actual memory and fallback measurements and unchanged correctness are mandatory. No hard cap was chosen. The donor 68 MiB figure is unverified. | Answered 2026-10-24 by the user. "Allow caching with measured memory use (Recommended)". Recorded in the S95.4 issue and PR refresh, 2026-10-24. |
| Q19 | F-X268 A1 | Refuse atomically in Rust and Python before mutation when a retained namespace owner would make saving fail. Valid documents keep working. Separate labelled behavior commit. | Answered 2026-10-24 by the user. "Refuse atomically in Rust and Python (Recommended)". Refined by Q21. |
| Q20 | F-X268 A6 | Targeted 100-paragraph time is at most 1.5 times the 25-paragraph time plus 5 ms. Report every 252-call residual. Keep the original 407 gate. | Answered 2026-10-24 by the user. "Use these bounds and report residuals (Recommended)". |
| Q21 | F-X268 A1 | Add the four checked Rust methods. The original four infallible methods keep legacy behavior and are exempt from Q19. Python uses the checked path without new names. | Answered 2026-10-24 by the user. "Add checked Rust methods (Recommended)". |
| Q22 | F-X268 A7 | Apply the measured-memory policy to the PR419 and PR420 caches. Measure each layer and the combined footprint. Do not invent a cap and do not accept the donor 68 MiB figure. | Answered 2026-10-24 by the user. "Apply the measured-memory policy (Recommended)". Root approved the extension the same day. |
| Q23 | F-X268 A6 and A9 | Add the 8 flat calls to the Q20 bound and add the five linear calls at 250 and 1000 paragraphs. Residuals and the 407 gate stay. | Answered 2026-10-24 by the user. "Use the added flat and linear gates (Recommended)". Root approved the extension the same day. |
| Q24 | F-X268 A6 | Require all 79 flat-cost gates. The current set is 76 catalogue keys plus the same 3 XML reads. The six additions are `Paragraph.replace_text`, `Cell.replace_text`, `Document.replace_text_at`, `Document.add_picture`, `Cell.add_picture`, and `Document.add_hyperlink_to_story`. The bound stays 100 at most 1.5 times 25 plus 5 ms. All 5 A9 linear calls stay at 250 and 1000, including the 2 that overlap, and all 252 stay reported. The historical 73 stays the Q23 record. Current residuals from that old trio are `Run.add_picture` and `Cell.merge`. | Answered 2026-10-25 by the user. `Require all 79 flat-cost gates (Recommended)`. Root approved the refresh4 contract the same day at baseline `d1f42516eb3747442fe4304a6331eab946dc9a0e`. |

Q1 through Q24 stay answered and Q1 through Q23 are not rewritten. Q18, Q19, Q20, and Q21 belong to the earlier approved contract. Q22 and Q23 belong to the approved refresh2 extension. The approved refresh3 extension opened no question. Q22 already measures the added cache layers, the combined footprint, and the refresh4 `decodes` fact. No open question remains. Private implementation and the Q22 memory policy are Root judgment, not a new user question.

## Contract closure

Codex root approved the contract that preceded this amendment on 2026-10-24 after reading the full draft and the revision delta. The canonical HEAD at that approval is `c0b58c7fa8e4e919b9bfe5c02932f74827cea36b`, plus the working draft of that day. Q18, Q19, Q20, and Q21 stay answered. That approval left the original 65 targeted calls, the split of 62 catalogue keys plus 3 reads, the 252-call catalogue, the cache error rule, the counter split, the memory rule, and the public APIs as then written. The 7 to 12 developer-day figure stays the historical estimate for that approved scope.

Codex root approved the refresh2 extension on 2026-10-24 after reading the full revised contract and the four-file delta. The canonical HEAD at that approval is `0d8dfb68396e33ddfcaa0d9400c30c006e12d2b7`, plus this working draft. The extension revises the combined-scope estimate to 10 to 17 developer-days and adds mechanisms 5 and 6, the 8 flat calls, and A9. No implementation has started. No gate pass is claimed.

Codex root approved the refresh3 extension on 2026-10-24 after reading the full corrected contract and the four-file delta. The canonical HEAD at that approval is `d6cefc239ab7de0d4dae63e13d730da558d3a3a5`, plus this working draft. Fact cutoff is 2026-10-24T17:02:28Z. The extension is the same F-X268 story and its seventh mechanism. It adds A10 and C7. The new helpers stay the captured signatures. The staging counter stays six units. The 73 flat calls, the 5 linear calls, and the 252-call catalogue stay the same. Q22 covers the added cache layers. No new question is opened and no timing promise is added. The 10 to 17 developer-day figure stays the historical Root judgment for the scope before PR422. It is not a measurement and not a capacity promise. PR421 stays an unmerged candidate and is not accepted code. No implementation has started. No gate pass is claimed. This paragraph records that Root review. It is not a self-review.

Codex root approved the refresh4 extension on 2026-10-25 after reading the reconciled four-file delta. The user answered Q24 the same day with `Require all 79 flat-cost gates (Recommended)`. The canonical baseline is `d1f42516eb3747442fe4304a6331eab946dc9a0e`. Fact cutoff remains 2026-10-24T23:05:39Z. The extension is the eighth mechanism. It adds A11 and C8 and extends A10 counted work from 9 calls to 14. The current flat set is 79. The combined planning judgment is a provisional 12 to 21 developer-days. The 10 to 17 figure stays historical and is not added to the seven-story 27 to 41 figure. PR422 stays an unmerged candidate and is not accepted code. F-X268 stays unclaimed. F-290 is unaffected. No implementation has started. No gate pass is claimed. This paragraph records that Root approval. It is not a self-review.
