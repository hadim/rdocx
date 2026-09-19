# F-266a, all, pass 2

**Reviewed**: `git diff 562af721..HEAD` in the worker worktree, the three
F-266a commits `a8bf6e5a` (bundled fonts, containment commit), `05cd97f5` (the
behaviour change) and `e96b7ddc` (the pass 1 remediation) reviewed as one
delta. 20 files changed, 1735 insertions and 60 deletions. Three of the 20 are
binary TTFs and one is the pass 1 review file itself, so 16 text files carry
the reviewable diff. Aspects run: correctness, contract, panics, ooxml, tests,
structure.

**Verdict**: 2 defects, 2 smells, 4 nitpicks

## What was verified rather than trusted

- `cargo fmt --all --check`, clean.
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`, clean.
- `cargo test -p rdocx-layout`, 289 pass plus the doctest.
- `cargo test -p rdocx --test integration_test --test regression_test`, 291 and
  515 pass.
- `cargo test -p oxml-layout`, 107 pass. `--no-default-features`, 105 pass, and
  both new `oxml-layout` units were confirmed by name to run in the
  no-default-features configuration.
- `cargo check --target wasm32-unknown-unknown -p rdocx-wasm -p rpptx-wasm`,
  clean.
- `python3 scripts/hash_harness.py --check`, 49 entries match.
- `python3 scripts/prose_check.py`, 0 violations.
- `python3 scripts/sync_agent_skills.py --check`, 26 skills in sync.
- `python3 -m scripts.test_sprint_workflow`, 122 tests, OK with 2 skipped.
- `cargo package -p oxml-layout --list` lists 27 TTFs, and
  `cargo publish --dry-run -p oxml-layout` produces a 4.41 MiB archive, well
  under the crates.io 10 MiB ceiling the plan's risk routing asked for.
- **`python3 scripts/golden_png_harness.py --check` now ran in the worktree, 7
  of 7 page-one pixel buffers match at 150 DPI, rasteriser reported as
  `pdftoppm version 26.01.0`.** The shim at
  `/private/tmp/claude-501/.../scratchpad/bin/pdftoppm` is a legitimate
  environment workaround and does not invalidate the result. It invokes the
  same `rdocx-pinned-oracles:s73` image, the same
  `/opt/poppler-26.01.0/bin/pdftoppm` entrypoint, the same user and the same
  read-only mount policy as `/private/tmp/rdocx-s73-bin/pdftoppm`. The only
  difference is one additional read-only bind mount for the worker worktree
  path. Nothing about the rasteriser, its version or its arguments changes, and
  the harness printed the pinned version back. The pass 1 gap is therefore
  closed by observation rather than by the implementer's claim.
- Facade reachability of the D1 input below was confirmed against
  `crates/rdocx/src/run.rs:671` and `crates/rdocx/src/run.rs:684`, which clear
  only the sibling form within one slot.

## Job 1, the state of every pass 1 finding

### D1, the `w:ascii` family outranks the slot's own theme attribute

**Resolved.** `crates/rdocx-layout/src/engine.rs:8427`. `resolve_font_family`
is now `word_font_for_slot(rpr, theme, slot).or_else(|| word_font_for_slot(rpr,
theme, WordFontSlot::Ascii))`, and `word_font_for_slot` at
`crates/rdocx-layout/src/engine.rs:8376` resolves explicit family then theme
attribute inside one slot before returning. The new case in the unit at
`crates/rdocx-layout/src/engine.rs:19240` sets `font_ascii` beside
`font_east_asia_theme` and `font_cs_theme` and requires the slot theme to win,
which is exactly the state pass 1 named. An ASCII-only run is byte-identical,
proved by `crates/rdocx-layout/src/engine.rs:19221`.

The fix is correct in ordering and reopens a different problem in resolution,
recorded below as pass 2 D1. That is a new finding, not this one reopened.

### S1, the four codepoint tables disagree about the Katakana phonetic extensions

**Partially resolved.** The specific disagreement is gone.
`crates/rdocx-layout/src/engine.rs:8494` now covers `0x31f0..=0x31ff`, and the
new sweep at `crates/rdocx-layout/src/engine.rs:19286` proves that every
codepoint `script_for_char` calls Hangul, Kana or Han takes the East Asian
language slot, the East Asian font slot and the rich path. I re-derived that
invariant by hand against `crates/oxml-layout/src/font.rs:1787` and
`crates/oxml-layout/src/font.rs:1788` and it holds.

What is not resolved is the general claim. The remediation widened
`word_font_slot` much further than the other three, so the tables disagree
again over a different set. Carried forward as pass 2 S1.

### S2, the slot table leaves out the East Asian ranges `w:eastAsia` is needed for

**Resolved.** `crates/rdocx-layout/src/engine.rs:8332` now claims
`0x3000..=0x33ff`, `0xa960..=0xa97f`, `0xac00..=0xd7ff`, `0xff00..=0xffef` and
`0x20000..=0x2fa1f` for `EastAsia`, and
`crates/rdocx-layout/src/engine.rs:8346` claims Latin Extended-D and the Latin
ligature block for `HighAnsi`. The `script_for_char` Hangul hole is closed at
`crates/oxml-layout/src/font.rs:1787`, which now reads
`0x1100..=0x11ff | 0x3130..=0x318f | 0xa960..=0xa97f | 0xac00..=0xd7ff`.

### S3, the gate does not distinguish slot resolution from coverage fallback

**Resolved.** `crates/rdocx/tests/integration_test.rs:18764` adds a Kanji
paragraph naming `Noto Sans SC` on `w:ascii` and `Noto Sans JP` on
`w:eastAsia`. I checked both repertoires:
`crates/oxml-layout/fonts/SUBSET-NotoSansSC.md:15` approves `U+4E16` and
`U+754C`, and `crates/oxml-layout/fonts/SUBSET-NotoSansJP.md:16` approves the
same two. Both faces cover `世界`, so coverage fallback cannot pick between
them and the family assertion at
`crates/rdocx/tests/integration_test.rs:18942` fails on a revert of slot
resolution alone. Reading order is now exact per-paragraph reassembly at
`crates/rdocx/tests/integration_test.rs:18978`, not containment. The spec
paragraph at `docs/hld/12-testing-strategy.md:1375` states the same thing
accurately.

### S4, Korean paragraphs silently leave the paragraph block cache

**Resolved.** `crates/rdocx-layout/src/engine.rs:19337` asserts the exclusion
directly, with a Latin control at `(8, 8)` and five complex scripts at
`(0, 16)`, so a Hangul range removed from `needs_word_multilingual_layout`
fails it. The consequence is recorded in
`docs/hld/08-rendering-spec.md:508`, `docs/hld/12-testing-strategy.md:1398` and
`docs/hld/14-development-backlog.md:2626`.

### S5, the plan deviation is defended only in a commit message

**Resolved on substance.** The control arm at
`crates/rdocx/tests/regression_test.rs:30822` is now one run with zero-width
non-joiners, and
`crates/rdocx/tests/regression_test.rs:30836` adds the stronger assertion that
joining produces a glyph the unjoined letters never take, so the test no longer
encodes the current no-joining-across-runs behaviour as its baseline. The gap
itself is recorded at `docs/hld/14-development-backlog.md:2622`.

The pointer inside the test is still wrong. See nitpick 1.

### S6, the geometry digest rationale

**Resolved.** `crates/rdocx/tests/integration_test.rs:18816` introduces
`fn number(value: f64)`, which maps any value equal to zero to positive zero
before formatting, so `-0.0` can no longer print `-0.0000`. Every coordinate,
advance and offset in `canonical_geometry` passes through it or through
`numbers`. The stated reason at
`crates/rdocx/tests/integration_test.rs:18707` and
`docs/hld/12-testing-strategy.md:1382` is now the reason that actually holds,
an f64 pipeline with no FMA contraction, a pure Rust shaper and bundled faces,
with four decimal places described as a guard band rather than as noise
absorption. The digest was re-recorded to
`516ebb6e45438731d3cb0983707ad00c9de55068401e073ef2a069a56f397402` and the
test passes.

### The seven pass 1 nitpicks

1. `crates/oxml-layout/src/font.rs:1779`, the self-defeating Kana and Han
   ordering sentence. **Fixed**, rewritten to explain why they are not folded
   into Han.
2. The Katakana phonetic extension comment justified by a sentence about
   Korean. **Fixed** at `crates/rdocx-layout/src/engine.rs:8597`, the reason is
   now stated once and applied to all three.
3. "both cover ASCII letters". **Fixed** at
   `crates/rdocx/tests/regression_test.rs:30875`, which now states the opposite
   and correct fact and cites the subset record.
4. Containment rather than reading order. **Fixed**, replaced by exact
   reassembly at `crates/rdocx/tests/integration_test.rs:18978`.
5. The per-run `run.text()` allocation. **Fixed** at
   `crates/rdocx-layout/src/engine.rs:6080`, where `run_text` is bound once and
   reused by `fm.resolve_font_for_text`, so no allocation was added.
6. The two rewrap orphans. **Half fixed.** `docs/hld/08-rendering-spec.md` is
   clean. `docs/hld/15-build-and-toolchain.md:273` still leaves `archive or one
   larger than the crates.io 10 MiB limit.` after a three-word line. Still a
   nitpick.
7. `docs/hld/14-development-backlog.md` untouched while named in the plan's
   `## HLD impact`. **Fixed**, it now carries eight lines.

