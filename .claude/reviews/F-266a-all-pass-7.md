# F-266a, all, pass 7

**Reviewed**: `git diff 8862a16f..HEAD` in the worker worktree, which is the
single commit `affcc278`. 3 files changed, 285 insertions and 4 deletions. One
of the three is `.claude/reviews/F-266a-all-pass-6.md` at 281 insertions and 0
deletions, so the reviewable diff is 2 files, 4 insertions and 4 deletions.
`crates/rdocx-layout/src/engine.rs` is 1 and 1, `docs/hld/08-rendering-spec.md`
is 3 and 3. For the record, the full story delta `git diff 562af721..HEAD
--stat` is 25 files changed, 3975 insertions and 73 deletions, of which three
are binary TTFs and six are the pass 1 to pass 6 review files, leaving 16 text
files carrying the reviewable work. Aspects run: correctness, contract, panics,
ooxml, tests, structure.

**Verdict**: 0 defects, 0 smells, 4 nitpicks

**This pass found zero defects and zero smells.** That is the plain result and
not a rounding. `affcc278` changes no production behaviour, the two prose
nitpicks it set out to close are closed, and the four nitpicks pass 6 carried
are still open, still correctly classified, and unmoved from the lines pass 6
cited. Pass 6 already met the `/microscope` exit condition at `8862a16f` and
`affcc278` does not disturb it.

## Job 1, does `affcc278` change any production behaviour

No. Verified three ways rather than read.

- `git show affcc278 --numstat` gives exactly three rows. The only non-review,
  non-prose row is `crates/rdocx-layout/src/engine.rs` at 1 insertion and 1
  deletion, and the diff hunk shows that single line is the `fn` name on the
  line after `#[test]`.
- That name sits at `crates/rdocx-layout/src/engine.rs:19208`, and the crate's
  only `mod tests` opens at `crates/rdocx-layout/src/engine.rs:9010` under the
  `#[cfg(test)]` at `crates/rdocx-layout/src/engine.rs:9009`. The changed line
  is therefore compiled out of every non-test build.
- Inside the test nothing else moved. The loop bounds `0x0250..=0x02af` at
  `crates/rdocx-layout/src/engine.rs:19211` and `0x02b0..=0x02ff` at
  `crates/rdocx-layout/src/engine.rs:19222`, the three inline comments, the
  Bopomofo input `"\u{3105}\u{02cb}"` at
  `crates/rdocx-layout/src/engine.rs:19234`, the four assertions at
  `crates/rdocx-layout/src/engine.rs:19236` to `:19239` and the two assertion
  messages are all outside the hunk. So no assertion, no range, no comment
  semantic and no production line moved.

The `docs/hld/08-rendering-spec.md` hunk is three lines replaced by three, and
the only word-level change is `Both costs` becoming `All three costs`. The
other two lines differ solely by rewrap around the longer phrase. No claim in
the paragraph was added, removed or restated.

Test counts confirm the same thing from the other side. `rdocx-layout` reports
290 passing, identical to pass 6, so the rename added no test and dropped none.

## Job 2, is the costs paragraph now internally consistent and true of the code

Yes, on both halves.

The paragraph opens "It has three costs" at
`docs/hld/08-rendering-spec.md:646` and I counted the costs it then names
rather than trusting the number. Three, and they are distinct.

1. `docs/hld/08-rendering-spec.md:646` to `:649`, a mixed Latin and East Asian
   run keeps the `w:ascii` family for its East Asian half.
2. `docs/hld/08-rendering-spec.md:649` to `:651`, a run with no letter whose
   remaining characters disagree loses `w:eastAsia` for its ideographic half.
3. `docs/hld/08-rendering-spec.md:651` to `:653`, `w:hAnsi` is reachable only
   by a run with no ASCII letter in it.

The closing sentence at `docs/hld/08-rendering-spec.md:653` now reads "All
three costs are bounded", so the count word and the enumeration agree and the
self-contradiction pass 6 recorded is gone.

