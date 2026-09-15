# F-253, all, pass 9

**Reviewed**: complete uncommitted working-tree diff on
`work/f-253-codex`, 5 files and 2,886 changed lines, comprising 2,840
insertions and 46 deletions
**Verdict**: 2 defects, 0 smells, 0 nitpicks

## Defects

### D1, contextual content-control grammar hides valid table-cell stories
`crates/rdocx/src/document.rs:3363`
`crates/rdocx-oxml/src/content_control.rs:327`

The pass 8 grammar uses one context-free child list for every `w:sdtContent`.
That list admits paragraph, table, and run content, but omits the `w:tr` owned
by a table-level content control and the `w:tc` owned by a row-level content
control. The typed model explicitly accepts those two owner-specific shapes.
A valid table whose row or cell is wrapped in a content control is therefore
made opaque at `w:tr` or `w:tc`. Traversal omits the nested table-cell story
and cannot project the control's visible text. The story grammar must carry
the content-control owner context rather than applying one flat
`w:sdtContent` allowlist.

### D2, nested complex fields corrupt cached-result projection and mutation
`crates/rdocx/src/document.rs:3865`
`crates/rdocx/src/document.rs:4095`
`crates/rdocx/tests/regression_test.rs:31`

Complex field discovery uses a stack, but both text paths reduce result state
to one boolean. An inner field's separator turns that boolean on even when the
outer field is still in its instruction, and the inner end turns it off even
when the outer field remains in its cached result. The outer field can
therefore project nested instruction text as result text or omit result text
that follows a nested field. Mutation applies the same wrong boundary and
leaves part of the old cached result unchanged. The regression covers only a
flat field, so it cannot detect either nested sequence.

## Smells

None. Count: 0.

## Nitpicks

None. Count: 0.

## Not found

- **Pass 8 flat complex field**: one balanced nonnested marker sequence is one
  field item, and its instruction plus neighbouring paragraph text survive a
  cached-result edit.
- **Pass 8 invalid parent-child boundaries**: misplaced known Word children
  and unknown DrawingML or VML wrappers remain opaque. D1 is the remaining
  owner-context gap for valid content controls.
- **Pass 8 relationship order**: note and comment story parts follow the
  package relationship list, with owner order retained inside each part.
- **Earlier remediation**: nested story-owner exclusion, entity and CDATA
  projection, whitespace preservation, complete owner fingerprints, note
  filtering, empty text nodes, namespace fail-closed behavior, identifier
  provenance, and optional-part ownership provenance remain intact.
- **Panics**: no production `unwrap`, `expect`, unchecked indexing, slicing,
  or non-wrapping arithmetic was added on untrusted story input.
- **Structure**: no new trait, generic parameter, feature flag, crate, module,
  forwarding wrapper, or dynamically dispatched concrete dependency was
  introduced.
