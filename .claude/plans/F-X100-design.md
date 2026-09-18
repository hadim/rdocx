# F-X100, Preserve explicit false table toggles

**Status**: completed
**Sprint**: S73
**Size**: S
**Depends on**: F-253

## Problem

`crates/rdocx-oxml/src/table.rs:986` reads `w:tblHeader` and `w:cantSplit` as
true from presence alone. The cell parser does the same for `w:noWrap` at line
1412. Their writers omit false values, so an explicit false producer value is
either changed to true or erased. PR 101 independently contributes the same
reported correction and must retain contributor credit if the sprint uses the
hardened equivalent already in progress.

## Spec reference

- `docs/hld/04-opc-and-packaging.md`, "Word table widths" and table serialization rules.
- `docs/hld/10-bindings-spec.md`, "Row, cell, run, paragraph, table, and body properties".
- `docs/hld/12-testing-strategy.md`, Word table round-trip coverage.
- `docs/hld/14-development-backlog.md`, F-X100 and PR 101 mapping.

## Approach

Parse each toggle with the existing namespace-aware `w:val` lookup and shared
`ST_OnOff` lexical vocabulary. Serialize present booleans as canonical `w:val`
values in their current schema slots. Absence remains `None`, and a bare element
remains true.

## Rejected alternatives

- Treating false as absence loses direct formatting semantics.
- Preserving only raw bytes would prevent F-258 from authoring these values
  through the typed model.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| round-trip | `explicit_false_table_toggles_remain_false` | `0`, `false`, and `off` remain false for row header, split policy, and cell wrapping after save and reopen. |
| unit | Existing table parser tests | Bare true and absent values retain their current meaning and schema position. |

The test gate is the backlog round-trip test named above.

## HLD impact

- `docs/hld/04-opc-and-packaging.md`
- `docs/hld/10-bindings-spec.md`
- `docs/hld/12-testing-strategy.md`
- `docs/hld/14-development-backlog.md`

## Risk routing

- Any parser or serialiser. Re-read `docs/hld/04-opc-and-packaging.md` and
  `docs/hld/06-presentationml-model.md`. Verify prefix-tolerant reads,
  fixed-prefix canonical writes, schema order, and raw sibling preservation.

## Hash harness

Expected to be unchanged. No sample contains an explicit false form for these
three table toggles.

## Implementation checklist

- [x] Add the failing table-toggle round-trip regression.
- [x] Parse all three toggles through the shared on-off vocabulary.
- [x] Serialize both true and false values canonically in schema order.
- [x] Run focused table tests, hash harness, full verification, and microscope.

## Open questions

None. The user approved an independently completable remediation story.
