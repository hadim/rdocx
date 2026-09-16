# F-X119, Complete round-three Python authoring and inspection

**Status**: completed
**Sprint**: S73
**Size**: L
**Depends on**: F-X106c, F-X108, F-X109, F-X115, F-X118

## Problem

Python still cannot insert Word pictures, target comments inside table cells,
edit presentation notes, inspect presentation shape geometry and identity or
run font details, read autofit mode, or replace presentation run text without
discarding formatting. These are the remaining concrete scripting gaps from
Issue 121 after the earlier binding stories.

## Spec reference

- `docs/hld/03-architecture.md`, path ownership and staged mutation.
- `docs/hld/05-drawingml-model.md`, presentation geometry and text properties.
- `docs/hld/10-bindings-spec.md`, Python document and presentation surfaces.
- `docs/hld/12-testing-strategy.md`, installed binding and typing gates.
- `docs/hld/14-development-backlog.md`, F-X119.

## Approach

Bind byte-based Word picture insertion with a safe filename, optional native or
explicit size, and checked insertion after a StoryItem. Add a path-aware comment
position for body and table-cell paragraphs while keeping existing direct-body
`RunPosition` source compatible. Bind the notes setter from F-X118. Expose
immutable presentation shape bounds, non-visual id and name, run font name,
size and color, and text autofit mode through existing handles. Add a Run text
setter so property-preserving replacement does not rebuild the paragraph.

## Rejected alternatives

- Require filesystem paths for picture insertion. Wheels already accept bytes
  elsewhere and in-memory automation must not need temporary files.
- Change the fields of the existing `RunPosition`. That would break callers
  constructing the published value directly.
- Make shape inspection depend on python-pptx. The requested data already
  exists in the native resolved model.
- Preserve only the first run while replacing a whole paragraph. A run setter
  makes the formatting ownership explicit.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| binding | `python_round_three_authoring_and_inspection_is_typed_and_lossless` | Picture insertion, cell comments, notes editing, geometry, identity, fonts, autofit, and run text match native values after reopen. |
| preservation | property-preserving edits | Run text and notes edits retain formatting, unrelated XML, relationships, and package identities. |
| typing | installed mypy and stubtest | Lengths, colors, ids, paths, optional sizing, and setters match runtime descriptors. |

The **test gate** is the binding test named in the backlog.

## HLD impact

- `docs/hld/03-architecture.md`
- `docs/hld/05-drawingml-model.md`
- `docs/hld/10-bindings-spec.md`
- `docs/hld/12-testing-strategy.md`
- `docs/hld/14-development-backlog.md`

## Risk routing

- Public API of published crates. Keep existing constructors source compatible,
  inspect the additive API diff, run rustdoc, package dry-runs, and archive
  checks.
- WASM or PyO3 bindings. Run both WASM checks, both installed Python packages,
  pytest, strict mypy, stubtest, clean abi3 installs, and wheel metadata checks.
- Any parser or serializer. Preserve table-cell comment paths, media
  relationships, and presentation child order exactly.

## Hash harness

Expected to be unchanged because samples are not generated through these
Python binding operations.

## Implementation checklist

- [x] Add installed runtime and typing failures for every Issue 121 item.
- [x] Bind Word picture insertion and checked StoryItem placement.
- [x] Add source-compatible path-aware table-cell comment positions.
- [x] Bind notes mutation and presentation geometry, identity, font, and autofit reads.
- [x] Add property-preserving presentation Run text mutation.
- [x] Run binding, preservation, WASM, API, hash harness, full verification, and microscope gates.

## Open questions

None.
