# F-X090, Accept part-local producer drawing identities

**Status**: completed
**Sprint**: S72
**Size**: S
**Depends on**: F-255

## Problem

`DocumentIdentifiers` validates every producer `wp:docPr/@id` in one global
set in `crates/rdocx/src/document.rs`. Word legitimately reuses a drawing id
in different physical parts, so GitHub Issue 72 makes an ordinary body image
plus header logo fail during open. Authored identities must still avoid the
complete occupied union.

## Spec reference

- `docs/hld/03-architecture.md`, the Word facade identifier owner.
- `docs/hld/04-opc-and-packaging.md`, part ownership and collision scans.
- `docs/hld/10-bindings-spec.md`, native deterministic mutation.
- `docs/hld/12-testing-strategy.md`, package identifier gates.
- `docs/hld/13-risks-and-open-questions.md`, preserved occupants.
- `docs/hld/14-development-backlog.md`, F-X090.

## Approach

Validate normalized drawing definitions in a fresh set for each physical XML
part, then merge the successful values into the existing global occupied and
preserved sets without a second uniqueness rejection. Keep the authored
allocator package-global. Preserve producer XML and continue treating numeric
character-reference aliases as duplicates inside one part.

## Rejected alternatives

- Renumber producer drawings on open. Opening must not mutate input XML.
- Make authored allocation part-local. F-249 and F-255 require globally fresh
  facade-authored drawing identities.
- Accept same-part duplicates. The interoperability report establishes only
  cross-part reuse.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| regression | `cross_part_producer_drawing_ids_do_not_block_document_open` | Body and header reuse one id, open and mutate, preserve both producer parts, allocate outside the union, save, and reopen. |
| regression | `same_part_normalized_drawing_ids_remain_invalid` | `1` and `&#49;` inside one part still fail before mutation. |
| round-trip | `part_local_drawing_identity_scope_survives_story_round_trip` | Body and related parts may reuse producer values across physical parts through repeated saves. |
| unit | `foreign_doc_pr_does_not_enter_drawing_identity_scope` | A foreign same-local-name element is ignored. |

The test gate is `cross_part_producer_drawing_ids_do_not_block_document_open`.

## HLD impact

- `docs/hld/03-architecture.md`
- `docs/hld/04-opc-and-packaging.md`
- `docs/hld/10-bindings-spec.md`
- `docs/hld/12-testing-strategy.md`
- `docs/hld/13-risks-and-open-questions.md`
- `docs/hld/14-development-backlog.md`

## Risk routing

- Any parser or serialiser. Prove prefix-tolerant reads, fixed output prefixes,
  schema order, and byte-exact producer drawing preservation.

## Hash harness

Expected unchanged across all 49 entries. Any delta blocks completion.

## Implementation checklist

- [x] Validate drawing definitions per physical XML part.
- [x] Merge valid producer values into global allocation occupancy.
- [x] Preserve the globally fresh authored allocator.
- [x] Add the positive and negative source-built regressions.
- [x] Run the full gate and unchanged hash harness.
- [x] Update exactly the listed HLD files.

## Open questions

None.
