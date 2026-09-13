# S72 sprint review, pass 9

**Reviewed**: current `sprint/s72` worktree at committed SHA
`a97218fc6543ee0cb3ab6e772559652dbf2c6c7d` against merge base
`5fdbfc578adf2e5a1ceca0a3f0cee601ece44746`, including the uncommitted B1
remediation and pass-8 review, 88 files, 19,736 insertions, 1,861 deletions,
21,597 changed lines, crates: `oxml-layout`, `rdocx-layout`, `rdocx-oxml`,
`rdocx-py`, `rdocx`
**Review boundary**: scheduled dependency-prefix boundary after F-255. The user
explicitly approved this pass-9 extension beyond the configured three-pass
limit to review the pass-8 B1 remediation before F-252 and F-256.
**Verdict**: 1 blocking, 0 should-fix, 3 previously recorded nice-to-have

## Blocking

### B1, a typed related-story flush overwrites canonical package XML
`crates/rdocx/src/document.rs:9849`

The remediation closes the package-backed producer-header case from pass 8.
It finds picture occurrences through only facade-authored image relationships,
removes their provisional ids from the global occupied set, and reallocates
them in related-part source order at lines 9057 through 9095 and 9205 through
9220. It also sorts only remapped authored relationship definitions inside
their existing slots at lines 9169 through 9202, leaving producer definitions
and their order untouched. The focused header regression at
`crates/rdocx/tests/regression_test.rs:729` proves exact final drawing ids,
whole-package byte equality, slot-local authored relationship order, preserved
producer drawing and relationship bytes, a shadowed `wp` lookalike, and raw XML.

The same canonical result is not published into typed related-story state.
`prepare_staged_package` canonicalizes the package and then calls
`flush_to_package`. A dirty footnotes cache serializes over the just-rewritten
part at `crates/rdocx/src/document.rs:10047`. Comments do the same whenever a
typed comments owner exists at `crates/rdocx/src/document.rs:10096`. Generic
footnote mutation makes the mismatch worse because
`set_story_source_xml` updates the package and clears dirty state without
updating the typed footnote owner at `crates/rdocx/src/document.rs:5072`. A
later typed footnote operation marks that stale owner dirty again at
`crates/rdocx/src/document.rs:11371`.

A public-facade probe against this worktree reproduced the remaining failure.
Starting from the same reopened producer footnote, one path inserted A then B,
moved A after B, and added a later typed footnote. The other inserted B then A
and added the same typed footnote. The operations were intended to converge on
the same story order. The resulting DOCX vectors differed, with lengths 6,370
and 6,372 bytes, and their picture `wp:docPr` orders remained `2,1` and `1,2`.
The stale typed flush can therefore undo an F-254 story move as well as restore
pre-canonical drawing and relationship references. For comments, the typed
owner has the moved content order, but its old relationship ids can resolve
against newly canonicalized definitions with the opposite image semantics.

This remains the pass-8 blocker because F-255 promises one staged package and
typed candidate, package-global final-order drawing ids, and deterministic
bytes. It blocks F-252's rich header and footer lifecycle and F-256's complete
dependency remapping. Remediation must make canonical related-story XML the
single state published by the final flush, including dirty footnotes and typed
comments. Focused regressions must cover equivalent reorder histories followed
by a typed footnote mutation and the corresponding typed comment path. They
must assert exact bytes, item order, relationship resolution, final drawing
order, and producer raw preservation.

## Should-fix

None. Count: 0.

## Nice-to-have

### N1, the F-253 final review retains its recorded trailing blank line
`.claude/reviews/F-253-all-pass-21.md:46`

This review-only whitespace observation remains non-behavioral and safe to
leave recorded.

### N2, the pass-4 sprint review retains its trailing blank line
`.claude/reviews/S72-sprint-review-pass-4.md:125`

This is also review-only whitespace with no gate effect.

### N3, the pass-5 sprint review retains its trailing blank line
`.claude/reviews/S72-sprint-review-pass-5.md:120`

`git diff --check 5fdbfc578adf2e5a1ceca0a3f0cee601ece44746..HEAD`
reports these three previously recorded review-file observations. None changes
behavior.

