# S72 sprint review, pass 1

**Reviewed**: `sprint/s72` at `15682f05ca71` against
`5fdbfc578adf2e5a1ceca0a3f0cee601ece44746`, 48 files, 7,819 changed lines,
crates: `rdocx-oxml`, `rdocx`, `rdocx-py`
**Verdict**: 0 blocking, 0 should-fix, 0 nice-to-have

## Blocking

None. Count: 0.

## Should-fix

None. Count: 0.

## Nice-to-have

None. Count: 0.

## Milestone gate

The M23 end gate requires all five private references to be generated from the
blank public facade, reopen without repair, match required package semantics
and reviewed deterministic visual thresholds, produce identical DOCX bytes on
repeat, and report no unexplained preservation-only fallback
(`docs/hld/14-development-backlog.md:2214`). That end gate does not yet hold at
this dependency-prefix checkpoint. Six S72 stories remain pending
(`docs/sprints/CURRENT_SPRINT.md:39`).

The completed F-253 prefix gate does hold. The backlog requires one generic
mutation to visit and edit the same supported content shape in every story with
identical error behavior (`docs/hld/14-development-backlog.md:2384`).
`one_generic_mutation_edits_the_same_shape_in_every_story` exercises body,
cell, text-box, header, footer, footnote, endnote, and comment owners and checks
the shared wrong-owner, kind, bounds, and stale failures
(`crates/rdocx/tests/integration_test.rs:94`).
`story_traversal_preserves_owner_order_and_raw_nodes` pins owner and item order
plus preserved XML across save and reopen
(`crates/rdocx/tests/integration_test.rs:186`).
`invalid_story_locations_are_atomic` verifies unchanged serialized bytes after
invalid mutations (`crates/rdocx/tests/regression_test.rs:1687`). The delivery
record reports the integrated full gate at `035b74a3d83f` and an unchanged
49-of-49 hash harness (`docs/sprints/AS_BUILT.md:12819`).

## Not found

- **interaction**: F-253 is the only implemented prefix story. Its staged
  location model leaves the section, relationship, and fragment APIs for the
  pending dependent stories, matching the recorded wave order
  (`docs/sprints/CURRENT_SPRINT.md:50`).
- **duplication**: no second public story tree or competing location resolver
  was added. Traversal and mutation share the existing facade scanner and
  checked source resolver (`crates/rdocx/src/document.rs:8992`).
- **layering**: no manifest changed, and the lower `rdocx-oxml` crate gained no
  dependency on an `rdocx-*` or `rpptx-*` crate.
- **harness**: no baseline file changed. The unchanged 49-of-49 result is
  declared in the F-253 completion record (`docs/sprints/AS_BUILT.md:12826`).
- **gate**: the F-253 integration, round-trip, atomicity, Python classification,
  and completed-capability owner evidence align with the implemented surface.
  The workflow invariant removes only completed F-253 from the expected live
  owner set (`scripts/test_sprint_workflow.py:8294`).
- **docs**: every HLD file named by the completed design plan was updated. The
  capability matrix marks DOCX-018 complete and clears its live owner
  (`docs/hld/02-scope-and-non-goals.md:225`).
- **deps**: no dependency or feature declaration changed.
- **surface**: the added concrete story identities, item views, staged text
  mutation, and error type are the native Rust API approved by the F-253 plan.
  Python receives only exhaustive error classification, not an unplanned story
  API (`crates/rdocx-py/src/lib.rs:68`).
