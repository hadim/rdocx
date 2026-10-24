# F-X266, Bound Word body reads and structural edit cost

**Status**: approved
**Sprint**: S95.4
**Size**: L
**Depends on**: F-X260

## Problem

Direct body reads and structural edits still walk more of the package than the changed owner. Issue 407 compares 100 paragraphs with 1000 paragraphs, each with six verbose formatted runs, on the original public calls. PR415 `0fbf3e96709198dbd01b9e810181109d106d9f68` sha256 `e459336d8eada8059b2f01b96349546bb2d03e4c4a2daaa1734881d093c66a0c` is a candidate. It was not executed. Donor timings are not evidence.

Base `a57be365` does not contain the helper signatures below. They are the proposed contract copied from that pinned diff and from `/private/tmp/s95.4-contributions/contracts.txt`.

## Spec reference

- `docs/hld/10-bindings-spec.md`, Python Word document operations.
- `docs/hld/03-architecture.md`, Owned publication and causal paths.
- `docs/hld/04-opc-and-packaging.md`, Checked Word operation source ownership.
- `docs/hld/12-testing-strategy.md`, Binding tests, and Word document-operation parity controls.
- `docs/hld/14-development-backlog.md`, F-X266. Test gate is regression.

## Approach

No Python public signature change. No new file, trait, or test binary. Planning estimate is 4 to 6 developer-days, provisional and not a measurement.

Bindings keep an owner index and splice path for direct body reads and edits, with a fallback to the existing rebuild when the plan is not proved. Eligible owners are carried only by `AppendBody`. `Conservative` insert and remove still retire every scope. There is no promise that a held handle survives every edit.

Hidden public cross-crate providers. They are not stable Python APIs. The method-privacy check is the targeted patch. They earn the published native API rider.

```rust
impl CT_Document {
    pub fn standalone_body_item_xml(&self, item: &BodyContent) -> oxml_core::Result<Vec<u8>>;
}
impl StoryId {
    pub fn set_owner_index(&mut self, owner_index: usize);
}
impl ContentLocation {
    pub fn set_index_path_head(&mut self, index: usize);
}
impl Document {
    pub fn direct_body_paragraph_xml(&self, slot: usize) -> Result<Option<Vec<u8>>>;
    pub fn direct_body_run_xml(&self, slot: usize, path: &AcceptedRunPath) -> Result<Option<Vec<u8>>>;
    pub fn direct_body_table_xml(&self, slot: usize) -> Result<Option<Vec<u8>>>;
    pub fn direct_body_cell_xml(&self, slot: usize, row: usize, cell: usize) -> Result<Option<Vec<u8>>>;
    pub fn direct_body_paragraph_facts(&self, paragraph: &CT_P) -> Result<Option<(String, Vec<AcceptedRunPath>)>>;
    pub fn direct_body_story_item_count(&self, slot: usize) -> Result<Option<usize>>;
    pub fn plain_removable_paragraph(&self, slot: usize) -> bool;
    pub fn detached_table_owner_snapshots(&self, table: &CT_Tbl) -> Result<(Vec<u8>, (Vec<StoryItemSnapshot>, Vec<(ContentLocation, usize, usize, StoryId)>, Vec<(ContentLocation, Vec<u8>)>, Vec<(ContentLocation, LinkInfo)>))>;
    pub fn detached_text_paragraph(text: &str) -> CT_P;
    pub fn detached_table(rows: usize, cols: usize) -> CT_Tbl;
}
impl<'a> Paragraph<'a> {
    pub fn detached(inner: &'a mut CT_P) -> Self;
}
```

`set_owner_index` and `set_index_path_head` require caller proof that physical identity and the fingerprint are unchanged. `None` from a direct reader means the slot is absent or the body child has the wrong kind. `Err` is an invalid row or cell path inside a table of the right kind that is present. Do not turn a missing owner into fallback success. `direct_body_paragraph_facts` returns `None` when the paragraph is not plain or owns a section. `plain_removable_paragraph` is `false` when the slot is missing, not plain, or a local edit is blocked. `Document::detached_text_paragraph` and `Document::detached_table` are infallible builders used before placement. `Paragraph::detached` is `impl<'a> Paragraph<'a> { pub fn detached(inner: &'a mut CT_P) -> Self }`.

