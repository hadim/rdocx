# F-X125, Compare table grid changes

**Status**: completed
**Sprint**: S74
**Size**: M
**Depends on**: F-X065, F-X097

## Problem

Issue 127 shows that adding, removing, or resizing a table column aborts the
whole comparison. The existing row and cell comparison cannot reproduce two
different active grids.

## Spec reference

- `docs/hld/04-opc-and-packaging.md`, body comparison and tracked section ownership.
- `docs/hld/12-testing-strategy.md`, tracked table-grid and regression gates.
- `docs/hld/14-development-backlog.md`, "F-X125, Compare table grid changes".

## Approach

When active grids differ, serialize the original table as a tracked deletion
and the edited table as a tracked insertion at the same body boundary. Reuse
the existing revision wrappers and identifiers. Keep row and cell comparison
for equal grids. The postconditions remain exact acceptance and rejection.

## Rejected alternatives

- Patch only grid widths. Added and removed cells would still be ambiguous.
- Return a diagnostic and ignore the grid. Acceptance would not equal edited.
- Drop the table. Rejection must reproduce the original.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| regression | `comparison_tracks_changed_table_grids_as_table_replacement` | Gain, loss, and resize accept and reject exactly. |
| round-trip | tracked replacement save and reopen | Wrapper order, grid XML, cells, and revision metadata survive. |
| regression | equal-grid controls | Row, cell, and surrounding text edits retain focused revisions. |

The **test gate** is the named regression.

## HLD impact

- `docs/hld/04-opc-and-packaging.md`
- `docs/hld/12-testing-strategy.md`
- `docs/hld/14-development-backlog.md`

## Risk routing

- Parser and serializer behavior. Prove schema sequence, prefix tolerance, raw
  table retention, exact acceptance, and exact rejection.

## Hash harness

Expected unchanged. Sample generation does not call comparison.

## Implementation checklist

- [x] Add the reported grid gain, loss, and resize matrix.
- [x] Emit one deletion and insertion replacement for unequal grids.
- [x] Preserve equal-grid row and cell comparison.
- [x] Run focused comparison, revision, round-trip, and hash verification gates.

## Open questions

None. Table replacement is the only bounded strategy that proves both exact
postconditions for arbitrary grid changes.
