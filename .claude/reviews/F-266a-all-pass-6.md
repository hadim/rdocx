# F-266a, all, pass 6

**Reviewed**: `git diff 562af721..HEAD` in the worker worktree, the seven
F-266a commits `a8bf6e5a`, `05cd97f5`, `e96b7ddc`, `31a95bc2`, `feb500fd`,
`9e03c7ec` and `8862a16f` reviewed as one delta. 24 files changed, 3694
insertions and 73 deletions. Three of the 24 are binary TTFs and five are the
pass 1 to pass 5 review files, so 16 text files carry the reviewable diff. The
new commit `8862a16f` is 3 files, 397 insertions and 11 deletions, of which
`crates/rdocx-layout/src/engine.rs` is 63 insertions and 1 deletion,
`docs/hld/08-rendering-spec.md` is 22 insertions and 10 deletions, and the rest
is the pass 5 review file. Aspects run: correctness, contract, panics, ooxml,
tests, structure.

**Verdict**: 0 defects, 0 smells, 6 nitpicks

Both pass 5 smells are closed, both of its actionable nitpicks are closed, and
`8862a16f` introduces nothing that computes a wrong answer for any input I
could construct. The three nitpicks pass 5 carried forward are carried forward
again for the same reasons, and one new nitpick is a count word left stale in
the paragraph this commit edited. **Zero defects and zero smells is the honest
result here, not a rounding of the findings.**

## What was verified rather than trusted

