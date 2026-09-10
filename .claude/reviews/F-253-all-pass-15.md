# F-253, all, pass 15

**Reviewed**: complete uncommitted working-tree diff on
`work/f-253-codex`, 8 files and 4,883 changed lines, comprising 4,822
insertions and 61 deletions
**Verdict**: 1 defect, 0 smells, 0 nitpicks

## Defects

### D1, one rejected field suppresses valid typed siblings in the paragraph
`crates/rdocx/src/document.rs:4502`
`crates/rdocx/src/document.rs:4520`
`crates/rdocx/src/document.rs:4551`
`crates/rdocx-oxml/src/text.rs:2580`

Admission is still paragraph-wide rather than position-preserving. The facade
scanner records every locally balanced field, while the typed parser rejects a
field that crosses a bookmark, comment, content-control, revision, or partial
hyperlink boundary. If one such scanner-only field shares a paragraph with one
valid typed complex field, there are two candidates but one typed source. The
count mismatch skips the whole paragraph, then the retain step removes both
candidates, including the valid field that exists in the typed tree. Admission
must associate each scanner span with the typed result at that source position
and reject only the unmatched span.

## Smells

None. Count: 0.

## Nitpicks

None. Count: 0.

## Not found

- **Pass 14 D1 cross-admission variant**: byte membership no longer admits an
  opaque simple-field lookalike in the focused regression. D1 is the remaining
  mixed valid and boundary-rejected sibling case caused by paragraph-wide
  cardinality.
- **Pass 14 D2**: the complex-field projection selects an existing Word prefix
  or a collision-free generated prefix, while retaining a foreign inherited
  `w` binding and the rest of the in-scope declarations.
- **Pass 14 D3**: marker-bounded `full` ranges isolate text, XML projection, and
  mutation for same-run sibling and nested fields. Typed grouping retains each
  same-run sibling and remaps following boundaries by the replacement count.
- **Correctness and contract outside D1**: owner ordering, checked location
  resolution, fingerprint staleness, and staged atomic mutation remain aligned
  with the approved contract.
- **Panics**: no added production panic, unchecked index, invalid slice, or
  non-wrapping arithmetic on story input was found.
- **OOXML outside D1**: no additional fixed-prefix read, namespace collision,
  schema-order, whitespace, note-filtering, or retained-subtree defect was
  found.
- **Tests**: the focused pass 14 cases exercise opaque identical bytes, a
  foreign inherited `w` binding, and same-run sibling and nested isolation.
  They do not combine a valid typed field with a boundary-rejected scanner
  sibling in one paragraph.
- **Structure**: no new trait, generic parameter, feature flag, crate, module,
  forwarding wrapper, or unnecessary dynamic dispatch was introduced.
