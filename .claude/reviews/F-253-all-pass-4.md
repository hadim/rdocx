# F-253, all, pass 4

**Reviewed**: complete uncommitted working-tree diff on
`work/f-253-codex`, 5 files and 1,812 changed lines, comprising 1,806
insertions and 6 deletions
**Verdict**: 1 defect, 0 smells, 0 nitpicks

## Defects

### D1, successful story mutation discards identifier provenance
`crates/rdocx/src/document.rs:7661`
`crates/rdocx/src/document.rs:6790`

After patching the staged package, `set_story_text` reopens it through
`Self::from_bytes` and commits that newly imported document. This rescans every
existing identifier as imported package state and loses the authored and
preserved classifications carried by the staged candidate. The repository's
dedicated staged-reopen path explicitly calls `reconcile_provenance` to avoid
that loss. A caller that authors identifiers, edits story text, then performs a
later identifier-bearing mutation or save can therefore retain operation-order
identifiers that should still be canonicalized as authored. The successful
text edit must reopen while preserving the candidate's identifier provenance
and its other nonserialized mutation state.

## Smells

None. Count: 0.

## Nitpicks

None. Count: 0.

## Not found

- **Pass 3 D1**: `w:id` is XML-decoded and normalized before integer parsing,
  and the focused escaped-nonpositive-id regression proves the remediation.
- **Correctness**: no additional wrong logic, boundary error, or unsupported
  mutation was found beyond D1.
- **Contract**: traversal, identity, item projection, and uniform location
  errors match the approved design apart from D1's loss of staged state.
- **Panics**: no production `unwrap`, `expect`, unchecked indexing, slicing,
  or non-wrapping arithmetic was added on untrusted input.
- **OOXML**: no additional namespace, schema-order, whitespace, escaped-value,
  or retained-subtree defect was found.
- **Tests**: the named integration, round-trip, and regression gates exercise
  the approved story behavior. They do not cover preserving authored identifier
  provenance across a successful story edit, which is the trigger for D1.
- **Structure**: no new trait, generic parameter, feature flag, crate, module,
  forwarding wrapper, or dynamically dispatched concrete dependency was
  introduced.
