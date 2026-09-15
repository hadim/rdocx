# F-X106b, correctness, pass 1

**Reviewed**: working tree against `HEAD`, 11 files, 506 insertions and 109
deletions
**Verdict**: 5 defects, 0 smells, 0 nitpicks

## Defects

### D1, the Run property has the wrong public name

`crates/rdocx-py/src/run.rs:118`

The approved contract and issue request expose the character style ID as
`Run.style_id`, but the binding publishes `Run.style`. Code written against
the requested API still raises `AttributeError`.

### D2, the named binding gate is absent

`crates/rdocx-py/tests/test_formatting_tables.py:284`

The plan and backlog require
`python_paragraph_and_run_formatting_matches_native_facades` as the binding
gate. The current tests split the behavior across differently named cases, so
the executable gate named by the delivery record does not exist.

### D3, the mixed-content regression omits fields and drawings

`crates/rdocx/src/run.rs:1327`

The regression covers text, a tab, a break, and one raw symbol. It does not
include the field and drawing children required by the test plan, so it cannot
prove that all named ordered run content survives the new setters.

### D4, the typing gate does not exercise three new property groups

`crates/rdocx-py/tests/typing_smoke.py:43`

The strict typing source assigns highlight and shading, but never assigns or
reads paragraph style, paragraph numbering, or run style ID. A wrong stub for
any of those properties can still pass the story's mypy gate. The highlight
stub also accepts every string instead of expressing the bounded Word keyword
set that runtime validation enforces.

### D5, uppercase auto is accepted and serialized unchanged

`crates/rdocx-py/src/formatting.rs:48`

The validator accepts case-insensitive forms such as `AUTO` and returns the
original spelling. The setter then emits that spelling as `w:fill`, although
the OOXML enumeration value is the case-sensitive token `auto`. Accepted input
can therefore produce schema-invalid XML.

## Smells

No smells found.

## Nitpicks

No nitpicks found.

## Not found

No additional correctness, contract, panic, OOXML child-order, preservation,
test-isolation, or structural-rule findings were found.
