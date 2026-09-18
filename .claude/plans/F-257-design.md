# F-257, Complete M23 table authoring

**Status**: completed
**Sprint**: S73
**Size**: L
**Depends on**: F-253

## Problem

The public `Table` facade in `crates/rdocx/src/table.rs` exposes selected
width, alignment, border, margin, layout, and grid setters, but it cannot
express the complete table property set required by the M23 corpus. Several
setters also accept values without checked validation, and the corpus gate
cannot distinguish an intentionally invisible border from an omitted border.

## Spec reference

- `docs/hld/02-scope-and-non-goals.md`, modern DOCX table authoring rows.
- `docs/hld/04-opc-and-packaging.md`, Word table property sequence and measurement values.
- `docs/hld/08-rendering-spec.md`, Word table geometry and pagination.
- `docs/hld/10-bindings-spec.md`, native Word facade stability.
- `docs/hld/12-testing-strategy.md`, private from-scratch DOCX conformance corpus.
- `docs/hld/14-development-backlog.md`, F-257.

## Approach

Complete the existing concrete `Table` API with checked setters for auto,
fixed twip, and percentage width, alignment, indentation, fixed or autofit
layout, shading, individual and aggregate borders, default cell margins,
column grid widths, and table-look flags. Reuse the existing OXML table types
and staged document mutation boundary. Represent an invisible border as an
explicit `nil` or `none` edge selected by the caller, never by deleting the
edge. Validate percentages, lengths, colors, column coverage, and overflow
before publishing any mutation.

## Rejected alternatives

- Expose `CT_TblPr` from the facade. That makes callers responsible for schema
  order and cross-property invariants.
- Infer invisible borders from a missing edge. Absence and an explicit
  invisible edge have different Word semantics.
- Add a forwarding table builder. The existing mutable handle already owns the
  operation and a wrapper would add another place to inspect.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| differential | `m23_layout_and_data_tables_match_word` | Sanitized tables match the pinned Word XML semantics, geometry, pagination, and deterministic rendering. |
| round-trip | complete table properties | Every authored table property reopens with the same typed value and correct schema order. |
| regression | explicit invisible borders | Invisible edges remain explicit through save and reopen. |
| failure | checked table setters | Invalid measurements, colors, spans, and overflow leave document bytes unchanged. |

The **test gate** is the differential test named in the backlog.

## HLD impact

- `docs/hld/02-scope-and-non-goals.md`
- `docs/hld/04-opc-and-packaging.md`
- `docs/hld/08-rendering-spec.md`
- `docs/hld/10-bindings-spec.md`
- `docs/hld/12-testing-strategy.md`
- `docs/hld/14-development-backlog.md`

## Risk routing

- Unit conversion. Retain the pinned truncating constructors, reject invalid
  caller values before conversion, and declare any geometry hash delta.
- Any parser or serialiser. Verify table property sequence, prefix-tolerant
  reads, fixed-prefix writes, and exact unmodelled subtree preservation.
- Layout. Use deterministic font mode for every geometry and raster baseline.
- Public API of a published crate. State the additive pre-1.0 surface, inspect
  rustdoc and API changes, and run package dry-runs and archive-size checks.
- External oracle comparison. Pin Word, LibreOffice, and Poppler identities and
  record the structural and raster tolerances.

## Hash harness

Expected changes are limited to samples that opt into newly authored table
properties. Each changed entry must be attributed to this story before any
baseline update.

## Implementation checklist

- [x] Add sanitized failing table property and layout differentials.
- [x] Complete checked table width, layout, appearance, border, margin, and grid setters.
- [x] Preserve explicit invisible edges and unrelated raw property slots.
- [x] Verify public-only construction, reopen, geometry, and failure atomicity.
- [x] Run parser, layout, public API, oracle, hash harness, full verification, and microscope gates.

## Open questions

None.
