# S72 sprint review, pass 11

**Reviewed**: current `sprint/s72` worktree at committed SHA
`a97218fc6543ee0cb3ab6e772559652dbf2c6c7d` against merge base
`5fdbfc578adf2e5a1ceca0a3f0cee601ece44746`, including the uncommitted
pass-10 remediation and pass-8 through pass-10 reviews, 91 files, 19,508
insertions, 718 deletions, 20,226 changed lines, crates: `oxml-layout`,
`rdocx-layout`, `rdocx-oxml`, `rdocx-py`, `rdocx`
**Review boundary**: scheduled dependency-prefix boundary after F-255. The
sprint-running session explicitly extended the configured three-pass limit
through pass 11 to verify the pass-10 remediation before F-252 and F-256.
**Verdict**: 0 blocking, 0 should-fix, 3 previously recorded nice-to-have

## Blocking

None. Count: 0.

## Should-fix

None. Count: 0.

## Nice-to-have

### N1, the F-253 final review retains its recorded trailing blank line
`.claude/reviews/F-253-all-pass-21.md:46`

This review-only whitespace observation remains nonbehavioral and safe to leave
recorded.

### N2, the pass-4 sprint review retains its trailing blank line
`.claude/reviews/S72-sprint-review-pass-4.md:125`

This is also review-only whitespace with no gate effect.

### N3, the pass-5 sprint review retains its trailing blank line
`.claude/reviews/S72-sprint-review-pass-5.md:120`

The sprint-base diff check reports these three previously recorded review-file
observations. None changes behavior.

## Prior finding closure

Pass-10 B1 is closed. `add_footnote` publishes the additive typed note into the
resolved package part, refreshes the typed cache from that exact package XML,
and commits once at `crates/rdocx/src/document.rs:11466`. Related-story
discovery now borrows footnote, endnote, and comment XML only from the package
at `crates/rdocx/src/document.rs:10773`. The producer regression obtains an
existing footnote location after the typed addition, edits through it, and
proves producer root and item XML plus repeated-save bytes at
`crates/rdocx/tests/regression_test.rs:1116`.

Pass-10 B2 is closed. Public field update first stages and publishes any pending
related-story model at `crates/rdocx/src/field.rs:807`. The field transaction
patches the package-authoritative footnotes part even when its typed cache was
dirty, then marks that cache synchronized at `crates/rdocx/src/field.rs:1140`.
The inverse-order regression proves the existing field result, later typed
note, producer raw XML, reopen state, and repeated-save bytes together at
`crates/rdocx/tests/regression_test.rs:1153`.

Pass-10 B3 is closed. `update_fields` owns a complete staged `Document`
candidate at `crates/rdocx/src/field.rs:807`. Its private transaction retains
the package snapshot, applies body and physical-story changes, and records
signature invalidation from both the typed flush delta and direct package delta
before publication at `crates/rdocx/src/field.rs:1068` and
`crates/rdocx/src/field.rs:1193`. The signed endnote regression proves the
standard invalidation marker, updated field, retained producer XML, and stable
reopen bytes at `crates/rdocx/tests/regression_test.rs:1203`.

The pass-8 and pass-9 closures remain intact. Partial producer story assets use
final physical order without remapping producer drawing or relationship
identities, covered at `crates/rdocx/tests/regression_test.rs:905`. Dirty
footnote and comment histories retain the same canonical package result across
inverse picture insertion orders and later typed mutation at
`crates/rdocx/tests/regression_test.rs:998` and
`crates/rdocx/tests/regression_test.rs:1061`.

## Integrated lifecycle audit

There is one coherent authority cycle for related stories. Typed note and
comment APIs publish their physical story parts before committing at
`crates/rdocx/src/document.rs:11466` and
`crates/rdocx/src/comments.rs:348`. Generic story writes parse the written XML
back into the matching typed cache and clear its dirty state at
`crates/rdocx/src/document.rs:5067`. Traversal reads the physical relationship
targets at `crates/rdocx/src/document.rs:10773`.

