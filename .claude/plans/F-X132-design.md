# F-X132, Match a retained namespace owner by structure, not by identity

**Status**: completed
**Sprint**: S74
**Size**: S
**Depends on**: F-X128, F-X131

## Problem

`python3 scripts/docx_ssim_harness.py --check` fails at its acceptor on
`corpus/docx/redlined_no_footer.docx`. The document opens, `accept_all` accepts
21 revisions, then `to_bytes` fails with

```
cannot identify retained `r` nested namespace owner after mutation
```

A source-built probe bisects the first failing commit.

| Commit | Result |
|---|---|
| `f80b8e14`, the sprint base | saves, 80237 bytes |
| `5bad2f26`, F-X129, the commit before F-X128 | saves, 80237 bytes |
| `976611ee`, F-X128 | regression starts |
| `f224c3b2`, the property grammar split | still failing |
| `f68ff760`, F-X131 | still failing |
| `5c6d1188`, the wave 4 head | still failing |

So it is a second F-X128 regression, alongside the five test failures F-X131
already fixed. It is not caused by the property split, by F-X131, or by any of
F-264, F-265, F-267, F-268a, F-269 or F-270.

## Spec reference

- `docs/hld/04-opc-and-packaging.md`, "Package integrity", the rule that
  canonical serialization writes fixed `w` attributes in schema order while
  unmodelled content retains its stored bytes.
- `docs/hld/12-testing-strategy.md`, "Test taxonomy", the `regression` row and
  the rule that a regression test is named as the failure it prevents.
- `docs/hld/14-development-backlog.md`, "F-X132, Match a retained namespace
  owner by structure, not by identity".

## The mechanism, as confirmed

The backlog entry and the integrator note both attribute the failure to
`logical_owner_snapshot` comparing `w:rsid*` values across an `accept_all` run
merge. Instrumenting the matcher shows a different cause, so the story is
solved at a different boundary and the backlog entry's body is corrected to
match. The story title stays as the sprint ledgers carry it, because the
integrator owns those rows.

The failing return is `crates/rdocx/src/document.rs:2584`, reached through
`Document::to_bytes -> prepare_staged_output -> prepare_staged_package ->
canonicalize_drawing_ids -> opaque_relationship_ids_from_serialized_story ->
replay_nested_namespace_declarations`, captured with a forced backtrace. It
refuses whenever an owner is both `ambiguous_without_namespace` and
`same_namespace_structural_alternate`, whatever its candidates look like.

The owner printed at the failure is

```
owner=r decls=[("xmlns:w", "...wordprocessingml/2006/main")] ncand=2
OWN.semantic=["start:{...}r:[{...}rsidRPr=00EA2362]", ..., "text:>", ...]
```

with two candidates, both semantically equal to the owner and byte identical to
each other.

Three measurements complete the picture.

1. `word/document.xml` inside `corpus/docx/redlined_no_footer.docx` contains
   exactly one `xmlns:w`, on the document root. No run and no paragraph in the
   producer's file declares one, so the producer authored no nested owner at
   all.
2. A probe that instruments `nested_modeled_namespace_owners` reports the part
   it reads three times during one `to_bytes`. The first two see the
   producer's 387397 bytes with one `xmlns:w`. The third, the failing one, sees
   a flushed 578815 bytes with 888 of them, because `prepare_staged_package`
   flushes the document part before `canonicalize_drawing_ids` reads it back.
3. Saving the same document with no revision work writes
   `<w:r w:rsidRPr="00EA2362" xmlns:w="...main">` and
   `<w:p w14:paraId="..." w:rsidR="..." xmlns:w="...main">`. F-X128 retains the
   producer root attributes, and F-X131's `used_prefixes` loop keeps the `w`
   declaration those attributes use, so every retained paragraph and run gains
   a `w` binding that the document root already owns.

Put together. A redundant `w` declaration lands on every retained paragraph and
run. The next pass over the flushed part reads those as nested namespace
owners. The corpus document contains two byte-identical `►` runs, so each is an
equally good owner of the other's declaration, both flags go true, and the
matcher refuses.

