# F-X267, Bound PowerPoint mutation cost across public publishers

**Status**: approved
**Sprint**: S95.4
**Size**: L
**Depends on**: F-X260, F-X262

## Problem

Public PowerPoint mutations still rebuild more of the package than the edited owner. Issue 403 asks for every mutating call in the external catalogue. PR413 `bdf22890de1762d025b89b83b549c56941ef7199` sha256 `03eeced92fef9ef9ecfde8ebd01761f04f6d21da2280ea1175fedce98ef56f28` leaves 107 package-bound calls. PR416 `768f8c109bd43fb9c1f6ecdfe2d913382bcc0368` sha256 `3497dd303faf1f7619c5a5ac096647bf2cadbbc892e8e056f0e111d938eb1592` is incremental on that head only. Incremental diff sha256 `690ca832aa7a26f69102c5f7fed282301ea0164ef52d452646a3bda8c670b80f`. Base is `a57be365`. Patches were not executed. Donor timings are not evidence.

Current storage is `parts: HashMap<String, Vec<u8>>` at `crates/oxml-opc/src/package.rs:75`. `set_part` takes `Vec<u8>` at line 350. `remove_part` returns `Option<Vec<u8>>` at line 358. `get_part` stays `Option<&[u8]>` at line 334.

## Spec reference

- `docs/hld/10-bindings-spec.md`, Packaging, Explicit PowerPoint text fitting, and PowerPoint footer, field, theme and transition authoring.
- `docs/hld/04-opc-and-packaging.md`, Checked PowerPoint XML transactions.
- `docs/hld/03-architecture.md`, PowerPoint checked XML ownership.
- `docs/hld/12-testing-strategy.md`, PowerPoint checked XML qualification, and PowerPoint footer and transition acceptance.
- `docs/hld/06-presentationml-model.md`, Public facade and Preservation strategy. New ownership prose is HLD impact under those existing sections.
- `docs/hld/14-development-backlog.md`, F-X267 story intent. HLD 15 is not in this list. No asset or version mechanism change.

## Approach

Planning estimate is 6 to 10 developer-days, provisional and not a measurement. No new file or trait. Python signatures stay. Q9 allows one bounded Rust break and no other public break. A release version waits.

### OPC storage (Q9)

Q9 allows the storage and removal break. The parameter is generic. Two instantiations are implemented with this feature. They do not have to already compile on the old `Vec<u8>`-only base. Do not wrap every `Vec` caller only to avoid the generic. Do not keep a concrete `Arc`-only parameter.

```rust
pub parts: HashMap<String, Arc<Vec<u8>>>,
pub fn set_part(&mut self, part_name: &str, data: impl Into<Arc<Vec<u8>>>),
pub fn remove_part(&mut self, part_name: &str) -> Option<Arc<Vec<u8>>>,
```

Instantiation one is the existing `Vec<u8>` caller: `package.set_part(name, mutate(xml).into_bytes())`. Instantiation two is `Arc::clone` of shared media at PR413 diff lines 3230, 3254 and 3268 in `rpptx` embed-media call sites. Both are structural and present in the implemented feature.

`get_part` stays `Option<&[u8]>`.

### Reread and compare

```rust
#[doc(hidden)]
pub fn reread(mut self, read: &OpcPackage) -> Result<Self>;
#[doc(hidden)]
pub fn same_package_as(&self, other: &OpcPackage) -> bool;
```

`reread` is the hidden public cross-crate provider. The standalone public helper documents this precondition: the baseline is still unmodified and read-equivalent. A package that was read and later mutated is not a baseline. The internal `Parsed.package` is the immutable baseline, and the candidate is edited from it. The trusted flag records that provenance. It does not detect an arbitrary public map edit. The known hand-built fallback is the untrusted flag, not a map scan. No snapshot uses the full ZIP fallback. Unsafe equivalence also uses that ZIP fallback, including normalization, a malformed producer, oversize input, and any other unproved fast-equivalence case. `same_package_as` compares packages that are a normalized read or a reread. It is not an unconditional comparison of an arbitrary hand-built package. Within the proved precondition, write then read matches, including errors, producer bytes, normalized names, backslash, duplicate case, relationship collision, content-type change, and malformed raw bytes.

