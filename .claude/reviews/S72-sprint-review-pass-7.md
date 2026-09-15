# S72 sprint review, pass 7

**Reviewed**: `sprint/s72` at
`cfb45f575e0e6ee2499d5af0450bcf6a1fcf589c` against merge base
`5fdbfc578adf2e5a1ceca0a3f0cee601ece44746`, 78 files, 15,066
insertions, 302 deletions, 15,368 changed lines, crates: `oxml-layout`,
`rdocx-layout`, `rdocx-oxml`, `rdocx-py`, `rdocx`
**Review boundary**: scheduled dependency-prefix boundary after F-254. This is
the approved extension beyond configured pass 3.
**Verdict**: 0 blocking, 0 should-fix, 3 previously recorded nice-to-have

## Blocking

None. Count: 0.

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

Pass-6 B1 is closed. `reject_section_owning_content_fragment` reparses the
namespace-closed fragment through the concrete `CT_Document` grammar, then
visits only typed paragraphs (`crates/rdocx/src/document.rs:4266`). Direct
paragraphs, block content controls, and nested content controls are covered.
The existing typed visitor also follows tables through rows and cells to every
modeled paragraph (`crates/rdocx/src/document.rs:6821`). Active section owners
therefore reject before all four generic operations publish a candidate.

Typed parsing supplies expanded-name handling for aliases and default Word
namespace bindings. The visitor stops at both `BodyContent::RawXml` and
`SdtContent::RawXml` (`crates/rdocx/src/document.rs:6809`,
`crates/rdocx/src/document.rs:6868`), so Word-qualified structural lookalikes
below opaque foreign roots no longer escape their preservation boundary. The
new regression begins at `crates/rdocx/tests/regression_test.rs:419` and proves
remove plus reinsert, clone, and move preserve the exact opaque bytes and
ordinary ordering. The direct and control active-owner rollback predicates at
lines 291 and 345 remain mutation-sensitive for insert, remove, clone, and
move. All three passed at the reviewed SHA.

## Milestone gate

The M23 end gate requires all five private references to be generated from a
blank public facade, reopen without repair, match required package semantics
and reviewed deterministic visual thresholds, reproduce identical DOCX bytes,
and report no unexplained preservation-only fallback
(`docs/hld/14-development-backlog.md:2214`). It does not yet hold. F-255,
F-252, and F-256 remain pending (`docs/sprints/CURRENT_SPRINT.md:42`).

The completed dependency prefix is clean and its four story gates hold. F-253
visits and mutates the same supported content shape in every story
(`crates/rdocx/tests/integration_test.rs:102`). F-250 preserves ordered section
and story ownership (`crates/rdocx/tests/integration_test.rs:915`). F-251's
pinned Word geometry and numbering predicate begins at
`crates/rdocx/tests/integration_test.rs:685`. F-254's interleaved operation gate
begins at `crates/rdocx/tests/regression_test.rs:80`, and the complete
regression binary passed 427 tests with 4 ignored after the boundary fixes.

F-255 is ready to claim because its F-253 and F-249 foundations are complete
and the integrated prefix has no blocking finding. F-252's F-250 and F-253
foundations are complete, but the sprint sequence correctly keeps it after
F-255 so rich header and footer content uses part-scoped relationships
(`docs/sprints/CURRENT_SPRINT.md:50`). F-256 remains unready until F-255 and
F-252 complete its F-246 through F-255 dependency range
(`docs/hld/14-development-backlog.md:2430`).

## Not found

- **interaction**: zero findings. F-254 now preserves the F-250 section graph
  boundary across direct paragraphs, controls, tables, aliases, opaque raw
  descendants, rollback, and ordinary generic operations. No other joint
  mismatch among F-253, F-250, F-251, and F-254 was found.
- **duplication**: zero findings. The remediation reuses the existing typed
  paragraph visitor and adds no second grammar, story tree, section tree, or
  mutation path.
- **layering**: zero findings. No Cargo manifest changed and no forbidden
  lower-to-facade dependency was introduced.
- **harness**: zero findings. No baseline changed, the remediation declares an
  unchanged result, and the current harness matched all 49 entries.
- **gate**: zero findings. Every completed story's named predicate exists and
  passes. The unfinished M23 end gate is not claimed as complete.
- **docs**: zero findings. The HLD and delivery records match the four completed
  public surfaces and retain F-255, F-252, and F-256 as live owners.
- **deps**: zero findings. No dependency, feature declaration, crate, module,
  or source file was added. The run-state wave order retains F-255 before F-252
  and F-256.
- **surface**: zero findings. The remediation adds no public API. The integrated
  story, content, section, and geometry surfaces remain those approved by
  F-253, F-254, F-250, and F-251.

## Checks

- `cargo test -p rdocx --test regression_test`, passed 427 tests and ignored 4.
- Both active section-owner rollback predicates passed.
- `generic_content_operations_preserve_word_lookalikes_below_opaque_roots`,
  passed.
- `removing_a_section_never_orphans_a_shared_story`, passed.
- `cargo check -p rdocx --all-targets`, passed.
- `cargo fmt --all --check`, passed.
- `python3 scripts/hash_harness.py --check`, passed 49 of 49.
