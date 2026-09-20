# F-266a, Script identity and font slot resolution

**Status**: completed
**Sprint**: S74
**Size**: L
**Depends on**: F-264, F-265

## Problem

The shaping stack under `oxml-layout` is real and complete. HarfRust receives
explicit script, language and direction at `crates/oxml-layout/src/font.rs:1618`
to `font.rs:1631`, and `unicode-bidi` runs a full UAX 9 pass paragraph-wide at
`font.rs:1395` and per fitted line at `crates/oxml-layout/src/line.rs:710` and
`line.rs:737`. Two layers above it are wrong in ways that make Korean and
Japanese text unrenderable and East Asian font authoring inert.

**Script identity stops short of Korean and Japanese.** `script_for_char` at
`crates/oxml-layout/src/font.rs:1763` maps Latin, Hebrew, Arabic, Devanagari,
Thai and Han. Hangul at `U+1100..U+11FF`, `U+3130..U+318F` and `U+AC00..U+D7AF`
and Kana at `U+3040..U+30FF` fall through to `TextScript::Common` and reach the
shaper as `harfrust::script::COMMON` at `font.rs:1783`. A shaper told the script
is `COMMON` applies no Hangul or Kana feature set.
`crates/rdocx-layout/src/engine.rs:7893` `word_language_slot` omits Hangul
too, so Korean text takes the direct `w:lang/@w:val` value where Word takes
`w:lang/@w:eastAsia`.

**The `w:rFonts` script slots are parsed and then ignored.**
`resolve_font_family` at `crates/rdocx-layout/src/engine.rs:7835` consults
`font_ascii` and the ASCII theme reference and nothing else.
`CT_RPr::font_east_asia` and `CT_RPr::font_cs`, modeled at
`crates/rdocx-oxml/src/properties.rs:1467` and `properties.rs:1469`, appear in
`engine.rs:3723` and `engine.rs:4748` only inside memory accounting. There is no
`WordFontSlot` counterpart to the `WordLanguageSlot` that already exists one
function away at `engine.rs:7886`. The same function collapses `majorEastAsia`,
`minorEastAsia`, `majorBidi` and `minorBidi` onto the **ASCII** theme font at
`engine.rs:7847`, which is a latent defect independent of this story.

**No bundled face can draw the scripts the gate names.**
`crates/oxml-layout/src/bundled_fonts.rs:106` to `bundled_fonts.rs:126` bundles
four non-Latin faces. Arabic is complete at 828 KB. Hebrew has no coverage
anywhere in the workspace. Hangul has no coverage anywhere. Japanese has none
either, because `NotoSansSC-FX058-subset.ttf` is an 88 KB subset whose entire
approved repertoire is ASCII space, comma, digits, Latin letters, `、〈〉` and
five Han characters, recorded in `crates/oxml-layout/fonts/SUBSET-NotoSansSC.md`.
It carries no Hiragana, no Katakana and no further Kanji.
`resolve_font_for_text` keeps the requested font when nothing can draw the text
(`crates/oxml-layout/src/font.rs:2991`), so a mixed page would render notdef
boxes and pin a meaningless baseline.

## Spec reference

- `docs/hld/08-rendering-spec.md`, "The renderer's input", the paragraph
  beginning "Complex text enters shared layout as one paragraph-wide logical
  sequence" and the paragraph beginning "Word `w:bidi` selects the paragraph
  base direction", for script and coverage segmentation, explicit shaper inputs
  and logical order retention.
- `docs/hld/02-scope-and-non-goals.md`, "Modern DOCX capability matrix", the
  DOCX-033 row and the legend paragraph beginning "This is the closed authoring
  contract for M23 and M24".
- `docs/hld/12-testing-strategy.md`, "The golden-PNG gate" and the two
  paragraphs after it describing the in-code deterministic two-view and
  cross-family goldens, for the recorded-digest pattern with no committed
  fixture.
- `docs/hld/15-build-and-toolchain.md`, the deterministic `FontManager`
  constructor section beginning "Normal `oxml-layout` construction may load
  system fonts", and the package inventory paragraph beginning "The dedicated
  package CI job compares `cargo package -p oxml-layout --list` against all 24
  TTFs".