Debug ZIP attribution, added after pass 1, is one hidden note and one crate-local helper. No new file, module or trait. `oxml-opc` does not depend on `rpptx`.

```rust
#[doc(hidden)]
#[cfg(debug_assertions)]
pub fn set_reread_zip_note(note: Option<fn()>);
```

Precondition: the note is installed by `rpptx` before `OpcPackage::reread`. It runs once immediately before the fallback `write_to`, including when that write returns an error. It does not run on the fast path and it does not run for any other `write_to` caller. `take_commit_work`, `saved_package_read_back` and `reopen` install `count_reread_zip_write`, which increments `zip_writes` through `note_commit_work`. `without_commit_work` pauses that increment, so the debug byte comparison does not count. `Presentation::note_embedded_zip_write` increments the same field once immediately before `commit_embedded_candidate`'s `write_to`, including when that write returns an error, and it does not count the following read. `to_bytes` and the parsed-`None` reopen and read-back paths keep their existing single increment and do not pass through the new notes.

```rust
#[doc(hidden)]
pub fn saves_as(&self, other: &Presentation) -> Result<bool>;
pub fn unchanged_slides_since(&self, earlier: &Presentation) -> Vec<Option<usize>>;
pub fn unchanged_layouts_and_masters_since(&self, earlier: &Presentation) -> bool;
```

`saves_as` is exact saved-byte equality, including pending direct edits, and it keeps the existing error semantics. `unchanged_slides_since` returns a conservative `Arc` allocation-origin map. `unchanged_layouts_and_masters_since` uses the same allocation and order. Both are doc-hidden. A newly allocated record with identical bytes may be `None`. These queries are not saved-byte equality. Do not describe every query as byte equality.

Debug only:

```rust
#[doc(hidden)]
#[cfg(debug_assertions)]
pub fn take_commit_work() -> [usize; 4];
```

Python `_inventory_work` and `_commit_work` stay debug test extensions. They are not a stable export.

### Behaviour commits (Q13)

Three separate declared reviewed behaviour commits, each with targeted evidence.

- Untouched slide and notes producer bytes stay preserved. Word fixture 49 stays unchanged. An intentional M21 digest delta for two handwritten notes is measured in its own reviewed behaviour commit. The oracle is this repository's golden, hash harness and regression. No PowerPoint application oracle is added for the digest. Genuine existing native gates stay.
- `fit_text` on a slide ignores an unrelated broken slide. Layout, master and notes still use every relevant context. Global `fit_report` and refresh stay unchanged.
- Transition and footer owner proofs cover only affected parts. Real structural ambiguity still refuses.

Unchanged owner incarnations stay preserved.

### Publishers and commits

Pointer identity is not enough. Source bytes, relationships, notes, comments, media, field ids and signature facts are fresh after a real edit. Tests go read bytes, then direct mutate, then package edit, then held, fresh, native and reopen, plus a fresh in-memory equivalent. A shared layout with two slide contexts is required. No-op detection compares output bytes, not pointers.

Distinct operations: `commit_candidate`, `commit_owned_parts`, and `commit_embedded_candidate`. Owner-proof preparation finishes every fallible Python result and revision before publish. Maps use allocation identity.

`take_commit_work` is `[usize; 4]`: parse, serialize, hash, ZIP. It is thread-local and resets on read. A separate owner-payload counter is `_inventory_work`. Do not treat the four-slot commit counter as that payload count. A local reopened target edit compares N with 4N. Payload parse, serialization, hash and changed-owner payload work stay equal. ZIP is 0 on the fast path. Global metadata, removal ordinals and reference remap are an explicit bounded linear exception. Signed and embedded fallback timings stay disclosed under Q14.

