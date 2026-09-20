# F-266a, all, pass 3

**Reviewed**: `git diff 562af721..HEAD` in the worker worktree, the four F-266a
commits `a8bf6e5a` (bundled fonts, containment commit), `05cd97f5` (the
behaviour change), `e96b7ddc` (the pass 1 remediation) and `31a95bc2` (the pass
2 remediation) reviewed as one delta. 21 files changed, 2254 insertions and 60
deletions. Three of the 21 are binary TTFs and two are the pass 1 and pass 2
review files, so 16 text files carry the reviewable diff. Aspects run:
correctness, contract, panics, ooxml, tests, structure.

**Verdict**: 0 defects, 4 smells, 5 nitpicks

## What was verified rather than trusted

- `cargo fmt --all --check`, clean.
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`, clean.
- `cargo test -p oxml-layout`, 107 pass. `--no-default-features`, 105 pass.
- `cargo test -p rdocx-layout`, 289 pass plus the doctest.
- `cargo test -p rdocx --test integration_test --test regression_test`, 291 and
  515 pass.
- `cargo check --target wasm32-unknown-unknown -p rdocx-wasm -p rpptx-wasm`,
  clean.
- `python3 scripts/hash_harness.py --check`, 49 entries match.
- `python3 scripts/golden_png_harness.py --check`, 7 of 7 page-one pixel buffers
  match at 150 DPI, rasteriser reported as `pdftoppm version 26.01.0`. The
  worktree-aware shim is the one pass 2 examined and judged legitimate.
- `python3 scripts/prose_check.py`, 0 violations.
- `python3 scripts/sync_agent_skills.py --check`, 26 skills in sync.
- The four codepoint tables were re-derived exhaustively outside the test suite,
  by replaying every range literal at
  `crates/rdocx-layout/src/engine.rs:8334`, `engine.rs:8516`,
  `engine.rs:8635` and `crates/oxml-layout/src/font.rs:1787` over the full
  codepoint space. Results are reported under S1 and are the basis for the pass
  2 S1 status below.
- `crates/rdocx/tests/integration_test.rs` is untouched by `31a95bc2`, so the
  recorded geometry digest
  `516ebb6e45438731d3cb0983707ad00c9de55068401e073ef2a069a56f397402` survived a
  production classification change unchanged. That is the strongest evidence
  available that no sample or fixture geometry moved.

## Job 1, the state of every pass 2 finding

### Pass 2 D1, the four non-Latin theme references resolved to the Latin typeface and outranked `w:ascii`

**Resolved.** `crates/rdocx-layout/src/engine.rs:8444`. The match on
`theme_ref` now answers only for `majorAscii`, `majorHAnsi`, `minorAscii` and
`minorHAnsi`, and every other value including the four non-Latin references
falls to `_ => None`. `resolve_font_family` at
`crates/rdocx-layout/src/engine.rs:8461` then falls through to the `Ascii`
slot, so the input pass 2 named, `w:cstheme="minorBidi"` beside
`w:ascii="Noto Sans Arabic"`, now resolves to `Noto Sans Arabic`. That case is
pinned directly at `crates/rdocx-layout/src/engine.rs:8461` through the unit at
`crates/rdocx-layout/src/engine.rs:19311`, which walks all four slots and
requires the author's family every time.

**On the plan wording.** The plan said the four should "read their own theme
entry instead of collapsing onto the ASCII font". `rdocx_oxml::theme::Theme`
carries only `major_font` and `minor_font`, both filled from `<a:latin>`, so the
entry the plan names does not exist in the model and creating it is a parser
change the plan's risk routing explicitly does not match. Between the two
remaining options, answering with a Latin face or declining, declining is the
right one: it is what Word produces for `minorBidi` against a stock theme whose
`<a:cs>` is empty, and it is the only option that does not let a face the
reference never named outrank an explicit family. The resolution is correct.

**On renaming the plan's named test.** The plan named
`east_asia_and_bidi_theme_references_read_their_own_theme_entry`. The landed
test is `east_asia_and_bidi_theme_references_never_answer_with_the_latin_typeface`
at `crates/rdocx-layout/src/engine.rs:19222`. This is an acceptable deviation.
The plan's name asserts a fact that is not true of the code and cannot be made
true without the excluded parser change, and a test whose name promises an
entry that does not exist is worse than one whose name states what holds. The
deviation is declared in the commit message for `31a95bc2` and the reasoning is
carried in the test's own doc comment at
`crates/rdocx-layout/src/engine.rs:19211`. The substance the plan asked for,
that the four stop collapsing onto whatever `w:asciiTheme` named, is delivered
and tested.

One consequence of the new `None` answer is a narrow regression, recorded below
as S3.

### Pass 2 D2, `w:rFonts/@w:hint` was inert for every character a real document uses it on

**Resolved.** `crates/rdocx-layout/src/engine.rs:8356`. The `HighAnsi` arm at
`crates/rdocx-layout/src/engine.rs:8350` was cut from `0x00a0..=0x058f |
0x1e00..=0x2bff` down to `0x00c0..=0x024f | 0x1e00..=0x1eff`, and the `Ascii`
arm at `crates/rdocx-layout/src/engine.rs:8347` from the whole seven-bit range
down to the Latin letters. Everything the two arms released now reaches the
hint arm.

Checked one category at a time against the current table:

- General punctuation `U+2000..U+206F`, including `U+2013`, `U+2014`, `U+2018`
  to `U+201D`, `U+2022` and `U+2026`, reaches the hint arm. No arm above it
  claims `0x2000..=0x2bff` any more.
- Enclosed alphanumerics `U+2460..U+24FF`, geometric shapes `U+25A0..U+25FF`
  and miscellaneous symbols `U+2600..U+26FF` likewise.
- Digits `U+0030..U+0039` reach it, since `Ascii` now claims only
  `0x0041..=0x005a | 0x0061..=0x007a`.
- Greek `U+0370..U+03FF` and Cyrillic `U+0400..U+04FF` reach it, since
  `HighAnsi` now stops at `U+024F`.

The unit at `crates/rdocx-layout/src/engine.rs:19170` exercises exactly that
set, `U+2014`, `U+2022`, `U+2026`, `U+24FF`, `U+25A0`, `U+03B1`, `U+0410`, `7`
and `U+0085`, and requires each to follow both an `eastAsia` and a `cs` hint.

No character with a real script identity can be moved by a hint. The three arms
above the hint arm return before `hint` is ever read, and they cover every
codepoint `script_for_char` gives a non-Common identity: Latin
`0x0041..=0x024f` and `0x1e00..=0x1eff` split across the `Ascii` and `HighAnsi`
arms, Hebrew, Arabic, Devanagari and Thai inside the `ComplexScript` arm, and
Hangul, Kana and Han inside the `EastAsia` arm, which I confirmed by replay is a
strict superset of the script table's East Asian set. The unit at
`crates/rdocx-layout/src/engine.rs:19199` pins ten of those against all four
hint values.

The one residual, that a run containing no alphabetic character cannot follow
its hint at all, comes from the pass 2 S2 remediation rather than this one and
is recorded as S2 below.

### Pass 2 S1, the four codepoint tables disagree

**Partially resolved.** Two of the three claims hold. The third does not, and
the residual is carried forward as pass 3 S1.

- **The font and language tables now agree exactly.** Verified by replaying both
  range lists, `crates/rdocx-layout/src/engine.rs:8334` and
  `crates/rdocx-layout/src/engine.rs:8516`, over every codepoint. The two East
  Asian sets are byte for byte the same twelve ranges and the computed sets are
  equal. Halfwidth Hangul jamo `U+FFA0..U+FFDC`, Bopomofo, Kanbun, the CJK
  strokes, the enclosed and compatibility blocks, fullwidth Latin and
  supplementary-plane Han all now return `Some(WordLanguageSlot::EastAsia)`, so
  the concrete failure pass 2 named, halfwidth Korean attaching to the preceding
  direct slot inside a rich paragraph, is gone.
- **Halfwidth Hangul jamo has a script identity and reaches the rich path.**
  `crates/oxml-layout/src/font.rs:1787` adds `0xffa0..=0xffdc` to the Hangul
  arm and `crates/rdocx-layout/src/engine.rs:8650` adds it to
  `needs_word_multilingual_layout`. The sweep at
  `crates/oxml-layout/src/font.rs:3106` still passes, because the range was
  `TextScript::Common` before and the sweep pins only the previously mapped
  ranges.
- **The sweep does not catch a one-sided widening in general.** The loop at
  `crates/rdocx-layout/src/engine.rs:19388` proves one direction, font implies
  language. It would catch a widening of `word_font_slot` alone, which is the
  widening that created pass 2 S1. It would not catch a widening of
  `word_language_slot` alone, and it does not catch the disagreement that
  exists in the tree right now between those two tables and
  `needs_word_multilingual_layout`. See S1.

### Pass 2 S2, the whole run takes one slot, so Latin inside a mixed run resolved through `w:eastAsia` or `w:cs`

**Resolved.** `crates/rdocx-layout/src/engine.rs:8379`. The loop now filters to
`character.is_alphabetic()`, takes the first claimant's slot, and returns
`WordFontSlot::Ascii` the moment a second claimant disagrees. The regression is
gone: `"Hello 世界"` returns `Ascii` at
`crates/rdocx-layout/src/engine.rs:19486`, so a run carrying `w:ascii="Arial"`
beside `w:eastAsia="MS Mincho"` draws from Arial again rather than newly
breaking the Latin half.

Single-script cases are intact, and the gate proves it rather than the unit
alone. Every fixture paragraph in the test gate is single-script once
non-alphabetic characters are discounted: `HEBREW` at
`crates/rdocx/tests/integration_test.rs:18690` is Hebrew letters and one space,
`JAPANESE` at `crates/rdocx/tests/integration_test.rs:18692` is Kana and Han
and one ideographic comma, `KANJI` at
`crates/rdocx/tests/integration_test.rs:18693` is two Han characters. The
`KANJI` paragraph is the one that makes the gate load bearing, because it names
`Noto Sans SC` on `w:ascii` and `Noto Sans JP` on `w:eastAsia` and both subsets
cover `U+4E16` and `U+754C`, so coverage fallback cannot choose between them.
It still resolves through `EastAsia`, and the family assertion at
`crates/rdocx/tests/integration_test.rs:18942` still requires `Noto Sans JP`.
Reverting slot resolution alone still fails it. The gate is still load bearing.

The divergence is named with its cost at
`docs/hld/08-rendering-spec.md:633`, "a mixed Latin and East Asian run is a
known divergence whose cost is that the East Asian half keeps the `w:ascii`
family", together with why the alternative is worse. That is what rule 5 of
`.claude/skills/differential-testing.md` asks for, and it is met. A second
consequence of the same rule is not named there and is recorded as S2.

### The four pass 2 nitpicks

1. The `F-266a handoff` pointer. **Fixed** at
   `crates/rdocx/tests/regression_test.rs:30796`, which now cites
   `docs/hld/14-development-backlog.md`.
2. The `docs/hld/15-build-and-toolchain.md:273` rewrap orphan. **Still open in a
   new form.** The orphan is gone, but the join produced a 105-column line at
   `docs/hld/15-build-and-toolchain.md:272` in a file whose every other prose
   line is at or below 80. Nitpick 1 below.
3. The 10 MB against 5 MB contradiction. **Still open.**
   `docs/hld/15-build-and-toolchain.md:476` now says "9.6 MB and 10.4 MB their
   full families weigh", and `.claude/plans/F-266a-design.md` under
   `## Rejected alternatives` still says "Each is roughly 5 MB". The spec set
   and the plan still disagree, by more than they did before. Nitpick 2 below.
