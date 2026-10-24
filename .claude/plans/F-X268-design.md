# F-X268, Bound Word owner inventory and unchanged package staging

**Status**: approved
**Sprint**: S95.4
**Size**: L
**Depends on**: F-X266, F-X267
**Estimate**: 7 to 12 developer-days. This is a provisional Root planning estimate. It is not a measurement and not a capacity promise. Do not add it to the historical seven-story figure of 27 to 41 developer-days.

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

One story, four serial mechanisms. The Q19 and Q21 behavior is its own labelled commit. Performance and memory steps are separate commits. No stack is reimported. The reviewed F-X266 and F-X267 source stays. F-288, F-289, and F-290 keep their approved contracts. Their fact-changing publishers are named in the consumer map.

Captured increment boundaries, not the stacked diffs against main:

- Old PR415 `0fbf3e96709198dbd01b9e810181109d106d9f68` to new PR415 `2839e22c14f96aeaeab5245581b5681509774559` is a diverged endpoint tree of 9 paths: `README.md`, `crates/rdocx-oxml/README.md`, `crates/rdocx-oxml/src/document.rs`, `crates/rdocx-py/src/document.rs`, `crates/rdocx-py/tests/test_core.py`, `crates/rdocx/src/document.rs`, `docs/hld/03-architecture.md`, `docs/hld/10-bindings-spec.md`, `scripts/readme_doctests.py`.
- PR417 `bdf49f0005d1d15078b76b5da83c5f3e2b8cce12` has six own commits after the new PR415 head. Its own tree is 14 paths: `README.md`, `crates/rdocx-oxml/README.md`, `crates/rdocx-oxml/src/borders.rs`, `crates/rdocx-oxml/src/text.rs`, `crates/rdocx-py/src/document.rs`, `crates/rdocx-py/src/lib.rs`, `crates/rdocx-py/src/paragraph.rs`, `crates/rdocx-py/src/run.rs`, `crates/rdocx-py/src/table.rs`, `crates/rdocx-py/tests/test_core.py`, `crates/rdocx/src/document.rs`, `crates/rdocx/src/lib.rs`, `docs/hld/03-architecture.md`, `scripts/readme_doctests.py`.
- PR418 `dba496fcd8b4d482995a23ab81d27ad2bfcbf5e7` has six own commits after merge `0912a40e19950b223745ef83e8164266b4c36b66`. The merge parents are the PR417 head and PR416 head `768f8c109bd43fb9c1f6ecdfe2d913382bcc0368`. Its own tree is 8 paths: `README.md`, `crates/rdocx-py/src/document.rs`, `crates/rdocx-py/src/lib.rs`, `crates/rdocx-py/tests/test_core.py`, `crates/rdocx/src/document.rs`, `crates/rdocx/src/lib.rs`, `docs/hld/03-architecture.md`, `scripts/readme_doctests.py`.

Donor timings and the donor 68 MiB claim are unverified. README and crate README numbers in those trees are not copied.

### 1. Qualified body splice and namespace replay

Amend the integrated F-X266 qualified body splice so an unrouted paragraph facade position still uses the checked route. Nested namespace replay is scoped to the exact current XML, model, and namespace-owner facts. A proof is not reused after an edit. Compatible positive owners stay. Do not adopt the donor's incidental redundant saved-namespace byte delta. Existing package bytes stay unchanged except for the requested edit.

### 2. Owner pieces

Keep one piece inventory per body block and per related physical part. Reuse on equality needs serialization and fact provenance. Semantic `PartialEq` alone is not proof. Cover the existing field source for form, span, derived parent cache, default, current, and typed nested cache, the tab source occurrence, and bookmark prefix facts, through the public and native publishers that change them. Reconstruct the same owners, parents, routes, projections, read facts, order, scope births, generations, and retired set as the checked full old inventory. Grammar is the exact qualified physical grammar, including independent text box ownership. Before any partial publish, an unsupported shape, a duplicate physical part, an invalid relationship, or a context change uses the old full rebuild. An error does not shift semantics.

### 3. Opened package staging

