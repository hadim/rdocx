# F-X092, correctness, pass 1

**Reviewed**: working tree, 6 files, 558 added lines and 66 removed lines
**Verdict**: 1 defect, 0 smells, 0 nitpicks

## Defects

### D1, the gate does not prove that rewritten text matrices preserve visual output

`crates/rdocx/src/document.rs:26156`

The implementation replaces each multilingual run's per-glyph matrices with a
cancelled current transform, a page-coordinate text matrix, and relative glyph
placement. The public regression checks only Poppler text extraction. It never
renders the resulting PDF or compares visual geometry, so an inverted, shifted,
or rotated rich run can pass even though the story requires unchanged glyph
paint and raster geometry. The unchanged 49-entry harness does not close this
gap because its seven documents use the legacy Latin `GlyphRun` path.

## Smells

None.

## Nitpicks

None.

## Not found

No other correctness, contract, panic, OOXML, test, or structure findings.
