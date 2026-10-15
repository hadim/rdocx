# F-X231, Preserve PowerPoint HTML import whitespace

**Status**: approved
**Sprint**: S92
**Size**: M
**Depends on**: F-X217

## Problem

Presentation HTML import collapses Unicode whitespace, including NBSP, in
`crates/rpptx/src/html.rs:2934-2948`. Its pending space takes the next text
piece's properties in `write_paragraphs` at `:2413-2450`. Spaces consequently
move into emphasis or hyperlinks instead of retaining their source owner.
Unicode trimming at `:1995`, `:2826` and `:2832` can also discard NBSP-only
content before projection.

Equivalent Word source-owner and ASCII-whitespace behavior already exists in
`crates/rdocx/src/html.rs:4243-4266` and `:4429-4453`. The existing
`fx217_review_nonbreaking_space_only_content_survives` at `:4962` covers its
visibility policy. Qualify advanced Word cases without duplicating implementation.

## Spec reference

- `docs/hld/14-development-backlog.md`, F-X231.
- `docs/hld/02-scope-and-non-goals.md`, capability table, HTML slide content
  import row and Word interchange boundary.
- `docs/hld/10-bindings-spec.md`, Native Word facade stability, HTML import and
  checked insertion paragraphs at lines1697-1734 and native presentation HTML
  import contract at lines1774-1780.
- `docs/hld/04-opc-and-packaging.md`, Media, caller-owned presentation HTML
  resources, and Package integrity, presentation HTML candidate publication.
- `docs/hld/06-presentationml-model.md`, Preservation strategy and Validation.

## Approach

Assess and reuse only PR359's PowerPoint whitespace hunks and applicable tests
at exact head `e974ea2240691c0de581efa172ba8c88a77c054b`, authored by
`hadim`. Preserve contribution attribution. Do not adopt the stacked PR's
already delivered Word, Markdown, dependency, binding or export implementation.

In existing `crates/rpptx/src/html.rs`, retain the properties of the first
piece producing a pending collapsed space, including hyperlink properties.
Flush it into its producing run or a separate run with saved properties before
differently formatted visible text. Whitespace across span boundaries produces
one space. Paragraph-leading and paragraph-trailing collapsible whitespace
remains trimmed. Explicit breaks reset the pending sequence.

Use the existing Word ASCII-whitespace collapse policy. NBSP and other
non-ASCII spacing characters remain literal text. Reproduce NBSP-only shape,
flow and cell losses, then narrowly replace the implicated HTML visibility
Unicode trimming with the same collapsible-whitespace predicate. Preserve
limits, ordered diagnostics and atomic validated candidate publication.

Exercise native `Presentation::from_html` and `Presentation::open_html`.
Presentation HTML import currently has no Python, CLI or WASM entrypoint.
The binding specification expressly retains that boundary. Fresh installed PPT
qualification loads the actual native-imported PPTX through `Presentation(path)`
and `Presentation.from_bytes`, checks exact public runs and links, saves and
reopens. No new presentation import binding is proposed.

Qualify already equivalent Word behavior through native from_html, open_html
and insert_html_fragment, installed Python from_html, open_html and insert_html,
and the existing HTML-to-DOCX CLI. Retain applicable supported Markdown
source-owner and code-space controls. These are regression controls, not a
second importer. Keep native Word tests inside the existing html.rs unit module
to avoid F-X178's integration_test.rs lease in wave1. Source-built fixtures and
tests use existing carriers only. No new product file, dependency or API.

## Rejected alternatives

- Adopt the entire stacked PR, which duplicates completed F-X203/F-X217 work.
- Trim each run independently, which loses inter-run spaces and source ownership.
- Assign spaces to the following run, which is the reported defect.
- Add PPT import bindings, which exceeds the registered repair and public scope.
- Use semantic export as import proof, which conflates F-X218's separate contract.

## Test plan

The **test gate** is **regression**, verbatim from F-X231.

