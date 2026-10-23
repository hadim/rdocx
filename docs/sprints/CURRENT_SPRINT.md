# Current Sprint, S95.4

**Milestone**: M24 Modern DOCX authoring completeness.

**Goal**: compose legacy fields and modern forms, and author mail-merge data.
Also admit the incoming Word and PowerPoint correctness and performance repairs.
Provide public creation of legacy form fields, build modern forms on the existing
content-control model, and author mail-merge packages with offline diagnostics.
The four repair stories stay pending a full design. They do not replace the
original authoring contract.

**Estimated effort**: the 12 to 17 developer-day figure applies only to the
original three Rust and Python authoring stories, F-288, F-289 and F-290.
The earlier 8 to 11 figure is the pre-Python native basis for those same three
stories. The four repair designs add a provisional 15 to 24 developer-days.
The expanded planning total is 27 to 41 developer-days. Those figures are
estimates, not measurements, and they are not a capacity promise. The visible
over-two-week scope is an approved intake. The sprint clock has not started.
It starts with the first `/start-feature`. F-X267 remains the largest repair.

## Spec references

- `docs/hld/14-development-backlog.md`, sections F-288, F-289 and F-290 define
  supported authoring behavior, dependencies and each story's acceptance gate.
  Sections F-X264, F-X265, F-X266 and F-X267 define the admitted repair scope.
  The durable issue and PR mapping is the S95.4 issue and PR intake, 2026-10-23 section in `docs/sprints/SPRINT_PLAN.md`.
  Each repair still needs its own design before approval or claim.
- `docs/hld/03-architecture.md`, the existing legacy form, field, content-control,
  protection and mail-merge ownership contracts settle reuse of the document
  model, staging and publication boundaries.
- `docs/hld/04-opc-and-packaging.md`, The package defines normalized part identity,
  relationship ownership, content types and validation before serialization.
- `docs/hld/10-bindings-spec.md`, the existing native field and merge surfaces
  and Python content-control contracts establish compatibility. Each design
  must explicitly decide its new consumer coverage and exact signatures.
- `docs/hld/12-testing-strategy.md`, Test taxonomy, the legacy form and glossary
  controls, mail-merge controls and Typed content-control native Word
  qualification define local regressions, round-trip proof and authentic oracle
  provenance. Existing evidence qualifies only its recorded source and scope.

## The wave

| F-ID | Title | Size | Status | Owner |
|------|-------|------|--------|-------|
| F-288 | Legacy form field creation | M | pending | - |
| F-289 | Modern Word form authoring | L | pending | - |
| F-290 | Mail-merge package and data-source authoring | M | pending | - |
| F-X264 | Repair Word defaults package inspection and comment projection | L | pending | - |
| F-X265 | Preserve fragment relationship targets and duplicate styles | M | pending | - |
| F-X266 | Bound Word body reads and structural edit cost | L | pending | - |
| F-X267 | Bound PowerPoint mutation cost across public publishers | L | pending | - |

## Sequencing note

The table above is the sprint inventory. It is not the proposed shared-file integration order. Formal dependency order follows. F-288 depends on completed F-237 and
F-278. Complete and integrate F-288 through its approved scoped
dependency-prefix checkpoint before implementing F-289. F-289 also depends on
completed F-275 and F-285 through F-287. Design approval alone does not satisfy
the F-288 implementation dependency.

F-290 depends on completed F-233, F-270 and F-278. It may proceed independently
of the F-288 to F-289 chain only when actual source and resource leases are
distinct. Shared facade, binding or HLD files require serial ownership.

All seven stories start pending and unclaimed. The original three remain in
the design phase on their existing drafts. Public APIs, supported properties,
binding coverage, native comparison inputs and exact refusal behavior require
explicit design contracts before implementation. S95.3's native fixtures do
not automatically certify new form authoring.

F-X264, F-X265, F-X266 and F-X267 are provisional sizes. Draft plans exist and
stay unapproved. They have no formal dependency on each other. Each depends
only on the completed S95.3 providers named in HLD14: F-X260 and F-X261 for
F-X264, F-X261 for F-X265, F-X260 for F-X266, and F-X260 and F-X262 for F-X267.

Proposed serial integration, because leases overlap, is F-X264, F-X265,
F-X266, F-X267, then F-288, then its verified local provider prefix, then
F-289, then F-290. That order is not a set of invented formal dependencies.
Preserve the F-288 verified local provider prefix before F-289. Final Word
qualification of that prefix stays pending.

## Definition of done for this sprint

- F-288 creates text, checkbox and drop-down legacy fields with their supported
  properties. Public-created fields pass an authentic pinned Word differential
  and retain typed values through update and reopen.
