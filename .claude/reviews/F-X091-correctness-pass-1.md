# F-X091, correctness, pass 1

**Reviewed**: working-tree implementation, 6 tracked files, 214 added lines and 10 removed lines
**Verdict**: 1 defect, 0 smells, 0 nitpicks

## Defects

### D1, the scope test does not distinguish declaration provenance from URI equality

`crates/rdocx/src/document.rs:20450`

The implementation deliberately treats any nested default declaration as a
shadow, including a declaration that repeats the root URI. The test covers a
different URI and an undeclaration, but not that same-URI case. It also uses an
unclosed element as its malformed case without covering an ambiguous duplicate
default declaration. Add both inputs so a future URI-only walk cannot pass the
contract and duplicate scope remains fail-closed.

## Smells

None.

## Nitpicks

None.

## Not found

No additional correctness, contract, panic, OOXML, public API, CLI atomicity,
or structure defects were found. The exact reporter attachment replaces and
reopens, used inherited defaults remain rejected, and the namespace cache now
tracks the canonical package bytes after publication.
