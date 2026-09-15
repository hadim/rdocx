# F-253, all, pass 11

**Reviewed**: complete uncommitted working-tree diff on
`work/f-253-codex`, 5 files and 3,197 changed lines, comprising 3,151
insertions and 46 deletions
**Verdict**: 2 defects, 0 smells, 0 nitpicks

## Defects

### D1, name-only grammar crosses wrappers that the typed parser retained as raw
`crates/rdocx/src/document.rs:3321`
`crates/rdocx/src/document.rs:3795`
`crates/rdocx-oxml/src/revision.rs:78`
`crates/rdocx-oxml/src/text.rs:1367`

The opacity decision uses only the current element's namespace and local name.
It does not reproduce the typed parser's acceptance decision for that current
child. A content revision without the required `w:author`, or a simple field
without a nonempty `w:instr`, is retained only as raw paragraph XML by the
typed parser. The story scanner still treats either wrapper as modeled,
projects its descendant text, exposes the simple field as a `Field` item, and
permits mutation inside the raw subtree. Self-closing forms have the same
problem because the typed paragraph and run parsers preserve an empty
`w:fldSimple` or `w:drawing` as raw while the story scanner classifies it by
name. The new regression at
`crates/rdocx/tests/regression_test.rs:36` even omits `w:author` from both
revision wrappers and therefore locks in traversal through raw XML rather than
proving the intended typed revision grammar.

### D2, complex-field discovery accepts sequences rejected by the typed field model
`crates/rdocx/src/document.rs:3809`
`crates/rdocx-oxml/src/text.rs:2540`

The story scanner emits a `Field` for every balanced begin, separate, and end
marker sequence. The typed paragraph projection additionally requires a
nonempty parsed instruction, exactly one separator, and modeled source runs.
For example, a balanced sequence with an empty instruction and cached-result
text remains ordinary runs plus retained markers in the typed paragraph, but
story traversal reports a field and `set_story_text` rewrites that result.
A second separator is another accepted-by-story, rejected-by-typed trigger.
The field item boundary must use the same validity conditions as the existing
field projection.

## Smells

None. Count: 0.

## Nitpicks

None. Count: 0.

## Not found

- **Pass 10 direct child split**: revisions admit direct runs, hyperlinks, and
  nested content revisions. Hyperlinks admit direct runs and content
  revisions. Simple fields admit direct runs. The name-level split no longer
  gives these three containers one shared child grammar.
- **Content-control contexts**: block, table, row, and inline content controls
  retain their distinct child shapes, including row and cell owners.
- **Nested complex-field result state**: projection and mutation retain one
  result-state entry per open field. Inner separators and ends do not change
  the outer result boundary.
- **Owner and location behavior**: relationship ordering, owner fingerprints,
  bounds and kind checks, stale rejection, and staged atomic commit behavior
  remain consistent with the approved contract.
- **Panics**: no production `unwrap`, `expect`, unchecked indexing, slicing,
  or non-wrapping arithmetic was added on untrusted story input.
- **OOXML outside the findings**: no additional fixed-prefix read, namespace,
  schema-order, whitespace, note-filtering, or retained-subtree defect was
  found.
- **Structure**: no new trait, generic parameter, feature flag, crate, module,
  forwarding wrapper, or dynamically dispatched concrete dependency was
  introduced.
