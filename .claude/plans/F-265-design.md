# F-265, Complete run property and inline authoring

**Status**: completed
**Sprint**: S74
**Size**: L
**Depends on**: F-260

## Problem

`CT_RPr` models 24 of the 39 `EG_RPrBase` children. The slot table already
enumerates the whole `xsd:sequence`, so the gap is explicit rather than
guessed. `crates/rdocx-oxml/src/properties.rs:348` declares constants for the
modeled slots and `crates/rdocx-oxml/src/properties.rs:2758` maps every
schema name to its ordinal, where slots 10 through 15, 17, 21, 27, 28, 30, 33,
34, 36, 37 and 38 carry a bare numeric literal and no field. Those are
`w:outline`, `w:shadow`, `w:emboss`, `w:imprint`, `w:noProof`, `w:snapToGrid`,
`w:webHidden`, `w:kern`, `w:effect`, `w:bdr`, `w:fitText`, `w:cs`, `w:em`,
`w:eastAsianLayout`, `w:specVanish` and `w:oMath`. They survive a round trip as
positioned raw XML, and they have no reader, no setter and no render
projection.

Three modeled elements lose attributes outright, because the writer rebuilds
them from typed fields and the parser reads only a subset. `w:rFonts` parsing
at `crates/rdocx-oxml/src/properties.rs:1574` accepts six attributes, so
`w:eastAsiaTheme`, `w:cstheme` and `w:hint` are dropped on save by the writer at
`crates/rdocx-oxml/src/properties.rs:2036`. `w:color` parsing at
`crates/rdocx-oxml/src/properties.rs:1679` accepts `w:val` and `w:themeColor`
only, so `w:themeTint` and `w:themeShade` are dropped by the writer at
`crates/rdocx-oxml/src/properties.rs:2087`. `CT_Shd` at
`crates/rdocx-oxml/src/properties.rs:27` holds `val`, `color` and `fill`, so
every `w:themeColor`, `w:themeFill`, `w:themeFillTint` and `w:themeFillShade`
on run shading is dropped.

Explicit-font replacement is silently ineffective against a theme font.
`Run::set_font_value` at `crates/rdocx/src/run.rs:558` writes all four explicit
slots and leaves `font_ascii_theme` and `font_hansi_theme` in place, and
`resolve_font_family` at `crates/rdocx-layout/src/engine.rs:7835` then prefers
the explicit name while Word prefers the theme attribute. The same shape exists
for colour, where `Run::set_color_value` at `crates/rdocx/src/run.rs:600` leaves
`color_theme` in place. `Run::set_color_value` also has no theme counterpart, so
a theme reference can be read but never authored.

Ordered inline content stops short of the special characters. `RunContent` at
`crates/rdocx-oxml/src/text.rs:681` has nine variants and none of them is
`w:sym`, `w:cr`, `w:noBreakHyphen`, `w:softHyphen` or `w:ptab`. The run parser
at `crates/rdocx-oxml/src/text.rs:1444` sends each of those to positioned raw
capture, so they reopen in order and surface publicly only as
`RunItemRef::UnsupportedXml`. `Run::add_symbol` at `crates/rdocx/src/run.rs:426`
appends a Unicode scalar as ordinary `w:t` text and never produces `w:sym`.
Other parts of the workspace already name these elements, for example the run
child allowlist at `crates/rdocx/src/document.rs:4890` and the redaction
visibility set at `crates/rdocx/src/redaction.rs:779`, which is the strongest
evidence that the grammar is the only place they are missing.

`docs/hld/02-scope-and-non-goals.md:239` records this as DOCX-032, partial, with
Create, Mutate, Remove, Save-reopen, Layout, Render, Determinism and Native all
at `P`, owned by F-265.

## Starting base

Before wave 1 claims, `crates/rdocx-oxml/src/properties.rs` is split into
separate paragraph-property and run-property modules as its own byte-identical
labelled sprint commit, with re-exports preserving every existing public path.
**F-265 does not perform that move. It starts from the moved base.** Every
`CT_RPr`, `CT_Shd` and run-slot change in this plan lands in the run property
module. The `properties.rs` line citations above are pre-split coordinates that
describe today's state, and the symbol names are the durable reference.

