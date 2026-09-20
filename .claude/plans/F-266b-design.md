# F-266b, Ruby and emphasis marks

**Status**: completed
**Sprint**: S74
**Size**: L
**Depends on**: F-266a

## Problem

Two of the nine behaviours DOCX-033 names, ruby phonetic guides and East Asian
emphasis marks, have no model anywhere in the workspace. Both survive a
round trip only as unmodelled XML through
`oxml_core::raw_xml::capture_element`, which is the `PV` in the DOCX-033 row at
`docs/hld/02-scope-and-non-goals.md:240`.

**`w:ruby` has no typed model.** The element appears in exactly one place in the
whole crate tree, a redaction skip list at `crates/rdocx/src/redaction.rs:809`
that names `ruby` and `rubyBase` so the redactor steps over them. The run
content model at `crates/rdocx-oxml/src/text.rs:681` carries `Text`,
`DeletedText`, `Tab`, `Break`, `Drawing`, `Field`, `FootnoteRef`, `EndnoteRef`
and `CommentReference`, and nothing else. There is no `Ruby` variant and no
paragraph-content variant either, which matters because `w:ruby` is a sibling of
`w:r` inside `w:p` rather than a child of `w:r`. The redaction fixture at
`redaction.rs:1933` nests it inside `w:r`, which is the shape a test author
reaches for when nothing in the codebase says otherwise.

Because there is no model there is no geometry. A ruby annotation carries a base
line and a phonetic line with their own size, raise and alignment, and raw
preserved XML expresses none of that. Text extraction is a second problem: with
no model, a future modelled ruby would return both the base and the phonetic
text from `text()`, which would double-count every annotated word for search,
redaction and `ActualText`.

**`w:em` has no typed model.** A grep for emphasis across `crates` returns only
table-style banding at `crates/rdocx-oxml/src/table.rs:434`, a character style
named `Emphasis` in tests, and unrelated PresentationML timeline states.
`CT_RPr` in the run property module has no emphasis field, so the five schema
values `none`, `dot`, `comma`, `circle` and `underDot` are unreachable and
unrendered.

## Spec reference

- `docs/hld/08-rendering-spec.md`, "The renderer's input", the paragraph
  beginning "Word uses the same paragraph-wide rich shaping and line path when a
  paragraph contains Arabic, Devanagari, Thai, or CJK text", and the paragraph
  beginning "`MultilingualGlyphRun` is the rich positioned output", for logical
  order retention in `ActualText`, SVG text and diagnostics.
- `docs/hld/04-opc-and-packaging.md`, the namespace-aware modeled part and
  schema-ordered serialization sections, for prefix-tolerant reading, fixed
  prefix writing and `xsd:sequence` child order.
- `docs/hld/02-scope-and-non-goals.md`, "Modern DOCX capability matrix", the
  DOCX-033 row and the paragraph beginning "An owner is required for every
  `partial` or `unsupported` row".
- `docs/hld/12-testing-strategy.md`, "The golden-PNG gate" and the two in-code
  golden paragraphs that follow it, for the recorded-digest pattern with no
  committed fixture.
- `docs/hld/14-development-backlog.md`, "F-266, International and vertical
  typography (L)".

## Prerequisites delivered by earlier stories

F-266a completes before this story starts. From it this story consumes
`TextScript::Hangul` and `TextScript::Kana`, the slot-aware
`resolve_font_family` with `WordFontSlot`, the three bundled subset faces, and
the shared deterministic golden fixture module holding
`mixed_script_page_matches_the_pinned_geometry_and_reading_order`.

F-265 owns the whole `w:rFonts` element including `@w:hint`, and owns the run
setters for `w:rtl`, `w:cs`, the `w:lang` East Asian and complex-script
attributes and the `w:bCs`, `w:iCs`, `w:szCs` pairs. This story adds no setter
F-265 owns.

`crates/rdocx-oxml/src/properties.rs` is split into separate paragraph-property
and run-property modules before wave 1. The `w:em` field lands in **the run
property module**.

## Approach

### C. Emphasis marks

In the run property module, `CT_RPr` gains one field:

```rust
/// East Asian emphasis mark (em).
pub emphasis_mark: Option<ST_Em>,

pub enum ST_Em { None, Dot, Comma, Circle, UnderDot }
```

