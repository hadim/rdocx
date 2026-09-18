# F-X116, Make Python story reads linear and complete

**Status**: completed
**Sprint**: S73
**Size**: L
**Depends on**: F-X106c, F-X109

## Problem

Python `Document.story_items` and `Document.hyperlinks` rebuild and rescan the
complete story inventory once per returned item or story, producing quadratic
runtime. Paragraph text and run views also disagree when visible text lives in
a tracked insertion or inline content control, so a caller can calculate the
wrong run position for a comment or edit.

## Spec reference

- `docs/hld/03-architecture.md`, story inventory and accepted text ownership.
- `docs/hld/10-bindings-spec.md`, paragraph and run handle semantics.
- `docs/hld/12-testing-strategy.md`, complexity and binding gates.
- `docs/hld/14-development-backlog.md`, F-X116.

## Approach

Build the native story source and owner inventory once per public snapshot and
project all item text and hyperlinks from that immutable inventory. Extend the
paragraph run address to carry the recursive inline path for visible runs in
accepted insertions and inline content controls. Use the same accepted-view
walker for `StoryItem.text`, `Paragraph.text`, and `Paragraph.runs`. Exclude
deleted text consistently, preserve source order, and reject stale path-backed
handles after structural mutation.

## Rejected alternatives

- Cache story XML indefinitely on Document. Mutation invalidation would be
  wider and more error-prone than one bounded snapshot.
- Document the paragraph view as partial. The existing views already disagree,
  and partial run indexes are unsafe mutation coordinates.
- Flatten nested runs into copied strings. Callers need checked live handles
  for formatting, splitting, and comment anchors.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| performance | `python_story_inventory_scales_linearly` | Doubling a large paragraph and cell corpus stays within the declared linear work and bounded timing ratio. |
| binding | `paragraph_views_include_accepted_nested_runs_in_source_order` | Plain, inserted, and inline-control runs agree across item text, paragraph text, and live run handles. |
| lifecycle | nested run mutation | Formatting, split, and comment coordinates target the visible run and stale handles fail loudly. |

The **test gate** is the performance test named in the backlog.

## HLD impact

- `docs/hld/03-architecture.md`
- `docs/hld/10-bindings-spec.md`
- `docs/hld/12-testing-strategy.md`
- `docs/hld/14-development-backlog.md`

## Risk routing

- Performance or cache behavior. Add counted traversal instrumentation and a
  bounded scaling gate that does not depend on one machine's absolute speed.
- WASM or PyO3 bindings. Run both WASM checks, full installed Python tests,
  strict mypy, stubtest, and a clean abi3 installation.
- Public API of a published crate. Inspect handle and snapshot API changes,
  rustdoc, packages, and archive sizes.

## Hash harness

Expected to be unchanged because the sample generator does not use Python
story snapshots.

## Implementation checklist

- [x] Add counted linear-scaling and nested-visible-run failures.
- [x] Materialize one native story inventory per snapshot call.
- [x] Use one accepted-view walker for item, paragraph, and run projections.
- [x] Carry recursive run paths through live Python handles and lifecycle checks.
- [x] Run performance, binding, WASM, API, hash harness, full verification, and microscope gates.

## Open questions

None.
