# F-X096, release, pass 1

**Reviewed**: Working-tree diff against `11339240`, 19 files, 594 insertions and 376 deletions
**Verdict**: 0 defects, 0 smells, 0 nitpicks

## Defects

None.

## Smells

None.

## Nitpicks

None.

## Not found

Correctness, contract, panic, release-authority, artifact-selection, workflow,
tests, documentation, and structure checks produced no findings. The two tag
families select independent native versions, non-selected matrix cells cannot
upload artifacts, manual dispatch cannot reach publication, and the publish
job validates exact filenames and embedded metadata before receiving PyPI OIDC
authority.
