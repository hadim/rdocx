# S72 sprint review, pass 8

**Reviewed**: `sprint/s72` at
`a97218fc6543ee0cb3ab6e772559652dbf2c6c7d` against merge base
`5fdbfc578adf2e5a1ceca0a3f0cee601ece44746`, 87 files, 19,381
insertions, 1,861 deletions, 21,242 changed lines, crates: `oxml-layout`,
`rdocx-layout`, `rdocx-oxml`, `rdocx-py`, `rdocx`
**Review boundary**: scheduled dependency-prefix boundary after F-255. The user
explicitly approved this pass-8 extension beyond the configured three-pass
limit so the integrated F-255 result could be reviewed before F-252 and F-256.
**Verdict**: 1 blocking, 0 should-fix, 3 previously recorded nice-to-have

## Blocking

### B1, producer-related story pictures retain edit-history drawing ids
`crates/rdocx/src/document.rs:9163`

Related-story canonicalization correctly distinguishes a fully authored part
from a producer part with only facade-authored relationships at lines 9059
through 9066. It source-orders and remaps the authored relationships in both
cases. The drawing pass, however, calls `rewrite_authored_doc_pr_ids` only when
the complete related part is facade-authored. A picture appended to an existing
producer header, footer, note, or comment therefore keeps the drawing id
reserved at insertion time even after F-254 moves reorder its paragraph.

A public-facade probe against the reviewed SHA reproduced the failure with one
producer header and two unequal images. Adding A then B and moving A after B
serialized final `wp:docPr` ids `2,1`. Adding B then A serialized the same final
semantic content with ids `1,2`. The relationship ids and media names were
canonical in both results, but the DOCX byte vectors differed. The allocation
at `crates/rdocx/src/document.rs:10780` and the partially authored branch at
`crates/rdocx/src/document.rs:9060` therefore lack the occurrence provenance or
equivalent source-order rewrite that the main part and fully authored related
parts receive.

This violates the F-249 final recursive document-order contract at
`docs/hld/14-development-backlog.md:2341`, the F-255 global per-occurrence
drawing contract at `docs/hld/14-development-backlog.md:2423`, and the sprint's
deterministic publication requirement at `docs/sprints/CURRENT_SPRINT.md:79`.
It is blocking before F-252 because that story must expose rich mutation of
existing per-section header and footer stories, including pictures, through the
same content operations. Remediation needs to retain producer drawing ids as
fixed global occupants while source-ordering only live facade-authored picture
occurrences in partially authored related parts. A regression should prove
equivalent insert and reorder histories in a producer header yield identical
bytes and preserve producer raw XML.

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

Pass 7 closed the F-254 opaque-subtree section-owner defect and reported no
blocking or should-fix finding at the four-story prefix. That closure remains
valid. The typed section-owner guard still stops at preserved raw grammar
boundaries, and the complete current regression binary passed its active-owner
and opaque-lookalike predicates.

F-255 microscope pass 5 reported no feature-local finding. Its main-part
interaction repairs remain effective. Live relationship occurrences are
reconciled before remapping at `crates/rdocx/src/document.rs:8957`, and modeled
body plus nested cell and text-box pictures are allocated together in source
order at `crates/rdocx/src/document.rs:8892`. The current lifecycle regressions
at `crates/rdocx/tests/regression_test.rs:916`,
`crates/rdocx/tests/regression_test.rs:974`, and
`crates/rdocx/tests/regression_test.rs:1054` passed. B1 is the remaining
interaction with producer-owned related parts, which those main-part lifecycle
matrices do not cover.

## Milestone gate

The M23 end gate requires all five private references to be generated from a
blank public facade, reopen without repair, match required package semantics
and reviewed deterministic visual thresholds, reproduce identical DOCX bytes,
and report no unexplained preservation-only fallback
(`docs/hld/14-development-backlog.md:2214`). It does not yet hold. F-252 and
F-256 remain pending at `docs/sprints/CURRENT_SPRINT.md:43` and
`docs/sprints/CURRENT_SPRINT.md:44`. F-263 remains the final five-reference
corpus gate at `docs/hld/14-development-backlog.md:2490`.

The completed prefix has strong individual story evidence. F-253's generic
story mutation predicate begins at `crates/rdocx/tests/integration_test.rs:102`.
F-250's shared-story section removal predicate begins at
`crates/rdocx/tests/integration_test.rs:1224`. F-251's pinned geometry and page
number predicate begins at `crates/rdocx/tests/integration_test.rs:685`. F-254's
interleaved content gate begins at
`crates/rdocx/tests/regression_test.rs:1267`. F-255's owner-scope round trip,
scope rejection, typed-footnote publication, and enclosing-owner interaction
predicates begin at `crates/rdocx/tests/regression_test.rs:288`,
`crates/rdocx/tests/regression_test.rs:415`,
`crates/rdocx/tests/regression_test.rs:597`, and
`crates/rdocx/tests/regression_test.rs:916`. All passed in the current regression
or focused integration runs.

The prefix is not ready to advance while B1 remains. F-252 explicitly requires
rich headers and footers to accept images and survive lifecycle changes at
`.claude/plans/F-252-design.md:43`. F-256 correctly retains F-252 and F-255 in
its complete dependency range at `.claude/plans/F-256-design.md:6`, and its
planned closure reserves package-global drawing identities separately from
part-local relationships at `.claude/plans/F-256-design.md:30`. No ownership
contract needs to move between stories. F-253 continues to own logical story
identity, F-254 owns same-story structural mutation, F-255 owns physical OPC
relationship scope, and F-256 owns cross-owner and cross-document remapping.

## Not found

- **interaction**: one blocking finding, B1. Main-part body, cell, and text-box
  assets retain correct story identity, physical owner scope, relationship
  semantics, and fresh per-occurrence drawing ids across F-254 operations. No
  other joint mismatch among F-250 through F-255 was found.
- **duplication**: zero findings. F-255 reuses the concrete `StoryId` and staged
  facade paths and introduces no second story tree, relationship abstraction,
  or package owner.
- **layering**: zero findings. No Cargo manifest changed and no forbidden
  lower-to-facade dependency was introduced.
- **harness**: zero findings. No baseline changed and the current hash harness
  matched all 49 entries.
- **gate**: B1 blocks the deterministic portion of the completed F-255 contract.
  The named owner-scope round-trip predicate passes, and the unfinished M23 end
  gate is not claimed as complete.
- **docs**: zero separate findings. The HLD and delivery records state the
  intended part-local relationship and package-global drawing contracts. B1 is
  an implementation mismatch with those current-intent documents.
- **deps**: zero findings. The wave order retains F-255 before F-252 and both
  before F-256, with F-256 owning the full F-246 through F-255 closure.
- **surface**: zero findings. The additive native Rust operations match the
  approved F-255 plan and add no Python, WASM, or CLI surface.

## Checks

- `cargo test -p rdocx --test regression_test`, passed 439 tests and ignored 4.
- `one_generic_mutation_edits_the_same_shape_in_every_story`, passed.
- `removing_a_section_never_orphans_a_shared_story`, passed.
- `mixed_orientation_sections_match_word_geometry_and_page_numbers`, passed.
- The producer-header public-facade probe reproduced B1 with unequal final DOCX
  bytes and `wp:docPr` orders `2,1` versus `1,2`.
- `python3 scripts/hash_harness.py --check`, passed 49 of 49.
- F-255's completion record reports the integrated `/verify --full` gate passed
  at `009e5c4d16a5517cd215d0dd97d7fc7bea569b8e`, with all 22 package dry runs
  below 10 MiB (`docs/sprints/AS_BUILT.md:13012`).
