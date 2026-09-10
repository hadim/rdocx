# F-253, all, pass 6

**Reviewed**: complete uncommitted working-tree diff on
`work/f-253-codex`, 5 files and 2,024 changed lines, comprising 2,011
insertions and 13 deletions
**Verdict**: 2 defects, 0 smells, 0 nitpicks

## Defects

### D1, body story mutation bypasses the namespace-shadow fail-closed gate
`crates/rdocx/src/document.rs:7444`
`crates/rdocx/src/document.rs:7670`

Body traversal is built from `CT_Document::to_xml` rather than the prepared
package part. After `flush_to_package` accepts an unchanged document with an
unsafe retained fixed-prefix shadow, `set_story_text` patches that canonical
projection and installs it directly into the package. It does not run the
modified XML through the existing nested-namespace replay and unsafe-shadow
check. A document such as the existing `wp` collision regression, with a
modeled paragraph that owns `xmlns:wp="urn:foreign"` and retained
`<wp:producer/>`, can therefore accept a story text edit that changes or loses
the retained node's namespace meaning. An ordinary typed mutation fails closed
for this input. Story mutation must either patch the exact prepared package
bytes or run the changed body through the same namespace safety path before it
can commit.

### D2, lexical scans promote content inside preserved subtrees into editable content
`crates/rdocx/src/document.rs:3341`
`crates/rdocx/src/document.rs:3461`

Owner discovery recognizes every Word-namespaced `body`, `tc`, `txbxContent`,
and related owner name at any XML depth. Item discovery similarly recognizes
controls, fields, and drawings at any depth outside a cell or text-box owner.
Neither scan tracks whether it has entered a direct child already classified
as `PreservedNode`. An `mc:AlternateContent` or other retained subtree that
contains a Word table, text box, field, drawing, or control is therefore
reported both as one preserved item and as separate editable stories or items.
`set_story_text` can then rewrite bytes inside content that the typed tree owns
only as an opaque preserved subtree. This violates the preservation boundary
and the contract to traverse the existing typed tree. Discovery must stop
descending for modeled story purposes once it enters preserved content, while
still retaining that complete subtree as one preserved item.

## Smells

None. Count: 0.

## Nitpicks

None. Count: 0.

## Not found

- **Pass 5 facade ownership**: prepared staged reopen now restores custom
  properties, settings, comments, and commentsExtended ownership. Owned
  settings also retain their pre-reopen typed model, so final authored setting
  removal can prune the generated part without preserving dirty state.
- **Pass 4 identifier provenance**: identifier and signature provenance remain
  reconciled through the prepared staged reopen path.
- **Correctness and contract outside D1 and D2**: owner ordering, location
  resolution, item-kind checks, visible-text replacement, and atomic rejection
  match the approved contract.
- **Panics**: no production `unwrap`, `expect`, unchecked indexing, slicing, or
  non-wrapping arithmetic was added on untrusted story input.
- **OOXML outside D1 and D2**: note separator normalization, nested owner
  exclusion, entity projection, CDATA handling, boundary-space handling, and
  unrelated first-text-tag bytes are handled correctly.
- **Tests**: the named integration, round-trip, and atomicity gates cover every
  named story and the pass 1 through pass 5 remediations. They do not exercise
  unsafe nested namespace shadows or Word content nested inside a preserved
  wrapper, which are the triggers above.
- **Structure**: no new trait, generic parameter, feature flag, crate, module,
  forwarding wrapper, or dynamically dispatched concrete dependency was
  introduced.
