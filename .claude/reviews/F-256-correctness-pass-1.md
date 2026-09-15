# F-256, correctness, pass 1

**Reviewed**: working-tree diff, 7 files, 810 inserted lines and 40 deleted lines
**Verdict**: 4 defects, 0 smells, 0 nitpicks

## Defects

### D1, Selected ranges import unrelated style and numbering definitions

`crates/rdocx/src/field.rs:1750`

The fragment package retains the source document's complete styles and
numbering models, and `insert_document_fragment_content_staged` merges both
models wholesale. A range that references one custom style or numbering
instance therefore imports every unrelated definition from the source. This
violates the selected dependency-closure contract and makes conflict-policy
results depend on content outside the fragment.

### D2, Import discards comment extension identity and thread state

`crates/rdocx/src/comments.rs:153`

Every imported comment has its paragraph ids replaced with `None`, and no
matching `commentsExtended` records are imported. A selected comment therefore
loses its resolved state and reply linkage. Repeated import must allocate fresh
paragraph ids and remap the selected extension closure instead of deleting it.

### D3, Package-authoritative selected XML is replaced by typed serialization

`crates/rdocx/src/field.rs:1649`

`DocumentFragment::from_range` captures exact package bytes, but import
immediately regenerates the body through `fragment.document.to_xml()` and later
copies typed `BodyContent`. Namespace scopes, lexical forms, and retained raw
gaps in the selected source are no longer proven exact. The implementation
needs source-span patching or an equivalent package-authoritative insertion
path, plus a byte-exact regression.

### D4, The story gate omits required dependency and failure classes

`crates/rdocx/tests/regression_test.rs:26619`

The dependency-rich fixture covers styles, numbering, pictures, charts,
bookmarks, and one basic comment. It does not exercise an embedded object,
field references, retained raw XML, dangling internal targets, malformed
relationship XML, or allocation exhaustion. The approved test plan names
these classes, so the current green gate cannot establish the story contract.

## Smells

None.

## Nitpicks

None.

## Not found

No additional correctness, contract, panic, OOXML ordering, namespace,
structure, or test-isolation findings were found.