The move is a pure file move with no behaviour change, so the hash harness must
be byte-identical across it and the move commit owns that evidence. F-265 must
not fold any behaviour change into it, and must not be blamed for a delta the
move introduced.

## Spec reference

- `docs/hld/02-scope-and-non-goals.md`, "Modern DOCX capability matrix", row
  DOCX-032 and the classification legend that defines `Y`, `P`, `PV` and `B`.
- `docs/hld/03-architecture.md`, "What stays put", for `rdocx-oxml` owning the
  WordprocessingML property grammar, expanded names and schema positions
  deciding typed meaning, and the facade computing effective run properties
  without a second reader model.
- `docs/hld/04-opc-and-packaging.md`, "What transfers unmodified", for
  positioned verbatim retention of everything outside the typed projection.
- `docs/hld/05-drawingml-model.md`, "Do not touch the Word path", for
  `rdocx_oxml::theme::apply_tint_shade` keeping its 0-255 convention, its naive
  sRGB interpolation and its name, and for the spec-correct `oxml-drawing`
  entry points staying separate.
- `docs/hld/08-rendering-spec.md`, "Text in a shape", for effective run
  resolution, the `cap` transformation applied before script-specific font
  selection, and the Wingdings trap that symbol-font mapping reuses.
- `docs/hld/08-rendering-spec.md`, "Word revision views", for run-level strike
  and visibility projection that the new toggles must not disturb.
- `docs/hld/10-bindings-spec.md`, "Native Word facade stability", including the
  paragraph on the mutable `Run` handle that currently states `add_symbol`
  stores one Unicode scalar as text and that Python, WASM and CLI gain no
  implicit surface.
- `docs/hld/12-testing-strategy.md`, "Test taxonomy", for the differential
  category, and "The Word render fidelity gate" for the pinned tool identities,
  the 150 dpi rasterisation and the source-built multi-script fixture pattern.

## Approach

### 1. Complete the `EG_RPrBase` sequence in `CT_RPr`

Name the 16 remaining slots as constants beside the existing ones in the run
property module, then add one field each and the matching parse arm, write arm,
`is_empty` clause and `merge_from` clause. The writer emits each new child at
its ordinal, between the neighbours already written, so the `xsd:sequence` stays
correct without a second ordering mechanism.

```rust
// the run property module, added to CT_RPr
pub outline: Option<bool>,          // slot 10
pub shadow: Option<bool>,           // slot 11
pub emboss: Option<bool>,           // slot 12
pub imprint: Option<bool>,          // slot 13
pub no_proof: Option<bool>,         // slot 14
pub snap_to_grid: Option<bool>,     // slot 15
pub web_hidden: Option<bool>,       // slot 17
pub kern: Option<HalfPoint>,        // slot 21
pub effect: Option<ST_TextEffect>,  // slot 27
pub border: Option<CT_Border>,      // slot 28
pub fit_text: Option<CT_FitText>,   // slot 30
pub complex_script: Option<bool>,   // slot 33
pub emphasis_mark: Option<ST_Em>,   // slot 34
pub east_asian_layout: Option<CT_EastAsianLayout>, // slot 36
pub spec_vanish: Option<bool>,      // slot 37
pub office_math: Option<bool>,      // slot 38
```

`CT_Border` already exists for paragraph and table borders and is reused rather
than duplicated. `CT_FitText` carries `val: Twips` and `id: Option<i32>`.
`CT_EastAsianLayout` carries `id`, `combine`, `combine_brackets`, `vert` and
`vert_compress`. `ST_TextEffect` and `ST_Em` are closed token enums with an
`Other(String)` arm, matching how `ST_NumberFormat` already retains
producer-defined tokens.

The eleven toggles reuse the existing `parse_word_toggle`, `write_toggle` and
modeled-attribute-carrier machinery, so an explicitly empty toggle and a toggle
carrying foreign attributes behave exactly as `w:b` and `w:vanish` do today.

### 2. Complete the attributes on the three lossy elements

F-265 owns the whole theme-attribute sweep across `w:rFonts`, `w:color` and
`w:shd`, because those three losses are one defect with one shape and splitting
them across stories would leave the sweep half done.