- `cargo fmt --all --check`, clean.
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`, clean.
- `cargo test -p rdocx-layout`, 290 pass plus the doctest, one more than pass 5,
  which is the single new test.
- `cargo test -p oxml-layout`, 107 pass plus 3 doctests. `--no-default-features`,
  105 pass plus 3.
- `cargo test -p rdocx --test integration_test --test regression_test`, 291 and
  516 pass.
- `cargo check --target wasm32-unknown-unknown -p rdocx-wasm -p rpptx-wasm`,
  clean.
- `python3 scripts/hash_harness.py --check`, 49 entries match.
- `python3 scripts/golden_png_harness.py --check`, 7 of 7 page-one pixel buffers
  match at 150 DPI, rasteriser reported as `pdftoppm version 26.01.0`.
- `python3 scripts/sync_agent_skills.py --check`, 26 skills in sync.
- Every claim in the commit message was checked against the file rather than
  read. All four hold.
- The new test was checked for being load bearing by hand-executing
  `word_language_ranges` at `crates/rdocx-layout/src/engine.rs:8650` under the
  previous `0x02ff` arm. `"\u{3105}\u{02cb}"` produced two ranges there,
  `(0, 3, EastAsia)` and `(3, 5, Direct)`, so the `ranges.len() == 1` assertion
  at `crates/rdocx-layout/src/engine.rs:19236` fails if the arm is widened
  again. The assertion is not vacuous.
- The Bopomofo case was checked for reachability through the rich path rather
  than assumed. `needs_word_multilingual_layout` at
  `crates/rdocx-layout/src/engine.rs:8726` answers true for `"\u{3105}\u{02cb}"`
  because `U+3105` is inside `is_east_asian`'s `0x3000..=0x33ff` arm at
  `crates/rdocx-layout/src/engine.rs:8328`, and
  `crates/rdocx-layout/src/engine.rs:8834` is the only production call site of
  `word_language_ranges` and sits inside that gated path. So the unit assertion
  describes a shape a real document reaches, not a function called only by its
  own test.
- The narrowing was checked against the pre-F-266a baseline rather than only
  against pass 4. `git show 562af721:crates/rdocx-layout/src/engine.rs` line
  8367 reads `0x0041..=0x024f`, so `0x02af` is not a revert. The IPA extensions
  `0x0250..=0x02af` stay claimed for `Direct`, which is still a behaviour change
  from the baseline, and it is the half pass 5 explicitly endorsed. It is pinned
  at `crates/rdocx-layout/src/engine.rs:19211`.
- The five-step spec sentence was walked against the code step by step rather
  than compared for tone. See the S1 section below.

## Job 1, the state of every pass 5 finding

### Pass 5 S1, the spec set forbidding the last resort the code takes

**Resolved.** Two edits, and both were checked against the code.

The old `docs/hld/08-rendering-spec.md` sentence is scoped rather than deleted.
`docs/hld/08-rendering-spec.md:662` now reads that the four non-Latin references
resolve to nothing "inside a slot" and that answering with the Latin typeface
"there" would be worse than declining. That matches
`crates/rdocx-layout/src/engine.rs:8496`, where the same claim is made about the
same place and no further.

The missing rule is added as a new paragraph at
`docs/hld/08-rendering-spec.md:671` to `:679`. Its two claims were checked
separately.

- "Declining inside the slot is what keeps the Latin face from outranking a
  named family, not a refusal to use it at all." That is
  `crates/rdocx-layout/src/engine.rs:8507` to `:8521` restated, and the
  mechanism is real. `word_font_for_slot` returns `None` for the four
  references at `crates/rdocx-layout/src/engine.rs:8500`, which is what lets
  `crates/rdocx-layout/src/engine.rs:8528` reach the author's `w:ascii` family
  first.
- "Full resolution is therefore five steps: the slot's explicit family, the
  slot's theme font, the same two for the `w:ascii` slot, then that last
  resort, then nothing so the default applies." Walked against the code. Step 1
  and step 2 are `crates/rdocx-layout/src/engine.rs:8469` and `:8483` inside
  `word_font_for_slot`. Steps 3 and 4 are the same function called again with
  `WordFontSlot::Ascii` at `crates/rdocx-layout/src/engine.rs:8529`. Step 5 is
  `crates/rdocx-layout/src/engine.rs:8534` to `:8551`. The trailing `None` is
  the `?` at `crates/rdocx-layout/src/engine.rs:8534` and the `_ => None` at
  `crates/rdocx-layout/src/engine.rs:8549`. Five, in that order, with no sixth
  and none skipped.

The highest-authority statement and the three in-file ones now agree, which is
what pass 5 asked for.

### Pass 5 S2, the widened language arm converting inheritance into a hard break

**Resolved, by the first of the two remedies pass 5 offered, and the second one
as well.** Pass 5 said "either narrow the arm to `0x0250..=0x02af` and say in
the arm's comment why modifier symbols keep inheriting, or keep `0x02ff` and
pin it with a test". The commit does both halves of the first and adds the
pinning from the second.

- `crates/rdocx-layout/src/engine.rs:8621` reads `0x0041..=0x02af`.
- The arm's comment at `crates/rdocx-layout/src/engine.rs:8610` to `:8620` says
  why, and says the thing pass 4 got wrong, that the two tables have different
  residual cases so mirroring a boundary between them is the wrong operation.
  Every factual claim in it holds. `word_font_slot` does claim `0x02b0..=0x02ff`
  for `w:hAnsi` at `crates/rdocx-layout/src/engine.rs:8370`. `None` does mean
  inherit, through the `continue` at
  `crates/rdocx-layout/src/engine.rs:8656`. `U+02C7`, `U+02CA`, `U+02CB` and
  `U+02D9` are the four Bopomofo tone marks and all four are inside
  `0x02b0..=0x02ff`.
- `a_modifier_symbol_keeps_the_language_of_the_letter_it_modifies` at
  `crates/rdocx-layout/src/engine.rs:19208` pins both sides. The IPA loop at
  `crates/rdocx-layout/src/engine.rs:19211` asserts `Direct` across
  `0x0250..=0x02af` and the modifier loop at
  `crates/rdocx-layout/src/engine.rs:19222` asserts `None` across
  `0x02b0..=0x02ff`, so a move in either direction fails. The Bopomofo case at
  `crates/rdocx-layout/src/engine.rs:19233` states the consequence the boundary
  exists for.

The boundary now sits exactly on the Unicode block edge between IPA Extensions
and Spacing Modifier Letters, which is a boundary a future editor can check
against a source outside this repository rather than against another arm in the
same file. That is the structural answer to "the fourth consecutive pass to find
a hand-mirrored range edit", and it is better than another hand-mirrored edit.

### Pass 5 nitpick 1, the run-slot doc comment

**Resolved.** `crates/rdocx-layout/src/engine.rs:8405` to `:8408` now states
that the second pass drops its `w:ascii` candidates before counting them and
why. It matches the body comment at `crates/rdocx-layout/src/engine.rs:8431`
and the `filter` at `crates/rdocx-layout/src/engine.rs:8437`. A reader
predicting from the header alone now answers `EastAsia` for
`"\u{3001}\u{3002} \u{300c}\u{300d}"`, which is what
`crates/rdocx-layout/src/engine.rs:19668` asserts.

### Pass 5 nitpick 2, the spec's no-letter sentence

**Resolved on both halves, with one word left stale.** The consensus rule and
the drop are at `docs/hld/08-rendering-spec.md:638` to `:642`, and the third
cost is at `docs/hld/08-rendering-spec.md:650` to `:652`. The example the spec
chose, an ideographic comma beside an em dash, is the exact input already
pinned at `crates/rdocx-layout/src/engine.rs:19689` and `:19693`, both
answering `Ascii`, so the spec names a case the test gate holds rather than a
case it merely describes. The stale word is new nitpick 1 below.

### Pass 5 nitpicks 3, 4 and 5

3. The field display-segment call site with no hint test. **Still open.**
   `crates/rdocx-layout/src/engine.rs:6365` still passes
   `segment_rpr.font_hint.as_deref()` and the only hint regression drives
   `crates/rdocx-layout/src/engine.rs:6084`. Carried as nitpick 4.
4. Numbering markers bypassing slot resolution. **Still open.**
   `crates/rdocx-layout/src/engine.rs:5910` still reads
   `marker_rpr.font_ascii.as_deref()`. Pre-existing and outside the diff,
   carried as nitpick 5.
5. The plan's "roughly 5 MB" against the spec's measured figures. **Still
   open.** `.claude/plans/F-266a-design.md:197` is unchanged and
   `docs/hld/15-build-and-toolchain.md:477` still gives 9.6 MB and 10.4 MB. The
   plan is a frozen record, so this stays a nitpick. Carried as nitpick 6.

## Job 2, new findings in the pass 5 remediation

## Defects

None.

## Smells

None. The three questions the task named were each worked to an answer.

- **Does narrowing the `Direct` arm reintroduce anything an earlier pass
  fixed?** No. Pass 2 S1, pass 3 S1 and pass 4 S1 were all about the four
  codepoint tables disagreeing over an **East Asian** set, and all four now ask
  `is_east_asian` at `crates/rdocx-layout/src/engine.rs:8313` once. The narrowed
  range `0x02b0..=0x02ff` is outside `is_east_asian`, outside
  `word_font_slot`'s complex-script arm and outside its East Asian arm, so no
  table's East Asian answer moves. The sweep at
  `crates/rdocx-layout/src/engine.rs:19554` still passes, which it would not if
  an East Asian codepoint had changed its language answer. The spec's claim at
  `docs/hld/08-rendering-spec.md:631` that "the language table claims the same
  set the font table does" is scoped to East Asian text in the same sentence,
  so the narrowing does not falsify it.
- **Is the new test's Bopomofo assertion reachable through the rich path?**
  Yes, checked rather than assumed, through
  `crates/rdocx-layout/src/engine.rs:8726` and the single production call site
  at `crates/rdocx-layout/src/engine.rs:8834`. Recorded in full above.
- **Does the rewritten spec prose match the code exactly?** On the rule, yes,
  walked step by step above. The one mismatch found is a count word and is
  nitpick 1, not a rule.

I also worked the narrowing for a case the commit message does not name, a
leading modifier with no letter before it. `word_language_ranges` seeds `slot`
with `Direct` at `crates/rdocx-layout/src/engine.rs:8653`, so `"\u{02cb}\u{3105}"`
yields `(0, 2, Direct)` then `(2, 5, EastAsia)`. That is the same answer the
pre-F-266a table gave, since an unclaimed leading character fell to the same
seed, so the narrowing does not change it and there is nothing to record.

## Nitpicks

- `docs/hld/08-rendering-spec.md:653`, the paragraph now opens "It has three
  costs" at `docs/hld/08-rendering-spec.md:646` and closes "Both costs are
  bounded by `w:hAnsi` and every other slot falling back to the `w:ascii`
  family". The count word was not updated when the third cost was inserted, so
  a reader asking whether the new middle cost is bounded gets no answer from
  the spec. The bound is in fact true of all three, because the third cost
  resolves to `w:ascii`, which is a named family by the same argument, so this
  misstates no rule and only reads as self-contradictory. It is in the
  paragraph `8862a16f` edited, which is why it is recorded here rather than
  treated as pre-existing.
- `crates/rdocx-layout/src/engine.rs:19208`, the test name
  `a_modifier_symbol_keeps_the_language_of_the_letter_it_modifies` describes the
  second and third of its three assertions. The first, at
  `crates/rdocx-layout/src/engine.rs:19211`, pins the IPA extensions as
  `Direct`, which is the opposite half of the boundary and is not a modifier
  symbol. The doc comment at `crates/rdocx-layout/src/engine.rs:19198` states
  the whole boundary, so a reader who opens the test is not misled, only one
  who reads the failure line.
- `crates/rdocx-layout/src/engine.rs:8610` against
  `crates/rdocx-layout/src/engine.rs:8365`, the two comments characterise
  `0x02b0..=0x02ff` differently. The font table calls the IPA and modifier
  blocks "Latin script by Unicode" and the language table says they "carry no
  language of their own". Both are defensible for their own table and the
  language arm's comment explains the asymmetry in the next sentence, so this
  is a reading speed bump rather than a contradiction.
- `crates/rdocx-layout/src/engine.rs:6365`, the field display-segment call site
  still has no test driving a hint through it. Carried from pass 5 nitpick 3.
- `crates/rdocx-layout/src/engine.rs:5910`, numbering markers still read
  `marker_rpr.font_ascii` directly and never pass through
  `resolve_font_family`. Carried from pass 5 nitpick 4, pre-existing and outside
  the diff.
- `.claude/plans/F-266a-design.md:197` against
  `docs/hld/15-build-and-toolchain.md:477`, "Each is roughly 5 MB" still
  disagrees with the measured 9.6 MB and 10.4 MB. Carried from pass 5 nitpick 5.
  The plan is a frozen record, so this is worth noting and not worth editing.

## Not found

- **correctness**. Nothing checked in this aspect produced a finding.
  `8862a16f` changes exactly one production expression, the upper bound of one
  arm in a `match` on `u32`. The 80 codepoints `0x02b0..=0x02ff` move from
  `Some(WordLanguageSlot::Direct)` to `None`, and `None` is handled by the
  `continue` at `crates/rdocx-layout/src/engine.rs:8656`, which is the branch
  those codepoints took before pass 4 widened the arm. Every other codepoint
  answers what it answered in `9e03c7ec`. No third answer is reachable.
- **panics**. Nothing in `8862a16f` can panic. The production change is a range
  bound. The two `expect("BMP scalar")` calls at
  `crates/rdocx-layout/src/engine.rs:19212` and `:19223` are inside
  `#[cfg(test)]` and are total, because `0x0250..=0x02ff` contains no surrogate
  and no value above `0x10ffff`. The test's `ranges[0]` indexing at
  `crates/rdocx-layout/src/engine.rs:19237` is guarded by the length assertion
  two lines above it. No new arithmetic, slicing or unwrap outside tests.
