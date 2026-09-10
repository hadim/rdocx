# F-253, all, pass 19

**Reviewed**: complete uncommitted working-tree diff on
`work/f-253-codex`, 8 files and 5,300 changed lines, comprising 5,239
insertions and 61 deletions
**Verdict**: 1 defect, 0 smells, 0 nitpicks

## Defects

### D1, an unclosed field in one paragraph hides valid fields in later paragraphs
`crates/rdocx/src/document.rs:4418`
`crates/rdocx/src/document.rs:4579`
`crates/rdocx/src/document.rs:4646`

The complex-field scan stack is allocated once for the complete story owner
and is not cleared when a paragraph closes. A begin marker without an end in
one paragraph therefore remains an ancestor of a valid standalone field in a
later paragraph. The later candidate passes its isolated typed admission, but
its recorded ancestor has no completed candidate and cannot have an admission
entry. The final ancestor check drops the valid later field. The typed model
parses each `CT_P` independently, so a malformed field in one paragraph must
not change field projection in a following paragraph. Reset or invalidate the
scanner state at each paragraph boundary while retaining same-paragraph nested
field context.

## Smells

None. Count: 0.

## Nitpicks

None. Count: 0.

## Not found

- **Pass 18 descendant shadowing**: isolated admission now obtains the complete
  namespace scope at the live enclosing run and chooses a prefix that is bound
  to WordprocessingML in that scope. A foreign descendant binding of the
  original paragraph prefix no longer changes the synthetic root namespace.
- **Pass 17 namespace and ancestry cases**: wrapper-local Word aliases remain
  available to isolated parsing, and invalid enclosing candidates suppress
  their descendants within one paragraph. D1 is limited to scanner state that
  incorrectly survives a paragraph boundary.
- **Earlier parser-equivalence cases**: no additional definite mismatch was
  found for preserved boundaries, modeled container grammar, simple fields,
  revisions, same-run siblings, cached-result nesting, or field instruction
  parsing.
- **OOXML and raw preservation**: no additional fixed-prefix read, incomplete
  namespace scope, schema-order, whitespace, note-filtering, or unrelated-byte
  rewrite defect was found. Complex-field XML remains a standalone projection
  with the needed in-scope bindings.
- **Identity, atomicity, and lifecycle**: no additional owner-order,
  fingerprint, bounds, kind, staged-commit, identifier provenance,
  optional-part ownership, dirty-state, or signature-state defect was found.
- **Panics**: no added production panic, unchecked index, invalid slice, or
  non-wrapping arithmetic on story input was found.
- **API and tests**: the public surface remains additive and concrete. The
  named integration, round-trip, and atomicity gates cover the approved story
  kinds and item categories, but the field regressions do not isolate a valid
  paragraph after an earlier paragraph with an unclosed field.
- **Structure**: no new trait, generic parameter, feature flag, crate, module,
  forwarding wrapper, or unnecessary dynamic dispatch was introduced.