Read is prefix-tolerant through the existing `is_word_element` helper. Write
uses the fixed `w:` prefix at the `w:em` position in the `EG_RPrBase`
sequence, which is the schema slot the raw-retention table already reserves.
The existing raw-slot machinery keeps a producer `w:em` with unsupported
attributes as a retained carrier, exactly as `w:bidi` and `w:rtl` already do at
`crates/rdocx-oxml/src/properties.rs:693` and `properties.rs:1776`.

`crates/rdocx/src/run.rs` gains the facade setter and reader pair beside the
existing `set_language` at `run.rs:576`.

The layout projection in `crates/rdocx-layout/src/engine.rs` emits one mark
glyph per base character, above the glyph box for horizontal text, using the
font already resolved for that character by F-266a's slot resolution. `None`
emits nothing. A mark codepoint the resolved font cannot draw records a
diagnostic and paints nothing, which matches the existing uncovered-character
policy proven at `crates/oxml-layout/src/font.rs:2991`. Marks never participate
in line breaking and never change the base advance.

### D. Ruby and phonetic guides

`w:ruby` is a paragraph-content sibling of `w:r`, carrying `w:rubyPr`, `w:rt`
and `w:rubyBase`, where the latter two each hold ordinary runs. The typed model
goes in a **new module**, `crates/rdocx-oxml/src/ruby.rs`, approved in the S74
consolidated design round. It is the second-largest typed grammar in the
paragraph content model after the run itself, and putting it in `text.rs` would
push that file past the point where a reader can answer "what does this do?"
from one file.

```rust
pub struct CT_Ruby {
    pub properties: Option<CT_RubyPr>,
    pub ruby_text: Vec<CT_R>,
    pub ruby_base: Vec<CT_R>,
    pub raw_xml: Vec<Vec<u8>>,
}

pub struct CT_RubyPr {
    pub align: Option<ST_RubyAlign>,
    pub hps: Option<HalfPoint>,
    pub hps_raise: Option<HalfPoint>,
    pub hps_base_text: Option<HalfPoint>,
    pub language: Option<String>,
    pub dirty: Option<bool>,
}
```

`CT_R` is reused rather than duplicated, so ruby inherits every run property
F-264 and F-265 delivered without a second content model to keep in step.
`raw_xml` preserves unmodelled `w:rubyPr` children verbatim.

The paragraph content model in `crates/rdocx-oxml/src/text.rs` gains a `Ruby`
variant beside its existing run variant. Layout in
`crates/rdocx-layout/src/engine.rs` measures the base line and the phonetic line
independently, places the phonetic line above the base at the `w:hps` size with
the `w:hpsRaise` offset, and distributes it horizontally per `w:rubyAlign`. The
annotation's contribution to line height is the base height plus the raise, so a
ruby-bearing line is taller and the paginator sees that through the existing
natural-advance path.

**Text extraction returns the base text only.** The phonetic line is an
annotation, not content. This keeps `ActualText`, SVG text, search, redaction
and round-trip XML in logical order as `docs/hld/08-rendering-spec.md` requires,
and it is pinned by its own regression test because a future refactor would
otherwise silently double-count.

### Files the diff touches

- `crates/rdocx-oxml/src/ruby.rs`, new module, approved
- `crates/rdocx-oxml/src/lib.rs`, to declare and re-export it
- `crates/rdocx-oxml/src/text.rs`
- The run property module, for `w:em`
- `crates/rdocx-layout/src/engine.rs`
- `crates/rdocx/src/run.rs`
- `crates/rdocx/src/paragraph.rs`
- `crates/rdocx/src/redaction.rs`
- `crates/rdocx/tests/integration_test.rs`
- `crates/rdocx/tests/regression_test.rs`

One new module, explicitly approved. No new crate. No new file beyond it.

## Rejected alternatives

- **Put `CT_Ruby` in `crates/rdocx-oxml/src/text.rs`.** Rejected in the S74
  round. That file already carries the whole run content model, and the ruby
  grammar is large enough that colocating them defeats the one-file rule.
- **Model `w:ruby` as raw preserved XML with a public accessor.** Rejected. The
  gate is rendered geometry and reading order, and raw XML has no geometry.
