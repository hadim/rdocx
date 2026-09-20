# F-X132, all, pass 1

**Reviewed**: the uncommitted working tree at the first complete implementation,
`crates/rdocx/src/document.rs` and `crates/rdocx/tests/regression_test.rs`,
2 files, 29 added and 4 removed in the library and 105 added in tests. Contract
read first, `.claude/plans/F-X132-design.md`, then
`docs/hld/04-opc-and-packaging.md` "Package integrity" and
`docs/hld/12-testing-strategy.md` "Test taxonomy".
**Verdict**: 0 defects, 2 smells, 4 nitpicks

## Defects

None. Reported explicitly rather than padded.

The filter compares a declaration by name and by value at
`crates/rdocx/src/document.rs:1584`, so a redeclaration to a different URI
survives and only a true no-op is dropped. No matcher arm, flag or comparison
is touched, which is what the plan promised and what
`indistinguishable_owners_of_a_rebinding_declaration_still_fail_closed` proves.

## Smells

### S1, an intended behaviour change is left with no test
`crates/rdocx/src/document.rs:1581`

Dropping a redundant declaration from `ModeledOwnerSpan::declarations` means a
modified save no longer replays such a declaration onto an owner that carries a
marker for it. The plan states the narrowing in prose, and nothing in the diff
pins it. A later reader cannot tell the narrowing from an accident, and a
future change could restore the old behaviour without any test objecting.

### S2, the whole-document span scan gained a second scope copy per element
`crates/rdocx/src/document.rs:1843`, `crates/rdocx/src/document.rs:1869`

`let inherited = scope_stack.last().cloned().unwrap_or_default();` followed by
`let mut bindings = inherited.clone();` allocates twice for every element in
the part, where the code previously allocated once. `modeled_owner_spans` runs
over the whole `word/document.xml`, once per owner derivation and once per
replay, so the cost lands on every save of every document. The inherited scope
only needs to be borrowed.

## Nitpicks

- `crates/rdocx/tests/regression_test.rs:31077`, `document_from_body` re-seeds a
  package by hand although `document_with_content_controls` at
  `crates/rdocx/tests/regression_test.rs:11767` and `wrap_word_body` at
  `crates/rdocx/tests/regression_test.rs:13809` already exist at file scope and
  do exactly this.
- `crates/rdocx/tests/regression_test.rs:31075`, the module declares its own
  `W_NS` constant although the file already imports
  `rdocx_oxml::namespace::W_NS` at `crates/rdocx/tests/regression_test.rs:29`.
- `crates/rdocx/tests/regression_test.rs:31136` and
  `crates/rdocx/tests/regression_test.rs:31159`, `format!` wraps a literal with
  no interpolation.
- `crates/rdocx/tests/regression_test.rs:31156`, the comment says the two runs
  "differ only in their text", and they also bind different prefixes.

## Not found

- **correctness**. Both comparison directions were checked. An owner
  declaration that is redundant in the serialized part but not in the producer
  part still reaches the replay write step, where `existing` already holds it
  and the `Some((_, existing_value)) if existing_value == value => {}` arm at
  `crates/rdocx/src/document.rs:2637` makes it a no-op. The reverse case skips
  the owner, which is the intended outcome.
- **panics**. No `unwrap`, `expect`, index or slice is added to library code.
  The added `retain` closure cannot panic. Test `unwrap` matches the file.
- **ooxml**. No child order changes, no prefix change on write, no unmodelled
  subtree dropped. `unsafe_nested_namespace_prefix` at
  `crates/rdocx/src/document.rs:2506` still sees every shadowing declaration,
  because a shadow that matches the enclosing scope is already caught at the
  root or the body by `unsafe_serializer_namespace_prefix`, and a shadow that
  does not match the enclosing scope is not redundant and survives the filter.
  `intermediate_raw_shadow_is_safe_but_direct_fixed_prefix_use_fails_closed`
  still passes unedited.
- **structure**. One new free function with two call sites. No trait, no
  generic parameter, no `Box<dyn>`, no forwarding wrapper, no feature flag, no
  new crate, module or file.
- **contract**. The diff matches `## Approach` in the plan and adds nothing the
  plan did not name.
