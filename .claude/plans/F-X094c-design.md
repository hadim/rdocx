# F-X094c, Priority rdocx Python collaboration, comparison, layout, and TOC

**Status**: approved
**Sprint**: S72
**Size**: L
**Depends on**: F-X094b, F-148, F-232, F-234, F-235

## Problem

The Python `Document` exposes IO, rendering, paragraphs, tables, and simple
mutation, but none of the native compare, comment, deterministic layout, or TOC
rebuild capabilities prioritized in GitHub Issue 76.

## Spec reference

- `docs/hld/08-rendering-spec.md`, deterministic Word layout results.
- `docs/hld/10-bindings-spec.md`, Python value and mutation conventions.
- `docs/hld/12-testing-strategy.md`, binding, typing, stub, and GIL gates.
- `docs/hld/14-development-backlog.md`, F-X094c.

## Approach

In existing binding files, add frozen typed `RunPosition`, `RunRange`,
`Comment`, `ComparisonDiagnostic`, `BoundingBox`, `LayoutFragment`,
`LayoutPage`, and `TocRebuildReport` values. Add compare, comment inspection and
mutation, deterministic layout and page lookup, and TOC rebuild methods.
Return immutable snapshots rather than borrowed handles or dictionaries.
Release the GIL for comparison, layout, TOC, and serialization. Advance the
binding revision exactly once after a successful structural mutation and never
after a failed staged operation.

## Rejected alternatives

- Return dictionaries. Installed stubs cannot express the contract precisely.
- Return borrowed layout or comment handles. They can stale after mutation.
- Expand to all-story comments or complete comparison. Later named stories own
  those surfaces.
- Add a new binding module. The existing file remains within the approved
  structural boundary for this bounded surface.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| binding | `priority_word_operations_return_typed_snapshots_and_remain_atomic` | Compare, comments, layout, TOC, strict typing, reopen, and failed-operation atomicity. |
| binding | GIL concurrency | Long comparison, layout, TOC, and serialization permit another Python thread to progress. |
| typing | installed mypy and stubtest | Frozen classes, nullable fields, tuples, and method signatures match runtime. |
| lifecycle | binding revision | Successful mutations invalidate once, failures invalidate nothing. |

The test gate is `priority_word_operations_return_typed_snapshots_and_remain_atomic`.

## HLD impact

- `docs/hld/08-rendering-spec.md`
- `docs/hld/10-bindings-spec.md`
- `docs/hld/12-testing-strategy.md`
- `docs/hld/14-development-backlog.md`

## Risk routing

- WASM or PyO3 bindings. Exclude both binding crates from workspace Rust tests,
  run both WASM checks, build the mixed package, and run pytest, strict mypy,
  stubtest, and clean abi3 wheel installation.
- Layout. Use deterministic fonts for every returned geometry assertion.

## Hash harness

Expected unchanged across all 49 entries.

## Implementation checklist

- [ ] Add and register frozen value classes in existing files.
- [ ] Implement detached methods and exact error mapping.
- [ ] Advance binding revision only after successful mutation.
- [ ] Update exports, stubs, README, typing, and thread tests.
- [ ] Run native, binding, wheel, WASM, and full gates.
- [ ] Update exactly the listed HLD files.

## Open questions

None.
