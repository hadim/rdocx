# F-X106a, Expose indexed content mutation and counted replacement in Python

**Status**: completed
**Sprint**: S73
**Size**: L
**Depends on**: F-254, F-X099

## Problem

The native facade can insert, clone, move, locate, and remove direct-body
content and can replace literal or regular-expression text across runs with an
exact count. Python exposes only append and removal, which prevents safe edit
passes that need a stable location or an expected replacement count.

## Spec reference

- `docs/hld/03-architecture.md`, staged document mutation.
- `docs/hld/10-bindings-spec.md`, Python handles, revisions, and exceptions.
- `docs/hld/12-testing-strategy.md`, installed binding and atomicity gates.
- `docs/hld/14-development-backlog.md`, F-X106a.

## Approach

Add Python methods for `insert_paragraph`, `insert_content`, `clone_content`,
`move_content`, handle-based `find_content_index`, `try_replace_text`, and
`replace_all_regex`. Bind the native `ContentFragment` as an opaque owned
Python value with a typed kind. A new location-aware pop operation returns that
value, and `insert_content` consumes a clone without exposing or reparsing its
XML. Use zero-based direct-body coordinates and existing Paragraph and Table
handles. Return exact indices or counts. Run every native fallible operation
before bumping the shared binding revision, then invalidate all structural
handles once after success.

## Rejected alternatives

- Expose `ContentFragment` as an untyped Python dictionary. The opaque native
  value preserves its checked owner metadata without creating a second model.
- Return no replacement count. Batch callers need the count to detect stale
  assumptions.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| binding | `python_indexed_content_mutation_is_counted_and_atomic` | Paragraph and fragment insert, pop, clone, move, locate, literal replacement, and regex replacement return exact results and reopen correctly. |
| lifecycle | handle revisions | Successful structural edits stale prior handles once, while rejected edits stale none. |
| typing | installed mypy and stubtest | Coordinates, handles, regex options, counts, and errors match the runtime. |

The **test gate** is the binding test named in the backlog.

## HLD impact

- `docs/hld/03-architecture.md`
- `docs/hld/10-bindings-spec.md`
- `docs/hld/12-testing-strategy.md`
- `docs/hld/14-development-backlog.md`

## Risk routing

- WASM or PyO3 bindings. Run both WASM checks, the mixed Python package,
  pytest, strict mypy, stubtest, and clean abi3 wheel installation.

## Hash harness

Expected to be unchanged because the sample generator does not call Python
mutation APIs.

## Implementation checklist

- [x] Add failing runtime and typing tests for every requested method.
- [x] Bind opaque fragment pop and insert plus indexed paragraph insert, clone, move, and lookup.
- [x] Bind counted literal and regex replacement.
- [x] Prove exact revision bump and failure atomicity.
- [x] Run binding, WASM, hash harness, full verification, and microscope gates.

## Open questions

None.
