# F-X116, all, pass 1

**Reviewed**: working-tree diff, 15 files, 1,374 changed lines
**Verdict**: 2 defects, 1 smell, 0 nitpicks

## Defects

### D1, a revision nested in a content control does not serialize its edit

`crates/rdocx-oxml/src/content_control.rs:747`

The recursive path mutates the typed `CT_Revision`, but `write_content` still
serializes the unchanged `SdtContent::RawXml` revision wrapper at
`crates/rdocx-oxml/src/content_control.rs:983`. A run under
`w:sdt/w:sdtContent/w:ins` therefore reports its new value in memory and loses
that value after save and reopen. The contract promises recursive source paths
through both accepted revisions and inline content controls.

### D2, a failed revision refresh leaves the live paragraph partly mutated

`crates/rdocx-oxml/src/revision.rs:233`

`replace_accepted_run_segments` changes `content_paragraph` before calling the
fallible refresh at line 236. A serialization failure, such as changing one
projection of a complex field that shares a physical run, returns an error but
leaves the typed accepted view changed while `raw_xml` remains unchanged. The
next read and the next save can then disagree. Mutation must stage the parsed
paragraph and publish both representations only after refresh succeeds.

## Smells

### S1, the binding gate does not exercise a recursively composed owner path

`crates/rdocx-py/tests/test_core.py:219`

The test places an insertion and a content control beside each other. It proves
one-segment paths of each kind, but it cannot fail when revision mutation inside
a content control, or content-control mutation inside a revision, is broken.
The save and reopen assertion needs at least one path containing both owner
segment kinds.

## Nitpicks

None.

## Not found

No additional contract, panic, OOXML child-order, namespace, performance, or
structure findings.
