# F-253, all, pass 5

**Reviewed**: complete uncommitted working-tree diff on
`work/f-253-codex`, 5 files and 1,886 changed lines, comprising 1,880
insertions and 6 deletions
**Verdict**: 1 defect, 0 smells, 0 nitpicks

## Defects

### D1, staged reopen still discards facade-owned part provenance
`crates/rdocx/src/document.rs:7659`
`crates/rdocx/src/document.rs:6789`
`crates/rdocx/src/document.rs:6556`
`crates/rdocx/src/document.rs:6577`

The pass 4 remediation now reopens through the provenance-reconciling staged
path, which preserves authored identifier history. That path still reconstructs
the document through `from_bytes_with_limits`, where `custom_properties_owned`,
`comments_owned`, and the other facade-ownership flags are reset to false. It
restores identifier and signature state only. A caller can create a custom
properties or comments part through the facade, perform a successful story text
edit, then remove the final custom property or comment. The later removal no
longer recognizes the part as facade-owned and leaves the newly authored empty
part and relationship behind. Story mutation must retain this nonserialized
ownership provenance as well as identifier and signature provenance.

## Smells

None. Count: 0.

## Nitpicks

None. Count: 0.

## Not found

- **Pass 4 identifier provenance**: `reopen_prepared_staged` reconciles the
  reopened identifier registry with the staged candidate, and the focused
  bookmark regression proves authored identifier history survives a successful
  story mutation. D1 is the remaining sibling state-loss path.
- **Correctness**: no additional wrong logic, boundary error, or unsupported
  mutation was found beyond D1.
- **Contract**: traversal, identity, item projection, stable ordering, uniform
  location errors, and atomic rejection match the approved design apart from
  D1's cross-operation state loss.
- **Panics**: no production `unwrap`, `expect`, unchecked indexing, slicing, or
  non-wrapping arithmetic was added on untrusted input.
- **OOXML**: no namespace, schema-order, whitespace, escaped-value, or retained
  subtree defect was found.
- **Tests**: the named integration, round-trip, and regression gates exercise
  the approved story behavior. The provenance regression covers authored
  identifiers but not facade-owned package-part state, which is the trigger for
  D1.
- **Structure**: no new trait, generic parameter, feature flag, crate, module,
  forwarding wrapper, or dynamically dispatched concrete dependency was
  introduced.
