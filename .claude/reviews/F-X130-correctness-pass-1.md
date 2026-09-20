# F-X130, correctness, pass 1

**Reviewed**: uncommitted working tree, 32 files changed with 1,010 insertions and 15 deletions
**Verdict**: 4 defects, 0 smells, 0 nitpicks

## Defects

### D1, Named comparisons are rejected with unbounded comparisons

`scripts/readme_doctests.py:500`

The regular expression rejects every occurrence of `faster than` and `smaller
than`. The approved contract rejects only a bare comparison without a named
subject in the same sentence. A bounded sentence such as `faster than release
0.13 on the named workload` therefore fails even though the design explicitly
permits that shape.

### D2, Recording mode does not read or validate the performance thresholds

`scripts/readme_doctests.py:1504`

Recording mode appends the frozen speed rows directly. It never calls
`performance_thresholds` or `validate_speed_bounds`. If the regression
constants drift, `--record-measurements` still prints stale speed rows and exits
successfully, contrary to the recording-mode contract that requires tier two
threshold reads.

### D3, Calendar-invalid measurement dates pass validation

`scripts/readme_doctests.py:688`

The date gate checks only the `YYYY-MM-DD` character shape. Values such as
`2026-99-99` pass even though they are not ISO calendar dates. The provenance
gate should parse the date, and its mutation test should cover a shaped but
invalid value.

### D4, The named regression gate does not prove examples or packaged README bytes

`scripts/test_sprint_workflow.py:6181`

The story's named regression checks narrative constants and measurement rows,
but it never invokes the validation path that compiles examples and builds the
22 archives for byte comparison. Those checks run through a separate command,
so reverting their connection to the F-X130 evidence contract would leave the
named gate green. The design assigns checked examples and byte-identical
packaged long descriptions to this named gate.

## Smells

None.

## Nitpicks

None.

## Not found

No additional contract, panic, OOXML, test, or structure findings were found.
The diff adds no trait, generic parameter, crate, module, feature flag, or
forwarding wrapper. The README and HLD changes preserve the approved package
boundaries and do not touch OOXML parsing or serialization.
