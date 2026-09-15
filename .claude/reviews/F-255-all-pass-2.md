# F-255, all, pass 2

**Reviewed**: exact current uncommitted implementation diff against claim base
`53c81086c95be671ea2f75bd14c31cab893409c0` on `work/f-255-codex`, excluding
review artifacts, 4 files, 928 insertions and 55 deletions
**Verdict**: 2 defects, 0 smells, 0 nitpicks

## Defects

### D1, fixed comment and note prefixes rebind preserved producer XML
`crates/rdocx-oxml/src/comments.rs:122`

The comments serializer now always binds `r` and `wp` to the OOXML
relationship and word-processing drawing namespaces. It then drops preserved
root declarations with either name at
`crates/rdocx-oxml/src/comments.rs:350`. A valid producer comments part can
bind `r` or `wp` to a foreign namespace and use that prefix in a retained raw
root child, comment child, or attribute. Parsing retains the raw bytes and root
declaration, but serialization removes the producer binding and silently
changes every retained qualified name to the newly imposed OOXML namespace.
The analogous new fixed `xmlns:wp` declaration on typed footnotes and endnotes
has the same effect on paragraph raw XML that inherited a producer `wp`
binding from the original note root (`crates/rdocx-oxml/src/footnotes.rs:193`).
That serializer does not retain the root declaration, while the paragraph can
retain the prefixed raw child bytes, so a later typed footnote flush rebinds
them to word-processing drawing.
This violates the required exact subtree and namespace-meaning preservation.
Choose collision-free fixed prefixes, or keep each producer binding and put
the required OOXML declarations on the authored content without shadowing it.
Add round-trip tests for both `r` and `wp` comment-root producer shadows with
retained raw elements and attributes, plus a note-root `wp` shadow inherited
by paragraph raw XML before a typed footnote mutation. The existing comments
alias test uses only an unshadowed `ext` prefix
(`crates/rdocx-oxml/src/comments.rs:434`).

### D2, main-part text-box assets remain history-dependent
`crates/rdocx/src/document.rs:8638`

Main-story canonical ordering collects relationship ids only from the typed
body tree. A text box in retained main-document XML shares the document
relationship scope, but its newly authored image and hyperlink references are
not collected. The owner helper also deliberately does not register authored
story relationships when that owner is the main part
(`crates/rdocx/src/document.rs:9852`). Those references are consequently
classified as opaque and excluded from relationship remapping, while image
canonicalization consumes the same incomplete typed-only list at
`crates/rdocx/src/document.rs:8776`. Adding equal body and text-box content in
the opposite operation order therefore leaves opposite local `rId` and media
part assignments even though their final document positions are identical.
Track authored nested-story references independently of physical owner, then
canonicalize them in serialized source order while leaving producer-owned raw
references untouched. Add a sensitivity test that builds the same body and
text-box picture plus hyperlink content in both operation orders and compares
complete package bytes, relationship ids, targets, and media bytes. The current
round-trip uses only body-first order (`crates/rdocx/tests/regression_test.rs:188`).

## Smells

None.

## Nitpicks

None.

## Not found

Both pass-1 defects are closed. Picture and hyperlink insertion now mirror the
new paragraph into the typed ordinary-footnote cache before publishing the
prepared candidate. A later typed footnote mutation retains both relationship
forms after save and reopen. Comment synchronization still parses the updated
part into its typed cache, and endnotes have no competing mutable typed cache.
Body publication keeps the typed document authoritative, while table cells and
text boxes update that same containing document part. Headers, footers, and
endnotes remain package-backed without a stale mutable facade owner.

Drawing canonicalization now starts from preserved ids, retains live ids in
content it will not rewrite, removes live ids from the typed body it will
rewrite, and rewrites fully authored related stories against the shared global
set. Retired header drawing ids no longer remain occupied, producer and raw ids
remain reserved, and replacement history produces byte-identical output. No
additional defect was found in malformed drawing-id handling reachable from a
valid opened document, global uniqueness, header replacement, relationship or
media pruning, related-story media ordering outside D2, custom part targets,
part-local duplicate ids, owner validation, target type or mode checks,
external image and internal hyperlink rejection, missing targets, schema child
order, staged failure atomicity, or the validated-package publication path.

The public Rust additions remain additive for the pre-1.0 crate. No new trait,
generic parameter, module, wrapper, or unjustified panic was introduced. The
five focused F-255 and pass-1 regression tests passed individually. All three
comments serializer unit tests passed. The current implementation diff passes
`git diff --check`.
