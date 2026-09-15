# S72 sprint review, pass 20

**Reviewed**: `sprint/s72` at `a1f5b8486033` against merge base
`5fdbfc578adf2e5a1ceca0a3f0cee601ece44746`, 194 files, 39,254
insertions, 4,892 deletions, 44,146 changed lines, crates: `oxml-layout`,
`oxml-pdf`, `rdocx-cli`, `rdocx-layout`, `rdocx-oxml`, `rdocx-py`,
`rdocx`, `rpptx-py`, `rpptx`
**Review boundary**: first pass at the scheduled post-ledger and pre-release
boundary. The sprint-running session extends the global pass number for the
explicit reason `scheduled post-ledger and pre-release boundary`.
**Verdict**: 0 blocking, 0 should-fix, 3 previously recorded nice-to-have

## Blocking

None. Count: 0.

## Should-fix

None. Count: 0.

## Nice-to-have

### N1, the F-253 final review retains its recorded trailing blank line

`.claude/reviews/F-253-all-pass-21.md:46`

This review-only whitespace observation remains nonbehavioral and safe to
leave recorded.

### N2, the pass-4 sprint review retains its trailing blank line

`.claude/reviews/S72-sprint-review-pass-4.md:125`

This review-only whitespace observation remains nonbehavioral and safe to
leave recorded.

### N3, the pass-5 sprint review retains its trailing blank line

`.claude/reviews/S72-sprint-review-pass-5.md:120`

This review-only whitespace observation remains nonbehavioral and safe to
leave recorded.

## Interaction audit

F-X095 reconciles the four contributor changes against the completed S72
story and package model. The regression at
`crates/rdocx/tests/regression_test.rs:27222` proves modeled empty paragraphs
survive repeated save in their intended self-closing form. The comparison
regression at `crates/rdocx/tests/regression_test.rs:15321` proves a
byte-identical comparison is a true package no-op, and the installed binding
gate at `crates/rdocx-py/tests/test_core.py:304` proves a previously returned
Python handle remains valid across that no-op.

The Presentation fidelity job now runs on Ubuntu 24.04 and installs the exact
Poppler and LibreOffice oracles at `.github/workflows/ci.yml:400`. This matches
the deterministic tooling used by the completed rendering work and removes the
moving Homebrew dependency without introducing another fidelity path.

F-X096 preserves independent native version lines. The workspace version is
0.13.1 at `Cargo.toml:33`, native `rpptx` is 0.11.0 at
`crates/rpptx/Cargo.toml:1`, `rdocx-py` inherits the workspace version at
`crates/rdocx-py/Cargo.toml:1`, and `rpptx-py` declares 0.11.0 at
`crates/rpptx-py/Cargo.toml:1`. Their PyPI metadata matches at
`crates/rdocx-py/pyproject.toml:5` and
`crates/rpptx-py/pyproject.toml:5`. The contract matrix at
`scripts/test_sprint_workflow.py:6418` checks every native manifest, binding
manifest, lockfile entry, Python project version, and tag.

The wheel workflow routes `py-rdocx-v*` and `py-rpptx-v*` independently and
publishes only the selected distribution at
`.github/workflows/wheels.yml:176`. Its exact seven-file positive and negative
artifact matrix is exercised at `scripts/test_sprint_workflow.py:6634`.
Manual dispatch remains build-only, while trusted PyPI publication remains
reachable only from a matching pushed Python tag.

No conflicting ownership, duplicated model, cross-family mutable state, or
lower-to-facade dependency was found in the final integrated delta.

## Milestone gate

The M23 end gate at `docs/hld/14-development-backlog.md:2214` does not yet
hold. Seven planned M23 stories remain outside S72. This is expected for a
milestone delivered across multiple sprints and does not weaken the S72 gate.

The S72 pre-release contract does hold. Every completed implementation item in
`docs/sprints/CURRENT_SPRINT.md:88` has focused and integrated evidence. The
two Python release routes at `docs/sprints/CURRENT_SPRINT.md:112` validate six
wheels and one source distribution each without claiming publication.
F-X094f remains in progress until separate release approval and external
verification complete both publications, as required at
`docs/hld/14-development-backlog.md:4872`.

## Not found

- **interaction**: zero findings. The contributor parser changes, S72 story
  ownership, comparison staging, bindings, and release workflow compose
  without conflict.
- **duplication**: zero findings. No second document tree, parser facade,
  rendering oracle, release inventory, or publication path was added.
- **layering**: zero findings. Manifest changes align package versions only,
  and no forbidden lower-to-facade dependency was introduced.
- **harness**: zero findings. The seven intentional Word `document.xml`
  baseline changes are declared at `scripts/hash_baseline.json:53` and
  reconciled with `docs/sprints/AS_BUILT.md:13538`. PNG and PDF fingerprints
  remain unchanged, and the reviewed current baseline passes 49 of 49.
- **gate**: zero findings for the completed implementation and pre-release
  contract. Publication remains deliberately gated by separate approval.
- **docs**: zero findings. The HLD impact lists, delivery records, independent
  version lines, and release tag names agree with the integrated code.
- **deps**: zero findings. No new dependency declaration was added.
- **surface**: zero findings. Public additions remain bounded by the approved
  issue and contributor stories.

## Checks

- Full verification passed at `9c790241dbf4` before the delivery-only ledger
  commit. It covered formatting, clippy, workspace tests, exact LibreOffice and
  Poppler rendering, both python-pptx oracles, both WASM targets, rustdoc,
  README inventories, workflow regressions, 22 package dry runs, archive
  limits, and the supply-chain audit.
- `python3 scripts/hash_harness.py --check` passed 49 of 49 at the integrated
  implementation boundary.
- `python3 scripts/prose_check.py` passed with 0 violations after the ledger
  update.
- `python3 scripts/sync_agent_skills.py --check` reported all 26 adapters in
  sync after the ledger update.
- The final full verification will rerun after this review-only file is
  committed so the release approval report can bind review and verification
  to one exact SHA.