The bound itself was checked against the code, not accepted because the count
now matches. The sentence claims `w:hAnsi` and every other slot falls back to
the `w:ascii` family so a run resolves to a named family either way. That is
`crates/rdocx-layout/src/engine.rs:8528` to `:8529`, where
`resolve_font_family` calls `word_font_for_slot(rpr, theme, slot)` and chains
`.or_else(|| word_font_for_slot(rpr, theme, WordFontSlot::Ascii))`. The
`or_else` arm is unconditional on the slot, so it covers cost 3's `w:hAnsi`
exactly as it covers cost 1's `EastAsia` and cost 2's. Each of the three costs
ends with a run resolving through the author's `w:ascii` family, which is a
named family, so the bound is true of all three and not only of the two the
sentence used to count. The edit corrected the count to match a bound that
already held, which is the weaker and safer of the two possible fixes.

I also checked the phrase did not survive anywhere else. `grep -rn "Both
costs" docs crates .claude` outside `.claude/reviews/` returns nothing.

## Job 3, does the new test name cover all three assertions

Yes. The name `a_latin_letter_takes_the_direct_language_slot_and_a_modifier_inherits`
at `crates/rdocx-layout/src/engine.rs:19208` has two clauses and the test has
two properties, with the third assertion being the second property demonstrated
at range level.

- Clause 1, "a latin letter takes the direct language slot", is the loop at
  `crates/rdocx-layout/src/engine.rs:19211` to `:19218`. It asserts
  `Some(WordLanguageSlot::Direct)` across `0x0250..=0x02af`, the IPA Extensions
  block, which is Latin script by Unicode and category `Ll` throughout. This is
  the assertion the old name did not describe at all, which is what pass 6
  recorded, and the new name now names it first.
- Clause 2, "a modifier inherits", is the loop at
  `crates/rdocx-layout/src/engine.rs:19222` to `:19229`, asserting `None`
  across `0x02b0..=0x02ff`, with `None` meaning inherit through the `continue`
  at `crates/rdocx-layout/src/engine.rs:8656`.
- The third assertion at `crates/rdocx-layout/src/engine.rs:19234` to `:19239`
  is clause 2's consequence, not a third independent property. It takes one
  modifier from that same range, `U+02CB`, and shows the inheritance producing
  a single `EastAsia` range rather than a split. A failure there is a failure
  of "a modifier inherits", so the name predicts it.

A failure on the IPA loop now prints a name consistent with what failed, which
is the whole point of the rename. The assertion message
`"U+{codepoint:04X} is a letter"` at
`crates/rdocx-layout/src/engine.rs:19216` agrees with the name's first clause,
and `"U+{codepoint:04X} is a modifier and must inherit"` at
`crates/rdocx-layout/src/engine.rs:19227` agrees with its second.

No reference to the old name survives outside `.claude/reviews/`. A repository
grep for `a_modifier_symbol_keeps_the_language_of_the_letter_it_modifies`,
excluding `.git` and `target`, returns two hits, both in
`.claude/reviews/F-266a-all-pass-6.md` at lines 121 and 221, which is the
review record quoting the name it reviewed and is correct there. The new name
has exactly one hit, the definition itself. Nothing in `docs/hld/`, no script,
no CI workflow and no other test names it, so no filter string, no
`--test`-level selector and no prose went stale.

## Job 4, the four carried nitpicks

All four are unchanged and all four are still nitpicks rather than smells. The
rename is at `crates/rdocx-layout/src/engine.rs:19208`, below every carried
citation, and the spec edit does not touch `docs/hld/15-build-and-toolchain.md`
or the plan, so no carried line number shifted. Each was re-read at its cited
line.

1. `crates/rdocx-layout/src/engine.rs:8610` against
   `crates/rdocx-layout/src/engine.rs:8365`. Both comments are present and
   unchanged. **Still a nitpick.** Each statement is true of its own table.
   `word_font_slot` at `crates/rdocx-layout/src/engine.rs:8370` really does put
   `0x02b0..=0x02ff` in `w:hAnsi` and its comment's reason, Latin script by
   Unicode, is the right reason for a font table with no inherit answer.
   `word_language_slot` at `crates/rdocx-layout/src/engine.rs:8621` really does
   stop at `0x02af` and its comment's reason, no language of their own, is the
   right reason for a table where `None` means inherit. Nothing here will
   become wrong later, which is the smell test, because the comment at
   `crates/rdocx-layout/src/engine.rs:8611` to `:8620` states the asymmetry
   explicitly and warns the next editor off mirroring the boundary. It costs a
   reader a moment, nothing more.