PR413's historical 107 package calls are contribution accounting only. They are not a product test and not an intermediate bench. The PR416 increment is consumed once. The final gate is the same 248 calls with every mechanism verified together. The removed test name `pr413_residual_stays_107_until_pr416` is not a gate.

Signed and embedded packages keep complete correctness and signature guarantees. Costly safety fallbacks are measured and disclosed. There is no constant-time promise.

### Performance gate (Q14)

External pin `3374348a48164f198ef077af46df485d0a1c469d`. Thresholds in the pinned `test_pptx_scaling.py` lines 26 to 30: N is 5, CALLS is 3, RATIO is 2, FLOOR_MS is 5, REOPEN is 16. The assertion shape is `large < 2 * small + 5` for decks of 5 and 20 slides. CALLS is 3 fresh actual targets, fastest sample. Reopen budget is 16. Large-deck early stop is 0.05 s.

Original growing shapes stay in memory: `shape_calls.py` sizes 10, 40 and 80, with and without media and notes, and `package_calls.py`. Raw outputs stay. Do not relabel them.

Added limit: each 40 and each 80 reported timed edit satisfies `unit_ms < 2 * 10_baseline_ms + 5`. Workload setup and aggregation match the originals. Shape timing is the mean of 10. Package timing is the mean of 5. Reported batch timing converts seconds to milliseconds and does not relabel a per-edit figure. The 40-slide large-image 600 case needs an additional 10-slide baseline with the same 600 payload. The original 40 case is still printed unchanged. No compiling during the release bench.

Whole-deck operations are the original 8. They are excluded from timing only by that meaning. The catalogue is the installed stub union of TIMED and WHOLE_DECK. The 89 names that are already constant stay constant. Original strict xfail and XPASS outcomes stay separate from `--runxfail`.

Static key count, without executing contributor code: 212 literal keys plus 36 dynamic loop keys is 248 unique keys. Core's 15 keys sit inside those 36. The installed stub completeness union keeps context variants. Do not collapse them for timings. The literal 212 figure is not a missing-key defect.

## Consumer map

| Rule | Entry points and outputs | Shared provider and caller policy | Lifecycle stages compared |
|---|---|---|---|
| Shared bytes | `set_part(&mut self, part_name: &str, data: impl Into<Arc<Vec<u8>>>)`, `remove_part`, `get_part` | Two instantiations in this feature: `into_bytes` and `Arc::clone`. Python unchanged. Consumers include `oxml-chart`, `oxml-sml`, signatures, Word comments and Flat OPC redaction, both native families, CLI and WASM. Source compatibility and output preservation. No per-call fake test | Read, serialize |
| Trusted reread | `OpcPackage::reread`, `same_package_as` | Baseline stays unmodified and read-equivalent. `Parsed.package` is immutable. The candidate is edited from it. No snapshot and unsafe equivalence use the ZIP fallback | Write, read |
| Slide delta | `unchanged_slides_since`, layout and master query, `saves_as` | Allocation provenance for the unchanged queries. `saves_as` is saved-byte equality, including pending edits | Commit, reopen |
| Fit text | Slide `fit_text` versus global `fit_report` | Unrelated broken slide is ignored only by the slide target | Update |
| Footer and transition | Affected-part owner proof | Structural ambiguity still refuses | Update, serialize |
| Cost | External 248 calls plus 10, 40, 80 shapes | Means and the 40 and 80 budget | Timed public calls |

Publisher map that must be rechecked before a fast path: `slide_mut`, `shapes_mut_at`, `core_properties_mut`, notes and handout `HeaderFooter_mut`, and `TextParagraphMut.default_run_properties_mut`. Each one changes facts that a `Parsed` `Arc` baseline must not keep. Direct mutation, package edit, held, fresh, native and reopen are the observations.

## Behavior contracts and evidence

