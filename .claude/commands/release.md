---
description: Release an already prepared and reviewed Rust or Python package family. The only command that creates and pushes v*, rpptx-v*, py-rdocx-v*, or py-rpptx-v* release tags or starts registry publication.
---

# /release {vX.Y.Z | rpptx-vX.Y.Z | py-rdocx-vX.Y.Z | py-rpptx-vX.Y.Z}

Release the exact reviewed sprint SHA for one package family. This is the only
command allowed to create or push a stable `v*` tag, an incubating `rpptx-v*`
tag, or a Python `py-rdocx-v*` or `py-rpptx-v*` tag, or to start crates.io or
PyPI publication. It never merges to `main` and never creates an `sNN` sprint
tag.

The version preparation is committed through its F-ID before this command
runs. This command does not edit versions, create a release commit, repair a
red gate, or switch a release from one family to the other.

## Family contract

Choose exactly one family from the requested tag before running any check.

### Stable family

For `vX.Y.Z`, `[workspace.package].version` must be exactly `X.Y.Z`. The exact
seven-package stable set is `rdocx-opc`, `rdocx-oxml`, `rdocx-layout`,
`rdocx-html`, `rdocx-pdf`, `rdocx`, and `rdocx-cli`, each at `X.Y.Z` through
workspace inheritance and matching `[workspace.dependencies]` pins.
`rdocx-wasm` may inherit the workspace version, but it remains
`publish = false` and is not in the crates.io set.

### Incubating family

For `rpptx-vX.Y.Z`, every selected package manifest and its corresponding
`[workspace.dependencies]` pin must be exactly `X.Y.Z`. The exact 15-package
incubating set is `oxml-core`, `oxml-opc`, `oxml-media`, `oxml-layout`,
`oxml-drawing`, `oxml-pdf`, `oxml-sml`, `oxml-cli-support`, `oxml-chart`,
`rpptx-oxml`, `rpptx-chart`, `rpptx-layout`, `rpptx-render`, `rpptx`, and
`rpptx-cli`. Stable packages are
not in this set. Binding, WASM, and unimplemented CLI crates remain outside it.

### Python family

For `py-rdocx-vX.Y.Z`, the exact selected distribution is `rdocx` from
`rdocx-py`. Its binding crate and `pyproject.toml` must resolve to the native
`rdocx` crate version `X.Y.Z`. For `py-rpptx-vX.Y.Z`, the exact selected
distribution is `rpptx` from `rpptx-py`. Its binding crate and
`pyproject.toml` must equal the native `rpptx` crate version `X.Y.Z`. Each
artifact set is exactly six `cp39-abi3` wheels, one for each reviewed Linux,
macOS, and Windows target in `wheels.yml`, plus exactly one source
distribution. No crates.io package, WASM package, npm package, Rust release
tag, or unselected Python distribution is in either family.

## Preconditions

Refuse before any tag or push if one check fails:

1. The argument is exactly `vX.Y.Z`, `rpptx-vX.Y.Z`,
   `py-rdocx-vX.Y.Z`, or `py-rpptx-vX.Y.Z`, and the selected family satisfies
   its complete version and package contract above.
   Reject any other prefix, suffix, mixed family, or partial version
   preparation.
2. The current branch is the active `sprint/sNN` branch and the tree is clean.
3. Run `python3 scripts/sprint_workflow.py release-notes <requested-tag>
   --check`, then run `python3 scripts/sprint_workflow.py release-notes
   <requested-tag> --render` and inspect that exact body. The source must be
   the committed `CHANGELOG.md` section headed by the exact tag at the current
   HEAD. The notes must cover only the selected family and contain reviewed
   highlights, additions, fixes, compatibility guidance, and contributor
   credit. Rebuild the selected-family contribution inventory required by
   `/release-notes`. Every included GitHub issue and pull request must appear
   as a direct link in the rendered body, and every authenticated external
   reporter or contributor must receive specific credit for the included
   outcome.