- `docs/hld/14-development-backlog.md`, "F-266, International and vertical
  typography (L)".

## Prerequisites delivered by earlier stories

F-264 and F-265 both complete before this story starts, enforced by the sprint's
dependency-prefix checkpoint. This story designs none of their surface and
consumes exactly this much of it:

| From | Consumed here |
|---|---|
| F-264 | The paragraph setter and reader for `w:bidi`, writing `CT_PPr::bidi` |
| F-265 | Run setters and readers for the complete `w:rFonts` slot set, including `eastAsia` and `cs` |
| F-265 | `w:rFonts/@w:hint`, both the typed `CT_RPr` field and its setter. F-265 owns the whole `w:rFonts` element, so it owns the hint |
| F-265 | Run setters for `w:rtl` and `w:cs` |
| F-265 | Run setters for `w:lang/@w:eastAsia` and `w:lang/@w:bidi`, over the already-modeled `CT_RPr::language_east_asia` and `language_bidi` |
| F-265 | The complex-script pairs `w:bCs`, `w:iCs` and `w:szCs`, already modeled at `properties.rs:1477`, `1481` and `1491` |

`crates/rdocx-oxml/src/properties.rs` is split into separate paragraph-property
and run-property modules before wave 1, as its own byte-identical labelled
commit with re-exports preserving every public path. This story reads
`CT_RPr` from **the run property module** and adds no field to it.

## Approach

### A. Script identity for Korean and Japanese

In `crates/oxml-layout/src/font.rs`:

```rust
/// Script identity used to select shaping behavior and deterministic fallback.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum TextScript {
    Latin, Arabic, Hebrew, Devanagari, Thai, Han,
    Hangul,
    Kana,
    Common,
}
```

`#[non_exhaustive]` lands in the same commit as the two variants, so the
downstream break is taken once rather than on every future script. The
workspace is pre-1.0 at an unreleased 0.14.0, which is the cheapest moment to
take it.

`script_for_char` at `font.rs:1763` gains
`0x1100..=0x11ff | 0x3130..=0x318f | 0xac00..=0xd7af` for `Hangul` and
`0x3040..=0x30ff | 0x31f0..=0x31ff` for `Kana`. No existing range moves, so no
character already classified changes script. `harfrust_script` at `font.rs:1775`
maps the two new variants to `harfrust::script::HANGUL` and
`harfrust::script::HIRAGANA`.

In `crates/rdocx-layout/src/engine.rs`, `word_language_slot` at `engine.rs:7893`
gains the same Hangul ranges under `WordLanguageSlot::EastAsia`, which is where
Word puts them. The Kana range is already covered by the existing
`0x3000..=0x30ff` arm.

### B. `w:rFonts` script-slot resolution

A `WordFontSlot` enum in `crates/rdocx-layout/src/engine.rs`, deliberately
mirroring the shape of the `WordLanguageSlot` that already sits beside it, so a
reader meets one pattern rather than two:

```rust
#[derive(Clone, Copy, PartialEq, Eq)]
enum WordFontSlot { Ascii, HighAnsi, EastAsia, ComplexScript }

fn word_font_slot(character: char, hint: Option<&str>) -> WordFontSlot;
fn word_font_for_slot(rpr: &CT_RPr, slot: WordFontSlot) -> Option<String>;
```

`word_font_slot` classifies by codepoint, with `w:rFonts/@w:hint` breaking the
tie only for characters whose slot is genuinely ambiguous, which is the
`Common` range. `word_font_for_slot` reads `font_ascii`, `font_hansi`,
`font_east_asia` and `font_cs` respectively, and falls back the way Word does
when a slot is absent.

`resolve_font_family` at `engine.rs:7835` becomes slot-aware and keeps its
present behaviour byte for byte for the `Ascii` slot, including the theme
reference path. The theme path is corrected at the same time so
`majorEastAsia`, `minorEastAsia`, `majorBidi` and `minorBidi` read their own
theme entry instead of collapsing onto the ASCII font.

