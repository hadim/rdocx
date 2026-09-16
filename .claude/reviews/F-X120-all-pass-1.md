# F-X120, all aspects, pass 1

**Reviewed**: the four-file working diff, 211 added lines and 8 removed lines
**Verdict**: 0 defects, 0 smells, 0 nitpicks

## Defects

None.

## Smells

None.

## Nitpicks

None.

## Not found

- Correctness: the signed boundary checks, exact half-away-from-zero rule,
  leading-zero handling, long fractional inputs, and safe integer casts are
  consistent across positive and negative values.
- Contract: the implementation is limited to paragraph `w:line`, retains the
  existing integer path, and canonically serializes the normalized twip.
- Panics: every slice and index is guarded by prior nonempty checks, and all
  arithmetic stays within the validated signed 32-bit magnitude.
- OOXML: namespace aliases remain accepted, sibling spacing attributes retain
  their modeled values, and the existing schema-positioned writer emits the
  canonical integer attribute.
- Tests: the named gate fails at the story baseline and proves public open,
  canonical save and reopen, plus byte-identical deterministic rendering
  against equivalent integer input. Unit tests cover exact and invalid lexical
  boundaries.
- Structure: the change adds one focused private parser helper with no new
  trait, generic, wrapper, module, crate, or feature flag.
