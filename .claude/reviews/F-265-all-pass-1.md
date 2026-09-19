# F-265, all, pass 1

**Reviewed**: the uncommitted working tree on `work/f-265-claude`, 28 files,
3171 insertions and 95 deletions
**Verdict**: 2 defects, 2 smells, 4 nitpicks

## Defects

### D1, a symbol read from a producer document leaves the ODT body silently
`crates/rdocx/src/odt.rs:503`

`RunContent::Symbol { .. } => 0` counts no ODF piece and the emitter writes
nothing, which is the right projection, but the validator at
`crates/rdocx/src/odt.rs:880` is the only place a caller learns that the
character was dropped. The two live in different passes and nothing ties them
together, so a later edit to one silently desynchronises the piece count from
the diagnostic. The piece count and the emitter agree today and the diagnostic
fires today, so this is currently correct behaviour with no test holding it.

**Resolution**: covered by adding an assertion that an authored symbol produces
an ODT diagnostic and no body text.

### D2, `w:kern` and `w:fitText` without their required value lose bytes
`crates/rdocx-oxml/src/run_properties.rs:760`, `crates/rdocx-oxml/src/run_properties.rs:775`

Before F-265 both elements were retained as positioned raw XML at their schema
slot. Typing them consumed the element as modeled, and a producer element
without its required `w:val` left the typed field `None`, so the element
disappeared on save. `w:sz`, `w:spacing`, `w:w` and `w:position` have the same
shape today, so the pattern was easy to copy, but copying it here introduces a
new loss rather than preserving an old one.

**Resolution**: fixed in this pass. Both arms now require the value in the
match condition and fall through to raw capture without it, with
`a_new_element_missing_its_required_value_stays_raw` locking it down.

## Smells

### S1, `set_color_theme` could author a tint with no theme colour
`crates/rdocx/src/run.rs:754`

Passing `None` for the reference with `Some` for the tint wrote
`<w:color w:themeTint="33"/>`, a state Word cannot resolve, because tint and
shade modify a theme colour that is not there.

**Resolution**: fixed in this pass. The tint and shade now follow the
reference, and the regression test asserts that clearing the reference clears
all three.

### S2, the EPUB heading label projected different text from `CT_R::text`
`crates/rdocx/src/epub.rs:1726`

`projected_paragraph_text` spelled out one arm per special character and
produced a non-breaking hyphen and a soft hyphen where `CT_R::text` produces
nothing. Two text projections of the same content that disagree is a place a
later offset bug hides.

**Resolution**: fixed in this pass. The helper now mirrors `CT_R::text`
exactly, with the reason in a comment.

## Nitpicks

- `crates/rdocx/src/run.rs:376`, `raw_is_last_rendered_page_break` matches on
  the local name because the captured subtree does not always carry the binding
  that named its prefix. The legacy horizontal rule uses a parse-time flag in
  the encoded position instead. The flag is the stronger mechanism, and the
  reason for not using it here is that the element needs no flag bit to be
  recognised and no Word-adjacent namespace defines the name. Recorded so the
  choice is visible.
- `crates/rdocx-oxml/src/run_properties.rs:743`, run shading moved from
  `CT_Shd::from_xml_attrs` to `from_xml_attrs_with_prefixes`, which is what
  table shading already uses. Paragraph shading still uses the prefix-agnostic
  reader. The move is deliberate, since the prefix-agnostic reader would type a
  foreign `x:themeTint` into the Word slot and re-emit it as `w:themeTint`,
  which is exactly the loss this story exists to remove.
- `crates/rdocx/src/epub.rs:3439`, `measure_shading` still counts only `val`,
  `color` and `fill`. The EPUB projection keeps only the literal fill, so the
  theme attributes contribute no output bytes and the bound stays sound.
- `crates/rdocx-oxml/src/run_properties.rs:800`, `<w:effect/>` and `<w:em/>`
  with no `w:val` serialise as `w:val="none"`. `w:u` already normalises this
  way, and the alternative is to drop the element, so the byte change is the
  better of the two.

## Not found

- **panics**: no `unwrap`, `expect`, slicing or arithmetic on parsed input in
  the new code outside `#[cfg(test)]`. `parse_uchar_hex` checks the length and
  the digit class before `from_str_radix`. `raw_is_last_rendered_page_break`
  indexes only within a bound it computed.
- **ooxml**: the 16 new children write at their `EG_RPrBase` ordinals between
  the neighbours already written, proved by
  `new_run_properties_write_at_their_schema_ordinals`. Reads accept an aliased
  Word prefix and writes use the fixed `w:` prefix. Every unmodelled subtree
  and attribute reaches the output, and `CT_BorderEdge`, `CT_FitText`,
  `CT_EastAsianLayout`, `CT_Shd`, `w:rFonts` and `w:color` all replay their
  retained attributes ahead of the modeled ones.
- **contract**: the diff carries the 16 members, the three lossy elements, the
  six theme-clearing rules, exactly two `RunContent` variants, the read-only
  `w:lastRenderedPageBreak` classification, the six named F-266a
  prerequisites, and `apply_tint_shade` made live with a zero-line diff on
  `crates/rdocx-oxml/src/theme.rs`. Nothing outside the plan's scope is added.
- **structure**: no new trait, no new generic parameter, no new crate, no new
  module and no new file. `Run::set_toggle` and `RunRef::property` take an
  `impl FnOnce` accessor and are instantiated ten and twenty ways in this diff.
- **tests**: the story gate would fail if the feature were reverted, because it
  asserts a byte-identical `w:r` across a save of the pinned reference and
  reads every new property back through the public facade.
