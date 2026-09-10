# F-254, all, pass 1

**Reviewed**: exact uncommitted working-tree diff on `work/f-254-codex`, 5 files,
1,186 insertions and 13 deletions
**Verdict**: 6 defects, 0 smells, 0 nitpicks

## Defects

### D1, public item locations do not identify the same destination boundary
`crates/rdocx/src/document.rs:4307`

The destination resolver indexes `direct_story_content_items`, but F-253 item
locations index the flattened `scan_story_items` result. That flattened result
also contains nested controls, fields, and drawings. A caller that passes the
actual location of a later direct child after one nested projection therefore
selects a different direct child or receives `OutOfBounds`. This violates the
plan's requirement that the methods consume F-253 locations. The new tests use
synthetic direct-child indices through `f254_boundary` and never pass such an
actual later-child location as a destination
(`crates/rdocx/tests/regression_test.rs:45`).

### D2, insertion cannot reach an empty package-backed owner
`crates/rdocx/src/document.rs:4125`

`story_owner_content_end` finds only an `End` event at depth one. A self-closing
package-backed owner such as an empty header, footer, comment, footnote,
endnote, or text box produces an `Empty` event and reaches EOF with an error.
The plan explicitly requires the zero boundary to work for an empty owner. The
package-backed regression covers only a dirty footnote that already contains a
paragraph (`crates/rdocx/tests/regression_test.rs:341`), so it does not exercise
the failing shape.

### D3, a valid same-cell move of the only paragraph is rejected
`crates/rdocx/src/document.rs:9994`

Every nonadjacent move of a cell's sole paragraph is rejected as though the
paragraph were leaving the cell. `move_content` only permits the same story
owner, so the paragraph is reinserted into that same cell and the required
paragraph invariant remains satisfied. A cell containing raw XML, a table, and
one paragraph therefore cannot reorder that paragraph even though the result
is valid. The move regression exercises body content only
(`crates/rdocx/tests/regression_test.rs:115`), while the cell assertion covers
removal rather than an in-owner move (`crates/rdocx/tests/regression_test.rs:243`).

### D4, identity freshening changes the meaning of a shadowed `w` prefix
`crates/rdocx/src/document.rs:4400`

The synthetic wrapper always binds `w` to WordprocessingML, then deliberately
skips the fragment's inherited `w` binding. If a valid owner uses another
prefix for WordprocessingML and shadows `w` to a producer namespace, foreign
`w:bookmarkStart`, `w:id`, or similar markup is scanned and rewritten as Word
identity markup. It can also be rejected as an ambiguous Word bookmark even
though it is foreign XML. The namespace regression covers only an inherited
`x` prefix (`crates/rdocx/tests/regression_test.rs:491`) and does not prove the
required shadowing behavior.

### D5, cloning fails when fresh identity text is shorter than source text
`crates/rdocx/src/document.rs:4421`

The fragment extractor computes the new slice by requiring the complete
rewritten wrapper to be at least as long as the original wrapper. Fresh numeric
identities are commonly shorter than producer values, for example remapping
`999999` to `1`. That legitimate clone returns `content identity rewrite
shortened its wrapper` instead of inserting the fragment. The identity
regression uses only short source values (`crates/rdocx/tests/regression_test.rs:393`)
and cannot expose this case.

### D6, bookmark counts do not prove balanced fragment ownership
`crates/rdocx/src/field.rs:6789`

The preflight compares one start and one end per id, but it does not verify
ordering or balanced nesting. A fragment with `bookmarkEnd` before its matching
`bookmarkStart` passes, receives a fresh id, and is cloned as an invalid range.
The plan requires cloning to fail when ownership cannot be proven. The added
unit cases cover unequal duplicate counts only
(`crates/rdocx/src/field.rs:12883`), and the regression covers isolated starts
or ends (`crates/rdocx/tests/regression_test.rs:471`).

## Smells

None.

## Nitpicks

None.

## Not found

No additional defects were found in stale owner-fingerprint validation,
same-owner forward and backward byte-offset adjustment, self and adjacent move
no-ops, body end insertion before section properties, cell raw-boundary replay,
relationship owner and dangling-reference rejection, dirty-footnote staging,
atomic staged reopen and publication, or additive public API compatibility.
No panic or structural-rule findings were found. `StoryError` was not extended,
so existing downstream exhaustive matches retain their current source shape.
