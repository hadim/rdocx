# F-X094d, all aspects, pass 2

**Reviewed**: Remediated uncommitted working tree
**Verdict**: 1 defect, 0 smells, 0 nitpicks

## Defects

1. `crates/rdocx-py/src/document.rs:913` emits every link owned by an ancestor
   story item before visiting its nested story items, while
   `crates/rdocx/src/document.rs:6622` now removes nested content-control links
   from the ancestor. If a nested content control precedes an ancestor-owned
   hyperlink, the ancestor-owned link is appended first and the physically
   earlier nested link is appended later. This reverses source order, which is
   part of the snapshot contract. The regression at
   `crates/rdocx-py/tests/test_core.py:463` contains no link after the nested
   control, so it cannot detect the interleaving failure. Build the story-wide
   hyperlink inventory from source positions while retaining the most-specific
   owner path, and cover a nested link followed by an ancestor-owned link.

## Smells

None.

## Nitpicks

None.

## Not found

Duplicate ownership, schema child order, panic paths, XML preservation, public
typing, and structural rule review produced no additional findings.
