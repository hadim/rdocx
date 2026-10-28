# Current Sprint, S95.4

**Milestone**: M24 Modern DOCX authoring completeness.

**Goal**: compose legacy fields and modern forms, and author mail-merge data.
Also admit the incoming Word and PowerPoint correctness and performance repairs.
Provide public creation of legacy form fields, build modern forms on the existing
content-control model, and author mail-merge packages with offline diagnostics.
Eight designs are approved. F-X264 and F-X265 are reviewed and integrated,
and their final sprint gates are still pending. F-X266 and F-X267 have the
approved local provider prefix recorded on
`79047bf193bcd9dfb229a0b37a93d979dc0997a7`. Their final sprint gates are still
pending. F-288 has its
approved local provider prefix. Its final Word and sprint gates stay pending.
F-289 is integrated and reviewed locally, owner grok, and stays in progress.
The consumed local checkpoint is native 91 and Python 13. Whole-feature pass 2
is clean, with 0 defects and 0 smells, at
`.claude/reviews/F-289-whole-feature-pass-2.md`. Native pass 5, with 0 defects,
0 smells, 0 nitpicks, and 86 affected native tests, stays a bounded historical
outcome. It is not the current whole-feature result. Final Word 16.114 A7, a
fresh final CLI, the full workspace, full verify, the full hash suite, WASM,
publish dry-run, strict installed mypy, and stubtest stay pending. F-289 is
not done.
F-290 is integrated at `79047bf193bcd9dfb229a0b37a93d979dc0997a7` and stays in progress, owner grok. Root whole-feature pass 3 is clean, with 0 defects, 0 smells, and 0 nitpicks, at `.claude/reviews/F-290-whole-feature-pass-3.md`. Public local checks are consumed. Final gates stay pending. Shared leases are released. No final acceptance is claimed.
F-X268 is in progress, owner codex. The design is approved, including the
2026-10-25 refresh4 extension, and the story is claimed for parallel work. Codex root
approved the 2026-10-24 refresh2 extension. Codex root approved the
2026-10-24 refresh3 extension. Codex root approved the refresh4 mechanism 8
contract on 2026-10-25 after the user answered Q24. The baseline is
`d1f42516eb3747442fe4304a6331eab946dc9a0e`. Those approvals do not change
one another. Local implementation is integrated at `bc933ab450891281fc5093ed816465b70f01772c`. Independent whole-feature pass 2 is clean with zero defects, smells and nitpicks. Scoped local evidence is consumed. All eight stories are now integrated for the final sprint gate. Performance, memory, native Word, hash and full-union acceptance remain pending. No final gate pass is claimed. It follows F-289 and F-290. The approved F-X266 and F-X267 local provider
checkpoints are satisfied on `79047bf193bcd9dfb229a0b37a93d979dc0997a7`.
That completion is the local prefix only. It is not final acceptance and it
is not product implementation of F-X268. The
repairs do not replace the original authoring contract. The sprint now has
eight stories. The previously authorized full run remains authorized. This
approval does not ask for a new permission. Q1 through Q24 stay answered.
No product choice is unanswered.

On 2026-10-26 the user stopped new Grok assignments. The active Grok process
was interrupted and exited before Codex took sole write ownership. Codex
continues on `work/f-x268-codex` in the existing worker checkout. The original
`work/f-x268-grok` branch and all uncommitted work are preserved. Six F-X268
mechanisms are checkpointed locally. Mechanism 7 remains in progress and
mechanism 8 has not started. Final acceptance, integration of F-X268 and the
release remain pending.

**Estimated effort**: the 12 to 17 developer-day figure applies only to the
original three Rust and Python authoring stories, F-288, F-289 and F-290.
The earlier 8 to 11 figure is the pre-Python native basis for those same three
stories. The four repair designs add a provisional 15 to 24 developer-days.
The expanded planning total is 27 to 41 developer-days for those original
seven stories. That figure stays historical. The approved F-X268 contract
recorded a provisional 7 to 12 developer-days. That figure stays historical.
The 2026-10-24 refresh2 amendment revises the combined scope to a provisional
10 to 17 developer-days. Codex root approved that extension. The approved
refresh3 extension keeps that same provisional figure. That 10 to 17 figure
stays historical for the scope before PR422. The approved refresh4
extension records a provisional 12 to 21 developer-days. That figure is a
planning judgment, not a measurement, not a capacity promise, and not actual
work. It is not added to the historical 27 to 41. Actual work is unknown. These figures are estimates, not measurements, and they are not
a capacity promise. The visible over-two-week
scope is an approved intake. The sprint clock started with F-X264
`/start-feature`. F-289 is integrated locally and stays in progress. The
consumed local evidence is not a full gate. F-X267 remains the largest repair.

