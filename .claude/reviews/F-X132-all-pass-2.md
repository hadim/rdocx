# F-X132, all, pass 2

**Reviewed**: the uncommitted working tree after the pass 1 remediation,
`crates/rdocx/src/document.rs` and `crates/rdocx/tests/regression_test.rs`,
2 files, 29 added and 4 removed in the library and 114 added in tests. Contract
read first, `.claude/plans/F-X132-design.md`, then
`docs/hld/04-opc-and-packaging.md` "Package integrity" and
`docs/hld/12-testing-strategy.md` "Test taxonomy".
**Verdict**: 0 defects, 0 smells, 2 nitpicks

## Defects

None.

## Smells

None.

Both pass 1 smells are cleared.

S1 is answered by
`a_table_redeclaring_the_binding_it_inherits_keeps_none_of_its_own` at
`crates/rdocx/tests/regression_test.rs:31164`. It uses a `w:tbl` that redeclares
the binding the document root already owns and holds an unmodelled `w:` child,
which is what gives the declaration a marker and made it a replayed owner
before. Reverting only the filter in `rebinding_namespace_declarations` turns
the saved part's single `xmlns:w` into two and the test red, so the narrowing is
pinned rather than described.

S2 is answered at `crates/rdocx/src/document.rs:1843` and
`crates/rdocx/src/document.rs:1869`, now
`scope_stack.last().map(Vec::as_slice).unwrap_or_default()` followed by
`inherited.to_vec()`. The scan allocates once per element again, as it did
before the diff, and `rebinding_namespace_declarations` takes the borrow.

## Nitpicks

- `crates/rdocx/src/document.rs:1572`, the doc comment explains the trigger by
  naming the retained root-attribute record, which lives in `rdocx-oxml`. It is
  the clearest available explanation of why the function exists, and it will
  read as stale if that record is ever renamed.
- `crates/rdocx/tests/regression_test.rs:31154`, the third test locates each run
  with `rfind("<w:r ")` on the serialized text rather than parsing. It is exact
  for this fixture, where both runs carry attributes, and it would silently
  match the wrong element if a future edit made a run attribute free.

## Not found

- **correctness**. The filter drops a declaration only when the inherited scope
  binds the same name to the same value
  (`crates/rdocx/src/document.rs:1581`). `apply_namespace_declarations` at
  `crates/rdocx/src/document.rs:1610` keys a scope by declaration name and
  replaces rather than appends, so the inherited scope holds at most one entry
  per name and the comparison cannot be confused by a stale duplicate. An
  `xmlns=""` undeclaration over a bound default differs in value and survives.
- **contract**. The diff matches `## Approach` in the plan. Four tests, the gate
  plus three named in `## Test plan`, and nothing the plan did not name.
- **panics**. No `unwrap`, `expect`, index or slice added to library code.
- **ooxml**. No schema child order change, no prefix change on write, no
  unmodelled subtree dropped, and
  `a_table_redeclaring_the_binding_it_inherits_keeps_none_of_its_own` asserts
  the retained `<w:producerOnly/>` survives the save it now no longer replays a
  declaration onto.
- **tests**. The gate test fails with the exact production error when only the
  filter is reverted, and so does the narrowing test. The two guard tests pass
  either way by design, which is their purpose, since they exist to fail if the
  matcher is loosened rather than if the filter is removed.
- **structure**. One new free function with two call sites. No trait, no
  generic parameter, no `Box<dyn>`, no forwarding wrapper, no feature flag, no
  new crate, module or file. The test module reuses the file's existing
  `document_with_content_controls`, `wrap_word_body` and `W_NS` rather than
  restating them.
