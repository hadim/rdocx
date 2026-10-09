# F-X196, Validate visible paragraph content without false empty warnings

**Status**: approved
**Sprint**: S91
**Size**: S
**Depends on**: none

## Problem

`crates/rdocx-cli/src/commands.rs:2318` counts trimmed text only. Paragraphs
containing pictures and other visible nontext content trigger empty warnings.

## Spec reference

- `docs/hld/10-bindings-spec.md`, CLIs, validation advisory findings.
- `docs/hld/12-testing-strategy.md`, Test taxonomy, regression.
- `docs/hld/14-development-backlog.md`, F-X196 and full Issue298 criteria.

## Approach

Keep the advisory body-paragraph inventory and metadata warnings. Replace
its text-only predicate with a private content-aware check in existing
commands.rs. Text-bearing paragraphs remain nonempty. For otherwise blank
paragraphs, obtain their existing checked paragraph story location and source
snapshot, then inspect the namespace-qualified preserved XML using existing
quick-xml. No new public facade surface or dependency is needed.

The scanner detects actual text and Word drawing/pict/object, field elements,
symbols, tabs, breaks, footnote/endnote references and OfficeMath content.
Descend through content-control content and supported inline wrappers. Do not
count paragraph properties, bookmarks, range markers, deleted text or
lastRenderedPageBreak producer hints as visible content. Properties inside
sdtPr are not rendered children. Use expanded names rather than local-name
matching foreign producer elements. A truly empty content control remains
empty, and a content control with a visible nontext child is nonempty.
The scan is read-only and never rewrites the source or changes revision.
Malformed snapshot input reports the existing CLI error rather than being
silently classified as empty. Preserve the original counting scope.

## Rejected alternatives

- Test only drawings, which leaves every other listed content kind broken.
- Count any XML tag as visible, which treats bookmarks and properties as text.
- Add a forwarding public facade method solely for this CLI predicate.

## Test plan

**Test gate**: regression.

| Category | Test | Asserts |
|---|---|---|
| regression | picture_paragraphs_do_not_trigger_empty_paragraph_warnings | Reported three-picture document gives metadata warnings only |
| regression | visible_nontext_paragraph_content_is_not_empty | Every Issue298 content kind and nested content-control child is recognized |
| regression | paragraph_properties_and_range_markers_do_not_invent_visible_content | Empty, whitespace-only, marker-only, property-only and empty-control paragraphs remain empty |
| round-trip | validating_nontext_content_preserves_document_bytes | Snapshot/validation is read-only and title/author diagnostics remain unchanged |

Add tests to the existing commands.rs test module and existing CLI
integration entrypoint. Use source-built XML and generated image bytes.

## HLD impact

- `docs/hld/10-bindings-spec.md`
- `docs/hld/14-development-backlog.md`

## Risk routing

- Parser: read HLD04 The package and HLD06 parser conventions, qualify expanded
  names and preserve opaque bytes. Positive namespace aliases and foreign
  lookalikes distinguish the scanner.
- Published CLI behavior changes only the incorrect advisory warning. No
  library API, version, dependency, flag, trait, generic, module or file change.

## Hash harness

Expected unchanged. Validation reads documents without rendering or mutation.

## Exclusive file claims

- `crates/rdocx-cli/src/commands.rs`
- Existing CLI integration test entrypoint
- HLD files above and shared Cargo/test execution

## Implementation checklist

- [ ] Prove the three-picture warning failure at exact Base.
- [ ] Add private content-aware predicate and qualified XML scan.
- [ ] Cover every visible kind and empty/metadata controls.
- [ ] Verify source preservation, scoped gate and clean microscope.
- [ ] Prepare validated handoff.

## Open questions

None. Empty means no visible content, as explicitly requested in Issue298.