- F-289 composes controls, protection, editable ranges, validation, placeholders
  and repeating data without a second document model. A dense public-created
  form passes the pinned Word behavior and rendering comparison across fill,
  reset, protect and reopen.
- F-290 authors settings, mappings, recipients, connection metadata and safe
  embedded data. Save and reopen retain the declared source and mappings.
  Unavailable external data produces explicit diagnostics, with retrieval
  policy-controlled and offline by default.
- Every F-ID satisfies its approved acceptance matrix, scoped verification,
  zero-defect and zero-smell microscope, and named HLD updates. The F-288
  prefix checkpoint retains all named final rider debt until actual acceptance.
- The integrated tree passes `/verify --full`, applicable risk riders and the
  independent sprint review. Unexplained hash changes block acceptance, and
  rendering baselines use deterministic font mode.
- Missing external evidence stays pending. Sprint closure does not imply
  completion of M24's later stability gate or authorize another publication.
- F-X264, F-X265, F-X266 and F-X267 meet the acceptance scope in HLD14 and in
  the S95.4 intake record. Donor patches stay candidates until a design
  decides each contract. Contributor CI, debug assertions and fuzz results
  are candidate evidence. They are not local qualified gates.

## Design checkpoint, 2026-10-23

Drafts: `.claude/plans/F-288-design.md`, `.claude/plans/F-289-design.md`, and `.claude/plans/F-290-design.md`.
Baseline `a57be365`. Intake is 4 open issues and 0 open PRs. That 4 and 0 figure is a historical snapshot of this design checkpoint, preserved as history, and it is not the later 19:37:06Z receipt. No acceptance work was added. Issue #381 stays a later AcroForm plan.
Mechanical answer recording is by Grok. Design validation is by Codex. The drafts are not approved.
Question status: Q1, Q2, Q3, Q4, Q5, Q6, Q7, and Q8 are answered. No user question remains unanswered. Q2 stays the 2026-10-23 source-analysis answer, with its historical provenance retained. Q1, Q3, Q4, Q5, Q6, Q7, and Q8 are user answers from 2026-10-23.
Q1 selects Rust and Python new APIs. WASM and CLI gain no new authoring methods. Existing consumers retain preservation and current-value behavior. The exact Python surfaces are now recorded in each plan under `## Python API contract`. Root Codex validation is still pending. This checkpoint does not approve the drafts.
Q3 stores settings and leaves execution to Word. No new native or Python calculation or formatting evaluator is added. Legacy `calculate_on_exit` defaults to false, matching the approved native options default.
Q4 starts F-289 only after a verified F-288 local provider checkpoint: local RED then GREEN for all five named focused native regressions plus the affected Python factory, attachment, current, and default checks, scoped verification, an independent clean microscope, and an integrated source ancestor. The semantic RED is default and current through the old existing providers. Additive factory absence is a compile failure, not that semantic RED. Final Word differential, full verification, and riders remain pending.
Q5 selects the proposed modern-form contract. Q6 selects the proposed render gate, and the native independent Word comparison remains unrun. Q7 selects an offline rdocx table plus Word metadata, versioned custom XML identified by document variable `rdocx.mailMerge.storeItemId`, caller-supplied external records, `Err` for offline plus supplied external records, and no pretended Word-native embedded source. Q8 selects payload caps of 16 MiB encoded XML, 100000 records, and 1024 columns.
Contract closure is still pending root Codex validation of the recorded Python contracts. That pending work is not an unanswered user choice.
The planning total recorded here is 12 to 17 developer-days for the original three stories, including Python. The prior 8 to 11 figure remains the native basis for those three. Both figures are estimates, not measurements. They are not an estimate of the later seven-story sprint.
Serial order stays F-288, then its verified local provider checkpoint, then F-289, then F-290, because the facade, tests, and HLD overlap. F-290 still needs distinct source and resource leases, and the Python expansion revises those leases.
All three features remain pending and unclaimed. Full gates and native qualification are unrun. These drafts do not claim implementation.
This checkpoint records design state only. It is not delivery or release credit.

## Resume checkpoint, user stop on 2026-10-23

All seven user choices are recorded under original Q1 through Q8 (Q2 was source-resolved). Plans F-288/F-289/F-290 remain draft. All features remain pending and unclaimed, run state stays phase design. No implementation, feature review, product verification, commit or push occurred. Codex contract closure still needs: F-288 generic cache publishers and current/default authority, including direct cached_result edits, typed/nested cache changes, attachment and fresh/reopened inventory. Reconcile legacy creation reachability with inventory/fill coverage for comments and text boxes while preserving existing ordinal compatibility. F-289 imported date/list/checkbox/placeholder/bound-value validation authority, and one physical control inventory across overlapping story owners without duplicate counts. Correct Python test paths to crates/rdocx-py/tests/test_core.py and use existing crates/rdocx-py/tests/typing_smoke.py. No further user choices are pending. Resume with contract reconciliation and validation before approving or claiming any feature. Grok handles mechanical work, Codex handles design/validation. The 12 to 17 developer-day total is a planning estimate for the original three stories, including Python.

