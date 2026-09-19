# F-266a, all, pass 5

**Reviewed**: `git diff 562af721..HEAD` in the worker worktree, the six F-266a
commits `a8bf6e5a`, `05cd97f5`, `e96b7ddc`, `31a95bc2`, `feb500fd` and
`9e03c7ec` reviewed as one delta. 23 files changed, 3308 insertions and 73
deletions. Three of the 23 are binary TTFs and four are the pass 1 to pass 4
review files, so 16 text files carry the reviewable diff. The new commit
`9e03c7ec` is 2 files, 467 insertions and 46 deletions, of which
`crates/rdocx-layout/src/engine.rs` is 110 insertions and 46 deletions and the
rest is the pass 4 review file. Aspects run: correctness, contract, panics,
ooxml, tests, structure.

**Verdict**: 0 defects, 2 smells, 5 nitpicks

The pass 4 remediation is real. Both pass 4 smells are closed in every place
pass 4 cited, the widened sweep and the widened language arm are both present,
and nothing in `9e03c7ec` computes a wrong answer for any input I could
construct. The two smells below are new. One is the pass 4 S1 problem surviving
in a fourth location that pass 4 explicitly exonerated. The other is the
consequence of the widened language arm, which pass 4 authorised on the stated
ground that it changed no reachable document.

## What was verified rather than trusted

- `cargo fmt --all --check`, clean.
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`, clean.
- `cargo test -p rdocx-layout`, 289 pass plus the doctest.
- `cargo test -p oxml-layout`, 107 pass plus 3 doctests. `--no-default-features`,
  105 pass plus 3.
- `cargo test -p rdocx --test integration_test --test regression_test`, 291 and
  516 pass.
- `cargo check --target wasm32-unknown-unknown -p rdocx-wasm -p rpptx-wasm`,
  clean.
- `python3 scripts/hash_harness.py --check`, 49 entries match.
- `python3 scripts/golden_png_harness.py --check`, 7 of 7 page-one pixel buffers
  match at 150 DPI, rasteriser reported as `pdftoppm version 26.01.0`.
- `python3 scripts/prose_check.py`, 0 violations.
- `python3 scripts/sync_agent_skills.py --check`, 26 skills in sync.
- `cargo test -p rdocx-layout every_east_asian_codepoint_agrees` on its own
  finishes in 0.30 s at the widened `0..=0x10ffff` bound, so the sweep widening
  costs nothing worth recording.
- The `consensus` rewrite was worked against the rule it replaced rather than
  read. For every input, `9e03c7ec` answers either exactly what `feb500fd`
  answered or `WordFontSlot::Ascii`, and never a third thing. That is a strict
  narrowing toward the pre-F-266a answer, which is why no sample moved.
- `script_for_char` at `crates/oxml-layout/src/font.rs:1772` was compared range
  by range against `is_east_asian` at `crates/rdocx-layout/src/engine.rs:8313`.
  Every Hangul, Kana and Han range in the former is inside the latter, so the
  shared set's doc comment claim to be a superset is true and the sweep's new
  upper bound is not papering over a gap.
- Every test name in the plan's `## Test plan` was looked up in the tree. Nine
  of the eleven exist verbatim. The two that do not,
  `east_asia_and_bidi_theme_references_read_their_own_theme_entry` and
  `arabic_runs_stay_joined_across_a_run_boundary`, are both closed deviations.
  The first is the test renamed twice by remediation and still present. The
  second is recorded in the spec set at
  `docs/hld/14-development-backlog.md:2623`, which is what pass 1 S5 asked for.
  Neither is a new finding.

## Job 1, the state of every pass 4 finding

### Pass 4 S1, the theme last resort documented as impossible in three places

**Resolved at all three cited locations.** The underlying problem survives
elsewhere, recorded as new smell S1 below.

- `resolve_font_family`'s doc comment at
  `crates/rdocx-layout/src/engine.rs:8502` now opens "Five steps, in order" and
  lists all five, with the last resort named as the fourth. The paragraph at
  `crates/rdocx-layout/src/engine.rs:8512` explains why it exists and why
  declining inside the slot and answering outside it is what keeps the Latin
  face from outranking a named family. The claim matches the code at
  `crates/rdocx-layout/src/engine.rs:8529` to `:8548`.
- `word_font_for_slot`'s comment at
  `crates/rdocx-layout/src/engine.rs:8491` no longer forbids the branch. It now
  says declining here is what stops the Latin face outranking the author's
  family, and it names `resolve_font_family` as the place that answers with it
  after both slots decline. Accurate on both halves.
