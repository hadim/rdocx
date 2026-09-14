# F-X094d, all aspects, pass 1

**Reviewed**: Uncommitted working tree, 15 files, 1,328 additions, 17 deletions,
1,345 changed lines
**Verdict**: 2 defects, 0 smells, 0 nitpicks

## Defects

1. `crates/rdocx-py/src/document.rs:915` inventories hyperlinks by scanning every
   story item independently, while `crates/rdocx/src/document.rs:6626` excludes
   only nested table cells and text boxes. A nested block content control is also
   emitted as its own story item, so a hyperlink inside it is returned once for
   the ancestor content-control item and again for the nested item. This breaks
   the promised exact source inventory and assigns one physical hyperlink to two
   index paths. Suppress hyperlinks owned by nested modeled content controls when
   scanning an ancestor item, and add a nested-control regression to the Python
   snapshot gate.

2. `crates/rdocx-py/tests/test_core.py:35` places `w:type` after `w:pgSz`,
   `w:pgMar`, `w:cols`, `w:pgNumType`, and `w:titlePg` in the first section, with
   the same ordering error at lines 41 and 49. `CT_SectPr` requires `w:type`
   before those modeled children. The fixture therefore injects schema-invalid
   OOXML and relies on parse and serialization to repair it before the assertions
   run. Reorder each fixture section to match the schema sequence so the gate
   starts from a valid package.

## Smells

None.

## Nitpicks

None.

## Not found

Contract coverage, panic paths, XML preservation, public typing, and structural
rule review produced no additional findings.
