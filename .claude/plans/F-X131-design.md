# F-X131, Retain only the namespace declarations a root attribute uses

**Status**: completed
**Sprint**: S74
**Size**: S
**Depends on**: F-X128

## Problem

`crates/rdocx-oxml/src/revision.rs:1369`,
`every_property_revision_keeps_owner_local_aliases`, fails on `sprint/s74`.
`cargo test -p rdocx-oxml` reports 500 passed and 1 failed. The failure is
present at `5d8e68cc` in a clean worktree with its own target directory, so it
is not a stale build and not a consequence of the property grammar split at
`f224c3b2`, which reports the same 500 and 1 on both sides.

F-X128 added ordered root-attribute retention so a no-op save keeps
`w14:paraId`, `w14:textId` and the `w:rsid*` family.
`capture_root_attribute_record` at `crates/rdocx-oxml/src/text.rs:54` records
every attribute on the source element and marks each one with
`namespace_declaration(name)` at `crates/rdocx-oxml/src/text.rs:84`, so a
namespace declaration is retained whether or not a retained attribute uses its
prefix. `push_root_attribute_record` at `crates/rdocx-oxml/src/text.rs:126`
then copies every recorded attribute onto the written element.

The source in the failing test declares `xmlns:sa` on `w:sectPr` and its two
`sa:sectPrChange` children each carry the same binding, because the existing
alias machinery materializes a binding onto each element that uses one. The
written document therefore declares `xmlns:sa` three times, once on the
retained `w:sectPr` root and once on each child. The test expects two.

`w:pPr` and `w:tblPr` are unaffected because F-X128 gave root-attribute
retention to `CT_P`, `CT_R` and `CT_SectPr` only, so the same input emits
`xmlns:pa` and `xmlns:ta` exactly twice. The output is inconsistent between
element kinds for no stated reason.

Running the wider gate with `/private/tmp/rdocx-s73-bin` first on `PATH`
exposed a second instance of the same defect class on the write side.
`repeated_saves_are_byte_identical_after_allocation` and
`typed_comment_flush_preserves_canonical_story_history` both fail in
`crates/rdocx/tests/regression_test.rs`. The first save writes
`<w:p w14:paraId="00000001">` and relies on `w:comments` binding `w14` at the
part root, which it does at `crates/rdocx-oxml/src/comments.rs:129`. On reopen
the retained record re-declares `xmlns:w14` on the paragraph itself, so saving
the reopened document produces different bytes from the save it was read from.
Both defects are one story because they are one mistake, retaining and
replaying a namespace binding that the surrounding scope already owns.

Two further tests, `intermediate_raw_shadow_is_safe_but_direct_fixed_prefix_use_fails_closed`
and `unused_fixed_prefix_declarations_do_not_reject_safe_raw_replay`, fail at
the same baseline and are fixed by the capture-side change. Five pre-existing
failures in total, all from F-X128.

## Spec reference

- `docs/hld/04-opc-and-packaging.md`, "Package integrity", the paragraph
  requiring canonical serialization with fixed `w` attributes in schema order
  while unmodelled content retains its stored bytes.
- `docs/hld/12-testing-strategy.md`, "Test taxonomy", the `regression` row and
  the rule that a regression test is named as a sentence describing the failure
  it prevents.
- `docs/hld/14-development-backlog.md`, "F-X131, Retain only the namespace
  declarations a root attribute uses".

## Approach

Two changes, one per side of the same mistake.

### The capture side

Stop pushing namespace declarations into `attributes`. Record only attributes
that are not declarations, and let the existing `used_prefixes` loop at
`crates/rdocx-oxml/src/text.rs:95` add a declaration for every prefix a
retained attribute actually uses, which is what that loop already exists to do.

Three consequences follow, and each is deliberate.

A source root carrying only namespace declarations and no other attribute now
produces no record at all, because `attributes` is empty and the function
returns `Ok(None)` at `crates/rdocx-oxml/src/text.rs:87`. That is correct.
There is nothing to retain, and the bindings reach the children through the
alias machinery that already carries them.

The guard `if attributes.iter().any(|(name, _, _)| name == &declaration)` in
the `used_prefixes` loop becomes unreachable, because `attributes` can no
longer hold a name beginning `xmlns:`. It is removed rather than left as a
vacuous condition.

The third tuple element, `namespace_declaration(name)`, is now always false and
is removed, leaving a `(String, String)` pair.

Resolution is preserved. `push_root_attribute_record` reads the record with an
`NsReader` and calls `reader.resolver().resolve_attribute` at
`crates/rdocx-oxml/src/text.rs:157` to give an authored paragraph identity
precedence over a retained one of the same expanded name. That resolution needs
the record to declare the prefixes its own retained attributes use, and the
`used_prefixes` loop guarantees exactly those. Only declarations that no
retained attribute uses are dropped, so no resolution loses its binding.

### The write side

`push_root_attribute_record` stops copying `xmlns:w14` onto the target when it
binds the canonical namespace. The record keeps the declaration, because its own
`NsReader` resolution needs it, but the target does not receive it.

