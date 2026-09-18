# F-X111, Attach portable CLI binaries to Rust releases

**Status**: completed
**Sprint**: S73
**Size**: L
**Depends on**: F-X095

## Problem

Rust releases publish CLI source crates but attach no executable assets to the
GitHub release. Installing either CLI therefore requires a Rust toolchain and a
full source build. The current release job creates the GitHub release only
after crates.io publication and uploads no files.

## Spec reference

- `docs/hld/10-bindings-spec.md`, CLI product boundary.
- `docs/hld/12-testing-strategy.md`, release workflow mutation tests.
- `docs/hld/14-development-backlog.md`, F-X111.
- `docs/hld/15-build-and-toolchain.md`, pinned hosted runners and release artifacts.

## Approach

Extend the existing Rust publish workflow with a six-target binary matrix for
the selected tag family. Build only `rdocx-cli` for `v*` and only `rpptx-cli`
for `rpptx-v*`. Use native Ubuntu x86-64 and arm64, macOS Intel and arm64, and
Windows runners, plus the reviewed musl toolchain for static Linux x86-64.
Archive exactly one executable with README and licence inventory, aggregate a
SHA-256 checksum manifest, and make GitHub release creation depend on both
crates.io publication and complete assets. Add manifest-local cargo-binstall
URL templates matching the reviewed names.

## Rejected alternatives

- Adopt cargo-dist as the release authority. Its tag and release automation
  overlaps the repository's reviewed `/release` ceremony and family allowlists.
- Upload Python wheels to Rust releases. PyPI and the Python tag workflows
  already own those artifacts.
- Cross-compile every target from one runner. Native runners avoid unreviewed
  linker and system-library substitutions.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| workflow | `rust_release_assets_are_complete_family_scoped_and_installable` | Six targets, exact selected CLI, checksums, ordering, failure propagation, and no Python assets. |
| package | archive inspection | Each archive contains one correctly named executable plus required README and licences. |
| install | cargo-binstall contract | Both manifests resolve their family tag and target archive without a source build. |
| hosted | binary smoke matrix | Every built binary reports the reviewed version and runs help plus a minimal command. |

The **test gate** is the release-preparation test named in the backlog.

## HLD impact

- `docs/hld/10-bindings-spec.md`
- `docs/hld/12-testing-strategy.md`
- `docs/hld/14-development-backlog.md`
- `docs/hld/15-build-and-toolchain.md`

## Risk routing

- Release scripting and version strings. Inspect every manifest and workflow
  diff, mutation-test family routing and failure propagation, and require a
  clean full gate plus separate approval before tagging.
- Public API of a published crate. Inspect cargo-binstall package metadata in
  both archives and run the complete publish dry-run and size gate.

## Hash harness

Expected to be unchanged. The workflow packages existing binaries and changes
no document output.

## Implementation checklist

- [x] Add failing workflow mutation and manifest metadata tests.
- [x] Add selected-family six-target native build and smoke jobs.
- [x] Package exact archives and aggregate checksums.
- [x] Gate release creation on publication and asset completeness.
- [x] Add and test cargo-binstall metadata for both CLIs.
- [x] Run workflow, archive, dry-run, hash harness, full verification, and microscope gates.

## Open questions

None. The repository keeps its current release authority and uses a native
runner matrix rather than cargo-dist.