Share the opened package, the parsed main part, the retained owners, and the check record with staged clones. Reopen resets that baseline. Reuse only checked unchanged main bytes plus model serialization, and only the relationship, section, drawing, namespace, media, related-part, signature, and embedded facts that were checked. `OpcPackage::reread` keeps its current `trusted_baseline` precondition. Public part maps are not assumed immutable. A save-equivalent compare requires exact bytes and the same errors in the same order. A new or in-memory document falls back to the old ZIP compare. `render_template`, `compare`, and `rebuild_toc` keep the old `to_bytes` error order. A related part that changed is still checked when the main part is unchanged. A failed cache check is not success. A scope-preserving property edit, callback, or direct candidate assignment invalidates the facts it changes even when the generation is unchanged. An opaque change is not ignored. Legacy form defaults, current values, placeholders, custom XML, protection, permissions, and mail-merge stores and settings compose. Dead owners are not revived.

### 4. Binding publication

Python publication uses the prepared inventory. It allocates every result object and revision token before commit. A no-op and a refusal keep the old bytes, revisions, and live handles. Conservative publication still retires prior scopes. Append keeps only eligible owners. An owned old snapshot stays old. A returned handle uses the prepared proved route or the old fallback. It does not start a new whole-document walk.

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

Adopt a donor helper only where the Python binding or `Document` is an actual consumer. The declarations below are methods, not free functions. None of these is a stable Python, WASM, or CLI export. `BodyItemContext` is the whole fact source for piece reuse, and a caller must hold that proof. No forwarding wrapper, trait, generic, module, crate, feature flag, or dependency is added. No new trait or generic is planned. The repository rule that needs two implementers or two instantiations applies to traits and generics, not to this method. `CT_Body::paragraph_count_before` is the concrete count. `Document::paragraph_count_before_content` is one call chain into that method, not a second independent consumer. A public cross-crate count helper needs the actual `Document` consumer. It does not need an arbitrary second consumer.

Optional eligibility `None` chooses the checked full path. A helper `Err` may take that full path only where the full path returns the same checked result, error, and error order the old path would have returned before publication. A full-path `Err` propagates through the existing native and Python mapping. Invalid ownership or invalid XML never becomes empty facts or success. `unsavable_body_edit` returning `None` is not a general proof that the document is savable.

Captured from PR417 `crates/rdocx/src/document.rs`. The structs are public doc(hidden) cross-crate Rust types. The functions are methods:

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

## Consumer map

| Rule | Entry points and outputs | Shared provider and caller policy | Lifecycle stages compared |
|---|---|---|---|
| Checked insert | `try_add_paragraph`, `try_insert_paragraph`, `try_add_table`, `try_insert_table`. Python `add_paragraph`, `insert_paragraph`, `add_table`, `insert_table`, `insert_paragraph_before`, `add_heading`, `add_page_break` | Q21. Old four infallible methods stay legacy. Binding keeps its signatures | Evaluate the guard, then update, then serialize. Reopen the saved package |
| Namespace replay | `unsavable_body_edit` against current XML, model, and owners | Proof dies when the owner, local namespace, text, or typed field changes | Before mutation and again after a fact change |
| Piece inventory | `body_item_owners`, `body_item_context`, `body_children`, story part helpers | Binding publication is the consumer. Context equality is caller proof | Read, project, serialize, reopen |
| Staging | `saves_as` and the opened baseline shared with a staged clone | OPC `trusted_baseline` stays. Related parts stay checked | Save, error order, reopen, drop |
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

## Dependency readiness

F-X266 source `9c6ef377971182a73b2e3cef07ccedde579d2f37` is integrated at `9ee48bf0902d976de016553b1dd0c79fed856612`. F-X267 source `70573c91030be7073d29e3451383060eb207207a` is integrated at `d0f17a6ac7ef675d0ed2a86cea641ce21d65bfdd`. Both are reviewed and in progress. Neither is done. Approval of those plans is not final acceptance.

Proposed local prefix, for Root to approve before F-X268 implementation. This draft does not edit the provider plans and does not mark those providers done. The next provider-plan amendment may cite these existing tables unchanged. It does not weaken their final performance, hash, or Word riders.

The prefix names every local scoped row already in those plans. Outcomes and commands come from the existing progress records. This draft does not invent a combined pass.

