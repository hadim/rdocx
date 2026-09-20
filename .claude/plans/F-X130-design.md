# F-X130, Show package depth, footprint, and speed

**Status**: completed
**Sprint**: S74
**Size**: L
**Depends on**: F-X089, F-264, F-265, F-266a, F-266b, F-266c, F-267, F-268a,
F-268b, F-269, F-270

## Problem

F-X089 made the README family capability-led and mutation-sensitive, but it
deliberately banned every footprint and speed statement rather than proving
one. `scripts/readme_doctests.py:646` rejects the words `binary size`,
`install size`, `memory`, `faster`, `fastest` and `most popular` anywhere in
the root `## Evidence-based alternatives` section, and its own design plan
records that decision in `.claude/plans/F-X089-design.md`, "Rejected
alternatives". The result is a README family that can say what the workspace
does and cannot say what it costs or how quickly it runs.

The measurements exist but reach no reader. The release-mode gate at
`crates/rdocx/tests/regression_test.rs:6568` already enforces 1,000 pages of
deterministic layout at or above 250 pages per second inside 64 MiB, and direct
PDF rendering at or above 1,000 pages per second inside a further 16 MiB. CI
enforces a 10 MiB archive ceiling at `.github/workflows/ci.yml:590` without
recording where any package actually sits under it. The six CLI release
archives carry a `SHA256SUMS` file and no size record. The two Python projects
publish their crate-local README as the PyPI long description, bound by
`workflow.PYTHON_RELEASE_METADATA`, and neither page states a wheel size, an
installed footprint, or a boundary cost.

The second half of the problem is authoring depth. S74 adds the complete
paragraph, run, typography, table, section, and settings authoring surface
through F-264, F-265, F-266a, F-266b, F-266c, F-267, F-268a, F-268b, F-269 and
F-270. Once that lands, the crate pages written for the S71 surface understate
what each package does, and the root page understates the Word authoring depth
that distinguishes it. This story runs last in S74 and alone, because it
documents the final public surface.

## Spec reference

- `docs/hld/15-build-and-toolchain.md`, "CI job matrix", the paragraph
  beginning "Workspace package READMEs in the docs job", which fixes the exact
  27-package inventory, the three root Rust examples, the seven-row workflow
  summary, and the bounded alternatives table.
- `docs/hld/15-build-and-toolchain.md`, "CI job matrix", the paragraph
  beginning "The large-document gate in the test job", which fixes the locked
  release-mode single-threaded invocation of the ignored 1,000-page test.
- `docs/hld/15-build-and-toolchain.md`, "Packaging", the crates.io 10 MiB
  archive ceiling, the `oxml-layout` 24-font inventory rule, and the rule that
  every CLI tag archive carries the selected CLI crate README as `README.md`
  plus the workspace `LICENSE`.
- `docs/hld/12-testing-strategy.md`, "Binding tests", the block beginning "All
  27 workspace packages explicitly declare one distinct README", through the
  paragraph describing `scripts/readme_doctests.py` mutation coverage and the
  22 publishable archive README byte comparisons.
- `docs/hld/12-testing-strategy.md`, "Binding tests", the paragraph beginning
  "The large-document regression source-builds 1,000 one-page paragraphs",
  which states the declared throughput and peak-allocation floors.
- `docs/hld/12-testing-strategy.md`, "Binding tests", the paragraph beginning
  "The Python metadata regression requires both projects to name a crate-local
  Markdown README", which binds wheel `METADATA` and source-distribution
  `PKG-INFO` to the README bytes.
- `docs/hld/10-bindings-spec.md`, "Packaging", the abi3-py39 wheel matrix, the
  always-bundled font decision and its stated per-wheel cost, and the rule that
  each Python project uses its crate-local `README.md` as the Markdown long
  description.
- `docs/hld/10-bindings-spec.md`, "CLIs", the rule that a CLI release archive
  contains exactly the executable, its crate README, and the workspace
  `LICENSE`.

## Approach

### The inventory is 27 pages and the count is correct

`ls crates/` lists 27 directories and `ls crates/*/README.md` lists 26 files.
The missing one is `crates/rdocx/README.md`, and that is deliberate.
`crates/rdocx/Cargo.toml:13` declares `readme = "../../README.md"`, so the
facade publishes the root page as its crates.io front page.
`readme_doctests.validate_inventory` therefore requires 27 packages and 27
distinct README sources, and both hold. The story's phrasing, the root plus 26
crate-local pages, is accurate. No file is missing and none is added.

