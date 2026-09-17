# S73 sprint review, pass 6

**Reviewed**: `sprint/s73` at `93e1edc1` against `029065ca3546559f982900260878eab4ac73f1a4`, 253 files, 33,671 changed lines, crates: oxml-chart, oxml-cli-support, oxml-core, oxml-drawing, oxml-layout, oxml-media, oxml-opc, oxml-pdf, oxml-sml, rdocx, rdocx-cli, rdocx-html, rdocx-layout, rdocx-opc, rdocx-oxml, rdocx-py, rdocx-wasm, rpptx, rpptx-chart, rpptx-cli, rpptx-layout, rpptx-oxml, rpptx-py, rpptx-render, rpptx-wasm
**Verdict**: 0 blocking, 0 should-fix, 4 nice-to-have

Bound extension: scheduled dependency-prefix boundary. Passes 1 to 3 closed
the earlier dependency-prefix boundaries clean at `a3be3d1f` and passes 4 to 5
closed the F-X114 record boundary clean at `0e9b425e`, so pass 6 is the first
review of the prepared F-X112 release boundary.

The delta since pass 5 is two commits. `aba94e0c` claims F-X112 and changes
only its BACKLOG and CURRENT_SPRINT rows. `93e1edc1` integrates the F-X112
worker, 66 files with 1,913 insertions and 233 deletions, of which six are its
microscope review files. The integrated tree is identical to the recorded
worker head `0a7f9d57`. The only content difference from the reviewed
`26cf1f7c` is the `ListLevel` literal at `CHANGELOG.md:248`, which applies the
F-X112 pass 6 nitpick exactly. This pass still reviewed the whole integrated
sprint diff, with the closest reading on the release preparation and its
interaction with the behavior the other 33 stories integrated.

F-X112 remains `in-progress` with owner `claude` in both trackers, `reviewed`
in the run state, and `approved` in its plan under the release exception. That
is the intended state and is not a finding.

## Blocking

None.

## Should-fix

None.

## Nice-to-have

### N1, The F-X114 Word records measure preservation rather than Word's TOC generation
`crates/rdocx/tests/regression_test.rs:8498`

Re-evaluated and retained at the same severity. No file under `crates/rdocx`
changed after pass 5, and the ignored capture still
opens the rebuilt document at `crates/rdocx/tests/regression_test.rs:8498` and
saves it at `crates/rdocx/tests/regression_test.rs:8500` without updating the
TOC field. The records prove Word retains rdocx's entries, not that they equal
entries Word would generate. The regression gate proves the contracted behavior
directly, so only the evidence wording is affected.

### N2, A created canonical TOC style uses an uppercase built-in name
`crates/rdocx/src/field.rs:5762`

Re-evaluated and retained at the same severity. The created style is still
named `TOC N`, while the selector compares names case-insensitively, so no
duplicate style appears on a repeated rebuild. No user-visible defect was
found.

### N3, The F-X112 test gate text credits the preparation test with post-publication checks
`docs/hld/14-development-backlog.md:5352`

The gate at `docs/hld/14-development-backlog.md:5352` to
`docs/hld/14-development-backlog.md:5356` says
`s73_release_contract_requires_four_version_aligned_families` proves both
seven-file Python artifact sets, separate tag approvals, registry owners, and
complete issue and pull-request notifications. The test at
`scripts/test_sprint_workflow.py:5422` proves versions, the 7 and 15 package
sets, both `pyproject.toml` versions, the four rendered note sections, the
per-family inventory against backlog and AS_BUILT citations, and the presence
of the wheel and CLI archive wording in the notes. It cannot observe an
approval, an owner, a built artifact, or a posted comment. Those belong to
`/release` and to the plan's separate package, Python, and release rows at
`.claude/plans/F-X112-design.md`, and HLD 12 describes the test accurately.
The text predates F-X112 (`274b8886`, in this sprint). No release can complete
on the test alone because F-X112 stays in progress until `/release` verifies
publication, so this is wording only. A fix would scope the sentence to the
preparation contract and name `/release` for the rest.

### N4, The large-picture release note reads as an encoded file-size change
`CHANGELOG.md:50`

