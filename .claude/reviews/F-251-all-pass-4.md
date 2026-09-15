# F-251, all aspects, pass 4

**Reviewed**: complete working-tree source diff from `25226ae5be006391463e509525e629e20f060d5b`, 7 files, 1,763 insertions, 103 deletions, 1,866 changed lines, plus pass 1, pass 2, and pass 3 reviews
**Verdict**: 1 defect, 0 smells, 0 nitpicks

## Prior-pass closure

All 14 prior defects are closed for their stated triggers.

- Pass 1 D1 remains closed. Page-number mutation uses a quote-aware start-tag
  scan and edits only the parsed Word attribute span.
- Pass 1 D2 remains closed. Duplicate `pgNumType` elements retain their source
  order and first-projection semantics.
- Pass 1 D3 remains closed. Retained children replay at modeled schema and
  repeated-reference boundaries.
- Pass 1 D4 remains closed. Checked geometry converts full-width EMUs before
  checked narrowing.
- Pass 1 D5 remains closed. Legacy document setters preserve their unchecked,
  infallible signatures and behavior.
- Pass 1 D6 remains closed. The normal gate compares complete Word-produced
  records, including multi-digit displayed numbers and physical page identity.
- Pass 2 D1 remains closed. Fresh and restarted endnote pages continue the
  final body page's displayed sequence, and PAGE substitution consumes it.
- Pass 2 D2 remains closed. All four signed margins are checked before any
  mutation, including the positive and negative twip-range boundaries.
- Pass 2 D3 remains closed. Missing page-number starts reuse a Word-bound alias
  or declare a collision-free generated alias.
- Pass 2 D4 remains closed. Distinct predecessor and successor identities keep
  retained children stable under front removal, clearing, and reorder.
- Pass 2 D5 remains closed. Oracle regeneration checks total pages and requires
  complete geometry and displayed-number vectors.
- Pass 2 D6 remains closed. The scoped artifact guard removes its exact unique
  directory on normal return and during unwinding.
- Pass 3 D1 is closed for unchanged equal references, String and vector moves,
  vector reallocation, `CT_SectPr` cloning, removal, and reorder. The allocation
  replacement case is the new D1 below.
- Pass 3 D2 is closed. Reserved generated-prefix names include the complete
  inherited and local scope, including shadowed prefixes and producer bindings
  used only by QName-valued retained data.

## Defects

### D1, allocation addresses are not stable repeated-reference identities
`crates/rdocx-oxml/src/document.rs:126`

The source identity is the allocation address of the public `rel_id` String,
and resolution treats that address as authoritative at
`crates/rdocx-oxml/src/document.rs:134`. This survives moving the String and
reallocating the containing vector, but it does not survive a value-preserving
String replacement. For two equal references with a retained child between
them, a caller can clone both `rel_id` values into replacement allocations,
assign those identical values back, then swap the references. Both stored
addresses are stale, so resolution falls back to equal-value occurrence counts
in the current order. The retained child stays between the two references
instead of following the original source instances, contrary to the source
identity policy exercised by the equal-reference reorder test. A later save and
reopen then makes that wrong boundary permanent. Freed-address reuse can also
bind an anchor to a newly allocated String. Source identity needs a durable
token that is independent of allocator addresses and public String storage.

## Smells

None.

## Nitpicks

None.

## Focused checks

The following exact filtered tests passed:

- `retained_boundaries_distinguish_equal_story_reference_sources`
- `missing_page_number_start_uses_a_prefix_bound_to_word`

No broad suite or external Word oracle was run in pass 4.

## Not found

- **Correctness**: no additional fresh or restarted endnote numbering, PAGE
  substitution, header parity, physical page identity, PageFrame cloning, or
  incremental-cache defect was found.
- **Contract**: signed margin range handling and atomicity, legacy infallible
  setters, checked section setters, and additive PageFrame API compatibility
  produced no additional finding.
- **Panics**: no new production `unwrap`, `expect`, unchecked index, or
  arithmetic panic was found.
- **OOXML**: schema child order, quoted and entity-bearing attributes, duplicate
  page-number elements, unchanged raw replay, existing namespace aliases,
  ancestor and local prefix shadowing, generated declaration collisions,
  QName-valued retained attributes, distinct references, and ordinary equal
  reference moves produced no additional finding beyond D1.
- **Oracle**: pinned producer and tool identities, exact total-page checks,
  complete geometry and displayed-number vectors, physical page records, and
  cleanup-guard scope produced no additional finding.
- **Tests**: focused coverage now reaches equal header and footer references,
  front and tail removal, reorder, custom cloning, inherited producer prefixes,
  QName-valued data, oracle vector completeness, and unwind cleanup. No
  additional test issue was found beyond the missing D1 allocation-replacement
  trigger.
- **Structure**: no unjustified trait, generic, dynamic dispatch, forwarding
  wrapper, feature flag, crate, module, or source file was added.