`CT_RPr` gains `font_east_asia_theme`, `font_cs_theme` and `font_hint`, plus
`font_extra_attributes: Vec<(String, String)>` and
`color_extra_attributes: Vec<(String, String)>`, mirroring the existing
`language_extra_attributes` at `crates/rdocx-oxml/src/properties.rs:1524`. It
also gains `color_theme_tint: Option<u8>` and `color_theme_shade: Option<u8>`,
parsed from the two-hex-digit Word convention.

**F-265 owns `w:rFonts/@w:hint`**, both the typed field and the public setter,
because F-265 owns the whole `w:rFonts` element. F-266a consumes it for
script-slot resolution and does not add it.

`CT_Shd` gains `theme_color`, `theme_tint`, `theme_shade`, `theme_fill`,
`theme_fill_tint`, `theme_fill_shade` and an ordered extra-attribute vector.
**F-265 owns the `CT_Shd` theme attributes.** `CT_Shd` is shared with paragraph
and table shading, and F-264 lands before F-265 and either consumes this change
or avoids the theme attributes entirely. There is no race and no first-to-land
rule.

### 3. The theme-clearing policy

`w:rFonts` has four script slots, and each slot has an explicit attribute and a
theme attribute. Word's own UI writes the explicit attribute and removes the
theme attribute for the slot the user changed. The policy is:

1. Setting the explicit font for a slot clears that slot's theme attribute and
   touches no other slot.
2. Setting the theme font for a slot clears that slot's explicit attribute.
3. Clearing a slot with `None` clears both attributes for that slot.
4. `w:hint` is independent and no font operation clears it.
5. Reading never normalises. A parsed `w:rFonts` carrying both an explicit and a
   theme attribute for one slot is retained exactly until a caller sets that
   slot.
6. The existing `Run::set_font` and `Run::set_font_value` keep their documented
   four-slot behaviour and therefore now clear all four theme attributes. This
   is a behaviour change and it is a correction, so it ships without a
   deprecation. Today the call produces a `w:rFonts` where Word still resolves
   the theme font, so the caller's explicit font silently does nothing. The
   corrected behaviour is stated in the setter's own doc comment.

Colour follows the same shape. `Run::set_color_value(Some(hex))` clears
`themeColor`, `themeTint` and `themeShade`, because it is an explicit
replacement and because matching the font policy keeps one rule rather than two.
A new `Run::set_color_theme` serves the caller who wants the reference. It sets
the theme triple and leaves `w:val` as the cached literal Word writes alongside
it, which reproduces Word's file shape rather than inventing a resolved value
the run has no theme to compute.

```rust
// crates/rdocx/src/run.rs
pub enum RunFontSlot { Ascii, HighAnsi, EastAsia, ComplexScript }

impl Run<'_> {
    pub fn set_slot_font(&mut self, slot: RunFontSlot, name: Option<&str>);
    pub fn set_slot_theme_font(&mut self, slot: RunFontSlot, theme: Option<&str>);
    pub fn set_font_hint(&mut self, hint: Option<&str>);
    pub fn set_color_theme(&mut self, theme: Option<&str>, tint: Option<u8>, shade: Option<u8>);
}
```

### 4. Named deliverables that F-266a depends on

F-266a cannot build its golden fixture through public APIs without these, so
they are contract, not incidental scope. Each is a public setter and reader on
the mutable `Run` handle and on `RunRef`, over fields that mostly exist already:

- The complete `w:rFonts` slot set, including the `eastAsia` and `cs` explicit
  slots and their theme counterparts, through `set_slot_font` and
  `set_slot_theme_font`.
- `w:rFonts/@w:hint`, through `set_font_hint`.
- `w:rtl`, over the existing `CT_RPr::rtl` field, which has no public setter
  today.
- `w:cs`, the new slot 33 complex-script toggle.
- The `w:lang` East Asian and bidirectional setters over the existing
  `CT_RPr::language_east_asia` and `CT_RPr::language_bidi` fields, which are
  parsed and written today but reachable only through `set_language` for the
  Latin slot.
