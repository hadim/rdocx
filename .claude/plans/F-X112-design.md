# F-X112, Publish the complete S73 package families

**Status**: completed
**Sprint**: S73
**Size**: L
**Depends on**: F-257 through F-263, F-X097 through F-X111, F-X113 through F-X122

## Problem

The stable Rust crates remain at unpublished 0.13.2 while PyPI rdocx already
serves 0.13.2. The incubating Rust and Python families remain at 0.11.0. S73 is
a milestone release and the user requested every standard publishable Rust
package plus both version-aligned Python distributions, with comments on all
included issues and pull requests, including the later Issue 115 through Issue
121 intake and its contributor discussions.

## Spec reference

- `docs/hld/10-bindings-spec.md`, package and distribution identity.
- `docs/hld/12-testing-strategy.md`, release and installed-artifact gates.
- `docs/hld/14-development-backlog.md`, F-X112.
- `docs/hld/15-build-and-toolchain.md`, exact family allowlists and publication workflow.
- `.claude/commands/release.md`, four separate release ceremonies.

## Approach

Prepare one reviewed S73 SHA for stable Rust and PyPI rdocx at 0.14.0, and
incubating Rust and PyPI rpptx at 0.12.1. Preserve the failed immutable
`rpptx-v0.12.0` tag, which published no registry packages and created no GitHub
release. Update every manifest, internal pin, lock entry, README example,
workflow assertion, package metadata record, and recovery CHANGELOG sections.
Assign all four exact tags to this release F-ID. At the reviewed SHA, execute
four separate `/release` actions with a fresh immediate approval before each
tag: incubating Rust, stable Rust, Python rdocx, and Python rpptx. The
incubating family publishes first because packaged stable crates require the
shared 0.12.1 registry family. Verify all 22
Rust crates, both seven-file Python distributions, both GitHub CLI asset sets,
owners, releases, and human notification comments before completing the F-ID.
The notification inventory includes every issue and pull request incorporated
through F-X122, whether it is open or closed at publication time.
After the verified comments are posted, close each included record that remains
open and whose released outcome fully addresses the record. Do not reopen or
otherwise change records that were already closed. The user explicitly
authorized this post-release state cleanup.

## Rejected alternatives

- Publish stable 0.13.2 as a historical backfill, then publish the milestone
  version. It adds an unnecessary release and Issue 99 is satisfied by a newer
  fixed stable train.
- Use one tag for multiple families. The release command and workflows require
  exact disjoint authority and artifact inventories.
- Publish WASM or binding support crates to crates.io. They are explicitly
  outside the standard publishable sets.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| release preparation | `s73_release_contract_requires_four_version_aligned_families` | Exact versions, 7 stable crates, 15 incubating crates, two Python distributions, four tags, selected assets, and the through-F-X122 notification inventory. |
| package | patched workspace dry-run | All 22 publishable Rust archives stage from the reviewed source graph and remain within size limits. |
| Python | build-only wheel matrix | Each selected distribution has six cp39-abi3 wheels and one source archive with complete metadata. |
| release | registry and notification verification | Every package, owner, release body, CLI asset, issue, PR, and comment URL is verified before completion. |

The **test gate** is the release-preparation test named in the backlog.

## HLD impact

- `docs/hld/03-architecture.md`
- `docs/hld/10-bindings-spec.md`
- `docs/hld/12-testing-strategy.md`
- `docs/hld/14-development-backlog.md`
- `docs/hld/15-build-and-toolchain.md`

## Risk routing

- Release scripting and version strings. Inspect all manifests, lockfile,
  README, workflows, notes, family allowlists, archives, and registry absence.
  Require a clean full gate and a separate final approval immediately before
  each tag.
- WASM or PyO3 bindings. Run both WASM checks, both mixed binding packages,
  pytest, strict mypy, stubtest, six-wheel builds, clean Python 3.9 and 3.12
  installs, and post-PyPI runtime checks.

## Hash harness

Expected behavioral hashes are the reviewed union of S73 stories. Version and
release preparation itself must add no unexplained output delta.

## Implementation checklist

- [x] Approve exact stable and incubating next-minor versions and standard family scope.
- [x] Prepare every version carrier, pin, lock entry, README, workflow assertion, and release note.
- [x] Run the full gate, all release riders, and clean sprint review at one exact SHA.
- [x] Obtain fresh approval and release the incubating Rust family with CLI assets.
- [x] Obtain fresh approval and release the stable Rust family with CLI assets.
- [x] Obtain fresh approval and release PyPI rdocx from its reviewed seven artifacts.
- [x] Obtain fresh approval and release PyPI rpptx from its reviewed seven artifacts.
- [x] Verify registries, owners, releases, assets, notes, and every human issue and PR comment.
- [x] Close each fully addressed included issue and pull request that remains open.

## Open questions

None. The user approved the immutable-tag recovery with rdocx 0.14.0 and rpptx
0.12.1 in matching native and PyPI versions. The standard flow selects the
7-package stable crates.io family, 15-package incubating crates.io family, PyPI
rdocx, and PyPI rpptx. Unpublished WASM and binding support crates keep their
current publication policy.
