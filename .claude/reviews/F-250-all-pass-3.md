# F-250, all, pass 3

**Reviewed**: complete uncommitted working-tree diff against `HEAD`, 3 files,
1,156 additions and 21 deletions
**Verdict**: 0 defects, 0 smells, 0 nitpicks

## Defects

None.

## Smells

None.

## Nitpicks

None.

## Not found

Pass 2 D1 is fixed at `crates/rdocx/src/document.rs:12719` by using the shared
first-usable resolver for both the inherited chain and following-section
overrides. The resolver at `crates/rdocx/src/document.rs:14859` selects the
first matching variant whose relationship is internal and exact-type and whose
target exists, has the correct root, and parses. The regression matrix covers
missing, external, cross-type, missing-target, and malformed-target references,
plus first-usable duplicate order. Valid header and footer inheritance across
default, first, and even slots uses independent state.

Pass 2 S1 is fixed by the concrete handles at
`crates/rdocx/src/document.rs:1778`. They expose document-order identity,
schema-final ownership, orientation state, dimension-normalizing orientation
mutation, and first-page configuration in addition to the underlying
properties, so they are not forwarding-only wrappers. Pass 1 S2 remains fixed
by relationship kind, target, and parsed-content assertions after reopen.

Staged atomicity, final-owner promotion, authored-story pruning, shared-target
retention, producer-owned story retention, raw section XML preservation,
schema child order, ordinal bounds, panic safety, arithmetic hazards, and
structural rules produced no findings.

## Checks

- `cargo test -p rdocx section_removal_`, passed 2 tests.
- `cargo test -p rdocx --test integration_test section_`, passed 12 tests.
- `cargo test -p rdocx --test integration_test removing_a_predecessor_materializes_inherited_header_and_footer_references -- --exact`, passed.
- `cargo check -p rdocx --all-targets`, passed.
- `git diff --check`, passed.