## Spec references

- `docs/hld/14-development-backlog.md`, sections F-288, F-289 and F-290 define
  supported authoring behavior, dependencies and each story's acceptance gate.
  Sections F-X264, F-X265, F-X266 and F-X267 define the admitted repair scope.
  Section F-X268 defines the claimed story. Its design is approved and the story is in progress, owner codex.
  The durable issue and PR mapping is the S95.4 issue and PR intake, 2026-10-23 section in `docs/sprints/SPRINT_PLAN.md`.
  The 2026-10-24 refresh is a later section in the same file.
  The 2026-10-24T13:49:35Z refresh is appended to that same intake.
  The 2026-10-24T17:02:28Z refresh is appended to that same intake.
  The 2026-10-24T23:05:39Z refresh is appended to that same intake.
  The scope approved through refresh3 stays approved. F-X268 is claimed and in progress, owner codex. Codex root approved the refresh4 extension on 2026-10-25. Q24 is answered.
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
| F-288 | Legacy form field creation | M | done | - |
| F-289 | Modern Word form authoring | L | in-progress | grok |
| F-290 | Mail-merge package and data-source authoring | M | in-progress | grok |
| F-X264 | Repair Word defaults package inspection and comment projection | L | in-progress | grok |
| F-X265 | Preserve fragment relationship targets and duplicate styles | M | in-progress | grok |
| F-X266 | Bound Word body reads and structural edit cost | L | done | - |
| F-X267 | Bound PowerPoint mutation cost across public publishers | L | done | - |
| F-X268 | Bound Word owner inventory and unchanged package staging | L | in-progress | codex |
| F-X270 | Prepare Word 0.16.4 and PowerPoint 0.14.4 patch releases | L | pending | - |

## Sequencing note

The table above is the sprint inventory. It is not the proposed shared-file integration order. Formal dependency order follows. F-288 depends on completed F-237 and
F-278. Complete and integrate F-288 through its approved scoped
dependency-prefix checkpoint before implementing F-289. F-289 also depends on
completed F-275 and F-285 through F-287. Design approval alone does not satisfy
the F-288 implementation dependency.

F-290 depends on completed F-233, F-270 and F-278. It may proceed independently
of the F-288 to F-289 chain only when actual source and resource leases are
distinct. Shared facade, binding or HLD files require serial ownership.

All seven stories started pending and unclaimed. The seven designs are now
approved. F-X264 and F-X265 are reviewed and integrated, with their final
sprint gates pending. F-X266 and F-X267 have the approved local provider
prefix recorded, and their final sprint gates stay pending. F-288 local prefix is recorded at
`96718dadf6840fe6ff73663dff3cd2b286418844`. F-289 is integrated and reviewed
locally, owner grok, and stays in progress. The consumed local checkpoint is
native 91 and Python 13. Whole-feature pass 2 is clean at
`.claude/reviews/F-289-whole-feature-pass-2.md`. Native pass 5 stays historical.
It is not the current whole-feature result. Pass 5 does not approve completion.
F-289 has not passed a full gate. Final Word 16.114 A7, a fresh final CLI, the
full workspace, full verify, the full hash suite, WASM, publish dry-run, strict
installed mypy, and stubtest stay pending. F-290
is integrated at `79047bf193bcd9dfb229a0b37a93d979dc0997a7` and stays in progress, owner grok. Root whole-feature pass 3 is clean, with 0 defects, 0 smells, and 0 nitpicks, at `.claude/reviews/F-290-whole-feature-pass-3.md`. Public local checks are consumed. Final gates stay pending. Shared leases are released. No final acceptance is claimed. The consumed
local gate
includes every named native group, A7 and the render fallback, the affected
Python checks, a clean microscope, and that integrated ancestor. Reviewed
status alone did not complete the edge. Final Word debt stays pending. Public
APIs, supported properties, binding coverage, native
comparison inputs and exact refusal behavior stay inside those approved
contracts. S95.3's native fixtures do not automatically certify new form
authoring.

F-X264, F-X265, F-X266 and F-X267 are provisional sizes. They have no formal
dependency on each other. Each depends only on the completed S95.3 providers
named in HLD14: F-X260 and F-X261 for F-X264, F-X261 for F-X265, F-X260 for
F-X266, and F-X260 and F-X262 for F-X267.