4. The unclaimed `0x0080..=0x009f` and the choice of `U+0085` as the only hint
   test character. **Resolved.** The ambiguous arm at
   `crates/rdocx-layout/src/engine.rs:8356` now receives every codepoint no arm
   claims, the C1 controls take `w:hAnsi` by the `codepoint <= 0x007f` guard at
   `crates/rdocx-layout/src/engine.rs:8359`, and the hint unit at
   `crates/rdocx-layout/src/engine.rs:19170` exercises eight real characters
   beside `U+0085`.

## Job 2, new findings in the pass 2 remediation

## Defects

None. Every behaviour change in `31a95bc2` that I could trace to a document
shape a real producer emits is either correct or unchanged. The four items
below are wrong only for inputs that are schema-valid and facade-reachable but
not what Word writes, or are gaps in what the gate proves rather than in what
the code does, which is why they are smells.

## Smells

### S1, the four tables still do not all agree, and the sweep's own doc comment says they must

`crates/rdocx-layout/src/engine.rs:19335`, with
`crates/rdocx-layout/src/engine.rs:19346` and
`crates/rdocx-layout/src/engine.rs:8635`

The test is named
`every_east_asian_codepoint_agrees_across_the_language_slot_and_font_slot_tables`
and its doc comment opens "The four codepoint tables must agree about East
Asian text". The constant's comment at
`crates/rdocx-layout/src/engine.rs:19346` promises more, "a range added to one
of them and forgotten in another fails". Neither statement is true of what the
test proves.

