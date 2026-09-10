# F-254, all, pass 5

**Reviewed**: exact current uncommitted working-tree implementation diff on
`work/f-254-codex`, excluding review artifacts, 5 files, 1,826 insertions and
14 deletions
**Verdict**: 1 defect, 0 smells, 0 nitpicks

## Defects

### D1, Rust whitespace admits XML-forbidden character references
`crates/rdocx/src/document.rs:4385`

The GeneralRef fix resolves a numeric reference and accepts it whenever Rust's
`is_ascii_whitespace` predicate returns true. quick-xml explicitly does not
enforce the XML Legal Char constraint in `resolve_char_ref`. Direct `&#11;`
and `&#12;` therefore resolve to vertical tab and form feed, which Rust treats
as ASCII whitespace even though neither character is legal in XML 1.0. The
content-control parser preserves the reference, so
`ContentFragment::content_control` accepts a checked block fragment containing
XML-forbidden content. The same overbroad predicate is used for direct text and
CDATA at `crates/rdocx/src/document.rs:4339` and
`crates/rdocx/src/document.rs:4349`. These checks need the four XML whitespace
characters and must reject every other XML-forbidden character.

## Smells

None.

## Nitpicks

None.

## Not found

Pass-4 D1 is otherwise closed. Decimal and hexadecimal references,
predefined entities, malformed numeric references, and unresolved entity
names take explicit non-panicking paths. Valid XML whitespace references are
accepted at the direct `w:sdt` and `w:sdtContent` depths. Non-whitespace
references are rejected at both depths. References inside foreign raw
subtrees remain opaque and unchanged. The scanner does not expand arbitrary
entities.

No additional defects were found in the XML element stack, namespace scope
transitions, block-control slot order, nested-control recursion, canonical
flattened destination anchors, explicit end locations, self-closing owner
expansion, body section-properties placement, identity or relationship
rewriting, bookmark nesting, staged atomicity, or reopen publication. No
additional panic, public API compatibility, OOXML preservation,
test-sensitivity, or repository structural-rule findings were found. The full
`rdocx` regression binary passed with 421 tests passed and 4 ignored.