Proposed serial integration, because leases overlap, is F-X264, F-X265,
F-X266, F-X267, then F-288, then its verified local provider prefix, then
F-289, then F-290. F-X268 follows F-289 and F-290. The approved F-X266 and
F-X267 local provider checkpoints are satisfied on
`79047bf193bcd9dfb229a0b37a93d979dc0997a7`. Those two providers are done for
that local prefix. They are not final-accepted. F-X268 is claimed for
parallel work, owner grok. Implementation has not started.
That order is not a set of invented formal dependencies. Formal dependencies
for F-X268 are F-X266 and F-X267. Preserve the F-288 verified local provider
prefix before F-289. Final Word qualification of that prefix stays pending.

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

BACKLOG and the current feature table stay pending with owner `-`. Earlier stop and intake snapshots stay as history. This run request supersedes them. There is no `AS_BUILT.md` or `SPRINT_TRACKER.md` credit. No feature is claimed. No product verification has run. The F-289 contract is approved, and its implementation waits for the actual F-288 local prefix. At approval, the missing unreduced Issue 404 A2 input was recorded as final acceptance debt and was not a design block. The Q17 checkpoint below replaces that debt. Every other unrun final gate stays pending. Approval does not mark any story completed.

## Resume checkpoint, Issue 404 qualification on 2026-10-23

Q17 is answered by the user on 2026-10-23 for F-X265 A2 only. The recorded words are: "Original input is unavailable we will fix it now as best as we can and leave a note to the reporter to test against their original image and the can reopen if this is not fixed with the example image". The plan stays approved. No other story, including F-X264, changes state in this checkpoint.

A2 now requires the actual available unused-relationship reproducer. Exact inserted picture byte identity holds through policies, save and reopen, including the best available copied-target and body-replaced scenario. Local RED then GREEN on those available cases is mandatory. Donor image 9 and image 10 remain a reconstructed additional case and are not claimed equivalent. The exact original stays unverified. That follow-up is not a final acceptance blocker, and no original-input requirement blocks sprint closure. Every other unrun final gate stays pending.

The user authorizes a reporter comment only after an actual tested fix is available on a reviewed sprint ref, or in the proper close phase. The note invites a test of the original image and a reopen if it still fails. This checkpoint does not send the comment, close Issue 404, fetch, push, or record a fix. The exact note is drafted later with the real head and the real test details. No feature is completed by this amendment.

## Resume checkpoint, refresh intake on 2026-10-24

The user adds the 2026-10-24T11:33:08Z refresh to active S95.4. The durable mapping is the S95.4 issue and PR refresh, 2026-10-24 section in `docs/sprints/SPRINT_PLAN.md`. F-X268 is pending, owner `-`, and unclaimed. No F-X268 design file is written in this checkpoint. Answered Q18, Q19, Q20, and Q21 do not approve a design. No product choice is unanswered.

Existing status and owners stay. F-288 stays done, with its recorded local prefix. F-289 stays claimed and in progress. Its native increment has five named tests and a passing native check. Sixty-four existing regressions pass. One CLI path fails with NotFound and stays pending. Python, HLD, and review stay pending. This checkpoint does not mark F-289 complete and does not claim a full gate. F-290 stays pending and unclaimed. F-X264, F-X265, F-X266, and F-X267 stay in progress and reviewed. Their final gates stay pending. Run state is not edited. `AS_BUILT.md` and `SPRINT_TRACKER.md` gain no row.

The original seven-story estimate of 27 to 41 developer-days stays historical. The F-X268 estimate is unknown in this checkpoint. The previously authorized full run remains authorized after Root approves the F-X268 design. This checkpoint does not ask for a new permission, claim a feature, commit, or push. F-X268 is not product implementation.

## Resume checkpoint, refresh2 amendment on 2026-10-24

Receipt cutoff 2026-10-24T13:49:35Z. The same S95.4 intake in `docs/sprints/SPRINT_PLAN.md` records 11 open issues and 12 open pull requests. No issue changed. PR419 and PR420 are new. Nothing closed or merged. Heads 409 through 418 are unchanged. F-X268 stays pending, owner `-`, and unclaimed. Codex root approved the refresh2 extension on 2026-10-24 after reading the full revised contract and the four-file delta. The earlier approval paragraph stays historical. No implementation has started. No gate pass is claimed.

Q22 is "Apply the measured-memory policy (Recommended)". Q23 is "Use the added flat and linear gates (Recommended)". Both were answered by the user on 2026-10-24. The combined-scope estimate is a provisional 10 to 17 developer-days. The earlier 7 to 12 figure and the seven-story 27 to 41 figure stay historical. Eight stories stay. Formal dependencies stay F-X266 and F-X267. Serial order still places F-X268 after F-289 and F-290. This checkpoint does not claim a feature, commit, push, or mark a story done.

