# F-X105, all aspects, pass 1

**Reviewed**: working-tree diff, 8 tracked files, 304 inserted lines and 60 deleted lines
**Verdict**: 2 defects, 0 smells, 0 nitpicks

## Defects

### D1, The HLD states two incompatible ancestor-selection rules

`docs/hld/07-inheritance-and-resolution.md:290`

The first rule says an absent source container selects the deepest eligible
layout or master shape. The later rule says an occupied but hidden layout shape
claims the type and prevents the master shape from appearing. Both cannot be
true for an occupied layout date without `p:hf` above an enabled master date.
The prose must state the implemented and oracle-backed shadowing rule once.

### D2, The promised empty-placeholder matrix is absent

`crates/rpptx-layout/src/context.rs:4907`

The source-specific layout and master matrix builds only occupied date, footer,
and number placeholders. The design contract also requires empty slide,
layout, and master variants. Existing nearby coverage checks one empty-layout
fallback indirectly, but it does not prove the complete source and type matrix
or that an empty slide placeholder leaves the inherited occupied value
eligible. Add focused empty cases before completion.

## Smells

None.

## Nitpicks

None.

## Not found

No additional correctness, contract, panic, OOXML, test-gate, or structure
finding was found. The direct-slide path ignores template flags without adding
a public abstraction. Source order and type-level deduplication remain intact.
The exact named gate fails against the previous shared-policy implementation,
pins LibreOffice and both Poppler tools, uses deterministic Rust fonts, and
checks both extracted text and bounded raster visibility.