I replayed all four range lists over the full codepoint space. The font table
at `crates/rdocx-layout/src/engine.rs:8334` and the language table at
`crates/rdocx-layout/src/engine.rs:8516` are equal, which is the improvement.
`needs_word_multilingual_layout` at `crates/rdocx-layout/src/engine.rs:8635` is
not. 65,241 codepoints are East Asian to both of the first two tables and never
reach the rich path:

```
U+2E80..U+2FDF   CJK radicals supplement and Kangxi radicals
U+2FF0..U+2FFF   ideographic description characters
U+3100..U+312F   Bopomofo
U+3190..U+31EF   Kanbun, Bopomofo extended, CJK strokes
U+3200..U+33FF   enclosed CJK and CJK compatibility
U+FE30..U+FE6F   CJK compatibility forms and small form variants
U+FF00..U+FF65   fullwidth ASCII and Latin
U+FFDD..U+FFEF   halfwidth and fullwidth tail
U+20000..U+2FA1F CJK unified ideographs extensions B and beyond
```

A paragraph made only of those, the plainest case being a line of CJK extension
B ideographs or a fullwidth Latin heading, never enters
`word_language_ranges`, so it takes `w:lang/@w:val` where Word takes
`@w:eastAsia`. That is the exact sentence the test's own doc comment uses to
justify its existence, still true, one table over.

