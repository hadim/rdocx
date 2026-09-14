# F-X094f, Prepare the version-aligned Python release paths

**Status**: approved
**Sprint**: S72
**Size**: M
**Depends on**: F-137, F-138, F-X094a, F-X094c, F-X094d, F-X094e, F-X096

## Problem

The wheel workflow builds both Python distributions, but the first prepared
contract incorrectly forced them onto one stable-workspace version. Native
`rdocx` was 0.13.1 while native `rpptx` is 0.11.0. GitHub Issue 76 also asks for
PyPI publication. The published `rdocx 0.13.1` files contain only the short
summary from `crates/rdocx-py/pyproject.toml` because that project metadata does
not reference `crates/rdocx-py/README.md`. The parallel `rpptx` project has the
same omission before its first publication. PyPI release files are immutable,
so S72 must prepare a metadata-complete `rdocx 0.13.2` correction and retain
`rpptx 0.11.0` for its first publication. Actual tag creation, publication,
external comments, and issue closure require fresh `/release` authority at the
final reviewed SHA.

## Spec reference

- `docs/hld/10-bindings-spec.md`, Python packaging and CI.
- `docs/hld/13-risks-and-open-questions.md`, PyPI authority risk.
- `docs/hld/14-development-backlog.md`, F-X094f.
- `docs/hld/15-build-and-toolchain.md`, trusted publication and release matrix.
- `.claude/commands/release.md` and `release-notes.md`, canonical ceremony.

## Approach

Retain the independent `py-rdocx-vX.Y.Z` and `py-rpptx-vX.Y.Z` release
families. Each tag selects one distribution at the exact version of its native
crate, six cp39-abi3 wheels, and one source distribution. Both Python projects
must use their crate-local README as Markdown long description and publish a
specific summary, authors, keywords, Python and topic classifiers, and project
URLs. Their READMEs must provide installation, compatibility, capability,
quick-start, typing, and project-link guidance for the PyPI page. Artifact
validation must reject a wheel or source distribution whose embedded metadata
lacks the reviewed long description or project links.

Bump the stable source family, native `rdocx`, `rdocx-py`, and their exact
version carriers to 0.13.2 so the corrective PyPI version still matches its
native crate. Keep native `rpptx`, `rpptx-py`, and PyPI `rpptx` at 0.11.0.
Prepare `py-rdocx-v0.13.2` and `py-rpptx-v0.11.0` notes and tests. Require a
successful build-only workflow run at the reviewed SHA, an absent target
version on PyPI, trusted publisher configuration, exact artifact metadata,
clean installs, installed typing and stub checks, reviewed release notes,
byte-identical GitHub release body, ownership verification, and contribution
evidence. Manual dispatch remains unable to publish. The immutable
`py-rdocx-v0.13.1` tag, PyPI files, and GitHub release remain unchanged.

## Rejected alternatives

- Treat Python as either Rust family. That can publish the wrong registry.
- Upload with a local token. Trusted tag-only OIDC is the authority boundary.
- Reuse `rpptx-v0.11.0`. That immutable tag already records the Rust release
  and cannot identify a new Python publication at the final S72 SHA.
- Create either tag during preparation. `/release` requires fresh immediate
  approval for the exact selected tag.
- Replace or delete `rdocx 0.13.1`. PyPI release files and the published tag are
  immutable, and the functional release remains valid.
- Publish `rdocx 0.13.2` while native `rdocx` remains 0.13.1. That would break
  the explicit native and Python version-alignment contract.
- Close Issue 76 after preparation. Both distributions must be live and verified.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| release preparation | `python_release_contract_rejects_partial_or_unapproved_publication` | Exact selected tag, matching distribution and native version, six wheels, one sdist, trusted publisher, reviewed notes, verification, and approval. |
| release preparation | `python_release_contract_keeps_distribution_versions_independent` | rdocx 0.13.2 and rpptx 0.11.0 remain independent across crates, project metadata, artifacts, tags, and PyPI. |
| release preparation | `python_release_contract_requires_complete_project_metadata` | Both project files and both embedded artifact formats carry a Markdown README description, specific summary, authors, keywords, classifiers, and project URLs. Full description comparison accepts only platform newline differences and terminal newline count. |
| mutation | Python release negative matrix | Family mismatch, manual publication, missing artifacts, moved tag, missing owner, or absent approval all fail. |
| install | clean Python 3.9 and 3.12 environments | Both distributions install together and pass priority runtime, typing, and stub smoke tests. |
| workflow | build-only reviewed SHA | Manual dispatch builds and verifies but cannot publish or create a release. |

The test gate is `python_release_contract_rejects_partial_or_unapproved_publication`.

## HLD impact

- `docs/hld/03-architecture.md`
- `docs/hld/10-bindings-spec.md`
- `docs/hld/12-testing-strategy.md`
- `docs/hld/13-risks-and-open-questions.md`
- `docs/hld/14-development-backlog.md`
- `docs/hld/15-build-and-toolchain.md`

## Risk routing

- Release scripting or version strings. Inspect every manifest and carrier,
  README version statement, lockfile entry, and release assertion. Require a
  clean full gate and review, and stop for separate final approval.
- WASM or PyO3 bindings. Verify both mixed distributions, binding exclusions,
  WASM targets, typing, stubs, and clean abi3 installs.
- New command adapters generated from canonical sources. Run the skill sync
  generator and drift gate after command edits.

## Hash harness

Expected unchanged across all 49 entries.

## Implementation checklist

- [x] Add both exact single-distribution Python families to release commands and validation.
- [x] Align each binding crate and Python project with its native crate version.
- [x] Harden the wheel artifact, trusted publisher, release-body, and owner checks.
- [x] Add exhaustive negative workflow mutations.
- [x] Prepare reviewed notes and contribution inventory for Issue 76.
- [x] Synchronize generated command adapters.
- [x] Add complete PyPI metadata and user-facing README guidance to both distributions.
- [x] Bump the stable and rdocx Python source carriers to 0.13.2 while retaining rpptx 0.11.0.
- [x] Reject artifacts without the reviewed Markdown description and project links.
- [x] Replace the rdocx release-note target with py-rdocx-v0.13.2.
- [ ] Re-run full local and hosted build-only verification.
- [x] Stop before tag, push, PyPI, external comment, or issue closure.
- [x] Update exactly the listed HLD files.

## Open questions

None. Actual publications remain later separately approved `/release` actions.