F-X266 test plan, all 9 local scoped rows: `direct_routes_match_full_body_bytes`, `append_keeps_eligible_owners`, `conservative_retires_all_scopes`, `failed_splice_publishes_nothing`, `fragment_omits_unused_w14`, `owner_scope_precedence_unchanged`, `held_fresh_reopen_match_old_semantics`, `body_work_counters_follow_owners`, `missing_run_path_refuses`.

F-X267 test plan, all 13 local scoped rows: `remove_part_returns_shared_bytes`, `reread_matches_write_and_read`, `reread_matches_shared_zip_name_limits`, `hand_built_baseline_uses_fallback`, `untouched_slide_bytes_stay`, `fit_text_ignores_unrelated_broken_slide`, `fit_text_at_selects_size_valid_in_distinct_shared_contexts`, `footer_proof_is_part_local`, `parsed_baseline_tracks_real_edits`, `fx211b_native_opaque_root_edit_retains_invalidated_signature`, `embedded_replace_counts_commit_zip_write`, `unnormalized_opaque_part_counts_reread_zip_fallback`, `commit_counters_follow_real_work`.

The prefix also consumes the actual scoped checks, clean Root reviews, and the current applicable artifact proof. An old artifact is not reused as a fresh qualifier. Rerun only the inputs a later change invalidates, including a checked build and import where binding evidence is affected. No full oracle rider runs at the prefix. Word, the full suite, the hash harness, the final release 65-call and 252-call runs, the F-X267 248-call run, and the memory rider stay pending. Before any F-X268 claim, verify that the F-X266 and F-X267 source heads above are actual ancestors, that this prefix is actually done, and that binding source proof exists where the binding is affected. `AS_BUILT.md` and `SPRINT_TRACKER.md` are written once, after the full union. A formal consumer claim waits for the actual done checkpoint.

F-289 may finish its own native checkpoint on its unchanged contract. Its interim native review 1 records 8 defects and is not clean. That review is not delivery credit. F-X268 does not start while the shared leases are held by F-289 and then F-290.

## Acceptance criteria

| ID | Input or setup | Expected observation or refusal | Consumers | Local test | Final gate or pending evidence |
|---|---|---|---|---|---|
| A1 | Q19 and Q21. Four new Rust methods and every named Python delegate. An old document whose save already fails. A valid document. Bad index, non-XML text, zero columns | Refusal is atomic. Model, bytes, and handles stay at the old unsavable state. The old four methods stay legacy. Valid bytes match the checked source. Errors keep the existing family | Native `Document` and Python document, paragraph, heading, and page break | `checked_body_insertions_refuse_unsavable_namespace_owners_atomically`, `test_checked_body_insertions_preserve_owners_on_namespace_refusal` | Local behavior commit. Not the performance commit |
| A2 | New PR415 unrouted facade position and nested namespace proof. Then an owner, local namespace, text, or typed field edit | First proof matches the checked source. The stale proof is not reused. Accepted aliases and unknown bytes stay. No incidental namespace byte change | Facade and fragment writer | `word_owner_piece_sources_match_the_full_physical_inventory` | Local regression |
| A3 | Piece inventory against the full inventory for body, nested cell, text box, comment, note, shared header, duplicate part, invalid relationship, identical blocks, and a direct returned owner. The publishers named in the consumer map, including direct candidate assignment and property callbacks | Equal owners, parents, routes, projections, read facts, order, births, generations, and retired set. Unsupported shape falls back before publish. Held, fresh, native, save, and reopen agree. Ignored serialization facts do not count as proof | Binding and native readers | `word_owner_piece_sources_match_the_full_physical_inventory`, `word_owner_pieces_follow_splices_and_fact_changing_publishers` | Local regression on the integrated ancestor that contains those publishers |
| A4 | Splices at head, middle, and tail, plus a missing item. A later edit. No-op and refusal | Route and later edit stay inside the work bound. Conservative births and retirement match the old path. Dead owners stay dead. No-op and refusal preserve bytes, revisions, and handles | Python publication | `word_owner_pieces_follow_splices_and_fact_changing_publishers`, `test_word_owner_pieces_follow_edits_splices_and_related_owners` | Local regression |
| A5 | Opened cache and `saves_as` for new, opened, clone, source-edited, relationship, media, signature, and embedded packages. Main unchanged with a changed related part | Exact bytes and native error order, or the ZIP fallback. Related changes are checked. A warm edit does not reparse an unchanged main part | OPC and binding | `unchanged_main_staging_preserves_save_bytes_and_error_order`, `unchanged_main_staging_reuse_counts_actual_work`, `test_unchanged_main_staging_keeps_forms_and_settings_authoritative` | Local regression |
| A6 | The 65 targeted calls and the full 252-call catalogue on a fresh release install | The 65 meet the Q20 bound. The 252 results and residuals are reported without xfail hiding. The original 407 gate still holds for its three calls | Installed Python 3.12 | Named in the performance section. Not a worker run | Final union |
| A7 | Q18 memory workload | Actual RSS samples at the named points. No 68 MiB pass line and no hard cap | Fresh release process | Named in the memory section | Final union |
| A8 | Installed Python, strict mypy, stubtest, rustdoc, publish archive, WASM, hash, and the prior seven stories | Prior Word, SSIM, risk, and hash gates stay mandatory and stay unrun here. Q19 compares the previous save failure, model, and handles. Two failing `to_bytes` calls are not atomicity proof | Final union | None in the worker | Final union |

