# F-X094d, rdocx Python sections, styles, rich stories, and hyperlinks

**Status**: approved
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
resolution. Add no setters, raw XML exposure, or second document tree. Keep
all implementations in existing binding files.

## Rejected alternatives

- Expose raw OXML values. That bypasses the native facade.
- Return dictionaries. They lose installed typing guarantees.
- Add borrowed related-story handles. They require a second stale-path model.
- Implement before F-252. The final header and footer vocabulary is not ready.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| binding | `word_structure_snapshots_preserve_order_ownership_and_types` | Three sections, inherited and independent rich variants, styles, and links return exact frozen records after reopen. |
| typing | installed mypy and stubtest | Section, style, story, variant, path, and hyperlink types match runtime. |
| relationship | story-scoped hyperlink snapshots | Equal relationship ids resolve through their physical owner only. |
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

- [ ] Wait for the completed F-252 owner and variant surface.
- [ ] Add frozen snapshots in existing binding files.
- [ ] Preserve source order, inheritance, and relationship scope.
- [ ] Update exports, stubs, README, and installed tests.
- [ ] Run native, binding, wheel, WASM, and full gates.
- [ ] Update exactly the listed HLD files.

## Open questions

None.