- The complex-script pairs `w:bCs`, `w:iCs` and `w:szCs`, over the existing
  `bold_cs`, `italic_cs` and `sz_cs` fields, which likewise have no public
  setter.

### 5. Symbols and special characters as ordered inline content

Two new `RunContent` variants, not six. `RunContent` is matched in ten files and
137 times in `crates/rdocx-layout/src/engine.rs` alone, so grouping the special
characters behind one enum reduces the cases a reader must consider at every
one of those sites.

```rust
// crates/rdocx-oxml/src/text.rs
pub enum RunContent {
    // ... existing nine variants ...
    Symbol { font: String, char_code: u16 },
    SpecialCharacter(SpecialCharacter),
}

pub enum SpecialCharacter {
    CarriageReturn,
    NoBreakHyphen,
    SoftHyphen,
    PositionalTab { alignment: ST_PTabAlignment, relative_to: ST_PTabRelativeTo, leader: ST_PTabLeader },
}
```

`w:lastRenderedPageBreak` stays in positioned raw capture and gains a read-only
classification through the existing `classify_raw_run_item` path at
`crates/rdocx/src/run.rs:296`, exactly as the legacy VML horizontal rule does.
It is a producer hint, so it needs no authoring surface and no `RunContent`
variant.

`Run::add_symbol` keeps its current documented meaning of one Unicode scalar as
text, because changing it would break a shipped F-260 API. A new
`Run::add_symbol_char(font: &str, char_code: u16)` produces `w:sym`, and
`Run::add_special_character(SpecialCharacter)` produces the other four.

### 6. Effective properties and rendering

`crates/rdocx-layout/src/engine.rs:5926` already merges style-resolved and
direct run properties into one `effective_rpr`. The new properties join that
projection with these render rules:

- `w:outline` paints stroke-only glyphs, `w:shadow`, `w:emboss` and `w:imprint`
  paint an offset or inset copy behind the glyph run.
- `w:bdr` paints a character border box around the contiguous run fragment.
- `w:kern` gates pair kerning below the stated half-point size.
- `w:fitText` scales the segment horizontally to the stated twip width.
- `w:sym` maps the font and code point through the existing Wingdings trap
  before font resolution, so a private-use bullet reaches a visible glyph.
- `w:noBreakHyphen` renders a hyphen that is not a break opportunity,
  `w:softHyphen` renders only when the line breaks at it, `w:cr` breaks the
  line, `w:ptab` places the following content at its absolute position.
- `w:effect`, `w:noProof`, `w:webHidden`, `w:specVanish` and `w:oMath` are
  modeled, authored and round-tripped with no visible render projection, which
  matches Word's own print behaviour. Each carries a test asserting the absence
  of a visual change, so the classification cannot rot into an oversight.
- `w:em`, `w:eastAsianLayout`, `w:snapToGrid` and `w:cs` are modeled, authored
  and round-tripped here, and F-265 does not render them.
  `docs/hld/02-scope-and-non-goals.md:240` owns their rendering under DOCX-033.
  `w:em` rendering belongs to F-266b, ruby and emphasis marks.
  `w:eastAsianLayout`, `w:snapToGrid` and `w:cs` rendering belongs to F-266c,
  character grid and vertical text.

`resolve_run_color` at `crates/rdocx-layout/src/engine.rs:7866` gains a tint and
shade step over the resolved theme colour, calling
`rdocx_oxml::theme::apply_tint_shade` unchanged, with its existing 0-255
signature that matches Word's `w:themeTint` and `w:themeShade` byte convention
exactly. That function has no production caller today, so F-265 makes it live
for the first time. **Making it live must not change its arithmetic.** Its maths
is not corrected, not renamed and not replaced by the `oxml-drawing` entry
points, per `docs/hld/05-drawingml-model.md`.

`resolve_font_family` at `crates/rdocx-layout/src/engine.rs:7835` keeps its
current explicit-before-theme priority, unchanged by this story. Word prefers
the theme attribute when a producer presents both for one slot, so this is a
**deliberate divergence recorded under rule 5 of
`.claude/skills/differential-testing.md`**, and the differential run is expected
to report it. The reason is that the theme-clearing policy means a document
authored through this facade never presents both for one slot, so the divergence
is reachable only on a producer document the caller never edited. On that
document the two defensible readings are to reinterpret the producer's bytes at
render time or to leave them as written, and leaving them as written is
consistent with the no-op save contract F-X128 established. The differential
test asserts our side so a later change cannot silently drop the decision.