The 65 targeted calls are 62 catalogue keys plus `Run.xml`, `Table.xml`, and `Cell.xml`. The pinned 252 does not cover those three reads.

Targeted calls from PR415, 13:

`Document.insert_paragraph`, `Document.remove_content`, `Paragraph.insert_paragraph_before`, `Document.remove_content_range`, `Document.add_paragraph`, `Document.insert_table`, `Document.add_table`, `Document.add_heading`, `Document.add_page_break`, `Paragraph.xml`, `Run.xml`, `Table.xml`, `Cell.xml`.

Targeted table, row, and cell setters from PR417, 26:

`Table.indent`, `Table.alignment`, `Table.width`, `Table.set_borders`, `Table.set_border`, `Table.set_cell_margins`, `Table.grid_widths`, `Table.set_column_width`, `Table.set_cell_grid_span`, `Table.set_cell_vertical_merge`, `Table.autofit`, `Table.first_row`, `Table.last_row`, `Table.first_column`, `Table.last_column`, `Table.banded_rows`, `Table.banded_columns`, `Row.height`, `Row.height_rule`, `Row.cant_split`, `Row.is_header`, `Cell.width`, `Cell.vertical_alignment`, `Cell.shading`, `Cell.set_border`, `Cell.set_margins`.

Other PR417 calls, 16:

`Cell.add_paragraph`, `Paragraph.text`, `Paragraph.add_run`, `Paragraph.add_hyperlink`, `Paragraph.add_equation`, `Run.remove`, `Run.add_field`, `Document.split_run`, `Document.add_comment`, `Document.reply_to`, `Document.resolve_comment`, `Document.add_chart`, `Document.set_text_watermark`, `Document.set_image_watermark`, `Document.set_note_policy`, `Settings.odd_and_even_pages_header_footer`.

PR418 calls, 10:

`Document.set_font`, `Document.embed_font`, `Document.remove_embedded_font`, `Document.remove_font`, `Document.embed_true_type_fonts`, `Document.save_subset_fonts`, `Document.embed_system_fonts`, `Document.add_numbering_definition`, `Document.add_numbering_instance`, `Document.link_style_to_numbering`.

That is 65 unique calls. `Run.xml`, `Table.xml`, and `Cell.xml` are the three additional required read timings. They are not catalogue keys.

### Post-pin public surface audit

The immutable 252-call catalogue stays. Its timed coverage checker predates F-288, F-289, and F-290. The final union public stub inventory enumerates every added or changed method and maps each new read-only, factory, global, and local method to its F-ID, its A-ID, and its timing disposition. Do not relabel a local method `WHOLE_DOCUMENT` only so the old checker passes. A transparent qualification driver may adapt only the post-pin coverage classification, with the new methods and the reasons named. It preserves all 252 old keys and all 65 required timings. It does not skip or drop broadly. A new local method that is not in the pinned 65 stays outside the Q20 timing promise unless the original authoring plan already owns a performance criterion. Correctness, runtime, and stub tests for those methods stay mandatory. Report the expanded inventory count separately. Do not claim that 252 covers it.