The revision identities are not what fails. They are equal across the two
candidates, and the printed `semantic` vectors match the owner exactly. The
`accept_all` run merge is only what makes the document modified, so the save
takes the canonical serialization path rather than the byte-for-byte one.

The confirming experiment is the fix itself. Reverting only the filter, with
the tests in place, reproduces the exact error from a source-built document
with no binary fixture.

## Approach

Treat a declaration that rebinds a prefix to the URI already in scope as owning
nothing.

`modeled_owner_spans` at `crates/rdocx/src/document.rs:1821` already computes
the enclosing scope before it applies an element's own declarations. Keep that
scope and record only the declarations that change it.

```rust
fn rebinding_namespace_declarations(
    element: &BytesStart<'_>,
    inherited: &[(String, String)],
) -> Result<Vec<(String, String)>>
```

The function filters `namespace_declarations` against the inherited scope by
declaration name and value. Both the `Event::Start` and `Event::Empty` arms of
`modeled_owner_spans` call it in place of `namespace_declarations`.

Everything downstream follows without further change. An owner whose
declarations all vanish is skipped by the existing
`if declarations.is_empty() { continue; }` in
`nested_modeled_namespace_owners`, so the corpus document produces no run or
paragraph owners and the replay is a no-op. A genuinely rebinding declaration
still produces an owner, still carries its markers, and still reaches the same
matcher unchanged.

Why this is the right boundary. A redeclaration to the URI already in scope
resolves no name differently from the scope it sits in. It carries no producer
intent that canonical serialization could lose, so there is nothing for the
retained-owner machinery to re-locate. F-X131 already named this the mistake,
"retaining and replaying a namespace binding that the surrounding scope already
owns", and applied it to the canonical `w14` binding on the write side. This is
the same rule on the read side, stated once for every prefix.

The matcher is not loosened. No arm, no flag and no comparison changes. An
owner that really does rebind a prefix, and really is indistinguishable from
another, still fails closed with the same message.

### What the fix does not do

No paragraph or run byte changes. The retained record still writes the
redundant `w` declaration, so every identity attribute and every declaration
F-X128 and F-X131 emit survive exactly as before, and the hash harness holds at
49 of 49.

One behaviour narrows, and it is observable only where the owner also holds an
unmodelled child that uses the prefix, because that child is what makes the
declaration carry a marker. A producer document that redundantly redeclares a
prefix on such a `w:tbl`, `w:tc`, `w:sdt` or `w:hyperlink` no longer has that
declaration replayed onto a modified save. The declaration resolved nothing, the
enclosing scope still binds the prefix, and the retained child is written
unchanged, so no name binding is lost.
`a_table_redeclaring_the_binding_it_inherits_keeps_none_of_its_own` pins it.

## Rejected alternatives

- Exclude `w:rsid*`, `w14:paraId` and `w14:textId` from the `semantic` and
  `structure` vectors, which is what the backlog entry proposed. The measured
  candidates agree on every one of those values, so the exclusion changes
  nothing about this failure, and it would remove the only signal that can tell
  two otherwise identical runs apart. It weakens the matcher and fixes nothing.
- Accept an ambiguous match when every candidate is byte identical. It is true
  that any assignment would then produce the same output, but it puts a second
  escape hatch in a matcher whose value is that it fails closed, to work around
  a declaration that should never have been an owner.
- Extend F-X131's write-side skip from `w14` to every canonical prefix, so the
  redundant `w` declaration is never written. It is the more thorough answer to
  the redundancy and it would also clear this failure, and it removes 190 KB of
  redundant declarations from this one corpus document. It is rejected here
  because it changes the saved bytes of every document that carries a producer
  root attribute, which is F-X128 and F-X131 territory rather than this story's,
  and because it leaves the matcher still treating a no-op declaration as an
  owner if anything else ever emits one. Recorded for the F-X131 line.