S74 adds one new module, `crates/rdocx-oxml/src/web_settings.rs` from F-270.
It is a module inside an existing crate and not a new crate, so the workspace
stays at 27 packages, `WORKSPACE_PACKAGE_COUNT` stays at 27,
`PUBLISHABLE_PACKAGE_COUNT` stays at 22, and the page inventory stays at the
root plus 26 crate-local files. A reviewer does not need to re-derive that.

### Evidence lives in the validator, provenance lives in the page

Every measured claim becomes one row of a fixed nine-column table under a new
`## Measured footprint and speed` heading. The columns are exactly
`Measurement`, `Value`, `Version`, `Platform`, `Build mode`, `Input`,
`Command`, `Statistic`, `Measured on`. That satisfies the story requirement
literally, in the page a reader is looking at, and it parses without ambiguity.

The approved values live in `scripts/readme_doctests.py` as module-level
constants beside `COMPARISON_ROWS`, `ROOT_WORKFLOW_CLAIMS` and
`COMPARISON_EVIDENCE`. **No new file is created.** That is the decided answer.
A separate `scripts/readme_evidence.json` was considered and rejected, because
the three new modules approved for S74 do not include it and the repository
already keeps approved README constants inside the validator. The constant is a
mapping from a stable measurement id to a frozen nine-tuple:

```python
MeasurementRow = tuple[str, str, str, str, str, str, str, str]

MEASUREMENT_ROWS: dict[str, MeasurementRow] = {...}
MEASUREMENT_PAGES: dict[Path, tuple[str, ...]] = {...}
MEASUREMENT_TIERS: dict[str, str] = {...}   # "rederived" | "gated" | "recorded"
```

`MEASUREMENT_PAGES` names which ids each README may carry. A page may carry no
row it is not listed for, and must carry every row it is listed for. A row that
appears on two pages must be byte-identical on both, so a crate page and the
root page cannot disagree.

### Two tiers land, the third is tracked and absent

**Tier one, re-derived on every validator run. Lands.** Rust `.crate` archive
footprint. `validate_package_archive` already builds all 22 publishable
archives with the same local patch set the release dry run uses. It gains the
size capture for free. The gate normalizes Cargo's generated
`.cargo_vcs_info.json` to a fixed-length clean revision before totaling member
bytes, then asserts that source-determined total and the member count exactly.
The compressed `.crate` byte size stays below the 10 MiB ceiling and within 64
bytes of the recorded observation under the pinned toolchain. Another
toolchain may produce an archive no more than 64 bytes above that observation.
The narrow allowance covers only the commit hash and dirty marker in Cargo's
generated metadata. Tracked source growth still changes the exact normalized
member total and fails the gate.

**Tier two, bound to a threshold that lives in code. Lands.** Layout and PDF
throughput and peak allocation. The README publishes both the enforced floor
and one dated observation. The validator parses
`MIN_LAYOUT_PAGES_PER_SECOND`, `MIN_PDF_PAGES_PER_SECOND`,
`MAX_LAYOUT_PEAK_BYTES` and `MAX_PDF_PEAK_BYTES` out of
`crates/rdocx/tests/regression_test.rs` and asserts that no published speed
claim is faster than the gate proves and no published memory claim is smaller
than the gate proves. A README speed number can therefore never outrun the
assertion that backs it. Raising the README figure without raising the test
constant fails. Lowering the test constant without lowering the README fails.

The dated observation is taken on the machine
`docs/hld/12-testing-strategy.md` already names for the Issue 67 evidence, one
Apple M5 Max on macOS 26.6.2 with rustc 1.97.1. Continuity with the recorded
evidence beats naming the Ubuntu 24.04 runner that enforces the floor, and the
row carries its platform cell either way.

Speed rows appear on exactly four pages, the root page,
`crates/rdocx-layout/README.md`, `crates/oxml-pdf/README.md` and
`crates/rdocx-py/README.md`. `crates/rdocx-pdf/README.md` and
`crates/rpptx-render/README.md` carry none, because a speed row on a page whose
consumer cannot reproduce it is noise.

