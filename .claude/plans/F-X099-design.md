# F-X099, Expose direct body ownership for story items

**Status**: completed
**Sprint**: S73
**Size**: M
**Depends on**: F-253, F-X094d

## Problem

`Document::story_items` assigns a flat scan ordinal to `index_path` at
`crates/rdocx/src/document.rs:11469`. Nested fields, drawings, and controls can
therefore shift that ordinal away from the direct body-child coordinate required
by `RunPosition.body_index`. The Python snapshot at
`crates/rdocx-py/src/document.rs:864` exposes no safe bridge between the two.

## Spec reference

- `docs/hld/03-architecture.md`, "Container-neutral Word story editing also belongs to the `rdocx` facade".
- `docs/hld/10-bindings-spec.md`, "Native Rust also exposes the concrete non-exhaustive `StoryKind`".
- `docs/hld/10-bindings-spec.md`, "Python `Document` facade" story snapshot contract.

## Approach

Add `StoryItemRef::direct_body_index() -> Result<Option<usize>>`. Resolve the
containing direct body child through the existing checked source spans. Direct
and nested main-story items return that owner. Items in other physical stories
return `None`. Add `direct_body_index: int | None` to the frozen Python
`StoryItem` constructor, property, snapshot, stub, typing smoke test, and runtime
test. Keep `index_path` unchanged.

## Rejected alternatives

- Reinterpreting `index_path` would silently break existing callers and checked
  content locations.
- Making `RunPosition` accept a story scan ordinal would mix two ownership
  models and remain unsafe for nested items.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| binding | `story_items_expose_safe_direct_body_owners` | Native and Python items map direct and nested main-story content to the containing body child, while related stories return no body index and legacy paths do not change. |
| typing | `typing_smoke.py` | Installed stubs expose `int | None`. |

The test gate is the backlog binding test named above.

## HLD impact

- `docs/hld/03-architecture.md`
- `docs/hld/10-bindings-spec.md`
- `docs/hld/12-testing-strategy.md`
- `docs/hld/14-development-backlog.md`

## Risk routing

- Public API of a published crate. State additive pre-1.0 semver impact, run
  `cargo publish --dry-run` for `rdocx`, and assert the packaged README and
  archive size remain within the existing gate.
- WASM or PyO3 bindings. Run the binding tests with the required workspace
  exclusions and `cargo check --target wasm32-unknown-unknown -p rdocx-wasm -p
  rpptx-wasm`.

## Hash harness

Expected to be unchanged. This adds inspection metadata only.

## Implementation checklist

- [x] Add failing native and Python body-owner regressions.
- [x] Resolve the containing direct owner through existing checked spans.
- [x] Expose the additive native and frozen Python properties.
- [x] Update stubs, typing coverage, and public binding specification.
- [x] Run binding gates, package dry run, hash harness, full verification, and microscope.

## Open questions

None. The user approved the additive bridge rather than changing `index_path`.
