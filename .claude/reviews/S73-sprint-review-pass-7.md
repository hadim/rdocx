# S73 sprint review, pass 7

**Reviewed**: `sprint/s73` at `a7da7afd` against `029065ca3546559f982900260878eab4ac73f1a4`, 254 files, 33,883 changed lines, crates: oxml-chart, oxml-cli-support, oxml-core, oxml-drawing, oxml-layout, oxml-media, oxml-opc, oxml-pdf, oxml-sml, rdocx, rdocx-cli, rdocx-html, rdocx-layout, rdocx-opc, rdocx-oxml, rdocx-pdf, rdocx-py, rdocx-wasm, rpptx, rpptx-chart, rpptx-cli, rpptx-layout, rpptx-oxml, rpptx-py, rpptx-render, rpptx-wasm
**Verdict**: 0 blocking, 0 should-fix, 2 nice-to-have

Bound extension: scheduled dependency-prefix boundary. This pass exists only
because `/release` requires a clean sprint review at the exact release HEAD,
and it re-establishes that clean result at `a7da7afd` after pass 6's
nice-to-have remediation.

## Scope of this pass

Pass 6 was clean at `93e1edc1` with 0 blocking, 0 should-fix, and 4
nice-to-have findings. The delta since is two commits and no code. `f370039e`
adds `.claude/reviews/S73-sprint-review-pass-6.md` and nothing else.
`a7da7afd` changes only `CHANGELOG.md` and
`docs/hld/14-development-backlog.md`, 9 and 8 lines, applying pass 6 findings
N3 and N4. `git diff --stat 93e1edc1..a7da7afd` reports exactly those three
files, so no crate source, manifest, lockfile, workflow, script, or test
changed. This pass therefore verified the two doc edits in full against the
code and the tests they describe, re-ran the cheap consistency checks at this
HEAD, re-confirmed the delivery records, and re-confirmed the pass 6
conclusions that the unchanged files carry.

Pass 6 findings N1 and N2 were deliberately left. They are retained below as
recorded observations at the same severity, not re-raised.

## Blocking

None.

## Should-fix

None.

## Nice-to-have

### N1, The F-X114 Word records measure preservation rather than Word's TOC generation
`crates/rdocx/tests/regression_test.rs:8498`

Retained from pass 6 at the same severity, with no new evidence. Nothing under
`crates/rdocx` changed since `93e1edc1`. The ignored capture still opens the
rebuilt document at `crates/rdocx/tests/regression_test.rs:8498` and saves it
at `crates/rdocx/tests/regression_test.rs:8500` without updating the TOC field,
so the records prove Word retains rdocx's entries rather than that they equal
entries Word would generate. Evidence wording only. The contracted behavior is
proved directly by the regression gate.

### N2, A created canonical TOC style uses an uppercase built-in name
`crates/rdocx/src/field.rs:5762`

Retained from pass 6 at the same severity, with no new evidence. The created
style is still named `TOC {level}` at `crates/rdocx/src/field.rs:5762`, while
the selector compares names case-insensitively, so a repeated rebuild creates
no duplicate style. No user-visible defect was found.

## Milestone gate

The M23 gate at `docs/hld/14-development-backlog.md:2214`:

```text
all five private references are generated from a blank public facade, reopen
without repair, match the required package semantics, and meet the reviewed
deterministic visual thresholds. Repeated generation produces identical DOCX
bytes, and no generated document reports an unexplained preservation-only
fallback.
```

The gate holds at this HEAD. Named evidence, not assertion:
`m23_private_from_scratch_corpus_passes_required_mode` at
`crates/rdocx/tests/integration_test.rs:2768` is the corpus gate, recorded
passing at `a809a244` by pass 3. The run state at
`.claude/scratch/S73-run.json` now records a full passing verification at
`a7da7afd` with the harness unchanged at 49 of 49 entries, which is the entry
pass 6 could not yet cite. `/release` precondition 5 is therefore satisfied at
this HEAD, where pass 6 recorded it as pending. This review ran no cargo
command, no private corpus run, and no hash harness run, and it relies on that
recorded verification rather than a rerun. No publication has happened and this
verdict claims no sprint closure.

## Not found

