# S72 sprint review, pass 13

**Reviewed**: current `sprint/s72` worktree at committed SHA
`7c9673d398232dec70cd3b6f0e3846ec1d23152b` against merge base
`5fdbfc578adf2e5a1ceca0a3f0cee601ece44746`, including the uncommitted
pass-12 output-boundary remediation and pass-12 review, 94 files, 19,921
insertions, 732 deletions, 20,653 changed lines, crates: `oxml-layout`,
`rdocx-layout`, `rdocx-oxml`, `rdocx-py`, `rdocx`
**Review boundary**: scheduled dependency-prefix boundary after F-255. The
sprint-running session explicitly extended the configured three-pass limit
through pass 13 to verify pass-12 B1 and the complete public package-output
caller set before F-252 and F-256.
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

Pass-12 B1 is closed. `prepare_staged_output` now has crate visibility at
`crates/rdocx/src/document.rs:9970`, and Flat OPC byte export calls it before
signature persistence and package projection at
`crates/rdocx/src/flat_opc.rs:84`. Flat OPC path save delegates to that byte
writer at `crates/rdocx/src/flat_opc.rs:97`, so both outputs share the corrected
comments and signature boundary.

The expanded custom-target predicate starts with an aliased comments root,
comment-level and root-level raw children, a nonconventional relationship
target, and a retained package-signature graph at
`crates/rdocx/tests/integration_test.rs:8722`. It proves ZIP and Flat OPC both
retain `/custom/comments-data.xml`, never create `/word/comments.xml`, emit the
fixed `w:` comments root, preserve both raw children exactly, retain the
signature part, write the invalidation marker, repeat byte-identically, reopen
with the typed comment, and remain stable across Flat OPC back to ZIP.

## Public package-output caller audit

Every public Word package output now reaches the canonical output boundary.

| Output | Boundary evidence |
|---|---|
| ZIP bytes | `to_bytes` calls it at `crates/rdocx/src/document.rs:9917` |
| ZIP path | `save` calls it at `crates/rdocx/src/document.rs:9905` |
| Package-class bytes | `to_bytes_as` calls it at `crates/rdocx/src/document.rs:9521` |
| Package-class path | `save_as_package_class` delegates to `to_bytes_as` at `crates/rdocx/src/document.rs:9545` |
| Signed package | `sign` calls it before signing at `crates/rdocx/src/document.rs:9575` |
| Encrypted bytes | `to_encrypted_bytes` calls it at `crates/rdocx/src/document.rs:9944` |
| Encrypted path | `save_encrypted` delegates to `to_encrypted_bytes` at `crates/rdocx/src/document.rs:9931` |
| Flat OPC bytes | `to_flat_opc_bytes` calls it at `crates/rdocx/src/flat_opc.rs:84` |
| Flat OPC path | `save_flat_opc` delegates to `to_flat_opc_bytes` at `crates/rdocx/src/flat_opc.rs:97` |

Field-update output helpers also terminate in the ZIP methods at
`crates/rdocx/src/field.rs:1312`. No direct package writer remains outside the
canonical boundary in non-test code.

The boundary remains output-specific. Internal preparation at
`crates/rdocx/src/document.rs:9958` and the validated reopen path at
`crates/rdocx/src/document.rs:9975` do not force an unrelated clean comments
model through the typed serializer. F-254 and F-255 mutations therefore
validate the caller's `StoryId` against current physical story bytes rather
than an output-only projection.

Comment traversal retains its established ownership surface while using the
package as authority. `story_sources` resolves the current comments
relationship target, copies those exact package bytes into `Cow::Owned`, and
does not consult a potentially stale typed serialization at
`crates/rdocx/src/document.rs:10778`. Generic writes refresh the typed cache and
package together through `set_story_source_xml` at
`crates/rdocx/src/document.rs:5067`. The exact owned-source predicate passes at
`crates/rdocx/tests/integration_test.rs:324`, and the complete all-story
mutation and stale-location predicate passes at
`crates/rdocx/tests/integration_test.rs:102`.

Output is atomic with respect to the live document because every byte and path
writer prepares a staged clone. Signing is the intentional publishing
exception. Its output preparation refreshes related caches before the signed
candidate is committed, so the typed comment model and canonical package part
remain synchronized. The feature-gated signing predicate passes at
`crates/rdocx/src/document.rs:20670`. The feature-gated encryption predicate at
`crates/rdocx/src/document.rs:22098` confirms encrypted bytes and path output do
not mutate the live package.

The unchanged raw-node and canonical identifier evidence remains green. Exact
three-comment round-trip bytes pass at
`crates/rdocx/tests/integration_test.rs:8622`, while owner order and raw nodes
pass at `crates/rdocx/tests/integration_test.rs:194`. The full regression binary
also passes the pass-8 through pass-11 producer ordering, typed comment and
footnote history, current `StoryId`, inverse field update, signature, repeated
save, F-249 equivalence, F-254 mutation, and F-255 owner-scope predicates.

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

The completed prefix now satisfies its package-output and related-story
interaction gate. The current regression, focused integration, signing,
encryption, and 49-entry hash evidence are green. This does not claim the
unfinished M23 end gate.

## Not found

- **interaction**: zero findings. Every public package writer uses the output
  boundary, while internal story staging retains physical package authority.
- **duplication**: zero findings. One crate-visible output helper serves every
  byte and path output directly or through delegation.
- **layering**: zero findings. No Cargo manifest changed and no forbidden
  lower-to-facade dependency was introduced.
- **harness**: zero findings. No baseline changed and all 49 entries match.
- **gate**: zero new findings for the completed prefix. The M23 end gate remains
  unfinished and is not claimed as complete.
- **docs**: zero findings. The HLD already requires typed comments to serialize
  to their resolved relationship target and all package outputs to stage.
- **deps**: zero findings. No dependency declaration changed.
- **surface**: zero findings. `prepare_staged_output` is crate-visible only, and
  no published API changed.

## Checks

- `cargo test -p rdocx --test regression_test`, passed 445 tests and ignored 4.
- `comments_part_uses_its_existing_relationship_target`, passed alone with its
  expanded ZIP, Flat OPC, raw XML, signature, repeat, and reopen assertions.
- `story_xml_distinguishes_owned_typed_sources_from_borrowed_package_slices`,
  passed alone.
- `one_generic_mutation_edits_the_same_shape_in_every_story`, passed alone.
- `three_comments_and_cross_paragraph_anchors_round_trip_byte_identically`,
  passed alone.
- `story_traversal_preserves_owner_order_and_raw_nodes`, passed alone.
- `cargo test -p rdocx --features digital-signatures --lib
  signing_and_subsequent_save_share_canonical_package_preparation`, passed.
- `cargo test -p rdocx --features agile-encryption --lib
  native_encrypted_save_round_trips_without_live_package_mutation`, passed.
- `cargo check -p rdocx --all-targets`, passed.
- `cargo fmt --all --check`, passed.
- `python3 scripts/hash_harness.py --check`, passed 49 of 49.
- `python3 scripts/prose_check.py`, passed with 0 violations before this review
  file was added.
- `python3 scripts/sync_agent_skills.py --check`, passed with 26 skills in sync.
- `git diff --check`, passed for the uncommitted remediation.
- The sprint-base diff check reports only the three previously recorded review
  EOF blank lines listed under Nice-to-have.
