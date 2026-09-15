# F-X094d, rdocx Python sections, styles, rich stories, and hyperlinks

**Status**: completed
**Sprint**: S72
**Size**: L
**Depends on**: F-252, F-255, F-X094c

## Problem

The Python binding has no ordered section or style inspection, no rich F-252
header and footer graph, and no relationship-resolved hyperlink inventory.
Existing body handles cannot represent related-story ownership requested in
GitHub Issue 76.

## Spec reference

- `docs/hld/03-architecture.md`, facade ownership and source identity.
- `docs/hld/04-opc-and-packaging.md`, story and relationship ownership.
- `docs/hld/10-bindings-spec.md`, Python API shape.
- `docs/hld/12-testing-strategy.md`, installed binding gates.
- `docs/hld/14-development-backlog.md`, F-252, F-255, and F-X094d.

## Approach

After F-252, mirror its final owner and variant vocabulary with frozen snapshot
classes for sections, styles, story items, header and footer variants, and
hyperlinks. Tuple-returning document accessors preserve source order,
inheritance, physical owner, nested path, and relationship-scoped URL
resolution. Add `StoryItemRef::links() -> Result<Vec<LinkInfo>>` to the existing
native story facade so the binding can enumerate modeled hyperlinks without
parsing raw XML or reopening a second package graph. Add a story-wide ordered
projection that pairs each existing `ContentLocation` with its existing
`LinkInfo`, so interleaved ancestor and nested-item links retain physical source
order. Resolve each relationship through the item's checked `StoryId`. Add no
setters, raw XML exposure, new public types, dependencies, modules, or second
document tree. Keep all implementations in existing files.

## Rejected alternatives

- Expose raw OXML values. That bypasses the native facade.
- Return dictionaries. They lose installed typing guarantees.
- Add borrowed related-story handles. They require a second stale-path model.
- Implement before F-252. The final header and footer vocabulary is not ready.
- Parse story XML in the binding. Item fragments can inherit namespace bindings
  from their physical story root, and reparsing would create a second model.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| binding | `word_structure_snapshots_preserve_order_ownership_and_types` | Three sections, inherited and independent rich variants, styles, and links return exact frozen records after reopen. |
| typing | installed mypy and stubtest | Section, style, story, variant, path, and hyperlink types match runtime. |
| relationship | story-scoped hyperlink snapshots | Equal relationship ids resolve through their physical owner only, and interleaved nested links retain source order. |
| native | `story_item_links_resolve_only_through_the_checked_owner` | Existing `ContentLocation` and `LinkInfo` values retain physical source order and resolve equal relationship ids through distinct body and header scopes. |
| lifecycle | snapshot stability | Returned values remain immutable and later document mutation returns fresh snapshots. |

The test gate is `word_structure_snapshots_preserve_order_ownership_and_types`.

## HLD impact

- `docs/hld/03-architecture.md`
- `docs/hld/04-opc-and-packaging.md`
- `docs/hld/10-bindings-spec.md`
- `docs/hld/12-testing-strategy.md`
- `docs/hld/14-development-backlog.md`

## Risk routing

- WASM or PyO3 bindings. Run workspace tests with binding exclusions, both
  WASM checks, mixed-package build, pytest, strict mypy, stubtest, and clean
  abi3 wheel installation.

## Hash harness

Expected unchanged across all 49 entries.

## Implementation checklist

- [x] Wait for the completed F-252 owner and variant surface.
- [x] Add frozen snapshots in existing binding files.
- [x] Add typed native story-item hyperlink enumeration using existing types.
- [x] Preserve source order, inheritance, and relationship scope.
- [x] Update exports, stubs, README, and installed tests.
- [x] Run native, binding, wheel, WASM, and full gates.
- [x] Update exactly the listed HLD files.

## Open questions

None.
