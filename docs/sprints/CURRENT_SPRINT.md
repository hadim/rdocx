# Current Sprint, S91

**Milestone**: M24 and X, modern DOCX authoring and contribution repairs.

**Goal**: resolve the included HTML, Python authoring, rendering, validation
and alternate text-box ownership issues and PRs, retaining completed stories.
Issues264 and281 remain excluded and open. The user moved the separate S90
bibliography and navigation carryovers, F-X192 and F-283, to S92.
S91 now contains twenty-six active stories, eight completed, one integrated
and reviewed, four running, eight approved pending implementation and five
new stories awaiting design approval. Rolling contribution intake is reopened at the user's latest instruction.

## Spec references

- `docs/hld/02-scope-and-non-goals.md`, DOCX-049 and DOCX-050, for the bibliography and navigation boundaries.
- `docs/hld/03-architecture.md`, inbound HTML, immutable field evaluation and physical story identity, for single-model ownership.
- `docs/hld/04-opc-and-packaging.md`, preserved XML and package relationships, for atomic text-box edits and saved source retention.
- `docs/hld/08-rendering-spec.md`, PDF backend, rasteriser, shape text, tables and Word bookmark field pagination, for fonts and shared metrics.
- `docs/hld/10-bindings-spec.md`, Python API shape and invalidation, for table, colour and render parity.
- `docs/hld/12-testing-strategy.md`, binding tests and checked Word scoped literal replacement, for installed public regressions and atomic failures.
- `docs/hld/14-development-backlog.md`, F-X193 through F-X213, for exact acceptance and test gates.

## The wave

| F-ID | Title | Size | Status | Owner |
|------|-------|------|--------|-------|
| F-X193 | Preserve semantic mark highlights through nested HTML runs | S | done | - |
| F-X194 | Complete Python table text and indentation APIs | M | done | - |
| F-X195 | Honor caller fonts and variable styles across render paths | L | done | - |
| F-X196 | Validate visible paragraph content without false empty warnings | S | done | - |
| F-X197 | Unify Python colour inputs and font colour formats | L | done | - |
| F-X198 | Preserve alternate text-box copies during scoped replacement | M | done | - |
| F-X199 | Model namespace-qualified empty table cell margins | S | done | - |
| F-X200 | Complete Python Word formatting and list parity | L | in-progress | codex |
| F-X201 | Complete Python section stories and page authoring | L | pending | - |
| F-X202 | Complete Word tables pictures and internal links | L | pending | - |
| F-X203 | Complete Python Word document operations and validation | L | pending | - |
| F-X204 | Complete shared CLI one-shot editing commands | L | pending | - |
| F-X205 | Complete Python PowerPoint text fill and export parity | L | pending | - |
| F-X206 | Complete PowerPoint shape authoring and live handles | L | in-progress | codex |
| F-X207 | Persist PowerPoint autofit and implement fit_text | L | in-progress | codex |
| F-X208 | Author PowerPoint footer fields themes and transitions | L | pending | - |
| F-X209 | Edit PowerPoint masters layouts and themes | L | pending | - |
| F-X210 | Extend metric-compatible fonts and bullet fallbacks | L | in-progress | codex |
| F-X211 | Complete Python script compatibility and safe XML access | L | pending | - |
| F-X212 | Align native accessibility with visible paragraph content | M | done | - |
| F-X213 | Preserve TOC entry boundary paragraphs during rebuild | M | in-progress | codex |
| F-X214 | Bound recursive PowerPoint parsing and traversal | L | pending | - |
| F-X215 | Create charts in Python and replace existing chart data | L | pending | - |
| F-X216 | Import complete slides and merge decks with source formatting | L | pending | - |
| F-X217 | Import HTML and Markdown through public Word surfaces | L | pending | - |
| F-X218 | Export presentations as semantic HTML and Markdown | L | pending | - |

## Carried to S92, user scope decision

F-X192 retains its approved bibliography contract, unfinished worker branch
`work/f-x192-codex`, dirty worktree `/private/tmp/rdocx-f-x192-codex`, reviews
and required native evidence. Native year-omission controls have completed
style selection and F9/save, but ordinary reopen and acceptance remain pending.
F-283 retains its approved navigation contract and dependency on F-X200.
Neither story is completed or part of S91 close acceptance. S92 is planned,
not opened. The existing S92 template and form programme remains planned.
All twenty-two current PRs and twenty-eight included issues remain in S91.

## Sequencing note

Complete actual prerequisites before starting any story. Source, Cargo,
shared tests, HLD edits and the hash baseline remain exclusive resources.
Design every unfinished story before implementation. Bibliography and
navigation retain their full deferred contracts and saved native evidence.
Reuse assessed contributor slices before implementing uncovered acceptance.
The wave plan is recorded in S91-run.json after design and conflict analysis.

