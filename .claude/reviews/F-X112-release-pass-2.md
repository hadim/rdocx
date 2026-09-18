# F-X112, release, pass 2

**Reviewed**: `git diff aba94e0c..a897aa84`, 61 files, 1083 insertions and 233
deletions. This covers the whole feature diff again, including remediation
commit `a897aa84` (6 files, 299 insertions, 56 deletions). The review file for
pass 1 is excluded from the scope.
**Verdict**: 0 defects, 1 smell, 5 nitpicks

## Pass 1 findings

### D1, confirmed fixed

`CHANGELOG.md:39` to `CHANGELOG.md:41` now credits
`Presentation::slide_png_deterministic` and `slide_pngs_deterministic` with
[Issue 76] in `rpptx-v0.12.0`, and `CHANGELOG.md:90` adds it to the
`@hadim` credit. Both methods exist at `crates/rpptx/src/lib.rs:1179` and
`crates/rpptx/src/lib.rs:1186`. They are absent at `rpptx-v0.11.0` and were
added by `4f564c9b` (F-X094e), which follows that tag. The inventory at
`scripts/test_sprint_workflow.py:5351` includes 76.

### D2, confirmed fixed

`CHANGELOG.md:145` to `CHANGELOG.md:154` credits `insert_picture_to_story`
and `add_story_comment` with Issue 121, plus the native helpers behind Issue 94
and PRs 108 to 111. `CHANGELOG.md:260` to `CHANGELOG.md:272` adds the matching
credit. The helper names match real additions after `v0.13.1`:
`content_index_of_paragraph` (F-X106a), `set_style_value`,
`set_highlight_value`, `set_shading_value`, `highlight_color` (F-X106b), and
`find_paragraph_indices` (F-X106c). The inventory at
`scripts/test_sprint_workflow.py:5344` to `scripts/test_sprint_workflow.py:5347`
matches.

### D3, confirmed fixed

`CHANGELOG.md:227` to `CHANGELOG.md:230` now says `rdocx replace` refuses any
existing output, including the input, and that in-place scripts must change.
That matches `crates/rdocx-cli/src/commands.rs:959`,
`crates/rdocx-cli/src/commands.rs:1134`, and
`crates/oxml-cli-support/src/lib.rs:112`. Published `v0.13.1` called
`doc.save(output)`. I also checked the other writing commands. `convert` raster
output and `render` already used `stage_and_publish` and
`ensure_output_paths_available` at `v0.13.1`, so `replace` is the only CLI
overwrite change that needed disclosure.

### D4, confirmed fixed

`CHANGELOG.md:419` to `CHANGELOG.md:421` now describes only the
`Slide.notes_text` setter, which exists at
`crates/rpptx-py/python/rpptx/_rpptx.pyi:119` to
`crates/rpptx-py/python/rpptx/_rpptx.pyi:120`. The diff of
`_rpptx.pyi` since `py-rpptx-v0.11.0` adds no replacement API, and no Python
note claims one now.

### D5, confirmed fixed

PR 123 and `@chevinbrown` were removed from `py-rpptx-v0.12.0`
(`CHANGELOG.md:436` to `CHANGELOG.md:438`, `CHANGELOG.md:450` to
`CHANGELOG.md:459`). The inventory at `scripts/test_sprint_workflow.py:5362`
to `scripts/test_sprint_workflow.py:5366` is now `{105}` and `{"hadim"}`.
Neither Python stub file nor either binding source contains `chart`.

### S1, confirmed fixed

`CHANGELOG.md:72` to `CHANGELOG.md:75` discloses the notes-inclusive
`Presentation::replace_text` scope and count. That matches
`crates/rpptx/src/lib.rs:1678` to `crates/rpptx/src/lib.rs:1700`. At
`rpptx-v0.11.0` the method summed only slide shape trees.

### S2, partially fixed and still open

See S1 below.

### S3, confirmed fixed

`.claude/plans/F-X112-design.md:6`, `.claude/plans/F-X112-design.md:39`, and
`.claude/plans/F-X112-design.md:55` now say through F-X121. That matches
`docs/hld/14-development-backlog.md:5349`, and the test asserts it at
`scripts/test_sprint_workflow.py:5484`.

### Pass 1 nitpicks

- `Table.clone_row` namespace entry is folded into Added at
  `CHANGELOG.md:314` to `CHANGELOG.md:316`. Fixed.
- The 64 MiB ceiling is now stated at `CHANGELOG.md:433` to
  `CHANGELOG.md:434`. Fixed.
- The `Paragraph.runs` index shift is disclosed at `CHANGELOG.md:364` to
  `CHANGELOG.md:366`. Fixed.
- `docs/hld/03-architecture.md:978` to `docs/hld/03-architecture.md:980` and
  `docs/hld/10-bindings-spec.md:1635` to `docs/hld/10-bindings-spec.md:1641`
  are rewrapped. The long prose lines in `CHANGELOG.md` are rewrapped. Only link
  line `CHANGELOG.md:284` exceeds 80 columns. Fixed.
- `docs/hld/03-architecture.md:995` to `docs/hld/03-architecture.md:996` now
  names the SHA directly. Fixed.

## Defects

None.

## Smells

### S1, The independent inventory check still cannot catch pass 1 D1 or D2, and misses pre-S73 records in the Rust ranges
`scripts/test_sprint_workflow.py:5485`

This is pass 1 S2, carried forward. The new cross-check builds `notified` as
the union of all four family inventories
(`scripts/test_sprint_workflow.py:5485` to
`scripts/test_sprint_workflow.py:5488`). It then only asserts
`cited - notified` is empty at `scripts/test_sprint_workflow.py:5529`. Two
gaps remain.