| Category | Test | Asserts |
|---|---|---|
| regression | `html_import_keeps_the_space_around_an_emphasised_span_outside_it` | Reuse donor exact text and underline, strike, bold and italic facts, plain inter-span spaces and spaces inside emphasis |
| regression | `html_import_collapses_spaces_on_both_sides_of_a_span_edge_into_one` | Reuse donor repeated-edge, nested-span, whitespace-only span, link and NBSP cases with first-source ownership |
| regression | `fx231_nbsp_only_content_is_not_discarded` | Positioned and flow NBSP-only text and table cells survive, repeated NBSP and adjacent ASCII spaces stay exact |
| regression | `fx231_pending_space_preserves_full_properties_and_link_owner` | Font, colour, marks and hyperlink relationship targets remain with their source, including whitespace-only intermediate spans |
| regression | `fx231_whitespace_boundaries_reset_between_paragraphs_and_breaks` | Leading/trailing ASCII whitespace, empty paragraphs and explicit breaks cannot leak pending state |
| integration | `fx231_native_html_entrypoints_save_reopen_exact_runs` | Both native constructors match explicit source-authored run expectations before and after bytes/path save/reopen in shapes and cells |
| regression | `fx231_unsupported_html_keeps_ordered_diagnostics` | Unsupported elements, CSS and resources retain location/property/message order while supported siblings keep corrected spaces |
| regression | `fx231_word_advanced_whitespace_equivalence` | Advanced donor Word HTML, links, nested marks, NBSP, Markdown and fragment cases pass unchanged, with reopened exact runs |
| integration | `test_fx231_installed_presentation_imported_whitespace_roundtrip` | Fresh installed PPT reads actual native output by path and bytes, preserving public run/link facts through save/reopen |
| regression | `test_fx231_installed_word_html_whitespace_entrypoints` | Fresh installed Word HTML construction, file import and insertion preserve the same explicit reopened run expectations |
| integration | `fx231_cli_word_html_whitespace_roundtrip` | Existing Word HTML CLI produces exact run/link/NBSP facts and unchanged diagnostics |

Native PPT tests use existing `crates/rpptx/tests/integration.rs`. Native Word
tests use existing `crates/rdocx/src/html.rs` test module. Python tests use the
existing test_core.py carriers. CLI tests use its existing integration carrier.
Do not filter whitespace-only runs from assertions or compare only joined text.
Check actual reopened OOXML and hyperlink relationship targets independently.

Establish genuine Base failures for PPT ownership, repeated spans, NBSP and
NBSP-only projection. Word controls should pass Base. Missing future symbols
do not count as RED evidence. Retain existing PPT HTML editable save/reopen,
diagnostics and resource-limit controls. Add an opaque-extension and unchanged
template-parts save/reopen control to demonstrate preservation.

Focused checks:

- `cargo check -p rpptx --all-targets`
- `cargo test -p rpptx --test integration html_import_`
- `cargo test -p rpptx --test integration fx231_`
- `cargo test -p rdocx --lib fx231_`
- `cargo test -p rdocx --lib fx217_review_nonbreaking_space_only_content_survives`
- Exact collected installed Python nodes above, with fresh wheel/source
  provenance and nonzero collected counts.
- `cargo test -p rdocx-cli --test integration fx231_`

Run the ordinary scoped feature verification and independent microscope. The
integrator runs final full verification and sprint review once.

## HLD impact

- `docs/hld/10-bindings-spec.md`, exact collapsed-space ownership and NBSP
  preservation in the existing native-only presentation import contract.

## Risk routing

- Parser or serialiser. Read 04 and 06. Preserve child sequence, prefix-tolerant
  reads and fixed-prefix writes. Add exact run/link save/reopen assertions,
  byte-for-byte injected opaque-extension retention and unchanged template-part
  assertions. Existing OOXML serializers remain the implementation provider.
- No public API signature, dependency graph, binding implementation, layout
  algorithm, font asset, feature flag, new file or external oracle is proposed.
  Fresh installed qualification is explicit story acceptance. No native Office
  or browser oracle result is claimed from product-generated expectations.
- If implementation expands into an applicable router row, revise the plan
  before that expansion and add the row's required riders.

## Hash harness

Existing 49 deterministic baselines are expected unchanged. Corrected selected
PPT HTML inputs intentionally change run-space ownership and NBSP preservation.
State those selected behavioral deltas explicitly in the labelled feature
commit. An existing baseline change requires measured, attributed and reviewed
evidence before recording.

## Implementation checklist

- [ ] Capture exact donor head, relevant hunks and contribution attribution.
- [ ] Establish native/installed entrypoint inventory, PPT RED and Word controls.
- [ ] Port necessary PPT source-owner collapse behavior only.
- [ ] Reproduce and narrowly repair NBSP visibility losses.
- [ ] Assert complete formatting, links and text through save/reopen.
- [ ] Qualify equivalent Word behavior once through existing public surfaces.
- [ ] Verify diagnostics, limits, opaque extensions and template preservation.
- [ ] Run focused checks, microscope, scoped verification and unchanged hash gate.
- [ ] Update listed HLD section and complete through the sprint workflow.

## Open questions

No material unanswered question. Installed PPT acceptance uses its existing
PPTX read/save APIs, while actual HTML import remains native Rust only.
NBSP-only supported content is part of NBSP preservation after a real Base
reproduction. No new file or dependency is proposed.
