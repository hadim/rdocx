# F-X094f, Prepare the py-v0.13.1 release path

**Status**: approved
**Sprint**: S72
**Size**: M
**Depends on**: F-137, F-138, F-X094a, F-X094c, F-X094d, F-X094e

## Problem

The wheel workflow builds both Python distributions, but the canonical release
commands and state validator do not accept the `py-v*` family. GitHub Issue 76
also asks for PyPI publication. S72 may prepare and verify the ceremony, while
actual tag creation, publication, external comments, and issue closure require
fresh `/release` authority at the final reviewed SHA.

## Spec reference

- `docs/hld/10-bindings-spec.md`, Python packaging and CI.
- `docs/hld/13-risks-and-open-questions.md`, PyPI authority risk.
- `docs/hld/14-development-backlog.md`, F-X094f.
- `docs/hld/15-build-and-toolchain.md`, trusted publication and release matrix.
- `.claude/commands/release.md` and `release-notes.md`, canonical ceremony.

## Approach

Extend the release grammar with `py-vX.Y.Z`. Define the family as `rdocx` and
`rpptx` Python distributions at one version, exactly six cp39-abi3 wheels and
one sdist each. Require a successful build-only workflow run at the reviewed
SHA, absent target versions on PyPI, trusted publisher configuration, exact
artifact metadata, clean installs, installed typing and stub checks, reviewed
release notes, byte-identical GitHub release body, ownership verification, and
contribution evidence. Manual dispatch remains unable to publish. Prepare
`py-v0.13.1` notes and tests, synchronize generated adapters, then stop before
any external mutation.

## Rejected alternatives

- Treat Python as the stable Rust family. That can publish the wrong registry.
- Upload with a local token. Trusted tag-only OIDC is the authority boundary.
- Create the tag during S72. `/release` requires a fresh immediate approval.
- Close Issue 76 after preparation. Both distributions must be live and verified.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| release preparation | `python_release_contract_rejects_partial_or_unapproved_publication` | Exact tag, two distributions, twelve wheels, two sdists, trusted publisher, reviewed notes, verification, and approval. |
| mutation | Python release negative matrix | Family mismatch, manual publication, missing artifacts, moved tag, missing owner, or absent approval all fail. |
| install | clean Python 3.9 and 3.12 environments | Both distributions install together and pass priority runtime, typing, and stub smoke tests. |
| workflow | build-only reviewed SHA | Manual dispatch builds and verifies but cannot publish or create a release. |

The test gate is `python_release_contract_rejects_partial_or_unapproved_publication`.

## HLD impact

- `docs/hld/10-bindings-spec.md`
- `docs/hld/13-risks-and-open-questions.md`
- `docs/hld/14-development-backlog.md`
- `docs/hld/15-build-and-toolchain.md`

## Risk routing

- Release scripting or version strings. Inspect every manifest and carrier,
  require a clean full gate and review, and stop for separate final approval.
- WASM or PyO3 bindings. Verify both mixed distributions, binding exclusions,
  WASM targets, typing, stubs, and clean abi3 installs.
- New command adapters generated from canonical sources. Run the skill sync
  generator and drift gate after command edits.

## Hash harness

Expected unchanged across all 49 entries.

## Implementation checklist

- [x] Add the exact Python family to release commands and validation.
- [x] Harden the wheel artifact, trusted publisher, release-body, and owner checks.
- [x] Add exhaustive negative workflow mutations.
- [x] Prepare reviewed notes and contribution inventory for Issue 76.
- [x] Synchronize generated command adapters.
- [ ] Run full local and hosted build-only verification.
- [x] Stop before tag, push, PyPI, external comment, or issue closure.
- [x] Update exactly the listed HLD files.

## Open questions

None. Actual publication remains a later separately approved `/release` action.
