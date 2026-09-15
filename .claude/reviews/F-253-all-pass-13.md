# F-253, all, pass 13

**Reviewed**: complete uncommitted working-tree diff on
`work/f-253-codex`, 8 files and 4,185 changed lines, comprising 4,137
insertions and 48 deletions
**Verdict**: 3 defects, 0 smells, 0 nitpicks

## Defects

### D1, a nested separator suppresses the separator required by its outer field
`crates/rdocx/src/document.rs:4702`
`crates/rdocx/src/document.rs:4791`
`crates/rdocx/src/document.rs:4871`

The mutation scan records one `complex_separator_found` flag for the complete
field span. A valid nested field separator therefore sets the flag even when
the selected outer field has no separator. An accepted separator-free outer
field whose instruction contains a valid nested field consequently reaches
the insertion path with this flag set. The mutation inserts the new `w:t` but
does not insert an outer separator, so the typed field still has an empty
cached result after reopen and the requested visible text does not land.
Separator state must be tracked per open field, like result visibility and end
marker depth already are.

### D2, separator insertion emits an unqualified Word attribute for default-prefixed stories
`crates/rdocx/src/document.rs:4860`
`crates/rdocx/src/document.rs:4874`

The separator authoring path derives both the element and attribute prefix
from the first run's qualified name. For a story using the Word namespace as
its default namespace, that prefix is empty and the generated XML is
`<fldChar fldCharType="separate"/>`. The default namespace applies to the
element but never to an unprefixed attribute, so `fldCharType` is not a Word
attribute. The typed parser and the next story scan do not recognize it as a
separator, which leaves the inserted text outside the cached result. The code
must select an in-scope nonempty Word prefix for the attribute or declare one.

### D3, the public XML documentation promises ownership and namespace completeness it does not provide
`crates/rdocx/src/document.rs:341`
`crates/rdocx/src/document.rs:365`
`crates/rdocx/src/document.rs:374`
`crates/rdocx/src/document.rs:8505`
`crates/rdocx/src/document.rs:8564`

`StoryItemRef` says XML for real subtrees borrows the opened package, and
`xml()` promises a well-formed XML representation. Body sources and comment
sources are serialized into owned buffers, so ordinary real paragraph items
from those stories return `Cow::Owned`. Conversely, a package-backed header
paragraph returns its exact borrowed subtree slice. Such a slice commonly
uses `w:` while inheriting `xmlns:w` only from the omitted `w:hdr` root, so it
is not a namespace-complete standalone XML representation. Only the complex
field branch constructs an owned namespace-complete projection. The public
documentation must distinguish owned typed-source slices, borrowed raw
subtrees that can depend on ancestor declarations, and the standalone complex
field projection.

## Smells

None. Count: 0.

## Nitpicks

None. Count: 0.

## Not found

- **Pass 12 D1**: direct content controls now use complete typed-parser
  admission, including self-closing and malformed controls.
- **Pass 12 D2**: revision wrappers now use complete typed-parser admission,
  including nesting-depth and child-parse rejection.
- **Pass 12 D3**: simple and complex field names now use the shared typed field
  lexer, including the empty quoted first-token behavior.
- **Pass 12 D4**: flat separator-free complex fields are admitted and receive
  a separator during mutation. D1 and D2 are the remaining nested and
  default-namespace variants of that path.
- **Pass 12 D5**: comment, bookmark, content-control, revision, and partial
  hyperlink boundaries continue to invalidate complex-field admission.
- **Pass 12 D6**: complex-field `xml()` constructs one owned, namespace-complete
  paragraph projection. D3 concerns the broader public promise made for the
  other return branches.
- **Owner identity and atomicity**: no additional relationship-order,
  fingerprint, bounds, kind, stale-resolution, or staged-commit defect was
  found.
- **Retained XML and namespaces outside the findings**: opaque child grammar,
  direct wrapper admission, unrelated text-tag attributes, and retained raw
  subtree bytes remain protected.
- **Lifecycle**: identifier provenance, optional-part ownership, signature
  state, and dirty-state restoration remain preserved through staged reopen.
- **Panics**: no added production panic, unchecked index, or non-wrapping
  arithmetic on story input was found.
- **Structure**: the public-hidden `rdocx-oxml` admission helpers directly
  expose existing concrete parsers to the facade. No new trait, generic
  parameter, feature flag, crate, module, forwarding wrapper, or dynamically
  dispatched concrete dependency was introduced.
- **Tests**: the pass 12 regressions cover the six reported triggers from that
  pass. They do not cover a separator-free outer field with a separated nested
  instruction field, default-namespace field authoring, or the documented
  `Cow` and namespace-completeness variants.