The full catalogue is 252, which is 250 timed calls plus 2 reads: 216 literal timed calls, 15 CoreProperties calls, 13 Section calls, 6 App calls, and 2 reads. Of those, 145 were the old growing set and 107 were the old non-growing set. Run all 252 without hiding status behind xfail. Report each slow residual. Do not copy a donor pass count. Existing non-growing calls that are not in the 65 keep the original bound of 2.5 times N plus 5 ms. Each of the 65 uses the Q20 bound: the 100-paragraph time is at most 1.5 times the 25-paragraph time plus 5 ms.

The final consumer is a fresh release install of Python 3.12 from the actual union source head, with the artifact SHA, interpreter, and config recorded. The external builders use 25 and 100 paragraphs, six formatted runs, a picture or table every 10th paragraph, two sections, a header, comments, and a control, for the qualified targets in the pin `4646db50ed783ddf4a2220f4b95a15ad7a225944`. Sampling alternates N and 4N, up to 3 fresh changed targets, and keeps the minimum the way `costs_ms` does. Two samples may stop early only when both fastest samples are above 50 ms. Setup, open, and build stay outside the clock. Every raw sample is retained. There is no concurrent compilation. The first sample is cold and stays visible. Warm statistics are separate. A cold walk and a fallback cost are disclosed. A warm minimum is not a constant-time claim. The original Issue 407 gate stays: mean of 3, 100 versus 1000 paragraphs, six verbose run-property runs, `large <= 3 * max(small, 1 ms)`, plus a separate fresh median of 7, for both cold and warm, on the original F-X266 methods. A native benchmark does not replace the Python public lookup, validation, and publication.

Local counters sit at the real walk, parse, and serialization sites. Separate the owner-projection phase from the total public call. On a qualified case, 20 versus 80 or 50 versus 200 unrelated body blocks of equal piece count, the piece owner projection reads at most 2 pieces, and at most 1 piece after a qualified top splice or tail removal. That phase does not serialize an unrelated body block. Underlying staging may still serialize the current typed main part as the exact byte proof. Count those whole-main serialization bytes and that time separately. Do not hide them inside a zero reparse or zero canonicalization count. After the first unchanged main check, an eligible warm edit does 0 main reparses, 0 main canonicalizations, and 0 ZIP compare writes. That is not zero serialization for the whole call and it is not an O(1) whole call. The total public Python time covers lookup, validation, publication, and serialization. The existing F-X266 qualified direct-read and splice bound of no unrelated serialization stays unchanged. Misses and visited metadata are counted, so an O(N) metadata walk is disclosed and is not called O(1). Registry owner work keeps the existing F-X266 linear bound and is not quadratic. The cold proof body walk is counted separately. Native counters cover the current main parse, owner scan, canonicalization, and ZIP writes. Related-part checks are still counted. A constant entry counter is not proof. A build with the cache forced off, or a mutant that skips invalidation, must fail these asserts. Absence of a new helper is a compile failure, not that semantic RED. The debug full-path oracle is internal agreement, not external parity. Debug counters and release guards do not skip the production check.

Memory, final Q18 rider only. The workload is 20000 paragraphs, six runs each, with fixed text and run formatting. Measure and report the actual main XML length. The donor 6 MiB claim is unverified. Use 3 paired alternating fresh processes. Record the median current RSS and report each peak, with the raw samples and the aggregation named, after open, after the first font edit, after 100 distinct warm edits, after clone and drop, after reopen and drop, and after the document drop. Pair a pre-cache commit baseline with the final release, sharing the inventory and otherwise equal. Record the baseline source, build, artifact, allocator, and process boundary. The shared cache may retain the opened baseline, one parsed model, and one serialized copy, and the current changed parts, plus bounded projection facts for each live piece. It does not retain a history per edit. N edits and 4N edits must not retain one baseline per edit. Clone ownership and reopen release stay safe. There is no 68 MiB pass claim and no hard budget the user did not choose. Measure authored and new-document fallback, and the unsavable namespace, duplicate part, signed, and embedded fallback times, separately. Correctness is mandatory on those paths.

