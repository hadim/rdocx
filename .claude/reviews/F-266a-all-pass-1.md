# F-266a, all, pass 1

**Reviewed**: `git diff 562af721..HEAD` in the worker worktree, the two F-266a
commits `a8bf6e5a` (bundled fonts, containment commit) and `05cd97f5` (the
behaviour change) reviewed as one delta. 18 files changed, 1170 insertions and
34 deletions. Three of the 18 are binary TTFs, so 15 text files carry the
diff. Aspects run: correctness, contract, panics, ooxml, tests, structure.

**Verdict**: 1 defect, 6 smells, 7 nitpicks

## What was verified rather than trusted

- `cargo fmt --all --check`, clean.
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`, clean.
- `cargo test --workspace --all-features --exclude rdocx-py --exclude rpptx-py`,
  every target passes except the known environment artifact
  `rpptx-cli validate_rejects_corruption_and_accepts_the_pinned_corpus`.
- `cargo test -p oxml-layout`, 107 pass. `--no-default-features`, 105 pass, and
  both new `oxml-layout` units run in both configurations.
- `python3 -m scripts.test_sprint_workflow`, 122 tests, OK with 2 skipped.
- `python3 scripts/hash_harness.py --check`, 49 entries match.
- `python3 scripts/prose_check.py`, 0 violations.
- `python3 scripts/sync_agent_skills.py --check`, 26 skills in sync.
- Font inventory. 27 TTFs and 6 legal files on disk. The three new output
  SHA-256 values in `SUBSET-NotoSansHebrew.md`, `SUBSET-NotoSansKR.md` and
  `SUBSET-NotoSansJP.md` match the committed bytes exactly. Each
  `--unicodes=` list is exactly the distinct codepoints of its declared fixture
  string plus space and comma, with no surplus and no omission. `CLAUDE.md`,
  `.github/workflows/ci.yml`, `scripts/test_sprint_workflow.py` and
  `docs/hld/15-build-and-toolchain.md` all read 27, and
  `crates/oxml-layout/Cargo.toml:15` does glob `fonts/*.ttf`, so the claim that
  it needed no change is correct.
- Containment. `a8bf6e5a` touches only font bytes, subset records, the notice,
  the inventory counts and the two `bundled_fonts.rs` tests. It contains no
  behaviour change, so the containment requirement is structurally satisfied.
- `TextScript` semver. No `match` on `TextScript` anywhere in the workspace is
  exhaustive. The only non-defining uses are equality comparisons and literal
  construction in `crates/rdocx/src/svg.rs:1161`,
  `crates/oxml-pdf/src/writer.rs:2040` and test code. The commit message's
  claim is accurate.
- **Not verifiable here**: `python3 scripts/golden_png_harness.py --check`.
  `/private/tmp/rdocx-s73-bin/pdftoppm` is a Docker wrapper that bind mounts
  only the canonical repository path, so the rasteriser cannot see a worker
  worktree. This is an environment artifact of the same class as the corpus
  failure and is not a finding against the diff, but the 7 of 7 claim rests on
  the implementer's run alone.

## Defects

### D1, the `w:ascii` family outranks the slot's own theme attribute, so `w:eastAsiaTheme` and `w:cstheme` stay inert on facade-authored runs

`crates/rdocx-layout/src/engine.rs:8370`, with
`crates/rdocx-layout/src/engine.rs:8391`

The plan states the priority as "the slot's explicit family > the slot's theme
font > None", and the function's own doc comment at
`crates/rdocx-layout/src/engine.rs:8375` repeats it. The code implements a
different order. `word_font_for_slot` ends in
`.or_else(|| rpr.font_ascii.clone())`, and `resolve_font_family` returns that
result before it ever looks at a theme attribute, so the real priority is
slot explicit, then `w:ascii` explicit, then slot theme, then `w:asciiTheme`.

Input that triggers it: a run carrying `w:rFonts w:ascii="Arial"
w:eastAsiaTheme="minorEastAsia"` and Korean or Japanese text. The East Asian
characters resolve to `Arial` and `w:eastAsiaTheme` is never read. The same
holds for `w:cstheme` beside a `w:ascii` family, for Hebrew and Arabic text.
That is exactly the "parsed and then ignored" condition the story exists to
remove, still reachable after the change.

The comment above the early return at
`crates/rdocx-layout/src/engine.rs:8382` pre-declares a divergence, but it
covers only the case where a producer "presents both for one slot", and it
rests on the sentence "setting either form through the facade clears the
other". That justification does not extend to the new code, because the
conflict here is across two different slots. `Run::set_slot_font` at
`crates/rdocx/src/run.rs:671` clears only that slot's theme attribute and its
own doc comment says it "touches no other slot", so
`set_slot_theme_font(RunFontSlot::EastAsia, Some("minorEastAsia"))` followed by
`set_slot_font(RunFontSlot::Ascii, Some("Arial"))` produces this state through
the public facade. The escape hatch of "a producer document the caller never
edited" does not apply.

The new unit
`east_asia_and_bidi_theme_references_read_their_own_theme_entry` at
`crates/rdocx-layout/src/engine.rs:19138` never sets an explicit family beside
a slot theme reference, so it does not reach the case.

## Smells

### S1, the four codepoint tables now disagree about the Katakana phonetic extensions

`crates/rdocx-layout/src/engine.rs:8474`

`script_for_char` at `crates/oxml-layout/src/font.rs:1784`, `word_font_slot` at
`crates/rdocx-layout/src/engine.rs:8326` and `needs_word_multilingual_layout`
at `crates/rdocx-layout/src/engine.rs:8589` all now cover `0x31f0..=0x31ff`.
`word_language_slot` does not. Those characters therefore return `None`, and
`word_language_ranges` at `crates/rdocx-layout/src/engine.rs:8516` skips a
`None` character and attaches it to whatever slot preceded it, which at the
start of a run is the `WordLanguageSlot::Direct` initial value at
`crates/rdocx-layout/src/engine.rs:8514`. A run that opens with a Katakana
phonetic extension now reaches the rich path and takes `w:lang/@w:val` where
Word takes `w:lang/@w:eastAsia`. The comment at
`crates/rdocx-layout/src/engine.rs:8474` says "Kana is already inside the
0x3000 arm", which was true before the sibling functions were widened past
`0x30ff` and is no longer true of the whole Kana set the diff recognises.

### S2, the new slot table leaves out the East Asian ranges `w:eastAsia` is most often needed for

`crates/rdocx-layout/src/engine.rs:8322`, with the default arm at
`crates/rdocx-layout/src/engine.rs:8336`

The `EastAsia` arm covers Hangul, CJK punctuation, Kana, Han and the
compatibility ideographs, and nothing else. Halfwidth and fullwidth forms
`0xff00..=0xffef`, which carry halfwidth Katakana `U+FF65..U+FF9F` and
fullwidth Latin, fall through. So do Bopomofo `0x3100..=0x312f`, Kanbun and
CJK strokes `0x3190..=0x31ef`, enclosed CJK and CJK compatibility
`0x3200..=0x33ff`, Hangul Jamo Extended-A `0xa960..=0xa97f`, Hangul Jamo
Extended-B `0xd7b0..=0xd7ff` and every supplementary-plane codepoint. Latin
Extended-D `0xa720..=0xa7ff` and the Latin ligature block `0xfb00..=0xfb1c`
fall through too, where they belong in `HighAnsi`.

None of this is a regression, because `Ascii` was the universal answer before
the diff, so the fallthrough reproduces the old behaviour. Two things make it
worth recording anyway. First, `w:eastAsia` stays inert for the Japanese text
most likely to set it. Second, everything that falls through lands in the
`w:hint` branch, so halfwidth Katakana in a run carrying `w:hint="cs"` resolves
through `w:cs`, and Word does not treat halfwidth Katakana as ambiguous.

`script_for_char` has the matching Hangul hole.
`crates/oxml-layout/src/font.rs:1783` covers `0xac00..=0xd7af` but neither
Hangul Jamo Extended-A nor Extended-B, so those still reach the shaper as
`harfrust::script::COMMON`.

### S3, the declared gate does not distinguish slot resolution from coverage fallback

`crates/rdocx/tests/integration_test.rs:18708`, with
`docs/hld/12-testing-strategy.md:1362`

The fixture doc comment claims "the page exercises slot resolution and not
coverage fallback", and the spec paragraph repeats it. Each of the five scripts
on the page has exactly one bundled face that covers it. With
`resolve_font_family` reverted to its pre-diff form, every one of those runs
would request no family at all, fall into the coverage scan in
`resolve_font_for_text`, and land on the same face. The family assertion at
`crates/rdocx/tests/integration_test.rs:18910` and the digest at
`crates/rdocx/tests/integration_test.rs:18991` would both still pass. The gate
does fail on a full revert, but only because `TextScript::Hangul` and
`TextScript::Kana` would no longer compile, which is a weaker guarantee than
the comment claims.

The property is genuinely proved elsewhere, by
`an_east_asian_run_reads_its_own_font_slot_and_not_the_ascii_one` at
`crates/rdocx/tests/regression_test.rs:30857`, which pits two faces that both
cover the text against each other. The coverage exists. What is wrong is the
claim attached to the gate and repeated in the spec set.

### S4, Korean paragraphs silently leave the paragraph block cache and nothing records or tests it

`crates/rdocx-layout/src/engine.rs:8586`, with
`crates/rdocx-layout/src/engine.rs:4926`

Admitting the Hangul ranges to `needs_word_multilingual_layout` puts Korean on
the rich path, and `inline_bytes` scores `InlineItem::MultilingualText` as
`usize::MAX`. The surrounding sum saturates, so no Korean paragraph can be
admitted to the paragraph block cache again. That is a real behaviour change
for every existing Korean document, separate from the shaping improvement the
commit message describes.

The response was to change the `mixed_editor_input` fixture text from Korean to
Latin at `crates/rdocx-layout/src/engine.rs:11909`. That is legitimate on its
own terms, and the doc comment at
`crates/rdocx-layout/src/engine.rs:11897` is honest about the reason, so this
is not a hidden regression. What is missing is the other half. After the
change, `mixed_editor_relayout_reuses_every_safe_unchanged_block_and_page` at
`crates/rdocx-layout/src/engine.rs:11938` and
`mixed_editor_table_mutation_rebuilds_only_the_changed_table` at
`crates/rdocx-layout/src/engine.rs:11989` carry no non-Latin text at all, the
new cache exclusion is asserted by no test, and it appears in no tracker, no
backlog entry and no `docs/hld/` paragraph. Either a rich-path cache assertion
or a recorded consequence is owed.

### S5, the plan deviation is defended only in a commit message

`crates/rdocx/tests/regression_test.rs:30787`

The substitute regression
`arabic_shaping_applies_contextual_joining_forms_within_one_run` is honest. Its
doc comment states plainly that joining does not cross a `w:r` boundary in this
engine, that Word multilingual reassembly keeps every shaped span inside its
inline item, and that the plan's
`arabic_runs_stay_joined_across_a_run_boundary` is therefore not writable here.
The deviation itself is acceptable and the renaming is the right call.

Two things do not hold up. First, the commit message says "the gap is recorded
in the handoff for its own F-ID", and no `.claude/handoffs/` directory exists
in this tree, no backlog row names cross-run Arabic joining, and
`docs/hld/14-development-backlog.md` does not either. The only record of the
gap is a commit message and a test doc comment. Second, the test builds its
control arm out of one run per character, which encodes the current
non-joining-across-runs behaviour as the expected baseline. A later change that
did join across runs would make `connected == isolated` and fail this test with
a message that points at the wrong conclusion.

### S6, the geometry digest is the first float-derived recorded digest in the suite and was pinned on one host

`crates/rdocx/tests/integration_test.rs:18703`, with
`crates/rdocx/tests/integration_test.rs:18790`

Every other recorded digest in `docs/hld/12-testing-strategy.md` hashes the
bytes of a produced artifact, for example the two SHA-bound files at
`docs/hld/12-testing-strategy.md:1353`. This one hashes
`format!("{value:.4}")` over f64 coordinates, advances and offsets. The
justification at `crates/rdocx/tests/integration_test.rs:18701`, repeated at
`docs/hld/12-testing-strategy.md:1375`, is that four decimal places "absorbs
last-bit float noise between hosts". Fixed-point rounding does not absorb
noise, it relocates the cliff. A value that lands within an ulp of a
`.00005` boundary flips the whole digest, and `format!("{:.4}", -0.0)` renders
`-0.0000`, so a sign-of-zero difference in an offset or advance moves it too.

The practical risk is low. The pipeline is f64 with no FMA contraction, the
shaper is pure Rust and the fonts are bundled. But the recording host is not
the CI host, nothing else in the suite establishes this serialisation as
host-stable, and the stated reason for believing it is stable is not the reason
that would actually make it stable.

## Nitpicks

- `crates/oxml-layout/src/font.rs:1779`, the comment says "Kana precedes Han
  because both live in the CJK planes and neither range overlaps". If neither
  range overlaps, the ordering the sentence defends cannot matter.
- `crates/rdocx-layout/src/engine.rs:8576`, the doc comment justifies the
  Katakana phonetic extensions with a sentence about Korean, which does not
  apply to them.
- `crates/rdocx/tests/regression_test.rs:30858`, "both cover ASCII letters" is
  false. `crates/oxml-layout/fonts/SUBSET-NotoSansJP.md` approves only space,
  comma, the Kana, `U+3001` and two Kanji, so the JP subset carries no Latin
  letters. The test is still discriminating, the comment is not accurate.
- `crates/rdocx/tests/integration_test.rs:18918`,
  `text.contains(logical.trim())` is a containment check rather than a reading
  order check. It still passes if runs are dropped, as long as what survives is
  a contiguous substring. The plan asked for reading order "asserted separately
  and readably".
- `crates/rdocx-layout/src/engine.rs:6083`, `word_font_slot_for_text(&run.text(),
  ...)` allocates a fresh `String` for every run on every layout pass, only to
  classify characters until the first non-ASCII slot.
- `docs/hld/15-build-and-toolchain.md:273` and
  `docs/hld/08-rendering-spec.md:505`, both rewraps leave an orphan line ("job
  then runs verified packaging" after a three-word line, and "projection selects
  the" alone).
- `docs/hld/14-development-backlog.md` is named in the plan's `## HLD impact`
  and is untouched by this diff, while the other three named files were edited
  inside it. `/complete-feature` executes that list, so the split is only
  inconsistent rather than wrong.

## Not found

- **panics**. Nothing in the production diff can panic. The added code is range
  matching on `character as u32`, `Option` cloning and `Vec` pushes. There is no
  new `unwrap`, `expect`, slicing, indexing or arithmetic on untrusted input
  outside `#[cfg(test)]` code. `include_bytes!` is compile time.
- **ooxml**. Nothing checked in this aspect produced a finding. The diff adds no
  parser and no serialiser and changes no element or attribute order. `w:rFonts`
  reading and writing, including `@w:hint`, belongs to F-265 and is untouched
  here. The hint values matched at
  `crates/rdocx-layout/src/engine.rs:8337` are the `ST_Hint` enumeration
  literals, so exact-case matching is right. No unmodelled subtree is dropped.
- **structure**. Nothing checked in this aspect produced a finding. The diff
  adds one private enum and three private functions to an existing file, next to
  the `WordLanguageSlot` pattern they deliberately mirror. No new trait, no new
  generic parameter, no `Box<dyn>`, no forwarding wrapper, no new feature flag,
  and no new crate, module or file. Both new test groups join existing
  integration entrypoints rather than adding a test binary.
- **contract, on the parts not listed above**. The bundled-font containment
  requirement, the subset record format, the licence and notice obligations, the
  27-TTF inventory, the `#[non_exhaustive]` landing in the same commit as the two
  variants, the "no existing range moves" claim and the deterministic font mode
  requirement for every new rendering assertion are all met as written.
