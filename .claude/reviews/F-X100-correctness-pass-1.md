# F-X100, correctness, pass 1

**Reviewed**: the two-file, 89-line F-X100 working diff in
`crates/rdocx-oxml/src/properties.rs` and `crates/rdocx-oxml/src/table.rs`, plus
the adjusted repeated-row expectation in the combined facade test
**Verdict**: 0 defects, 0 smells, 0 nitpicks

## Defects

None.

## Smells

None.

## Nitpicks

None.

## Not found

No correctness, contract, panic, OOXML order, test, or structure findings were
found. All three properties use the shared namespace-aware on-off parser, so
bare values remain true and `0`, `false`, and `off` remain false. Both values
serialize through the existing canonical toggle writer at the same row and cell
property schema slots. Absence remains `None`, foreign same-local-name elements
remain raw, and existing true and ordered-extra coverage remains applicable.
