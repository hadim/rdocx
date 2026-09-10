# F-251, all aspects, pass 1

**Reviewed**: working-tree diff from `25226ae5be006391463e509525e629e20f060d5b`, 7 files, 797 insertions, 97 deletions, 894 changed lines
**Verdict**: 6 defects, 0 smells, 0 nitpicks

## Defects

### D1, page-number rewriting can edit bytes inside another attribute value
`crates/rdocx-oxml/src/document.rs:784`

`replace_page_number_start` locates the first `>` without respecting quoted
attribute values, then searches the resulting byte slice for the qualified
start name without tokenizing attributes. A retained value such as
`x:note="q:start='producer'" q:start="3"` is a valid trigger. The search can
select the text inside `x:note`, rewrite producer data, and leave the real
Word start attribute unchanged. A `>` inside any earlier quoted value also
truncates the search region and can make insertion corrupt that value. The
targeted edit must use an XML-aware start-tag scan and alter only the parsed
Word attribute occurrence.

### D2, duplicate page-number elements are serialized in reverse order
`crates/rdocx-oxml/src/document.rs:297`

On a second `w:pgNumType`, the parser moves the earlier raw element into slot
6 and retains the later element as the typed value. Serialization writes the
typed value before slot 6 at `crates/rdocx-oxml/src/document.rs:649`, so source
order `first, second` becomes `second, first`. This violates the stable retained
child order contract and makes malformed or producer-extended duplicate input
change even without an authored edit. The same logic exists for start-form
elements at `crates/rdocx-oxml/src/document.rs:394`.

### D3, schema slots do not preserve a retained child's exact source boundary
`crates/rdocx-oxml/src/document.rs:552`

The representation records only one coarse schema slot, then emits every
modeled repeated child before that slot. For example, a retained ignorable
child between two `w:headerReference` elements is recorded as slot 1, but both
headers are emitted before slot 1 at `crates/rdocx-oxml/src/document.rs:560`.
The child therefore moves past the second header. The plan requires retained
children to replay at stable positions, including around repeated modeled
children, so a slot needs enough occurrence information to reconstruct that
boundary.

### D4, checked setters accept overflowing positive lengths as unrelated twips
`crates/rdocx/src/document.rs:1898`

The setters convert `Length` to the `i32`-backed twip value before validation.
For example, `Length::emu(((u32::MAX as i64) + 2) * 635)` wraps through
`Length::as_twips()` to one twip and is accepted as a positive page dimension.
The same pattern affects spacing, gutter, and header or footer distances at
`crates/rdocx/src/document.rs:1950`, `crates/rdocx/src/document.rs:1972`, and
`crates/rdocx/src/document.rs:2027`. This is neither checked range handling nor
the caller's requested geometry. Validate the EMU value against the supported
twip range before the narrowing conversion while retaining the specified
integer truncation rule.

### D5, legacy document setters silently discard validation failures
`crates/rdocx/src/document.rs:13146`

The infallible final-section conveniences now call fallible handle setters and
discard their results. `Document::set_page_size(Length::twips(0), ...)` and
`Document::set_columns(0, ...)` return normally without applying the operation
or reporting that it failed. The same silent failure exists for columns,
header or footer distances, and gutter at
`crates/rdocx/src/document.rs:13190`, `crates/rdocx/src/document.rs:13201`, and
`crates/rdocx/src/document.rs:13212`. This does not satisfy either the setter's
documented action or the plan's rejected-value behavior. Delegation needs a
publicly observable failure contract or a compatibility-preserving path whose
behavior is explicit.

### D6, the named differential gate does not compare with a Word-produced record

`crates/rdocx/tests/integration_test.rs:531` labels the story gate as a Word
differential, but the only oracle interaction is comparing a version constant
to the same literal. Geometry and page values are then asserted from the same
numbers used to build the rdocx document. There is no normalized Word-produced
record or ignored regeneration test comparable to the repository's other Word
oracles. The test also reads only the first digit of each PAGE result at
`crates/rdocx/tests/integration_test.rs:611` and never asserts that physical
`PageFrame::page_number` remains `[1, 2, 3]`. This test would pass if its
expected values were derived without running Word and is insensitive to
multi-digit displayed values or loss of physical page identity. Treat this as
completion blocking because the required differential gate is not yet
demonstrated.

## Smells

None.

## Nitpicks

None.

## Focused checks

Focused review runs passed:

- `mixed_orientation_sections_match_word_geometry_and_page_numbers`
- `section_geometry_round_trips_with_unsupported_children_in_order`
- `rejected_section_geometry_is_atomic`

## Not found

- **Correctness**: no additional displayed-number continuation, PAGE field,
  header parity, or physical page-number implementation defect was found.
- **Contract**: no additional property-coverage or final-section delegation
  defect was found.
- **Panics**: no new production `unwrap`, `expect`, unchecked index, or
  arithmetic panic was found.
- **OOXML**: prefix-tolerant `pgNumType` recognition, foreign `start`
  attributes, unchanged raw-subtree replay, fixed-prefix new authoring, and the
  ordinary single-instance schema sequence produced no additional finding.
- **Tests**: the three named tests fail to expose the defects above, but no
  additional test issue was found.
- **Structure**: no unjustified trait, generic, dynamic dispatch, forwarding
  wrapper, feature flag, crate, module, or source file was added.