`CHANGELOG.md:50` to `CHANGELOG.md:52` says "pictures larger than 16 MiB
decode within a checked 64 MiB ceiling", and `CHANGELOG.md:474` to
`CHANGELOG.md:476` repeats it for `py-rpptx-v0.12.0`. The code raised only the
decoded-pixel ceiling to 64 MiB at `crates/rpptx/src/lib.rs:8513`. The encoded
image limit stays 16 MiB at `crates/rpptx/src/lib.rs:8511` and still rejects a
larger file at `crates/rpptx/src/lib.rs:8520`. The reported 22.9 MiB and
16.8 MiB pictures are decoded sizes, which the test at
`crates/rpptx/tests/integration.rs:13987` pins with fixtures below 16 MiB
encoded. The sentence is true in that reading, and an over-limit file still
reports a visible diagnostic, so no caller is silently misled. "pictures whose
decoded pixels exceed 16 MiB" would be exact before the body becomes an
immutable GitHub release.

## Milestone gate

The M23 gate at `docs/hld/14-development-backlog.md:2214`:

```text
all five private references are generated from a blank public facade, reopen
without repair, match the required package semantics, and meet the reviewed
deterministic visual thresholds. Repeated generation produces identical DOCX
bytes, and no generated document reports an unexplained preservation-only
fallback.
```

The gate holds for this boundary, on recorded evidence rather than a rerun.
Pass 3 recorded `m23_private_from_scratch_corpus_passes_required_mode` at
`crates/rdocx/tests/integration_test.rs:2768` passing at `a809a244`. Since
pass 5, the only files under the Word and shared layout and PDF crates that
changed are version lines in `crates/oxml-layout/Cargo.toml`,
`crates/oxml-pdf/Cargo.toml`, and four crate READMEs. The one runtime effect of
the version move is `docProps/app.xml` `AppVersion` through
`crates/rdocx/src/document.rs:4739`, which is deterministic per build, so
repeated generation stays byte-identical. The hash harness does not hash that
part, per `collect_hashes` in `scripts/hash_harness.py:334`, and no PDF writer
embeds a crate version, so the release preparation predicts no harness delta.

This pass ran no cargo command, the private corpus, or the hash harness,
because a full verification was using the shared target directory. The run
state records its latest passing full verification at `0e9b425e`, not at
`93e1edc1`, so `/release` precondition 5 is not yet satisfied at this HEAD.
That is the next scheduled step of the release dependency extension, not a
finding. No publication has happened and this verdict claims no sprint
closure.

## Not found

- Interaction: the F-X112 source edits are version literal assertions only, in
  `crates/oxml-chart/src/lib.rs`, `crates/oxml-drawing/src/lib.rs`,
  `crates/rpptx-render/src/lib.rs`, `crates/rdocx-wasm/src/lib.rs`,
  `crates/rpptx-wasm/src/lib.rs`, and two integration tests. The release notes
  were spot-checked against integrated behavior and all checked claims hold:
  `OpcError::DuplicatePartName` at `crates/oxml-opc/src/error.rs:23` is absent
  at `rpptx-v0.11.0`, `LayoutLine::forced_break_after` at
  `crates/oxml-layout/src/line.rs:258`, `FieldSource` at
  `crates/oxml-layout/src/output.rs:121`, `ResolvedImage::opacity` at
  `crates/rpptx-layout/src/lib.rs:226`, `slide_png_deterministic` at
  `crates/rpptx/src/lib.rs:1179`, notes counted by `replace_text` at
  `crates/rpptx/src/lib.rs:1690`, the zero-match, expected-count, and
  existing-output refusals at `crates/rpptx-cli/src/commands.rs:292` and
  `crates/oxml-cli-support/src/lib.rs:121`, `Error::Story` at
  `crates/rdocx/src/error.rs:58`, `insert_picture_to_story`,
  `add_story_comment`, `update_page_fields`, `update_layout_backed_fields`,
  and `set_update_fields_on_open` in the facade, and the `NsReader` signature
  at `crates/rdocx-oxml/src/drawing.rs:969`. The Python rpptx stub diff since
  `py-rpptx-v0.11.0` is additive and exposes no `replace_text`, so "calls
  remain source compatible" at `CHANGELOG.md:485` holds. The per-family story
  maps agree with ancestry: F-X092 and F-X087 are inside the Python tag SHA
  `2b009243` and correctly absent from the Python notes, while F-X101 is not
  and is present. Sampled authorship through read-only `gh` matches the
  credited handles for Issues 69, 99, 115, 120, and 121 and PRs 71, 77, 105,
  122, and 123.
- Duplication: F-X112 adds no helper. The release contract test reuses the
  existing `render_release_notes` and `RELEASE_TAG_RE`.
