# F-253, all, pass 12

**Reviewed**: complete uncommitted working-tree diff on
`work/f-253-codex`, 5 files and 3,511 changed lines, comprising 3,465
insertions and 46 deletions
**Verdict**: 6 defects, 0 smells, 0 nitpicks

## Defects

### D1, direct content controls bypass their typed admission result
`crates/rdocx/src/document.rs:3985`
`crates/rdocx/src/document.rs:4036`
`crates/rdocx-oxml/src/document.rs:738`
`crates/rdocx-oxml/src/document.rs:767`

Direct owner children are classified from their name whenever their parent is
not opaque. The classification does not use the current child's `opaque`
result. A self-closing `w:sdt` is retained as `BodyContent::RawXml` by the typed
body parser, but story traversal reports it as `ContentControl`. A started
control whose child parsing makes `CT_Sdt::from_body_raw` return `None` is also
retained as one raw body child while story traversal still exposes the control
and any earlier modeled descendants. The item admission decision must follow
the typed parser for the complete direct child.

### D2, revision admission validates only the wrapper attributes
`crates/rdocx/src/document.rs:3589`
`crates/rdocx-oxml/src/revision.rs:55`
`crates/rdocx-oxml/src/revision.rs:64`
`crates/rdocx-oxml/src/text.rs:3793`

The story grammar accepts a content revision after checking only `w:author`
and a numeric `w:id`. The typed projection accepts the complete subtree only
when `CT_Revision::from_raw` also validates its nesting depth and parses its
children. For example, a revision deeper than the 32-level typed limit, or one
with an otherwise modeled run that fails typed parsing, is retained as raw
paragraph XML. Story traversal crosses that raw wrapper and exposes its text
and nested items for mutation.

### D3, an empty quoted first field token is skipped during admission
`crates/rdocx/src/document.rs:3542`
`crates/rdocx-oxml/src/text.rs:7021`
`crates/rdocx-oxml/src/text.rs:7105`

`field_instruction_has_name` ignores an empty quoted token and accepts a later
nonempty token as the field name. The shared typed lexer preserves an empty
quoted token, then the typed parser makes that first token the empty field name
and rejects the field. A simple or complex instruction such as `"" PAGE` is
therefore raw in the typed tree but becomes an editable story field.

### D4, complex fields without a separator are omitted
`crates/rdocx/src/document.rs:3695`
`crates/rdocx-oxml/src/text.rs:2540`

Story admission requires exactly one separator. The typed complex-field
projection rejects a second separator but does not require any separator, so a
balanced begin, instruction, and end sequence with no cached-result separator
is a typed field with an empty result. Story traversal omits that field and
therefore does not traverse the existing typed tree.

### D5, complex fields cross boundaries that invalidate typed projection
`crates/rdocx/src/document.rs:3735`
`crates/rdocx-oxml/src/text.rs:2560`
`crates/rdocx-oxml/src/text.rs:2664`

The story field stack remains valid across every modeled element between its
begin and end markers. The typed projection additionally rejects a field span
that crosses a comment range, bookmark, content control, revision, or partial
hyperlink boundary. A balanced field with a bookmark or content-control
boundary between its markers is consequently ordinary typed content, but the
story scanner exposes it as one editable `Field` and can rewrite its cached
result.

### D6, complex field XML is not an item subtree
`crates/rdocx/src/document.rs:362`
`crates/rdocx/src/document.rs:3737`
`crates/rdocx/src/document.rs:3768`

Complex-field spans begin at the opening field marker and end after the closing
marker. Normal markers live inside separate `w:r` owners, so the returned byte
range starts after the first run's opening tag and ends before the last run's
closing tag. `StoryItemRef::xml`, documented as the exact item subtree, thus
returns a non-well-formed fragment for every ordinary complex field.

## Smells

None. Count: 0.

## Nitpicks

None. Count: 0.

## Not found

- **Owner identity and ordering**: no additional relationship-order,
  fingerprint, owner-index, or stale-resolution defect was found.
- **Text replacement outside the findings**: nested owner exclusion, XML text
  decoding, CDATA handling, empty text nodes, and `xml:space` updates remain
  aligned and atomic.
- **Namespaces and retained XML outside the findings**: fixed-prefix reads,
  unrelated opening-tag rewrites, and foreign-wrapper traversal did not recur.
- **Lifecycle**: identifier provenance, optional-part ownership, signature
  state, and dirty-state restoration remain preserved through staged reopen.
- **Panics**: no added production panic, unchecked index, or non-wrapping
  arithmetic on story input was found.
- **Structure**: no new trait, generic parameter, feature flag, crate, module,
  forwarding wrapper, or dynamically dispatched concrete dependency was
  introduced.
- **Tests**: the existing pass 11 regressions do not cover self-closing direct
  controls, full-subtree revision rejection, an empty quoted first field token,
  separator-free complex fields, typed boundary crossings, or complex-field
  `xml()` well-formedness.
