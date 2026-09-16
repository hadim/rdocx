# F-X107, Clone and remove existing table rows

**Status**: completed
**Sprint**: S73
**Size**: M
**Depends on**: F-258, F-X106a

## Problem

Public Table handles can inspect rows and cells but cannot add to an existing
formatted table or remove a row. Rebuilding the table loses row properties,
cell formatting, nested content, and preserved producer XML. Row header and
split setters also cannot clear a true value.

## Spec reference

- `docs/hld/04-opc-and-packaging.md`, table child order and package identities.
- `docs/hld/08-rendering-spec.md`, row geometry and pagination.
- `docs/hld/10-bindings-spec.md`, table handles and Python mutation lifecycle.
- `docs/hld/12-testing-strategy.md`, table and binding gates.
- `docs/hld/14-development-backlog.md`, F-X107.

## Approach

Add staged `Document::clone_table_row(table_location, source, insert_at) ->
Result<usize>` and `Document::remove_table_row(table_location, index) ->
Result<bool>` operations with zero-based row coordinates. Python `Table`
methods route through their existing document identity and checked table path.
Clone the complete modeled and preserved row, then reconcile document-wide
drawing, bookmark, comment, and relationship identities through the existing
staged document mutation boundary before publication. Keep the current
zero-argument row toggle conveniences and add boolean companion setters. Bump
the Python structural revision once after success. Normalize inherited body
namespace attribute names through `namespace_prefix()` before constructing the
row fragment scope, including an unused root default namespace written by Word.

## Rejected alternatives

- Add only an empty row. The reported use case depends on retained formatting.
- Copy the row bytes without identity reconciliation. Duplicate document IDs
  and range markers can make the package invalid.
- Put the native operation only on `Table<'_>`. That handle has no package or
  document identity owner and cannot reconcile relationships safely.
- Put row insertion on a new trait. Only the concrete document owner
  implements this operation today.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| binding | `table_rows_clone_remove_and_clear_through_native_and_python` | A formatted nested row clones, clears toggles, removes, and reopens through Rust and Python. |
| regression | identity reconciliation | Drawings, bookmarks, comments, links, and nested content remain valid and unique. |
| regression | root default namespace | A Word-style unused default namespace does not become an illegal `xmlns:xmlns` binding during clone. |
| layout | row geometry | Cloned exact and minimum heights plus repeating and split policy render correctly. |

The **test gate** is the binding test named in the backlog.

## HLD impact

- `docs/hld/04-opc-and-packaging.md`
- `docs/hld/08-rendering-spec.md`
- `docs/hld/10-bindings-spec.md`
- `docs/hld/12-testing-strategy.md`
- `docs/hld/14-development-backlog.md`

## Risk routing

- Any parser or serialiser. Verify table schema order, fixed-prefix output, and
  exact raw-subtree preservation after clone and removal.
- Layout. Use deterministic fonts for the row pagination assertion.
- Public API of a published crate. State the additive document operations and
  companion setters, run rustdoc, inspect the API diff, and run package
  dry-runs and size checks.
- WASM or PyO3 bindings. Run both WASM checks, pytest, strict mypy, stubtest,
  and clean abi3 installation.

## Hash harness

Expected to be unchanged because current samples do not clone or remove rows.

## Implementation checklist

- [x] Add failing native and Python formatted-row tests.
- [x] Implement staged clone with identity and relationship reconciliation.
- [x] Normalize default and prefixed root namespace bindings for row fragments.
- [x] Implement checked removal and bool row toggles.
- [x] Bind operations and lifecycle semantics in Python.
- [x] Run serialization, layout, binding, API, hash harness, full verification, and microscope gates.

## Open questions

None.
