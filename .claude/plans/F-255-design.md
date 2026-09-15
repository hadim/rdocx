# F-255, Part-scoped assets, links, and relationships

**Status**: completed
**Sprint**: S72
**Size**: M
**Depends on**: F-253, F-249

## Problem

Body picture insertion allocates against `doc_part_name` at
`crates/rdocx/src/document.rs:6983`, while header image code contains a
separate special path at `crates/rdocx/src/document.rs:8307`. Public callers
cannot request the same image, hyperlink, chart, or related content operation
for an arbitrary story owner, and a main-document assumption can produce a
valid package whose local `r:id` resolves in the wrong relationship scope.

## Spec reference

- `docs/hld/02-scope-and-non-goals.md`, "Modern DOCX capability matrix",
  DOCX-020.
- `docs/hld/03-architecture.md`, relationship resolution and story ownership.
- `docs/hld/04-opc-and-packaging.md`, "Relationship types", header and footer
  ownership, and "Media".
- `docs/hld/05-drawingml-model.md`, preservation and schema order.
- `docs/hld/09-charts-spec.md`, Word chart part closure.
- `docs/hld/14-development-backlog.md`, "F-255, Part-scoped assets, links, and relationships".

## Approach

Centralize the existing concrete relationship and media allocation helpers on
an explicit F-253 `StoryId` owner. Add fallible story-scoped picture and
hyperlink insertion, relationship lookup, and internal relationship
validation. Route existing body and header helpers through the same owner-aware
implementation. Resolve text-box ownership through its containing story part.
Keep chart and other related payloads on the same private owner helpers so
later public authoring stories cannot regress to the main-document assumption.

## Rejected alternatives

- A relationship-owner trait has only one package implementation today.
- Globally unique `r:id` values do not change OOXML's part-local lookup rule.
- Copying a relationship into every part creates ambiguous and orphaned graphs.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| round-trip | `equal_related_content_resolves_only_through_its_story_owner` | Equal content in body, header, footer, note, and text-box stories uses only the correct owner relationships after reopen. |
| regression | `wrong_scope_relationships_are_rejected_atomically` | Missing, external, wrong-type, and foreign-owner relationship references leave bytes unchanged. |
| integration | `legacy_body_and_header_helpers_use_the_shared_owner_path` | Existing helpers retain output while sharing one allocation implementation. |

The story test gate is round-trip: equal content in body, header, footer, note,
and text-box stories resolves only through its correct owner relationships.

## HLD impact

- `docs/hld/02-scope-and-non-goals.md`
- `docs/hld/03-architecture.md`
- `docs/hld/04-opc-and-packaging.md`
- `docs/hld/05-drawingml-model.md`
- `docs/hld/09-charts-spec.md`
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

Expected to be unchanged. Existing body and header operations must retain their
final relationship and media allocation.

## Implementation checklist

- [x] Resolve every supported story to its owning OPC part.
- [x] Centralize part-scoped relationship lookup and validation.
- [x] Add story-scoped picture and hyperlink insertion.
- [x] Route existing helpers through the explicit owner path.
- [x] Test identical local ids and content across independent owners.

## Open questions

None. The story covers ordinary relationship-bearing content only. Executable
OLE and ActiveX mutation remain with their dedicated APIs. A text box uses the
relationships of the part containing the drawing.
