# F-261, all aspects, pass 1

**Reviewed**: working tree against `373d5a701615bef07a2970be2d6709388899e624`, 12 files, 899 insertions and 59 deletions
**Verdict**: 3 defects, 0 smells, 1 nitpick

## Defects

### D1, unsupported story kinds are accepted by the public operation

`crates/rdocx/src/document.rs:12582`

The implementation validates that the supplied story exists but never limits
the destination to body, cell, header, or footer. A footnote, endnote, comment,
or text-box location therefore enters the generic XML branch even though the
approved contract and public specification expose only the four tested
containers. This silently expands the public API into unreviewed story grammar
and relationship paths.

### D2, main-part cell lookup can target the wrong modeled cell

`crates/rdocx/src/document.rs:8008`

`StoryId::owner_index` counts every discoverable table-cell owner in physical
XML order, including cells nested under a text-box paragraph. The typed lookup
counts only body tables and content controls and explicitly skips paragraphs
and raw XML. If a discoverable cell occurs in a text box before an ordinary
body table, insertion into the ordinary cell uses the larger physical owner
index and either mutates a later typed cell or returns `OwnerNotFound`. The
checked destination must be mapped to the exact typed owner or rejected before
projection rather than interpreted in a different coordinate system.

### D3, the named differential gate contains no rendering or external oracle

`crates/rdocx/tests/integration_test.rs:4837`

The gate checks reopened structure, relationships, diagnostics, and atomicity,
but never renders the document and never compares a pinned Word, LibreOffice,
or Poppler record. The approved test contract requires equivalent Word content
and rendering, and the risk rider explicitly requires pinned Word,
LibreOffice, and Poppler evidence at `.claude/plans/F-261-design.md:47` and
`.claude/plans/F-261-design.md:69`. The current test name claims a differential
that it does not perform.

## Smells

None.

## Nitpicks

- `crates/rdocx/src/html.rs:115`, `MhtmlProjection` now carries ordinary HTML fragment resources and links, so its format-specific name obscures the shared role.

## Not found

No additional correctness, panic, OOXML child-order, namespace-preservation,
diagnostic-order, atomicity, or structural-rule findings were identified.