## Defects

### D1, `majorEastAsia`, `minorEastAsia`, `majorBidi` and `minorBidi` resolve to the Latin typeface, and that answer now outranks an explicit `w:ascii` family

`crates/rdocx-layout/src/engine.rs:8413`, with
`crates/rdocx-oxml/src/theme.rs:179` and
`crates/rdocx-layout/src/engine.rs:8427`

`rdocx_oxml::theme::Theme` holds exactly two typefaces, `major_font` and
`minor_font` at `crates/rdocx-oxml/src/theme.rs:15` and
`crates/rdocx-oxml/src/theme.rs:17`, and `parse_font_scheme` at
`crates/rdocx-oxml/src/theme.rs:179` fills them from `<a:latin>` only. `<a:ea>`
and `<a:cs>` are never parsed. So the match at
`crates/rdocx-layout/src/engine.rs:8413` maps `majorEastAsia` and `majorBidi`
onto the Latin major typeface and `minorEastAsia` and `minorBidi` onto the
Latin minor typeface. The plan asks for these four to "read their own theme
entry instead of collapsing onto the ASCII font". What landed reads their own
**attribute**. The **entry** still collapses onto the Latin one, because no
other entry exists to read.

On its own that would be an unchanged half-truth. Pass 1's D1 fix makes it
active. `resolve_font_family` at `crates/rdocx-layout/src/engine.rs:8427` now
returns the slot's theme answer before it ever looks at `w:ascii`, so a Latin
theme typeface now beats an explicit complex-script or East Asian family.

