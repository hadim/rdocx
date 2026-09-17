# F-X122, all aspects, pass 1

**Reviewed**: uncommitted canonical diff from planning commit `6277e06f`, 53 files and 599 changed lines
**Verdict**: 0 defects, 0 smells, 1 nitpick

## Defects

None.

## Smells

None.

## Nitpicks

- `scripts/test_sprint_workflow.py:5055`, the unused historical `v0.12.0`
  release-note helper was renamed to mention 0.12.1 even though it still renders
  and checks `v0.12.0`.

## Not found

- Correctness: newline normalization is limited to CRLF-to-LF conversion for
  README and licence equality, while member, executable, nonempty, checksum,
  and workflow-order checks remain exact.
- Contract: the complete shared and PowerPoint source family, Python and WASM
  carriers, workspace pins, lock entries, examples, release notes, release
  mappings, and notification text move coherently to 0.12.1. Stable packages
  remain at 0.14.0 and require shared 0.12.1.
- Panics: no new production panic, unchecked index, slice, arithmetic, or
  untrusted-input path was introduced.
- OOXML: no parser, serializer, namespace, whitespace, child-order, or
  unmodelled-subtree behavior changed.
- Tests: the named regression failed before implementation, checks CRLF-safe
  reviewed text equality, rejects changed text and removed comparisons, and
  pins every recovery carrier and tag. The 122-test workflow suite, full
  workspace gate, 49-entry hash harness, 22-package dry run, and clean-source
  Python rider all pass.
- Structure: no new trait, generic, crate, module, wrapper, feature flag, or
  public API was introduced.
