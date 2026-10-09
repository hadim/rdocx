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
| F-X196 | Validate visible paragraph content without false empty warnings | S | in-progress | codex |
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
