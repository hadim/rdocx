# S72 sprint review, pass 12

**Reviewed**: current `sprint/s72` worktree at committed SHA
`7c9673d398232dec70cd3b6f0e3846ec1d23152b` against merge base
`5fdbfc578adf2e5a1ceca0a3f0cee601ece44746`, including the uncommitted
17-insertion and 7-deletion pass-11 boundary correction, 92 files, 19,688
insertions, 716 deletions, 20,404 changed lines, crates: `oxml-layout`,
`rdocx-layout`, `rdocx-oxml`, `rdocx-py`, `rdocx`
**Review boundary**: scheduled dependency-prefix boundary after F-255. The
sprint-running session explicitly extended the configured three-pass limit
through pass 12 to audit the comment output-boundary correction before F-252
and F-256.
**Verdict**: 1 blocking, 0 should-fix, 3 previously recorded nice-to-have

## Blocking

### B1, Flat OPC bypasses canonical comment output preparation
`crates/rdocx/src/flat_opc.rs:86`

The correction introduces `prepare_staged_output` as the public package-output
boundary. It marks an existing comments model dirty before normal staged
preparation at `crates/rdocx/src/document.rs:9970`. DOCX byte and path output,
package-class output, encryption, and signing now all use that boundary at
`crates/rdocx/src/document.rs:9521`, `crates/rdocx/src/document.rs:9575`,
`crates/rdocx/src/document.rs:9904`, `crates/rdocx/src/document.rs:9916`, and
`crates/rdocx/src/document.rs:9942`.

Flat OPC byte export is the remaining public package output, but it still calls
`prepare_staged_package` directly. A reopened comments model starts clean at
`crates/rdocx/src/document.rs:9769`, so this path never republishes it. The Flat
OPC writer then copies the current package part bytes at
`crates/rdocx/src/flat_opc.rs:471` and writes their XML events at
`crates/rdocx/src/flat_opc.rs:511`. For the existing custom-target fixture, ZIP
output canonicalizes producer `<x:comments>` to `<w:comments>` while preserving
`/custom/comments-data.xml`, as required at
`crates/rdocx/tests/integration_test.rs:8722`. Flat OPC output instead retains
the producer root. The two public serializers therefore no longer share the
document's declared comments output contract at
`docs/hld/04-opc-and-packaging.md:420`.

This also leaves Flat OPC outside the signature measurement performed after
the dirty flag is set. Remediation should make the output preparation boundary
available within the crate and call it from `to_flat_opc_bytes`, which also
covers `save_flat_opc`. Extend the custom-target predicate through Flat OPC and
prove the resolved target, fixed `w:` root, raw children, signature state,
reopen semantics, and repeated output bytes match the ZIP boundary.

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

## Boundary correction audit

The corrected ZIP boundary preserves the existing comments relationship
target. `from_package` resolves and stores that target at
`crates/rdocx/src/document.rs:9710`, the output boundary marks that same model
dirty at `crates/rdocx/src/document.rs:9970`, and the dirty flush writes only to
the stored part name at `crates/rdocx/src/document.rs:10254`. The unchanged
custom-target integration predicate passes and confirms that no conventional
`/word/comments.xml` part appears.

Comment story traversal remains package-authoritative. `story_sources` reads
the relationship-resolved package part and returns an owned copy of those
current bytes for comments at `crates/rdocx/src/document.rs:10778`. This keeps
the established typed-source ownership contract without serializing the typed
cache during ordinary traversal. The exact ownership predicate passes at
`crates/rdocx/tests/integration_test.rs:323`.

Ordinary staged story mutation still uses `prepare_staged_package`, not the
output-only canonicalization boundary. It validates a `StoryId` against the
same current package bytes and writes the accepted story XML back through
`set_story_source_xml` at `crates/rdocx/src/document.rs:5067`. The complete
all-story mutation predicate passes at
`crates/rdocx/tests/integration_test.rs:101`, including stale-location rejection
and comment mutation after every preceding story edit.

The fixed output boundary preserves raw comment XML through the typed comments
model, canonicalizes the root and Word namespace once, and retains existing
identifiers. The exact three-comment bytes predicate passes at
`crates/rdocx/tests/integration_test.rs:8621`, and the owner-order plus raw-node
predicate passes at `crates/rdocx/tests/integration_test.rs:193`. Signing with
the digital-signatures feature also passes its canonical preparation and
subsequent-save verification predicate at
`crates/rdocx/src/document.rs:20668`.

The pass-8 through pass-11 related-story lifecycle corrections remain intact.
The full regression binary passes the producer asset ordering predicates at
`crates/rdocx/tests/regression_test.rs:905`, the typed footnote and comment
history predicates at `crates/rdocx/tests/regression_test.rs:998` and
`crates/rdocx/tests/regression_test.rs:1061`, the current producer footnote
identity predicate at `crates/rdocx/tests/regression_test.rs:1116`, the inverse
footnote field update predicate at `crates/rdocx/tests/regression_test.rs:1153`,
and the signed package-story field predicate at
`crates/rdocx/tests/regression_test.rs:1203`.

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

The completed prefix is not ready to advance while B1 leaves one public package
writer outside the corrected comments and signature boundary. All other
pass-12 predicates and the current 49-entry hash harness are green.

## Not found

- **interaction**: one blocking finding. ZIP output, signing, generic story
  mutation, and package-authoritative comment traversal are coherent. Flat OPC
  is the sole missed public package-output interaction.
- **duplication**: zero findings. The correction adds one small output wrapper
  over the existing staged-package path.
- **layering**: zero findings. No Cargo manifest changed and no forbidden
  lower-to-facade dependency was introduced.
- **harness**: zero findings. No baseline changed and all 49 entries match.
- **gate**: B1 blocks the completed-prefix output gate. The M23 end gate remains
  unfinished and is not claimed as complete.
- **docs**: zero separate findings. Current HLD text already requires saving to
  serialize the typed comments model back to its resolved target.
- **deps**: zero findings. No dependency declaration changed.
- **surface**: zero findings. The correction changes no public API.

## Checks

- `cargo test -p rdocx --test regression_test`, passed 445 tests and ignored 4.
- `cargo test -p rdocx --test integration_test`, passed 220 tests, ignored 4,
  and reached only the two known environment failures at
  `crates/rdocx/tests/integration_test.rs:4546` and
  `crates/rdocx/tests/integration_test.rs:1864` because pinned LibreOffice is
  unavailable.
- `comments_part_uses_its_existing_relationship_target`, passed alone.
- `story_xml_distinguishes_owned_typed_sources_from_borrowed_package_slices`,
  passed alone.
- `one_generic_mutation_edits_the_same_shape_in_every_story`, passed alone.
- `three_comments_and_cross_paragraph_anchors_round_trip_byte_identically`,
  passed alone.
- `story_traversal_preserves_owner_order_and_raw_nodes`, passed alone.
- `comments_extended_part_uses_its_existing_relationship_target`, passed alone.
- `cargo test -p rdocx --features digital-signatures --lib
  signing_and_subsequent_save_share_canonical_package_preparation`, passed.
- `cargo check -p rdocx --all-targets`, passed.
- `cargo fmt --all --check`, passed.
- `python3 scripts/hash_harness.py --check`, passed 49 of 49.
- `python3 scripts/prose_check.py`, passed with 0 violations before this review
  file was added.
- `python3 scripts/sync_agent_skills.py --check`, passed with 26 skills in sync.
- `git diff --check`, passed for the uncommitted correction.
- The sprint-base diff check reports only the three previously recorded review
  EOF blank lines listed under Nice-to-have.