## Test matrix

| Case | Modes or state combined | Expected difference or agreement | Wrong implementation rejected |
|---|---|---|---|
| C1 | Zero, one, and many body blocks. Direct and nested. Alias and shadowed namespace | Guard and inventory match the full path | A stale proof after a text edit fails |
| C2 | Current versus default form value. Implicit and explicit protection | Facts change even when the generation does not | Semantic equality used as the only proof fails |
| C3 | New document and reopened package. Partial write and whole write | Bytes and error order agree, or the ZIP fallback runs | A main-unchanged skip that ignores a related part fails |
| C4 | No-op, refusal, and conservative splice | Handles and retired scopes match the old publication | A refusal that mutates, or a dead owner that returns, fails |

Omitted permutations are the full cross product of those axes. Each axis above has one discriminating case. Same-consumer round trip does not replace the native, Python, stored, layout, and reopen comparison named in A3 and A5.

Native groups go in the existing `crates/rdocx/tests/integration_test.rs`:

- `checked_body_insertions_refuse_unsavable_namespace_owners_atomically`
- `word_owner_piece_sources_match_the_full_physical_inventory`
- `word_owner_pieces_follow_splices_and_fact_changing_publishers`
- `unchanged_main_staging_preserves_save_bytes_and_error_order`
- `unchanged_main_staging_reuse_counts_actual_work`

Python groups go in the existing `crates/rdocx-py/tests/test_core.py`:

- `test_checked_body_insertions_preserve_owners_on_namespace_refusal`
- `test_word_owner_pieces_follow_edits_splices_and_related_owners`
- `test_unchanged_main_staging_keeps_forms_and_settings_authoritative`
- `test_unchanged_main_staging_counts_actual_work`

`crates/rdocx-py/tests/typing_smoke.py` changes only if a public Python signature changes. None is expected. The native test calls all four new `try_` methods. Each local batch names one RED case on the old unsavable mutation, one GREEN case on a valid document, and one negative control. Worker verification is scoped `cargo fmt` and clippy on the changed crates. It is not the full suite, Word, the hash harness, or the performance rider.

The Q19 RED captures the model, the bytes, and the live handles at the point where the old save already fails. It does not treat two failing `to_bytes` results as proof that nothing changed.

## Rejected alternatives

- Importing PR415, PR417, or PR418 as a wholesale merge.
- Trusting semantic `PartialEq` as the only cache proof.
- Dropping the full-rebuild fallback in order to keep a fast path.
- Making the old infallible Rust methods panic or return `Result`. Q21 rejects both.
- Claiming the whole 252-call catalogue is constant, or copying a donor benchmark as credit.
- Persisting a second shadow form model beside the integrated F-288, F-289, and F-290 facts.

## Test plan

| Timing | Category | Test | Asserts |
|---|---|---|---|
| local scoped | regression | The five native groups and four Python groups above | Old-source semantic RED on the unsavable mutation. Fixed-source GREEN. Negative control on cache-off or a skipped invalidation |
| local scoped | regression | Four `try_` methods | Index, text, zero-column, and namespace results. Old four methods unchanged |
| final sprint | performance | Release Python 3.12, the 65 calls, the 252 catalogue, and the original 407 gate | A6 bounds and an individual residual report |
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

No new source file, module, or test binary. README, the rdocx crate README, and `scripts/readme_doctests.py` change only for an actual final archive inventory, on the same pattern as a reviewed measurement, and not by copying donor figures. Stable Python is unchanged, so no new stub export is added.

## Risk routing

Matched rows of `.claude/skills/risk-routing.md`, each added to the existing final union and not run per worker:

