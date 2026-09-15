# S72 sprint review, pass 17

**Reviewed**: `sprint/s72` at `a5624991cb79` against merge base
`5fdbfc578adf2e5a1ceca0a3f0cee601ece44746`, 160 files, 34,799
insertions, 3,846 deletions, 38,645 changed lines, crates: `oxml-layout`,
`rdocx-cli`, `rdocx-layout`, `rdocx-oxml`, `rdocx-py`, `rdocx`,
`rpptx-py`, `rpptx`
**Review boundary**: scheduled dependency-prefix boundary after F-X094d and
F-X094e. The sprint-running session extends the configured three-pass limit
through pass 17 for the explicit reason `scheduled dependency-prefix boundary`.
Prior clean passes covered the earlier dependency prefixes.
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

F-X094d projects the completed native section, style, story, and hyperlink
models rather than opening a second package graph. Story-item hyperlink
enumeration resolves through the checked physical owner at
`crates/rdocx/src/document.rs:644`, while the installed binding test exercises
ordered immutable snapshots and relationship ownership at
`crates/rdocx-py/tests/test_core.py:365`. This composes with the earlier
F-X094c revision and snapshot boundary without adding a parallel mutation
path.

F-X094e similarly projects the established PowerPoint facade. Comment authors
and comments remain facade-owned operations at `crates/rpptx/src/lib.rs:1741`
and `crates/rpptx/src/lib.rs:1785`. The native PNG convenience uses the
existing resolved raster path, whose parity gate is
`crates/rpptx/tests/integration.rs:9021`. The installed binding gate combines
rendering, notes, comments, save and reopen, atomic invalid identity handling,
and immutable snapshots at
`crates/rpptx-py/tests/test_documented_examples.py:745`.

The two bindings add no shared mutable state and no cross-family dependency.
Their delivery rows are complete and ownerless at
`docs/sprints/CURRENT_SPRINT.md:56` and
`docs/sprints/CURRENT_SPRINT.md:57`. The AS_BUILT evidence names the exact
native and installed gates at `docs/sprints/AS_BUILT.md:13405` and
`docs/sprints/AS_BUILT.md:13445`.

## Milestone gate

The M23 end gate requires all five private references to be generated from a
blank public facade, reopen without repair, meet package and deterministic
visual requirements, reproduce identical bytes, and report no unexplained
preservation-only fallback
(`docs/hld/14-development-backlog.md:2214`). It does not yet hold. F-X094f and
F-X092 remain pending at `docs/sprints/CURRENT_SPRINT.md:58` and
`docs/sprints/CURRENT_SPRINT.md:59`. The current review establishes only the
completed dependency prefix needed before release preparation.

The completed prefix satisfies its narrower binding gates. The issue contract
requires the exact F-X094d and F-X094e installed tests at
`docs/hld/14-development-backlog.md:4843` and
`docs/hld/14-development-backlog.md:4861`. Both passed from freshly built
`cp39-abi3` wheels together with strict mypy and stubtest. The full workspace
gate passes at the reviewed state, the hash harness matches all 49 entries,
the exact 22-crate dry run succeeds, and no archive exceeds 10 MiB. This does
not claim the unfinished Python release path, logical PDF text story, issue
wave, or M23 end gate.

## Not found

- **interaction**: zero findings. Both bindings delegate to existing native
  ownership and staging boundaries, and their installed parity gates pass.
- **duplication**: zero findings. No second document tree, relationship
  resolver, layout engine, rasterizer, or collaboration store was added.
- **layering**: zero findings. No Cargo manifest changed in this dependency
  prefix, and no forbidden lower-to-facade dependency was introduced.
- **harness**: zero findings. No baseline changed and all 49 entries match.
- **gate**: zero findings for the completed prefix. The full issue wave and M23
  end gate remain explicitly unfinished.
- **docs**: zero findings. Every HLD file named by both design plans was
  updated, and the delivery records agree with the code and tests.
- **deps**: zero findings. No dependency declaration changed.
- **surface**: zero findings. The additive immutable snapshots and facade
  methods are bounded by F-X094d and F-X094e.

## Checks

- `cargo fmt --all --check`, passed.
- `cargo clippy --workspace --all-targets --all-features --exclude rdocx-py
  --exclude rpptx-py -- -D warnings`, passed.
- `cargo test --workspace --all-features --exclude rdocx-py --exclude
  rpptx-py`, passed with pinned LibreOffice, Poppler, and corpus tooling.
- `python3 scripts/hash_harness.py --check`, passed 49 of 49.
- `python3 scripts/prose_check.py`, passed with 0 violations before this review
  file was added.
- `python3 scripts/sync_agent_skills.py --check`, passed with 26 skills in
  sync.
- `python3 -m unittest scripts.test_sprint_workflow`, passed 115 tests with 2
  skipped.
- `cargo test -p oxml-layout --no-default-features`, passed 102 tests and 3
  doctests.
- `cargo check --target wasm32-unknown-unknown -p rdocx-wasm -p rpptx-wasm`,
  passed.
- `RUSTDOCFLAGS='-D warnings' cargo doc --workspace --no-deps`, passed.
- `python3 scripts/readme_doctests.py`, passed 27 README and 22 publishable
  package inventories.
- `cargo publish --workspace --dry-run` with the 22 reviewed local patches,
  passed, and every archive remained below 10 MiB.
- `cargo deny check`, passed with the recorded duplicate and unmatched-license
  warnings.
- Fresh `rdocx` and `rpptx` `cp39-abi3` wheels passed 52 binding tests with 2
  environment-specific skips, strict mypy for both packages, and stubtest for
  all 11 extension modules.
