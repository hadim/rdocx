# Current Sprint, S91

**Milestone**: M24 and X, modern DOCX authoring and contribution repairs.

**Goal**: finish S90's bibliography catalogue and numbering-aware navigation
carryovers and resolve the new HTML, Python authoring, rendering, validation
and alternate text-box ownership reports before templates and forms at S92.
Issues264 and281 remain excluded. The expanded scope contains twenty-two active
stories. The original nine-story estimate of 23 to 29 developer-days no longer
covers the sprint. Estimate the twelve new large stories and the added medium accessibility story during detailed design
and keep rolling contribution intake open.

## Spec references

- `docs/hld/02-scope-and-non-goals.md`, DOCX-049 and DOCX-050, for the bibliography and navigation boundaries.
- `docs/hld/03-architecture.md`, inbound HTML, immutable field evaluation and physical story identity, for single-model ownership.
- `docs/hld/04-opc-and-packaging.md`, preserved XML and package relationships, for atomic text-box edits and saved source retention.
- `docs/hld/08-rendering-spec.md`, PDF backend, rasteriser, shape text, tables and Word bookmark field pagination, for fonts and shared metrics.
- `docs/hld/10-bindings-spec.md`, Python API shape and invalidation, for table, colour and render parity.
- `docs/hld/12-testing-strategy.md`, binding tests and checked Word scoped literal replacement, for installed public regressions and atomic failures.
- `docs/hld/14-development-backlog.md`, F-X192 through F-X212 and F-283, for exact acceptance and test gates.

## The wave

| F-ID | Title | Size | Status | Owner |
|------|-------|------|--------|-------|
| F-X192 | Complete the remaining Word bibliography catalogue | L | pending | - |
| F-283 | Complete numbering-aware navigation fields | L | pending | - |
| F-X193 | Preserve semantic mark highlights through nested HTML runs | S | in-progress | codex |
| F-X194 | Complete Python table text and indentation APIs | M | pending | - |
| F-X195 | Honor caller fonts and variable styles across render paths | L | pending | - |
| F-X196 | Validate visible paragraph content without false empty warnings | S | done | - |
| F-X197 | Unify Python colour inputs and font colour formats | L | pending | - |
| F-X198 | Preserve alternate text-box copies during scoped replacement | M | in-progress | codex |
| F-X199 | Model namespace-qualified empty table cell margins | S | in-progress | codex |
| F-X200 | Complete Python Word formatting and list parity | L | pending | - |
| F-X201 | Complete Python section stories and page authoring | L | pending | - |
| F-X202 | Complete Word tables pictures and internal links | L | pending | - |
| F-X203 | Complete Python Word document operations and validation | L | pending | - |
| F-X204 | Complete shared CLI one-shot editing commands | L | pending | - |
| F-X205 | Complete Python PowerPoint text fill and export parity | L | pending | - |
| F-X206 | Complete PowerPoint shape authoring and live handles | L | pending | - |
| F-X207 | Persist PowerPoint autofit and implement fit_text | L | pending | - |
| F-X208 | Author PowerPoint footer fields themes and transitions | L | pending | - |
| F-X209 | Edit PowerPoint masters layouts and themes | L | pending | - |
| F-X210 | Extend metric-compatible fonts and bullet fallbacks | L | pending | - |
| F-X211 | Complete Python script compatibility and safe XML access | L | pending | - |
| F-X212 | Align native accessibility with visible paragraph content | M | pending | - |

## Sequencing note

Complete actual prerequisites before starting any story. Source, Cargo,
shared tests, HLD edits and the hash baseline remain exclusive resources.
Design every unfinished story before implementation. Bibliography and
navigation retain their full deferred contracts and saved native evidence.
New contributions are assessed before duplicated implementation work begins.
The wave plan is recorded in S91-run.json after design and conflict analysis.

## Rolling contribution intake

Refresh open issues and PRs before design, implementation/resume checkpoints,
integration and final verification/close. Attach later contributor PRs to an
existing issue's F-ID, read updated immutable heads, reuse accepted changes,
and implement uncovered acceptance only. New independent scope receives a
new F-ID. Reconcile the plan, backlog, HLD14 and this record before proceeding.
Issues264 and281 stay excluded at every refresh. SPRINT_PLAN.md defines the
full rolling procedure. PR294 and Issues295 through301 are included at opening.
PR302 entered the rolling intake at immutable head
`30e4a93df7549990c5838da9c95866d19f1247b0`, adding independent F-X199.
It has no linked issue and is now ready for review, contributed by `pedroassumpcao`.
Before implementation its head advanced to
`7ac2851cb61b5e7b26427004517cc0cd4db0dbd9`. The native patch is unchanged.
Archive records will use actual integrated local measurements.