This is not a pre-existing condition that the remediation merely failed to fix.
Before `31a95bc2` the language table's East Asian set was a subset of
`needs_word_multilingual_layout`, so the two agreed in the direction that
matters. The remediation widened the language table by nine ranges and left
`needs_word_multilingual_layout` alone except for halfwidth Hangul jamo, which
created the new gap. The sweep cannot see it, because it only asserts
`needs_word_multilingual_layout` over `EAST_ASIAN_SCRIPTS` at
`crates/rdocx-layout/src/engine.rs:19350`, a hand-written list that is a strict
subset of both widened tables.

Either `needs_word_multilingual_layout` grows to the same East Asian set, or the
test's name and both comments are corrected to state the two-table invariant
they actually prove and the third table's narrower contract is written down
where a reader will meet it.

### S2, a run with no alphabetic character loses its slot, so East Asian punctuation and fullwidth digits resolve through `w:ascii`

`crates/rdocx-layout/src/engine.rs:8379`, with
`crates/rdocx-layout/src/engine.rs:8387` and the assertion at
`crates/rdocx-layout/src/engine.rs:19467`

`char::is_alphabetic` is the right predicate for the job it was given. It is the
Unicode `Alphabetic` derived property, so it is true for Han, Kana, Hangul,
Hebrew, Arabic, Thai and Devanagari letters, true for the Kana prolonged sound
mark `U+30FC` and for fullwidth Latin letters, and false for spaces, digits and
the CJK punctuation that should not decide a run's slot. As a filter it does
what the comment at `crates/rdocx-layout/src/engine.rs:8371` claims.

The problem is the fallback beside it. `claimed.unwrap_or(WordFontSlot::Ascii)`
at `crates/rdocx-layout/src/engine.rs:8387` applies the same answer to a run
with a disagreement and to a run with no claimant at all, and those are
different situations. A run that is entirely East Asian punctuation, `「」`, `、`,
`・`, or entirely fullwidth digits, `２０２６`, has nothing to take "whatever the
rest of the run takes" from, and Word draws every one of those from
`w:eastAsia`. Word splits runs at formatting, proofing and revision boundaries,
so a run of exactly this shape is ordinary in Japanese and Chinese prose. The
behaviour is pinned rather than incidental: the unit at
`crates/rdocx-layout/src/engine.rs:19467` requires
`word_font_slot_for_text("、。 2026 「」", None)` to be `Ascii`. Before
`31a95bc2` that run resolved through `EastAsia`, so this is a narrowing the
remediation introduced.