Input that triggers it, reachable through the public facade because
`Run::set_slot_theme_font` at `crates/rdocx/src/run.rs:684` and
`Run::set_slot_font` at `crates/rdocx/src/run.rs:671` each touch one slot only:

```
run.set_slot_theme_font(RunFontSlot::ComplexScript, Some("minorBidi"));
run.set_slot_font(RunFontSlot::Ascii, Some("Noto Sans Arabic"));
```

with Arabic text and a theme whose minor Latin typeface is `Calibri`. Before
the diff the run resolved to `Noto Sans Arabic`, which is correct. After the
diff it resolves to `Calibri`, which cannot draw a single Arabic letter, and
the run survives only because `resolve_font_for_text` scans for coverage and
replaces the face wholesale. Word resolves `minorBidi` to `<a:cs>`, which is
empty in every stock Office theme, and therefore falls back to the run's own
family. The same shape arrives from producer documents, since Word's own
`docDefaults` carries `w:cstheme="minorBidi"` and `w:eastAsiaTheme=
"minorEastAsia"` while direct run formatting sets only `w:ascii` and `w:hAnsi`.

The behaviour is pinned rather than accidental. The unit at
`crates/rdocx-layout/src/engine.rs:19255` requires `MajorFace`, the Latin major
typeface, for the `EastAsia` and `ComplexScript` slots of a run whose only
explicit family is `AsciiFace`.

Either `Theme` gains the `<a:ea>` and `<a:cs>` typefaces so the four references
resolve to something real, or the four non-Latin references must yield `None`
and fall through to the `w:ascii` family rather than answering with a Latin
face. Answering with a Latin face is the one option that is wrong in both
directions.

### D2, `w:rFonts/@w:hint` is inert for every character a real document uses it on

`crates/rdocx-layout/src/engine.rs:8346`, with
`crates/rdocx-layout/src/engine.rs:8350` and the test at
`crates/rdocx-layout/src/engine.rs:19150`

The plan scopes the hint precisely: "`w:rFonts/@w:hint` breaking the tie only
for characters whose slot is genuinely ambiguous, which is the `Common`
range." `Common` has a definition in this workspace, it is what
`script_for_char` at `crates/oxml-layout/src/font.rs:1790` returns.

`word_font_slot` claims `0x00a0..=0x058f` and `0x1e00..=0x2bff` for `HighAnsi`
unconditionally, and the hint arm at
`crates/rdocx-layout/src/engine.rs:8350` is reached only by what no arm
claimed. Between them those two ranges swallow the entire set Word's hint
exists for: general punctuation `U+2000..U+206F` including `U+2013`, `U+2014`,
`U+2018` to `U+201D`, `U+2022` and `U+2026`, enclosed alphanumerics
`U+2460..U+24FF`, geometric shapes `U+25A0..U+25FF`, miscellaneous symbols
`U+2600..U+26FF`, and Greek and Cyrillic at `U+0370..U+04FF`. Every one of
those is `TextScript::Common` by this workspace's own table, and every one of
those is a character Word moves to the East Asian font when
`w:rFonts/@w:hint="eastAsia"` is present. A CJK document whose bullet glyph or
dash is meant to come from the East Asian face now takes `w:hAnsi` instead, and
nothing the author writes can change it.

