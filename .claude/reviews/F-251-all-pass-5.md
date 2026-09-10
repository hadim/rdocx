# F-251, all aspects, pass 5

**Reviewed**: complete working-tree source diff from `25226ae5be006391463e509525e629e20f060d5b`, 7 files, 2,022 insertions, 127 deletions, 2,149 changed lines, plus passes 1 through 4
**Verdict**: 3 defects, 1 smell, 0 nitpicks

## Prior-pass closure

All 15 prior defects are closed for their stated triggers.

- Pass 1 D1 remains closed. Page-number mutation tokenizes the quote-aware
  start tag and edits only the parsed Word attribute span.
- Pass 1 D2 remains closed. Duplicate `pgNumType` elements preserve source
  order and first-projection semantics.
- Pass 1 D3 remains closed. Retained children replay at schema and repeated
  reference boundaries.
- Pass 1 D4 remains closed. Checked geometry converts full-width EMUs before
  checked narrowing.
- Pass 1 D5 remains closed. Legacy document setters preserve their unchecked,
  infallible behavior.
- Pass 1 D6 remains closed. The normal Word gate checks complete geometry,
  multi-digit displayed values, and physical page identities.
- Pass 2 D1 remains closed. Fresh and restarted endnote pages continue the
  final body sequence and substitute PAGE from the displayed value.
- Pass 2 D2 remains closed. Signed margins are range checked before the four
  fields are published.
- Pass 2 D3 remains closed. Missing starts use a Word-bound prefix or a safe
  generated binding.
- Pass 2 D4 remains closed. Distinct reference removal and reorder retain the
  intended predecessor or successor boundary.
- Pass 2 D5 remains closed. Regeneration checks the total PDF page count and
  complete geometry and PAGE vectors.
- Pass 2 D6 remains closed. The exact oracle artifact directory is guarded on
  success and unwind paths.
- Pass 3 D1 remains closed for ordinary equal-reference removal, reorder, and
  reopen behavior.
- Pass 3 D2 remains closed. Generated prefix reservation includes inherited and
  local scopes, shadows, and QName-valued producer data.
- Pass 4 D1 is closed. Opaque integer tokens replace allocation addresses,
  clones keep their local token map, retired tokens are not reused, and the
  focused value-preserving String replacement regression passes.

## Defects

### D1, the reference wrapper breaks the published Vec field contract
`crates/rdocx-oxml/src/document.rs:428`

`CT_SectPr.header_refs` and `footer_refs` were public `Vec<HdrFtrRef>` fields and
are now a different public type. Existing assignments such as
`section.header_refs = vec![reference]`, explicit `Vec` annotations, reverse
comparisons, and functions accepting `&Vec<HdrFtrRef>` no longer compile.
`From<Vec<_>>` cannot make assignment coercive. The wrapper also omits ordinary
previously available mutations including `extend`, `truncate`, `append`,
`drain`, `splice`, `split_off`, `sort_by`, `reverse`, rotation, deduplication,
and mutable-slice access. This is a broad non-additive source break in a
published model for a story whose routed API impact is additive pre-1.0. The
identity implementation must not silently remove the existing Vec surface.

### D2, mutable iteration can reorder values without their source tokens
`crates/rdocx-oxml/src/document.rs:160`

`iter_mut` and mutable `IntoIterator` return ordinary slice iterators over only
the reference values. A caller can take two yielded references and use
`std::mem::swap`, or call `iter_mut().as_mut_slice().sort_by_key(...)`. The
values then move while `source_identities` remains in its old order. This
bypasses the wrapper's lockstep `swap` and `sort_by_key` methods and attaches
each retained boundary to the wrong source reference. Index mutation is
deliberately token-preserving for field or whole-value replacement, but an
iterator-driven reorder is observably the same operation as the wrapper's
token-moving reorder and cannot have different identity semantics. Every
exposed mutation path must either move tokens with values or deliberately
retire affected anchors.

### D3, identity exhaustion can leave the parallel vectors inconsistent
`crates/rdocx-oxml/src/document.rs:170`

`push` publishes the reference before requesting the next token, and `insert`
does the same at `crates/rdocx-oxml/src/document.rs:181`. The token allocator
panics on exhaustion at `crates/rdocx-oxml/src/document.rs:113`. If that panic
is caught, the value vector has grown while the token vector has not. Later
removal, indexing, or anchoring can then panic or associate the wrong token.
The practically remote counter limit does not justify an explicitly handled
failure path corrupting the collection. Acquire a unique token before either
parallel vector is mutated, and define exhaustion without a new production
panic.

## Smells

### S1, hidden process-global tokens leak into Debug-based cache accounting
`crates/rdocx-oxml/src/document.rs:117`

Derived `Debug` prints `source_identities`, so independently parsed but equal
section models have different debug text based on prior process activity. The
layout engine includes that text length in retained-byte accounting at
`crates/rdocx-layout/src/engine.rs:4555`, even though fixed token storage is
already counted by `retained_capacity_bytes`. Crossing a decimal digit boundary
therefore changes cache accounting for the same semantic input and can alter
eviction behavior. Equality correctly ignores tokens and no serde surface is
present, but Debug should follow the same hidden-state policy or the cache
accounting must stop consuming it.

## Nitpicks

None.

## Focused checks

The following exact filtered test passed:

- `equal_reference_boundaries_survive_string_replacement_reorder_and_reopen`

No broad suite or external Word oracle was run in pass 5.

## Not found

- **Correctness**: no additional namespace, fresh or restarted endnote, PAGE
  substitution, header parity, physical page identity, PageFrame clone, or
  incremental-layout output defect was found.
- **Contract**: margin boundaries and atomicity, checked section setters,
  legacy infallible setters, token-aware clone and equality behavior, and
  direct indexed replacement produced no additional finding beyond D1 through
  D3.
- **Panics**: no additional production panic was found beyond token exhaustion
  and its lockstep consequence in D3.
- **OOXML**: schema sequence, unsupported-child replay, duplicate page-number
  ordering, exact raw retention, alias prefixes, ancestor and local shadowing,
  QName-valued retained data, generated declarations, equal-reference reopen,
  removal, and explicit swap produced no additional finding.
- **Oracle**: exact tool pins, total-page detection, complete geometry and PAGE
  vectors, physical records, and guarded cleanup produced no additional
  finding.
- **Tests**: the pass-4 allocation replacement, retired-token, clone, and reopen
  regression is sensitive to its named triggers. No additional test issue was
  found beyond missing public compile coverage, iterator reorder coverage, and
  exhaustion-path coverage for D1 through D3.
- **Structure**: the collection is not a forwarding-only wrapper because both
  header and footer paths use its real source-identity state. No unjustified
  trait, generic, dynamic dispatch, feature flag, crate, module, or file was
  added.