2. `crates/rdocx-layout/src/engine.rs:6365` still reads
   `word_font_slot_for_text(value, segment_rpr.font_hint.as_deref())`.
   **Still a nitpick.** The call site is correct, it passes the hint it should
   pass, and the gap is test coverage at one of two call sites of a function
   already covered at `crates/rdocx-layout/src/engine.rs:6084`. An untested but
   correct call site of a tested function is taste, not a latent wrong answer.
3. `crates/rdocx-layout/src/engine.rs:5910` still reads
   `let marker_font_family = marker_rpr.font_ascii.as_deref();`. **Still a
   nitpick.** It is pre-existing, outside this story's diff, and the plan does
   not claim numbering markers. Classifying a pre-existing behaviour this
   story did not touch as a smell would make this diff answerable for code it
   never changed.
4. `.claude/plans/F-266a-design.md:197` still reads "Each is roughly 5 MB" and
   `docs/hld/15-build-and-toolchain.md:477` still gives 9.6 MB and 10.4 MB.
   **Still a nitpick.** A design plan is a frozen record of what was believed
   at design time, the spec carries the measured figures where a reader will
   look, and the rejected-alternative conclusion survives the correction by a
   wide margin. Editing the plan would destroy the record to fix nothing.

## Scope note, the working tree is not clean

Outside the reviewed commit range, `git status --porcelain` in the worktree
reports `.claude/plans/F-266a-design.md` modified and
`.claude/handoffs/F-266a-ready.md` untracked. The plan change is `**Status**:
approved` becoming `completed` at `.claude/plans/F-266a-design.md:3` and the
implementation checklist at `.claude/plans/F-266a-design.md:319` to `:340`
being ticked. Both are `/complete-feature --prepare` output rather than
anything this pass reviews, and neither touches
`.claude/plans/F-266a-design.md:197`, so nitpick 4 above cites an unmodified
line.

Recording one thing plainly rather than leaving it unsaid. The handoff file
already asserts "pass 7 confirming the two prose nitpicks closed after it" and
already names four remaining nitpicks, which was written before this pass ran.
It is not a finding against the diff and it is not a code issue, and as it
happens the claim is accurate, since that is exactly what this pass found. It
is noted so the record shows the review reached its conclusion independently
of a document that had already stated one.

## Defects

None.

## Smells

None. The commit under review changes one test function name and one count
word in prose. There is no mechanism in it that can become wrong later.

## Nitpicks

All four are carried from pass 6 unchanged. No new nitpick was found, and the
two pass 6 nitpicks that `affcc278` set out to close are closed and are not
restated here.

- `crates/rdocx-layout/src/engine.rs:8610` against
  `crates/rdocx-layout/src/engine.rs:8365`, the two comments characterise
  `0x02b0..=0x02ff` from their own table's point of view, which is a reading
  speed bump the adjacent comment exists to explain.
- `crates/rdocx-layout/src/engine.rs:6365`, the field display-segment call site
  still has no test driving a font hint through it.
- `crates/rdocx-layout/src/engine.rs:5910`, numbering markers still read
  `marker_rpr.font_ascii` directly and never pass through
  `resolve_font_family`. Pre-existing and outside this diff.
- `.claude/plans/F-266a-design.md:197` against
  `docs/hld/15-build-and-toolchain.md:477`, "Each is roughly 5 MB" against the
  measured 9.6 MB and 10.4 MB. The plan is a frozen record.

## Not found

- **correctness**. Nothing checked in this aspect produced a finding.
  `affcc278` contains no production expression at all. The one code line it
  changes is an identifier inside `#[cfg(test)]`, and an identifier carries no
  behaviour. Every codepoint answers what it answered at `8862a16f`.