## Prior finding closure

Pass-8 B1 is partially closed. The package-only producer header path now has
the required namespace-aware drawing occurrence selection, global final-order
allocation, authored relationship slot normalization, producer relationship
and drawing preservation, raw-byte retention, and equivalent-history byte
equality. The regression exercises a real producer `wp:docPr` id, a foreign
shadow of the conventional prefix, and an exact raw sentinel. It also compares
both package parts and the header relationship vector before asserting complete
DOCX byte equality.

The main-part and fully authored related-part paths remain unchanged by the new
partial-part branch. Main body, table-cell, and text-box occurrence provenance
still reconciles against live XML at `crates/rdocx/src/document.rs:8957`.
Fully authored related parts continue through
`rewrite_authored_doc_pr_ids` at `crates/rdocx/src/document.rs:9231`. The full
regression run passed the main-part reorder, clone, removal, enclosing-owner,
and fully authored header history predicates. The remaining B1 is confined to
the later typed flush of partially authored note and comment parts.

## Milestone gate

The M23 end gate requires all five private references to be generated from a
blank public facade, reopen without repair, match required package semantics
and reviewed deterministic visual thresholds, reproduce identical DOCX bytes,
and report no unexplained preservation-only fallback
(`docs/hld/14-development-backlog.md:2214`). It does not yet hold. F-252 and
F-256 remain pending at `docs/sprints/CURRENT_SPRINT.md:43` and
`docs/sprints/CURRENT_SPRINT.md:44`. F-263 remains the final five-reference
corpus gate at `docs/hld/14-development-backlog.md:2490`.

The completed prefix's named story predicates continue to pass. F-253's generic
story mutation gate begins at `crates/rdocx/tests/integration_test.rs:102`.
F-250's shared-story removal gate begins at
`crates/rdocx/tests/integration_test.rs:1224`. F-251's pinned geometry gate
begins at `crates/rdocx/tests/integration_test.rs:685`. F-254's interleaved
content gate begins at `crates/rdocx/tests/regression_test.rs:1393` after this
remediation's inserted test lines. F-255's owner-scope round trip begins at
`crates/rdocx/tests/regression_test.rs:321`. The full current regression binary
passed, but B1 proves that the broader deterministic publication contract does
not yet hold for every supported related-story state.

The prefix is not ready to advance. F-252 requires rich story content to
survive save, reopen, replacement, and inheritance changes at
`.claude/plans/F-252-design.md:43`. F-256 retains F-252 and F-255 in its complete
dependency range at `.claude/plans/F-256-design.md:6`. Its planned closure also
depends on a single atomic package and typed publication boundary at
`.claude/plans/F-256-design.md:37`.

## Not found

- **interaction**: one blocking finding, B1. The header-specific F-254 and F-255
  interaction is repaired. The remaining typed note and comment publication
  path is jointly inconsistent with the same ownership and mutation contracts.
- **duplication**: zero findings. The remediation extends the existing concrete
  source-order drawing scanner and related-part canonicalization path without a
  second story tree or identifier owner.
- **layering**: zero findings. No manifest changed and no forbidden
  lower-to-facade dependency was introduced.
- **harness**: zero findings. No baseline changed and the current harness
  matched all 49 entries.
- **gate**: B1 continues to block F-255's deterministic publication contract.
  The named round-trip predicate and the new header-focused predicate pass, and
  the unfinished M23 end gate is not claimed as complete.
- **docs**: zero separate findings. Current HLD text already requires
  package-global per-occurrence drawing ids and one validated staged candidate.
- **deps**: zero findings. No dependency or feature declaration changed, and
  the approved F-255, F-252, then F-256 wave order remains necessary.
- **surface**: zero findings. The remediation changes no public API.

## Checks

- `producer_header_picture_ids_follow_final_recursive_order`, passed.
- `cargo test -p rdocx --test regression_test`, passed 440 tests and ignored 4.
- The typed-footnote public-facade probe reproduced B1 with unequal DOCX bytes
  and `wp:docPr` orders `2,1` versus `1,2`.
- `python3 scripts/hash_harness.py --check`, passed 49 of 49.