- The test is renamed at `crates/rdocx-layout/src/engine.rs:19297` to
  `east_asia_and_bidi_theme_references_never_outrank_the_family_the_author_named`,
  and its doc comment at `crates/rdocx-layout/src/engine.rs:19279` to `:19296`
  states both halves and says why both are asserted. The body's two assertions
  match the name, the fall-through set at
  `crates/rdocx-layout/src/engine.rs:19313` and the last-resort set at
  `crates/rdocx-layout/src/engine.rs:19415`. No stale reference to the old name
  survives anywhere outside `.claude/reviews/`, checked by grep across `.rs`
  and `.md`.

### Pass 4 S2, the no-claimant branch decided by position

**Resolved.** `crates/rdocx-layout/src/engine.rs:8408` is one `consensus`
helper, called at `crates/rdocx-layout/src/engine.rs:8420` for the letter pass
and at `crates/rdocx-layout/src/engine.rs:8431` for the no-letter pass. The
helper's answer is the unique slot when the candidates agree and
`WordFontSlot::Ascii` when they do not, which is order independent by
construction, so the pass 4 input no longer decides by position.

The two orderings pass 4 named are pinned at
`crates/rdocx-layout/src/engine.rs:19627` and `:19631`, both answering `Ascii`.
The same-slot pairs are pinned at `crates/rdocx-layout/src/engine.rs:19635`
answering `EastAsia` and `:19639` answering `HighAnsi`, so the assertions are
not vacuous and a helper that always answered `Ascii` would fail them. The doc
comment at `crates/rdocx-layout/src/engine.rs:8393` to `:8403` describes both
passes. One imprecision in it is nitpick 1 below.

### The six pass 4 nitpicks

1. The field display-segment call site with no hint test. **Still open.**
   `crates/rdocx-layout/src/engine.rs:6365` still passes
   `segment_rpr.font_hint.as_deref()` and the only hint regression drives
   `crates/rdocx-layout/src/engine.rs:6084`. Carried as nitpick 3.
2. The sweep stopping at `0x2fa1f`. **Fixed.**
   `crates/rdocx-layout/src/engine.rs:19492` now reads `0..=0x10ffff_u32`, and
   `char::from_u32` skips the surrogate range, so the whole scalar space is
   covered. A range added to `is_east_asian` above `0x2fa1f` and forgotten in
   the declared list now fails at `crates/rdocx-layout/src/engine.rs:19497`.
3. The language table's `Direct` arm not mirroring the font table's widening.
   **Fixed as asked, with a consequence pass 4 did not anticipate.**
   `crates/rdocx-layout/src/engine.rs:8603` now reads `0x0041..=0x02ff`. The
   consequence is new smell S2 below.
4. Numbering markers bypassing slot resolution. **Still open.**
   `crates/rdocx-layout/src/engine.rs:5910` still reads
   `marker_rpr.font_ascii.as_deref()`. Pre-existing and outside the diff,
   carried as nitpick 4.
5. The plan's "roughly 5 MB" against the spec's measured figures. **Still
   open.** `.claude/plans/F-266a-design.md:197` is unchanged and
   `docs/hld/15-build-and-toolchain.md:476` still gives 9.6 MB and 10.4 MB. The
   plan is a frozen record, so this stays a nitpick. Carried as nitpick 5.
6. The Latin-1 punctuation run taking `w:hAnsi` with no test naming it.
   **Fixed.** `crates/rdocx-layout/src/engine.rs:19647` asserts that
   `"\u{00ab}\u{00bb}"` answers `WordFontSlot::HighAnsi`, with a comment saying
   why that is bounded.

## Job 2, new findings in the pass 4 remediation

## Defects

None. The `consensus` rewrite is order independent and strictly narrowing, the
widened sweep is not vacuous, the widened language arm cannot make any table
disagree about an East Asian codepoint, and the three rewritten comments and
the renamed test all describe the code they sit beside.

## Smells

### S1, the spec set still forbids the last resort the code takes

`docs/hld/08-rendering-spec.md:660`, against
`crates/rdocx-layout/src/engine.rs:8529`

Pass 4 S1 said "`docs/hld/08-rendering-spec.md` already carries the correct
rule and only the three in-file statements lag". That is not what the file
says. `docs/hld/08-rendering-spec.md:659` to `:662` reads that the four
non-Latin references "resolve to nothing and fall through to the run's own
`w:ascii` family. Answering with the Latin typeface would be worse than
declining, because it is a face that usually cannot draw the text and it would
outrank the family the author actually named."

That is word for word the comment `9e03c7ec` deleted from
`crates/rdocx-layout/src/engine.rs:8491` precisely because it forbids what the
code does. `resolve_font_family` answers with the Latin typeface at
`crates/rdocx-layout/src/engine.rs:8546`, deliberately, for exactly those four
values, whenever both slots decline. The spec paragraph names no fifth step and
no last resort at all, so the rule a reader takes from the spec set is the one
pass 3 S3 found to be a defect.