Binding-local, not a stable export:

```rust
pub(crate) fn plan_body_splice(&self, splice: BodySplice) -> PyResult<Option<BodySplicePlan>>;
pub(crate) fn prepare_body_splice(&mut self, plan: BodySplicePlan, operation: &'static str, publication: &WordPublication) -> PyResult<PreparedBodySplice>;
pub(crate) fn commit_body_splice(&mut self, prepared: PreparedBodySplice, publication: &WordPublication) -> PyResult<()>;
pub(crate) fn direct_body_paragraph_slot(&self, state: &RevisionState) -> Option<usize>;
pub(crate) fn direct_body_run_slot(&self, state: &RevisionState) -> Option<(usize, AcceptedRunPath)>;
pub(crate) fn direct_body_cell_slot(&self, state: &RevisionState) -> Option<(usize, usize, usize)>;
pub(crate) fn direct_body_table_slot(&self, state: &RevisionState) -> Option<usize>;
pub(crate) fn word_whole_body_work() -> (usize, usize);
pub(crate) fn note_word_whole_source_read();
#[cfg(debug_assertions)]
pub(crate) fn assert_direct_read(direct: &[u8], whole: impl FnOnce() -> PyResult<Vec<u8>>, owner: &str);
```

`assert_direct_read` compares direct bytes with a whole-source read. The second generic callback instantiations are the direct paragraph callback and the direct run, table and cell callbacks from the actual patch. One closure is not by itself a second instantiation. The helper is not a public generic API. The public native hidden APIs earn the published native API rider.

`plan_body_splice` returns `None` to keep the rebuild when the inventory is not canonical, a slot is out of range, or the removed span is not a plain removable paragraph. `prepare_body_splice` returns an error for a publication other than `AppendBody` or `Conservative`, and for an owner allocation failure.

Selected fast path. `PreparedRevision` originates from the same exclusive `PyDocument` borrow. Origin, current serial, next generation, document state, and every retired token are unchanged between prepare, the native qualified simple edit, and commit. No callback and no other registry operation runs in that window. `commit_scopes` rejects only `ForeignOwner`, `StalePreparation` and `Retired`, and it does so before any retirement, at `oxml-py-support/src/lib.rs` lines 484 through 506. Those rejects are impossible under the proved preconditions. Every recoverable input check, allocation and Python result is finished before the native edit. There is no hypothetical owner-only rollback. If any qualified-path precondition cannot be proved, use the old staging candidate before any mutation. Native failure checks and the global dirty model, revision, owners and read-fact state match the old path. Tests cover invalid input before the edit and the proof invariants. There is no blanket handle-preservation promise. Conservative edits retire all scopes with the old semantics. Append carries only eligible owners.

Slot helpers return `None` when the live owner is not a direct body paragraph, run, cell, or table. That `None` selects the fallback. It is not a success.

Debug `rdocx._rdocx._word_whole_body_work() -> tuple[int, int]` returns `(inventory_rebuilds, whole_source_reads)`. It does not return bytes. Guard work is excluded. A second private debug helper, `_word_splice_work() -> tuple[int, int]`, returns `(visited_owner_records, serialized_xml_bytes)`. Those counters sit in the real owner lookup, plan, prepare, commit and index loops and in the actual XML emit. They reset on read. They are not a constant zero at entry and they are not a stable export.

Non-mutating direct reads: owner work equals N and 4N, touched XML bytes stay constant, and whole-body reads stay 0. A simple mutation may retire global owner and index metadata in linear time. `visited_owner_records` is at most `16 * max(owner_count, total_scope_state_count) + 16 * inserted_owner_count + 16` for one qualified one-block splice. The bound is not quadratic. Unrelated whole-body serialization stays 0. Touched XML bytes are bounded by the inserted or read owner. Counter units are loop visits and emitted XML bytes. If the actual code cannot meet that bound, report it to Codex before approval. Do not invent a passing counter.

