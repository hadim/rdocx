# F-X095, Integrate PRs 77 through 80 and restore deterministic CI

**Status**: approved
**Sprint**: S72
**Size**: L
**Depends on**: F-X071, F-253

## Problem

Contributor PRs 77 through 80 add closely related Word reader and serializer
corrections across `crates/rdocx/src/document.rs:7266`,
`crates/rdocx/src/paragraph.rs:29`, `crates/rdocx/src/field.rs`, and the
corresponding `rdocx-oxml` document, revision, table, and text models. They were
authored from the pre-S72 main branch, so a direct merge would overwrite newer
section, story, CLI, binding, and release work and would carry local-name and
raw-byte shortcuts past the current namespace and preservation contracts.

The newest hosted runs also fail before contributor behavior is exercised.
The Python `rdocx` failure is already repaired by F-X094c on the sprint branch.
The Presentation fidelity job at `.github/workflows/ci.yml:399` runs on macOS
and installs a moving Homebrew LibreOffice build, while its harness requires
exact LibreOffice 26.2.5.2. HLD 12 and HLD 15 already define the supported
official Linux archive and Ubuntu 24.04 environment.

## Spec reference

- `docs/hld/04-opc-and-packaging.md`, namespace-aware Word preservation,
  relationship attributes, repeated save and reopen, and schema child order.
- `docs/hld/10-bindings-spec.md`, native Word reader facts and pre-1.0 public
  API compatibility.
- `docs/hld/12-testing-strategy.md`, Word reader-fact regression gate, render
  fidelity gate, and hosted CI job matrix.
- `docs/hld/14-development-backlog.md`, "F-X095, Integrate PRs 77 through 80
  and restore deterministic CI".
- `docs/hld/15-build-and-toolchain.md`, one checksum-pinned LibreOffice
  installer and dedicated Presentation fidelity job.
- PRs 77 through 80 at source heads
  `aed8f14d826e43fee52b6da75c47fe2c5d37645a`,
  `10c3606b864d174ca1d1bcd90c26a156f7a69f51`,
  `8ede12b4102fb8bdc9421c22e059b7df010e2113`, and
  `9a9d3e8eeab2a5b8f2088930beae50ebce918f23`.

## Approach

Build a current-tree hardened equivalent in one worker because all four PRs
overlap the same Word parser, facade, and integration-test files. Retain Pedro
Assumpcao's authenticated GitHub contribution in the design, delivery record,
release notes, and final PR comments. Preserve contributor commits where they
apply cleanly, then keep maintainer reconciliation and hardening distinct.

Adopt PR 77's narrowed numbering evidence contract. Adopt PR 78's strict
document and body boundaries, first section-properties retention, modeled
self-closing paragraphs and tables, bounded nested tables, author-optional
revision projections, empty simple fields, and glossary body parsing. Resolve
elements and relationship attributes by expanded name, preserve owner namespace
bindings, enforce schema order, and fail closed on malformed or truncated XML.

Adopt PR 79's marker child-content facts and public complex-field effective
properties. Use parsed XML events for child-content detection and update every
current exhaustive consumer of `ParagraphItemRef`. Adopt PR 80's cell-margin
round trip, empty-cell and empty-table behavior, self-closing paragraph
serialization, and corrected dynamic TOC coordinates. Keep valid historical
`w:tblGridChange` modeled and preserve only structurally empty changes as raw
XML in their original slot.

Move `presentation-fidelity` to Ubuntu 24.04 and install LibreOffice through
`scripts/install_pinned_libreoffice.py`, with the existing checksum-pinned
Poppler installer and required Linux packages. Extend workflow mutation tests
so a runner change, package-manager LibreOffice, missing installer, skipped
step, or moved oracle setup fails the contract.

Do not add a crate, module, test binary, feature flag, trait, generic parameter,
or dynamic dispatch. Add tests only to existing modules and integration
binaries.

## Rejected alternatives

- Blindly merge the four PR branches. Their main-based diffs conflict with S72
  and do not satisfy current namespace and malformed-input hardening.
- Split the work into four concurrent stories. The same parser and facade files
  plus one hash baseline make the work inseparable without semantic conflicts.
- Accept the latest Homebrew LibreOffice. That changes the external oracle
  without review and contradicts the existing exact-version contract.