| Surface or placement | Exact behavior and boundary examples | Primary source identity and location | Local discriminator or pending evidence |
|---|---|---|---|
| Arc storage | `remove_part` returns the same allocation a later `set_part` can share | `package.rs:75`, `350`, `358`. Q9 | `remove_part_returns_shared_bytes` |
| Reread | Write then reread matches a ZIP read, including a malformed raw part | PR416 incremental, unexecuted | `reread_matches_write_and_read` |
| Hand-built baseline | Parsed `None` and an untrusted hand-built package | One ZIP fallback. No equivalence promise | `hand_built_baseline_uses_fallback` |
| Untouched slide | Edit slide 2. Slide 1 producer bytes stay | Q13 commit | `untouched_slide_bytes_stay` |
| Fit text | Broken unrelated slide. Target slide still fits. Layout and notes still see their own context | Q13 commit | `fit_text_ignores_unrelated_broken_slide` |
| Footer | Edit one slide footer. Another slide's footer part is untouched. Ambiguous structure still refuses | Q13 commit | `footer_proof_is_part_local` |
| No-op | Set the same XML the part already has | Output bytes equal. Pointer equality is not the proof | `noop_compares_output_bytes` |
| 248 calls | 212 literal keys plus 36 dynamic keys. Core 15 sits inside the 36 | Pin lines, not a live run | `mutating_call_catalogue_is_248` pending execution |
| 40 and 80 | Same setup as the original scripts | `unit_ms < 2 * baseline_ms + 5` | Pending release bench |

## Dependency readiness

F-X260 and F-X262 are done. F-X267 is pending and unclaimed. F-X264's `zip_entries` contract is a neighbour, not a formal dependency. Implement this story only on a base that already contains any integrated OPC signature this plan assumes. Today the base still has `Vec<u8>`. This draft is not approval.

## Acceptance criteria

| ID | Input or setup | Expected observation or refusal | Consumers | Local test | Final gate or pending evidence |
|---|---|---|---|---|---|
| A1 | Issue 403 catalogue. Static 248 keys. 89 constant names stay constant. 8 whole-deck names excluded from timing by meaning | Catalogue matches the stub union. Strict xfail stays separate from `--runxfail` | External pin and installed stubs | `mutating_call_catalogue_is_248` | Execution pending |
| A2 | N 5 and 20, CALLS 3, reopen 16, early stop 0.05 s. Fresh actual targets, fastest sample | `large < 2 * small + 5` | Public mutating calls | `slide_ratio_matches_pinned_gate` | Pending release bench. No compilation |
| A3 | Original 10, 40, 80 in-memory shapes with and without media and notes. Original package script. Raw lines retained | Printed originals unchanged. 40 and 80 each satisfy `unit_ms < 2 * 10_baseline_ms + 5`. The 600-payload 40 case has a matching 10-slide baseline | `shape_calls.py` and `package_calls.py` workloads, reimplemented in existing tests | `shape_40_and_80_meet_added_budget` | Pending. Means stay 10 and 5 |
| A4 | Q13 untouched slide and notes bytes | Producer bytes preserved. M21 digest delta measured, not copied | Commit | `untouched_slide_bytes_stay` | Separate behaviour commit |
| A5 | Q13 `fit_text` | Unrelated broken slide ignored. A shared master owner with two consuming slide contexts that differ in fit uses `fit_text_at` to choose a size valid in both. A notes owner fits, then refuses when that notes-master theme relationship is removed. Global report unchanged | Fit publishers | `fit_text_ignores_unrelated_broken_slide`, `fit_text_at_selects_size_valid_in_distinct_shared_contexts` | Separate behaviour commit |
| A6 | Q13 transition and footer | Affected parts only. Structural ambiguity refuses. Unchanged owners stay | Those publishers | `footer_proof_is_part_local` | Separate behaviour commit |
| A7 | Direct mutate, package edit, held, fresh, native, reopen, and a second in-memory presentation built with `Presentation::new`. Shared layout, two slides | The common facts are subject, slide number start 5, the collection-marker text and the default-run bold payload. Notes and handout header edits stay on the reopened fixture. Whole-package equality is not required. No-op uses output bytes | The publisher list above | `parsed_baseline_tracks_real_edits` | Local regression |
| A8 | Signed and embedded complete packages | Correctness and signature checks hold. Fallback cost is disclosed. No constant-time claim | Signature readers | Existing `fx211b_native_opaque_root_edit_retains_invalidated_signature`. Embedded ZIP count is `embedded_replace_counts_commit_zip_write` | Local correctness. Timing is separate |
| A9 | `reread` and `same_package_as` on an unmodified read-equivalent baseline, plus parsed `None` and one untrusted hand-built package | Proved unmodified baseline matches. No snapshot, normalization, malformed producer, oversize, and the hand-built package use the ZIP fallback. One function covers parsed `None` and the hand-built package. A trusted baseline with one unnormalized ordinary part counts a ZIP fallback in `unnormalized_opaque_part_counts_reread_zip_fallback` | OPC and `saves_as` | `reread_matches_write_and_read`, `hand_built_baseline_uses_fallback` | Local regression |
| A10 | Historical PR413 residual of 107 | Contribution accounting only. Not a product test, not an intermediate bench, and not delivery credit. The final mechanisms are verified together on the 248 gate | Inventory note | None | Accounting only |
| A11 | Counters for parse, serialize, hash, ZIP and owners | Bounds follow N and 4N. Zero ZIP only where promised. Metadata work is classified | `take_commit_work` | `commit_counters_follow_real_work` | Debug only |

