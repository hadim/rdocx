# F-252, all aspects, pass 1

**Reviewed**: the 5-file working diff, 1,115 added lines and 16 removed lines
**Verdict**: 3 defects, 0 smells, 0 nitpicks

## Defects

### D1, create does not enable an existing first-page story

`crates/rdocx/src/document.rs:14950`

`create_section_story` returns immediately when the section already has a
direct story. For an imported section with a usable `First` reference and
`titlePg` absent or false, this path leaves first-page behavior disabled. The
design contract says first-page creation enables `titlePg`, and the other
installation paths do so through `set_direct_section_story_reference`.

### D2, even stories have no public operation that enables their selection

`crates/rdocx/tests/regression_test.rs:67`

The differential fixture must reopen a privately patched OPC package to add
`w:evenAndOddHeaders`. The public API can create, link, replace, and remove an
`Even` story, but it cannot enable or disable the document-wide selection
setting. This leaves the design's explicit-operation decision unavailable to
the caller and makes a newly authored even story impossible to select through
the public facade alone.

### D3, the Word oracle leaks its artifact directory on failure

`crates/rdocx/tests/regression_test.rs:286`

The ignored oracle creates its unique directory before many fallible assertions
and removes it only at the successful end of the test. Any Word, Poppler,
parsing, or record assertion failure leaves both source and PDF artifacts in
the Word container. This occurred during pass 1 when the initial `pdfinfo`
parser rejected the valid per-page label. The oracle needs unwind-safe cleanup.

## Smells

None.

## Nitpicks

None.

## Not found

No additional correctness, contract, panic, OOXML, test, or structure findings
were found. The staged mutation boundary, same-type inheritance, complete
part-local relationship copying, collision-safe allocation, schema placement,
rich-content lifecycle checks, and facade-owned pruning were reviewed.
