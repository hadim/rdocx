# F-X198, Preserve alternate text-box copies during scoped replacement

**Status**: approved
**Sprint**: S91
**Size**: M
**Depends on**: F-X187

## Problem

Scoped replacement selects only the modeled physical story at
`crates/rdocx/src/document.rs:26846`. It splices only that owner's paragraph
at `document.rs:27012`. Story discovery deliberately selects the first
Choice containing a text box at `document.rs:8221`, leaving Fallback opaque.
The result can edit Choice while the VML Fallback retains old text.

## Spec reference

- `docs/hld/03-architecture.md`, physical story ownership and one document model.
- `docs/hld/04-opc-and-packaging.md`, The package, opaque XML and atomic mutation.
- `docs/hld/12-testing-strategy.md`, Checked Word scoped literal replacement.
- `docs/hld/14-development-backlog.md`, F-X198, full Issue301 and F-X187.

## Approach

Use the issue's explicitly permitted atomic-refusal outcome for paired
AlternateContent text boxes. Leave document-wide replacement unchanged.
Before a positive scoped edit can publish, inspect the selected text box's
physical enclosing AlternateContent in the actual retained source. Use
existing namespace-qualified story scanner conventions and exact span
containment, not matching text or a global substring search.

Add a small private detector in existing document.rs. It returns whether the
selected StoryKind::TextBox owner lies in an MC AlternateContent Choice with
another compatible text-box representation in a Choice or Fallback. Refuse
with Error::Other explaining that scoped edits of alternate text-box copies
require document-wide replacement. This maps through the existing Python
adapter to RdocxError. Standalone/single-representation text boxes keep their
supported scoped behavior. Opaque foreign lookalikes do not masquerade as MC.
Ambiguous alternate ownership refuses rather than selecting an identical box.

Perform the guard before splicing/publishing the candidate. Existing invalid
text, checked source selection, expected-count mismatch and zero no-op retain
atomic semantics. Candidate preparation never changes the live document.
Preserve source bytes, comments, formatting, revisions and unrelated members.
No public method, type, module, file or dependency is added.

## Rejected alternatives

- Edit the first matching Fallback text, which can redirect to an identical box.
- Reimplement global replacement with a scoped substring filter, losing
  physical ownership and existing counted transactions.
- Claim Choice-only edits are valid, leaving producer alternates inconsistent.

## Test plan

**Test gate**: regression.

| Category | Test | Asserts |
|---|---|---|
| regression | scoped_text_box_replacement_refuses_alternate_copies_atomically | Issue301 source-built paired box errors with byte-equal full package and live model |
| regression | scoped_single_representation_text_boxes_keep_their_supported_edit | Nonpaired box still edits selected paragraph and counts once |
| regression | alternate_text_box_refusal_tracks_physical_owner_and_namespace | Multiple identical boxes, aliases, shadows, unequal fallback and opaque siblings cannot redirect ownership |
| integration | scoped_text_box_count_mismatch_and_zero_noop_preserve_every_copy | Count mismatch, stale locations, invalid text and zero matches remain atomic |
| round-trip | document_wide_text_box_replacement_updates_both_copies_once | Existing global replacement produces both PIN copies with count1 after save/reopen |
| regression | test_scoped_alternate_text_box_refusal_is_rdocxerror_without_mutation | Installed Python StoryItem refusal, unchanged revision, handles and bytes |

Use existing native regression_test.rs and existing Python replacement test
file. Build controls from current public add_text_box_to_story and generated
XML for alternate namespace/topology discriminators. Read contributor updates
before implementing and attach a later Issue301 PR to this F-ID.

## HLD impact

- `docs/hld/10-bindings-spec.md`
- `docs/hld/12-testing-strategy.md`
- `docs/hld/14-development-backlog.md`

## Risk routing

- Parser/serializer: HLD04/HLD06, qualified MC/Word/VML ownership, exact opaque
  byte retention, positive namespace aliases and negative shadows.
- PyO3: actual installed Python tests and both WASM target checks. Workspace
  tests exclude both binding crates.
- Public behavior: a deliberate safety refusal of unsupported paired edits,
  not a new public API. Package dry-run and 10 MiB gate for affected carriers.
- No new file, crate, module, trait, generic, flag or dependency.

## Hash harness

Expected unchanged. No sample scoped alternate-text-box replacement is used.

## Exclusive file claims

- `crates/rdocx/src/document.rs`
- `crates/rdocx/tests/regression_test.rs`
- Existing Python replacement test entrypoint
- Listed HLD files and all Cargo/binding/test execution

## Implementation checklist

- [ ] Prove Choice-only failure at exact Base.
- [ ] Add qualified physical-owner alternate detector and atomic refusal.
- [ ] Cover standalone, multiple boxes, namespaces and producer topology.
- [ ] Verify global replacement, count mismatches and save/reopen retention.
- [ ] Run native/installed Python gates, scoped verify and clean microscope.
- [ ] Prepare validated handoff.

## Open questions

None. Issue301 expressly permits atomic refusal. Use that outcome to retain
opaque XML without guessing equivalent alternate structure.