## Test matrix

| Case | Modes or state combined | Expected difference or agreement | Wrong implementation rejected |
|---|---|---|---|
| C1 | Edit one slide. Read the other slide's bytes | Untouched bytes match the producer | A full ZIP rewrite labelled constant fails |
| C2 | Shared layout, two slides, edit one placeholder | Both contexts stay coherent. The other slide's unrelated part stays | Pointer identity as a no-op check fails |
| C3 | Broken other slide during `fit_text`, and a broken layout used by the target | Target ignores only the unrelated slide. Its layout still participates | A global skip fails |
| C4 | Ambiguous footer structure | Refusal, no partial part write | A best-effort footer write fails |
| C5 | 10-slide 600-payload baseline beside the 40-slide 600 case | Same payload class | Comparing 600 against a tiny image fails |
| C6 | Strict xfail run versus `--runxfail` | Outcomes stay in separate reports | Marking the strict run green fails |

Omitted: a Python signature change, a release version bump, and equivalence for a hand-built baseline.

## Rejected alternatives

- A concrete `Arc<Vec<u8>>` parameter that rejects `impl Into`.
- Skipping the ZIP fallback when no authenticated baseline exists, or when fast equivalence is unproved.
- Copying the donor M21 digest instead of measuring it.
- Folding the three behaviour changes into the performance commit.
- Treating the literal 212 as a missing-key defect.

## Test plan

Backlog test gate, verbatim: regression.

| Timing | Category | Test | Asserts |
|---|---|---|---|
| local scoped | regression | `remove_part_returns_shared_bytes` | A concrete Arc round-trip |
| local scoped | regression | `reread_matches_write_and_read` | A9 proved precondition |
| local scoped | regression | `hand_built_baseline_uses_fallback` | A9 parsed `None` and untrusted hand-built |
| local scoped | regression | `untouched_slide_bytes_stay` | A4 |
| local scoped | regression | `fit_text_ignores_unrelated_broken_slide` | A5 unrelated slide, notes owner, and broken notes-master theme |
| local scoped | regression | `fit_text_at_selects_size_valid_in_distinct_shared_contexts` | A5 shared master contexts with different fit results |
| local scoped | regression | `footer_proof_is_part_local` | A6 |
| local scoped | regression | `parsed_baseline_tracks_real_edits` | A7 |
| local scoped | regression | `fx211b_native_opaque_root_edit_retains_invalidated_signature` | A8. Existing test name. The forwarding alias was removed |
| local scoped | regression | `embedded_replace_counts_commit_zip_write` | A8 embedded ZIP write |
| local scoped | regression | `unnormalized_opaque_part_counts_reread_zip_fallback` | A9 trusted baseline with one unnormalized part |
| local scoped | regression | `commit_counters_follow_real_work` | A11 |
| final sprint | performance | Release build. N 5 and 20. Shapes 10, 40, 80. No compilation | A1, A2, A3 pending |
| final sprint | regression | M21 two-note digest measured against golden, harness and regression | A4 pending. No PowerPoint application oracle |
| final sprint | risk rider | `/verify --full`, hash, installed typing, stub, package dry-run, WASM | Union once |

