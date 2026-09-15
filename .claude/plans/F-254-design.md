# F-254, Generic insert, move, clone, and remove operations

**Status**: completed
**Sprint**: S72
**Size**: L
**Depends on**: F-253, F-249

## Problem

Body insertion and removal at `crates/rdocx/src/document.rs:6889` and
`crates/rdocx/src/document.rs:6954` are direct, body-only mutations. They panic
or return a boolean for bounds, do not share one error model with nested
containers, and cannot validate stale locations or cross-owner movement before
partially changing a document.

## Spec reference

- `docs/hld/02-scope-and-non-goals.md`, "Modern DOCX capability matrix",
  DOCX-019.
- `docs/hld/03-architecture.md`, Word grammar ownership and staged mutation.
- `docs/hld/04-opc-and-packaging.md`, "Package integrity".
- `docs/hld/10-bindings-spec.md`, "Native Word facade stability".
- `docs/hld/14-development-backlog.md`, "F-254, Generic insert, move, clone, and remove operations".

## Approach

Add an owned concrete `ContentFragment` over the supported existing OXML node
types. Add fallible `insert_content`, `remove_content_at`, `clone_content`, and
`move_content` methods that consume F-253 locations and publish only a fully
validated staged document. Moves require one owner and adjust the destination
after source removal. Clones allocate fresh document identities and preserve
relationship references only where the owner scope is unchanged. Unsupported
or raw nodes remain movable and cloneable within one owner without parsing.
For insertion before existing content, the destination is a canonical F-253
location returned by `StoryItemRef::location`. It identifies that flattened
item, which must be a genuine direct owner child of the matching kind. Nested
projections are not insertion boundaries. `ContentLocation::end(story)` is the
distinct boundary after final direct content, including for empty and
self-closing owners. The body end boundary remains before the schema-final
section properties.

## Rejected alternatives

- Extending the body-only index methods would not cover cells or stories.
- Cross-owner moves would require dependency-closure import and belong to F-256.
- In-place multi-step edits cannot guarantee atomic failure.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| regression | `interleaved_content_operations_preserve_order_references_and_raw_xml` | Insert, move, clone, and remove across body and cell content retain exact order, references, and untouched raw XML. |
| unit | `same_owner_move_adjusts_destination_after_removal` | Forward and backward moves land at the requested final position. |
| regression | `invalid_or_stale_content_operations_are_atomic` | Bounds, stale paths, kind mismatch, ranges, and cross-owner moves leave bytes unchanged. |

The story test gate is regression: interleaved operations across body and cell
content preserve exact order, references, and untouched raw XML.

## HLD impact

- `docs/hld/02-scope-and-non-goals.md`
- `docs/hld/03-architecture.md`
- `docs/hld/04-opc-and-packaging.md`
- `docs/hld/10-bindings-spec.md`
- `docs/hld/12-testing-strategy.md`
- `docs/hld/13-risks-and-open-questions.md`
- `docs/hld/14-development-backlog.md`

## Risk routing

- Any parser or serialiser. Read `docs/hld/04-opc-and-packaging.md` and
  `docs/hld/06-presentationml-model.md`. Prove fixed write prefixes, schema
  order, prefix-tolerant reads, and byte-exact retained subtrees.
- Public API of a published crate. Read `docs/hld/10-bindings-spec.md` and the
  structural rules in `CLAUDE.md`. State additive pre-1.0 semver impact, run
  `cargo publish --dry-run -p rdocx`, and assert the packaged crate size.

## Hash harness

Expected to be unchanged. Unrelated content and retained XML must remain byte
identical after each successful or rejected operation.

## Implementation checklist

- [x] Add the owned supported content fragment value.
- [x] Resolve source and destination locations before staged mutation.
- [x] Implement insert, remove, clone, and same-owner move.
- [x] Reject stale, invalid, and cross-owner operations atomically.
- [x] Preserve raw nodes and remap fresh document identities for clones.

## Open questions

None. Cloning fails when preserved XML contains an in-scope identity or
relationship reference whose ownership cannot be proven. Cross-owner movement
is rejected and cross-document import belongs to F-256.
