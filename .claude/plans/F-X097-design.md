# F-X097, Preserve namespace-scoped drawings and complex fields in comparison

**Status**: completed
**Sprint**: S73
**Size**: M
**Depends on**: F-X093, F-234

## Problem

`crates/rdocx-oxml/src/drawing.rs:1498` materializes inherited bindings for
parsing an inline or anchored drawing, but stores the original inner wrapper as
the serialization source. Prefixes declared on the story root or outer
`w:drawing` are therefore absent when comparison serializes a fresh wrapper.
`crates/rdocx/src/comparison.rs:2177` separately assumes one physical direct
run span per modeled run, which is false for complex fields whose projection
collapses several physical runs.

## Spec reference

- `docs/hld/04-opc-and-packaging.md`, "Document comparison uses the same package boundary".
- `docs/hld/10-bindings-spec.md`, "Native callers generate tracked changes".
- `docs/hld/12-testing-strategy.md`, "Drawing preservation coverage".

## Approach

Before comparison flushes its staging copies, close only the inherited bindings
used by detached inline or anchor wrappers. Keep the ordinary parser's retained
bytes unchanged so open and save, mail merge, and hash-bound chart artifacts do
not gain redundant declarations. Replace the comparison-only physical-run
scanner with an ownership projection that groups the begin, instruction,
separator, result, and end runs of a complex field into the modeled owner
emitted by the paragraph parser. Reuse that projection in whole-run and granular
replacement paths.

## Rejected alternatives

- Adding fixed declarations to every `w:drawing` would invent bindings and
  still miss producer aliases.
- Disabling exact source reuse for field paragraphs would lose preserved XML
  outside the edited text.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| regression | `comparison_preserves_inherited_drawing_namespaces_and_complex_fields` | Ancestor-scoped body and header drawings plus a changed complex-field paragraph compare, save, reopen, accept, and reject correctly. |
| focused | Existing F-X093 comparison drawing tests | Existing local declarations and malformed drawing rejection remain covered. |

The test gate is the backlog regression named above.

## HLD impact

- `docs/hld/04-opc-and-packaging.md`
- `docs/hld/10-bindings-spec.md`
- `docs/hld/12-testing-strategy.md`
- `docs/hld/14-development-backlog.md`

## Risk routing

- Any parser or serialiser. Re-read `docs/hld/04-opc-and-packaging.md` and
  `docs/hld/06-presentationml-model.md`. Verify schema order, prefix-tolerant
  reads, fixed-prefix writes, and a namespace-complete raw subtree round trip.

## Hash harness

Expected to be unchanged. The samples do not invoke comparison.

## Implementation checklist

- [x] Add failing ancestor-namespace and complex-field comparison regressions.
- [x] Retain namespace-complete drawing wrapper bytes in comparison staging.
- [x] Project physical complex-field runs onto modeled comparison owners.
- [x] Run focused comparison tests, hash harness, full verification, and microscope.

## Open questions

None. The user approved the proposed remediation boundary.