- Stop `prepare_staged_package` flushing the document part before
  `canonicalize_drawing_ids` reads it back, so the owners always come from the
  producer's bytes. That is a much larger change to the staging order, with no
  evidence that reading the flushed part is wrong in itself.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| regression | `accepting_revisions_still_saves_when_runs_carry_revision_identities` | **The test gate.** A source-built redlined document whose runs carry `w:rsid` identities, with two runs identical in every retained fact, accepts all revisions, saves, and reopens. Every `w:rsidR`, `w:rsidRDefault` and `w:rsidRPr` value survives into the saved part, the accepted `w:ins` wrapper is gone, and a second save is byte identical. Fails with the exact production error when the filter is reverted. |
| regression | `indistinguishable_owners_of_a_rebinding_declaration_still_fail_closed` | Two runs carrying a declaration that genuinely binds a prefix the enclosing scope does not, identical in every other fact, still refuse to save with `cannot identify retained `r` nested namespace owner after mutation`. The fix cannot be a blanket loosening. |
| regression | `a_semantic_difference_still_routes_each_rebinding_declaration_home` | Two runs with different genuine declarations and different text each keep their own declaration after a mutation, so semantic discrimination still works. |
| regression | `a_table_redeclaring_the_binding_it_inherits_keeps_none_of_its_own` | The narrowing, pinned. A `w:tbl` that redeclares the binding its root owns, and holds an unmodelled `w:` child so the declaration carries a marker, writes the document's single `w` declaration and the retained child unchanged. Fails when the filter is reverted. |
| regression | `every_modeled_container_replays_nested_namespaces_after_modification` | Existing, unedited. Genuine nested declarations on all six modeled owners still replay. |
| regression | `word_namespace_alias_used_by_raw_marker_replays_after_save_and_reopen` | Existing, unedited. An alias prefix bound to the Word namespace is not redundant and still replays. |

The **test gate** is
`accepting_revisions_still_saves_when_runs_carry_revision_identities`.

## HLD impact

- `docs/hld/14-development-backlog.md`

## Risk routing

- **Any parser or serialiser.** `crates/rdocx/src/document.rs` changes which
  elements the retained-namespace machinery treats as owners. Extra checks: a
  round-trip test proving a genuine nested declaration and its unmodelled
  subtree still survive byte for byte, reads stay prefix tolerant, writes keep
  the fixed `w` prefix, and a modified save still refuses an ambiguity it
  cannot resolve.

Not matched: unit conversion, theme colour, layout and pagination, crate
dependency graph, bundled fonts, public API of a published crate, WASM or PyO3
bindings, a new feature flag, a new trait or module, an external oracle,
release scripting, a file move.

## Hash harness

**Expected unchanged, 49 of 49.** The change drops only declarations that
rebind a prefix to the URI already in scope, and the replay it suppresses would
have written a declaration the serialized element already carries, so no
recorded part changes. The seven samples come from
`crates/rdocx/examples/generate_all_samples.rs`, which authors no nested
namespace declaration at all. Confirmed after the change by
`python3 scripts/hash_harness.py --check`.

## Implementation checklist

- [x] Reproduce the acceptor failure with a source-built probe against the
      corpus document.
- [x] Instrument the matcher, identify the failing arm and its backtrace, and
      print the owner and its candidates.
- [x] Measure the producer's declarations, the part the owners are read from,
      and the declarations a plain save writes.
- [x] Add `rebinding_namespace_declarations` and use it from both arms of
      `modeled_owner_spans`.
- [x] Add the gate regression, the two guard tests and the narrowing test in
      one named module at the end of `crates/rdocx/tests/regression_test.rs`.
- [x] Confirm the gate test fails with the production error and the narrowing
      test fails when only the filter is reverted, and that both guard tests
      pass either way.
- [x] Correct the backlog entry so it records the confirmed mechanism.
- [x] Run the full gate, including `scripts/docx_ssim_harness.py --check`.

## Open questions

None. The mechanism was established by instrumentation and measurement rather
than by reading, because the reading in the backlog entry pointed at a
comparison that the measurements rule out.
