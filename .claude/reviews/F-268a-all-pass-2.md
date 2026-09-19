# F-268a, all, pass 2

**Reviewed**: `git diff` on `work/f-268a-claude` after the pass 1 remediation,
13 files, 2324 insertions and 72 deletions, against
`.claude/plans/F-268a-design.md`.
**Verdict**: 0 defects, 0 smells, 4 nitpicks

## Defects

None.

## Smells

None.

## Nitpicks

Carried forward from pass 1 unchanged, all four accepted.

- `crates/rdocx-oxml/src/table.rs:799`, a duplicate `w:tblpPr` keeps the last
  one and drops the first, which is how `w:tblW` and `w:jc` already behave in
  this parser.
- `crates/rdocx-oxml/src/table.rs:422`, the three new enums are not
  `#[non_exhaustive]`. Each is a closed ECMA vocabulary and `AnchorAlignH`
  beside them is closed too.
- `crates/rdocx-layout/src/table.rs:456`, `omitted_after` is computed for every
  row and consumed only by a bidirectional one.
- `crates/rdocx/src/table.rs:510`, a `w:tblpPr` carrying neither an offset nor
  an alignment spec reads back as a zero offset rather than as absent, which is
  Word's default and is stated in the function's doc comment.

## Pass 1 findings, re-checked

- **D1 closed.** `crates/rdocx-oxml/src/table.rs:407` unescapes the two
  free-text `w:val` attributes and keeps a malformed entity verbatim. The
  round-trip test authors `Totals & targets` and
  `Region < quarter, by "total"`, asserts the escaped bytes in
  `/word/document.xml`, and asserts the decoded values after reopen. The other
  `w:val` readers in the family are token valued and are untouched.
- **D2 closed.** `crates/rdocx-layout/src/table.rs:1102` adds the horizontal
  margin once to each of the two measurements. The golden and the corpus-shaped
  guard were re-pinned to the corrected 20.34 point column in one labelled edit,
  with the reason recorded in the constant's doc comment.
- **S1 closed.** `crates/rdocx-layout/src/table.rs:1159` gives a cell that
  holds a nested table that table's declared grid rather than a measured width,
  so autofit costs one layout per nesting level instead of three. The
  production pass still lays the nested table out for real, and
  `docs/hld/08-rendering-spec.md` states the rule and the reason.
- **S2 closed.** `crates/rdocx-layout/src/table.rs:1044` resolves the row's
  properties before reading `w:gridBefore`, so measurement and painting assign
  cells to the same grid columns.

## Not found

- **Correctness.** The engagement predicate is exercised in both arms by
  `autofit_engages_only_for_an_auto_width_autofit_table` across an absent
  width, an explicit `fixed` layout, an authored `dxa` width, an authored `pct`
  width and an `auto` width. The distribution is exercised at a roomy and a
  tight caller width. Bidirectional placement is checked against the real
  paginator through `Document::layout_deterministic`, not against a
  reimplementation of the rule in the test.
- **Contract.** Nothing outside the plan. Floating placement, row splitting,
  `w:cantSplit` as a distinction, a non-floating table beside a float, and
  binding parity are all absent, as the parent records.
- **Panics.** No new `unwrap`, `expect`, indexing or slicing on parsed input.
- **OOXML.** Prefix-tolerant read, fixed `w:` write, `xsd:sequence` order held
  across both slot lists, and `capture_element` still preserving every
  unmodelled sibling.
- **Structure.** No new trait, generic parameter, `Box<dyn>`, wrapper, feature
  flag, crate, module or file.
- **Tests.** No new file under any `tests/` directory. One clearly named
  top-level `mod` appended to each of the two existing entrypoints. No binary
  fixture.
- **Harness.** 49 of 49 entries match after every remediation.
