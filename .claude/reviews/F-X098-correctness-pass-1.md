# F-X098, correctness, pass 1

**Reviewed**: the one-file, 149-line F-X098 working diff in
`crates/rdocx-oxml/src/content_control.rs`
**Verdict**: 0 defects, 0 smells, 0 nitpicks

## Defects

None.

## Smells

None.

## Nitpicks

None.

## Not found

No correctness, contract, panic, OOXML order, test, or structure findings were
found. The first supported type retains its unmodelled attributes and child
bytes, while duplicate type elements continue through the ordered raw slots.
An unchanged discriminator reuses the payload under the fixed output prefix.
A changed discriminator emits one canonical empty replacement at the original
slot. The table-driven regression covers both WordprocessingML and extension
namespace type elements with attributes and children.