Existing `crates/rpptx-py/tests/test_documented_examples.py` and the native integration entrypoint. No new test binary. Original `shape_calls.py` and `package_calls.py` stay external raw controls. Their workloads are reproduced by tests in the existing files.

## HLD impact

- `docs/hld/03-architecture.md`, PowerPoint checked XML ownership
- `docs/hld/04-opc-and-packaging.md`, Checked PowerPoint XML transactions
- `docs/hld/06-presentationml-model.md`, Public facade and Preservation strategy, plus new ownership prose under those sections
- `docs/hld/10-bindings-spec.md`, public API and bindings
- `docs/hld/12-testing-strategy.md`, tests
- `docs/hld/14-development-backlog.md`, F-X267 story intent
- HLD 15 is not an impact file. No asset or version mechanism changes.

## Risk routing

- Public API. The Arc storage and `remove_part` return are the only approved Rust break. Python unchanged. Release version is later. Dry-run is final-only.
- Parser or serialiser. `reread` and `write_to` share entry rules. Child order stays.
- PyO3 and WASM. No new Python signature. WASM check is final-only. Debug helpers are not exports.
- Performance. A2 and A3 pin the workload, means, conversion from seconds to milliseconds, and the ban on compilation.
- Cached facts. The publisher list is the fact-changing set. Held, fresh, native and reopen are required. Pointer identity is rejected.
- Compatibility and ordering. Part names, backslash, duplicate case and content types are inside A9.
- Shared consumers. Slide, layout, master, notes, footer, transition, core properties, and text defaults.
- Contributed criteria. The 248 gate and the three behaviour commits stay separate. The historical 107 count is accounting only.
- Digest evidence is the repository golden, the hash harness and regression. No new PowerPoint application oracle. Genuine existing native gates remain.
- Release scripting. This story does not bump a version. The row is a prohibition, not a task.

## Hash harness

The untouched-slide commit may change the M21 deck digest. That delta is measured in its own reviewed behaviour commit. It is not copied from the donor and not folded into a performance or file-move commit. Other fixtures stay unchanged unless a named behaviour commit states the delta. No baseline is recorded in this approval. Rendering baselines stay on deterministic fonts.

## Implementation checklist

- [x] Change `parts` and `remove_part` to `Arc<Vec<u8>>`. Change `set_part` to `impl Into<Arc<Vec<u8>>>` with both instantiations.
- [x] Add `reread`, `same_package_as`, `saves_as`, and the unchanged-since queries.
- [x] Split the three Q13 behaviour commits. Untouched producer bytes, slide fit scope, and part-local footer proofs are separate labelled commits. Independent M21 measurement stays pending.
- [x] Prove publishers against output bytes in the local tests named above. The pass 2 repair checks a checked package commit, a real collection edit, the common in-memory facts, and distinct shared fit contexts.
- [x] Leave the release bench and the digest measurement pending. The M21 constant is the original `411f028c4b51c6aac0a0e2ae11bc63592a6d0947ab1a5f0d15116d19bfe766bc`. The donor digest was copied and has been removed. Measurement is final only.
- [x] Keep Python signatures and the release version unchanged.

## Open questions