Standalone fragment bytes match exactly after stripping only a redundant unrelated canonical root `xmlns:w14`. A binding the root declares, or a binding the fragment uses, stays. There is no arbitrary prefix or subtree normalization. Namespace closure is asserted. Saved package bytes stay exact. The old saved-bytes error behaviour stays.

`commit_scopes` rejects only `ForeignOwner`, `StalePreparation` and `Retired` before retirement. Tests cover that table and the fast-path proof. A case that cannot prove every precondition uses the old candidate before mutation.

Python routes that must hit the fast path or the proved fallback: run, table and cell XML, body adds, inserts, ranges and tables, plus comments, related stories, and the nested fallback.

Performance, unexecuted. The original mean of 3 uses `Paragraph.xml`, `insert_paragraph(10, "x")` and `remove_content(10)` on 100 and 1000 paragraphs with six verbose formatted runs. Ratio `large <= 3 * max(small, 1 ms)`. A secondary median of 7 uses fresh identical documents and samples. Cold and warm are separate states. Do not mix the secondary median into the reported mean. The final successful release build is identified. No concurrent compilation runs during the measurement. Host and setup sit outside the timed section.

Held, fresh, native and reopened bytes and errors keep the old semantics whenever the fallback runs. Append may keep eligible direct-body owners. Conservative still retires all scopes.

## Consumer map

| Rule | Entry points and outputs | Shared provider and caller policy | Lifecycle stages compared |
|---|---|---|---|
| Direct read | `direct_body_*_xml` and facts | `None` falls back. Wrong run path is `Err` | Read |
| Splice | `plan`, `prepare`, `commit` | Append keeps eligible owners. Conservative retires all | Update, serialize, reopen |
| Fallback | Existing rebuild | Same bytes and errors as today | Update, reopen |
| Fragment XML | `standalone_body_item_xml` | Omit only redundant `w14`. Opaque content stays | Serialize |
| Allocation | `ForeignOwner`, `Retired`, `DuplicateScope` | Existing precedence | Update |
| Cost | Three original public calls | Counters are owner loops and serialized bytes | Timed public calls only |

## Behavior contracts and evidence

| Surface or placement | Exact behavior and boundary examples | Primary source identity and location | Local discriminator or pending evidence |
|---|---|---|---|
| Paragraph XML | Slot holds a paragraph. `None` if it holds a table | Donor `direct_body_paragraph_xml` | `direct_body_xml_matches_full_read` |
| Run miss | Paragraph exists, path does not | `Err`, no splice | `missing_run_path_refuses` |
| Append | Add one plain paragraph at the end | Eligible owners stay. New handle is fresh | `append_keeps_eligible_owners` |
| Conservative | Insert or remove in the middle | Every prior scope is retired | `conservative_retires_all_scopes` |
| Prevalidated publish | Exclusive borrow proof holds, or it does not | Proved path commits. An unproved path uses the old candidate before mutation. No owner-only rollback after mutation | `failed_splice_publishes_nothing` |
| w14 | Fragment with no `w14` use | Exact old fragment bytes except the named canonical redundant unrelated root `xmlns:w14`. Saved package bytes stay exact | `fragment_omits_unused_w14` |

## Dependency readiness

F-X260 is done. F-X266 is pending and unclaimed. The live-handle contract from F-X260 is the provider. This draft does not weaken it. Approval is not implementation.

## Acceptance criteria

