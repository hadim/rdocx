# F-251, all aspects, pass 3

**Reviewed**: complete working-tree source diff from `25226ae5be006391463e509525e629e20f060d5b`, 7 files, 1,534 insertions, 101 deletions, 1,635 changed lines, plus pass 1 and pass 2 reviews
**Verdict**: 2 defects, 0 smells, 0 nitpicks

## Prior-pass closure

All six pass 1 defects remain closed.

- Pass 1 D1 is closed. Existing `start` edits use parsed attribute spans, and
  the start-tag scanner respects quoted values.
- Pass 1 D2 is closed. The first `pgNumType` remains typed and later duplicate
  elements replay after it.
- Pass 1 D3 is closed for distinct repeated references. Retained children now
  carry predecessor and successor anchors instead of only a schema slot.
- Pass 1 D4 is closed. Checked geometry converts full-width EMUs to twips
  before the checked `i32` narrowing.
- Pass 1 D5 is closed. Legacy document setters keep their explicit unchecked,
  infallible compatibility path.
- Pass 1 D6 is closed. The checked-in Word record has a pinned regeneration
  route, and the normal gate checks complete multi-digit displayed values and
  physical page identity.

All six pass 2 defects are closed for their stated triggers.

- Pass 2 D1 is closed. Fresh and restarted endnote pages continue the final
  body page's displayed sequence, and late PAGE substitution is covered by a
  warm-versus-fresh cache comparison.
- Pass 2 D2 is closed. `Section::set_margins` validates all four signed values
  through the checked conversion before mutating any field.
- Pass 2 D3 is closed for existing Word aliases and a locally shadowed `w`
  prefix. Missing `start` insertion reuses a Word-bound alias or declares a
  generated alias.
- Pass 2 D4 is closed for distinct reference values under front removal,
  clearing, and reordering. The remaining equal-value case is D1 below.
- Pass 2 D5 is closed. Regeneration reads the total PDF page count and requires
  complete size and displayed-number vectors.
- Pass 2 D6 is closed. A scope guard removes the oracle directory on success
  and during unwinding, with a focused unwind check.

## Defects

### D1, equal repeated references collapse the retained-child boundary
`crates/rdocx-oxml/src/document.rs:883`

The new predecessor and successor anchors store cloned `HdrFtrRef` values, then
resolve the successor with the first equal vector element and the predecessor
with the last equal element. For source order `A, retained, A`, both anchors are
equal to both references, so unchanged serialization resolves the successor to
index zero and emits `retained, A, A`. The retained child has moved even before
any caller mutation. Removing or reordering equal references remains ambiguous
for the same reason. Stable repeated-child replay needs occurrence identity in
addition to value equality, and coverage must include identical header and
footer references.

### D2, generated restart prefixes can shadow an inherited producer binding
`crates/rdocx-oxml/src/document.rs:978`

The generated-prefix collision check examines only namespace declarations and
qualified names in the captured `pgNumType` start tag. It does not consider
prefixes already bound in the inherited scope. A default-Word `pgNumType` can
therefore inherit `xmlns:rdocxWord="urn:producer"`, locally shadow `w`, and
retain an extension attribute whose QName-valued content is
`rdocxWord:ProducerType`. Adding `start` declares local
`xmlns:rdocxWord` for the Word namespace. The retained attribute bytes are
unchanged, but their namespace meaning has changed. Generated names must avoid
every in-scope prefix, including bindings used only by retained QName-valued
content, not only names visible in the detached start tag.

## Smells

None.

## Nitpicks

None.

## Focused checks

No test suite was run in pass 3. The audit inspected the complete current diff,
both prior reviews, and the focused remediation code and tests. The external
Word oracle was not rerun.

## Not found

- **Correctness**: no additional fresh or restarted endnote numbering, PAGE
  substitution, header parity, physical page identity, PageFrame clone, or
  incremental-cache defect was found.
- **Contract**: signed margin minimum and maximum boundaries, atomic rejection,
  legacy infallible document setters, new checked section setters, and current
  public API compatibility produced no additional finding.
- **Panics**: no new production `unwrap`, `expect`, unchecked index, or
  arithmetic panic was found.
- **OOXML**: quoted values, entities, existing Word aliases, duplicate
  `pgNumType` ordering, distinct-reference front removal and reorder, clearing,
  ordinary schema child order, and unchanged raw-subtree replay produced no
  additional finding beyond D1 and D2.
- **Oracle**: exact total-page checks, complete geometry and displayed-number
  vectors, physical page records, version pins, and cleanup-guard behavior
  produced no additional finding.
- **Tests**: existing focused tests cover the two pass-2 endnote paths, signed
  margin conversion, distinct reference mutations, generated-prefix reparsing,
  oracle vector completeness, and unwind cleanup. No additional test issue was
  found beyond the missing triggers for D1 and D2.
- **Structure**: no unjustified trait, generic, dynamic dispatch, forwarding
  wrapper, feature flag, crate, module, or source file was added.