- Release note truth, picture limits: the new wording is exact.
  `MAX_RENDER_PREVIEW_BYTES` is the encoded-file limit at 16 MiB
  (`crates/rpptx/src/lib.rs:8511`), enforced at
  `crates/rpptx/src/lib.rs:8520`, and `MAX_RENDER_PREVIEW_DECODED_BYTES` is
  the decoded-pixel limit at 64 MiB (`crates/rpptx/src/lib.rs:8513`), enforced
  at `crates/rpptx/src/lib.rs:8533` and `crates/rpptx/src/lib.rs:8547`. At tag
  `rpptx-v0.11.0` both constants were 16 MiB, so the decoded limit rose and the
  encoded limit did not. At that tag the bounds function returned a bare
  `bool`, with an over-limit picture dropped and no diagnostic, which makes
  "instead of being skipped silently" accurate for the prior behavior, and the
  current signature returns a reported reason. The wording appears at
  `CHANGELOG.md:51` in the `rpptx-v0.12.0` section that starts at
  `CHANGELOG.md:7`, and at `CHANGELOG.md:475` in the `py-rpptx-v0.12.0`
  section that starts at `CHANGELOG.md:450`. Both are correct placements,
  because the two constants exist only in `crates/rpptx/src/lib.rs` and in no
  stable-family crate. The `v0.14.0` and `py-rdocx-v0.14.0` sections claim only
  the premultiply half of Issue 119 at `CHANGELOG.md:201` and
  `CHANGELOG.md:380`, which remains right for the stable family.
- Release note truth, gate description: the new F-X112 test gate text at
  `docs/hld/14-development-backlog.md:5352` to
  `docs/hld/14-development-backlog.md:5358` matches the test. Version
  agreement is asserted at `scripts/test_sprint_workflow.py:5455` and
  `scripts/test_sprint_workflow.py:5487`, the 7 and 15 package allowlists at
  `scripts/test_sprint_workflow.py:5470` and
  `scripts/test_sprint_workflow.py:5471`, the four tags at
  `scripts/test_sprint_workflow.py:5492` and
  `scripts/test_sprint_workflow.py:5494`, and the per-tag issue, pull-request,
  and contributor equality in `assert_s73_release_notes_truth_contract` at
  `scripts/test_sprint_workflow.py:5393`, cross-checked against the backlog and
  AS_BUILT citations of each shipped story at
  `scripts/test_sprint_workflow.py:5560`. The test observes no built artifact,
  approval, registry owner, or posted comment, so moving those to `/release` is
  correct. It does not contradict `docs/hld/12-testing-strategy.md:2563` to
  `docs/hld/12-testing-strategy.md:2573`, which already described the test by
  what it renders and compares, nor the plan row at
  `.claude/plans/F-X112-design.md:55`, whose "selected assets" is satisfied by
  the asset wording the test requires in the rendered notes at
  `scripts/test_sprint_workflow.py:5499` and
  `scripts/test_sprint_workflow.py:5502`, while the artifacts themselves stay
  with the plan's own package, Python, and release rows at
  `.claude/plans/F-X112-design.md:56` to `.claude/plans/F-X112-design.md:58`.
- Consistency checks at this HEAD: `release-notes --check` reports ok for
  `v0.14.0`, `rpptx-v0.12.0`, `py-rdocx-v0.14.0`, and `py-rpptx-v0.12.0`.
  `python3 -m unittest scripts.test_sprint_workflow` is OK, 121 tests with the
  2 registry-only proofs skipped as designed. `python3 scripts/prose_check.py`
  reports 0 violations. `python3 scripts/sync_agent_skills.py --check` reports
  26 skills in sync. The working tree is clean at `a7da7afd`.
- Backlog counts: an independent recount of every AUTOGEN table matches every
  summary row at `docs/sprints/BACKLOG.md:17` to `docs/sprints/BACKLOG.md:44`.
  The milestone tables hold 318 rows, 250 done and 68 pending, and the X table
  at `docs/sprints/BACKLOG.md:537` holds 131 rows, 126 done, 1 in progress, 0
  pending, and 4 archived. The totals of 449, 376, 1, and 68 follow, with no
  duplicate F-ID anywhere.
- Delivery records: F-X114 is done with owner `-` at
  `docs/sprints/CURRENT_SPRINT.md:72`, recorded in AS_BUILT at
  `docs/sprints/AS_BUILT.md:15063` and in the tracker at
  `docs/sprints/SPRINT_TRACKER.md:470`, and completed in the run state. F-X112
  is `in-progress` with owner `claude` at `docs/sprints/CURRENT_SPRINT.md:80`
  and in BACKLOG, `reviewed` in the run state with integration commit
  `93e1edc1`, and correctly has no AS_BUILT or tracker row under the release
  exception. The other 33 sprint rows in `docs/sprints/CURRENT_SPRINT.md` are
  all done with owner `-`.
- Layering: no `crates/oxml-*/Cargo.toml` names an `rdocx-*` or `rpptx-*`
  dependency.
- Harness: `scripts/hash_baseline.json` changed once in the whole sprint, in
  `295672fa` for F-X102, exactly as passes 1 to 6 recorded. Nothing since
  `93e1edc1` touches it.
- Interaction, duplication, deps, and surface: no source, manifest, lockfile,
  workflow, or test file changed since pass 6, so the pass 6 conclusions in
  those aspects stand unchanged at `a7da7afd` and were not re-derived.