| ID | Input or setup | Expected observation or refusal | Consumers | Local test | Final gate or pending evidence |
|---|---|---|---|---|---|
| A1 | Issue 407. 100 versus 1000 paragraphs, six formatted runs. Named calls `Paragraph.xml`, `insert_paragraph(10, "x")`, `remove_content(10)`. Cold and warm separate. Mean of 3. Secondary median of 7 on fresh identical documents | `large <= 3 * max(small, 1 ms)`. The median is not mixed into the mean. Setup outside the timer. No concurrent compilation | Those three Python calls | `body_edit_cost_ratio_on_release_build` | Pending. The contract is approved and this measurement is unrun |
| A2 | Run, table and cell XML, adds, inserts, ranges, tables, comments, related stories, nested fallback, and standalone fragment bytes | Fast path or fallback. Bytes match the old full read. Fragment bytes match the old bytes except the one named canonical redundant unrelated root `xmlns:w14` normalization. Saved package bytes stay exact | Python bindings | `direct_routes_match_full_body_bytes` | Local regression |
| A3 | Append versus conservative insert and remove | Only append keeps eligible owners. Conservative retires every scope | Handle liveness | `append_keeps_eligible_owners`, `conservative_retires_all_scopes` | No blanket survival claim |
| A4 | Invalid input before the edit, or a qualified-path precondition that cannot be proved | Old candidate runs before mutation. No fake owner rollback. No partial splice | Splice | `failed_splice_publishes_nothing` | Local regression |
| A5 | Standalone fragment without a `w14` use, plus opaque unknown child bytes | Exact old fragment bytes. The only allowed normalization is the named canonical redundant unrelated root `xmlns:w14`. Opaque bytes stay. Saved package bytes stay exact | Fragment writer | `fragment_omits_unused_w14` | Local regression |
| A6 | `ForeignOwner`, `Retired`, `DuplicateScope` | Existing precedence | `oxml-py-support` | `owner_scope_precedence_unchanged` | Local regression |
| A7 | Held, fresh, native and reopened package after a successful append and after a fallback | Old error and byte semantics | Python and native | `held_fresh_reopen_match_old_semantics` | Local regression |
| A8 | Debug counters on a non-empty owner loop | `_word_whole_body_work` is `(inventory_rebuilds, whole_source_reads)`. `_word_splice_work` is `(visited_owner_records, serialized_xml_bytes)`. Counts follow real loops. A zero-entry counter is not the proof. The linear visit bound is a local proof inside the chosen implementation budget | Both debug helpers | `body_work_counters_follow_owners` | Test-only. Not a stable export. Not a pre-approval block and not a future product test |

## Test matrix

| Case | Modes or state combined | Expected difference or agreement | Wrong implementation rejected |
|---|---|---|---|
| C1 | 100 and 1000, cold then warm | Ratio bound and separate median | A setup-inclusive timer fails |
| C2 | Direct paragraph slot versus a table in that slot | XML versus `None` | Scanning the whole body and still returning the table XML as a paragraph fails |
| C3 | Append one paragraph, then a conservative mid-body insert | First keeps eligible owners. Second retires all | A blanket handle survivor fails |
| C4 | Invalid input, or an unproved exclusive-borrow precondition | Old candidate before mutation. Bytes unchanged. No post-mutation owner rollback | A fake owner rollback fails |
| C5 | Comment and a related story on the nested fallback | Same bytes as the full path | A fast path that drops the comment fails |

Omitted: a new public Python method, and a promise that every handle survives.

## Rejected alternatives

- Changing a Python public signature.
- Exporting `word_whole_body_work` as a stable function.
- A new module for the splice planner.
- Treating a constant entry counter as the work bound.

## Test plan

Backlog test gate, verbatim: regression.

| Timing | Category | Test | Asserts |
|---|---|---|---|
| local scoped | regression | `direct_routes_match_full_body_bytes` | A2 |
| local scoped | regression | `append_keeps_eligible_owners` | A3 append |
| local scoped | regression | `conservative_retires_all_scopes` | A3 |
| local scoped | regression | `failed_splice_publishes_nothing` | A4 |
| local scoped | regression | `fragment_omits_unused_w14` | A5 |
| local scoped | regression | `owner_scope_precedence_unchanged` | A6 |
| local scoped | regression | `held_fresh_reopen_match_old_semantics` | A7 |
| local scoped | regression | `body_work_counters_follow_owners` | A8 |
| local scoped | regression | `missing_run_path_refuses` | Run `Err` |
| final sprint | performance | Identified release build. 100 and 1000. Mean of 3. Separate median. Cold and warm. No compilation | A1 threshold. Pending |
| final sprint | risk rider | `/verify --full`, hash, installed typing, stub, package dry-run, WASM | Union once |