The same fallback has a second consequence that
`docs/hld/08-rendering-spec.md:625` does not name. `w:hAnsi` is now unreachable
for almost all Western European text, because an accented word nearly always
carries unaccented ASCII letters too and the two disagree. `"ÉTÉ"` claims
`HighAnsi`, `Ascii`, `HighAnsi` and therefore resolves through `Ascii`. The
spec paragraph records the cost for the mixed Latin and East Asian case only,
and a reader will take from it that `w:hAnsi` became live in this story.

The cheap remedy for the first half is to distinguish the two cases: when no
alphabetic character claimed a slot, classify over every character rather than
returning `Ascii` unconditionally. The second half needs a sentence in
`docs/hld/08-rendering-spec.md` or an F-ID.

### S3, `w:asciiTheme` and `w:hAnsiTheme` carrying a non-Latin reference now resolve to no family at all

`crates/rdocx-layout/src/engine.rs:8444`, with
`crates/rdocx-layout/src/engine.rs:8461` and `crates/rdocx/src/run.rs:683`

`_ => None` is the right answer when the reference arrives on `w:eastAsiaTheme`
or `w:cstheme`, because the run then falls through to its own `w:ascii` family,
which is what Word produces. It is not the right answer when the same value
arrives on `w:asciiTheme` or `w:hAnsiTheme`, because there is nothing left to
fall through to. `resolve_font_family` at
`crates/rdocx-layout/src/engine.rs:8461` retries the `Ascii` slot, which for an
`Ascii` slot is the same call, so the result is `None` and the run takes the
engine default rather than the theme's typeface.

Before this diff, `resolve_font_family` handled all eight `ST_Theme` values on
`w:asciiTheme` and returned the major or minor Latin face for every one of
them. So a run whose only font property is `w:asciiTheme="minorEastAsia"`
rendered in the theme's minor typeface before and renders in the fallback
`Calibri` at `crates/rdocx-layout/src/engine.rs:8206` now. That is a silent
font change on a document that worked.

The input is schema-valid, since `ST_Theme` admits all eight values in all four
attributes, and it is reachable through the facade without validation:
`Run::set_slot_theme_font` at `crates/rdocx/src/run.rs:683` writes whatever
string it is handed into the slot's theme field. It is not a shape Word emits,
which is why this is a smell rather than a defect, but nothing in the diff
records the narrowing and no test covers either side of it.

The remedy that keeps the pass 2 D1 fix intact is to move the Latin answer
behind the `w:ascii` fallback rather than deleting it, so a non-Latin reference
still yields the Latin typeface as a last resort but can never outrank an
explicit family.

### S4, the hint never reaches rendering in any test, so both call sites could drop it silently

`crates/rdocx-layout/src/engine.rs:6084`, with
`crates/rdocx-layout/src/engine.rs:6365` and
`crates/rdocx-layout/src/engine.rs:19162`

`a_font_hint_decides_only_the_ambiguous_slot` calls `word_font_slot` and
`resolve_font_family` directly. Production calls neither with a character.
It calls `word_font_slot_for_text` at
`crates/rdocx-layout/src/engine.rs:6084` and
`crates/rdocx-layout/src/engine.rs:6365`, and every test call to
`word_font_slot_for_text` passes `None` for the hint, at
`crates/rdocx-layout/src/engine.rs:19451`, `:19455`, `:19459`, `:19462`,
`:19467`, `:19471`, `:19475`, `:19486` and `:19490`. Nothing in
`crates/rdocx/tests/` exercises `w:rFonts/@w:hint` through layout either. The
three hits at `crates/rdocx/tests/integration_test.rs:16355`, `:16526` and
`:16545` are F-265 round-trip assertions on the saved XML.

Replacing `effective_rpr.font_hint.as_deref()` with `None` at both call sites
would leave the whole suite green. The story's headline claim is that a parsed
attribute stopped being ignored, and the gate does not prove the attribute
reaches a rendered font family. One unit calling `word_font_slot_for_text` with
a Greek or Cyrillic run and `Some("eastAsia")`, or one paragraph in the
existing regression entrypoint, closes it.

