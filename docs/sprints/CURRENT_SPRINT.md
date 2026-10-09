# Current Sprint, S91

**Milestone**: M24 and X, modern DOCX authoring and contribution repairs.

**Goal**: finish S90's bibliography catalogue and numbering-aware navigation
carryovers and resolve the new HTML, Python authoring, rendering, validation
and alternate text-box ownership reports before templates and forms at S92.
Issues264 and281 remain excluded. The nine included stories are estimated at
23 to 29 developer-days, with rolling contribution intake kept open.

## Spec references

- `docs/hld/02-scope-and-non-goals.md`, DOCX-049 and DOCX-050, for the bibliography and navigation boundaries.
- `docs/hld/03-architecture.md`, inbound HTML, immutable field evaluation and physical story identity, for single-model ownership.
- `docs/hld/04-opc-and-packaging.md`, preserved XML and package relationships, for atomic text-box edits and saved source retention.
- `docs/hld/08-rendering-spec.md`, PDF backend, rasteriser, shape text, tables and Word bookmark field pagination, for fonts and shared metrics.
- `docs/hld/10-bindings-spec.md`, Python API shape and invalidation, for table, colour and render parity.
- `docs/hld/12-testing-strategy.md`, binding tests and checked Word scoped literal replacement, for installed public regressions and atomic failures.
- `docs/hld/14-development-backlog.md`, F-X192 through F-X199 and F-283, for exact acceptance and test gates.

## The wave

| F-ID | Title | Size | Status | Owner |
|------|-------|------|--------|-------|
| F-X192 | Complete the remaining Word bibliography catalogue | L | pending | - |
| F-283 | Complete numbering-aware navigation fields | L | pending | - |
| F-X193 | Preserve semantic mark highlights through nested HTML runs | S | in-progress | codex |
| F-X194 | Complete Python table text and indentation APIs | M | pending | - |
| F-X195 | Honor caller fonts and variable styles across render paths | L | pending | - |
| F-X196 | Validate visible paragraph content without false empty warnings | S | pending | - |
| F-X197 | Unify Python colour inputs and font colour formats | L | pending | - |
| F-X198 | Preserve alternate text-box copies during scoped replacement | M | pending | - |
| F-X199 | Model namespace-qualified empty table cell margins | S | pending | - |

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
It has no linked issue and remains a draft contribution by `pedroassumpcao`.

## Parallel implementation resources

The source phase of wave 1 can run F-X193, F-X198 and F-X199 in separate
worktrees. Their exclusive files are respectively html.rs, document.rs and
regression_test.rs, and rdocx-oxml/table.rs and integration_test.rs.
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
- Every feature has clean independent microscope review and scoped evidence. Final full verification and sprint review cover actual integrated HEAD and declared hash deltas.
- GitHub closure waits for verified sprint close. This run creates no release tags or publication.