- **ooxml**. Nothing checked in this aspect produced a finding. `8862a16f` adds
  no parser and no serialiser, changes no element or attribute order, adds no
  namespace prefix and drops no unmodelled subtree. It touches one lookup table,
  one test and two prose blocks.
- **structure**. Nothing checked in this aspect produced a finding. No new type,
  trait, generic parameter, `Box<dyn>`, forwarding wrapper, feature flag, crate,
  module or file. The diff is the smallest one that closes both pass 5 smells,
  and the new test joins the existing in-file `#[cfg(test)]` module rather than
  adding a binary under `tests/`.
- **tests**. Beyond the naming nitpick, nothing checked in this aspect produced
  a finding. The new test fails if the arm moves in either direction and fails
  if `word_language_ranges` stops inheriting, checked by hand-executing both.
  No existing test was weakened, and the counts moved by exactly one, from 289
  to 290 in `rdocx-layout`, with every other entrypoint unchanged.
- **contract**. Nothing checked in this aspect produced a finding. `8862a16f`
  changes no item in the plan's `## Test plan`, `## Risk routing` or
  `## Hash harness` sections and adds no file. It edits
  `docs/hld/08-rendering-spec.md`, which the plan's `## HLD impact` names. The
  hash harness and the golden PNG baselines are unchanged at 49 of 49 and 7 of
  7, which is what the plan's `## Hash harness` section requires.