**Tier three, human or tag-job measurement. Does not land in this story.**
Python wheel and source-distribution sizes, installed site-packages footprint,
CLI release archive sizes, WASM bundle sizes, and any Python boundary timing.
None can be produced by a native `cargo test` in this repository. They need
either the six-runner `wheels.yml` matrix, a Rust `v*` tag run of `publish.yml`,
or a pinned interpreter with an installed wheel. **Their rows stay absent
rather than unproven.** An absent row is honest. A row carrying a number nobody
can reproduce is the exact failure this story exists to prevent.

The absence is deliberate and legible rather than an oversight. The
implementation adds a tracked human-action follow-up to
`docs/hld/14-development-backlog.md` naming each deferred measurement, its
producing workflow, and the page that will carry it once taken. The
`MEASUREMENT_TIERS` mapping keeps the `recorded` tier defined with no members,
so the shape is ready and the gate states that the set is empty on purpose.

### Recording mode

`scripts/readme_doctests.py` gains `--record-measurements`, which runs the tier
one derivations and the tier two threshold reads and prints the rows in the
exact Markdown table syntax the README expects. The implementer pastes the
output rather than retyping a number, so the value in the page and the value
the command produced cannot diverge by transcription.

### Bounded comparison and superlative ban

The existing `validate_comparison_evidence` volatile-word ban on the
`## Evidence-based alternatives` section stays exactly as it is. Footprint and
speed never enter the comparison table. A new `validate_measurement_evidence`
adds a workspace-wide ban across all 27 pages on unbounded superlatives and
unbounded set claims, at minimum `fastest`, `smallest`, `lightest`, `every
library`, `any other library`, `all other`, `industry-leading`, `unmatched`,
`best-in-class`, and a bare `faster than` or `smaller than` that is not
followed by a named subject inside the same sentence. Every uniqueness sentence
must remain inside the one approved date-bounded conclusion that
`ROOT_UNIQUENESS_CLAIM` already pins. Cross-workload comparison is prevented
structurally: a row states one workload in `Input` and one statistic in
`Statistic`, and two rows are never presented as a ratio.

### Depth summaries

Each of the 27 pages keeps its F-X089 structure and gains depth for its own
consumer. The root page carries the final S74 authoring depth by deepening the
existing `Rich authoring` row and **adds no eighth row**.
`ROOT_WORKFLOW_CLAIMS` is an exact seven-row contract and
`docs/hld/15-build-and-toolchain.md` fixes the same seven-row shape, so an
eighth row would be a narrative change in two places rather than a depth
statement in one. `crates/rdocx-py/README.md` and
`crates/rpptx-py/README.md` gain explicit native-engine, typed-API,
local-execution, rendering, review and package-preservation statements above
the fold, because those two files are the PyPI long descriptions. Their edits
flow into wheel `METADATA` and source-distribution `PKG-INFO` through the
existing binding, so the Python metadata regression covers them unchanged.

### What this story needs from the rest of S74

F-266 has split into F-266a, F-266b and F-266c, and F-268 into F-268a and
F-268b. The parents close only when their children close. This story cannot
start before all ten authoring stories complete, and it needs four concrete
things from them.

1. The final public authoring API list for `rdocx` and `rdocx-oxml`, so the
   capability bullets on the root page and on the four `rdocx-*` pages name
   what shipped and not what was planned.
2. Any new default-off Cargo feature, because
   `validate_opt_in_security_claims` binds advertised opt-ins to named
   default-off features and a new one needs the same treatment.
3. Any new `rdocx-py` surface, because the Python snippets must match the
   installed surface.
4. Final integrated source, because every tier one and tier two measurement
   must be taken after the last authoring story integrates. A measurement taken
   earlier is a measurement of a workspace that no longer exists. This is why
   F-X130 runs last in S74 and alone.

## Rejected alternatives

- **A new `scripts/readme_evidence.json` file.** It is the obvious shape and it
  is a new file, which the structural rules require an explicit ask for. The
  three new modules approved for S74 do not include it, and the repository
  already keeps approved README constants inside `readme_doctests.py`, so the
  module-level table adds no place to look. Decided against.
- **Free prose claims with a footnote.** A footnote cannot be parsed against
  the command that produced it, so the claim drifts silently.
