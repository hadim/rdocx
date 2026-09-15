# F-253, all, pass 17

**Reviewed**: complete uncommitted working-tree diff on
`work/f-253-codex`, 8 files and 4,949 changed lines, comprising 4,888
insertions and 61 deletions
**Verdict**: 2 defects, 0 smells, 0 nitpicks

## Defects

### D1, isolated admission loses an inline owner's namespace scope
`crates/rdocx/src/document.rs:4366`
`crates/rdocx/src/document.rs:4376`
`crates/rdocx-oxml/src/text.rs:3771`

Per-candidate admission keeps the enclosing run shell and marker-bounded
field bytes, but discards every modeled wrapper around that run. It then
passes only the paragraph's Word-prefix list to the typed parser. A valid
hyperlink can declare a Word alias such as `q` on `w:hyperlink`, use an
ordinary `w:r`, and use `q:fldChar`, `q:instrText`, and `q:t` below that run.
The normal typed paragraph path carries the hyperlink declaration into its
child parser, while the isolated fragment has no `q` binding. The exact field
is consequently omitted from story traversal even though it is admitted by
the existing typed tree. Candidate admission must retain the namespace scope
at the candidate's original run, not only the scope at the paragraph root.

### D2, an invalid enclosing field does not invalidate an admitted inner candidate
`crates/rdocx/src/document.rs:4095`
`crates/rdocx/src/document.rs:4110`
`crates/rdocx/src/document.rs:4366`
`crates/rdocx-oxml/src/text.rs:2554`

The scanner pushes every locally valid inner field before it knows whether
the enclosing field is valid. Admission then removes the enclosing markers
and reparses the inner candidate alone. For example, an outer field with an
empty instruction followed by a separator and a valid inner field in its
cached-result side produces one admitted inner story field. The complete typed
paragraph parser attaches an inner field only on the instruction side of a
valid parent. It discards this cached-result child while the invalid outer
field is open, then rejects the outer field. The original typed tree therefore
contains no field, but story traversal exposes and can mutate the isolated
inner bytes. Per-candidate admission must retain the enclosing field validity
context while still distinguishing the exact marker-bounded candidate.

## Smells

None. Count: 0.

## Nitpicks

None. Count: 0.

## Not found

- **Pass 16 direct trigger**: the exact same-run field nested in a valid outer
  cached result is now exposed independently. Its projection, mutation, raw
  neighbours, and stale outer location remain isolated in the focused test.
- **Earlier typed boundaries outside D1 and D2**: rejected sibling fields,
  opaque lookalikes, controls, revisions, simple fields, and misplaced known
  children remain excluded without suppressing valid direct siblings.
- **OOXML outside D1**: no additional fixed-prefix read, namespace collision,
  schema-order, whitespace, note-filtering, or unrelated-byte rewrite defect
  was found.
- **Identity, atomicity, and lifecycle**: no additional owner-order,
  fingerprint, bounds, kind, staged-commit, provenance, optional-part, dirty
  state, or signature-state defect was found.
- **Panics**: no added production panic, unchecked index, invalid slice, or
  non-wrapping arithmetic on story input was found.
- **Tests**: the pass 16 focused regression passed. It does not cover a
  wrapper-local Word alias or a valid inner field whose enclosing field is
  rejected.
- **Structure**: no new trait, generic parameter, feature flag, crate, module,
  forwarding wrapper, or unnecessary dynamic dispatch was introduced.