## Rejected alternatives

- Correct `apply_tint_shade` while adding its first caller. Forbidden by
  `CLAUDE.md` and `docs/hld/05-drawingml-model.md`, and it would move every
  future theme-derived colour in a way indistinguishable from a port defect.
- Route Word `w:themeTint` through `oxml_drawing::color::apply_tint_shade_pct`.
  The Word attribute is a 0-255 byte and the DrawingML function takes
  `Percent1000` in linear gamma. Converting between them is the correction under
  a different name.
- Model and round-trip `w:themeTint` and `w:themeShade` but leave the render
  projection to a later story. They are dropped on save today, which is data
  loss, and a modeled attribute with no effect invites a second story to
  rediscover the same question.
- Change `resolve_font_family` to Word's theme-before-explicit priority in this
  story. That is a render behaviour change with a hash-harness consequence,
  landing inside a story whose harness expectation is unchanged.
- One `RunContent` variant per special character. Six variants across ten files
  and more than 200 match sites increases the places a reader must look with no
  gain, since all four share one placement contract.
- Change `Run::add_symbol` to emit `w:sym`. It is a shipped F-260 API documented
  in `docs/hld/10-bindings-spec.md` as storing a Unicode scalar as text.
- Model `w:lastRenderedPageBreak` as typed content. It is a producer hint that
  must never be authored, and the existing raw classification path already gives
  it a read projection at no structural cost.
- Keep the 16 missing children as raw XML and add readers only. Raw capture
  cannot be mutated, removed or rendered, so DOCX-032 would stay at `P` for
  Create, Mutate, Remove, Layout and Render.
- Normalise a producer `w:rFonts` that carries both an explicit and a theme
  attribute at parse time. That discards producer intent on a document the
  caller never edited, and it breaks the no-op save contract F-X128 established.
- Add Python, WASM or CLI surface. `docs/hld/02-scope-and-non-goals.md:239`
  classifies all three as `B` for DOCX-032.
- Split the theme-attribute sweep so F-264 takes `w:shd` and F-265 takes
  `w:rFonts` and `w:color`. One defect with one shape belongs in one diff.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| differential | `complete_run_formatting_and_inline_order_match_the_pinned_word_reference` | Every modeled run property and inline item read from the pinned Word corpus produces the same effective run formatting and the same inline order as the oracle, after save, reopen and 150 dpi render. |
| differential | python-docx `Font` property matrix | The 26 `Font` attributes python-docx 1.2.0 exposes, including `outline`, `shadow`, `emboss`, `imprint`, `no_proof`, `snap_to_grid`, `web_hidden`, `spec_vanish`, `math` and `complex_script`, agree with our readers over the parsed tree. Each disagreement is triaged into our defect, oracle out of scope, or a decided ECMA reading, per `.claude/skills/differential-testing.md` rule 5. |
| differential | `explicit_font_priority_divergence_from_word_is_deliberate` | On a producer `w:rFonts` carrying both an explicit and a theme attribute for one slot, our resolved family is the explicit one, and the assertion names the divergence so a later change cannot drop it silently. |
| round-trip | every new `w:rPr` child and attribute survives a no-op save | All 16 new children, the three `w:rFonts` theme attributes, `w:hint`, `w:themeTint`, `w:themeShade` and the six `w:shd` theme attributes reopen with identical values, in schema order, alongside untouched positioned raw siblings. |
| round-trip | ordered special characters and symbols reopen in source order | `w:sym`, `w:cr`, `w:noBreakHyphen`, `w:softHyphen`, `w:ptab` and `w:lastRenderedPageBreak` interleaved with text, tabs, breaks, drawings and fields reopen in exact source order. |
| round-trip | `f266a_prerequisite_run_properties_author_and_reopen` | The complete `w:rFonts` slot set, `w:hint`, `w:rtl`, `w:cs`, the `w:lang` East Asian and bidirectional slots, and `w:bCs`, `w:iCs` and `w:szCs` are all publicly authored, read back and reopened without raw XML. This is the F-266a contract. |
| regression | `explicit_font_replacement_clears_only_its_own_theme_slot` | Setting an explicit font for one slot clears that slot's theme attribute, leaves the other three slots and `w:hint` untouched, and leaves a producer document unedited. |
| regression | `explicit_colour_replacement_clears_theme_colour_tint_and_shade` | Setting an explicit colour clears the theme triple, and `set_color_theme` leaves the cached `w:val` in place. |
| unit | prefix-tolerant read, fixed-prefix write, alias and foreign attributes | Each new element and attribute parses under an aliased Word prefix and writes as `w:`, and foreign attributes survive through the extra-attribute vectors. |
| unit | `is_empty` and `merge_from` cover every new field | A `w:rPr` carrying only a new field is not treated as empty, and style inheritance merges every new field. |
| unit | `apply_tint_shade_arithmetic_is_unchanged` | The pinned values in `crates/rdocx-oxml/src/theme.rs` still hold after F-265 makes the function live. |
| golden | deterministic render of outline, shadow, emboss, imprint, border, kern, fitText and sym | Deterministic font mode, pinned geometry, one source-built fixture page. |
| regression | non-rendering properties change no pixels | `w:effect`, `w:noProof`, `w:webHidden`, `w:specVanish` and `w:oMath` produce a byte-identical render against the same document without them. |

