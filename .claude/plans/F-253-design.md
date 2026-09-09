# F-253, Container-neutral story editing

**Status**: approved
**Sprint**: S72
**Size**: L
**Depends on**: F-240

## Problem

The facade has separate body traversal at `crates/rdocx/src/document.rs:6599`,
cell traversal in `crates/rdocx/src/table.rs`, plain-text footnote projection,
and private header and footer package resolution. There is no public identity
or location that addresses the same supported content shape across body,
cells, related stories, notes, comments, and text boxes.

## Spec reference

- `docs/hld/02-scope-and-non-goals.md`, "Modern DOCX capability matrix",
  DOCX-018.
- `docs/hld/03-architecture.md`, Word grammar ownership and "Facade
  conventions".
- `docs/hld/04-opc-and-packaging.md`, relationship-scoped story parts.
- `docs/hld/10-bindings-spec.md`, "Native Word facade stability".
- `docs/hld/14-development-backlog.md`, "F-253, Container-neutral story editing".

## Approach

Add concrete non-exhaustive `StoryKind`, owned `StoryId`, `ContentLocation`,
and borrowed `StoryItemRef` values in the existing document facade file. A
location records the owning story and an index path into the existing typed
tree. `Document::stories` and `Document::story_items` discover owners in stable
package and document order and expose paragraphs, tables, controls, fields,
drawings, and preserved nodes without cloning or building another tree. Add one
fallible text mutation that resolves a location against a staged candidate so
every supported story returns the same stale, kind, owner, and bounds errors.

## Rejected alternatives

- A trait hierarchy has no second storage implementation and would add
  indirection.
- A flattened cloned tree would lose owner relationships and raw XML identity.
- Separate public editors per story would multiply traversal and error rules.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| integration | `one_generic_mutation_edits_the_same_shape_in_every_story` | One location resolver visits and edits the same supported content shape in body, cell, header, footer, note, comment, and text-box stories with identical errors. |
| round-trip | `story_traversal_preserves_owner_order_and_raw_nodes` | All supported item kinds and retained nodes keep order and owner identity after save and reopen. |
| regression | `invalid_story_locations_are_atomic` | Wrong owner, kind, index, and stale paths leave document bytes unchanged. |

The story test gate is integration: one generic mutation visits and edits the
same supported content shape in every story with identical error behavior.

## HLD impact

- `docs/hld/02-scope-and-non-goals.md`
- `docs/hld/03-architecture.md`
- `docs/hld/04-opc-and-packaging.md`
- `docs/hld/10-bindings-spec.md`
- `docs/hld/12-testing-strategy.md`
- `docs/hld/14-development-backlog.md`

## Risk routing

- Any parser or serialiser. Read `docs/hld/04-opc-and-packaging.md` and
  `docs/hld/06-presentationml-model.md`. Prove fixed write prefixes, schema
  order, prefix-tolerant reads, and byte-exact retained subtrees.
- Public API of a published crate. Read `docs/hld/10-bindings-spec.md` and the
  structural rules in `CLAUDE.md`. State additive pre-1.0 semver impact, run
  `cargo publish --dry-run -p rdocx`, and assert the packaged crate size.

## Hash harness

Expected to be unchanged. Read-only traversal emits nothing and staged edits
must preserve equivalent existing serialization.

## Implementation checklist

- [ ] Add concrete story identity, content location, item kind, and error values.
- [ ] Discover supported owners in deterministic order.
- [ ] Traverse the existing typed tree without cloning it into a second model.
- [ ] Resolve one staged generic mutation with uniform errors.
- [ ] Test every named story and supported item category.

## Open questions

None. The shared story model stays in existing files, and index paths are
operation-scoped and re-resolved after mutation.
