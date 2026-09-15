# F-254, all, pass 2

**Reviewed**: exact revised uncommitted working-tree diff on
`work/f-254-codex`, 5 files, 1,463 insertions and 13 deletions
**Verdict**: 2 defects, 0 smells, 0 nitpicks

## Defects

### D1, synthetic direct indexes still compete with canonical F-253 anchors
`crates/rdocx/src/document.rs:4335`

Actual F-253 item locations are the canonical insertion anchors, and a nested
projection must be rejected. The fallback nevertheless reinterprets the same
index as a synthetic direct-child index when the projected kind does not match.
Whether one `ContentLocation` means an actual flattened anchor or a synthetic
direct boundary therefore depends on incidental item kinds. For example, a
nested `Field` at flattened index one followed by a direct `Paragraph` at
direct index one makes `(Paragraph, [1])` act as the synthetic paragraph
boundary, while replacing both with content controls makes the same shape hit
the nested projection and fail. The tests rely throughout on the synthetic
`f254_boundary` helper (`crates/rdocx/tests/regression_test.rs:45`), including
the new flattened-location test (`crates/rdocx/tests/regression_test.rs:514`).
The implementation needs one canonical-anchor interpretation and an explicit,
unambiguous end or empty-owner location using the existing public type. The
special body end must continue to resolve before `sectPr`.

### D2, the public block content-control constructor accepts non-block controls
`crates/rdocx/src/document.rs:342`

`ContentFragment::content_control` documents a block content control but accepts
any `CT_Sdt` and serializes it without validating its owner grammar. The public
standalone parser admits paragraph, table, row, cell, and run children
(`crates/rdocx-oxml/src/content_control.rs:322`), while every story owner used
by this API admits only paragraph and table children for block controls. A
standalone control containing a row, cell, or run can therefore be wrapped as a
`ContentFragment` and inserted into a body or cell. Reopen preserves the
inadmissible child as raw XML instead of making the output schema-valid. No new
test calls the `content_control` constructor, so the public constructor's
claimed block contract is unproved (`crates/rdocx/tests/regression_test.rs:49`).

## Smells

None.

## Nitpicks

None.

## Not found

Pass-1 defects D2 through D6 are closed. Self-closing package-backed owners are
expanded at the checked boundary while retaining their qualified root name.
Same-cell movement no longer rejects the sole paragraph. Identity rewriting
uses a collision-free wrapper prefix and structurally rescans the rewritten
fragment. Shorter replacements are accepted. Bookmark ownership now rejects
reversed, crossing, duplicate, and incomplete ranges while accepting properly
nested and sibling ranges.

No additional defects were found in direct-child source validation, section
properties placement, same-owner move arithmetic and byte offsets, cell raw
boundary replay, namespace closure, relationship ownership and dangling
references, stale fingerprints, dirty-footnote persistence, staged atomicity,
panic safety, additive public API compatibility, tests outside the two gaps
above, or repository structural rules. `StoryError` remains unchanged, so the
feature does not break downstream exhaustive matches by adding a variant.
