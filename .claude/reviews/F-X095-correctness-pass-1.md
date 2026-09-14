# F-X095, correctness, pass 1

**Reviewed**: `0d76e972..4e961942`, 24 files, 1,015 insertions and 218 deletions
**Verdict**: 2 defects, 0 smells, 0 nitpicks

## Defects

### D1, later section-property owners are silently discarded

`crates/rdocx-oxml/src/document.rs:1531`

The body reader parses every direct start-and-end `w:sectPr`, but stores only
the first and drops the complete parsed value for every later occurrence. The
self-closing path does the same at line 1552. A producer document containing a
duplicate or misplaced later section owner therefore loses that subtree on the
next typed serialization. Retaining the first typed section is the intended
reader fact, but later owners must remain opaque body content or make the input
fail closed. They cannot disappear.

### D2, start-and-end section leaves discard unsupported attributes

`crates/rdocx-oxml/src/document.rs:472`

The new start-and-end handling decides whether `pgSz`, `pgMar`,
`headerReference`, and `footerReference` can be modeled by checking only child
content. It then discards the captured raw subtree. Unsupported attributes are
therefore lost. For `pgMar`, line 603 also parses every attribute value as a
number before checking its expanded name, so a harmless foreign string
attribute rejects the document. These forms must be modeled only when their
complete attributes and content satisfy the typed grammar. Otherwise the
captured namespace-complete subtree must stay opaque in its schema slot.

## Smells

None.

## Nitpicks

None.

## Not found

No additional findings in contract scope, panic safety, OOXML child order,
namespace-aware relationship identity, tests, public API shape, or structural
rules.