- **panics**. Nothing checked in this aspect produced a finding. No `unwrap`,
  `expect`, index, slice or arithmetic was added, removed or moved. The two
  `expect("BMP scalar")` calls the renamed test already contained are
  unchanged and remain total over `0x0250..=0x02ff`, which holds no surrogate
  and no value above `0x10ffff`.
- **ooxml**. Nothing checked in this aspect produced a finding. No parser, no
  serialiser, no element or attribute order, no namespace prefix, no
  whitespace handling and no captured subtree is touched.
- **structure**. Nothing checked in this aspect produced a finding. No type,
  trait, generic parameter, `Box<dyn>`, forwarding wrapper, feature flag,
  crate, module or file. The diff is two edits and could not be smaller while
  still closing both nitpicks.
- **tests**. Nothing checked in this aspect produced a finding. The rename
  weakens nothing, since every assertion and both loop bounds are byte
  identical, and `rdocx-layout` still reports 290 passing. The name now covers
  all three assertions, which was the point.
- **contract**. Nothing checked in this aspect produced a finding. `affcc278`
  adds no file, changes no item in the plan's `## Test plan`, `## Risk routing`
  or `## Hash harness` sections, and edits only
  `docs/hld/08-rendering-spec.md`, which the plan's `## HLD impact` at
  `.claude/plans/F-266a-design.md:236` names. The hash harness and the golden
  PNG baselines are unchanged at 49 of 49 and 7 of 7, which is what
  `## Hash harness` requires.

## Gates run at HEAD `affcc278`

Every command below ran in the worker worktree with the pinned oracle path
first. All are green.

| Command | Result |
|---|---|
| `cargo fmt --all --check` | exit 0 |
| `cargo clippy --workspace --all-targets --all-features -- -D warnings` | exit 0 |
| `cargo test -p oxml-layout` | 107 passed, 0 failed, plus 3 doctests |
| `cargo test -p oxml-layout --no-default-features` | 105 passed, 0 failed, plus 3 doctests |
| `cargo test -p rdocx-layout` | 290 passed, 0 failed, plus 1 doctest |
| `cargo test -p rdocx` | 464, 291 and 516 passed, 0 failed, plus 2 doctests |
| `python3 scripts/hash_harness.py --check` | 49 entries match, exit 0 |
| `python3 scripts/golden_png_harness.py --check` | 7 page-one pixel buffers match at 150 DPI, rasteriser `pdftoppm version 26.01.0`, exit 0 |
| `python3 scripts/prose_check.py` | 0 violations, exit 0 |
| `python3 -m unittest scripts.test_sprint_workflow` | Ran 122 tests, OK, 2 skipped, exit 0 |
| `python3 scripts/sync_agent_skills.py --check` | 26 skills in sync, exit 0 |
| `cargo check --target wasm32-unknown-unknown -p rdocx-wasm -p rpptx-wasm` | exit 0 |

The test totals match pass 6 exactly in every entrypoint, which is the
independent confirmation that `affcc278` neither added nor removed a test.

The known environment artifacts were not run and are not counted here. The
`rpptx-cli` corpus validation test, the three `rpptx-layout`
`context::tests::*corpus*` tests and `cargo publish --dry-run -p rdocx-layout`
fail in any worker worktree for reasons outside this diff.

## Addendum, HEAD moved while this pass was running

Between the gate run and this file being written, a concurrent
`/complete-feature --prepare` created `87f92eb2`, "F-266a, prepare integration
handoff", which became the new branch tip and swept this review file into
itself. Its `--numstat` is three rows and all three are under `.claude/`:
`.claude/handoffs/F-266a-ready.md` at 30 insertions, `.claude/plans/F-266a-design.md`
at 11 and 11, and `.claude/reviews/F-266a-all-pass-7.md` at 270 insertions.

No crate, no `docs/hld/` file, no script and no workflow is touched, so the
code and prose at the new tip are byte identical to `affcc278` and every gate
result recorded above still describes the branch tip. The diff this pass
reviewed, `8862a16f..affcc278`, is unchanged and the verdict stands. The plan
modification and the untracked handoff noted in the scope section above are
this commit, now committed rather than pending.
