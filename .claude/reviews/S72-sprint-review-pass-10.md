# S72 sprint review, pass 10

**Reviewed**: current `sprint/s72` worktree at committed SHA
`a97218fc6543ee0cb3ab6e772559652dbf2c6c7d` against merge base
`5fdbfc578adf2e5a1ceca0a3f0cee601ece44746`, including the uncommitted
pass-9 remediation and pass-8 plus pass-9 reviews, 90 files, 20,363
insertions, 1,896 deletions, 22,259 changed lines, crates: `oxml-layout`,
`rdocx-layout`, `rdocx-oxml`, `rdocx-py`, `rdocx`
**Review boundary**: scheduled dependency-prefix boundary after F-255. The
sprint-running session explicitly extended the configured three-pass limit
through pass 10 to verify the pass-9 remediation before F-252 and F-256.
**Verdict**: 3 blocking, 0 should-fix, 3 previously recorded nice-to-have

## Blocking

### B1, dirty producer footnotes expose identities that become stale before mutation
`crates/rdocx/src/document.rs:10791`

When a typed footnote addition is pending, `story_sources` builds the complete
footnote source with `self.footnotes.to_xml_footnotes()`. That model does not
retain producer root children, root attributes, note attributes, or nonparagraph
note children. Public traversal therefore omits those preserved nodes and
fingerprints every pre-existing producer note from canonical typed bytes.

Every F-254 and F-255 mutation correctly starts by flushing the staged
candidate. The new additive flush preserves the producer package XML and
inserts only the pending note. That means an existing producer note no longer
has the fingerprint returned by `stories()` immediately before the mutation,
so the checked operation rejects its own current `StoryId` as stale. The
existing dirty-staging regression uses a new fully typed footnotes part, where
the two serializations happen to agree, and does not cover this producer case.

This breaks the single public traversal and mutation model required at
`docs/sprints/CURRENT_SPRINT.md:68` and the F-253 stable owner contract at
`docs/hld/14-development-backlog.md:2392`. The source used for traversal must be
the same package-authoritative XML plus pending additions that a staged flush
would publish, without mutating the document or dropping producer markup. A
regression must add a typed note to a reopened producer part, obtain an existing
note location, mutate through that location, and retain exact producer root and
item XML.

### B2, additive footnote publication silently drops an existing typed field update
`crates/rdocx/src/document.rs:10215`

The new footnote flush classifies changes only by `(id, note_type)` and appends
identities absent from the package. That is insufficient once the typed cache
contains both a pending addition and an edit to an existing note. In particular,
`update_fields_with_policy` updates the typed existing footnote but deliberately
skips its package patch whenever `footnotes_dirty` was already true at
`crates/rdocx/src/field.rs:1135`. If `add_footnote` made the cache dirty first,
the count check at lines 10226 through 10229 still succeeds, the new note is
appended, and the existing note's updated field result is silently discarded.

This violates the one complete staged candidate requirement at
`docs/sprints/CURRENT_SPRINT.md:79` and makes operation order observable across
the F-253 field traversal and the pass-9 typed-note repair. Remediation must
represent pending additions separately from edits or patch the authoritative
package source while retaining those additions. A regression must update an
existing footnote field while a new typed footnote is pending, then prove the
new note, updated cached field result, producer raw XML, repeated-save bytes,
and reopen state all survive.

### B3, package-story field updates bypass signed-package invalidation
`crates/rdocx/src/field.rs:1182`

Header, footer, footnote, and endnote field patches are written directly into
`self.package` after validation. This path neither commits a staged `Document`
candidate nor calls the synchronized package signature comparison. A document
whose only field is in an endnote can therefore change a signed package part
before `to_bytes` runs. The later retained-signature check compares a staged
flush against the already modified package, observes no delta, and leaves the
retained package signature unmarked.

The behavior contradicts the complete staged package boundary at
`docs/hld/14-development-backlog.md:2343` and the sprint-wide atomic publication
rule at `docs/sprints/CURRENT_SPRINT.md:79`. Field publication across every
physical story must commit through the same signed-package-aware boundary. A
focused regression must install the synthetic package signature graph, update
only a header, footer, footnote, or endnote field, and prove that the saved
package carries the standard invalidation marker without losing the field
result or producer XML.

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

