# F-251, all aspects, pass 6

**Reviewed**: complete working-tree source diff from `25226ae5be006391463e509525e629e20f060d5b`, 7 files, 1,772 insertions, 102 deletions, 1,874 changed lines, plus passes 1 through 5
**Verdict**: 0 defects, 0 smells, 0 nitpicks

## Prior-pass closure

All prior actionable findings are closed.

- Pass 1 D1 remains closed. Page-number edits use a quote-aware start-tag scan
  and the parsed Word attribute span.
- Pass 1 D2 remains closed. Duplicate `pgNumType` elements preserve source
  order and first-projection semantics.
- Pass 1 D3 remains closed. Unsupported children replay at schema and repeated
  reference boundaries.
- Pass 1 D4 remains closed. Checked geometry converts full-width EMUs before
  checked narrowing.
- Pass 1 D5 remains closed. Legacy document setters retain their unchecked,
  infallible signatures and behavior.
- Pass 1 D6 remains closed. The normal differential gate checks the complete
  Word record, multi-digit displayed values, and physical page identity.
- Pass 2 D1 remains closed. Fresh and restarted endnote pages continue the
  final body sequence and substitute PAGE from the displayed number.
- Pass 2 D2 remains closed. Every signed margin is range checked before any of
  the four fields is changed.
- Pass 2 D3 remains closed. Missing page-number starts use a Word-bound alias or
  a collision-free generated binding.
- Pass 2 D4 remains closed. Distinct reference removal and reorder retain the
  surviving predecessor or successor boundary.
- Pass 2 D5 remains closed. Oracle regeneration checks the total PDF page count
  and complete geometry and displayed-number vectors.
- Pass 2 D6 remains closed. The exact oracle artifact directory is guarded on
  success and unwind paths.
- Pass 3 D1 is resolved by the documented value-occurrence policy. Equal public
  values remain at a deterministic ordinal boundary without claiming hidden
  object provenance.
- Pass 3 D2 remains closed. Generated-prefix reservation includes the complete
  inherited and local scope, including shadowed and QName-only producer uses.
- Pass 4 D1 is closed. No allocator address participates in identity or output.
  Value-preserving allocation changes cannot affect serialization.
- Pass 5 D1 is closed. `header_refs` and `footer_refs` remain the published
  `Vec<HdrFtrRef>` fields with their full assignment and mutation surface.
- Pass 5 D2 and D3 are closed. There is no parallel token vector, mutable-path
  lockstep requirement, global token counter, or exhaustion path.
- Pass 5 S1 is closed. No hidden token enters Debug output or retained-cache
  accounting.

## Defects

None.

## Smells

None.

## Nitpicks

None.

## Focused checks

The following exact filtered tests passed:

- `retained_boundaries_use_occurrences_for_equal_story_references`
- `ambiguous_equal_reference_mutations_are_byte_stable`

`git diff --check` also passed. No broad suite or external Word oracle was run
in pass 6.

## Not found

- **Correctness**: no fresh or restarted endnote numbering, PAGE substitution,
  header parity, physical page identity, PageFrame clone, incremental-cache, or
  repeated-reference output defect was found.
- **Contract**: the published reference vectors retain their complete native
  surface. Section setters are additive, margin and other geometry validation
  is atomic, legacy document setters remain infallible, and the non-exhaustive
  PageFrame addition preserves construction compatibility.
- **Panics**: no new production panic, unchecked index, or arithmetic failure
  was found in the F-251 paths.
- **OOXML**: prefix-tolerant parsing, quoted and entity-bearing attributes,
  inherited and local namespace shadowing, QName-valued retained data,
  generated declarations, duplicate page-number elements, exact raw replay,
  schema child order, distinct references, equal references, removal, reorder,
  clone, repeated save, and reopen produced no finding.
- **Oracle**: the source-built subject, exact Word and Poppler pins, full PDF
  page count, complete geometry and PAGE vectors, physical page records, and
  guarded artifact cleanup produced no finding.
- **Tests**: the three named story gates cover round-trip preservation, atomic
  rejection, and the pinned differential record. Focused regressions are
  sensitive to scanner quoting, duplicate order, namespace collision, distinct
  and equal reference placement, signed overflow, endnote restart numbering,
  oracle completeness, and unwind cleanup. No missing blocking trigger was
  found.
- **Structure**: the final diff adds no reference wrapper or identity subsystem.
  No unjustified trait, generic, dynamic dispatch, forwarding wrapper, feature
  flag, crate, module, or source file was added.
