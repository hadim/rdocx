# F-266a, all, pass 4

**Reviewed**: `git diff 562af721..HEAD` in the worker worktree, the five F-266a
commits `a8bf6e5a` (bundled fonts, containment commit), `05cd97f5` (the
behaviour change), `e96b7ddc`, `31a95bc2` and `feb500fd` (the pass 1, pass 2 and
pass 3 remediations) reviewed as one delta. 22 files changed, 2886 insertions
and 72 deletions. Three of the 22 are binary TTFs and three are the pass 1, pass
2 and pass 3 review files, so 16 text files carry the reviewable diff. Aspects
run: correctness, contract, panics, ooxml, tests, structure.

**Verdict**: 0 defects, 2 smells, 6 nitpicks

Both smells are the same kind. `feb500fd` changed three behaviours and left the
comments, the doc comments and one test name describing the behaviour it
replaced, so three separate places in `crates/rdocx-layout/src/engine.rs` now
state the opposite of what the code does. Nothing in the delta computes a wrong
answer for any input I could construct.

## What was verified rather than trusted

- `cargo fmt --all --check`, clean.
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`, clean.
- `cargo test -p oxml-layout`, 107 pass plus 3 doctests. `--no-default-features`,
  105 pass plus 3.
- `cargo test -p rdocx-layout`, 289 pass plus the doctest.
- `cargo test -p rdocx --test integration_test --test regression_test`, 291 and
  516 pass.
- `cargo check --target wasm32-unknown-unknown -p rdocx-wasm -p rpptx-wasm`,
  clean.
- `python3 scripts/hash_harness.py --check`, 49 entries match.
- `python3 scripts/golden_png_harness.py --check`, 7 of 7 page-one pixel buffers
  match at 150 DPI, rasteriser reported as `pdftoppm version 26.01.0`.
- `python3 scripts/prose_check.py`, 0 violations.
- `python3 scripts/sync_agent_skills.py --check`, 26 skills in sync.
- The shared East Asian set at `crates/rdocx-layout/src/engine.rs:8313` and the
  declared list in the sweep at `crates/rdocx-layout/src/engine.rs:19430` were
  both replayed over the full codepoint space outside the test suite. The two
  sets are equal, exactly, with no codepoint on either side. Results feed the
  S1 status below.
- The old and new `needs_word_multilingual_layout` range lists were replayed the
  same way. The gate gained 65,273 codepoints and lost none.
- The rich path was driven through the public facade from a scratch binary
  outside the worktree, laying out `"Hello world"` beside `"Hello world"` plus
  one newly admitted character. Results feed the rich-path note below.

## Job 1, the state of every pass 3 finding

### Pass 3 S1, the three tables still did not all agree

**Resolved, and the sweep catches both directions.**
`crates/rdocx-layout/src/engine.rs:8313` is now the one place the question is
asked. The three callers are `word_font_slot` at
`crates/rdocx-layout/src/engine.rs:8362`, `word_language_slot` at
`crates/rdocx-layout/src/engine.rs:8570` and `needs_word_multilingual_layout`
at `crates/rdocx-layout/src/engine.rs:8679`.

Verified by replay rather than by reading:

- The declared list at `crates/rdocx-layout/src/engine.rs:19430` and
  `is_east_asian` compute the identical set. The declared list splits
  `0x3000..=0x33ff` into eight named blocks, `0x3400..=0x9fff` into three and
  `0xfe30..=0xfe6f` into two, and every split closes exactly. No codepoint is
  in one and not the other.
- The sweep at `crates/rdocx-layout/src/engine.rs:19456` asserts equality, not
  implication, at three points. `is_east_asian` against the declared list at
  `:19461`, the font table at `:19466` and the language table at `:19471`. A
  widening of `is_east_asian` fails at `:19461` on the first added codepoint,
  and a narrowing fails there on the first removed one. Both directions hold.
- No earlier match arm shadows the shared set in any of the three callers. The
  `ComplexScript` arm at `crates/rdocx-layout/src/engine.rs:8358` and the `Bidi`
  arm at `crates/rdocx-layout/src/engine.rs:8572` sit above it, and neither
  overlaps `is_east_asian` at any codepoint. That is what makes the font and
  language assertions in the sweep non-vacuous, since a future arm inserted
  above would fail them.
- `needs_word_multilingual_layout` is asserted one-directionally at
  `crates/rdocx-layout/src/engine.rs:19476`. That is correct rather than a gap,
  because the gate is deliberately a union of the shared set with the complex
  scripts, so equality would be the wrong assertion. The union half is covered
  separately at `crates/rdocx-layout/src/engine.rs:19487`.

The 65,241 codepoints pass 3 measured now reach the rich path. The recomputed
figure is 65,273, the difference being the 32 Devanagari Extended codepoints
added for pass 3 nitpick 3.

**On the widening itself.** Routing the whole shared set through the rich-path
gate is paragraph-wide, at `crates/rdocx-layout/src/engine.rs:6731`, so one
newly admitted character switches an entire otherwise Latin paragraph onto the
shaping path. I drove that through the facade. `"Hello world"` lays out as two
`PositionedElement::Text` runs with the second at x 97.663 and baseline y 80.25.
`"Hello world"` followed by `U+33A1`, `U+FF21` or `U+2E80` lays out as
`PositionedElement::MultilingualText` runs with the Latin half at exactly the
same x 97.663 and the same baseline y 80.25. The Latin geometry is preserved,
which is why the recorded sample digests and the golden PNGs did not move. This
is a behaviour change on documents that previously worked, but it is the change
pass 3 S1 asked for, it is required for the language slot to be reachable at
all, and it does not disturb the Latin text beside it. Not a finding.

### Pass 3 S2, a run with no alphabetic character lost its slot

**Resolved.** `crates/rdocx-layout/src/engine.rs:8407` separates the two cases.
A claimant now returns its own slot, and only the genuine no-claimant case
falls to the branch at `crates/rdocx-layout/src/engine.rs:8416`, where the
remaining characters decide. A run of East Asian punctuation resolves through
`w:eastAsia`, pinned at `crates/rdocx-layout/src/engine.rs:19570`, and so does a
run of fullwidth digits, pinned at
`crates/rdocx-layout/src/engine.rs:19574`. The three cases that must not move
are pinned beside them, ASCII digits at `:19577`, ASCII punctuation at `:19578`
and a letter outranking the punctuation beside it at `:19582`.

The second half is delivered. `docs/hld/08-rendering-spec.md:649` now states
that `w:hAnsi` is reachable only by a run with no ASCII letter in it, and gives
the reason. I checked the sentence against the code and it is accurate on both
paths, since a run of `"ÉTÉ"` has no ASCII letter and a run of `"—"` has no
letter at all.

The branch itself carries a new finding about how it decides, recorded as S2
below.

### Pass 3 S3, a non-Latin reference on `w:asciiTheme` resolved to nothing

**Resolved, and the fix does not reintroduce pass 2 D1.** The last resort at
`crates/rdocx-layout/src/engine.rs:8499` sits behind the whole
`if let Some(family)` at `crates/rdocx-layout/src/engine.rs:8493`, so it runs
only when the slot's own lookup and the `w:ascii` fallback have both declined.

I worked the ordering rather than trusting it. `word_font_for_slot` at
`crates/rdocx-layout/src/engine.rs:8441` returns an explicit family before it
reads any theme attribute, so an explicit `w:ascii` family always answers first
and the last resort is unreachable whenever one exists. The pass 2 D1 input,
`w:cstheme="minorBidi"` beside `w:ascii="Noto Sans Arabic"`, still resolves to
`Noto Sans Arabic`, and that exact input is now asserted at
`crates/rdocx-layout/src/engine.rs:19403`. The other side, that
`w:asciiTheme="majorEastAsia"` alone resolves to the Latin major face rather
than the engine default, is asserted for all four references at
`crates/rdocx-layout/src/engine.rs:19389`.

The one case where the last resort answers while an explicit family exists is a
run carrying `w:hAnsi` and nothing on `w:ascii`, since `w:hAnsi` is not in
Word's fallback chain and this engine never consulted it either. That is not a
regression, because before F-266a `resolve_font_family` read only `font_ascii`
and `font_ascii_theme` and answered `None` for that shape.

### Pass 3 S4, the hint never reached rendering

**Resolved for the run path.** `a_font_hint_reaches_the_rendered_font_family` at
`crates/rdocx/tests/regression_test.rs:30886` drives the attribute through
`Document::layout_deterministic`.

Verified load bearing by reading, as instructed, rather than by mutating. The
run text is `", , ,"`, so no character is alphabetic and the no-claimant branch
at `crates/rdocx-layout/src/engine.rs:8416` decides. With `hint` at `None`,
every character falls to the ambiguous arm at
`crates/rdocx-layout/src/engine.rs:8376`, takes `Ascii` by the
`codepoint <= 0x007f` guard at `crates/rdocx-layout/src/engine.rs:8379`, and the
run resolves to `Noto Sans SC`. With `hint` at `Some("eastAsia")` the same
characters take `WordFontSlot::EastAsia` at
`crates/rdocx-layout/src/engine.rs:8377` and the run resolves to
`Noto Sans JP`. Replacing `effective_rpr.font_hint.as_deref()` with `None` at
`crates/rdocx-layout/src/engine.rs:6084` makes both calls return `Noto Sans SC`,
which fails the second assertion at
`crates/rdocx/tests/regression_test.rs:30916`. The test is load bearing.

Coverage fallback cannot move either answer, as the test's doc comment claims. I
checked `crates/oxml-layout/fonts/SUBSET-NotoSansSC.md` and
`crates/oxml-layout/fonts/SUBSET-NotoSansJP.md` and both approve ASCII comma and
space.

The text has no East Asian character, so `needs_word_multilingual_layout` is
false and the paragraph takes the legacy path. The test therefore reaches only
the first of the two production call sites. The second, at
`crates/rdocx-layout/src/engine.rs:6365`, is the field display-segment path and
is still unproven. Nitpick 1 below.

### The five pass 3 nitpicks

1. The 105-column line at `docs/hld/15-build-and-toolchain.md:272`. **Fixed.**
   The paragraph is rewrapped across `docs/hld/15-build-and-toolchain.md:272` to
   `:275` and no line in it exceeds 80 columns.
2. The 10 MB against 5 MB contradiction. **Partially fixed.**
   `docs/hld/15-build-and-toolchain.md:476` now reads "several megabytes a full
   family weighs" with the measured figures attributed to the upstream sources
   at the time of retrieval, which is internally consistent and no longer over
   80 columns. `.claude/plans/F-266a-design.md:197` still says "Each is roughly
   5 MB", so the two documents still disagree. Nitpick 5 below.
3. The false "union of every range" claim. **Fixed.** The doc comment at
   `crates/rdocx-layout/src/engine.rs:8666` no longer makes the claim, and
   `0xa8e0..=0xa8ff` is admitted at `crates/rdocx-layout/src/engine.rs:8688` so
   the narrower claim it does make is true. Pinned at
   `crates/rdocx-layout/src/engine.rs:19487`.
4. Numbering markers bypassing slot resolution. **Still open.**
   `crates/rdocx-layout/src/engine.rs:5910` still reads
   `marker_rpr.font_ascii.as_deref()` and never calls `resolve_font_family`.
   Pre-existing and outside the diff, carried forward as nitpick 4.
5. The IPA and modifier-letter gap. **Fixed.** The `HighAnsi` arm at
   `crates/rdocx-layout/src/engine.rs:8370` is widened to `0x00c0..=0x02ff`, so
   a hint can no longer move those two blocks. The widening changes nothing
   without a hint, since both blocks already took `w:hAnsi` from the ambiguous
   arm's `codepoint <= 0x007f` guard.

## Job 2, new findings in the pass 3 remediation

## Defects

None. Every behaviour `feb500fd` changed answers correctly for every input I
could construct, including the schema-valid inputs Word never writes, and the
Latin geometry beside a newly admitted character is unmoved.

## Smells

### S1, the theme last resort is documented as impossible in the three places a reader will meet it

`crates/rdocx-layout/src/engine.rs:8483`, with
`crates/rdocx-layout/src/engine.rs:8470` and
`crates/rdocx-layout/src/engine.rs:19261`

The branch added at `crates/rdocx-layout/src/engine.rs:8499` is correct. What
surrounds it is not.

`resolve_font_family`'s doc comment at
`crates/rdocx-layout/src/engine.rs:8483` states the function's priority list in
full, "the slot's explicit family, then the slot's theme font, then the same two
for the `w:ascii` slot, then None so the default applies". There are now five
steps, not four, and the fifth begins eleven lines below the sentence that says
the fourth is the last. A reader answering "what does this do?" from this one
function is told the wrong answer by its own header.

`word_font_for_slot`'s comment at `crates/rdocx-layout/src/engine.rs:8470` goes
further and forbids the new branch. It says the honest answer for the four
non-Latin references is `None`, and that "Answering with the Latin typeface
would be worse than declining, because it is a face that usually cannot draw the
text". Twenty-nine lines later the caller answers with exactly that face, on
purpose, for exactly those four values.

The test is the worst of the three, because a test name is a contract statement.
`east_asia_and_bidi_theme_references_never_answer_with_the_latin_typeface` at
`crates/rdocx-layout/src/engine.rs:19261` now contains, at
`crates/rdocx-layout/src/engine.rs:19389`, an assertion that
`font_ascii_theme: Some("majorEastAsia")` resolves to `"MajorFace"`, which is
the Latin major typeface. Its doc comment at
`crates/rdocx-layout/src/engine.rs:19249` repeats "the honest answer is None".
The name, the doc comment and the body of one test now disagree with each other.

This is how the next reader reverts the fix. The pass 2 and pass 3 reviews each
had to reconstruct this rule from scratch, and the three statements above are
what they would read first. The remedy is small, since
`docs/hld/08-rendering-spec.md` already carries the correct rule and only the
three in-file statements lag.

### S2, the no-claimant branch decides by position, and the function's own doc comment still says the opposite

`crates/rdocx-layout/src/engine.rs:8418`, with
`crates/rdocx-layout/src/engine.rs:8391` and
`crates/rdocx-layout/src/engine.rs:8404`

The letter loop requires consensus. Two letters that disagree return
`WordFontSlot::Ascii` at `crates/rdocx-layout/src/engine.rs:8404`, and the whole
point of that rule is that a run's answer must not depend on which character
happened to come first. The no-claimant branch applies the opposite rule. `find`
at `crates/rdocx-layout/src/engine.rs:8418` takes the first character whose slot
is not `Ascii` and stops, so a run of non-letters that disagree answers by
position.

Concretely, both characters below are non-alphabetic, so both reach the branch.
`U+3001` is East Asian at `crates/rdocx-layout/src/engine.rs:8362` and `U+2014`
is `HighAnsi` from the ambiguous arm at
`crates/rdocx-layout/src/engine.rs:8380`. So
`word_font_slot_for_text("\u{3001}\u{2014}", None)` is `EastAsia` and
`word_font_slot_for_text("\u{2014}\u{3001}", None)` is `HighAnsi`. Same two
characters, same run, two different families, decided by which one the author
typed first. A Japanese run of `"「」 — "` and one of `"— 「」"` are both
ordinary shapes, and Word draws both from two slots, so neither answer is
wholly right, but the engine's answer should not turn on ordering when the rule
twelve lines above it exists precisely to stop that.

The doc comment was not updated with the branch. Lines
`crates/rdocx-layout/src/engine.rs:8391` to `:8392` still say "Spaces, digits
and punctuation take whatever the rest of the run takes, so they must not decide
it", which is now false for every run without a letter.
`docs/hld/08-rendering-spec.md:637` was updated and says the right thing, so the
spec and the code agree and only the code's own comment lags.

Either the branch takes the same consensus rule the letter loop takes, or both
the comment and the branch state that the first non-ASCII character wins and
why that is acceptable.

## Nitpicks

- `crates/rdocx-layout/src/engine.rs:6365`, the field display-segment call site
  passes `segment_rpr.font_hint.as_deref()` and no test drives a hint through
  it. The new regression covers only
  `crates/rdocx-layout/src/engine.rs:6084`, so pass 3 S4's "both call sites" is
  closed on one.
- `crates/rdocx-layout/src/engine.rs:19456`, the sweep stops at `0x2fa1f`, so
  the constant's claim at `crates/rdocx-layout/src/engine.rs:19427` that a range
  added to `is_east_asian` and forgotten fails is true only below that
  codepoint. A future CJK extension G range at `0x30000` would be added and
  never swept.
- `crates/rdocx-layout/src/engine.rs:8370` against
  `crates/rdocx-layout/src/engine.rs:8573`, the `HighAnsi` widening to
  `0x02ff` was not mirrored in the language table's `Direct` arm, which still
  stops at `0x024f`. This is the same one-sided-widening shape pass 2 S1 and
  pass 3 S1 both found, and the sweep covers only the East Asian arms. It is a
  nitpick rather than a smell because an unclaimed character attaches to the
  preceding language range, which for a Latin word is `Direct` either way, so
  no reachable document changes.
- `crates/rdocx-layout/src/engine.rs:5910`, numbering markers still read
  `marker_rpr.font_ascii` directly and never pass through
  `resolve_font_family`. Carried from pass 3 nitpick 4, pre-existing and outside
  the diff.
- `.claude/plans/F-266a-design.md:197` against
  `docs/hld/15-build-and-toolchain.md:476`, "Each is roughly 5 MB" still
  disagrees with the measured 9.6 MB and 10.4 MB. Carried from pass 3 nitpick 2.
  The spec text is now internally consistent and the plan is a frozen record, so
  this is worth noting and not worth editing.
- `crates/rdocx-layout/src/engine.rs:8419`, a run of only Latin-1 punctuation,
  `"«»"` or a lone non-breaking space, now takes `w:hAnsi` where it took
  `w:ascii` before `feb500fd`. This matches Word and is bounded by `w:hAnsi`
  falling back to the `w:ascii` family, but it is a behaviour change on an
  ordinary run shape and no test names it.

## Not found

- **panics**. Nothing in the production half of `feb500fd` can panic. It added
  one `matches!` over `u32`, one `find` over an iterator of `Copy` values, one
  `if let` early return and one `match` on `&str` behind two `?` operators on
  `Option`. No new `unwrap`, `expect`, slicing, indexing or arithmetic on
  untrusted input outside `#[cfg(test)]`. The two `unwrap_or` calls at
  `crates/rdocx-layout/src/engine.rs:8419` and the `map(str::to_owned)` at
  `crates/rdocx-layout/src/engine.rs:8518` are total. `character as u32` is
  infallible, and `char::from_u32` is checked at
  `crates/rdocx-layout/src/engine.rs:19457`.