This makes the retained path agree with the authored path, which already writes
`w14:paraId` bare at `crates/rdocx-oxml/src/text.rs:4808`. It cannot leave an
unbound prefix. `w:comments` binds `w14` at its root, and for `w:document` the
`known_ns` list at `crates/rdocx-oxml/src/document.rs:1855` excludes only `w`,
`r`, `mc` and the default declaration from `extra_namespaces`, so a source
`xmlns:w14` is captured and replayed at the document root.

The direction matters. Making the authored path declare `xmlns:w14` instead
would also make the two sides agree, but it changes authored bytes and
therefore breaks `WORD_COMMENT_CANDIDATE_SHA256` at
`crates/rdocx/src/comments.rs:1683`, which binds the exact file a human opened
in Word 16.104 to confirm no repair. That evidence cannot be re-obtained here,
so authored bytes stay untouched.

## Rejected alternatives

- Change the test to expect three declarations. The duplicate is redundant
  output, and accepting it would also leave `w:sectPr` inconsistent with
  `w:pPr` and `w:tblPr` for no reason.
- Give `CT_PPr` and `CT_TblPr` root-attribute retention too, making all three
  emit three declarations. That spreads the redundancy rather than removing it,
  and neither story asked for it.
- Fix only the capture side. It leaves the write side rebinding `w14` on every
  reopened paragraph, so two byte-identity regressions stay red.
- Strip every declaration in `push_root_attribute_record`. Resolution inside the
  record would break, because the reader needs the declaration to resolve a
  prefixed retained attribute. Only the canonical `w14` binding is skipped, and
  only on the way out to the target.
- Make the authored path declare `xmlns:w14` so both sides match that way. It
  changes authored bytes and invalidates the Word no-repair evidence bound by
  `WORD_COMMENT_CANDIDATE_SHA256`.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| regression | `a_section_root_retains_no_namespace_declaration_its_attributes_do_not_use` | **The test gate.** A `w:sectPr` whose source declares a prefix used only by its children emits that declaration once per child and not on the root. Named as the failure it prevents. |
| regression | `every_property_revision_keeps_owner_local_aliases` | The existing S47 test passes again, unchanged. It is the reason this story exists and it is not edited. |
| unit | `a_retained_root_attribute_keeps_the_declaration_its_own_prefix_needs` | A root carrying `i:paraId` with `xmlns:i` still records both, so the binding a retained attribute depends on survives. |
| unit | `a_root_with_only_namespace_declarations_records_nothing` | `capture_root_attribute_record` returns `None` rather than a record of pure declarations. |
| regression | `a_reopened_paragraph_identity_does_not_rebind_the_prefix_its_part_root_owns` | A reopened paragraph keeps its bare `w14:paraId` and adds no local declaration, so the write side matches the authored side. |
| regression | `repeated_saves_are_byte_identical_after_allocation` | Save, reopen and save again produces identical bytes. Existing test, unedited, and the reason the write-side fix exists. |
| regression | `paragraph_root_attributes_reject_alias_duplicates_and_authored_id_wins` | F-X128's expanded-name precedence and duplicate-alias rejection still hold. Existing test, unedited. |

The **test gate** is
`a_section_root_retains_no_namespace_declaration_its_attributes_do_not_use`.

## HLD impact

- `docs/hld/04-opc-and-packaging.md`
- `docs/hld/14-development-backlog.md`

## Risk routing

- **Any parser or serialiser.** `crates/rdocx-oxml/src/text.rs` changes what a
  modeled root serializes. Extra checks: prove with a round-trip test that a
  retained attribute and the binding it needs both survive, that an unmodelled
  subtree is still preserved byte for byte, and that reads stay prefix tolerant
  while writes keep the fixed `w` prefix.

Not matched: unit conversion, theme colour, layout and pagination, crate
dependency graph, bundled fonts, public API of a published crate, WASM or PyO3
bindings, a new feature flag, a new trait or module, an external oracle, release
scripting, a file move.

## Hash harness

**Expected unchanged, 49 of 49.** The seven samples are generated by
`crates/rdocx/examples/generate_all_samples.rs`, which authors no producer root
attribute and no namespace alias, so no sample root carries a declaration to
drop. Confirmed after the change by `python3 scripts/hash_harness.py --check`.

## Implementation checklist

- [x] Record only non-declaration attributes in `capture_root_attribute_record`.
- [x] Remove the now unreachable guard in the `used_prefixes` loop.
- [x] Reduce the recorded tuple to a pair.
- [x] Stop `push_root_attribute_record` writing `xmlns:w14` onto the target
      when it binds the canonical namespace, so a reopened save matches the
      save it came from. Leave authored bytes untouched, because
      `WORD_COMMENT_CANDIDATE_SHA256` binds Word no-repair evidence.
- [x] Add the gate regression and the two unit tests.
- [x] Confirm `every_property_revision_keeps_owner_local_aliases` passes
      without being edited.
- [x] Run `cargo test -p rdocx-oxml`, `cargo test -p rdocx`, scoped clippy,
      `cargo fmt --all --check`, and the hash harness.

## Open questions

None. The defect and its fix were established by reading the code and
reproducing the failure in an isolated worktree.