## Nitpicks

- `docs/hld/15-build-and-toolchain.md:272`, the pass 2 orphan was closed by
  joining the lines into a 105-column line in a file wrapped at 80 everywhere
  else.
- `docs/hld/15-build-and-toolchain.md:476`, "9.6 MB and 10.4 MB their full
  families weigh" now contradicts `.claude/plans/F-266a-design.md` "Each is
  roughly 5 MB" by more than the pass 2 text did, and the line is 82 columns.
- `crates/rdocx-layout/src/engine.rs:8622`, "This is the union of every range
  `script_for_char` gives a non-Latin script identity" is false for
  `U+A8E0..U+A8FF`, the Devanagari Extended block, which
  `crates/oxml-layout/src/font.rs:1777` calls Devanagari and
  `needs_word_multilingual_layout` does not admit. Thirty-two codepoints, all
  combining marks and digits that in practice accompany a base Devanagari
  letter, so the claim is wrong without being harmful.
- `crates/rdocx-layout/src/engine.rs:5910`, numbering markers read
  `marker_rpr.font_ascii` directly and never pass through `resolve_font_family`,
  so a Korean or Japanese list whose `w:eastAsia` names the face that can draw
  its bullet still does not use it. Pre-existing and outside the diff, but it is
  the bullet half of the case pass 2 D2 described and the story added a
  numbering-marker regression test without touching it.
- `crates/rdocx-layout/src/engine.rs:8350`, the Latin arm omits
  `0x0250..=0x02af`, the IPA extensions, and `0x02b0..=0x02ff`, the spacing
  modifier letters. Both are alphabetic, both are Latin by any reading, and both
  now sit in the ambiguous set where an `eastAsia` hint can move them.

## Not found

- **panics**. Nothing in the production diff can panic. `31a95bc2` added one
  `for` loop with an early return, one `Option` state machine, a guard arm on an
  integer comparison and two match arms. No new `unwrap`, `expect`, slicing,
  indexing or arithmetic on untrusted input outside `#[cfg(test)]`. The
  `unwrap_or` at `crates/rdocx-layout/src/engine.rs:8387` is a total function on
  `Option`. `character as u32` is infallible. The two `?` operators at
  `crates/rdocx-layout/src/engine.rs:8429` and `engine.rs:8430` are early
  returns.
- **ooxml**. Nothing checked in this aspect produced a finding. The delta adds
  no parser and no serialiser and changes no element or attribute order. The
  hint values matched at `crates/rdocx-layout/src/engine.rs:8357` are the
  `ST_Hint` literals and the theme values matched at
  `crates/rdocx-layout/src/engine.rs:8434` are `ST_Theme` literals, so
  exact-case matching is right in both. No unmodelled subtree is dropped, and
  the round-trip assertion at
  `crates/rdocx/tests/integration_test.rs:19034` still proves the saved bytes
  stay in logical order.
- **structure**. Nothing checked in this aspect produced a finding. `31a95bc2`
  added no type, no trait, no generic parameter, no `Box<dyn>`, no forwarding
  wrapper, no feature flag, no module and no file. It changed the bodies of four
  existing private functions, renamed one test and added one. Nothing in the
  whole four-commit delta introduces a new crate, module or file beyond the
  three fonts and three subset records the plan names.
- **contract, on everything not listed above**. The bundled-font containment
  commit, the subset records, the licence and notice obligations, the 27-TTF
  inventory, the archive size, the `#[non_exhaustive]` landing with the two new
  variants, the "no existing range moves" sweep at
  `crates/oxml-layout/src/font.rs:3106`, the deterministic font mode on every
  rendering assertion, the four named `## HLD impact` files all edited, and the
  unchanged hash harness and golden PNG baselines are all met as written, and
  none of them moved in `31a95bc2`.
- **correctness, on the theme resolution order**. The order at
  `crates/rdocx-layout/src/engine.rs:8461`, slot explicit, then slot theme, then
  ascii explicit, then ascii theme, is right and is pinned from both directions,
  at `crates/rdocx-layout/src/engine.rs:19311` for the explicit family and at
  `crates/rdocx-layout/src/engine.rs:19325` for a resolving slot theme
  reference outranking the ascii fallback.
