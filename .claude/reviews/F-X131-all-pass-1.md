# F-X131, all, pass 1

**Reviewed**: the working tree diff for F-X131. Four code and record files,
`crates/rdocx-oxml/src/text.rs`, `crates/rdocx-oxml/src/revision.rs`,
`scripts/test_sprint_workflow.py` and the two backlog files, 152 insertions and
39 deletions across seven paths.
**Verdict**: 0 defects, 0 smells, 2 nitpicks

## Scope

The diff carries two behaviour changes and their tests. The capture side stops
recording a namespace declaration no retained attribute uses. The write side
stops rebinding `w14` on an element whose part root already owns it. The plan
described the first. The second was found while running the wider gate and is
recorded in this review and in the plan's problem statement.

## Contract

`capture_root_attribute_record` now records only non-declaration attributes,
`crates/rdocx-oxml/src/text.rs:65`, and the `used_prefixes` loop at
`crates/rdocx-oxml/src/text.rs:91` re-declares exactly the prefixes those
attributes use. That matches the approach section of
`.claude/plans/F-X131-design.md` including the three stated consequences. The
now unreachable guard and the third tuple element were both removed as the plan
required.

The second change at `crates/rdocx-oxml/src/text.rs:153` was not in the
approved plan. It is in scope for the same defect class, it is the reason
`repeated_saves_are_byte_identical_after_allocation` and
`typed_comment_flush_preserves_canonical_story_history` were failing, and it is
covered by a named regression. Recorded rather than deferred, because splitting
it into a separate F-ID would leave the branch red between them.

## Correctness

The record still declares every prefix its own retained attributes use, so the
`NsReader` resolution at `crates/rdocx-oxml/src/text.rs:161` that gives an
authored paragraph identity precedence over a retained one of the same expanded
name keeps its bindings. Only declarations that nothing in the record resolves
against are dropped. Verified by
`paragraph_root_attributes_reject_alias_duplicates_and_authored_id_wins`
passing unedited.

Skipping `xmlns:w14` on the target cannot leave an unbound prefix. The authored
path at `crates/rdocx-oxml/src/text.rs:4808` already writes `w14:paraId` bare,
so it makes the same assumption. For a reopened document the source must have
bound `w14` to carry the attribute at all, and `known_ns` at
`crates/rdocx-oxml/src/document.rs:1855` excludes only `w`, `r`, `mc` and the
default declaration from `extra_namespaces`, so a source `xmlns:w14` is
captured and replayed at the document root at
`crates/rdocx-oxml/src/document.rs:1993`. `w:comments` declares it directly at
`crates/rdocx-oxml/src/comments.rs:129`.

The early `continue` at `crates/rdocx-oxml/src/text.rs:69` runs before the
duplicate-expanded-name check, which is correct. A declaration has no expanded
attribute name to collide on, and the previous code also skipped the check for
declarations.

`attributes.is_empty()` at `crates/rdocx-oxml/src/text.rs:85` now returns
`Ok(None)` for a root carrying only declarations. That is the intended
behaviour and is pinned by
`a_root_with_only_namespace_declarations_records_nothing`.

## OOXML

No schema child order changes. No prefix is written other than the fixed `w`
and the retained producer aliases. Reads stay prefix tolerant. Unmodelled
subtrees are untouched by this diff.

## Tests

Five tests were added or confirmed. The gate,
`a_section_root_retains_no_namespace_declaration_its_attributes_do_not_use`
at `crates/rdocx-oxml/src/revision.rs:1454`, was verified to fail against the
unfixed capture side and pass against the fixed one, so it is a real gate and
not a test that passes against unmodified code.
`a_reopened_paragraph_identity_does_not_rebind_the_prefix_its_part_root_owns`
covers the write side. Both are named as the failure they prevent, per the
regression rule in `docs/hld/12-testing-strategy.md`.

`every_property_revision_keeps_owner_local_aliases` and
`repeated_saves_are_byte_identical_after_allocation` pass again without being
edited, which is the point. No existing assertion was weakened, and no recorded
baseline was re-recorded.

`WORD_COMMENT_CANDIDATE_SHA256` at `crates/rdocx/src/comments.rs:1683` is
unchanged. An earlier attempt fixed the write side by making the authored path
declare `xmlns:w14`, which changed the candidate bytes and would have
invalidated the Word 16.104 no-repair evidence that SHA binds. That approach
was abandoned for the one in this diff, which leaves authored bytes untouched.

## Structure

No new trait, generic, module, file or feature flag. The two map entries added
to `scripts/test_sprint_workflow.py` register the split property modules with
their capability families, which the guard at
`scripts/test_sprint_workflow.py:9604` requires of any modeled module.

## Nitpicks

### N1, raw byte comparison of a namespace URI
`crates/rdocx-oxml/src/text.rs:153`

`attribute.value.as_ref()` is the raw attribute value, so the comparison
against `W14_NS` would miss a value written with a character entity. No
producer escapes a namespace URI and the surrounding code compares the same
way, so this is taste rather than a defect.

### N2, the skip is keyed on the prefix spelling
`crates/rdocx-oxml/src/text.rs:153`

An alias other than `w14` bound to the same namespace still gets a local
declaration, so `i:paraId` and `w14:paraId` are treated differently. That is
deliberate, since only the literal `w14` prefix is the one the part roots emit,
and `a_retained_root_attribute_keeps_the_declaration_its_own_prefix_needs`
pins the alias behaviour.

## Exit

Zero defects and zero smells. The story may proceed to completion.