| ID | Affected F-IDs and A-IDs | Decision and recommendation | Status and answer/source |
|---|---|---|---|
| Q9 | F-X267 A7 and the OPC storage row | Bounded Rust Arc storage. | Answered 2026-10-23 by user answer. Allow owned `Vec<u8>` to `Arc<Vec<u8>>` for OPC public Rust storage, removal, and that signature break. Python unchanged. Release version later. Not permission for any other public break. Root contract the same day: `set_part(&mut self, part_name: &str, data: impl Into<Arc<Vec<u8>>>)`. The two instantiations are the `Vec` `into_bytes` caller and the `Arc::clone` media caller. They are implemented with this feature. |
| Q13 | F-X267 A4, A5, A6 | Three PR416 behaviour changes. | Answered 2026-10-23 by user answer. Separate declared reviewed behaviour commits. Untouched slide and notes producer bytes stay, with an intentional M21 digest delta measured and not copied. Slide `fit_text` ignores an unrelated broken slide. Layout, master and notes still use relevant context. Global `fit_report` and refresh stay. Transition and footer proofs cover affected parts only. Structural ambiguity still refuses. Unchanged owner incarnations stay. |
| Q14 | F-X267 A1, A2, A3 | 248-call gate and the 40 and 80 budget. | Answered 2026-10-23 by user answer. `large_ms < 2 * small_ms + 5`. N is 5 and 20. CALLS is 3 fresh actual targets, fastest sample. REOPEN is 16. Large early stop is 0.05 s. Original 10, 40 and 80 in-memory `shape_calls.py`, with and without media and notes, and `package_calls.py`, stay with raw outputs. Added limit: each reported 40 and 80 timed edit is `unit_ms < 2 * 10_baseline_ms + 5`. Setup and aggregation match the originals. Means stay 10 for shapes and 5 for package calls. Seconds convert to milliseconds without relabelling a per-edit figure. The 40-slide 600 image needs a 10-slide 600 baseline. The original 40 line stays. No compilation during the release bench. Signed and embedded correctness stays. No constant-time promise. The original 8 whole-deck operations are excluded from timing by meaning. The 89 constant names stay constant. Strict xfail and XPASS stay separate from `--runxfail`. 212 literal keys plus 36 dynamic keys is 248 unique. Core 15 is inside the 36. Context variants stay uncollapsed. |

No unanswered user choice remains. Q16 is answered on F-290 only. Its provenance is the 2026-10-23 user choice "Author Word-compatible canonical slots (Recommended)".

## Contract closure

Codex root semantic validation on 2026-10-23 approved this contract at source `a57be365`, including the shared `Arc<Vec<u8>>` buffer contract. Every timing and digest claim is unexecuted. Implementation and final gates stay pending. This approval does not mark the story completed.

Operations: `set_part` and `remove_part` prove storage. `reread` and `same_package_as` prove the package comparison. `saves_as` and the unchanged-since queries prove presentation comparisons. `fit_text`, footer and transition publishers prove A5 and A6. `take_commit_work` proves counters and is debug-only. No new Python method is added.

`set_part` takes `impl Into<Arc<Vec<u8>>>`. The two instantiations are part of this feature. A concrete Arc-only parameter is rejected.

Trusted reread requires a baseline that is still unmodified and read-equivalent. `Parsed.package` stays the immutable baseline. The candidate is edited from it. Parsed `None` and an untrusted hand-built package share `hand_built_baseline_uses_fallback`. No snapshot and unsafe equivalence keep the ZIP fallback. A7 lists the publishers whose edits must refresh source bytes, relationships, notes, comments, media, field ids and signatures.

A1's 248 count is 212 literal keys plus 36 dynamic keys, with Core 15 inside the 36. It is not a claim that the suite was executed.

HLD impact is 03, 04, 06, 10, 12 and 14. HLD 15 is absent. No version statement is added. The M21 digest is measured here. Word 49 is expected unchanged. A10 is accounting only.