Pass-8 B1 is closed. Partial producer related parts now discover only picture
occurrences attached through facade-authored image relationships, reserve their
global drawing ids in final physical source order, remap relationship ids from
the same source order, and sort only the authored relationship definitions
inside their original authored slots. Producer drawing ids, producer
relationship definitions, foreign prefix shadows, and raw XML remain fixed.
The focused producer-header predicate proves both histories have drawing ids
`88,77,78,79`, relationship slots `producerImage,rId0,rId1`, equal package
parts, equal relationship vectors, and equal DOCX bytes.

Pass-9 B1 is closed for its reported operation order. Dirty footnote and comment
models are flushed before related-part drawing and relationship canonicalization,
then refreshed from the canonical package before the final flush. The two new
focused predicates prove that picture insertion and movement followed by a
typed footnote or comment mutation no longer restore provisional ids or old
content order. They also prove repeated-save and reopen equality. B1 and B2
show that the same repair is incomplete when a typed footnote addition precedes
traversal or an existing-note field edit.

## Milestone gate

The M23 end gate requires all five private references to be generated from a
blank public facade, reopen without repair, match required package semantics
and reviewed deterministic visual thresholds, reproduce identical DOCX bytes,
and report no unexplained preservation-only fallback
(`docs/hld/14-development-backlog.md:2214`). It does not yet hold. F-252 and
F-256 remain pending at `docs/sprints/CURRENT_SPRINT.md:43` and
`docs/sprints/CURRENT_SPRINT.md:44`. F-263 remains the final five-reference
corpus gate at `docs/hld/14-development-backlog.md:2490`.

The completed prefix retains strong evidence for its principal story gates.
The full regression binary passes the F-254 interleaved operation matrix, the
F-255 owner-scope round trip, the related-story lifecycle matrices, and both
pass-9 remediation predicates. The current hash harness also matches all 49
entries. B1 through B3 demonstrate that the shared staged-publication gate does
not yet hold for every supported note and package-story interaction, so the
prefix is not ready to advance to F-252 or F-256.

## Not found

- **interaction**: three blocking findings. The pass-8 partial producer drawing
  and relationship interaction is repaired. Remaining failures combine typed
  footnote additions with story identities or field publication, plus physical
  story field publication with signed packages.
- **duplication**: zero findings. The remediation adds one related-story dirty
  flush and one cache refresh path rather than a second story tree or package
  owner.
- **layering**: zero findings. No Cargo manifest changed and no forbidden
  lower-to-facade dependency was introduced.
- **harness**: zero findings. No baseline changed and the current harness
  matched all 49 entries.
- **gate**: B1 through B3 block the completed-prefix staged-publication gate.
  The named owner-scope and pass-9 history predicates pass, and the unfinished
  M23 end gate is not claimed as complete.
- **docs**: zero separate findings. Current HLD text already requires stable
  story identity, package-global final-order drawing ids, and one atomic staged
  package publication boundary.
- **deps**: zero findings. No dependency declaration changed, and F-252 plus
  F-256 correctly remain pending behind this boundary.
- **surface**: zero findings. The remediation changes no public API.

## Checks

- `cargo test -p rdocx --test regression_test`, passed 442 tests and ignored 4.
- `producer_header_picture_ids_follow_final_recursive_order`, passed inside the
  full regression binary.
- `typed_footnote_flush_preserves_canonical_story_history`, passed inside the
  full regression binary.
- `typed_comment_flush_preserves_canonical_story_history`, passed inside the
  full regression binary.
- `package_backed_footnote_content_operation_survives_dirty_staging`, passed
  for its fully typed fixture and does not exercise B1's producer case.
- `typed_story_fields_have_stable_order_and_physical_parts_are_evaluated_once`,
  passed without a pre-existing dirty footnote cache or a package signature.
- `cargo fmt --all --check`, passed.
- `cargo check -p rdocx --all-targets`, passed.
- `python3 scripts/hash_harness.py --check`, passed 49 of 49.
- `python3 scripts/prose_check.py`, passed with 0 violations before this review
  file was added.
- `git diff --check`, passed for the uncommitted remediation.
- The sprint-base diff check reports only the three previously recorded review
  EOF blank lines listed under Nice-to-have.
