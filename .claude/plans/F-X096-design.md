# F-X096, Align Python distribution versions and release tags

**Status**: completed
**Sprint**: S72
**Size**: M
**Depends on**: F-X094e

## Problem

`crates/rdocx/Cargo.toml` and the `rdocx-py` package are 0.13.1, while
`crates/rpptx/Cargo.toml` is 0.11.0 and `rpptx-py` currently inherits 0.13.1.
The prepared paired `py-v0.13.1` contract would therefore publish PyPI
`rpptx` at a version that does not match its native crate. The user requires
each PyPI distribution to match its corresponding crate.

The obvious `rpptx-v*` Python tag is already the immutable incubating Rust
release namespace in `.github/workflows/publish.yml`. Reusing it could start
crates.io publication or target an old SHA. The Python namespaces must remain
disjoint from both Rust workflows.

## Spec reference

- `docs/hld/10-bindings-spec.md`, Python packaging, CI, and version carriers.
- `docs/hld/03-architecture.md`, native and binding version-family boundaries.
- `docs/hld/12-testing-strategy.md`, Python wheel and release-version gates.
- `docs/hld/13-risks-and-open-questions.md`, PyPI authority and partial-release
  risk.
- `docs/hld/14-development-backlog.md`, "F-X096, Align Python distribution
  versions and release tags".
- `docs/hld/15-build-and-toolchain.md`, release tag families, trusted PyPI
  publication, artifact verification, and version synchronization.
- `.claude/commands/release.md` and `release-notes.md`, canonical release
  ceremony and notes contract.

## Approach

Keep `rdocx-py` and PyPI `rdocx` at native `rdocx` version 0.13.1. Give
`rpptx-py` an explicit package version 0.11.0 and set PyPI `rpptx` to 0.11.0,
matching native `rpptx`. Update every tested version carrier and installed
module assertion.

Replace the paired Python tag with two unambiguous families:
`py-rdocx-vX.Y.Z` and `py-rpptx-vX.Y.Z`. A tag publish selects exactly one
distribution, six cp39-abi3 wheels, and one source distribution. Manual
dispatch remains build-only and may build both matrices for preflight evidence.
Keep `v*` and `rpptx-v*` exclusive to their existing Rust families.

Teach release parsing, artifact validation, release-note selection, trusted
publisher checks, and workflow mutation tests about the selected Python
distribution. Prepare separate reviewed changelog sections for
`py-rdocx-v0.13.1` and `py-rpptx-v0.11.0`. Each `/release` invocation retains
its own final approval, exact tag, artifacts, PyPI verification, GitHub release,
and notification evidence.

Do not introduce a new workflow or script. Extend the existing wheel workflow,
release commands, state validator, and test module.

## Rejected alternatives

- Publish both Python projects at 0.13.1. That contradicts native `rpptx`
  0.11.0 and the user's version-alignment requirement.
- Reuse `rpptx-v0.11.0`. That tag is an existing Rust release at a different
  immutable SHA.
- Use bare `rdocx-v*` and `rpptx-v*`. The latter collides with crates.io
  publication and makes registry intent ambiguous.
- Publish with tokens or manual uploads. Trusted tag-only OIDC remains the
  authority boundary.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| release preparation | `python_release_contract_keeps_distribution_versions_independent` | Native crate, binding crate, project metadata, artifact, tag, and PyPI versions agree for rdocx 0.13.1 and rpptx 0.11.0. |
| workflow | Python tag-routing mutation matrix | Each Python tag publishes only its matching six wheels and one sdist, while manual dispatch and Rust tags cannot publish to PyPI. |
| artifact | Single-distribution artifact validator matrix | Missing, extra, mixed-family, wrong-version, wrong-ABI, and wrong-platform artifacts fail closed. |
| install | Clean Python 3.9 and 3.12 environments | Each selected distribution installs at its exact native version and passes runtime, typing, and stub checks. |

The **test gate** is release preparation:
`python_release_contract_keeps_distribution_versions_independent` plus its
workflow and artifact mutation matrices.

## HLD impact

- `docs/hld/10-bindings-spec.md`
- `docs/hld/03-architecture.md`
- `docs/hld/12-testing-strategy.md`
- `docs/hld/13-risks-and-open-questions.md`
- `docs/hld/14-development-backlog.md`
- `docs/hld/15-build-and-toolchain.md`

## Risk routing

- **Release scripting or version strings**. Inspect every manifest and version
  carrier, require a clean full gate and review, and stop for separate final
  approval immediately before each tag's first external mutation.
- **WASM or PyO3 bindings**. Verify binding exclusions, both WASM targets,
  typing, stubs, and clean abi3 installs at each selected version.
- **CI workflow or toolchain policy**. Read HLD 15 and mutation-test tag
  routing, artifact selection, permissions, environment, and failure
  propagation.
- **Generated command adapters**. Regenerate from canonical command sources
  and run the adapter drift gate.

No parser, serializer, dependency-direction, layout, font, unit, colour,
module, file-move, or external-oracle row is triggered.

## Hash harness

Expected unchanged across all 49 entries.

## Implementation checklist

- [x] Align `rdocx-py` with native `rdocx` 0.13.1.
- [x] Align `rpptx-py` with native `rpptx` 0.11.0.
- [x] Add disjoint `py-rdocx-v*` and `py-rpptx-v*` release parsing.
- [x] Route each Python tag to exactly one wheel and source distribution set.
- [x] Keep manual dispatch build-only and both Rust tag families PyPI-free.
- [x] Validate one selected distribution's exact artifact inventory.
- [x] Split release notes, PyPI preflight, owner checks, and notifications by distribution.
- [x] Add positive and negative workflow, metadata, and artifact tests.
- [x] Synchronize generated adapters and update exactly the listed HLD files.
- [x] Run the full release, binding, WASM, package, and unchanged-harness gates.

## Open questions

None. The user explicitly requires crate and PyPI version alignment and asked
for the repository's standard safe publication flow.