This matters more than the three in-file statements did.
`docs/hld/08-rendering-spec.md` is named in the plan's `## HLD impact`, it is
what `/complete-feature` executes against, and `docs/hld/README.md` makes the
spec set win over a code comment on any rule question. So after the
remediation, the three lowest-authority statements are right and the highest
authority one is wrong, which is the worst arrangement of the four. The
remedy is one sentence added after
`docs/hld/08-rendering-spec.md:662`, saying what
`crates/rdocx-layout/src/engine.rs:8512` now says.

### S2, the widened language arm converts inheritance into a hard break, and nothing pins the boundary

`crates/rdocx-layout/src/engine.rs:8603`, with
`crates/rdocx-layout/src/engine.rs:8328` and
`crates/rdocx-layout/src/engine.rs:8370`

Pass 4 nitpick 3 asked for `0x024f` to be widened to `0x02ff` "to mirror the
font table", and classified it a nitpick because "an unclaimed character
attaches to the preceding language range, which for a Latin word is `Direct`
either way, so no reachable document changes". The preceding range is not
always `Direct`, and reachable documents do change.

`word_language_slot` answering `None` means inherit. `word_language_ranges` at
`crates/rdocx-layout/src/engine.rs:8637` skips an unclaimed character with
`continue`, so it stays inside whichever range it fell in. Answering
`Some(WordLanguageSlot::Direct)` means start a new range. The 176 codepoints
`0x0250..=0x02ff` moved from the first to the second.

The two tables being mirrored do not have the same residual case, which is what
makes boundary-for-boundary mirroring the wrong operation here.
`word_font_slot` has no inherit answer at all. Its ambiguous arm at
`crates/rdocx-layout/src/engine.rs:8377` defaults every unclaimed codepoint to
`HighAnsi`, and the run-level rule at
`crates/rdocx-layout/src/engine.rs:8404` then discards non-letters entirely
while a letter is present. `word_language_slot` is per character with no
run-level rule above it, so its `None` is load bearing in a way the font
table's default is not.

The concrete reachable case is Bopomofo. Its letters are East Asian through
`0x3000..=0x33ff` at `crates/rdocx-layout/src/engine.rs:8328`, so a Bopomofo
text node passes `needs_word_multilingual_layout` and reaches
`word_language_ranges`. Its tone marks are `U+02C7`, `U+02CA`, `U+02CB` and
`U+02D9`, all inside the widened arm. Before `9e03c7ec` a syllable and its tone
mark were one `EastAsia` range taking `w:lang/@w:eastAsia`. After it they are
two ranges, and the tone mark takes `w:lang/@w:val` and is shaped as a separate
slice at `crates/rdocx-layout/src/engine.rs:8819`. A modifier that carries no
language of its own now breaks the language run it modifies. The upper half of
the widened block, `0x02b0..=0x02ff`, is spacing modifier letters and modifier
symbols generally, and the same argument covers all of them. The lower half,
`0x0250..=0x02af`, is IPA Extensions, which are real letters, and `Direct` is
right for those.

The second half of this is that nothing pins the boundary either way. The sweep
at `crates/rdocx-layout/src/engine.rs:19461` asserts only the East Asian answer
for both tables, and `korean_text_takes_the_east_asian_language_slot` at
`crates/rdocx-layout/src/engine.rs:19153` checks `Direct` for `'A'`, `'\u{915}'`
and `'\u{e01}'` only. No test names a codepoint in `0x0250..=0x02ff`, so this
boundary moved once with no assertion behind it and can move again the same
way. This is the fourth pass in a row to find a one-sided or hand-mirrored range
edit between these two tables, which is the reason to stop fixing it by hand.

Either narrow the arm to `0x0250..=0x02af` and say in the arm's comment why
modifier symbols keep inheriting, or keep `0x02ff` and pin it with a test that
states the Bopomofo consequence as intended. Note also that the mirroring is
partial whichever way it goes, since `0xa720..=0xa7ff` and `0xfb00..=0xfb1c` are
`HighAnsi` to the font table at `crates/rdocx-layout/src/engine.rs:8370` and
still unclaimed by the language table.

## Nitpicks

- `crates/rdocx-layout/src/engine.rs:8399`, the doc comment says "Both passes
  take the same consensus rule" and "Any disagreement resolves to `w:ascii`",
  and never says the second pass drops its `w:ascii` candidates first. A reader
  predicting from the header alone answers `Ascii` for
  `"\u{3001}\u{3002} \u{300c}\u{300d}"`, because the space is an `Ascii`
  candidate that disagrees. The code answers `EastAsia`, pinned at
  `crates/rdocx-layout/src/engine.rs:19606`. The body comment at
  `crates/rdocx-layout/src/engine.rs:8428` does state it, so the reader is
  corrected four lines later.