### C. Bundled deterministic subset faces

Three faces, following `crates/oxml-layout/fonts/SUBSET-NotoSansSC.md` exactly
as the precedent it already is:

| File | Source | Repertoire |
|---|---|---|
| `NotoSansHebrew-F266a-subset.ttf` | `ofl/notosanshebrew/NotoSansHebrew[wdth,wght].ttf` | The approved Hebrew fixture string plus ASCII space and comma |
| `NotoSansKR-F266a-subset.ttf` | `ofl/notosanskr/NotoSansKR[wght].ttf` | The approved Hangul fixture string plus ASCII space and comma |
| `NotoSansJP-F266a-subset.ttf` | `ofl/notosansjp/NotoSansJP[wght].ttf` | The approved Kana and Kanji fixture string plus ASCII space and comma |

Each gets its own `SUBSET-*.md` with the `pyftsubset` command, the source
SHA-256 and the output SHA-256, in the same form as the existing record. Each
gets a `NOTICE-Noto` entry. All three are SIL OFL 1.1 and therefore need **no
new licence file**, since `fonts/LICENSE-Noto` already covers them.

Everything that counts the inventory grows together: the `include` list in
`crates/oxml-layout/Cargo.toml`, the expected-font list in
`.github/workflows/ci.yml` lines 546 to 570, the `family_licences` table and
the fixture-repertoire test in `crates/oxml-layout/src/bundled_fonts.rs:135` and
`bundled_fonts.rs:170`, and `bundled_font_data` itself. CI inventory goes from
24 TTFs to 27.

Shipped coverage is the fixture repertoire only, not the full families. That is
what DOCX-033 will say, because it is what is true.

### Files the diff touches

- `crates/oxml-layout/src/font.rs`
- `crates/oxml-layout/src/bundled_fonts.rs`
- `crates/oxml-layout/fonts/`, three TTFs, three `SUBSET-*.md`, `NOTICE-Noto`
- `crates/oxml-layout/Cargo.toml`
- `crates/rdocx-layout/src/engine.rs`
- `crates/rdocx/tests/integration_test.rs`
- `crates/rdocx/tests/regression_test.rs`
- `.github/workflows/ci.yml`

No new crate, module or file.

## Rejected alternatives

- **Add a shaping or BiDi dependency.** Rejected. `harfrust` 0.12 and
  `unicode-bidi` `=0.3.18` are already workspace dependencies and already do
  complex-script joining and UAX 9. Nothing is missing at that layer.
- **Bundle full Noto Sans KR and Noto Sans JP.** Rejected. Each is roughly 5 MB
  against a `oxml-layout` archive already near 4.6 MB and a crates.io 10 MiB
  ceiling gated at `docs/hld/15-build-and-toolchain.md:273`.
- **Put the three faces under `scripts/oracle-fonts/`.** Rejected in the S74
  round. It leaves the archive untouched but proves nothing about the shipped
  deterministic mode, which is what the gate exists to prove.
- **Leave `TextScript` exhaustive and avoid the semver break.** Rejected. The
  enum will keep growing, so the break is deferred rather than avoided.
- **Map Hangul and Kana onto `TextScript::Han`.** Rejected. They need different
  shaper feature sets and different line-break behaviour, and conflating them
  would silently change existing Han text.
- **Resolve font slots inside `oxml-layout`.** Rejected. `w:rFonts` is
  WordprocessingML grammar, and `oxml-*` must not learn `w:` semantics.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| golden | `mixed_script_page_matches_the_pinned_geometry_and_reading_order` | **The test gate.** One in-code document built through the public facade with Arabic, Hebrew, Korean, Japanese and Latin paragraphs, laid out with `FontManager::new_deterministic`. A canonical serialisation of every glyph run's font identity, glyph ids, x and y advances and offsets, logical cluster ranges and painted visual order hashes to a digest recorded as a constant in the test. Reading order is asserted separately and readably, so a failure says which property broke |
