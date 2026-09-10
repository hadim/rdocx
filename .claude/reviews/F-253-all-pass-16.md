# F-253, all, pass 16

**Reviewed**: complete uncommitted working-tree diff on
`work/f-253-codex`, 8 files and 4,949 changed lines, comprising 4,888
insertions and 61 deletions
**Verdict**: 1 defect, 0 smells, 0 nitpicks

## Defects

### D1, same-run fields nested in a cached result are dropped from traversal
`crates/rdocx/src/document.rs:4570`
`crates/rdocx/src/document.rs:4575`
`crates/rdocx-oxml/src/text.rs:2554`

Candidates that share one enclosing run also share one `scan` range. The
admission code asks the typed projection only for a count, then admits that
many candidates from the sorted group. When a valid inner field is nested in
the cached-result side of a valid outer field and both are contained in one
run, the typed projection completes the inner field while an outer field is
open, but does not attach it to the outer instruction or add it to the
top-level completed list. The helper therefore returns one source for the
shared run even though the facade scanner found two valid fields. Sorting by
marker start admits the outer field and drops the inner one. The same shape
across separate runs is already required to expose both fields by
`crates/rdocx/tests/regression_test.rs:896`. Per-span admission must identify
the exact marker-bounded candidate instead of using a shared-run source count.

## Smells

None. Count: 0.

## Nitpicks

None. Count: 0.

## Not found

- **Pass 15 mixed siblings**: independently scanned valid fields remain
  admitted when a different run span in the paragraph crosses a typed
  boundary. D1 is limited to multiple marker-bounded candidates that share one
  enclosing run.
- **Pass 14 opaque lookalikes**: isolated scan parsing prevents a byte-equal
  field inside a preserved wrapper from borrowing admission from a field in a
  different paragraph or run span.
- **Pass 14 namespaces**: complex-field projection retains inherited foreign
  bindings and selects a collision-free Word prefix.
- **Pass 14 range isolation**: marker-bounded `full` ranges keep same-run
  sibling and instruction-nested field text, XML, and mutation separate. D1 is
  an admission omission for a field nested after the outer separator.
- **Correctness and contract outside D1**: owner order, story identity,
  checked location resolution, fingerprint staleness, and staged atomic commit
  remain aligned with the approved contract.
- **OOXML outside D1**: no additional fixed-prefix read, namespace collision,
  schema-order, whitespace, note-filtering, or retained-subtree defect was
  found.
- **Panics**: no added production panic, unchecked index, invalid slice, or
  non-wrapping arithmetic on story input was found.
- **Lifecycle**: identifier provenance, facade-owned part state, signature
  state, and dirty-state restoration remain preserved through staged reopen.
- **Tests**: the named integration, round-trip, and atomicity gates cover the
  approved story surface. Existing same-run coverage exercises siblings and an
  instruction-nested field, but not a field nested in the outer cached result.
- **Structure**: no new trait, generic parameter, feature flag, crate, module,
  forwarding wrapper, or unnecessary dynamic dispatch was introduced.
