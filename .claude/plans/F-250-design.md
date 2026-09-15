# F-250, Ordered mutable section facade

**Status**: completed
**Sprint**: S72
**Size**: L
**Depends on**: F-249

## Problem

The facade can inspect only the final body section through
`Document::section_properties`, even though paragraph properties can own every
preceding section boundary. The private iterator at
`crates/rdocx/src/document.rs:9958` proves the ordered data already exists, but
callers cannot look up or mutate it. Existing body removal at
`crates/rdocx/src/document.rs:6954` can also remove a section-ending paragraph
without validating the related header and footer story graph.

## Spec reference

- `docs/hld/02-scope-and-non-goals.md`, "Modern DOCX capability matrix",
  DOCX-015.
- `docs/hld/03-architecture.md`, "Facade conventions".
- `docs/hld/04-opc-and-packaging.md`, "Package integrity".
- `docs/hld/14-development-backlog.md`, "F-250, Ordered mutable section facade".

## Approach

Add concrete borrowed `SectionRef` and `Section` handles over the existing
`CT_SectPr` owners. Add total `section_count`, `sections`, `section`, and
`section_mut` accessors in document order. Add fallible staged insertion and
removal operations that manipulate paragraph-level or final section properties
without reordering body content. A removed boundary transfers the surviving
section semantics deliberately and prunes an authored related story only when
no remaining section reference reaches it. Existing final-section convenience
methods continue to operate on the final section.

## Rejected alternatives

- A second section tree would duplicate ownership and drift from the body.
- Raw XML surgery would bypass typed section state and package validation.
- Persistent synthetic section identifiers would leak non-OOXML state into a
  position-owned grammar.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| round-trip | `ordered_section_mutations_preserve_independent_story_references` | A portrait, landscape, portrait document retains all three ordered sections and independent references after lookup, insertion, mutation, removal, save, and reopen. |
| regression | `removing_a_section_never_orphans_a_shared_story` | Shared references survive and an unreferenced facade-owned part is retired only after its last owner disappears. |
| unit | `section_lookup_is_total_for_every_index` | Ordered access covers paragraph-level properties followed by the final section and rejects out-of-range indices without mutation. |

The story test gate is round-trip: a portrait, landscape, portrait document
retains all three ordered sections and their independent references after every
mutation.

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
- Layout and pagination. Read `docs/hld/08-rendering-spec.md`. Run the
  deterministic-font render gate and do not record a system-font baseline.
- Public API of a published crate. Read `docs/hld/10-bindings-spec.md` and the
  structural rules in `CLAUDE.md`. State the additive pre-1.0 semver impact,
  run `cargo publish --dry-run -p rdocx`, and assert the packaged crate size.

## Hash harness

Expected to be unchanged. The new mutation paths must reproduce existing
serialization for equivalent section state.

## Implementation checklist

- [x] Add ordered immutable and mutable section handles and total lookup.
- [x] Add staged section insertion and removal without a second tree.
- [x] Preserve paragraph order, final-section ownership, and raw section XML.
- [x] Retain shared stories and prune only unreferenced facade-owned stories.
- [x] Add round-trip and atomic-failure coverage.

## Open questions

None. Insertion creates an empty section at a section ordinal. Removal merges
with the following section, rejects removal of the sole section, and deletes
only unreachable facade-owned stories.