- **Content-hash pinning of the measured source, as F-X075 does at
  `crates/rdocx/tests/regression_test.rs:6781`.** Correct for a one-time
  benchmark against two fixed reference commits. Wrong for a README, because
  the manifest covers every tracked crate byte and would fail on the next
  unrelated commit. Version binding is the right granularity for a page that
  ships with a release.
- **Publishing a benchmark against python-docx or docx4j.** The story forbids
  comparing timings from different workloads or environments, and no reviewed
  official source states a comparable figure.
- **Re-deriving wheel sizes in the validator.** They come from six platform
  runners in `wheels.yml`. A native run cannot produce them, and a fabricated
  local number is worse than an absent one.
- **Landing tier three rows from a one-off manual run.** Rejected with the rest
  of tier three. A number that only one person on one machine can regenerate
  cannot be defended by a gate, so the row stays absent and tracked.
- **A separate size checker script.** F-X089 already decided to extend the one
  validator rather than add a second checker.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| regression | `readme_depth_footprint_and_speed_claims_are_evidence_backed` | The exact 27-page inventory, the complete family and Python depth summaries, bounded official comparisons, every measurement row's provenance against the command that produced it, checked examples, and byte-identical packaged long descriptions. The story's named test gate. |
| regression | `test_measurement_rows_match_rederived_archive_footprint` | Each tier one row equals the uncompressed member total, member count and compressed size of the archive the validator builds, and every archive stays below the 10 MiB ceiling. |
| regression | `test_measurement_speed_claims_never_exceed_the_gated_floor` | Every published throughput and peak-allocation figure is bounded by the constants in `crates/rdocx/tests/regression_test.rs`. Mutating either side fails. |
| regression | `test_measurement_rows_require_complete_dated_provenance` | Blanking any of the nine cells, removing the date, using a non-ISO date, or naming a version that differs from `cargo metadata` or `pyproject.toml` fails. |
| regression | `test_measurement_pages_carry_exactly_their_approved_rows` | A page carrying a row it is not listed for, missing a row it is listed for, or disagreeing with the root row for the same id fails. Speed rows appear on exactly the root page, `rdocx-layout`, `oxml-pdf` and `rdocx-py`, and a speed row added to `rdocx-pdf` or `rpptx-render` fails. |
| regression | `test_deferred_measurements_stay_absent_and_tracked` | The `recorded` tier is empty, no wheel, sdist, installed-footprint, CLI-asset, WASM-bundle or Python-boundary figure appears on any of the 27 pages, and each deferred measurement has its named follow-up entry. |
| regression | `test_readme_family_rejects_unbounded_superlatives` | Each banned superlative and unbounded set phrase fails on each of the 27 pages, and the approved date-bounded uniqueness sentence still passes. |
| regression | `test_root_readme_approved_comparison_evidence_matrix` | Unchanged. Footprint and speed vocabulary is still rejected inside the alternatives section. |
| regression | `test_crate_readmes_present_capabilities_and_audience` | Extended to the post-F-270 capability bullets for the `rdocx-*` family. |
| regression | `test_python_metadata_contract` | `rdocx-py` and `rpptx-py` long descriptions keep every required section after the depth rewrite. |
| integration | `python3 scripts/readme_doctests.py` | All Rust examples compile, non-Rust snippets match public surfaces, versions match metadata, local links resolve, and the 27-package inventory passes. |
| package | 22 publishable archives | Exactly one packaged README per archive, byte-identical to the declared source. |
| external | `python3 scripts/readme_doctests.py --check-official-links` | Every approved comparison source resolves during implementation review, not in default CI. |
| manual | pinned release-mode measurement run | The tier two observation is taken once on the named Apple M5 Max running macOS 26.6.2 with rustc 1.97.1, with the locked release binary, the ignored 1,000-page test selected, and one test thread. Its output is pasted from `--record-measurements`. |

The **test gate is regression**, named
`readme_depth_footprint_and_speed_claims_are_evidence_backed`. It is a
`unittest` method in `SprintWorkflowTests` in `scripts/test_sprint_workflow.py`,
where the whole F-X089 mutation family already lives. Python collection
requires a `test_` prefix, so the method is declared as
`test_readme_depth_footprint_and_speed_claims_are_evidence_backed`. The backlog
entry's spelling without the prefix is the human-readable name and stays
exactly as it is. This story does not edit it.

## HLD impact