- **Give ruby its own run-like content type instead of reusing `CT_R`.**
  Rejected. It would double the number of places a run property change must be
  applied, which is exactly the "increase the places a reader must look"
  failure the structural rules name.
- **Return ruby phonetic text from `text()` alongside the base.** Rejected. It
  double-counts every annotated word for search, redaction, `ActualText` and
  the differential corpus.
- **Render emphasis marks as a font feature rather than placed glyphs.**
  Rejected. The five `ST_Em` values do not map onto an OpenType feature, and the
  bundled subset faces would not carry it if they did.
- **Let emphasis marks affect the base advance.** Rejected. Word places them
  without changing the base line's metrics, and changing the advance would move
  every line containing a mark.
- **Defer emphasis marks to F-266c.** Rejected. Marks and ruby are both East
  Asian annotation placed relative to a base glyph, and they share the
  measurement code.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| golden | `ruby_and_emphasis_page_matches_the_pinned_geometry_and_reading_order` | **The test gate.** The shared deterministic fixture gains one page carrying ruby-annotated Japanese, emphasis-marked Japanese and Korean, and a Latin control, all built through the public facade with `FontManager::new_deterministic`. Its own digest is recorded as a constant. F-266a's digest is asserted unmoved in the same test module, so this story cannot silently disturb its sibling's baseline |
| round-trip | `emphasis_marks_reopen_as_modeled_state` | All five `ST_Em` values author, save, reopen and return typed, land in the correct `EG_RPrBase` sequence position, and leave unrelated producer XML byte for byte |
| round-trip | `a_producer_emphasis_mark_with_unknown_attributes_is_retained_verbatim` | The raw-carrier path keeps an unsupported `w:em` intact rather than normalising it away |
| round-trip | `ruby_authors_saves_and_reopens_with_base_and_phonetic_runs` | `w:ruby` with `w:rubyPr`, `w:rt` and `w:rubyBase` round-trips typed, child order matches `xsd:sequence`, and a prefix-aliased producer `w:ruby` reads correctly and writes with the fixed `w:` prefix |
| round-trip | `unmodelled_ruby_properties_survive_a_noop_save` | An unknown `w:rubyPr` child is preserved verbatim through `capture_element` |
| regression | `ruby_phonetic_text_is_not_returned_by_paragraph_text_extraction` | Base text only. Named as the failure it prevents, so a reintroduction is obvious from the test name |
| regression | `redaction_still_steps_over_ruby_after_it_is_modeled` | The skip list at `crates/rdocx/src/redaction.rs:809` keeps working against the typed model |
| regression | `an_emphasis_mark_does_not_change_the_base_advance` | A marked and an unmarked run produce identical x advances |
| integration | `a_ruby_line_is_taller_and_paginates_on_its_real_height` | The raise and phonetic size reach line height, and a page break falls where the taller line puts it |
| integration | `an_undrawable_emphasis_mark_records_a_diagnostic_and_paints_nothing` | The uncovered-character policy holds for marks |

The **test gate** is the golden
`ruby_and_emphasis_page_matches_the_pinned_geometry_and_reading_order`.

Every rendering assertion uses `FontManager::new_deterministic`. No baseline is
recorded against system fonts. No binary fixture is added. New tests join the
existing `crates/rdocx/tests/integration_test.rs` and `regression_test.rs`
entrypoints, and crate-local units join the existing in-file `#[cfg(test)]`
modules.

## HLD impact

- `docs/hld/08-rendering-spec.md`
- `docs/hld/04-opc-and-packaging.md`
- `docs/hld/12-testing-strategy.md`
- `docs/hld/14-development-backlog.md`

`docs/hld/02-scope-and-non-goals.md` is deliberately absent. DOCX-033 reaches
its final classification in F-266c.

## Risk routing

Four rows of `.claude/skills/risk-routing.md` match.

1. **Layout, pagination, line breaking, text shaping.** Read
   `docs/hld/08-rendering-spec.md`. Extra checks: every baseline uses
   `FontManager::new_deterministic`. Ruby changes line height, so the paginator
   is exercised deliberately rather than incidentally, and F-266a's recorded
   digest is asserted unmoved.