Existing `crates/rdocx-py/tests/test_core.py` and the native integration entrypoint. Debug helper stays `cfg` test or debug, not a public export.

## HLD impact

- `docs/hld/03-architecture.md`, Owned publication and causal paths
- `docs/hld/04-opc-and-packaging.md`, Checked Word operation source ownership
- `docs/hld/10-bindings-spec.md`, Python Word document operations
- `docs/hld/12-testing-strategy.md`, Binding tests
- `docs/hld/14-development-backlog.md`, F-X266 status only at completion

## Risk routing

- Public API. No stable Python signature change. The hidden native helpers are published cross-crate providers and earn that rider. State both at review.
- PyO3. The timed consumer is the existing Python call, not a native-only bench.
- Performance. Workload, mean of 3 on the three named calls, secondary median of 7 kept separate, cold and warm, release build, and the ratio are pinned. Counters are a separate control.
- Cached facts and live handles. Append versus conservative is the publisher split. Held, fresh, native and reopen are required.
- Parser or serialiser. Fragment `w14` omission must not change saved package bytes.
- Shared consumers. Run, table, cell, comment, related and nested fallback.
- Contributed criteria. PR415 routes in A2. Donor timings are not adopted.
- Raw XML. Opaque subtree bytes stay.

## Hash harness

Expected unchanged. The fragment declaration omission must not change saved package bytes. If a test fixture package changes, that is a defect, not a baseline edit. No behaviour commit is pre-authorized here.

## Implementation checklist

- [x] Add the named helpers on existing document and binding types.
- [x] Keep `plan` returning `None` as the fallback.
- [x] Publish only after prepare and the native edit succeed.
- [x] Instrument real owner and byte counters.
- [ ] Leave the release ratio unrun until the final union.

## Open questions

| ID | Affected F-IDs and A-IDs | Decision and recommendation | Status and answer/source |
|---|---|---|---|
| none | F-X266 | The assembly request fixed the helper contract, liveness and the ratio. No user choice remains. Q16 is answered on F-290 only. | Answered 2026-10-23 by the assembly request. Q16 provenance is the 2026-10-23 user choice on F-290, "Author Word-compatible canonical slots (Recommended)". |

## Contract closure

Codex root semantic validation on 2026-10-23 approved this contract at source `a57be365`. Evidence is unexecuted. Implementation and the release ratio stay pending. This approval does not mark the story completed.

Direct readers return `None` for an absent or wrong-kind body child and `Err` for an invalid row or cell inside a present right-kind table. Python has no new stable signature. `_word_whole_body_work` counts rebuilds and whole-source reads. `_word_splice_work` counts visited owner records and serialized XML bytes.

Publishers are append and conservative splice, plus the fallback rebuild. The fast path mutates only after the proof above. Otherwise the old candidate runs before mutation. Append is the only mode that may keep eligible direct-body owners.

The timed calls are `Paragraph.xml`, `insert_paragraph(10, "x")` and `remove_content(10)`. Native counters do not replace that measurement. The mean of 3 and the median of 7 stay separate.

The closure generic has two actual callback instantiations from the patch: direct paragraph, and direct run, table and cell. The private splice types are justified by the direct-body fast path and the existing rebuild fallback. Both exist as soon as this story lands.

The linear visit bound is specified and is part of the chosen implementation budget. Local test `body_work_counters_follow_owners` must prove it during implementation. That proof was not a design block. If the landed loops miss the bound, report that during implementation. Do not invent a passing counter. The contract is approved and the proof stays pending.