- `docs/hld/12-testing-strategy.md`
- `docs/hld/14-development-backlog.md`
- `docs/hld/15-build-and-toolchain.md`

## Risk routing

Matched rows from `.claude/skills/risk-routing.md`:

- **Release scripting, version strings.** The README family is a release input.
  The 22 packaged archives carry these bytes, the six CLI tag archives carry
  the selected CLI crate README verbatim, and both PyPI long descriptions are
  these bytes. Extra checks: inspect every README version string against
  `cargo metadata` and both `pyproject.toml` files and confirm this diff
  changes none of them, run the patched 22-package publish dry run with the
  archive-size gate, and require a clean full `/verify` before completion. No
  tag is created and no publication is started by this story.
- **WASM or PyO3 bindings.** `crates/rdocx-py/README.md`,
  `crates/rpptx-py/README.md`, `crates/rdocx-wasm/README.md` and
  `crates/rpptx-wasm/README.md` are consumer-facing binding surfaces whose
  snippets and scoped build commands are validated against the real packages.
  Extra checks: keep `--exclude rdocx-py --exclude rpptx-py` on every workspace
  run, re-run the scoped WASM import and build-command assertions, and check
  each Python snippet against an installed wheel before recording any Python
  boundary row.

Rows considered and not matched:

- **Public API of a published crate.** This diff changes no Rust public API.
  F-264 to F-270 carry that row.
- **A new trait, generic parameter, crate, module or file.** The decided
  approach adds no file, no module, no trait and no generic parameter. The
  evidence constants live in `scripts/readme_doctests.py`. F-270's new
  `crates/rdocx-oxml/src/web_settings.rs` belongs to that story, not this one,
  and leaves the 27-package and 27-README inventory unchanged.
- **An external oracle comparison.** The comparison set is documentation, not a
  running oracle. Its equivalent pin is the dated reviewed source set and the
  `--check-official-links` run.
- Unit conversion, theme colour, layout and pagination, parser or serialiser,
  crate dependency graph, bundled fonts, new feature flag, file move: none.

## Hash harness

Expected unchanged at 49 of 49 entries. Confirmed by reading
`scripts/hash_harness.py`. Its only input is
`cargo run -p rdocx --example generate_all_samples`
at line 319, and it hashes three `word/*.xml` parts, a deterministic page-one
PNG, and three PDF fingerprint entries for each of the seven samples, which is
7 times 7 and gives the 49 entries. No README, no validator constant and no
documentation file reaches that command. This story records no baseline change
and must not touch `scripts/hash_baseline.json`.

## Implementation checklist

- [x] Confirm F-264, F-265, F-266a, F-266b, F-266c, F-267, F-268a, F-268b,
      F-269 and F-270 are all completed and integrated, and that the sprint
      branch is the final S74 surface, before taking any measurement.
- [x] Add `MEASUREMENT_ROWS`, `MEASUREMENT_PAGES` and `MEASUREMENT_TIERS` to
      `scripts/readme_doctests.py`.
- [x] Add `--record-measurements` and capture archive footprint inside the
      existing 22-archive build.
- [x] Parse the four throughput and peak constants out of
      `crates/rdocx/tests/regression_test.rs` and bound every published figure
      by them.
- [x] Take the tier two observation once on the named Apple M5 Max with the
      locked release binary and one test thread.
- [x] Leave every tier three row absent, keep the `recorded` tier empty, and
      add the tracked human-action follow-up entries to
      `docs/hld/14-development-backlog.md`.
- [x] Add `## Measured footprint and speed` to the root page and to each listed
      crate page, with speed rows on exactly the root page, `rdocx-layout`,
      `oxml-pdf` and `rdocx-py`.
- [x] Rewrite the depth summaries on all 27 pages for the final S74 surface,
      deepening the existing `Rich authoring` row and adding no eighth row.
- [x] Rewrite both Python long descriptions for native engine, typed API, local
      execution, rendering, review and package preservation.
- [x] Add `validate_measurement_evidence` and the superlative ban.
- [x] Add the named test gate and every mutation test in the table.
- [x] Run `readme_doctests.py`, `--check-official-links`, the patched publish
      dry run, `prose_check.py`, `sync_agent_skills.py --check` and full
      `/verify`.
- [x] Update exactly the three listed HLD files.

## Open questions

None. Resolved in the S74 consolidated design round.