## Rolling contribution intake reopened

The user reopened issue and PR intake after parking bibliography and navigation
at S92. The current read-only GitHub inventory contains thirty open issues and
twenty-two open PRs. Excluding Issues264 and281 leaves twenty-eight included
issues. New Issues339 and340 have matching PR342 andPR341 respectively.
Assess each full immutable contribution once and bind it to an acceptance owner.
Issue339 andPR342 extend the existing all-story text contract in F-X203.
Issue340 andPR341 have an approved bounded TOC repair, F-X213, separate
from parked F-283. Full immutable contributions have been assessed. Neither
contributor patch is accepted as runtime evidence or adopted wholesale.
The prior twenty-six issues and nineteen PRs remain included. New arrivals and
advanced heads are reconciled at implementation, integration and final close
checkpoints. Final closure requires complete issue acceptance and explicit
integrated or superseded PR resolution. Keep Issues264 and281 open.
Read-only intake is retained at /private/tmp/S91-intake-reopened-20261011.
Earlier frozen-intake paragraphs are historical and superseded by this decision.
The 2026-10-11 18:43 UTC refresh found no new records or changed PR heads.
All current records retain the acceptance owners in SPRINT_PLAN.md.

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

- F-X192 and F-283 are carried to S92 with their full contracts and retained work. They do not gate S91 close.
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

## Latest rolling PR intake

The23:18:12UTC refresh contains14 open PRs and24 issues. All14 PRs and
22 issues are included, with264 and281 still excluded. New PR327 attaches
to Issue312/F-X209, PR328 to Issue304/F-X201 and PR329 to Issue310/F-X207.
No new F-ID or duplicate implementation is created. Advanced heads318,319
and324 retain F-X205, F-X210 and F-X208 respectively. Immutable source and
discussion assessment is complete for327/329 and pending for328 and the
three advanced heads at this checkpoint. Intake is not acceptance evidence.

PR327 at3231c0c32ec42be781192fef07cbff52dee4394f has fresh PR324 head
232b5dbb63a29cfe72462a1716d2b17d79a7d252 as its exact ancestor. Its seven
new commits and24-file feature delta belong to F-X209. The inherited F-X208
work is counted once. Preserve explicit master selection/layout mapping,
fresh imported owners, destination theme opaque data and full package usage
checks rather than adopting its replacement policy.

PR329 atd6cd56d6554119ea4cfc4b4897a56d660dbe44af remains source reference
for shared stacking and explicit native fit operations. Its implicit save and
read-only refresh, globally rounded stored sizes and unfit-floor writes do
not meet F-X207. Retain exact existing62500 stored rendering and authentic
92500/0 native controls. Contributor pass counts are not local oracle proof.

Independent root review verified all156 frozen F-X211 artifacts,100 syntax
checks, seven exact pinned upstream PPT blocks and three read-in-place input
hashes. No binding body ran and no compatibility rate exists. The movie
candidate has valid container facts but frame decoding failed. Its acceptance
and the twelve additional native obligations remain due. The full manifest
SHA is1889f6683eb788fc2a9007e84c66028cf5f1701a22a452de8767edbb0c8042be.

The advanced318/319/324 head assessments are now complete. All three whole
Git trees are identical to their previous captured heads, with91 retained
blobs reauthenticated and no duplicated source downloads. F-X207 design is
now approved after actual22-control dual-layout measurement:21 fitting
controls select the exact native pair, and the unfit exact-spacing control
retains explicit refusal. No production or general native parity is claimed.
F-X210's revised asset answer remains pending, and new source claims still
wait for every plan's material questions to be settled.

PR328 source assessment is complete and remains F-X201/Issue304. Root checked
its104 artifacts and75 blobs plus the91 reused advanced-head blobs without
mismatch. All14 current PR heads are assessed. Its read-side story creation,
VML-only watermarks and tiny-length refusals require adaptation to the approved
contract. No intake item is called completed on contributor test claims.

## Continued rolling intake

The23:30:53UTC snapshot contains15 open PRs and24 issues, with22 issues
in scope and264/281 still excluded. New PR330 at
274e3ca82b3db0393f0b3e0833b640ae2bbf9dc8 closes Issue307 and attaches
to F-X204. It is stacked on PR325, whose inherited F-X203 work is counted
once. PR318 advanced again to1eab7066b69203e36df3782a2d343ff1ae1defd2
and stays F-X205. Both fresh heads are under immutable source/discussion
assessment. Added author/draft metadata is capture enrichment, not changed
issue acceptance. No new implementation or duplicate F-ID is created.