- **ooxml**. Nothing checked in this aspect produced a finding. `feb500fd` adds
  no parser and no serialiser, changes no element or attribute order and drops
  no unmodelled subtree. The `ST_Theme` literals matched at
  `crates/rdocx-layout/src/engine.rs:8514` are the spec's exact spellings, so
  case-sensitive matching is right, and they are the same four the declining arm
  at `crates/rdocx-layout/src/engine.rs:8476` names.
- **structure**. Nothing checked in this aspect produced a finding. `feb500fd`
  adds one private function, `is_east_asian`, and that is the rule's good case
  rather than its bad one. It removes three duplicated range lists and leaves
  one, so it reduces the number of places a reader must look. It adds no type,
  no trait, no generic parameter, no `Box<dyn>`, no forwarding wrapper, no
  feature flag, no module and no file.
- **contract**. Nothing checked in this aspect produced a finding beyond the
  documentation drift recorded as S1 and S2. Every item in the plan's
  `## Test plan`, `## HLD impact`, `## Risk routing` and `## Hash harness`
  sections that pass 3 verified is unmoved by `feb500fd`, and the four named
  HLD files remain edited. The hash harness and the golden PNG baselines are
  unchanged at 49 of 49 and 7 of 7.
- **correctness, on the shared East Asian set**. Replayed exhaustively against
  the declared list and against all three callers. No disagreement at any
  codepoint, in either direction, and no earlier match arm shadows it.
- **correctness, on the theme last resort**. Worked through every combination of
  the four slots against explicit families, slot theme references and the
  `w:ascii` theme reference. The branch is unreachable whenever any explicit
  family or any resolving Latin reference exists, so pass 2 D1 is not
  reintroduced.
