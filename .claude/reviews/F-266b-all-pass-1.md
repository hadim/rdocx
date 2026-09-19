# F-266b, all, pass 1

**Reviewed**: the uncommitted working tree on `work/f-266b-claude`, 9 tracked
files plus the new `crates/rdocx-oxml/src/ruby.rs`, 1257 insertions and 6
deletions tracked, 547 lines in the new module.
**Verdict**: 5 defects, 4 smells, 4 nitpicks

Aspects reviewed: correctness, contract, panics, ooxml, tests, structure.

## Defects

### D1, a raw child at the first base run of a ruby is dropped from layout
`crates/rdocx-layout/src/engine.rs:6053`

The ruby span check sat before `push_equations_before_order`, so the `continue`
that skips a base run also skipped the equation and raw ordering pass for that
boundary. A `w:oMath` captured at the boundary equal to `ruby.base_start` never
reached `inline_items`, so it vanished from the page while staying in the file.

### D2, a hand-built ruby span past the end of the run list panics
`crates/rdocx-layout/src/engine.rs:6061`

`&para.runs[ruby.base_range()]` slices with a range taken from public fields.
The parser and `Paragraph::add_ruby` both produce valid spans, but `CT_Ruby`
exposes `base_start` and `base_end` publicly, so a caller that sets `base_end`
past `runs.len()` gets a slice-index panic inside layout rather than a skipped
annotation.

### D3, a ruby change cannot move the paragraph layout fingerprint
`crates/rdocx-layout/src/engine.rs:5048`

`paragraph_fingerprint` walked `runs` and `properties` only. Two paragraphs with
identical runs and different annotations, for example one with `w:hpsRaise`
changed, produced the same fingerprint, so a cached block laid out before the
change could be served after it.

### D4, a run inserted before a ruby left the span on the wrong runs
`crates/rdocx-oxml/src/text.rs:4164`

`CT_P::split_run` and `CT_P::insert_unwrapped_run` shift every recorded run
boundary, and `split_run` grows a `w:hyperlink` that contains the split point.
Ruby spans were not in either list, so splitting a run before an annotation
left `base_start` and `base_end` pointing one run short. The next save wrapped
the neighbouring run in `w:rubyBase` and put the annotated text outside it.

### D5, collapsing a complex field moved the runs out from under a ruby
`crates/rdocx-oxml/src/text.rs:3372`

`remap_complex_field_boundaries` rewrites every run boundary when the parser
replaces a `w:fldChar` sequence with one synthetic run. A ruby after such a
field in the same paragraph kept its original indices while every run moved,
so a producer file with a field before a ruby reopened with the annotation on
the wrong runs.

## Smells

### S1, a `w:rubyPr` child without its required `w:val` was normalised away
`crates/rdocx-oxml/src/ruby.rs:158`

`read_child` reported `w:hps` and `w:rubyAlign` modelled even when
`get_word_val_attr` returned `None`, which left the typed slot empty and
discarded the element. The run property module keeps a valueless `w:kern` raw
for exactly this reason, and the same rule belongs here.

### S2, whitespace between ruby children rejected the crate's own output
`crates/rdocx-oxml/src/ruby.rs:131`

The first cut rejected every character-data event between children. This crate's
writer indents `document.xml`, so a ruby it had just written failed to re-model
on reopen and degraded to raw. Rejecting non-whitespace data is the property
worth having, rejecting indentation is not.

### S3, the new module was invisible to the capability matrix audit
`scripts/test_sprint_workflow.py:9605`

`test_modern_docx_matrix_covers_public_facade_and_modeled_property_families`
asserts the modelled module set equals its family map, so a new
`rdocx-oxml` module fails the workflow suite until it is placed in a family.

### S4, `ParagraphRef` could not read a ruby
`crates/rdocx/src/paragraph.rs:2489`

`add_ruby`, `ruby` and `ruby_mut` landed on the mutable `Paragraph` only, so a
document reopened through `Document::paragraphs` could round-trip an annotation
it had no way to read back. An asymmetric accessor pair is the kind of gap a
caller discovers at the worst time.

## Nitpicks

- `crates/rdocx-layout/src/engine.rs:3750`, `paragraph_key_retained_bytes` does
  not count a ruby's retained bytes, so a ruby-heavy document under-reports
  against the cache budget. The same function already omits `language` and
  `emphasis_mark`, so this is the existing level of approximation rather than a
  new gap.
- `crates/rdocx-layout/src/engine.rs:8790`, the emphasis group paints its
  highlight over the annotation box rather than the full line height the
  ordinary text path uses, so a highlight behind a marked run is one line-gap
  shorter than behind an unmarked one.
- `crates/rdocx-oxml/src/text.rs:5280`, two ruby annotations sharing one
  `base_start` open only the first, so the second's phonetic line is not
  written. Overlapping spans are caller error and the parser cannot produce
  them.
- `crates/rdocx-layout/src/engine.rs:8688`, an emphasis-marked run carries no
  `hyperlink_url`, so a marked run inside a hyperlink gets no link annotation
  rectangle. The text and the underline are unaffected.

## Not found

- **Panics**, beyond D2. No `unwrap`, `expect`, integer subtraction on an
  unchecked value or unchecked indexing in the new parser, serialiser or layout
  code. `distribute_ruby_line` decrements `remaining` exactly once per glyph it
  counted, so it cannot underflow, and the zero-glyph case never enters the
  loop.
- **Schema order**, nothing found. `w:rubyPr`, `w:rt` and `w:rubyBase` write in
  `xsd:sequence` order, `w:rubyPr`'s six children write in theirs, and `w:em`
  was already in its `EG_RPrBase` slot from F-265. The order is asserted rather
  than assumed by
  `ruby_authors_saves_and_reopens_with_base_and_phonetic_runs` and
  `emphasis_marks_reopen_as_modeled_state`.
- **Prefix handling**, nothing found. Read is tolerant through
  `word_prefixes_at` at every nesting level including a binding declared on the
  `w:ruby` element itself, and write is the fixed `w:` prefix throughout.
- **Unmodelled subtrees**, nothing found beyond S1. A `w:ruby` this model
  cannot reproduce exactly stays in `extra_xml` verbatim, and an unmodelled
  `w:rubyPr` child is retained.
- **Structure**, nothing found. No new trait, no new generic parameter, no
  `Box<dyn>`, no forwarding wrapper, no new feature flag. One new module, which
  the design plan approved. `AnnotationBase`, `AnnotationLine` and
  `AnnotationContext` are private argument groupings that replace long
  parameter lists, following the existing `ParagraphBoundary` precedent.
- **Stack budget**, nothing found. `CT_P` grows by one `Vec`, 2360 to 2384
  bytes, and `size_of::<Document>` is 27192 before and after. Nothing new is
  held by value, so no member needed boxing and `RUST_MIN_STACK` is untouched.
- **Hash harness**, nothing found. 49 of 49 entries match. Every new branch is
  entered only on an element no sample carries.
