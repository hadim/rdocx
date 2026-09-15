# F-253, all, pass 10

**Reviewed**: complete uncommitted working-tree diff on
`work/f-253-codex`, 5 files and 3,118 changed lines, comprising 3,072
insertions and 46 deletions
**Verdict**: 1 defect, 0 smells, 0 nitpicks

## Defects

### D1, inline container grammar still crosses typed preservation boundaries
`crates/rdocx/src/document.rs:3374`
`crates/rdocx-oxml/src/revision.rs:535`
`crates/rdocx-oxml/src/text.rs:1391`
`crates/rdocx-oxml/src/text.rs:5962`

The contextual grammar gives hyperlinks and every content revision the same
three allowed children, and also treats content controls and hyperlinks below
`w:fldSimple` as modeled. The typed owners have different grammars. A content
revision models runs, hyperlinks, and nested revisions. A hyperlink models runs
and content revisions, while a simple field models only direct runs. The story
scan consequently makes a valid hyperlink or nested revision inside `w:ins`
opaque and omits its visible text. In the other direction, it projects and
rewrites a content control or simple field inside a hyperlink, or a content
control or hyperlink inside a simple field, even though the typed owner retains
that child only as raw XML. The parent-child rules must distinguish revisions,
hyperlinks, and simple fields so valid typed content stays reachable and raw
children stay opaque.

## Smells

None. Count: 0.

## Nitpicks

None. Count: 0.

## Not found

- **Pass 9 content-control contexts**: block, table, row, and inline
  `w:sdtContent` owners retain their distinct child grammars, including nested
  row and cell stories.
- **Pass 9 nested complex fields**: item discovery, projection, and mutation
  keep one result-state entry per open field, so nested separators and ends do
  not change the outer field state.
- **Correctness and contract outside D1**: owner ordering, stable identity,
  checked location resolution, and staged atomic mutation match the approved
  contract.
- **Panics**: no production `unwrap`, `expect`, unchecked indexing, slicing,
  or non-wrapping arithmetic was added on untrusted story input.
- **OOXML outside D1**: no additional namespace, schema-order, whitespace,
  note filtering, or retained-subtree defect was found.
- **Tests**: the named integration, round-trip, and atomicity gates cover every
  named story and the pass 1 through pass 9 remediations. They do not cover
  typed hyperlinks or nested revisions inside revision content, or raw child
  containers inside hyperlinks and simple fields, which trigger D1.
- **Structure**: no new trait, generic parameter, feature flag, crate, module,
  forwarding wrapper, or dynamically dispatched concrete dependency was
  introduced.
