# F-X091, Serialize unused root default namespaces safely

**Status**: completed
**Sprint**: S72
**Size**: M
**Depends on**: F-255

## Problem

`unsafe_serializer_namespace_prefix` rejects every unknown root default
namespace without checking effective use. GitHub Issue 73 shows the unused
Word task namespace blocking typed saves. The CLI then calls infallible
`Document::replace_text`, whose preflight failure panics instead of reaching
the existing CLI error boundary.

## Spec reference

- `docs/hld/04-opc-and-packaging.md`, namespace scope and atomic serialization.
- `docs/hld/10-bindings-spec.md`, native and CLI mutation boundaries.
- `docs/hld/12-testing-strategy.md`, preservation and CLI integration gates.
- `docs/hld/13-risks-and-open-questions.md`, namespace preservation risk.
- `docs/hld/14-development-backlog.md`, F-X091.

## Approach

Walk the retained source with namespace scopes. Treat the root default as
unsafe only when an unprefixed element inherits that exact binding. Nested
default redeclarations shadow it, and unprefixed attributes do not use it.
Malformed or ambiguous scope remains fail-closed. Add additive
`Document::try_replace_text(&mut self, &str, &str) -> Result<usize>` over the
existing staged batch path. Keep the legacy method and route only the CLI
through the fallible twin.

## Rejected alternatives

- Whitelist one Microsoft URI. Safety depends on effective use.
- Ignore all unknown defaults. Live unprefixed elements could change meaning.
- Catch the CLI panic. Panic recovery does not establish atomic error handling.
- Break the legacy replacement signature. An additive twin is sufficient.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| regression | `unused_root_default_namespace_allows_atomic_save` | The reported shape mutates, saves, reopens, and preserves raw expanded names. |
| regression | `used_root_default_namespace_still_fails_atomically` | A live inherited default returns the named error without mutation. |
| unit | `default_namespace_use_respects_element_scope` | Nested shadows and unprefixed attributes classify correctly, malformed scope fails closed. |
| integration | `cli_replace_reports_namespace_preflight_errors_without_panicking` | Exit one, stable error, no panic, and no partial output. |
| unit | `try_replace_text_publishes_only_a_preflighted_candidate` | Success commits once and failure preserves identical bytes. |

The test gate is `unused_root_default_namespace_allows_atomic_save`.

## HLD impact

- `docs/hld/04-opc-and-packaging.md`
- `docs/hld/10-bindings-spec.md`
- `docs/hld/12-testing-strategy.md`
- `docs/hld/13-risks-and-open-questions.md`
- `docs/hld/14-development-backlog.md`

## Risk routing

- Any parser or serialiser. Prove namespace-aware reads, fixed output prefixes,
  schema order, and byte-exact retained raw XML.
- Public API of a published crate. Record additive pre-1.0 impact, run the
  patched publish dry-run, and enforce the 10 MiB archive gate.

## Hash harness

Expected unchanged across all 49 entries. Any delta blocks completion.

## Implementation checklist

- [x] Classify root default use with a scope-aware walk.
- [x] Keep used, ambiguous, and malformed cases fail-closed.
- [x] Add the fallible single-placeholder replacement twin.
- [x] Route CLI replacement through its existing Result boundary.
- [x] Add namespace, atomicity, and compiled CLI regressions.
- [x] Run the full gate, package gate, and unchanged hash harness.
- [x] Update exactly the listed HLD files.

## Open questions

None.
