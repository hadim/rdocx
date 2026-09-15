# F-X099, correctness, pass 1

**Reviewed**: the five-file, 89-line combined F-X099 working diff, excluding
unrelated F-X097 and F-X100 hunks in `crates/rdocx/src/document.rs`
**Verdict**: 1 defect, 0 smells, 0 nitpicks

## Defects

### D1, the binding gate does not prove the approved ownership matrix

`crates/rdocx/tests/regression_test.rs:16661`

The native regression covers a paragraph containing a field and drawing plus a
second paragraph. It does not include a direct table or a nested content
control. The Python assertion at `crates/rdocx-py/tests/test_core.py:444` uses
only four direct body paragraphs and the final section properties node. It does
not exercise any nested item mapping. The approved test contract requires
native and installed Python coverage for direct paragraphs and tables plus
nested fields, drawings, and content controls. A regression in those untested
ownership cases could pass the named gate.

## Smells

None.

## Nitpicks

None.

## Not found

No additional correctness, contract, panic, OOXML, or structure findings were
found. The accessor resolves the checked flat story item first, then selects
the containing direct body span without changing `index_path`. Non-body stories
return `None`, and the Python constructor, property, snapshot, and stub agree
on the optional integer type.