- `docs/hld/08-rendering-spec.md:638`, the spec's no-letter sentence still reads
  "its remaining characters decide" and carries no consensus rule, so the one
  place pass 4 sent readers for this rule now states half of it. The two costs
  listed at `docs/hld/08-rendering-spec.md:642` do not mention the third one
  `9e03c7ec` created, which is that a no-letter run whose non-ASCII characters
  disagree loses `w:eastAsia` for its ideographic half.
- `crates/rdocx-layout/src/engine.rs:6365`, the field display-segment call site
  still has no test driving a hint through it. Carried from pass 4 nitpick 1.
- `crates/rdocx-layout/src/engine.rs:5910`, numbering markers still read
  `marker_rpr.font_ascii` directly and never pass through
  `resolve_font_family`. Carried from pass 4 nitpick 4, pre-existing and outside
  the diff.
- `.claude/plans/F-266a-design.md:197` against
  `docs/hld/15-build-and-toolchain.md:476`, "Each is roughly 5 MB" still
  disagrees with the measured 9.6 MB and 10.4 MB. Carried from pass 4 nitpick 5.
  The plan is a frozen record, so this is worth noting and not worth editing.

## Not found

- **structure**. Nothing checked in this aspect produced a finding. The nested
  `fn consensus(slots: impl Iterator<Item = WordFontSlot>)` at
  `crates/rdocx-layout/src/engine.rs:8408` is an anonymous generic parameter,
  and `AGENTS.md` allows one when it is instantiated two ways today. It is, at
  `crates/rdocx-layout/src/engine.rs:8420` with `Map<Filter<Chars>>` and at
  `crates/rdocx-layout/src/engine.rs:8431` with `Filter<Map<Chars>>`, both
  present in this diff and neither hypothetical. `AGENTS.md` also asks for one
  sentence naming the second instantiation, and the helper's doc comment at
  `crates/rdocx-layout/src/engine.rs:8405` is exactly that sentence. The helper
  reduces the number of cases a reader must hold, since the two passes had
  drifted apart and now cannot. No new type, trait, `Box<dyn>`, forwarding
  wrapper, feature flag, module or file.
- **panics**. Nothing in `9e03c7ec` can panic. It adds one nested function over
  an iterator of `Copy` values, one `filter`, one widened range in an existing
  `match` on `u32`, and test assertions. No new `unwrap`, `expect`, slicing,
  indexing or arithmetic on untrusted input outside `#[cfg(test)]`. The single
  `unwrap_or` at `crates/rdocx-layout/src/engine.rs:8436` is total, and the
  widened sweep at `crates/rdocx-layout/src/engine.rs:19492` guards every
  codepoint through `char::from_u32`, so the surrogate range cannot reach
  `char`.
- **ooxml**. Nothing checked in this aspect produced a finding. `9e03c7ec` adds
  no parser and no serialiser, changes no element or attribute order, adds no
  namespace prefix and drops no unmodelled subtree. It touches only two lookup
  tables, one run-level rule and four comments.
- **correctness, on the consensus rewrite**. Worked exhaustively over the three
  shapes a run can take. With at least one letter the second pass is
  unreachable and the answer is unchanged from `feb500fd`. With no letter and
  agreeing non-ASCII candidates the answer matches the old first-wins answer.
  With no letter and disagreeing candidates the answer is `Ascii`, which is
  what the run resolved through before F-266a existed. No input produces an
  answer that neither rule produced, so no document that worked before this
  commit resolves to a family it did not resolve to before F-266a.
- **correctness, on the widened sweep**. The new upper bound does not make the
  sweep fail or pass vacuously. Every codepoint above `0x2fa1f` is outside
  `is_east_asian` and outside the declared list, and both the font and language
  tables answer not-East-Asian for all of them, so the three equality
  assertions hold across the added space rather than being trivially satisfied
  on one side only.
- **tests**. Beyond the coverage gaps already recorded as smell S2 and nitpicks
  1 to 3, nothing checked in this aspect produced a finding. The four new
  assertions at `crates/rdocx-layout/src/engine.rs:19627` to `:19648` are load
  bearing in both directions, since two of them would fail under the
  `feb500fd` rule and the other two would fail under a helper that always
  answered `Ascii`.
- **contract**. Beyond smell S1, nothing checked in this aspect produced a
  finding. `9e03c7ec` changes no item in the plan's `## Test plan`,
  `## Risk routing` or `## Hash harness` sections, adds no file, and leaves the
  four named `## HLD impact` files edited. The hash harness and the golden PNG
  baselines are unchanged at 49 of 49 and 7 of 7.