## Resume checkpoint, refresh3 approval on 2026-10-24

Receipt cutoff 2026-10-24T17:02:28Z. The same S95.4 intake in `docs/sprints/SPRINT_PLAN.md` records 11 open issues and 13 open pull requests. No issue changed. Nothing closed or merged. Heads 409 through 418 and 420 are unchanged. The PR419 head stays `9916bcdc01bc4acc065ae45afa8bd8ae07a2d30c`. PR421 is the only new pull request and stays an unmerged candidate. It is not accepted code. F-X268 stays pending, owner `-`, and unclaimed. Eight stories stay. The prior approved scope stays approved. Codex root approved the refresh3 extension on 2026-10-24 after reading the full corrected contract and the four-file delta. The canonical HEAD is `d6cefc239ab7de0d4dae63e13d730da558d3a3a5`, plus the working draft. The extension is the same F-X268 story and its seventh mechanism. It adds A10 and C7. The new helpers stay the captured signatures. The staging counter stays six units. The 73 flat calls, the 5 linear calls, and the 252-call catalogue stay the same. Q22 covers the added cache layers. No new question is opened and no timing promise is added. This record is that Root review. It is not a self-review. No implementation has started. No gate pass is claimed.

Q1 through Q23 stay answered. No new question is opened. Q22 covers the added cache layers. The combined-scope estimate stays a provisional 10 to 17 developer-days. Reassess it after decomposition. It is not a new promise and actual work is unknown. The earlier 7 to 12 figure and the seven-story 27 to 41 figure stay historical. Formal dependencies stay F-X266 and F-X267. Serial order still places F-X268 after F-289 and F-290. This checkpoint does not claim a feature, commit, push, reset run state, or mark a story done.

## Resume checkpoint, refresh4 approval on 2026-10-25

Receipt cutoff 2026-10-24T23:05:39Z. The same S95.4 intake in `docs/sprints/SPRINT_PLAN.md` records 11 open issues and 14 open pull requests. No issue changed. Nothing closed or merged. Heads 409 through 421 are unchanged. PR422 is the only new pull request and stays an unmerged candidate. It is not accepted code. F-X268 stays pending, owner `-`, and unclaimed. Eight stories stay. The prior approved scope stays approved. Codex root approved the refresh4 mechanism 8 contract on 2026-10-25. The baseline is `d1f42516eb3747442fe4304a6331eab946dc9a0e`. F-290 stays in progress under its existing approved implementation and review. This checkpoint does not edit run state, `AS_BUILT.md`, or `SPRINT_TRACKER.md`.

Q1 through Q23 stay answered. Q24, answered 2026-10-25 by the user, is `Require all 79 flat-cost gates (Recommended)`. The current flat set is 79, which is 76 catalogue keys plus 3 XML reads, under the same bound of 100 at most 1.5 times 25 plus 5 ms. The historical Q23 set of 73 stays historical. The five A9 linear calls stay, including the two that also sit in the 79. The 252 catalogue, the original 407 gate, and the PowerPoint 248-call run stay. A10 counted work is 14 calls and is independent of that timing gate. A11 and C8 are approved. The combined planning judgment is a provisional 12 to 21 developer-days. The 10 to 17 figure stays historical for the scope before PR422. It is not added to the seven-story 27 to 41 figure. Formal dependencies stay F-X266 and F-X267. Serial order still places F-X268 after F-289 and F-290. This checkpoint does not claim a feature, commit, push, or mark a story done.


## Release preparation intake, 2026-10-27

The user's existing instruction requests release preparation after the stories, with publication waiting for login and final approval. Root selects the recommended fresh patch candidates0.16.4/0.14.4 under the user's instruction to select recommended options without further questions. Actual24registry queries and remote-tag query find all candidates absent. The old0.16.3/0.14.3 tags and registry entries already exist from S95.3 and must not be reused. F-X270 is the ninth story for that requested work. Its completed formal provider is F-X263's release machinery. The eight product stories are integrated but their final criteria remain mandatory, including Cell.add_picture's open gate. Release preparation does not bypass that debt. No release tag, publication or fresh approval is granted here.

The eight-story execution descriptions above are historical checkpoints. F-X270 remains pending until its bounded contract is independently reviewed and approved. Root remains sole canonical writer. SizeL is the normal provisional release-preparation planning category, not a measured duration or a new finish-time promise. Actual developer effort remains unknown.