2. **Any parser or serialiser.** Read `docs/hld/04-opc-and-packaging.md` and
   `docs/hld/06-presentationml-model.md`. Extra checks: `w:em`, `w:ruby`,
   `w:rubyPr`, `w:rt` and `w:rubyBase` are prefix-tolerant on read and fixed
   `w:` prefix on write, each lands in its correct `xsd:sequence` position, and
   each carries a round-trip test proving `capture_element` preserved its
   unmodelled subtree byte for byte.

3. **A new trait, generic parameter, crate, module or file.** Read the
   `CLAUDE.md` structural rules. `crates/rdocx-oxml/src/ruby.rs` is a new
   module and was explicitly approved in the S74 consolidated design round,
   which is the ask the rule requires. No new trait, no new generic parameter,
   no `Box<dyn>` and no new crate. `CT_Ruby` reuses `CT_R` rather than
   introducing a second run abstraction.

4. **Public API of a published crate.** Read `docs/hld/10-bindings-spec.md` and
   the `CLAUDE.md` structural rules. Extra checks: `CT_Ruby`, `CT_RubyPr`,
   `ST_RubyAlign` and `ST_Em` are additive to `rdocx-oxml`, and the `CT_RPr`
   field and paragraph-content variant are additive to types that are not
   `#[non_exhaustive]`, so the variant addition is stated as breaking. Run
   `cargo publish --dry-run` for `rdocx-oxml`, `rdocx-layout` and `rdocx`, with
   the archive-size assertion.

Not matched: unit conversion, theme colour tint and shade, bundled fonts, crate
dependency graph, feature flags, file moves, external oracle, release scripting.
The golden is a recorded geometry digest and needs no rasteriser.

## Hash harness

**Expected unchanged. All 49 entries and all 7 golden-PNG entries.**

Every change in this story is absent-by-default. The seven `samples/` documents
generated by `crates/rdocx/examples/generate_all_samples.rs` contain no `w:ruby`
and no `w:em`, and this story adds neither to them. `CT_RPr::emphasis_mark`
defaults to `None` and emits nothing, and the ruby paragraph-content variant is
only constructed where a `w:ruby` element exists in the source or a caller adds
one.

The three code paths a sample could reach are the run property parser, the
paragraph content parser and the layout line-height calculation. In each the
new branch is entered only on the presence of an element no sample carries, so
the sample path is byte-identical.

This story bundles no font, so the coverage-fallback risk that made F-266a's
font commit sensitive does not exist here. No `oxml-layout` change is proposed
at all.

This story does not claim the sprint's exclusive baseline re-record, and it
asserts F-266a's golden digest unmoved rather than re-recording it.

## Implementation checklist

- [x] Confirm F-266a is `done` and that the shared golden fixture module and
      its recorded digest exist.
- [x] Confirm the run-property module split has landed and add
      `CT_RPr::emphasis_mark` there.
- [x] Add `ST_Em`, its parser at the `w:em` schema slot, its serialiser at the
      same position, and the raw-carrier path for unsupported attributes.
- [x] Add the `Run` emphasis setter and reader in `crates/rdocx/src/run.rs`.
- [x] Project emphasis marks into layout, above the glyph box, with the
      diagnostic for an undrawable mark and no change to the base advance.
- [x] Create `crates/rdocx-oxml/src/ruby.rs` with `CT_Ruby`, `CT_RubyPr` and
      `ST_RubyAlign`, reusing `CT_R`, and declare it in `lib.rs`.
- [x] Add the `Ruby` paragraph-content variant in
      `crates/rdocx-oxml/src/text.rs` and the facade authoring surface in
      `crates/rdocx/src/paragraph.rs`.
- [x] Project ruby into layout, base plus phonetic line, with `w:rubyAlign`
      distribution and the corrected line height.
- [x] Make text extraction return the base only, and confirm redaction still
      steps over ruby.
- [x] Add every test in `## Test plan` to the existing entrypoints.
- [x] Record this story's golden digest, with its reason, in the test and in
      `docs/hld/12-testing-strategy.md`, and assert F-266a's digest unmoved.
- [x] Run `/verify`, plus `cargo test -p oxml-layout --no-default-features`,
      `cargo check --target wasm32-unknown-unknown -p rdocx-wasm -p rpptx-wasm`,
      and `cargo publish --dry-run` with the archive-size assertion.

## Open questions

None. Resolved in the S74 consolidated design round.