What remains reachable by the hint is the C1 control block, unassigned gaps,
Khmer, Myanmar, Lao, Tibetan, the private use area, the CJK radicals blocks and
the astral planes outside `0x20000..=0x2fa1f`. The only character the unit test
exercises is `U+0085`, a C1 control, chosen at
`crates/rdocx-layout/src/engine.rs:19133` precisely because it "belongs to no
script range". The test then pins the defect at
`crates/rdocx-layout/src/engine.rs:19150`, which asserts that `'\u{2014}'` is
unambiguous and that no hint may move it.

This is the same "parsed and then ignored" condition the story exists to
remove, applied to the one `w:rFonts` attribute F-265 handed over specifically
so that F-266a could consume it.

## Smells

### S1, the four codepoint tables disagree again, over a different set, and both the comment and the new sweep say otherwise

`crates/rdocx-layout/src/engine.rs:8316`, with
`crates/rdocx-layout/src/engine.rs:8338` and
`crates/rdocx-layout/src/engine.rs:19289`

The remediation widened `word_font_slot` to `0x3000..=0x33ff`,
`0xff00..=0xffef` and `0x20000..=0x2fa1f`, and widened the other three tables
to a strictly smaller set. `word_language_slot` at
`crates/rdocx-layout/src/engine.rs:8496` stops at `0xff66..=0xff9f`,
`needs_word_multilingual_layout` at
`crates/rdocx-layout/src/engine.rs:8618` stops at the same place, and
`script_for_char` at `crates/oxml-layout/src/font.rs:1788` likewise.

The concrete case is halfwidth Hangul jamo, `U+FFA0..U+FFDC`. The font table
calls it East Asian, the language table returns `None`, the script table
returns `Common`, and the rich path never admits it. Under
`word_language_ranges` at `crates/rdocx-layout/src/engine.rs:8526` a `None`
character attaches to the preceding slot, which at the start of a run is
`WordLanguageSlot::Direct`, so halfwidth Korean beside other Korean takes
`w:lang/@w:val` where Word takes `@w:eastAsia`. That is the exact failure mode
pass 1 S1 described, surviving in the story's own headline script. Bopomofo
`0x3100..=0x312f`, Kanbun and the CJK strokes `0x3190..=0x31ef`, the enclosed
and compatibility blocks `0x3200..=0x33ff`, fullwidth Latin `0xff01..=0xff65`
and supplementary-plane Han `0x20000..=0x2fa1f` are in the same position.

Two statements assert the opposite. The doc comment at
`crates/rdocx-layout/src/engine.rs:8316` says "Word draws every CJK-adjacent
block from `w:eastAsia`", while CJK Radicals Supplement `U+2E80..U+2EFF`,
Kangxi Radicals `U+2F00..U+2FDF`, CJK Compatibility Forms `U+FE30..U+FE4F` and
Small Form Variants `U+FE50..U+FE6F` all fall through to the hint arm. And the
commit message for `e96b7ddc` says "All four now cover the same East Asian
set", which is not what the code does.

The new sweep cannot catch any of this. `EAST_ASIAN` at
`crates/rdocx-layout/src/engine.rs:19289` states a set narrower than all four
tables, so it proves a floor and never an agreement. Its own doc comment at
`crates/rdocx-layout/src/engine.rs:19280` promises that "a range added to one
of them and forgotten in another fails", and the remediation added five ranges
to one table and none to the others without the sweep noticing.

### S2, the whole run takes one slot, so Latin text inside a mixed run now resolves through `w:eastAsia` or `w:cs`

`crates/rdocx-layout/src/engine.rs:8364`, with
`crates/rdocx-layout/src/engine.rs:6081` and
`crates/rdocx-layout/src/engine.rs:19385`

