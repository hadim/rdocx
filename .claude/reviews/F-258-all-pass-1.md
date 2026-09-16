# F-258, all aspects, pass 1

**Reviewed**: working diff, 14 files, 1,337 changed lines
**Verdict**: 3 defects, 0 smells, 0 nitpicks

## Defects

### D1, the declared differential gate has no external oracle comparison

`crates/rdocx/tests/integration_test.rs:6991`

The approved plan classifies this test as differential, but the test constructs
and reopens only an rdocx document. The Word version constant appears only in
an assertion failure message at line 7119. No python-docx tree, sanitized Word
record, or other external result is compared with the candidate. The test can
therefore pass when rdocx writes and reads the same wrong property value, so it
does not prove the sprint's named differential gate.

### D2, the named gate does not cover all six text directions it claims

`docs/hld/12-testing-strategy.md:1783`

The HLD says `m23_nested_rows_and_cells_match_word` authors all six text
directions, while that test authors only
`TopToBottomRightToLeft` at
`crates/rdocx/tests/integration_test.rs:7037`. A separate round-trip test covers
the six enum values, but it does not make the named gate satisfy its documented
contract or compare those values with the oracle.

### D3, the layout regression does not assert the promised layout behaviours

`crates/rdocx/tests/integration_test.rs:7404`

The approved layout test must cover exact and minimum heights, repeated
headers, split policy, vertical text, and wrapping. This test authors only row
height, header, and split values, then checks that two page counts are equal and
greater than one. It never authors vertical text or wrapping, and it does not
inspect row geometry, repeated header content, split placement, or render
pixels. An implementation that ignores all of these properties can still pass.

## Smells

None.

## Nitpicks

None.

## Not found

No additional findings in correctness, panic safety, OOXML namespace and
sequence handling, raw subtree preservation, or structure.