- Public API of a published crate. The four `try_` methods are stable Rust. State the semver note. `cargo publish --dry-run` and the `.crate` size assertion stay under 10 MiB.
- WASM or PyO3 bindings. `cargo check --target wasm32-unknown-unknown`. Workspace tests keep `--exclude rdocx-py --exclude rpptx-py`.
- Any parser or serialiser. Namespace replay and saved bytes stay exact apart from the requested edit.
- Cached read facts or projections. Every fact-changing publisher is in A3, including an unchanged generation.
- Performance budget or bounded-edit optimization. A6 and the local counters.
- Shared semantic behavior with multiple consumers. Native, Python, stored package, layout invalidation, and reopen.
- Public API accepting raw XML bytes. Opaque bytes stay. The new refusal is only the namespace-owner save failure, not a general XML validator.
- Grammar, scope, compatibility mode or empty collection. The C1 through C4 matrix.
- Contributed PR or new issue acceptance. The three heads are pinned and are not accepted code.
- Compatibility, numeric conversion or collection ordering. Owner order, error order, and the retained 407 gate.

Rustdoc warnings are denied. Installed stub tests run in the union. F-288 A6 stays the native Word gate in that plan: Word-created originals and library-created files for all three kinds, with stored-property equality separated from Word-open formatting and calculation effects. It is unchanged and it is not an SSIM gate. F-289 A7 stays equal page count, exact text and checkbox semantics, per-page SSIM at least 0.95 at 144 DPI, matched fonts, deterministic drawing, and documented margin alignment. It is unrun. Other original story riders stay unchanged. No new Word oracle and no performance comparison against Word is invented. F-X267 M21 behavior stays on that story and is not attributed to F-X268.

## Hash harness

Expected unchanged for existing fixtures touched by F-X268. Do not re-record a baseline to hide a byte change. The Q19 commit may make a previously unsavable insertion fail earlier, and only on the new checked Rust methods and the Python checked path. That commit is labelled and reviewed on its own. No new render byte change is claimed. The F-X267 M21 digest stays that story's declared behavior.

## Implementation checklist

- [ ] Add the four `try_` methods without changing the old four signatures.
- [ ] Teach the named Python delegates to refuse before mutation.
- [ ] Keep the qualified splice and namespace proof on current facts.
- [ ] Add the hidden piece and staging helpers with the captured signatures.
- [ ] Count real walk, parse, and serialization work.
- [ ] Land the behavior commit separately from the performance commit.
- [ ] Leave the 65-call, 252-call, memory, Word, and hash riders for the final union.

## Open questions

| ID | Affected F-IDs and A-IDs | Decision and recommendation | Status and answer/source |
|---|---|---|---|
| Q18 | F-X268 A5 and A7 | Allow caching with measured memory use. Actual memory and fallback measurements and unchanged correctness are mandatory. No hard cap was chosen. The donor 68 MiB figure is unverified. | Answered 2026-10-24 by the user. "Allow caching with measured memory use (Recommended)". Recorded in the S95.4 issue and PR refresh, 2026-10-24. |
| Q19 | F-X268 A1 | Refuse atomically in Rust and Python before mutation when a retained namespace owner would make saving fail. Valid documents keep working. Separate labelled behavior commit. | Answered 2026-10-24 by the user. "Refuse atomically in Rust and Python (Recommended)". Refined by Q21. |
| Q20 | F-X268 A6 | Targeted 100-paragraph time is at most 1.5 times the 25-paragraph time plus 5 ms. Report every 252-call residual. Keep the original 407 gate. | Answered 2026-10-24 by the user. "Use these bounds and report residuals (Recommended)". |
| Q21 | F-X268 A1 | Add the four checked Rust methods. The original four infallible methods keep legacy behavior and are exempt from Q19. Python uses the checked path without new names. | Answered 2026-10-24 by the user. "Add checked Rust methods (Recommended)". |

No open question remains. Answered questions do not approve this draft.

## Contract closure

Codex root approved this contract on 2026-10-24 after reading the full draft and the revision delta. The canonical HEAD at approval is `c0b58c7fa8e4e919b9bfe5c02932f74827cea36b`, plus this working draft. Q18, Q19, Q20, and Q21 stay answered. The 65 targeted calls, the split of 62 catalogue keys plus 3 reads, the 252-call catalogue, the cache error rule, the counter split, the memory rule, and the public APIs are unchanged by this approval. The 7 to 12 developer-day figure stays a provisional planning estimate. No implementation has started. No gate pass is claimed.
