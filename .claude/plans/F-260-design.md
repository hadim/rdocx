# F-260, Ordered run content authoring

**Status**: completed
**Sprint**: S73
**Size**: L
**Depends on**: F-253

## Problem

`Run` can append text, while `set_text` replaces the complete run content.
There is no complete public authoring surface for tabs, line, page, and column
breaks, drawings, fields, and symbols in one stable mixed-content sequence.
Formatting changes must not reconstruct that sequence or discard non-text
children.

## Spec reference

- `docs/hld/03-architecture.md`, ordered Word run ownership and layout lowering.
- `docs/hld/04-opc-and-packaging.md`, run child sequence and field preservation.
- `docs/hld/05-drawingml-model.md`, WordprocessingDrawing ownership.
- `docs/hld/08-rendering-spec.md`, tabs, line breaking, drawings, and fields.
- `docs/hld/10-bindings-spec.md`, native ordered run facade.
- `docs/hld/12-testing-strategy.md`, round-trip and deterministic render gates.
- `docs/hld/14-development-backlog.md`, F-260.

## Approach

Complete the existing concrete `Run` handle with append operations for tab,
typed line, page, or column break, pre-embedded drawing, typed field, and
Unicode symbol content. Every operation appends one existing `RunContent`
variant at the current end and retains prior content and raw boundaries. Keep
`set_text` as an explicitly replacing operation, and verify every formatting
setter mutates only `w:rPr`. Add checked facade helpers where relationship or
field ownership requires document staging rather than exposing low-level XML.

## Rejected alternatives

- Add a new run-content builder. The mutable `Run` is already the single owner
  and a forwarding wrapper would duplicate its surface.
- Store mixed children as raw XML. Typed order is required by layout, fields,
  comparison, bindings, and later run splitting.
- Make `set_text` preserve non-text items implicitly. That changes the stated
  replacement contract. Additive append methods make caller intent explicit.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| round-trip | `mixed_run_content_reopens_and_renders_in_source_order` | Text, tab, every break, drawing, field, symbol, and trailing text retain exact order after reopen. |
| regression | formatting preserves mixed content | Every run-formatting setter changes only properties and retains all children and raw slots. |
| layout | mixed run placement | Tabs and line, page, and column breaks lower at the expected position with deterministic fonts. |
| failure | staged relationship operations | Invalid drawing or field ownership publishes no partial run or package relationship. |

The **test gate** is the round-trip test named in the backlog.

## HLD impact

- `docs/hld/03-architecture.md`
- `docs/hld/04-opc-and-packaging.md`
- `docs/hld/05-drawingml-model.md`
- `docs/hld/08-rendering-spec.md`
- `docs/hld/10-bindings-spec.md`
- `docs/hld/12-testing-strategy.md`
- `docs/hld/14-development-backlog.md`

## Risk routing

- Any parser or serialiser. Verify `w:r` child order, fixed-prefix writes,
  alias-tolerant reads, and exact unmodelled subtree preservation.
- Layout and line breaking. Use deterministic fonts and review any line or page
  output delta before changing a baseline.
- Public API of a published crate. State the additive methods, inspect rustdoc
  and API changes, and run package dry-runs and size checks.

## Hash harness

Expected changes are limited to samples that opt into the new mixed-run
authoring operations. Every changed entry must name the responsible child.

## Implementation checklist

- [x] Add the failing all-child round-trip and formatting-preservation matrix.
- [x] Add typed append operations on the existing Run handle.
- [x] Add staged facade operations only where package ownership requires them.
- [x] Verify parser order, rendering positions, fields, drawings, and raw slots.
- [x] Run parser, layout, public API, hash harness, full verification, and microscope gates.

## Open questions

None.