The **test gate** is the differential test named in the backlog,
`complete_run_formatting_and_inline_order_match_the_pinned_word_reference`.

### Evidence policy and pinned oracles

The gate passes on evidence reproducible on this machine. Pins are recorded in
the harness and not in a comment:

- python-docx 1.2.0, the structural object-model oracle, already pinned at
  `crates/rdocx-py/tests/test_python_docx_parity.py:5` and
  `.github/workflows/ci.yml:220`.
- LibreOffice Writer 26.2.5.2 build
  `cd7284b4cbbfeb507e630c1aac019f4157393acb` and pdftoppm 26.01.0 at 150 dpi,
  the raster path, already installed and already pinned at
  `scripts/docx_ssim_harness.py:39`.

**Word GUI capture is not available on this machine and is not a blocker.** The
Microsoft Word no-repair confirmation is recorded as a tracked human action
rather than an automated step, with its backlog home in
`docs/hld/14-development-backlog.md` under the Milestone 24 end-of-milestone
gate, which is where the existing "without Word repair" requirement already
lives. The test named as the gate does not depend on it and does not skip
without it.

The render side adds one new source-built one-page fixture in the
`scripts/docx_ssim_harness.py` `MULTI_SCRIPT_FIXTURES` pattern, because the
existing corpus documents exercise no outline, emboss, `w:bdr`, `w:fitText` or
`w:sym`. No binary fixture is added.

Every Rust test is a module added to the existing
`crates/rdocx/tests/integration_test.rs` and
`crates/rdocx/tests/regression_test.rs` entrypoints. No new file under any
`tests/` directory.

## HLD impact

- `docs/hld/02-scope-and-non-goals.md`
- `docs/hld/03-architecture.md`
- `docs/hld/04-opc-and-packaging.md`
- `docs/hld/05-drawingml-model.md`
- `docs/hld/08-rendering-spec.md`
- `docs/hld/10-bindings-spec.md`
- `docs/hld/12-testing-strategy.md`
- `docs/hld/14-development-backlog.md`

## Risk routing

Matched rows of `.claude/skills/risk-routing.md` and the exact extra check each
one adds.

- **Theme colour, tint, shade, colour mapping.** Read
  `docs/hld/05-drawingml-model.md`, "Do not touch the Word path".
  **The review's acceptance check**: `crates/rdocx-oxml/src/theme.rs` shows a
  zero-line diff, `apply_tint_shade` keeps its name, its 0-255 signature and its
  naive sRGB arithmetic, its pinned unit values still hold, and no call site on
  the Word path reaches `oxml_drawing::color::apply_tint_shade_pct` or any other
  spec-correct DrawingML entry point. F-265 makes the function live without
  changing what it computes.