- Layering: no `oxml-*` manifest names an `rdocx-*` or `rpptx-*` dependency.
- Harness: `scripts/hash_baseline.json` changed once in the sprint, in
  `295672fa` for F-X102, as passes 1 to 5 recorded. F-X112 leaves it unchanged
  and its trailer defers the harness to the sprint gate.
- Gate, version carriers: `Cargo.toml:34` is 0.14.0. Among the internal pins
  at `Cargo.toml:55` to `Cargo.toml:78`, the 16 incubating pins are 0.12.0 and
  the 8 stable pins are 0.14.0. All 17 explicit `oxml-*` and `rpptx*` manifests are 0.12.0 and the
  10 inheriting packages resolve to 0.14.0. `Cargo.lock` holds 17 entries at
  0.12.0 and 10 at 0.14.0. `crates/rdocx-py/pyproject.toml:7` is 0.14.0 and
  `crates/rpptx-py/pyproject.toml:7` is 0.12.0. Every crate README, the root
  README, `scripts/readme_doctests.py`, and the CI WASM literals at
  `.github/workflows/ci.yml:366` and `.github/workflows/ci.yml:367` agree. The
  publishable set is exactly the 22-package union, with `rdocx-py`,
  `rpptx-py`, `oxml-py-support`, and both WASM crates at `publish = false`.
- Gate, `/release` preconditions 7 to 9: `.github/workflows/publish.yml:159`
  and `.github/workflows/publish.yml:176` bind `v*` and `rpptx-v*` to exactly
  the 7 and 15 package sets with bare `cargo publish -p`, no failure masking,
  and a registry wait between each package. Both orders respect every internal
  normal dependency read from the crate manifests. The dry run at
  `.github/workflows/publish.yml:135` patches exactly 22 packages. The four
  test names at `.github/workflows/publish.yml:122` and
  `.github/workflows/publish.yml:126` exist at
  `scripts/test_sprint_workflow.py:5154`, `scripts/test_sprint_workflow.py:5571`,
  `scripts/test_sprint_workflow.py:5781`, and
  `scripts/test_sprint_workflow.py:6230`, and all four plus the S73 contract
  test pass under `unittest`, with the registry-only proof skipped locally as
  designed. The binstall `pkg-url` templates match the archive names the
  workflow builds. `.github/workflows/wheels.yml` is unchanged in the sprint.
  Its publish job at `.github/workflows/wheels.yml:176` runs only on pushed
  `py-rdocx-v*` or `py-rpptx-v*` tags, is the only job with `id-token: write`
  at `.github/workflows/wheels.yml:184`, selects one distribution, and
  validates the exact tag-derived seven-file set through
  `validate_python_release_artifacts`. Manual dispatch builds both
  distributions and reaches no publication or release step. Every action in
  both workflows is pinned to a commit SHA. `release-notes --check` reports ok
  for all four tags.
- Docs: HLD 03, 10, 12, 14, and 15 and the plan agree on stable Rust and PyPI
  `rdocx` 0.14.0, incubating Rust and PyPI `rpptx` 0.12.0, the four tags, the
  incubating-first order, and the superseded 0.13.2 crates.io train. The HLD
  counts of 8 stable pins, 10 inherited lockfile packages, 17 explicit
  manifests, 16 incubating pins, and 17 lockfile entries match the workspace.
  The historical 0.11.0 and 0.13.2 statements that remain are accurate release
  history. `CLAUDE.md` was updated to the prepared versions.
- Delivery records: F-X114 is done with owner `-` at
  `docs/sprints/CURRENT_SPRINT.md:72` and in BACKLOG, and completed in the run
  state. F-X112 is `in-progress` with owner `claude` at
  `docs/sprints/CURRENT_SPRINT.md:80` and in BACKLOG, `reviewed` in the run
  state with integration commit `93e1edc1`, and has no AS_BUILT or tracker
  row, as the release exception requires. An independent recount of every
  AUTOGEN table matches every summary row at `docs/sprints/BACKLOG.md:17` to
  `docs/sprints/BACKLOG.md:44`. The X table holds 126 done, 1 in progress, 0
  pending, and 4 archived rows for 131, and the totals are 449, 376, 1, and 68
  with no duplicate F-ID. `sync_agent_skills.py --check` reports all 26 skills
  in sync.
- Dependencies: the only manifest changes in the sprint beyond versions are
  the F-X111 binstall metadata and `system-fonts` features in
  `crates/rdocx-cli/Cargo.toml` and `crates/rpptx-cli/Cargo.toml`. F-X112 adds
  no dependency and `Cargo.lock` gains no package.
- Surface: F-X112 adds no public native, Python, WASM, or CLI item.
