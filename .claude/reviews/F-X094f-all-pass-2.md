# F-X094f, all, pass 2

**Reviewed**: working-tree corrective release diff, 34 files, 683 insertions,
163 deletions
**Verdict**: 2 defects, 0 smells, 0 nitpicks

## Defects

### D1, the rdocx PyPI page advertises output formats the binding does not expose

`crates/rdocx-py/README.md:47`

The Python `Document` surface exposes PDF, PNG, JPEG, and TIFF output, but no
SVG, HTML, or Markdown method. Listing those formats on the Python
distribution page overstates what an installed `rdocx` package can do. Limit
the capability to formats reachable through the public Python binding.

### D2, artifact validation accepts an unreviewed long description

`scripts/sprint_workflow.py:765`

The validator checks six marker strings but never compares the artifact payload
with the selected crate-local README. A stale or substantially replaced
description containing those headings passes even though the release contract
requires the reviewed README. Compare the complete embedded payload with the
selected source README and add a mutation that changes prose while retaining
all marker strings.

## Smells

None.

## Nitpicks

None.

## Not found

No additional correctness, contract, panic, OOXML, test, or structure findings.
The version carriers remain aligned at rdocx 0.13.2 and rpptx 0.11.0. The
dynamic README metadata builds valid wheels from both source distributions,
and the external release boundary remains fail closed.
