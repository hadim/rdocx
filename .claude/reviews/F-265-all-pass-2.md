# F-265, all, pass 2

**Reviewed**: the uncommitted working tree on `work/f-265-claude` after the
pass 1 remediation, 28 files, about 3230 insertions and 95 deletions
**Verdict**: 0 defects, 0 smells, 4 nitpicks

## Defects

None. The two recorded in pass 1 are closed.

- **D1**, the symbol projection now has a test.
  `a_dropped_symbol_is_diagnosed_and_the_special_characters_are_exported` at
  `crates/rdocx/tests/integration_test.rs` asserts the ODT and RTF exporters
  both diagnose the dropped symbol and that the RTF output carries `\_` for the
  non-breaking hyphen and `\line ` for the carriage return, so the piece count,
  the emitter and the diagnostic cannot drift apart unnoticed.
- **D2**, `crates/rdocx-oxml/src/run_properties.rs:760` and
  `crates/rdocx-oxml/src/run_properties.rs:777` now require the value in the
  match condition, so `<w:kern/>` and `<w:fitText w:id="4"/>` fall through to
  positioned raw capture and reopen byte for byte.
  `a_new_element_missing_its_required_value_stays_raw` locks it down.

## Smells

None. The two recorded in pass 1 are closed.

- **S1**, `crates/rdocx/src/run.rs:756` passes the tint and shade through
  `Option::and` on the reference, so clearing the theme colour clears them too
  and `<w:color w:themeTint="33"/>` with no reference is unrepresentable
  through the facade.
  `explicit_colour_replacement_clears_theme_colour_tint_and_shade` asserts it.
- **S2**, `crates/rdocx/src/epub.rs:1726` now mirrors `CT_R::text` exactly, so
  the heading label and the run text it labels cannot disagree.

## Nitpicks

Carried unchanged from pass 1, each with its reason on the record.

- `crates/rdocx/src/run.rs:376`, `raw_is_last_rendered_page_break` matches on
  the local name rather than on a parse-time position flag.
- `crates/rdocx-oxml/src/run_properties.rs:743`, run shading uses the
  prefix-aware `CT_Shd` reader while paragraph shading keeps the
  prefix-agnostic one.
- `crates/rdocx/src/epub.rs:3439`, `measure_shading` counts only the three
  attributes the EPUB projection keeps.
- `crates/rdocx-oxml/src/run_properties.rs:807`, `<w:effect/>` and `<w:em/>`
  with no `w:val` serialise as `w:val="none"`, following `w:u`.

## Not found

- **correctness**: the slot table, the parse arms, the write arms, `is_empty`
  and `merge_from` agree on all 16 new members and on every new attribute. The
  ordinal ordering, the aliased-prefix read, the fixed-prefix write, the
  producer-token retention and the foreign-attribute retention each have a
  test. `size_of::<CT_RPr>()` is 696 before and after, `size_of::<CT_PPr>()`
  falls from 2144 to 2080 and `size_of::<Document>()` falls from 27040 to
  26848, so no stack budget moved against the F-084 precedent and no
  `RUST_MIN_STACK` was touched.
- **contract**: every plan deliverable is present and nothing beyond it. The
  one scope reduction, the render projection for `w:outline`, `w:shadow`,
  `w:emboss`, `w:imprint`, `w:bdr`, `w:kern` and `w:fitText`, is recorded in
  `docs/hld/02-scope-and-non-goals.md` on the `DOCX-032` row, in
  `docs/hld/08-rendering-spec.md` and in the `F-265` backlog entry, so the row
  stays `partial` with `F-265` as its live owner rather than claiming a
  completeness the diff does not deliver.
- **panics**: none introduced.
- **ooxml**: schema child order, prefix tolerance, fixed-prefix writes and
  verbatim retention all hold. The single normalisation, the upper-case
  `ST_UcharHexNumber` spelling, is stated in
  `docs/hld/04-opc-and-packaging.md` and a non-hex value is retained verbatim
  instead.
- **structure**: no new trait, generic parameter, crate, module or file.
- **tests**: the named gate, the round-trip set, the F-266a contract, the
  theme-clearing regressions, the pinned `apply_tint_shade` arithmetic, the
  declared font-priority divergence and the no-pixel-change assertion are all
  present, and every one of them fails if the change it covers is reverted.
- **deliberately wrong code**: `crates/rdocx-oxml/src/theme.rs` has a zero-line
  diff. `apply_tint_shade` keeps its name, its 0-255 signature and its naive
  sRGB arithmetic, and no Word call site reaches
  `oxml_drawing::color::apply_tint_shade_pct`. Unit constructors still truncate
  with `as i64`. `rdocx-oxml`'s `drawing.rs` is untouched.
