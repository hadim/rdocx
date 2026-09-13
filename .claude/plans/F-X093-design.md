# F-X093, Preserve drawings through document comparison staging

**Status**: approved
**Sprint**: S72
**Size**: M
**Depends on**: F-255

## Problem

Comparison prepares both packages, then reconstructs main-story XML from the
typed document and reparses a tracked body. A drawing whose inline or anchor
payload inherits namespace declarations from its original `w:drawing` wrapper
can lose those declarations. GitHub Issue 75 then reports a misleading missing
`wp:docPr/@id` error although each input opens and saves independently.

## Spec reference

- `docs/hld/04-opc-and-packaging.md`, namespace-complete staged comparison.
- `docs/hld/10-bindings-spec.md`, native comparison and accept or reject.
- `docs/hld/12-testing-strategy.md`, comparison and drawing preservation.
- `docs/hld/13-risks-and-open-questions.md`, raw XML staging invariants.
- `docs/hld/14-development-backlog.md`, F-X093.

## Approach

Build the exact issue shape in the existing regression binary. Use prepared
package-authoritative main XML for comparison input and tracked-body
replacement, retaining the parsed models only for owner alignment. Preserve
complete drawing wrappers, namespace declarations, relationships, media, and
raw payloads. Let staged reopen validate the complete result. If the exact
test already passes after F-255, add only the permanent closure regression and
HLD clarification.

## Rejected alternatives

- Strip and reinsert drawings. That can duplicate or reorder nontext content.
- Invent missing drawing ids. Truly malformed inputs must remain invalid.
- Copy guessed namespace declarations. The prepared package already owns the
  exact scopes.
- Diff DrawingML internals. This story preserves drawings through text compare.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| regression | `document_compare_preserves_inline_drawings_through_staging` | Body and header drawings compare, save, and reopen with exact wrappers, ids, relationships, and media. |
| round-trip | `comparison_drawings_survive_accept_and_reject` | Accept reproduces edited text, reject reproduces original text, and both preserve the drawing graph. |
| regression | `comparison_preserves_anchored_and_extended_doc_pr_payloads` | Anchors, aliases, and extended `docPr` children remain complete. |
| regression | `comparison_rejects_a_genuinely_missing_doc_pr_atomically` | Malformed input returns the required error without publication. |

The test gate is `document_compare_preserves_inline_drawings_through_staging`.

## HLD impact

- `docs/hld/04-opc-and-packaging.md`
- `docs/hld/10-bindings-spec.md`
- `docs/hld/12-testing-strategy.md`
- `docs/hld/13-risks-and-open-questions.md`
- `docs/hld/14-development-backlog.md`

## Risk routing

- Any parser or serialiser. Prove prefix-tolerant reopen, fixed revision
  prefixes, schema order, and byte-exact unchanged drawing and relationship
  preservation.

## Hash harness

Expected unchanged across all 49 entries. Any delta blocks completion.

## Implementation checklist

- [ ] Add the exact source-built comparison reproduction before source edits.
- [ ] Read prepared main XML from package-authoritative bytes.
- [ ] Preserve drawing wrappers, namespace scopes, relationships, and media.
- [ ] Prove compare, save, reopen, accept, and reject.
- [ ] Retain atomic failure for a genuinely malformed drawing.
- [ ] Run the full gate and unchanged hash harness.
- [ ] Update exactly the listed HLD files.

## Open questions

None.