- **Any parser or serialiser.** Read `docs/hld/04-opc-and-packaging.md`. The 16
  new children write at their `xsd:sequence` ordinals, interleaved with
  positioned raw siblings. Prefix-tolerant on read with an aliased Word prefix,
  fixed `w:` prefix on write. A round-trip test proves `capture_element`
  preserved every unmodelled subtree byte for byte, including a run whose only
  `w:rPr` child is one of the newly modeled elements.
- **Layout, pagination, line breaking, text shaping.** Read
  `docs/hld/08-rendering-spec.md`. Deterministic font mode for every baseline.
  The new golden page is recorded deliberately as new coverage, never as a
  re-record of an existing baseline.
- **Public API of a published crate.** Read `docs/hld/10-bindings-spec.md`,
  "Native Word facade stability", and the `CLAUDE.md` structural rules.
  `rdocx::RunProperties` is a re-export of `CT_RPr` at
  `crates/rdocx/src/run.rs:52`, so every added field is public surface. State
  the pre-1.0 minor semver impact, confirm the run property module split kept
  every existing public path through its re-exports, run
  `cargo publish --dry-run` and the `.crate` size assertion, and add no surface
  the story did not ask for.
- **An external oracle comparison.** Read
  `.claude/skills/differential-testing.md`. Pin and record python-docx 1.2.0,
  LibreOffice Writer 26.2.5.2 build
  `cd7284b4cbbfeb507e630c1aac019f4157393acb` and pdftoppm 26.01.0, with the
  150 dpi rasterisation and the stated SSIM threshold. Triage every
  disagreement into one of the three verdicts and assert the chosen side. The
  explicit-before-theme font priority is pre-declared as a deliberate
  divergence, so the reviewer sees a decision rather than a surprise.

Not matched by this story: **a file move or rename with no behaviour change**.
The run property module split is its own labelled sprint commit that lands
before wave 1 claims, and it owns the byte-identical harness evidence. F-265
must not fold any behaviour change into it.

Not matched: **a new trait, generic parameter, crate, module or file**. No
trait, no generic parameter, no new crate and no new file in this story's diff.
The new types are plain enums and structs in the existing run property module.

Not matched: **WASM or PyO3 bindings**, since DOCX-032 keeps `B` for Python,
WASM and CLI, and no binding gains a method. The workspace test exclusions and
the wasm target check remain part of the `/verify` floor regardless.

## Hash harness

**Expected unchanged.**

Justification. The seven harness samples are generated by
`crates/rdocx/examples/generate_all_samples.rs`, which sets fonts only as
explicit families such as `.font("Arial")` and `.font("Georgia")` and references
no theme font, no theme colour, no tint or shade, no symbol and no special
character. The theme-clearing policy therefore clears attributes that are
already absent, so it is a no-op for every sample. Every new `w:rPr` child
serialises only when the field is `Some`, and every new attribute only when it
was parsed or authored, so no sample gains a byte. `resolve_run_color` gains a
tint and shade step that is skipped when both are `None`, which is the case for
all seven samples, and `resolve_font_family` keeps its current priority
unchanged.

The run property module split is expected to be byte-identical under its own
labelled commit. If the harness has already moved when F-265 starts, the delta
belongs to the move and not to this story.

An unexplained delta blocks the merge. If a delta appears it must name the
responsible child or attribute and the sample that gained it, and it lands as
its own labelled commit rather than folded into this story's main commit.

## Implementation checklist

- [x] Confirm the run property module split has landed and the harness is
      byte-identical across it before starting.
- [x] Add the failing differential, round-trip and ordering tests first.
- [x] Name the 16 remaining `EG_RPrBase` slot constants in the run property
      module beside the existing ones.
- [x] Add the 16 fields with parse, write, `is_empty` and `merge_from` clauses.
- [x] Add the `w:rFonts`, `w:color` and `w:shd` theme attributes, `w:hint` and
      the ordered extra-attribute vectors, as one sweep.
- [x] Deliver the F-266a prerequisites explicitly: the complete `w:rFonts` slot
      set including `eastAsia` and `cs`, `w:hint`, `w:rtl`, `w:cs`, the `w:lang`
      East Asian and bidirectional setters, and `w:bCs`, `w:iCs` and `w:szCs`.