## Resume checkpoint, intake only on 2026-10-23

The latest user request folds the 2026-10-23T19:37:06Z intake into S95.4 and leaves implementation paused. The durable mapping is the S95.4 issue and PR intake, 2026-10-23 section in `docs/sprints/SPRINT_PLAN.md`. Original plans F-288, F-289 and F-290 stay draft. All seven features stay pending. The original three-story estimate remains 12 to 17 developer-days. The expanded seven-story estimate is unknown. Original Q1 through Q8 and the three draft plans stay as recorded. Contract closure for those drafts is still outstanding. The four new repairs need designs, a contribution assessment and an estimate before any approval or claim. The original drafts do not need rewriting for this intake because the repairs have distinct owners. Provider reconciliation is still required before any claim, including the original stories. This checkpoint is not delivery credit in AS_BUILT or SPRINT_TRACKER, and it changes no release or version. The seven-story sprint remains not yet estimated, with a visible capacity risk on F-X267. There is no automatic ongoing intake loop.

## Resume checkpoint, run request on 2026-10-23

Q9 is answered on 2026-10-23. A bounded Rust API change from owned `Vec<u8>` part bytes to shared `Arc<Vec<u8>>` is allowed for OPC. Python stays unchanged. A release version change waits for a later release. Original Q1 through Q8 stay as recorded. Q10 is pending a user choice between consistent numeric multiples across default, paragraph and style, and a narrower rule. This checkpoint does not answer Q10.

The intake-only pause in the previous 2026-10-23 checkpoint is superseded by the request to execute `/run-sprint` once this intake preparation is complete. Implementation has not started. The contribution pin is `/private/tmp/s95.4-contributions/`, with the source assessment at `/private/tmp/s95.4-contributions/contracts.txt`. Live GitHub heads matched the captured SHAs before those patches were fetched.

## Resume checkpoint, design assembly on 2026-10-23

Q9 through Q14 are answered by the user on 2026-10-23. Q9 allows a bounded OPC public Rust change from owned `Vec<u8>` storage to `Arc<Vec<u8>>`, including removal and the signature break. Python stays unchanged. A release version waits. This is not permission for other public breaks. Q10 is consistent numeric spacing on the Python default, paragraph and style surfaces, with `Length` tested before `int`. Q11 keeps the old strict comment snapshot and adds `comment_anchor_snapshot_results`, plus a readonly `anchor_issue`. Q12 makes `part_names` and `part_xml` a save-equivalent view, with additive `zip_entries`. Q13 adopts the three PR416 behavior changes only as separate declared reviewed behavior commits. Q14 pins the 248-call gate, the 40 and 80 edit budgets, and keeps original strict xfail outcomes separate from `--runxfail`. The literal 212 count is not the runtime catalogue. The loop adds Core 15, app strings 6, app numbers 11 and app bools 4, which is 36, for a total of 248.

Drafts now include `.claude/plans/F-X264-design.md` through `.claude/plans/F-X267-design.md`, plus the reconciled F-288, F-289 and F-290 drafts. All seven stay `draft`. All seven features stay pending and unclaimed. Run state stays phase design. The initial 4 and 0 intake remains a historical snapshot. The latest scope is owned by these seven stories. The planning total is 27 to 41 developer-days, which is the original 12 to 17 plus repair estimates of 15 to 24. Estimates are not measurements. The visible over-two-week scope is approved intake. The clock has not started. Proposed serial order is F-X264, F-X265, F-X266, F-X267, F-288, the verified F-288 local prefix, F-289, then F-290. Formal dependencies are only the HLD14 edges. No native, oracle or full-suite execution happened. This checkpoint is design assembly pending root Codex semantic review. It is not approval, delivery credit, or a release action.

## Resume checkpoint, mechanical reconciliation on 2026-10-23

Q1 through Q15 are answered. Q15, answered 2026-10-23 after the previous assembly, adds `Field::legacy_form_default_value` and `Document::legacy_form_fields_with_defaults`. It does not add a field to `LegacyFormFieldData` or `LegacyFormFieldInfo`. F-290 does not inherit that default type. The latest user request is a full `/run-sprint` after intake. That request supersedes the earlier intake-only pause. Implementation has not started.