The plan's signature is `fn word_font_slot(character: char, hint) ->
WordFontSlot`, one slot per character. `word_font_slot_for_text` is an addition
the contract does not name, and it collapses the run onto the slot of its first
non-ASCII character. `resolve_font_family` is then called once per run at
`crates/rdocx-layout/src/engine.rs:6081`.

The consequence is asserted deliberately at
`crates/rdocx-layout/src/engine.rs:19385`, where `"Hello 世界"` resolves to
`WordFontSlot::EastAsia`. For a run carrying `w:ascii="Arial"` and
`w:eastAsia="MS Mincho"`, Word draws `Hello ` from Arial and `世界` from MS
Mincho. Before this diff rdocx drew the whole run from Arial, right for the
Latin half and wrong for the Kanji. After this diff it draws the whole run from
MS Mincho, right for the Kanji and newly wrong for the Latin. Mixed
Latin-and-CJK runs are the normal shape of East Asian prose, and the same holds
for any Latin run containing one fullwidth character.

`docs/hld/08-rendering-spec.md:624` records the mechanism, "One family is
resolved per run, since coverage fallback already replaces one face for a whole
run", but does not record it as a divergence from Word or name what it costs.
Rule 5 of `.claude/skills/differential-testing.md` asks for the decision to be
written down and our side asserted. Half of that is done. The engine already
owns per-slot segmentation for language in `word_language_ranges` at
`crates/rdocx-layout/src/engine.rs:8526`, so the structure to do this properly
exists one function away, which is what makes it a smell rather than a
permanent constraint. Either the divergence is named as one in
`docs/hld/08-rendering-spec.md` with its cost, or it gets an F-ID.

## Nitpicks

- `crates/rdocx/tests/regression_test.rs:30796`, "That gap is recorded in the
  F-266a handoff". `.claude/handoffs/` contains only `.gitkeep`. The gap is
  genuinely recorded, at `docs/hld/14-development-backlog.md:2622`, so the
  sentence points a reader at the wrong place rather than at no place.
- `docs/hld/15-build-and-toolchain.md:273`, the pass 1 rewrap orphan survives,
  `archive or one larger than the crates.io 10 MiB limit.` still hangs after a
  three-word line.
- `docs/hld/15-build-and-toolchain.md:477`, "roughly 10 MB each full family
  weighs" contradicts the design plan's "Each is roughly 5 MB", in
  `.claude/plans/F-266a-design.md` under `## Rejected alternatives`. The
  argument survives either number, the spec set and the plan should not
  disagree about it.
- `crates/rdocx-layout/src/engine.rs:8342`, `0x0000..=0x007f` is claimed for
  `Ascii` but `0x0080..=0x009f` is not claimed by anything, so the C1 controls
  are the only text the hint arm sees in practice. That is what forced the unit
  test to pick `U+0085`. Claiming `0x0000..=0x009f` would cost nothing and
  would make the choice of test character honest.

## Not found

- **panics**. Nothing in the production diff can panic. The added and changed
  code is range matching on `character as u32`, `Option` cloning, `as_deref`
  and iterator chains. No new `unwrap`, `expect`, slicing, indexing or
  arithmetic on untrusted input outside `#[cfg(test)]`. The remediation added
  `.as_deref()?` and `theme?` in `word_font_for_slot`, both of which are early
  returns rather than unwraps. `include_bytes!` remains compile time.
- **ooxml**. Nothing checked in this aspect produced a finding. The delta adds
  no parser and no serialiser and changes no element or attribute order. The
  hint values matched at `crates/rdocx-layout/src/engine.rs:8351` are the
  `ST_Hint` enumeration literals, so exact-case matching is right. The theme
  attribute values matched at `crates/rdocx-layout/src/engine.rs:8413` are the
  `ST_Theme` literals and all eight are handled. No unmodelled subtree is
  dropped, and the round-trip assertion at
  `crates/rdocx/tests/integration_test.rs:19034` proves the saved bytes stay in
  logical order.
- **structure**. Nothing checked in this aspect produced a finding. The
  remediation added no type, no trait, no generic parameter, no `Box<dyn>`, no
  forwarding wrapper, no feature flag and no file. It changed three existing
  private functions and one signature, and added three units to the existing
  in-file test module and none to a new test binary. The `WordFontSlot`
  argument threaded into `word_font_for_slot` replaces a second lookup rather
  than adding a parameter for its own sake.
- **contract, on the parts not listed above**. The bundled-font containment
  commit, the subset records, the licence and notice obligations, the 27-TTF
  inventory across `CLAUDE.md`, `.github/workflows/ci.yml`,
  `scripts/test_sprint_workflow.py` and
  `docs/hld/15-build-and-toolchain.md`, the archive staying under 10 MiB, the
  `#[non_exhaustive]` landing with the two new variants, the "no existing range
  moves" sweep at `crates/oxml-layout/src/font.rs:3105`, the deterministic font
  mode on every rendering assertion, the four named `## HLD impact` files all
  edited, and the unchanged hash harness and golden PNG baselines are all met
  as written.
