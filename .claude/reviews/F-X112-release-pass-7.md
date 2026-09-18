# F-X112, release, pass 7

**Reviewed**: recovery delta `54f4567b..5a33b2a6`, 58 files and 835 changed
lines, together with the complete four-family release contract prepared by
F-X112 at the current sprint head
**Verdict**: 0 defects, 0 smells, 0 nitpicks

## Defects

None.

## Smells

None.

## Nitpicks

None.

## Not found

- Release scope: the exact 15-package shared and PowerPoint family is prepared
  at 0.12.1, the seven-package stable family remains at 0.14.0, and the two
  Python distributions match their native families. Binding and WASM crates
  retain their no-publish policy.
- Release order: `rpptx-v0.12.1` precedes `v0.14.0`, followed by
  `py-rdocx-v0.14.0` and `py-rpptx-v0.12.1`. Packaged stable dependencies
  require shared 0.12.1, so the registry graph has no unpublished edge when
  stable publication starts.
- Immutable recovery: `rpptx-v0.12.0` remains at reviewed SHA `54f4567b`.
  Failed workflow run `35268763196` published no crate and created no GitHub
  release. The recovery neither moves nor deletes that tag.
- Workflow correctness: CRLF-to-LF normalization applies only to README and
  licence comparison. Archive names, exact members, executable mode, nonempty
  binaries, content equality after normalization, SHA-256 generation, family
  predicates, dependency ordering, and publication failure propagation remain
  enforced.
- Release notes: `release-notes --check` passes for all four requested tags.
  Each rendered section is family-scoped and contains reviewed highlights,
  additions, fixes, compatibility guidance, linked records, and authenticated
  contributor credit.
- Contribution inventory: the prepared notification map contains exactly 51
  records, issues 69, 72 through 76, 83 through 86, 88 through 100, and 115
  through 121, plus pull requests 71, 77 through 80, 101 through 114, 122, and
  123. Every entry names its selected families, authenticated handle, and
  direct or hardened landing classification.
- Registry and tag absence: crates.io returned not found for every selected
  0.12.1 and 0.14.0 crate, PyPI returned not found for `rdocx 0.14.0` and
  `rpptx 0.12.1`, and origin returned no requested recovery tag.
- Test and package gates: all four release-contract regressions pass. The full
  workspace gate, 49-entry hash harness, 22-package patched dry run,
  archive-size check, cargo-deny, WASM graphs, docs, README inventories, and
  clean-source Python 3.9 and 3.12 binding rider pass at the recovery source
  snapshot.
- Structure and public surface: the recovery adds no trait, generic, crate,
  module, feature flag, wrapper, or public API. It changes only release
  validation, version carriers, exact notes, and delivery records required by
  the approved recovery.
