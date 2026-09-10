# F-250, all, pass 2

**Reviewed**: complete uncommitted working-tree diff against `HEAD`, 2 files,
697 additions and 1 deletion
**Verdict**: 1 defect, 1 smell, 0 nitpicks

## Defects

### D1, malformed references can still displace an inherited valid story
`crates/rdocx/src/document.rs:12619`
`crates/rdocx/src/document.rs:12633`
`crates/rdocx/src/document.rs:14703`

The remediation computes inheritance from every typed reference without
checking that its relationship is internal, has the header or footer type
required by its owner vector, and reaches a usable target. It also overwrites
the effective slot for every duplicate, so the last same-variant reference wins.
The existing story resolver instead rejects external and cross-type
relationships and resolves only eligible story targets.

For example, section 0 can own valid default header A while the following
section has a default `headerReference` whose id names a footer relationship.
Before removal, that malformed reference is ineligible and the following
section inherits A. During removal, the `any` check treats the malformed entry
as an explicit default header, declines to materialize A, and pruning can delete
facade-owned A after section 0 disappears. Similarly, a later malformed or
duplicate entry on the removed chain overwrites an earlier eligible entry in
the effective array. The materialization pass must use the same first eligible
same-type relationship and target semantics as normal header and footer
resolution, both while building the effective chain and while deciding whether
the following section already overrides a variant.

## Smells

### S1, removing the wrappers also removes the approved facade handles
`.claude/plans/F-250-design.md:28`
`crates/rdocx/src/document.rs:12487`

Pass 1's forwarding-only wrappers are gone, but the replacement exposes
`CT_SectPr` directly from all four ordered accessors. The approved approach and
the facade convention require concrete immutable and mutable section handles.
This also makes the new public section surface indistinguishable from exposing
the wire model itself. Resolve the conflict by giving the handles behavior that
justifies them under the structural rules, or amend the approved contract
before completing the feature.

## Nitpicks

None.

## Not found

Pass 1 D1 is fixed for valid unique header and footer references. Pass 1 S2 is
fixed by assertions over each surviving reference's variant, relationship
type, target, and parsed story text. Ordinal behavior, insertion, final-owner
promotion, staged failure atomicity, raw section XML preservation, empty
facade-boundary cleanup, shared target retention, producer-owned story
retention, schema child order, panic safety, and arithmetic hazards produced no
additional findings.

## Checks

- `cargo test -p rdocx --test integration_test section_`, passed 12 tests.
- `cargo test -p rdocx --test integration_test removing_a_predecessor_materializes_inherited_header_and_footer_references -- --exact`, passed.