| unit | `hangul_and_kana_receive_their_own_script_and_shaper_tag` | `script_for_char` returns `Hangul` and `Kana` for the new ranges, `harfrust_script` maps both, and an exhaustive sweep proves no previously mapped codepoint changed script |
| unit | `korean_text_takes_the_east_asian_language_slot` | `word_language_slot` returns `EastAsia` for Hangul, and the existing Kana and Han answers are unchanged |
| unit | `run_fonts_resolve_from_the_matching_script_slot` | `eastAsia`, `cs`, `hAnsi` and `ascii` each win for their own characters, and an absent slot falls back the way Word does |
| unit | `a_font_hint_decides_only_the_ambiguous_slot` | `w:rFonts/@w:hint` changes the resolved family for a `Common` character and changes nothing for a character whose slot is unambiguous |
| unit | `east_asia_and_bidi_theme_references_read_their_own_theme_entry` | `majorEastAsia`, `minorEastAsia`, `majorBidi` and `minorBidi` no longer collapse onto the ASCII theme font |
| unit | `every_bundled_font_family_has_a_licence_file` | Extended, not replaced. The three new families appear with `LICENSE-Noto`, and the three `SUBSET-*.md` records exist |
| unit | `deterministic_complex_script_fonts_cover_the_approved_fixture_repertoire` | Extended with the Hebrew, Hangul and Japanese fixture strings, so a mis-subset font fails at the unit layer rather than in the golden |
| regression | `a_right_to_left_paragraph_keeps_logical_order_in_extracted_text` | UAX 9 reordering changes painted order only. `ActualText`, SVG text and round-trip XML stay logical |
| regression | `arabic_runs_stay_joined_across_a_run_boundary` | One Arabic word split into two `w:r` elements keeps its joining forms |
| regression | `numbering_markers_stay_on_the_leading_edge_of_a_right_to_left_paragraph` | Markers participate in line-local visual order, per `docs/hld/08-rendering-spec.md` |

The **test gate** is the golden
`mixed_script_page_matches_the_pinned_geometry_and_reading_order`.

Every rendering assertion uses `FontManager::new_deterministic`. No baseline is
recorded against system fonts. No binary fixture is added. New tests join the
existing `crates/rdocx/tests/integration_test.rs` and `regression_test.rs`
entrypoints, and the `oxml-layout` and `rdocx-layout` units join those crates'
existing in-file `#[cfg(test)]` modules.

## HLD impact

- `docs/hld/08-rendering-spec.md`
- `docs/hld/12-testing-strategy.md`
- `docs/hld/15-build-and-toolchain.md`
- `docs/hld/14-development-backlog.md`

`docs/hld/02-scope-and-non-goals.md` is deliberately absent. DOCX-033 reaches
its final classification in F-266c, the last child to close, so the row is not
edited three times toward three different half-truths.

## Risk routing

Three rows of `.claude/skills/risk-routing.md` match.

1. **Layout, pagination, line breaking, text shaping.** Read
   `docs/hld/08-rendering-spec.md`. Extra checks: every baseline here uses
   `FontManager::new_deterministic`. `python3 scripts/hash_harness.py --check`
   and `python3 scripts/golden_png_harness.py --check` both run on the
   bundled-font commit in isolation. No baseline is re-recorded incidentally.

2. **Bundled fonts.** Read `docs/hld/15-build-and-toolchain.md`. Extra checks:
   all three families are SIL OFL 1.1 under the existing `fonts/LICENSE-Noto`,
   each gets a `NOTICE-Noto` entry carrying its upstream source SHA-256, and
   each gets a `SUBSET-*.md` reproduction record with source and output SHA-256
   matching `SUBSET-NotoSansSC.md`. The `Cargo.toml` include list, the
   `.github/workflows/ci.yml` expected-font list and the
   `every_bundled_font_family_has_a_licence_file` test all grow to 27 TTFs
   together. The files live under `crates/oxml-layout/fonts/` so they reach the
   published tarball.