Save preparation preserves that authority ordering. Typed comment identifiers
are canonicalized and the comments model is marked dirty at
`crates/rdocx/src/document.rs:8402`. Preparation then measures the retained
typed signature delta, republishes dirty notes and comments, canonicalizes
physical drawing and relationship identities, refreshes the related caches,
and performs the final package flush at `crates/rdocx/src/document.rs:9958`.
The additive footnote publication inserts only absent note identities after the
last modeled note while retaining producer root bytes and namespaces at
`crates/rdocx/src/document.rs:10197`. The cache refresh at
`crates/rdocx/src/document.rs:10310` prevents the final flush or a later public
mutation from restoring provisional package state.

The inverse orderings are covered by code inspection and the full regression
binary. Field update followed by typed note addition uses the already patched
package as its append source. Typed addition followed by field update is the
focused pass-10 predicate. Generic F-254 moves write through
`set_story_source_xml`, while later typed mutation starts from the refreshed
cache. Typed mutation followed by an F-254 or F-255 operation starts from the
immediately published package part. Canonical F-249 allocation still follows
final order, with equivalence and repeated-save gates at
`crates/rdocx/tests/regression_test.rs:25114` and
`crates/rdocx/tests/regression_test.rs:25219`. F-254 transactional move and
atomic-failure gates remain at `crates/rdocx/tests/regression_test.rs:1826` and
`crates/rdocx/tests/regression_test.rs:1962`. F-255 owner resolution and wrong
scope atomicity remain at `crates/rdocx/tests/regression_test.rs:497` and
`crates/rdocx/tests/regression_test.rs:624`.

## Milestone gate

The M23 end gate requires all five private references to be generated from a
blank public facade, reopen without repair, match required package semantics
and reviewed deterministic visual thresholds, reproduce identical DOCX bytes,
and report no unexplained preservation-only fallback
(`docs/hld/14-development-backlog.md:2214`). It does not yet hold. F-252 and
F-256 remain pending in the sprint record at
`docs/sprints/CURRENT_SPRINT.md:43` and `docs/sprints/CURRENT_SPRINT.md:44`.
F-263 remains the final five-reference corpus gate at
`docs/hld/14-development-backlog.md:2490`.

The completed prefix now satisfies its interaction gate. The full regression
binary passes all F-249 equivalence, F-254 mutation, F-255 part-scope, pass-8,
pass-9, and pass-10 remediation predicates. The current hash harness matches
all 49 entries. This is evidence for advancing the prefix, not evidence that
the unfinished M23 milestone gate is complete.

## Not found

- **interaction**: zero findings. Pass-10 B1 through B3 are closed, and no
  inverse related-story ordering or newly stale typed cache remains.
- **duplication**: zero findings. One related-story publication helper and one
  cache refresh path serve notes, comments, fields, and generic story edits.
- **layering**: zero findings. No Cargo manifest changed and no forbidden
  lower-to-facade dependency was introduced.
- **harness**: zero findings. No baseline changed and all 49 entries match.
- **gate**: zero new findings for the completed prefix. The M23 end gate remains
  unfinished and is not claimed as complete.
- **docs**: zero findings. The current HLD already records the one staged
  package boundary, final-order identifiers, and package-owned story sources.
- **deps**: zero findings. No dependency declaration changed.
- **surface**: zero findings. The remediation changes no public API.

## Checks

- `cargo test -p rdocx --test regression_test`, passed 445 tests and ignored 4.
- `cargo test -p rdocx --lib comments::tests`, passed 9 tests and ignored 1.
- `cargo test -p rdocx --lib field::tests`, passed 34 tests.
- `cargo test -p rdocx --lib`, passed 449 tests, ignored 6, and reached the one
  known environment-only failure at
  `crates/rdocx/src/document.rs:24437` because the external chart pixel
  rasterizer was unavailable. The focused affected unit suites pass.
- `cargo check -p rdocx --all-targets`, passed.
- `cargo fmt --all --check`, passed.
- `python3 scripts/hash_harness.py --check`, passed 49 of 49.
- `python3 scripts/prose_check.py`, passed with 0 violations before this review
  file was added.
- `python3 scripts/sync_agent_skills.py --check`, passed with 26 skills in sync.
- `python3 scripts/sprint_workflow.py status`, reports S72 in implementation
  with five completed stories and F-252 plus F-256 approved for later waves.
- `git diff --check`, passed for the uncommitted remediation.
- The sprint-base diff check reports only the three previously recorded review
  EOF blank lines listed under Nice-to-have.
