# F-X114, all, pass 1

**Reviewed**: working diff against `eef191c3`, 3 files, 446 insertions and 21 deletions
**Verdict**: 2 defects, 0 smells, 0 nitpicks

## Defects

### D1, Extreme section values can panic TOC rebuild

`crates/rdocx/src/field.rs:5815`

The section text-width calculation subtracts three parsed `i32` values with
unchecked arithmetic. A document can supply an extreme negative margin, so a
debug build panics while rebuilding the TOC instead of falling back safely.
Use checked arithmetic and retain the documented default for an invalid width.

### D2, The contracted style-preservation gate is absent

`crates/rdocx/tests/regression_test.rs:8312`

The implementation now flushes the styles part while resolving or creating TOC
styles, but the named regression only checks modeled style ids and paragraph
tabs. The design plan requires unrelated styles and unmodelled style children
to remain byte-preserved. A regression in the styles-part preservation boundary
would therefore pass this gate.

## Smells

None.

## Nitpicks

None.

## Not found

Contract scope, OOXML child order, namespace handling, numbering suffix order,
style-name selection, section selection, structural design, and ordinary test
coverage produced no other findings.