3. **Public API of a published crate.** Read `docs/hld/10-bindings-spec.md` and
   the `CLAUDE.md` structural rules. Extra checks: the `TextScript` change is
   breaking and is stated as such, with `#[non_exhaustive]` in the same commit.
   Run `cargo publish --dry-run` for `oxml-layout` and `rdocx-layout`, and
   assert the `oxml-layout` `.crate` archive stays below the crates.io 10 MiB
   limit with the three new faces included.

Not matched: unit conversion, theme colour tint and shade, any parser or
serialiser, crate dependency graph, feature flags, file moves, new trait or
generic, external oracle, release scripting. The golden is a recorded geometry
digest and needs no rasteriser, which is why the external-oracle row stays
unmatched.

## Hash harness

**Expected unchanged. All 49 entries and all 7 golden-PNG entries.**

The seven `samples/` documents generated by
`crates/rdocx/examples/generate_all_samples.rs` are ASCII plus Latin-1
punctuation. No Arabic, Hebrew, Han, Kana or Hangul character appears in any of
them. Taking the change groups in turn:

- **Script identity.** The added Hangul and Kana ranges are absent from every
  sample and no existing range moves, so `script_for_char` returns an identical
  answer for every character the samples contain. `#[non_exhaustive]` is a
  compile-time attribute with no runtime effect.
- **Font slot resolution.** Sample runs set only `w:rFonts/@w:ascii`, and every
  sample character classifies to the `Ascii` slot, which is exactly what
  `resolve_font_family` does today. The theme correction touches only
  `majorEastAsia`, `minorEastAsia`, `majorBidi` and `minorBidi`, and no sample
  uses any of the four.

**The one real risk.** Adding faces to `bundled_font_data` at
`crates/oxml-layout/src/bundled_fonts.rs:24` enlarges the deterministic
`fontdb`, and `resolve_font_for_text` scans that database for coverage when the
requested family cannot draw a character (`font.rs:997` to `font.rs:1036`). If
a sample character is silently taking a coverage fallback today, a newly
bundled face could win that scan and move a sample. Named-family resolution is
unaffected, and Carlito and the Liberation families cover every character the
samples use including `U+2014`, so the expectation is no movement.

**Containment, which is a requirement and not a suggestion.** The bundled-font
addition lands as **its own labelled commit with nothing else in it**, and both
harnesses run on that commit alone. A delta there is not folded into a
behaviour change and is not accepted as incidental. It would mean a sample was
already falling back, which is a finding in its own right and stops the story
per the `.claude/WORKFLOW.md` escalation table.

This story does not claim the sprint's exclusive baseline re-record.

## Implementation checklist

- [x] Confirm F-264 and F-265 are `done` and that all six consumed surfaces in
      "Prerequisites delivered by earlier stories" exist. Stop and re-design if
      any is absent or shaped differently.
- [x] Confirm the run-property module split has landed and read `CT_RPr` from
      it.
- [x] Produce the three subsets with `pyftsubset`, record source and output
      SHA-256 in three `SUBSET-*.md` files, and add the `NOTICE-Noto` entries.
- [x] Add the three faces to `bundled_font_data`, the `Cargo.toml` include list,
      the CI expected-font list, and both `bundled_fonts.rs` tests.
      **Own labelled commit, nothing else in it. Run both harnesses on it
      alone.**
- [x] Add `TextScript::Hangul`, `TextScript::Kana` and `#[non_exhaustive]` in
      one commit, with the ranges and the HarfRust tags.
- [x] Add the Hangul East Asian language slot in `word_language_slot`.
- [x] Add `WordFontSlot`, slot-aware `resolve_font_family`, and the four
      corrected East Asian and complex-script theme references.
- [x] Add every test in `## Test plan` to the existing entrypoints.
- [x] Record the golden geometry digest, with its reason, in the test and in
      `docs/hld/12-testing-strategy.md`.
- [x] Run `/verify`, plus `cargo test -p oxml-layout --no-default-features`,
      `cargo check --target wasm32-unknown-unknown -p rdocx-wasm -p rpptx-wasm`,
      and `cargo publish --dry-run` with the archive-size assertion.

## Open questions

None. Resolved in the S74 consolidated design round.
