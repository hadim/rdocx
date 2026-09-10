# F-253, all, pass 1

**Reviewed**: complete uncommitted working-tree diff on
`work/f-253-codex`, 5 files and 1,152 changed lines, comprising 1,146
insertions and 6 deletions
**Verdict**: 8 defects, 0 smells, 0 nitpicks

## Defects

### D1, text mutation crosses nested story ownership boundaries
`crates/rdocx/src/document.rs:3557`
`crates/rdocx/src/document.rs:3627`

The item scanner deliberately excludes content under nested `w:tc` and
`w:txbxContent` owners, but both text extraction and replacement collect every
descendant `w:t` without the same owner check. Setting text on a body-level
table therefore rewrites text owned by its table-cell stories, and setting text
on a body paragraph that contains a text box rewrites the text-box story. A
mutation addressed to one story must not silently mutate another owner.

### D2, text projection returns XML entity spellings instead of visible text
`crates/rdocx/src/document.rs:3565`

`BytesText::decode` decodes bytes but does not unescape XML entities. A run
containing `A &amp; B` is therefore projected as `A &amp; B`, not `A & B`.
Passing that projected value back to `set_story_text` escapes it again and
changes the visible text to `A &amp; B`. The public text view must unescape text
events while leaving CDATA content literal.

### D3, replacement loses significant leading and trailing spaces
`crates/rdocx/src/document.rs:3666`
`crates/rdocx/src/document.rs:3681`

The replacement changes only text bytes, and the empty-paragraph path creates
`w:t` without `xml:space="preserve"`. Values beginning or ending with spaces
therefore serialize without the OOXML whitespace marker and Word may collapse
those spaces. The mutation must update the first retained `w:t` attribute, or
author a preserving attribute on a new one, according to the replacement.

### D4, operation-scoped locations remain valid after text and attribute mutations
`crates/rdocx/src/document.rs:3359`
`crates/rdocx/src/document.rs:7408`

Staleness is decided by a fingerprint that hashes only element namespace,
local name, and start, empty, or end shape. It ignores text and every
attribute. Reusing a location after `set_story_text`, or after a nonstructural
format mutation, therefore succeeds even though the approved contract says
locations are operation-scoped and re-resolved after mutation. The regression
only adds a paragraph, so it does not expose this gap.

### D5, separator records are exposed as editable note stories
`crates/rdocx/src/document.rs:3275`
`crates/rdocx/src/document.rs:3306`

Every `w:footnote` and `w:endnote` element becomes a story without inspecting
`w:type`. Real packages also contain separator, continuation-separator, and
continuation-notice records. Those are retained infrastructure rather than
normal note owners, and the existing typed note facade filters them. This API
currently enumerates and permits mutation of those separator records as user
stories.

### D6, the integration gate does not test identical error behavior across stories
`crates/rdocx/tests/integration_test.rs:106`
`crates/rdocx/tests/regression_test.rs:30`

The approved integration gate requires the same mutation and identical errors
for every supported story. The integration loop exercises only successful
paragraph edits. All wrong-owner, kind, bounds, and stale assertions are made
against the body alone in the regression test. Story-specific resolver bugs in
headers, footers, notes, comments, cells, or text boxes can therefore leave the
named gate green.

### D7, the round-trip gate does not prove item order or retained nodes in every story
`crates/rdocx/tests/integration_test.rs:151`
`crates/rdocx/tests/integration_test.rs:210`

The initial category assertion collects into a `HashSet`, which discards the
order the test is meant to prove. The only byte-exact raw-node assertion checks
one foreign child in the main document part. No related story contains a
retained node. Comparing the implementation's pre-save traversal to its own
post-save traversal proves repeatability, but cannot detect a consistently
wrong owner or item order. This does not satisfy the approved all-categories,
all-retained-nodes round-trip contract.

### D8, the borrowed traversal clones every source and every item
`crates/rdocx/src/document.rs:7280`
`crates/rdocx/src/document.rs:7426`

The approved approach requires borrowed `StoryItemRef` values over the
existing typed tree without cloning it into a second representation. Related
parts are copied into `StorySource`, then each returned item's XML and text are
copied again into owned `Cow` values. The lifetime on `StoryItemRef` does not
represent an actual borrow from the document. This violates the explicit
container-model contract and makes traversal allocate proportional to both the
whole story graph and the sum of all item subtrees.

## Smells

None. Count: 0.

## Nitpicks

None. Count: 0.

## Not found

- **Panics**: no production `unwrap`, `expect`, unchecked index, or
  non-wrapping arithmetic was added on untrusted input.
- **OOXML child order and prefixes**: apart from D3, no fixed-prefix read
  assumption or schema-order violation was found in the added scanner and
  patch path.
- **Structure**: no new trait, generic parameter, feature flag, crate, module,
  forwarding wrapper, or dynamically dispatched concrete dependency was
  introduced. D8 is the contract-specific representation issue.
- **Focused execution**: all three named feature tests pass, which does not
  close the coverage defects above.