1. **Union only.** A record dropped from one family but present in another
   still passes. I reran the new logic against the pass 1 inventory, the one
   without 76 in `rpptx-v0.12.0` and without 94, 121, and 108 to 111 in
   `v0.14.0`. `cited - notified` is empty. So the D1 and D2 inventory still
   passes, which is the failure mode pass 1 S2 named.
2. **S73 scope only.** `sprint_fids` at `scripts/test_sprint_workflow.py:5495`
   comes from `docs/sprints/CURRENT_SPRINT.md`. But `v0.13.1..HEAD` and
   `rpptx-v0.11.0..HEAD` also contain the S72 stories F-X083 to F-X096. That
   includes F-X094e, the source of D1. The resulting `cited` set has no Issue
   69, 72, 73, 74, or 76 and no PR 71, 77, 78, 79, 80, or 114. Removing any of
   those records from every notes section and from the inventory together
   would still pass.

The AS_BUILT parser at `scripts/test_sprint_workflow.py:5526` collects only PR
numbers. It also stops at an Oxford-comma list: `PRs 108, 109, and 110`
yields only 108 and 109. No current S73 entry trips this, so it is latent.

A fix could derive the F-ID set from the commits in each family's tag range.
It could also check per family, for example by requiring every record cited by
a story whose commits touch `crates/rpptx*` or `crates/oxml-*` in the
`rpptx-v0.12.0` set. The other option is to narrow the plan and HLD claim to
union coverage of S73 citations.

## Nitpicks

- `CHANGELOG.md:419` to `CHANGELOG.md:421`: `py-rpptx-v0.12.0` credits the
  `Slide.notes_text` setter to Issue 120. The setter was requested in Issue
  121 under "rpptx: edit speaker notes", and Issue 120 defers notes editing to
  #121. Both are linked, so credit is complete. Only the pairing is off.
- `CHANGELOG.md:260` to `CHANGELOG.md:262`: "asked for the native editing
  helpers in Issue 94" overstates. Issue 94 is titled "Python bindings, round
  2", and PRs 109 and 110 changed only `crates/rdocx-py` and `docs/hld`. The
  native helpers came from the hardened F-X106a and F-X106c equivalents. The
  Added wording at `CHANGELOG.md:147` to `CHANGELOG.md:154` is accurate.
- `CHANGELOG.md:39` to `CHANGELOG.md:41`: "reproducible automation output
  requested in Issue 76" is loose. Issue 76 asked for rpptx Python rendering,
  not determinism.
- `CHANGELOG.md:366`: the paragraph rewrap leaves "again. This" as a short
  line before `CHANGELOG.md:367`. Rendered output is unaffected.
- `docs/hld/12-testing-strategy.md:2563` to
  `docs/hld/12-testing-strategy.md:2570` describes the S73 regression but not
  its new backlog and AS_BUILT citation cross-check at
  `scripts/test_sprint_workflow.py:5491` to
  `scripts/test_sprint_workflow.py:5529`.

## Not found

- **Version carrier completeness.** No current-state 0.13.2 or 0.11.0 carrier
  remains in manifests, `Cargo.lock`, READMEs, `.github/workflows`, `scripts`,
  or crate sources. Everything left is registry dependencies (`sha1`, `sha2`,
  `md-5`, `ego-tree`, `synstructure`) or published and immutable history.
  The lockfile has ten workspace packages at 0.14.0 (plus registry `cfb`) and
  17 at 0.12.0. There are eight stable pins, 16 incubating pins, and 17
  explicit incubating manifests. Renamed workflow test identifiers resolve to
  existing tests at `scripts/test_sprint_workflow.py:5154`,
  `scripts/test_sprint_workflow.py:5535`, and
  `scripts/test_sprint_workflow.py:6194`.
- **Immutable history.** Older `CHANGELOG.md` sections, the `v0.11.0`,
  `rpptx-v0.11.0`, `py-rdocx-v0.13.2`, and `py-rpptx-v0.11.0` tests, and HLD
  history paragraphs are unchanged in meaning.
- **Family ranges.** `py-rdocx-v0.13.2` and `py-rpptx-v0.11.0` share
  `2b009243`, and every S73 story lands after it, so both Python sections are
  S73 only. Both Rust ranges also contain S72 stories. Their records (Issues
  69 and 72 to 76, PRs 71 and 77 to 80) appear only in the Rust sections,
  which is correct because they shipped in the earlier Python tags.
- **Python compatibility.** Every `_rdocx.pyi` signature change since
  `py-rdocx-v0.13.2` is additive (`add_comment` widens to `StoryRunRange` and
  adds optional `date`, `reply_to` adds optional `date`). Every `_rpptx.pyi`
  change since `py-rpptx-v0.11.0` is additive.
- **Authenticated authors.** `gh` confirms `hadim` for Issues 94, 99, 100,
  120, and 121 and PRs 102, 105, and 108 to 114, `pedroassumpcao` for PR 122,
  and `chevinbrown` for PR 123. That is consistent with every `Contributors`
  section.
- **Test gate.** The test passes. `release-notes --check` passes for all four
  tags. The plan wording and the notes still agree with the inventory.
- **HLD consistency.** HLD 03, 10, 12, 14, and 15 state the same versions,
  tags, incubating-first order, and superseded 0.13.2 crates.io train. The
  duplicated "The The" in HLD 10 is gone.
- **Voice rules.** `prose_check` reports no violations in the changed gated
  Markdown or in the `a897aa84` commit message.