- Suppress the failed jobs. Both jobs are release gates and must execute.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| regression | `numbering_level_unmodeled_fact_contract_is_limited_to_extras` | Modeled metadata and raw overlays do not masquerade as unmodelled facts, while retained extras do. |
| round-trip | `contributor_document_body_reader_semantics_survive_reopen` | Strict document boundaries, first section properties, self-closing blocks, missing revision authors, empty fields, and bounded nested tables retain exact accepted semantics through repeated save and reopen. |
| regression | `paragraph_markers_report_whether_their_source_elements_contain_children` | Comment and bookmark markers expose parsed child-content facts and namespace decoys remain raw. |
| regression | `reader_resolves_complex_field_display_segment_properties` | Complex-field display segments expose their effective direct properties in source order. |
| round-trip | `table_cells_and_empty_paragraphs_preserve_serialized_coordinates` | Cell margins, empty cell content, historical grids, self-closing paragraphs, and dynamic TOC coordinates remain stable. |
| workflow | `presentation_fidelity_uses_exact_linux_oracle` | The hosted job runs on Ubuntu 24.04 and installs exact LibreOffice and Poppler before the fidelity harness. |
| binding | `test_render_errors_reacquire_and_map_cleanly` | The already integrated font-table-aware Python repair remains green. |

The **test gate** is regression: run every focused test above, the complete
`rdocx-oxml` and `rdocx` suites, the Python `rdocx` binding suite,
`python3 -m unittest scripts.test_sprint_workflow`, and the hash harness.

## HLD impact

- `docs/hld/04-opc-and-packaging.md`
- `docs/hld/10-bindings-spec.md`
- `docs/hld/12-testing-strategy.md`
- `docs/hld/14-development-backlog.md`
- `docs/hld/15-build-and-toolchain.md`

## Risk routing

- **Any parser or serialiser**. Read HLD 04 and HLD 06. Add alias,
  namespace-shadow, fixed-prefix, schema-order, raw-subtree, malformed-input,
  bounded-depth, and repeated save-reopen regressions.
- **Public API of a published crate**. Read HLD 10 and the structural rules.
  The native marker and field facts are additive pre-1.0 API changes. Run
  rustdoc with warnings denied, inspect the API diff, run publish dry-runs, and
  enforce archive size limits.
- **CI workflow or toolchain policy**. Read HLD 15. Mutation-test the exact
  runner, versioned installers, ordering, failure propagation, and absence of a
  package-manager LibreOffice substitute.
- **External oracle comparison**. Pin LibreOffice 26.2.5.2 and Poppler 26.01.0,
  record exact commands and versions, retain the gate artifacts, and treat an
  unexpected oracle or rendering delta as a blocker.

No dependency-graph, layout-engine, font-selection, unit, colour, binding-API,
feature, module, or file-move row is triggered.

## Hash harness

Expected change to exactly seven `word/document.xml` entries because modeled
empty paragraphs serialize as self-closing XML. All seven PNG entries and all
PDF fingerprints must remain unchanged. Any other delta blocks integration.

## Implementation checklist

- [ ] Pin and audit all four exact contributor source heads.
- [ ] Adopt PR 77's narrowed numbering evidence contract.
- [ ] Integrate and harden PR 78's document, revision, table, text, and glossary reader semantics.
- [ ] Integrate and harden PR 79's marker and field reader facts across every consumer.
- [ ] Integrate and harden PR 80's table-cell serialization and TOC coordinates.
- [ ] Move Presentation fidelity to the exact Ubuntu LibreOffice and Poppler oracle.
- [ ] Add the focused Rust, Python, and workflow mutation regressions.
- [ ] Record and review the exact seven-entry Word XML hash delta.
- [ ] Run the complete test gate, full verification, public API riders, and external-oracle gate.
- [ ] Update exactly the listed HLD files and retain contributor credit.
- [ ] Leave specific human-written final comments on issues 72 through 76 and PRs 77 through 80.

## Open questions

None. The user explicitly requested the four PR outcomes, CI repair,
version-aligned PyPI publication, and final comments. The source heads and
deterministic oracle are pinned, and the separate release approval remains
mandatory at the final reviewed SHA.
