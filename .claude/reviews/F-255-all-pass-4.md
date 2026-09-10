# F-255, all, pass 4

**Reviewed**: exact current uncommitted implementation diff against claim base
`53c81086c95be671ea2f75bd14c31cab893409c0` on `work/f-255-codex`, excluding
review artifacts, 3 files, 1982 insertions and 111 deletions
**Verdict**: 2 defects, 0 smells, 0 nitpicks

## Defects

### D1, authored nested content cannot be removed or cloned
`crates/rdocx/src/document.rs:10864`

The two F-255 insertion methods permanently register each main-part
relationship, and picture insertion also permanently registers its drawing id.
The generic removal path deletes the selected XML and proceeds directly to
prepare and reopen without unregistering either identity
(`crates/rdocx/src/document.rs:11140`). Canonicalization retains the relationship
as an unreferenced authored relationship, then rejects its zero serialized
occurrences because every registered relationship must occur exactly once
(`crates/rdocx/src/document.rs:8992`). Drawing remapping likewise requires every
registered drawing to remain present (`crates/rdocx/src/document.rs:3488`). The
generic clone path copies the relationship reference, freshens only identities
that are defined as clone-local, and then prepares without registering a second
occurrence (`crates/rdocx/src/document.rs:11176`). It therefore makes the
registered relationship occur twice and trips the same exactly-once check. This
regresses the explicit same-owner clone contract, which permits two references
to one relationship (`crates/rdocx/tests/regression_test.rs:1560`). The new
owner-structure matrix mutates only an enclosing item before the authored
content, so it does not exercise the lifecycle of that content
(`crates/rdocx/tests/regression_test.rs:911`). Add removal and same-owner clone
sensitivities for the paragraph returned by each F-255 picture and hyperlink
insertion in body, text-box, and table-cell stories. Removal must save and
reopen atomically. Hyperlink clone must preserve two references to the shared
relationship. Picture clone must preserve its relationship semantics while
allocating a fresh drawing identity.

### D2, the reorder test does not prove the body XML target associations
`crates/rdocx/tests/regression_test.rs:1007`

The pass-3 semantic test extracts the actual text-box `r:embed` and `r:id`
attributes and resolves those exact references. For the body half, it instead
enumerates the document relationship table and selects whichever image and
hyperlink ids differ from the text-box ids. It then resolves those selected
table entries without showing that either id appears in the authored body
paragraph (`crates/rdocx/tests/regression_test.rs:1008`). A simultaneous-remap
bug that left the body XML pointing at the text-box relationships would still
pass as long as the package retained separate body relationships with the
expected targets. The operation-order byte comparison does not establish this
missing association either (`crates/rdocx/tests/regression_test.rs:846`). Extract
the relationship attributes from the specific authored body picture and
hyperlink paragraphs after reorder, then resolve those exact ids to the
expected body image bytes and URL.

## Smells

None.

## Nitpicks

None.

## Not found

Pass-3 D1 is closed. Standalone footnote and endnote roots retain the standard
relationship namespace, while `Document` serialization removes that fixed
declaration only when replaying a producer-shadowed root declaration. Direct
serializer tests prove inherited raw relationship attributes remain bound.

Pass-3 D2 is closed for the requested enclosing-owner edits. Provenance is now
part-local, registration occurs only after F-255 insertion, and the eight
text-box and table-cell cases cover insertion, removal, clone, and move of a
preceding enclosing body item. The distinct lifecycle defect in D1 concerns
removing or cloning the authored relationship-bearing item itself.

Pass-3 D3 is closed in the implementation. Relationship and drawing remaps use
namespace-aware XML scans, retain the original registered-id sets during
simultaneous rewrites, reject missing or duplicate registered occurrences, and
update provenance only after rewriting. Modeled and registered nested drawing
slots remain disjoint. Counted modeled occurrences preserve legacy duplicate
provisional typed ids. The remaining semantic target issue is the test-gate
defect in D2, not another confirmed remap defect.

Both pass-1 defects remain closed. Related footnote insertion survives later
typed mutation, and header drawing allocation is independent of replacement
history. Both pass-2 defects remain closed. Producer `r` and `wp` namespace
meanings survive retained raw content, and body plus text-box authoring order
produces a deterministic package.

No additional correctness, contract, panic, OOXML, test, or structure defects
were found in the original F-255 story APIs, owner-scope validation, stale story
rejection, related-story typed caches, relationship target and mode checks,
part-local media ownership, custom part targets, external image rejection,
internal hyperlink rejection, schema child order, raw XML preservation,
namespace aliases, character references, foreign same-local-name attributes,
producer-owned raw references, canonicalization, serialization and reopen
atomicity, or complete-package determinism outside D1. No new trait, generic,
module, file, forwarding wrapper, unjustified panic, or other structural smell
was introduced. The public Rust additions remain additive for the pre-1.0
crate.

Focused regression tests passed for enclosing-owner structural edits, semantic
reorder, body and text-box operation order, footnote producer shadows,
part-local equal related content, wrong-scope atomic rejection, related
footnote typed mutation, and header drawing replacement history. Both focused
`rdocx-oxml` inherited raw relationship-attribute tests passed. The exact
implementation diff passes `git diff --check`.
