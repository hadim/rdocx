# F-266b, all, pass 2

**Reviewed**: the uncommitted working tree on `work/f-266b-claude` after the
pass 1 remediation, 13 tracked files plus `crates/rdocx-oxml/src/ruby.rs`.
**Verdict**: 0 defects, 0 smells, 4 nitpicks

Aspects reviewed: correctness, contract, panics, ooxml, tests, structure.

## Defects

None.

Each pass 1 defect was re-read against the current tree.

- **D1** is closed. The ruby span check now sits after
  `push_equations_before_order` and before the `w:vanish` skip, at
  `crates/rdocx-layout/src/engine.rs:6070`, so a raw child or equation at the
  boundary of the first base run is emitted before the span is consumed.
- **D2** is closed. `crates/rdocx-layout/src/engine.rs:6080` takes
  `para.runs.get(ruby.base_range())` and skips an out-of-range span instead of
  slicing it.
- **D3** is closed. `crates/rdocx-layout/src/engine.rs:5051` folds every ruby
  span and phonetic string into `paragraph_fingerprint`, and writes nothing
  when there is no annotation, so no existing fingerprint moved.
- **D4** is closed. `CT_P::shift_ruby_spans_for_insert` at
  `crates/rdocx-oxml/src/text.rs:4237` moves or grows every span, and both
  `insert_unwrapped_run` and `split_run` call it beside the hyperlink span
  maintenance they already did.
  `splitting_a_run_before_a_ruby_keeps_the_annotation_on_its_own_base` fails
  without it.
- **D5** is closed. `ComplexFieldProjection` and `ComplexFieldBoundariesMut`
  carry `rubies`, and `remap_complex_field_boundaries` at
  `crates/rdocx-oxml/src/text.rs:3434` remaps both span ends through the same
  closure the hyperlink spans use.

## Smells

None.

- **S1** is closed. `CT_RubyPr::read_child` at
  `crates/rdocx-oxml/src/ruby.rs:158` reports a child missing its required
  `w:val` unmodelled, so it is retained verbatim.
  `a_ruby_property_child_without_its_required_value_stays_raw` pins it.
- **S2** is closed. `crates/rdocx-oxml/src/ruby.rs:131` rejects only
  non-whitespace character data, so a ruby this crate wrote re-models on
  reopen. `ruby_authors_saves_and_reopens_with_base_and_phonetic_runs` reads
  back the annotation it saved, which is the path that failed before.
- **S3** is closed. `scripts/test_sprint_workflow.py:9606` places `ruby` in the
  `paragraph` family, which is where the `DOCX-033` row sits.
- **S4** is closed. `ParagraphRef::rubies` and `ParagraphRef::ruby` at
  `crates/rdocx/src/paragraph.rs:2489` match the mutable pair, and the
  regression tests read annotations back through `Document::paragraphs`.

## Nitpicks

Carried forward from pass 1, all four deliberate and recorded rather than
fixed.

- `crates/rdocx-layout/src/engine.rs:3768`, `paragraph_key_retained_bytes` does
  not count a ruby's retained bytes. The same function already omits
  `language` and `emphasis_mark`, so this matches the existing approximation.
- `crates/rdocx-layout/src/engine.rs:8810`, the emphasis group paints its
  highlight over the annotation box rather than the full line height.
- `crates/rdocx-oxml/src/text.rs:5299`, two ruby annotations sharing one
  `base_start` open only the first. The parser cannot produce that shape.
- `crates/rdocx-layout/src/engine.rs:8708`, an emphasis-marked run carries no
  `hyperlink_url`, so no link annotation rectangle is emitted for it.

## Not found

- **Contract**, nothing found. Both work groups the plan assigns to this story
  are delivered. `w:em` was already modelled by F-265, so work group C reduced
  to the render projection the plan describes, and every named test exists.
  Two deviations from the plan's literal text are recorded in the handoff:
  `CT_Ruby` holds its base as a run span rather than an owned `Vec<CT_R>`,
  which is what gives text extraction the base line for free, and `CT_RubyPr`
  carries its own `raw_xml` so an unmodelled child is written back inside
  `w:rubyPr` rather than outside it.
- **Correctness**, nothing found beyond the closed defects. The annotation
  geometry was checked by hand against the group placement contract: with the
  default raise the phonetic line occupies the band above the base and the
  base top sits exactly at the phonetic line's descent, and with an explicit
  raise the group ascent is the larger of the base ascent and the raised
  phonetic top, so nothing is clipped.
- **Panics**, nothing found. No `unwrap`, `expect`, unchecked index or
  unchecked subtraction in the new code paths.
- **OOXML**, nothing found. Schema order, prefix tolerance on read, fixed `w`
  prefix on write and verbatim retention of everything unmodelled are each
  asserted by a test rather than assumed.
- **Tests**, nothing found. The gate fails against reverted code, because the
  digest covers the annotation glyphs and the paint order the projection
  produces, and the assertions before the digest name each property
  separately.
- **Structure**, nothing found. One new module, approved by the plan. No new
  trait, generic parameter, `Box<dyn>`, forwarding wrapper or feature flag.
- **Stack budget**, nothing found. `CT_P` 2360 to 2384 bytes,
  `size_of::<Document>` 27192 unchanged, `RUST_MIN_STACK` untouched.
- **Hash harness**, nothing found. 49 of 49 entries match.