The latest330/318 source assessments are complete. Root authenticated117
source blobs and145 artifacts without mismatch. PR330's ten unique CLI
commits attach to X204, with inherited325 work counted once under X203.
Its checkbox extension is shared X203 work and consumes qualified X195/X210
font semantics. The approved transaction, clock, generated image-name and
complete fit-owner contracts govern adaptation. Latest318 has the exact
same full Git tree and parent as its previous assessed head, so its source
work is not repeated. All15 heads from the23:30:53UTC snapshot are assessed.
Intake is not implementation or verification completion. Revised X210 asset
approval and the final integrated sprint gates remain outstanding.

Issue [331](https://github.com/tensorbee/rdocx/issues/331) entered the
2026-10-10T00:50:55Z refresh. F-X211 owns the shared native content-control
nesting admission for document open and raw XML replacement. Its separate
regressions cover normal-stack acceptance at256 and recoverable refusal
at257/5000, namespace ownership, all story-part paths and atomic replacement.
No duplicate CLI/Python fix or new F-ID is created. The frozen100-script
Base remains unchanged. This checkpoint has15 open PRs and23 included
issues, excluding264 and281.

The user approved the3968-byte F-X210 checkbox subset revision on
2026-10-10. Its exact selected bytes and SHA are recorded in the approved
F-X210 plan and shared F-X200 consumer contract. All18 unfinished active
plans are now approved. No material design question remains from this
asset intake. The implementation completion and integration barriers remain.

The2026-10-10T06:46:18Z intake has17 open PRs and25 included issues,
excluding264 and281. Issues332 and334 join F-X203's native export/creation
provider, with explicit independent behavior gates. PR333/Issue316 remains
F-X211 and its120 contributed specimens do not replace the frozen100.
PR335/Issue323 remains F-X206. Its group hunks duplicate assessed PR326,
so they are implemented once. Member recovery has explicit diagnostics and
a strict unresolved count that includes boundsless omissions. Complete
source assessments are retained in the existing plans. Intake is not
implementation acceptance. No additional F-ID or draft plan is created.

## Resolution coverage checkpoint, 2026-10-10 07:05:15 UTC

The refreshed inventory contains eighteen open PRs and twenty-seven open
issues. All twenty-five included issues and eighteen PRs are explicitly
mapped to S91 acceptance owners in SPRINT_PLAN.md. Issues264 and281 remain
open as requirements discussions and are excluded from resolution. Four
stories are completed, F-X194 is in progress and seventeen approved stories
remain. Final integrated verification and sprint review are still due.

New PR [336](https://github.com/tensorbee/rdocx/pull/336), head
`c6ab986008782e8a376a9cdd0bda2afd4847cf47`, closes Issues331 and321. Its parser
slice belongs to F-X211 and numbering slice to F-X200. Assess its complete
immutable source before adoption, including its added inline-wrapper and
cross-control limits. This creates no new F-ID or duplicate implementation.

## Parallel source reservation checkpoint

F-X212 starts alongside F-X194 using the existing document.rs inline native
audit tests and CLI integration carrier. Its production and test paths are
disjoint. F-X194 retains Cargo and package-carrier priority. F-X212 waits for
explicit Cargo handover and for the unchanged F-X194 hash gate before its
intentional baseline phase. F-X197 remains queued after F-X194 integration.

PR336's 07:27:15 UTC updated head is
`935ac165afcd29d34454568ba3b2287a4bac52dc`. Complete incremental source
assessment is now reconciled with existing F-X200 and F-X211 ownership. Keep
qualified256 controls and the selected per-level numbering contract. Proposed
64-control and512-element limits are not adopted. F-283 owns the added
cross-instance REF native discriminator.
The open inventory remains eighteen PRs and twenty-five included issues.

## Additional disjoint font worker reservation

F-X195 starts its existing font-manager/output and PDF/PNG/SVG core tests in
parallel with F-X194 and F-X212. CLI commands, Python/native facade carriers
and shared tests remain reserved to their current owners until handover.
Cargo, HLD/carrier writes and baseline changes remain serialized. F-X195's
font baseline phase follows the completed F-X194 unchanged gate and F-X212's
reviewed accessibility delta. No formal dependency is changed.

## Current implementation checkpoint

Four included stories are complete. F-X194, F-X212 and F-X195 are in
progress, with fifteen approved stories still to implement. F-X194's full
installed Python suites passed199 Word and86 PowerPoint tests. Corrected
native/lint/WASM/hash and package gates remain outstanding. F-X212 retains
real semantic Base failures and is completing the distinct typed field-cache
control. F-X195 has six frozen disjoint core-font test stubs, with actual
Base runs queued. Its thirty-eight immutable PR315 head and base blobs
are independently authenticated. No contributor gate is credited locally.
Final integrated full verification and sprint review have not run.

## Rolling Issue337 reconciliation

The 2026-10-10 08:47:21 UTC refresh includes new
[Issue337](https://github.com/tensorbee/rdocx/issues/337). Its nested text-box
complexity and stack safety belong to F-X211's existing native admission
contract. Freeze bare2000-level and mixed-owner subprocess controls, linear
retained/traversal cost and release boundary timing. The precise resource
boundary requires local measurement and independent review before adoption.
PR336 changed its body but retains head935ac165afcd29d34454568ba3b2287a4bac52dc.
The inventory is eighteen PRs and twenty-six included issues. Issues264 and
281 stay excluded. No duplicate feature or contributor acceptance is recorded.

F-X212's mechanical extraction now passes its scoped check, five exact tests
and all49 unchanged output hashes. Its intentional behavior phase remains
unverified. F-X195 has ten frozen controls and the additional Base runs are
queued. Final integrated verification and sprint review remain outstanding.

## Advanced contribution heads reconciled, 2026-10-10

The09:01:30UTC refresh retained18 open PRs and26 included issues. Issues264
and281 remain excluded. Root authenticated complete manifests and237 source
records and inspected exact deltas for PR318 at4fc65a5, PR328 at381f766 and
PR333 atb228d5d. They remain F-X205/Issue308, F-X201/Issue304 and
F-X211/Issue316 respectively. Existing plans now include spacing boundary and
atomicity controls, default story styles, and qualified XML context controls.
No duplicate story, contributor gate credit or completion is created. Full
immutable heads and evidence are recorded in those three canonical plans.

## PR338 and further PR333 head intake

The09:35:24UTC refresh contains19 open PRs and26 included issues. Issues264
and281 remain excluded. NewPR338 atc189dbd73b9a61543ed64a03d2305d0017bbf498
closes existing Issue337 and joins F-X211, including its stacked PR336 work
once. PR333 advanced to193819ca0a1dcf3415017c0be1186fe8df6bcb86 and retains
F-X211/Issue316. Root authenticated255 source records and inspected exact
incremental deltas. Complete source assessment is retained in the existing
F-X211 plan. Intake grants no contributor gate credit or arbitrary new nesting
policy. The approved256 control policy and complete textbox resource,
namespace, discovery and public compatibility acceptance remain mandatory.

## Historical S91 intake and verification checkpoint, 2026-10-10 12:40UTC

The12:40:16UTC GitHub refresh found no additional or changed records. The
subsequent all-state PR lookup still identifies PR338 as the newest PR.
All19 PRs and26 included issues remain mapped above, with Issues264 and281
excluded. PR338 and PR333 source assessment is complete in F-X211's existing
plan. Their acceptance implementation and verification remain pending.

F-X194 is completed and integrated with authenticated native, installed
Python, typing, WASM, verified22 package, unchanged49-entry hash and141 policy
gates. Two publication-dependent policy cases remain skipped.
F-X195 core and provider source reviews are clean, with runtime and consumer
wiring pending. F-X197 remediation is reviewed, with fresh installed-binding
and rendering gates pending. F-X192 owns reviewed regression stubs in its
separate worktree, with actual Base runtime and catalogue implementation pending.
F-X212 native, CLI and attributed49-entry hash gates pass. All22 package
verification commands passed, and the corrected archive capture authenticates
the fresh artifacts. The original failed capture wrapper remains recorded.
Its worker policy rerun passed all141 cases with two publication-dependent
cases skipped after the bounded roadmap-validator correction. The combined
sprint tree passes the focused native and CLI checks and all49 hash entries.
Independent integration review is clean. Combined archive measurements retain
F-X194 and F-X212 source exactly. Affected verified packaging and integrated
policy checks passed with verified3 affected packages and policy141 with two publication-dependent skips, all actual exits0.
F-X212 is formally completed with the reviewed combined source.

Six stories are completed and integrated, three are in progress and thirteen
remain approved. No design question is outstanding. Final integrated
verification, sprint review and close preflight remain outstanding. The sprint
is not ready for close.

## PR #294 reconciliation, 2026-10-10 13:26 UTC

The latest refresh retains 19 open PRs and 26 included issues, excluding
issues #264 and #281. PR #294 advanced from
63cd119dcb8fa9c367bf93141f79780db7f64886 to
81494509dc27cbcdd3c6d31067089175cdc2313f. Its additional commit changes
only README.md and the existing archive measurement script. Source and
discussion assessment is complete. HTML production remains byte-identical
to completed F-X193. Retain the stronger integrated tests and locally
verified archive measurements. No story is reopened or duplicated.
The subsequent description update adds validation notes without changing
source or scope.

F-X195 has six passing PDF controls and six passing Word controls after
independently reviewed corrections. The corrected PowerPoint test has passed
independent review. All five PowerPoint controls and all nine deterministic
font controls pass.
Consumer wiring and the remaining feature gates are outstanding.
Seven stories are completed and integrated. F-X192, F-X195, F-X200 and
F-X206 are in progress. Eleven stories have approved designs awaiting
implementation.
F-X197 completes its reviewed shared colour contract with scoped652 and
strict typing/stubs, explicit retained R14 Word354 and fresh R15 PowerPoint583,
native/WASM gates, verified22 packages and policy141 with2 expected
publication-dependent skips. Worker49 match its retained worker baseline.
The integrated prefix passed all seven commands with ROOT-authenticated
logs and source guards, including canonical F-X212 baseline49 confirmation.
All49 canonical accepted entries match. Final integrated sprint verification,
sprint review and close remain due. Scope remains frozen at22 stories,
19 PRs and26 included issues, with264/281 excluded.

## Current delivery checkpoint, 2026-10-11

Seven stories remain complete and integrated. F-X195, F-X200 and F-X206
are in progress. Ten approved stories await implementation. F-X192 and
F-283 are carried to S92 under the latest user scope decision. No
design approval is outstanding, and contribution intake remains frozen.

F-X195's current native Word library passed531 tests with8 documented ignores.
The PowerPoint library passed86 with4 documented ignores. The Word CLI child
passed73 with no failures or ignores. Its failed output-parsing wrapper is
retained separately from the independently qualified raw results. The next
PowerPoint CLI run passed34 and failed1 because its required pinned corpus
was absent from the worker. All50 existing corpus decks now match the pinned
manifest and can be read through the existing corpus environment option.
The failed run remains recorded. The corpus-admitted rerun passed all35
PowerPoint CLI tests with no ignores. The full Word regression suite then
passed2331 tests with7 documented ignores and no failures. Both Word
documentation tests passed, but their wrapper rejected rustdoc's compile-only
name suffix. The failed wrapper remains recorded separately. Strict source-bound
qualification now matches exactly the two compile-only source identities.
The PowerPoint documentation target completed with zero examples, receiving
no product or example coverage credit. The source-bound native composition
retains every failed wrapper separately. Reviewed scoped integration checks
finished with361 ordinary tests passed,3 failed and8 documented ignores.
Two failures invoke LibreOffice conversions, which abort in the sandbox.
The identical ODT conversion succeeds outside it with an isolated profile.
The public consumer reported an external-command timeout. Its nested Cargo
inherits the shared build target, but lock interaction remains unproved. The failed run and cleanup remain recorded. Reviewed focused
retry then passed the actual ODT child with1 passed and no failures or ignores.
Its strict wrapper rejected interleaved LibreOffice console output. Separate
source-bound qualification retains that actual ODT pass and the failed wrapper.
The next reviewed continuation passed both original remaining retries, public
authoring conformance and theme/font resolution, plus both pinned Word controls.
Its second pinned control exited0 with1 passed, but another interleaved
LibreOffice message failed the plain-line wrapper. All seven owned process
groups are reaped and absent. The failed receipt is retained. Source-bound
qualification now confirms that actual pinned pass while preserving its failed
wrapper. The next continuation rebuilt the current PowerPoint integration
binary and passed293 tests with26 documented ignores and no failures.
Both pinned Poppler versions and source guards passed. The remaining scoped
run stopped at a Clippy failure in font-trace byte accounting. Its unchanged
source and terminal process cleanup are recorded. The lint correction requires a fresh scoped Clippy pass. The reviewed SVG regression compiled and reproduced the default-instance
defect. Its reviewed production fix and equivalent Clippy byte-accounting
correction are now adopted. The exact regression rerun passed1 with no failures or ignores.
Second independent whole-feature review is clean. The first review found one defect:
searchable SVG can select weight400 for a retained variable-font default100.
The reviewed correction resolves that source finding. Fresh installed104,
eight-owner SVG and remaining scoped verification are required. Remaining
feature gates and completion remain due.

F-X192's four new Khmer, Lao, Nepali and Manipuri dense grammar tests have
been adopted after clean independent review. The thirteen-command Base queue
passed independent static review. Its pending successor retains all thirteen
commands and appends only the Manipuri refusal control. These fourteen
commands have not executed. Three additional full17 historical matrices
and their bounded comparison primitive are now adopted after clean independent
review, retaining51 original native expectations. The next queue preserves
the fourteen earlier controls and adds four tests, total18. None has run.
The eleven-way prospective formatter composition
also passed independent review without production adoption.
Three generated pagination controls now have actual pinned Word F9 and
byte-identical ordinary reopen results. The original source identities and
instructions survive. Each saved pagination marker disappears, while paragraph
properties change. The Sinhala control also changes127 scalar-aligned rich
properties. These bounded observations preserve the original historical
expectations and do not establish full catalogue or arbitrary-context parity.
A reviewed full17 Sinhala context control now has actual pinned Word F9
and byte-identical ordinary reopen evidence. All17 entries survive. K15
reproduces the same127 font-property changes as the compact control, so
restoring historical source and rich context did not explain the typography
difference. Historical expectations are unchanged. A second native control explicitly
sets the Bibliography complex-script font to Arial Unicode MS. That property
survives F9 and byte-identical ordinary reopen, but all17 entries retain the
same text and formatting as the unmodified control. Twenty-one revision-ID
metadata changes remain recorded separately. Independent evidence review is
clean. This does not establish physical font selection or catalogue acceptance.
Catalogue, rich-run, save/reopen and native authority acceptance remain due.
F-X200's full188-case installed queue and F-X206's nineteen-carrier composition
are reviewed preparations, not completed runtime acceptance. Independent
whole-feature review found that EPUB numbering omits table-style list items
in F-X200. The proposed fix passed independent review and its twelve-context regression
is adopted without production changes. Actual Base execution is queued.
F-X206 review records five
missing or defective surfaces. Four have existing unadopted fixes, while
selected placeholder picture replacement needs the remaining implementation.
Independent proposal review found that generated picture conversion drops
rectangle geometry in saved XML. Separate correction is underway.
Actual Base, native output and installed acceptance remain required.

Final integrated full verification, sprint review and close preflight have not
run. S91 is not ready for close.

## Current delivery checkpoint, 2026-10-11 reopened intake

S91 has21 active stories, seven complete, four in progress and ten approved
pending implementation. F-X213 is claimed to codex in its isolated worker
work/f-x213-codex at /private/tmp/rdocx-f-x213-codex. Its genuine test-only
Base controls both reproduced genuine semantic failures. The independently
reviewed R2 production repair is now adopted in three existing carriers, with
all other tracked inputs preserved. The first corrected runtime build found
two Rust type/binding errors, now corrected with explicit type and binding.
Nine contributor controls passed. The retained namespace control exposed a
two-to-one preservation failure. Its narrowly guarded retained-source fix is
applied and independently reviewed. Corrected runtime remains pending. Bibliography and full navigation remain carried to S92. Fresh F-X195 wheels, installed provenance, typing and
stub checks completed. All104 installed font controls and the native SVG
eight-owner control passed before equivalent test/lint normalization. Remaining
scoped gates, output
hash attribution, packaging and feature integration are still required.
These results do not mark F-X195 or S91 complete. Final integrated full
verification and clean sprint review precede close-sprint readiness.

The 2026-10-11 09:33:11 UTC read-only intake found no new or changed issues
or PRs. The SPRINT_PLAN resolution map exactly covers twenty-eight included
issues and twenty-one PRs. Issues264 and281 remain excluded. Corrected
F-X195 scoped Clippy found five test-only clone warnings. They now use borrowed
slices, and the corrected eleven-crate lint gate passed. F-X213 working review and its HLD
proposal review are clean. Production runtime and integration remain pending.

F-X195 current SVG module passed26 controls, no failures. Its one ignored
installed-artifact control has separate prior actual acceptance, retained with
the reviewed equivalent test normalization. Five current shaping-cache controls
passed. The no-default-font gate passed131 unit tests and3 documentation tests,
with no failures or ignores. Both Word and PowerPoint WASM facade checks
passed, including PowerPoint with rendering enabled. Remaining scoped risk
gates run serially before integration.

## Latest contribution and verification checkpoint, 2026-10-11

The 09:50:39 UTC read-only refresh found no new issues or PRs. PR342 advanced
to `9fd0b8da374b8c641e7ab89c0bd3d76bd2f4519a` and remains with F-X203.
Its immutable update has been assessed without duplicating the existing repair.
Reuse the qualified incomplete and deleted field handling under F-X203.
Simple-field break, namespace, source and installed-consumer controls remain
required. F-X211 retains complete cross-paragraph field ownership.
All twenty-eight included issues and twenty-one PRs have acceptance owners.
Issues264 and281 remain excluded and must remain open at sprint close.

F-X195 public documentation, rdocx-oxml Clippy and full tests, and
oxml-py-support full tests passed on the guarded current source. The exact
Rust1.93.0 check stopped before compilation because rustup could not write
its normal cache inside the sandbox. The authorized retry passed with the
exact Rust1.93.0 toolchain across all eleven selected crates, all targets and
all features. The first five-selector PowerPoint Base attempt stopped on fourteen
compiler errors before any regression ran. The worker was safely restored.
The independently reviewed minimal error conversions and test import are now
adopted, with all2714 source guards intact. Corrected Base execution remains
pending. F-X195 is running the49-entry deterministic hash check. Remaining
gates, feature integration and final sprint verification and review are still
required.

The 10:37:49 UTC refresh found no new or closed issues or PRs and no advanced
PR heads. All twenty-eight included issues and twenty-one PRs remain mapped
to existing acceptance owners. Intake remains open through final reconciliation.
Issues264 and281 are the two excluded discussion issues and must remain open.

## Current hash and publication checkpoint, 2026-10-11

The F-X19549-entry measurement completed with three differences against its
worker baseline. All49 actual entries exactly match the already accepted
canonical sprint baseline and the independently reviewed F-X212 accessibility
outputs. Independent hash-delta pass96 is clean. This is a stale worker baseline
input, with no new F-X195 output delta or baseline reason. The original exit1
remains recorded. Synchronization and a true check rerun remain pending.

The supply-chain check passed. README validation stopped before its public
consumer body because two script date constants lag their README rows. The
verified22-package publication dry run is live. Retain its archives before
reconciling actual package measurements, README rows and validation constants.
No feature completion or integration is claimed from these partial gates.

## PR343 intake checkpoint, 2026-10-11 10:56:28 UTC

New [PR343](https://github.com/tensorbee/rdocx/pull/343), head
`c6a78f683f327be8d7674af509874f74945080f3`, extends the existing related-story
picture requirements. Its complete immutable patch and exact base/head files
are assessed under /private/tmp/S91-PR343-intake. Native checked story-media
ownership belongs to F-X201. Atomic Run picture insertion and physical-picture
identity belong to F-X202. No new F-ID or duplicate implementation is added.
The donor has no Python implementation and does not prove visible drawings.
Keep installed owner/refusal, saved media/drawing, note and render acceptance.

The resolution map now covers all twenty-two open PRs and twenty-eight included
open issues. Issues264 and281 remain excluded discussion issues. Story totals
remain seven complete, four in progress and ten pending. PR343 source has not
been adopted and no contributor test is credited as local acceptance.

## F-X195 prerequisite completion checkpoint, 2026-10-11

F-X195 is completed through the formal dependency-prefix route. Its validated
worker source229bba9c and handoff tipbc38d1d9 are integrated locally as989950b8.
All317 relevant source records retain the exact tested bytes, sizes and modes.
The eight semantic carrier conflicts preserve completed table, colour and
visibility contracts. Independent integration pass99 reports zero defects and
zero smells. Canonical focused native6 and installed typing/provenance5 commands
passed. All49 output hashes match the unchanged accepted baseline. Nine fresh
integrated archive measurements pass, with every non-VCS member matching its
reviewed worker carrier and current canonical VCS metadata authenticated.
The original network-only measurement failure is retained separately.

S91 now has21 active stories, eight completed, three running and ten pending.
F-X195 scoped acceptance includes current full104 installed font controls,
caller19 across13 routes, mandatory eight-owner SVG controls and the declared
risk riders. Ignored historical controls retain their explicit qualifications.
Full integrated sprint verification and clean sprint review remain required.
The 12:22:15 UTC intake refresh has22 open PRs and28 included issues, with no
new records or changed heads. Issues264 and281 remain excluded and open.
Bibliography and the deferred authoring carryovers remain atS92.

## F-X210 parallel source reservation, 2026-10-11

The completed F-X195 prefix permits F-X210 to start in its isolated codex
worker. Its first source phase owns existing bundled_fonts.rs and the exact
nine approved font assets only. It creates deciding tests in the existing
inline test module before asset and registration production changes. Shared
font.rs, output, Word/PPT lowering and consumer carriers remain reserved to
the current owners until explicit source handover. Cargo, package and baseline
work remain serialized by the integrator. No formal dependency is changed.
After this claim S91 has eight completed stories, four in progress and nine
pending. Full sprint verification and clean sprint review remain due.

## S91 TOC repair integration checkpoint, 2026-10-11

F-X213 is scoped-verified and integrated at4b5eb59b. Issue340 andPR341
acceptance passes for entry-owned TOC rebuild, opaque properties, namespace
closure, repeated rebuild and save/reopen. Independent integration pass17
is clean. Native and installed controls, typing, bothWASM, unchanged49
hashes, policy, verified22 archives, docs and README consumers pass.
The worker handoff is consumed. Its branch and scoped evidence remain
available. Final integrated sprint verification, review and delivery ledgers
remain due, so this feature is reviewed rather than counted completed.
Eight stories are completed, one is integrated and reviewed, four are
running, and eight approved stories await their prerequisite checkpoints.
GitHub reconciliation follows the sprint-close workflow. Issues264 and281
remain excluded and open.

## Current verification checkpoint, 2026-10-11 18:43 UTC

Eight of21 active stories are completed. F-X213 is scoped-verified and
integrated, awaiting final sprint verification and ledgers. F-X200, F-X206,
F-X207 and F-X210 remain running. Eight dependent stories remain approved
and have not started. All28 included issues and22 PRs retain their acceptance
owners. The latest GitHub refresh found no new records or changed heads.

F-X200 has actual188 installed passes, scoped Clippy, format and WASM passes,
and four verified package dry-runs. Independent pass62 attributes exactly two
XML hash changes to eight canonical namespace declarations with equal expanded
XML trees. The expected changes are declared, with baseline handling still due.
Pass63 reviews the four actual archive records and active-clear HLD wording.
Current compressed remeasurement, scoped completion and integration remain due.

F-X206's namespace repair passes independent pass70 and173 drawing tests,
with two historical ignores. Broader native verification found two Word
library render-control failures. They remain failed and are under diagnosis.
F-X207 full working pass3 found affine anchor, font attribute and unused shared
cell inventory defects. Distinct remediation and focused verification are
running. F-X210 has fresh paired wheels built locally and both pinned Word
render controls passing. Public consumer compilation exceeded its unchanged
300-second limit. Exact feature-graph prebuild precedes the unchanged test
retry, with no conformance acceptance claimed.

Only completed prerequisites unlock the remaining stories. Full integrated
verification, clean sprint review and close preflight remain required.
Issues264 and281 remain excluded and open. S91 is not ready for close.

## Rolling intake, 2026-10-11 18:56UTC

The actual refresh found ten new Issues344 through353 and no new or changed
PR heads. Forty open issues include thirty-eight in scope after excluding
Issues264 and281. All twenty-two PRs remain included. The five new acceptance
owners F-X214 throughF-X218 are registered in the backlog and sprint plan.
Their designs and resource/dependency routing precede implementation. Existing
completed records and the S92 carryovers retain their status.

## Current verification checkpoint, 2026-10-11 18:59UTC

F-X210 has actual fresh paired-wheel installation, both installed diagnostic
companion controls and four strict typing/stub gates passing. Two fixture
setup failures remain preserved, with independently reviewed test-only
corrections and unchanged production wheels. The public conformance selector
now passes with its unchanged300-second limit after exact consumer build
warmup. Its native integration qualification is366 passes and8 historical
ignores. Genuine font paint, joint checklist and remaining scoped gates are due.

F-X206 caller-wrapper repair has clean independent pass72. The actual
DrawingML suite passes174 with2 historical ignores, the shape reread control
passes, and scoped check/Clippy pass. The formerly failing Word/PPT
reading-order/raster control now passes. Chart acceptance still stops at the
old package hash and requires exact intentional-name attribution before its
pixel assertions. No expected hash was changed.

F-X207 unused shared opaque-owner omission is resolved by independent pass5
with29 focused controls passing. Actual inherited Clippy failures remain
under a separate narrow remediation. Native paint and installed acceptance
remain due. F-X200 exact source and two-key hash proposals are prepared, with
its joint checklist gate and reviewed completion lifecycle still outstanding.
All four stories remain in progress. No final sprint gate is claimed.

## Further scoped checkpoint, 2026-10-11 19:03UTC

F-X206 exact chart control now passes the unchanged150DPI fixed750 by450
crop and zero-differing-pixel assertion. Independent attribution pass73
approved only the measured package expectation for two specified placeholder
names and explained lexical serialization. The actual reading-order/raster
control also remains passing. Remaining ten native package rows are admitted
under the existing private target, with all failed historical receipts retained.

F-X207 scoped lint remediation has clean independent pass6, all-target
checks, Clippy and29 meaningful controls passing. Whole-feature installed
and native paint acceptance remain due. F-X210 actual genuine-font paint
acceptance has a new failed symbol fallback result. The physically painted
Noto checkmark assertion failed and is under read-only diagnosis. It is not
a sixteen-case pass. Both installed companions and four typing gates remain
their separately authenticated passing results.