## Expanded everyday coverage intake

The 2026-10-09 refresh adds Issues303 through313 as F-X200 throughF-X210.
Issue314 is the umbrella acceptance checklist for these stories and the
existing fixes, not another implementation. No additional open PR was found.
PR294 and PR302 retain their previously assessed immutable heads.

Reuse existing issue work: Issue303 consumes F-X197 colours, Issue305 consumes
F-X194 indentation and F-X198 alternate-owner safety, Issue306 consumes
F-X196 validation, Issue308 consumes F-X194 cell text and F-X197 colours, and
Issue313 consumes F-X195 caller fonts and variable instances. Issue305
delegates Issue300 to F-X194 and Issue301 to the existing scoped owners.
Issue301 requires synchronized alternate updates in F-X202 beyond the earlier
F-X198 atomic refusal.
Keep the completed refusal safety and implement only that uncovered capability.

Detailed designs and resource waves for the added scope must be approved
before its implementation starts. Existing first-wave work continues under
its approved contracts. No new rendering baseline, asset, crate or module is
authorized merely by recording intake. Google import compatibility remains
best effort with explicit evidence limits. Charts, scripting, Excel, SmartArt,
OLE and signatures retain their existing scope boundaries.


PR [315](https://github.com/tensorbee/rdocx/pull/315) entered the next rolling
refresh at immutable head `af388f255d3b178aaa0aef03313034b815c71eb2`. It closes
Issues296 and297 and belongs to existing F-X195, with Issue313 crediting that
same implementation. No new story is created. Read the complete changed
files, comments and reviews before claiming font work. Reuse accepted loader,
render-path and instance changes, then implement uncovered approved criteria.
Its stated omissions include synthetic italic, bundled variable instances and
caller-font handout/PDF-A paths. Its fallback on subset failure, directory
precedence and synthesis geometry must be reconciled against the approved
plan. Contributor gate claims are not local evidence. Archive measurements
will be generated from actual integrated source.

Issue [316](https://github.com/tensorbee/rdocx/issues/316) entered the next
refresh. Existing stories retain the known font, link, section and selected
handle fixes. F-X211 owns the uncovered first-draft script corpus and CI
metric, validated writable XML, remaining stable-handle cases, actionable
errors and systematic silent-failure audit. Its evidence depends on the
provider stories, with no duplicate repair under this story. Detailed design
and resource routing precede implementation.

PR [317](https://github.com/tensorbee/rdocx/pull/317) entered the next refresh
at immutable head `3d056f46e97e9cc1caf2969b5096eeb6775a5fa0`, based on
main `733ce6f7b2b816c5760692abf5c99d68b29ae897`. Its42 changed files and
complete comments/reviews are captured for assessment. Issues295 and300 map
to F-X194, Issue298 CLI validation to F-X196, Issue299 to F-X197 and
Issue301 to F-X198. Its cell tcPr anchor maps to existing F-X206, distinct
from TextFrame bodyPr anchor. Successful alternate-copy synchronization is
reuse evidence for F-X202 beyond the completed safety refusal.

Its additional native accessibility-audit change proposes three
feature_showcase hash deltas. F-X212 owns that added behavior outside the current read-only
CLI contract, with a separate labelled reviewed delta before adoption. Do not replace frozen fixes, duplicate their F-IDs or copy its
baseline/archive measurements. Reconcile qualified visibility, atomicity,
handle generations and the shared colour extractor before reuse.

## Parallel implementation resources

The source phase of wave 1 can run F-X193, F-X196, F-X198 and F-X199 in separate
worktrees. F-X193 owns html.rs. F-X198 owns document.rs and
regression_test.rs. F-X199 owns rdocx-oxml/table.rs and integration_test.rs.
F-X196 owns commands.rs and the existing CLI test entrypoint.
F-X198 uses its approved regression entrypoint, not integration_test.rs.
Serialize shared HLD completion edits and delivery-record integration.
Queue Cargo execution against the shared target to avoid duplicate build
artifacts. F-X194 follows this wave because it uses both native Word test
entrypoints. Other conflicting stories retain their approved later waves.

## Definition of done for this sprint

- Full pinned Word bibliography catalogue and numbering-aware navigation pass their approved gates without guessed locale or cache fallbacks.
- PR294 semantic highlights survive nested runs and save/reopen without sibling or ordinary CSS background leakage.
- Issues295 and300 provide editable Python table text frames and signed, removable Word table indentation.
- Issues296 and297 provide consistent caller-font measurement and output, including actual variable bold/italic glyphs or verified synthetic fallback.
- Issue298 distinguishes visible nontext paragraph content from genuinely empty paragraphs.
- Issue299 provides consistent colour inputs/getters and nested font colour formats across both installed bindings.
- Issue301 synchronizes alternate text-box copies and counts once, or refuses atomically with retained bytes.
- PR302 models qualified, attribute-free empty cell margins while foreign or attributed lookalikes remain exact opaque XML. Namespace, schema-order and all 21 cell texts survive save/reopen.
- Issues303 through313 satisfy their full linked everyday coverage contracts, with shared acceptance credited once and Issue314 checked against the resulting APIs.
- Every feature has clean independent microscope review and scoped evidence. Final full verification and sprint review cover actual integrated HEAD and declared hash deltas.
- GitHub closure waits for verified sprint close. This run creates no release tags or publication.

PR [318](https://github.com/tensorbee/rdocx/pull/318) entered rolling intake at
immutable head b1aa842bf531a65c70a49d4ef282033d26e64020, based on733ce6f7.
It closes Issue308 and is attached to existing F-X205. It contributes text,
fill, table, metadata, media and export binding surfaces plus the existing
16:9 template repair. Its stated App/custom-property omission remains part
of F-X205 acceptance. Shared colours retain F-X197 ownership, cell text
retains F-X194 ownership and structural-handle improvements retain F-X211
ownership. Assess all24 changed files and the actual native template before
reuse. Contributor tests, archive values and artifact hashes are claims to
verify locally, not completed sprint evidence. No additional F-ID or asset
inventory is created by this contribution.

## Rolling contribution reconciliation, 2026-10-09

PR [319](https://github.com/tensorbee/rdocx/pull/319), immutable head
`9a36bc966d1e12e4e9e2f38b777889bd687c5dd9`, closes Issue313 and attaches
to existing F-X210. Caller fonts and variable instances remain F-X195 and
PR315. Its proposed four modified static Gelasio faces and new Noto Sans
Symbols2 subset differ from the explicitly approved eight-file inventory.
Capture and assess their sources, license terms, metrics, glyph mappings,
italic selection and diagnostic lifecycle before selecting any contribution.
The existing asset approval does not authorize that different inventory.
Its twelve claimed rendering deltas are contribution evidence, not accepted
local baselines. Root retains exclusive baseline ownership.

PR [320](https://github.com/tensorbee/rdocx/pull/320), immutable head
`252acccbba9adab9264cfb109c4b1d720ed79d4a`, closes Issue303 and attaches
to existing F-X200. Its colours remain F-X197. Assess its tri-state setters,
tab ownership and enums, defaults preservation, list helpers and installed
binding behavior against the full approved contract before reuse. Its body
explicitly leaves native shared numbering counters unchanged.

Issue [321](https://github.com/tensorbee/rdocx/issues/321) therefore maps to
that same F-X200, which already owns the native shared-definition counter
change and its authenticated Word evidence. It adds no duplicate F-ID or
second implementation. Its explicit two-instance continuation, first-use
startOverride and sub-level restart criteria are covered by the same native
state contract and require local regressions and explained sample changes.
F-283 retains its F-X200 completion prerequisite. Do not close Issue303 or
Issue321 based only on the contributor's narrower binding patch.

The refreshed intake contains seven open PRs and23 open issues. Issues264
and281 remain excluded. Contributor test, measurement and baseline claims
never substitute for local verification.

## Next rolling refresh, 2026-10-09

PR [322](https://github.com/tensorbee/rdocx/pull/322), immutable head
`6b5e2e352047357299d1f366f807f4661037430e`, closes Issue305 and attaches
to existing F-X202. Its table grid edits, pictures, links and handle policy
are contribution input for the full existing contract. Its explicit nested
table exclusions and main-document-only picture inventory do not satisfy
all qualified owners automatically. Indentation remains F-X194. Successful
alternate-copy text replacement remains F-X202 after F-X198 safety, reusing
the separately captured PR317 slice. Stable-handle residuals remain F-X211.
No new F-ID is created, and contributor gates are not local evidence.

PR318 advanced from the retained assessed b1aa842 head to
`bfdd6ab732e8c0003ea8c6110e3c891fb098d5b3`. The immutable comparison shows
one new commit changing only the PowerPoint chart artifact SHA in a Word
native test. Complete changed source, Git OID/bytes/SHA and refreshed reviews
are captured independently. Keep the prior assessment, with this addition
attached to F-X205's template acceptance. Measure the actual locally authored
artifact after adopting approved geometry. Do not copy the contributor's
expected hash or rewrite immutable cross-viewer acceptance evidence.

This refresh has eight open PRs and23 open issues, with all scope linked to
existing owners. Issues264 and281 remain excluded.

## Recently updated closed-record reconciliation

The rolling intake also inspects recently updated closed records. Ten PRs
and fourteen issues updated on2026-10-09 map to the existing completed
F-X177, F-X179 through F-X188, F-X190 and F-X191 deliveries. All fourteen
issues have authenticated completed close reasons. The ten unmerged closed
PRs have explicit integrated/superseded close comments. Their S89/S90 delivery
commits20888b7a and733ce6f7 are ancestors of this sprint. Later release comments
account for the recent updated timestamps. They create no new S91 F-ID and
are not reimplemented.

Exact record-to-acceptance ledger citations and close-comment URLs are in
.claude/scratch/S91-closed-delivery-reconciliation.json and its read-only
GitHub receipt. Their source hashes were independently checked. Historical
AS_BUILT integration-time notes are retained as checkpoint history, not
interpreted as current issue status. This classification concerns only those
24 authenticated closed records. Current open/new work continues through
its existing S91 acceptance owners, with264 and281 excluded.

## Rolling head refresh for PR318 and PR319

PR318 now points to b8e212f8db1841a5533646aa7190847724c73733 and stays with
F-X205/Issue308. Its additional edit is the current authored SmartArt fallback
artifact hash. PR319 now points to f0e6be942d4e958f4a07014466c57e7e03f6b8b7
and stays with F-X210/Issue313. Its additional edit is three deterministic
pixel expectations and attribution. Both complete incremental assessments
are recorded in those canonical plans, retaining unchanged prior findings.
No new story, repeated implementation or contributor baseline adoption is
introduced. The rolling inventory remains eight open PRs and23 open issues,
with264 and281 excluded.

The user separately approved the exact3916-byte Noto symbols asset,
SHA2567f5a31d932df09902c1097af020569b63d6296c067bc804ae6a597ef9b3ad21b.
Together with the earlier eight-file approval, this authorizes nine exact new
asset files. Actual font-story adoption remains behind its normal provider,
design, source and verification gates. The unchecked-box coverage gap remains.


## Rolling Issue323 and PR324 intake

The latest open inventory has nine PRs and24 issues, including excluded264
and281. Issue323 joins F-X206's existing group/connector acceptance: preserve
resolvable members when a connector is added inside a rotated, flipped and
scaled group, with an owner-specific diagnostic on unresolved members.
PR324 closes Issue311 and joins F-X208. PR322 advanced to
347091e6c9f2747c73de4b1b171c84203a9041ae and retains F-X202 ownership.
Complete new contribution and incremental source assessment is in progress.
No additional F-ID, duplicate implementation or completed acceptance is claimed.

F-X205 now retains actual native positive Notes and two-slide Hand-outs PDF
outputs under /private/tmp/S91-F-X205-native-notes-handout. Root authenticated
all45 artifacts. The two-page notes and one-page handout content streams
remain identical after native normal reopen/save. No handoutMaster exists
in the source or saved deck, establishing a default-print compatibility case
that the current native exporter must still resolve. These bounded native
controls are design evidence, not local renderer or sprint verification.

## Completed rolling contribution reconciliation

The22:20:37UTC refresh has11 open PRs and24 issues, of which22 issues
are in scope after264 and281 are excluded. All new and advanced heads are
completely captured. PR325/Issue306 remains F-X203 at87d4b71c, and
PR326/Issue309 plus Issue323 remain F-X206. PR324 remains F-X208 and
advanced PR322 remains F-X202. Updated assessments are in those canonical
plans. No duplicate F-ID or repeated feature implementation was created.

Independent checks authenticated202 evidence artifacts,124 Git blobs and
15 causal source references with no mismatches. PR325's advance changes
only workflow classification, preserving all26 cumulative feature paths.
Fresh comments/reviews/inline reviews for325 and326 contain no entries.
Previously captured heads and closed completed deliveries keep their existing
owners. Contributor test claims are not local completion or closure evidence.

## One causal provider and acyclic consumer ordering

F-X194 retains initial numeric cell-text generation support. Completed
F-X194/F-X197 release F-X206, which implements the bounded causal foundation
and existing Word/PPT consumer migration once. F-X201/F-X202/F-X205/F-X208
consume completed F-X206, and F-X209 already requires it. No F-X194-to-F-X206
reverse edge or provider-to-F-X211 edge is introduced. F-X211 audits residuals
and the final corpus after providers, with exactly100 allocated script IDs.
Its allocation digest is not a full statement-body freeze or executed rate.
Formal completion and exclusive file claims remain required before source work.