Grok applied mechanical contract corrections to the seven existing drafts and to the F-288 definition in HLD 14. All seven plans stay `draft`. All seven features stay pending and unclaimed. Run state stays phase design. The planning total stays 27 to 41 developer-days. Proposed serial order stays F-X264, F-X265, F-X266, F-X267, F-288, the verified F-288 local prefix, F-289, then F-290. This checkpoint does not approve a plan, claim a feature, or pass a gate. Root validation is still outstanding. The F-X266 linear visit bound is specified and not proved against current loops.

## Resume checkpoint, final contract closure on 2026-10-23

Q1 through Q15 stay answered. No new user choice was opened. Grok closed only the F-288, F-289, and F-X264 drafts. F-290, F-X265, F-X266, and F-X267 were not rewritten. All seven plans stay `draft`. All seven features stay pending and unclaimed. Run state stays phase design. The planning total stays 27 to 41 developer-days. Proposed serial order is unchanged.

F-288 now leases existing `crates/rdocx-layout/src/engine.rs` for the character-offset, segment, and note-cache consumers, in addition to the files already named. It adds `Field::effective_display`, `Field::effective_projected_text`, and `Field::effective_display_field_segments` with `std::borrow::Cow` returns. Borrowed `projected_text`, `cached_display_field_segments`, and `FieldRef::cached_display_segments` stay physical. `cached_run_display` stays physical. Literal replacement stays on the existing character map. The local test is `cross_consumer_render_fallback_before_empty_assignment` with deterministic fonts. The existing native Word gate stays the only Word gate.

F-289 no-op is candidate bytes, not typed-value equality. The same current value can repair display and bound state. Checkbox absence and a valueless checked child follow MS-DOCX `CT_OnOff` 2.6.3.17 and `CT_SdtCheckbox` 2.6.3.28. The private physical inventory and the Q15 paired default query stay.

F-X264 records the exact `n/240` domain, the `1.0021` truncation, the signed `i32` endpoints, and the next representable neighbours. A pure Python arithmetic probe is recorded as design evidence and is not a product gate. No story is approved, claimed, committed, or pushed.

## Resume checkpoint, preapproval polish on 2026-10-23

Q1 through Q16 are answered. Q16, answered 2026-10-23 after the previous closure, selects "Author Word-compatible canonical slots (Recommended)" for F-290 only. Primary source is MS-OI29500 2.1.369. Original Q1 through Q15 stay as recorded. Grok reconciled F-X265, F-X266, F-X267, and F-290, plus the F-290 definition in HLD 14. All seven plans stay `draft`. All seven features stay pending and unclaimed. Run state stays phase design. The planning total stays 27 to 41 developer-days. Proposed serial order is unchanged. This checkpoint does not approve a plan, claim a feature, commit, push, or pass a gate. Root Codex review is still outstanding.

## Resume checkpoint, preapproval draft fixes on 2026-10-23

Q1 through Q16 stay answered. No new user question was opened. Grok edited the existing F-288, F-X264, and F-290 drafts, plus the F-290 and F-X264 pending definitions in HLD 14. F-289, F-X265, F-X266, and F-X267 were not rewritten. All seven plans stay `draft`. All seven features stay pending and unclaimed. Run state stays phase design. The planning total stays 27 to 41 developer-days. Proposed serial order is unchanged.

The F-288 active provider checkpoint is every named local native group, including A7 and the render-fallback regression, plus the Python bindings. The historical Q4 answer still records that the original list was five groups. The gate is not weakened. F-X264 Python multiples use exact stored-grid rescue and, otherwise, a raw closed `i32` product bound before truncation. The `Comment` stub constructor is `__new__` and returns `Comment`. Anchored run edits publish comment text. F-290 `None` means no recognized owned configuration. Recipient parts stay at most one. Source edges use the exact `mailMergeSource` type and qualified XML ownership. This checkpoint does not approve a plan, claim a feature, commit, push, or pass a gate.

## Resume checkpoint, design approval on 2026-10-23

Codex root completed semantic design validation and approved the seven S95.4 contracts at source `a57be365`. Q1 through Q16 stay answered. No source implementation has started. The provisional planning total stays 27 to 41 developer-days. The implementation order is serial: F-X264, F-X265, F-X266, F-X267, F-288, the actual scoped F-288 local prefix, F-289, then F-290. The order follows shared OPC, facade, Python binding, test, layout, and HLD leases. It does not add a formal dependency beyond the HLD 14 edges and the F-288 provider prefix.

BACKLOG and the current feature table stay pending with owner `-`. Earlier stop and intake snapshots stay as history. This run request supersedes them. There is no `AS_BUILT.md` or `SPRINT_TRACKER.md` credit. No feature is claimed. No product verification has run. The F-289 contract is approved, and its implementation waits for the actual F-288 local prefix. The missing unreduced Issue 404 A2 input stays final acceptance debt and is not a design block. Every other unrun final gate stays pending. Approval does not mark any story completed.
