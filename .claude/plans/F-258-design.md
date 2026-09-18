# F-258, Complete M23 row and cell authoring

**Status**: completed
**Sprint**: S73
**Size**: L
**Depends on**: F-257, F-X100

## Problem

`Row` and `Cell` in `crates/rdocx/src/table.rs` cover a subset of the typed
OXML model. The facade cannot author all M23 row and cell values, cannot clear
the existing one-way header, split, and wrapping toggles, and does not enforce
grid and merge consistency before publication.

## Spec reference

- `docs/hld/02-scope-and-non-goals.md`, modern DOCX row and cell authoring rows.
- `docs/hld/04-opc-and-packaging.md`, row and cell schema order and table invariants.
- `docs/hld/08-rendering-spec.md`, row pagination, cell layout, and nested tables.
- `docs/hld/10-bindings-spec.md`, native Word facade stability.
- `docs/hld/12-testing-strategy.md`, M23 table property differential gate.
- `docs/hld/14-development-backlog.md`, F-258.

## Approach

Extend the existing `Row` and `Cell` handles with checked setters for exact or
minimum height, repeating header, split policy, row alignment, grid-before and
grid-after omissions, horizontal and vertical merges, cell width, individual
borders and margins, shading, vertical alignment, text direction, conditional
formatting, wrapping, and nested tables. Keep the existing convenience methods
that set a flag true, and add explicit optional or boolean setters that can
write false or remove the direct value. Validate the complete candidate table,
including grid coverage and merge topology, before the staged document is
published.

## Rejected alternatives

- Change existing zero-argument setters to boolean parameters. That would
  break source compatibility when additive companion setters are sufficient.
- Treat row cloning as part of this story. Issue 95 requires identity
  reconciliation and remains isolated in F-X107.
- Permit invalid grid or merge state temporarily. A saved intermediate table
  can be a valid ZIP that Word refuses or repairs.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| differential | `m23_nested_rows_and_cells_match_word` | Sanitized nested tables reproduce every reviewed row and cell property without unmodelled fallback. |
| round-trip | row and cell property matrix | Every supported value, explicit false toggle, alias, and schema slot survives reopen. |
| regression | merge and grid invariants | Valid omissions and merges lay out correctly, while invalid topology is rejected atomically. |
| layout | pagination matrix | Exact and minimum heights, repeated headers, split policy, vertical text, and wrapping match deterministic output. |

The **test gate** is the differential test named in the backlog.

## HLD impact

- `docs/hld/02-scope-and-non-goals.md`
- `docs/hld/04-opc-and-packaging.md`
- `docs/hld/08-rendering-spec.md`
- `docs/hld/10-bindings-spec.md`
- `docs/hld/12-testing-strategy.md`
- `docs/hld/14-development-backlog.md`

## Risk routing

- Unit conversion. Keep truncating conversions and attribute every geometry
  delta to the reviewed row or cell value.
- Any parser or serialiser. Verify row and cell `xsd:sequence`, prefix aliases,
  explicit false values, and exact raw-subtree preservation.
- Layout. Use deterministic fonts for row pagination and nested-table output.
- Public API of a published crate. State additive and companion-setter impacts,
  inspect rustdoc and API changes, and run package dry-runs and size checks.
- External oracle comparison. Pin Word, LibreOffice, and Poppler and record the
  exact structural and visual commands.

## Hash harness

Expected changes are limited to samples that opt into newly completed row or
cell properties. Every delta must be reviewed against this plan.

## Implementation checklist

- [x] Add failing sanitized row, cell, merge, and nested-table differentials.
- [x] Complete checked row height, grid, alignment, and toggle authoring.
- [x] Complete checked cell border, margin, width, fill, alignment, direction, merge, and wrap authoring.
- [x] Validate the whole table and preserve unrelated raw slots before commit.
- [x] Run parser, layout, public API, oracle, hash harness, full verification, and microscope gates.

## Open questions

None.
