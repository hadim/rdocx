# F-X201, Complete Python section stories and page authoring

**Status**: draft
**Sprint**: S91
**Size**: L
**Depends on**: none

## Problem

Issue304 requires complete rich section and note authoring. Native insertion
at `crates/rdocx/src/document.rs:23892` creates empty section properties.
Python `update_section` at `crates/rdocx-py/src/document.rs:2531` supplies
page dimensions only when a dimension keyword is given. Native
`Section::set_orientation` at document.rs:3230 already swaps existing
width/height correctly, but cannot swap the absent dimensions of a new section.
Python `create_section_story` at document.rs:3059 delegates without enabling
even-page settings. Current story and section accessors return snapshots
rather than live rich authoring handles, per HLD10's section contract.

## Spec reference

- `docs/hld/10-bindings-spec.md`, Native Word facade stability, section
  geometry, story lifecycle, footnote/endnote authoring and Python snapshots.
- `docs/hld/10-bindings-spec.md`, The chosen design and invalidation problem.
- `docs/hld/04-opc-and-packaging.md`, The package and preserved relationships.
- `docs/hld/08-rendering-spec.md`, section pagination and note layout.
- `docs/hld/14-development-backlog.md`, F-X201 and Issue304 acceptance.

## Approach

Preserve the existing checked orientation normalization. New sections inherit
resolved previous-section page size, margins, gutter and header/footer
geometry, filling actual layout defaults where source geometry is absent.
Do not accidentally clone explicit header/footer relationships or other
section ownership. Define index-zero inheritance from the current first
section, and an empty document from the existing Letter defaults. Invalid
lengths, indices and partial changes remain atomic. Add incomplete pgSz
findings to the shared validation contract consumed by F-X203.

Even story creation and settings must publish in one staged transaction.
Expose `settings.odd_and_even_pages_header_footer` through the existing
native settings API, with success-only revision changes. A refused story
creation must not turn the setting on. Retain inherited and first-page
ownership and opaque source content.

Implement live rich section/header/footer access in existing binding modules,
with existing checked story identity and paragraph/run resolution. Existing
snapshot fields and Document.update_section callers remain usable. Story
paragraphs, add_paragraph, add_table, run.add_picture and fields address the
actual owning part, never build in the body then silently transfer content.
Expose different_first_page_header_footer and is_linked_to_previous through
native checked lifecycle operations.

Expose note creation/deletion plus numbering/restart from the native staged
note APIs. Accept a checked paragraph or run from the same document, rejecting
foreign or stale handles before publishing the note or reference. Add run
page/column breaks through native run authoring. Page colour writes both
w:background and displayBackgroundShape while retaining unknown children.
Expose existing text/image watermarks and page borders. Prefer DrawingML
behind-text header watermarks for the requested Google import behavior.

Section lengths remain EMU integers for compatibility and accept existing
Length subclasses, including Twips, Pt and Inches, by their EMU value. Preserve
pinned truncation and diagnose invalid unit/type values explicitly.

## Rejected alternatives

- Rewrite orientation normalization, which already works with dimensions.
- Copy complete sectPr relationships when only geometry is inherited.
- Author section content in the body and expose detached snapshots as live.
- Enable even settings before a later failing story creation.

## Test plan

**Test gate**: regression.

| Category | Test | Asserts |
|---|---|---|
| regression | inserted_section_inherits_geometry_and_normalizes_orientation | New section has complete geometry, landscape swaps dimensions, inherited margins and distances survive reopen |
| regression | even_footer_creation_and_setting_are_atomic | Even footer is effective, failed creation preserves settings and bytes |
| regression | section_lengths_accept_existing_length_units | EMU integers and Twips/Pt/Inches agree, no silent twip confusion |
| round-trip | rich_footer_page_x_of_y_uses_live_story_handles | Entire footer built in Python, styled fields, picture and table use footer relationships and render correct numbers |
| regression | footnote_endnote_authoring_rejects_foreign_and_stale_locations | Creation/deletion and numbering are correct, errors publish nothing |
| round-trip | page_authoring_properties_preserve_unknown_source | Breaks, colour, watermarks and borders reopen and render with opaque bytes retained |
| regression | incomplete_page_size_is_reported_consistently | Native/Python/CLI shared findings agree after F-X203 integration |

Add cases to existing native regression and Python test entrypoints. Build a
fresh installed extension from the actual worker source, run strict mypy and
stubtest, and record exact pinned Word/LibreOffice render controls in
deterministic font mode. Reproduce the two reported geometry/footer defects
on the claimed Base before production changes.

## HLD impact

- `docs/hld/02-scope-and-non-goals.md`
- `docs/hld/04-opc-and-packaging.md`
- `docs/hld/08-rendering-spec.md`
- `docs/hld/10-bindings-spec.md`
- `docs/hld/12-testing-strategy.md`
- `docs/hld/14-development-backlog.md`

## Risk routing

- Units: HLD01 units and pinned truncation, test equivalent EMU inputs.
- Parser/serializer: HLD04 and HLD06 conventions, namespace qualification,
  schema sequence and exact opaque-subtree preservation.
- Layout: HLD08 section and note pagination, deterministic native controls.
- Published API: additive pre-1.0 authoring, reviewed local-source packaging
  dry run and actual affected archive records below the size ceiling.
- PyO3: HLD10 handles, isolated installed tests, strict typing and stubtest,
  plus both WASM target checks for the changed common native facade.
- Oracle: pin Word/LibreOffice versions and retain actual compared output.

## Hash harness

The geometry and even-story repairs intentionally change output for authored
new sections and enabled even stories. Identify any affected hash fixture
before implementation and declare its exact delta in an approved revision.
Unrelated output must remain unchanged. No baseline movement is authorized
by this draft. Geometry-changing commits are labeled separately from wrappers.

## Exclusive file claims

Existing rdocx document/paragraph/run modules, rdocx-oxml section/settings
models if needed, rdocx-py document/paragraph/run/formatting modules and stubs,
existing native and installed Python test entrypoints, validation consumers,
and the HLD sections above. Resolve collisions with X198, X194, X200, X202,
X203, X192 and F-283 before assigning the wave. No new source module or crate.

## Implementation checklist

- [ ] Finalize exact live Section/Story handle signatures and native gaps.
- [ ] Approve validation ownership and measured expected hash changes.
- [ ] Claim after source conflicts clear and reproduce actual defects.
- [ ] Implement geometry and even-story atomicity, then rich APIs.
- [ ] Run all Issue304 acceptance, scoped riders and independent microscope.
- [ ] Prepare a validated feature-local handoff.

## Open questions

Technical design remains unfinished: determine the smallest live section and
story accessors compatible with existing snapshot consumers, and determine
which validation finding representation can be shared with F-X203 without
introducing a forwarding wrapper or library dependency on the CLI. Verify
requested note APIs can address run locations without reducing native safety.
This draft is not approved for implementation.
