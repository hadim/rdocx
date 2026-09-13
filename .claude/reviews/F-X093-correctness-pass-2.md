# F-X093, correctness, pass 2

**Reviewed**: remediated working-tree diff, 6 tracked files, 582 inserted lines and 145 deleted lines
**Verdict**: 1 defect, 0 smells, 0 nitpicks

## Defects

### D1, granular comparison can duplicate a multi-unit exact run

`crates/rdocx/src/comparison.rs:2631`

The source-aware granular interleaver substitutes the complete original run
when the first aligned unit belongs to an equal run. A run can contribute
multiple units, such as multiple words or a text item beside a drawing. Later
units from that same owner still append their generated replacements because
`exact_run` is reset on every loop iteration. The result duplicates content in
word or character comparison whenever another run in the paragraph changes.
Exact whole-run substitution must be limited to owners that contribute one
unit, or the remaining units must be suppressed.

## Smells

None.

## Nitpicks

None.

## Not found

The pass found no remaining defect in default run-granularity drawing
preservation, package-authoritative revision resolution, atomic publication,
OOXML schema order, tests, or structure.