- [x] Add `RunContent::Symbol` and `RunContent::SpecialCharacter`, with the
      parser, writer and every match site across the ten consuming files.
- [x] Classify `w:lastRenderedPageBreak` through the existing raw path.
- [x] Add the facade setters, readers and `RunItemRef` variants, native only.
- [x] Implement the theme-clearing policy, state the corrected `set_font`
      behaviour in its doc comment, and add the regression tests.
- [x] Add the effective-property projection, the inline-content render rules
      for `w:sym`, `w:cr`, `w:noBreakHyphen`, `w:softHyphen` and `w:ptab`, and
      the tint and shade step, with the non-rendering set asserted as producing
      no pixel change. See "Deviations" for the visual effects.
- [x] Call `apply_tint_shade` unchanged, prove `theme.rs` has a zero-line diff,
      and assert its pinned arithmetic still holds.
- [x] Assert the explicit-before-theme divergence. See "Deviations" for the
      SSIM fixture.
- [x] Record the Word no-repair confirmation as a tracked human action in
      `docs/hld/14-development-backlog.md` under the Milestone 24 gate.
- [x] Run the oxml, layout, facade, round-trip, differential, hash harness,
      full verification and microscope gates.

## Deviations

Two checklist items shipped in reduced form. Both reductions are recorded in
the capability matrix and in the spec set rather than only here, so nothing
claims a completeness the diff does not deliver.

1. **The visual-effect render rules are not built.** `w:outline`, `w:shadow`,
   `w:emboss`, `w:imprint`, `w:bdr`, `w:kern` and `w:fitText` are modeled,
   authored, mutated, removed and round-tripped, and they have no render
   projection. Stroke-only glyph painting, offset and inset relief, a character
   border box, pair-kerning gating below a half-point threshold and horizontal
   segment scaling each need new `oxml-layout` segment state and new PDF
   backend work, which is a second story's worth of change in a
   format-neutral crate. `DOCX-032` therefore keeps its `partial`
   classification with `F-265` as its live owner, its `Layout`, `Render` and
   `Determinism` columns stay `P`, and its `Create`, `Read`, `Mutate`,
   `Remove`, `Save-reopen` and `Native` columns move to `Y`. The boundary is
   named on the matrix row, in `docs/hld/08-rendering-spec.md` and in the
   `F-265` backlog entry. This follows the `DOCX-030` and `F-264` precedent,
   where positioned frame placement stayed with its owner for the same reason.

2. **No new SSIM fixture was added.** The existing
   `scripts/docx_ssim_harness.py` `MULTI_SCRIPT_FIXTURES` set exercises the
   multi-script render path, and a fixture for stroke, relief, border, kerning
   and fitted text would record a baseline for a render projection that does
   not exist yet. It belongs with the render work in deviation 1. The
   explicit-before-theme divergence is asserted instead by
   `explicit_font_priority_divergence_from_word_is_deliberate`, which compares
   three deterministic 150 dpi renders through the public facade.

Two further points that the plan did not anticipate.

3. **The stack budget was paid for by boxing three existing members.** Adding
   16 members plus nine attributes to `CT_RPr` and six to `CT_Shd` would have
   grown both past the 2 MiB test-thread ceiling that F-264 hit.
   `CT_RPr::shading`, `CT_RPr::change` and `CT_PPr::shading` are now boxed
   alongside the new `border`, `fit_text` and `east_asian_layout` members.
   `size_of::<CT_RPr>()` is 696 before and after, `size_of::<CT_PPr>()` falls
   from 2144 to 2080 and `size_of::<Document>()` falls from 27040 to 26848. No
   stack limit was raised.

4. **One serialization normalisation was introduced.** `ST_UcharHexNumber`
   values are typed as `u8` so `apply_tint_shade` can take them, and they are
   written back as the two upper-case hex digits Word writes. A value that is
   not two hex digits is retained verbatim through the element's ordered
   attribute vector instead of being parsed. This is stated in
   `docs/hld/04-opc-and-packaging.md`.

## Open questions

None. Resolved in the S74 consolidated design round.
