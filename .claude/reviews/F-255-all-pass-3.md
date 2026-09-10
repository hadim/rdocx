# F-255, all, pass 3

**Reviewed**: exact current uncommitted implementation diff against claim base
`53c81086c95be671ea2f75bd14c31cab893409c0` on `work/f-255-codex`, excluding
review artifacts, 3 files, 1765 insertions and 91 deletions
**Verdict**: 3 defects, 0 smells, 0 nitpicks

## Defects

### D1, standalone note serialization drops inherited relationship bindings
`crates/rdocx-oxml/src/footnotes.rs:187`

`CT_Footnotes::to_xml_footnotes` and `to_xml_endnotes` now put only `xmlns:w`
on the root. This removes the existing standard `xmlns:r` binding from a
public serializer. A caller can parse a valid note whose paragraph contains an
unmodelled child with an `r:id` inherited from the note root. `CT_P` retains
that child as raw XML at `crates/rdocx-oxml/src/text.rs:2810`, but the standalone
serializer writes it beneath a root where `r` is unbound. The private
`Document` replay at `crates/rdocx/src/document.rs:10141` repairs only facade
serialization and cannot protect direct `rdocx-oxml` callers. Keep the
standalone root namespace-complete without shadowing producer declarations,
and add direct footnote and endnote round-trip tests with an inherited raw
relationship attribute.

### D2, nested-story provenance becomes stale after owner-order mutations
`crates/rdocx/src/document.rs:2793`

Nested relationship and drawing provenance is keyed only by physical part,
story kind, and `owner_index`. That ordinal is recomputed from current source
order, while provenance reconciliation copies the old key unchanged at
`crates/rdocx/src/document.rs:2992`. A caller can author an asset in the second
text box, then insert or remove a body item containing a text box before it.
The generic mutation changes owner ordinals before canonicalization at
`crates/rdocx/src/document.rs:11225`. Relationship remapping then looks up the
old ordinal without checking the original fingerprint at
`crates/rdocx/src/document.rs:9075`. It either rejects a valid structural edit
as a lost owner or rewrites a different surviving text box. Track stable owner
identity across edits, or explicitly rebase provenance when owner order
changes. Add insert, remove, clone, and enclosing-paragraph move coverage for
multiple text boxes and table cells after story-scoped asset authoring.

### D3, cyclic nested remaps can suppress a modeled relationship rewrite
`crates/rdocx/src/document.rs:8862`

The nested XML remap runs first and replaces its registry values with the new
ids. The modeled remap is then filtered by testing old map keys against those
new nested values at `crates/rdocx/src/document.rs:8870`. In a cycle such as
modeled `rId5 -> rId6` and nested `rId6 -> rId5`, the new nested set contains
`rId5`, so the modeled rewrite is discarded. The package relationship formerly
named `rId5` has already moved to `rId6` at
`crates/rdocx/src/document.rs:8796`, leaving the modeled body reference pointed
at the nested relationship. This is reachable by authoring body and text-box
assets and then moving their enclosing body paragraphs across each other. The
current operation-order test only varies authoring order and does not reorder
already-authored owners (`crates/rdocx/tests/regression_test.rs:744`). Preserve
the original nested-id set for ownership filtering, and add a semantic test
that reorders already-authored body and text-box content and resolves each
reference to its expected target bytes or URL after reopen.

## Smells

None.

## Nitpicks

None.

## Not found

Both pass-1 defects remain closed. A related footnote insertion survives a
later typed mutation, and header drawing allocation no longer depends on
replacement history. The pass-2 namespace-shadow defect is closed for the
`Document` path. Producer `r` and `wp` meanings survive comment mutation, and
producer `wp` meaning survives picture, hyperlink, and later typed footnote
mutation. The pass-2 ordinary body and text-box operation-order case is also
closed, subject to the later-reorder defect in D3.

No additional defects were found in comment root collisions, locally complete
authored hyperlink or drawing fragments, aliases, character references,
foreign same-local-name attributes, duplicate registered occurrences,
producer-owned raw references, related-part relationship scope, part-local
media ownership, serialization and reopen atomicity, custom part targets,
external image or internal hyperlink rejection, schema insertion order, or
complete-package determinism outside D2 and D3. No new trait, generic, module,
file, forwarding wrapper, unjustified panic, or other structural smell was
introduced. The public Rust additions remain additive for the pre-1.0 crate.

The focused regressions for retained footnote insertion, header replacement,
comment namespace shadows, footnote namespace shadows, and body plus text-box
operation order passed individually. The exact implementation diff passes
`git diff --check`. The progress checkpoint records 435 passing regression
tests with 4 ignored, 476 passing `rdocx-oxml` tests plus its doctest, scoped
check, clippy, formatting, hash harness, prose, and skill-sync gates as passing.
