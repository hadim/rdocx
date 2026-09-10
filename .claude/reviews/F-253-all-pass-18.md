# F-253, all, pass 18

**Reviewed**: complete uncommitted working-tree diff on
`work/f-253-codex`, 8 files and 5,226 changed lines, comprising 5,165
insertions and 61 deletions
**Verdict**: 1 defect, 0 smells, 0 nitpicks

## Defects

### D1, descendant prefix shadowing can change the synthetic paragraph root namespace
`crates/rdocx/src/document.rs:4374`
`crates/rdocx/src/document.rs:4377`
`crates/rdocx/src/document.rs:4378`
`crates/rdocx-oxml/src/text.rs:3683`

Candidate admission correctly gathers the complete namespace scope at the
enclosing run, but applies that descendant scope to the original paragraph
qualified name. A valid paragraph can use `w` for WordprocessingML, then
contain a `q:hyperlink` that declares `q` as WordprocessingML and shadows `w`
as `urn:producer`, with a `q:r` and `q:` complex-field children below it. The
normal typed paragraph path recognizes the original `w:p`, then carries the
child scope into the hyperlink and run. Admission instead constructs a root
equivalent to `<w:p xmlns:q="...word..." xmlns:w="urn:producer">`. The typed
fragment parser rejects that root because its `w` prefix is no longer a Word
prefix, so the valid field is omitted from story traversal. The synthetic
paragraph needs its own Word-qualified name that cannot collide with the
materialized run scope.

## Smells

None. Count: 0.

## Nitpicks

None. Count: 0.

## Not found

- **Pass 17 enclosing validity**: candidate ancestry now records every open
  field marker and requires each enclosing candidate to pass typed admission.
  A valid inner result field under an invalid outer field is excluded.
- **Pass 16 valid outer nesting**: a valid outer field and a same-run field in
  its cached result are independently admitted, projected, and edited by the
  focused regression. Marker-bounded ranges keep their content separate.
- **Parser equivalence outside D1**: no additional definite mismatch was found
  for rejected boundaries, opaque lookalikes, sibling fields, or nested-field
  validity.
- **OOXML outside D1**: no additional fixed-prefix read, schema-order,
  whitespace, note-filtering, or retained-subtree defect was found.
- **Identity, atomicity, and lifecycle**: no additional owner-order,
  fingerprint, bounds, kind, staged-commit, provenance, optional-part,
  dirty-state, or signature-state defect was found.
- **Panics**: no added production panic, unchecked index, invalid slice, or
  non-wrapping arithmetic on story input was found.
- **API and tests**: the public surface remains additive and the named
  integration, round-trip, and atomicity gates exercise the approved story
  kinds and item categories. The namespace regression covers a local Word
  alias but does not shadow the paragraph prefix below the paragraph root.
- **Structure**: no new trait, generic parameter, feature flag, crate, module,
  forwarding wrapper, or unnecessary dynamic dispatch was introduced.