4. The release F-ID assigned to the exact requested tag is `reviewed` in the
   sprint run state, remains `in-progress` in both delivery trackers, and every
   dependency is completed. For `py-rdocx-v0.13.1` and
   `py-rpptx-v0.11.0`, that release F-ID is F-X094f.
5. The latest recorded `/verify --full` passed at the current HEAD with the
   declared hash-harness result.
6. The latest recorded `/sprint-review SNN` is clean at the current HEAD, and
   its review file reports zero blocking findings.
7. At the reviewed HEAD, inspect the selected family metadata. Rust families
   use `cargo metadata --no-deps` to confirm their exact package set, versions,
   publication eligibility, and internal pins. A Python family uses the
   selected binding `Cargo.toml`, its `pyproject.toml`, and the corresponding
   native crate manifest to confirm the exact distribution name, import name,
   matching version, `>=3.9` floor, and `abi3-py39` contract. An unselected
   family must not enter the selected workflow allowlist.
8. For a Rust family, run the exact locally patched `cargo publish
   --workspace --dry-run` command in `/verify` step 10 from the clean tree.
   The 22 patches keep packaged internal dependencies on this reviewed source graph
   instead of the reserved registry placeholders, and they do not enter any
   archive. A dry run uploads nothing. It must stage exactly the 22-package
   union of the two Rust family sets, and every archive must remain below 10
   MiB. The `oxml-layout` archive must contain its complete bundled TTF and
   legal-file inventory. The `rdocx-layout` archive must not duplicate those
   assets. The `rpptx` archive must contain `assets/default.pptx`.

   For the Python family, run the local binding, typing, stub, and clean-install
   gates from `/verify`. Confirm that the manual `wheels.yml` path builds both
   distributions but is build-only. Manual dispatch must create no tag, PyPI
   file, or GitHub release.
   The hosted build-only run cannot precede the sprint-branch push at the
   reviewed SHA, so it is the first post-push release gate and still precedes
   tag creation.
9. `.github/workflows/publish.yml` binds the stable predicate to exactly the
   stable set and the incubating predicate to exactly the incubating set, each
   in dependency order. Every real crates.io publish command is the bare
   verified form `cargo publish -p <package>`, failures propagate, and registry
   waits remain between dependency layers. `.github/workflows/wheels.yml`
   binds Python publication to pushed `py-rdocx-v*` and `py-rpptx-v*` tags,
   builds and publishes exactly the selected distribution's six wheels and one
   source distribution, and gives `id-token: write` only to the `pypi`
   environment publish job. Its manual path is build-only and every action is
   pinned to a reviewed immutable SHA.
10. For a Python family, query PyPI and refuse unless the selected project's
    target version is absent. Verify that the `pypi` environment has a trusted
    publisher entry for the exact selected project name, this repository, the
    `wheels.yml` workflow, and the `pypi` environment, with no long-lived token.
    Record the selected project's authenticated PyPI owner or maintainer roles
    that will be checked again after publication.
11. Fetch the remote release-tag namespaces. The exact requested tag must be
   absent locally and from `origin`. Refuse a conflicting or already-published
   version rather than treating it as success.

## Final approval

Report the exact HEAD SHA, requested tag, selected family, selected package or
distribution set, artifact inventory, version, remote, workflow that will run,
and notes source as
`CHANGELOG.md` under the exact tag heading. Include the rendered notes in the
review. Also report the exact included issue and pull-request URLs,
authenticated contributor handles, direct versus hardened-equivalent
classification, verified registry owners, trusted publisher identity, and the
comment that will be posted to each record after a successful release. Ask for
a separate explicit go or no-go immediately before the first external
mutation. Approval given earlier in the feature or sprint does not count at
this boundary.

## Release

After approval, preserve this order:

1. Push the active `sprint/sNN` branch at the reviewed HEAD.
2. For the Python family, manually dispatch `wheels.yml` at that exact branch
   and SHA, then wait for the successful build-only run. Download its artifacts
   and select only the requested distribution's six wheels and one source
   distribution. Run `python3
   scripts/sprint_workflow.py python-release-artifacts <requested-tag>
   <download-directory>` to inspect every wheel and source distribution for
   exact name and version metadata, and every wheel for a `cp39-abi3` tag and
   one reviewed platform. Install the selected distribution in clean Python
   3.9 and 3.12 environments. Run its priority runtime suite in both. Run
   exact `mypy==2.3.0 --strict` checks and stubtest under Python 3.12, because
   that mypy version requires Python 3.10 or newer. Verify again that the manual
   run created no tag, PyPI file, or GitHub release. Stop before tag creation if
   any build-only check fails. Rust families have no action in this step.
3. Create one annotated tag for the requested argument at that exact HEAD with
   message `Release <requested-tag>`.
4. Push only that requested tag. A Rust tag starts
   `.github/workflows/publish.yml`, whose matching predicate publishes only the
   selected Rust family with verification and then creates the GitHub release.
   A Python tag starts `.github/workflows/wheels.yml`, whose tag-only publish
   job publishes only the selected distribution's seven-file artifact set
   through PyPI trusted publishing. Manual dispatch never reaches that job.
5. Watch the workflow through completion. A failed job is a failed release.
   Do not rerun blindly and do not convert an authentication, network,
   compilation, duplicate-version, or registry failure into success.
6. For a Rust family, verify `cargo info <package>@X.Y.Z` for every package in
   the selected set and verify each crates.io owner. For the Python family,
   verify the selected exact project version and all seven files through PyPI,
   reinstall that exact version from PyPI in clean Python 3.9 and 3.12
   environments, rerun its priority runtime suite in both, rerun typing and
   stub checks under Python 3.12, and verify the recorded owner or
   maintainer roles. Then create the GitHub release
   from a freshly rendered reviewed body if the selected workflow did not
   create it. Inspect the release tag and target SHA. Fetch the published
   GitHub release body and compare it byte
   for byte with a fresh `python3 scripts/sprint_workflow.py release-notes
   <requested-tag> --render` result
   from the reviewed SHA. Do not claim an unselected family was published.
7. After publication and release-body verification succeed, notify every issue
   and pull request in the reviewed contribution inventory. Each comment must
   name and link the published tag, summarize the included outcome, state
   whether the work landed directly or through a hardened equivalent, and
   thank the authenticated reporter or contributor. Record every resulting
   comment URL. Do not close, reopen, or otherwise change a record's state
   unless the release F-ID explicitly authorizes that separate action.

If the branch push succeeds but tag push fails, report that exact state. If the
tag push succeeds but publication fails, retain the tag and report the failed
package and workflow. Do not delete or move a published release tag.
If publication succeeds but a contributor notification fails, retain the tag
and release, report the exact record and error, and leave the release F-ID
in-progress until the missing notification is posted and verified.

## Finalise the release F-ID

Only after every selected registry version, required artifact, owner, and the
matching GitHub release are verified:

1. Create the F-ID's `AS_BUILT.md` entry with the release evidence, the complete
   included issue and pull-request inventory, verified contributor handles,
   and every notification comment URL.
2. Complete its sprint tracker and backlog records, clear its owner, and set
   its design plan to completed.
3. Record the release F-ID completed in sprint state.
4. Re-run the sprint's ledger checks and continue `/run-sprint` to its final
   review and `/close-sprint` handoff.

## Refused situations

- A version bump or uncommitted change is still required.
- Verification or sprint review covers a different SHA.
- The requested tag and prepared family do not match exactly.
- The selected Python project version already exists on PyPI, the build-only run is
  missing or covers another SHA, or the artifact inventory is partial.
- PyPI trusted publisher identity or the recorded post-publication owner check
  is absent, ambiguous, or backed by a long-lived token.
- The exact reviewed release-note section is missing, invalid, or differs from
  the published GitHub release body.
- An included issue or pull request is unlinked, a verified external
  contributor is uncredited, or the post-release notification list is
  incomplete.
- A local dry-run is offered as a substitute for successful publication.
- Any command would merge to `main`, create an `sNN` tag, or create both
  release-family tags.
- The user has not given the separate final approval.
