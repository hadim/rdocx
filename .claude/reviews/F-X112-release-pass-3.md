# F-X112, release, pass 3

**Reviewed**: `git diff aba94e0c..3043785e`, 62 files, 1314 insertions and 233
deletions. This covers the whole feature diff again, including remediation
commit `3043785e` (4 files, 279 insertions, 48 deletions, of which 190 are the
pass 2 review file). Review files are excluded from the scope.
**Verdict**: 2 defects, 1 smell, 2 nitpicks

## Pass 2 findings

### S1, confirmed fixed for the two named gaps

`scripts/test_sprint_workflow.py:5369` to `scripts/test_sprint_workflow.py:5391`
adds `S73_RELEASE_STORIES`, and `scripts/test_sprint_workflow.py:5547` to
`scripts/test_sprint_workflow.py:5557` checks each mapped story's citations
against its own family inventory. The union-only gap is gone, and S71 and S72
stories are now in scope. I mutation-tested the gate in a temporary copy of
`3043785e`. The baseline passes. Each of these mutations, applied to both the
inventory and the rendered links of one family, now fails:

- Issue 76 dropped from `rpptx-v0.12.0` (pass 1 D1). Fails on F-X094e.
- Issues 94 and 121 and PRs 108 to 111 dropped from `v0.14.0` (pass 1 D2).
  Fails on F-X106a.
- Issue 69 dropped from `v0.14.0`. Fails on F-X088, an S71 story.
- Issue 75 dropped from `py-rdocx-v0.14.0`. Fails.
- PR 122 dropped from `py-rdocx-v0.14.0`. Fails on F-X120.

The Oxford-comma form is now accepted at `scripts/test_sprint_workflow.py:5539`.
The map itself is hand-maintained. Dropping a story from the map together with
its only unique record still passes, for example F-263 with Issue 93 from
`v0.14.0`. The HLD states this honestly as "a reviewed map" at
`docs/hld/12-testing-strategy.md:2568`, so I checked the map by hand against
every story in each tag range. See D1 and S1 below for the two misses.

### Pass 2 nitpicks

- Issue 121 is now credited for the `Slide.notes_text` setter at
  `CHANGELOG.md:423` to `CHANGELOG.md:426`. Fixed.
- The Issue 94 credit at `CHANGELOG.md:261` to `CHANGELOG.md:263` now says it
  requested the Python editing surface. Fixed.
- The Issue 76 credit at `CHANGELOG.md:39` to `CHANGELOG.md:42` now says
  presentation rendering automation. Fixed.
- The short line is rewrapped at `CHANGELOG.md:369` to `CHANGELOG.md:371`.
  Fixed.
- `docs/hld/12-testing-strategy.md:2568` to
  `docs/hld/12-testing-strategy.md:2571` now describes the per-family
  cross-check. Fixed.

## Defects

### D1, F-263 is missing from the `rpptx-v0.12.0` story map, and its outcome is noted there without Issue 93
`scripts/test_sprint_workflow.py:5379`

I compared each map entry with the F-ID commits in each previous-tag range.
F-263 (Issue 93, `@hadim`) changed the incubating family after
`rpptx-v0.11.0` in `5ffd0492`. It added the public `oxml-layout` type
`FieldSource` at `crates/oxml-layout/src/output.rs:121` and new public
`field_source` fields at `crates/oxml-layout/src/line.rs:176`,
`crates/oxml-layout/src/output.rs:235`, and
`crates/oxml-layout/src/output.rs:263`. The `rpptx-v0.12.0` notes claim this
outcome at `CHANGELOG.md:43` to `CHANGELOG.md:44` ("Layout exposes stable field
provenance"), but the section never links
[Issue 93](https://github.com/tensorbee/rdocx/issues/93). The `@hadim` credit at
`CHANGELOG.md:88` to `CHANGELOG.md:98` omits it too. F-X101 is the same kind of
shared `oxml-layout` change behind a Word issue, and it is mapped and linked
(Issue 88 and PR 102 at `CHANGELOG.md:63` to `CHANGELOG.md:65`), so the
treatment is inconsistent. Adding F-263 to the rpptx map in the temporary copy
fails on Issue 93. `/release-notes` refuses an included record that is not
linked in the rendered notes. Either map F-263 into `rpptx-v0.12.0` and link and
credit Issue 93, or drop the provenance bullet from the incubating notes.

Every other exclusion checks out. Stories left out of a family either touch
none of its crates or cite only records already in its inventory (F-X094a to
F-X094d and F-X096 cite Issue 76). Maintainer verification PR 82
(`@mantissaman`, never merged, workflow files only) is correctly left out.
F-X081 to F-X083 and F-X088 change no crate after either Rust tag.

### D2, both Rust Compatibility sections call the release additive, but exhaustive public structs gained fields
`CHANGELOG.md:227`

`v0.14.0` says "The native Rust APIs remain additive pre-1.0 surfaces and no
migration action is required for existing library callers" at
`CHANGELOG.md:227` to `CHANGELOG.md:228`. `rpptx-v0.12.0` says "The new Rust APIs
are additive pre-1.0 surfaces" at `CHANGELOG.md:73`. Both are false for callers
who build these structs with full struct literals.

- `ChartData` gained `category_axis_title`, `value_axis_title`, and `palette`
  at `crates/oxml-chart/src/lib.rs:144` to `crates/oxml-chart/src/lib.rs:146`
  (F-X087). It is re-exported by the stable facade at
  `crates/rdocx/src/lib.rs:89` and by `crates/rpptx/src/lib.rs:22`. At
  `v0.13.1` and `rpptx-v0.11.0` it had no `Default` derive, so every existing
  caller used a full literal and fails to compile against 0.14.0 or 0.12.0.
- `LayoutLine.forced_break_after` at `crates/oxml-layout/src/line.rs:258`
  (F-X101), and the `field_source` fields on `TextSegment`, `GlyphRun`, and
  `MultilingualGlyphRun` (F-263, cited in D1).
- `ResolvedImage.opacity` at `crates/rpptx-layout/src/lib.rs:226` (F-X104).
- In the stable family, `CT_Style`, `CT_Lvl`, `CT_Num`, `CT_PPr`, `CT_SectPr`,
  `CT_TcPr`, and the anchor and inline drawing types in `rdocx-oxml`, plus
  `TocRebuildReport` in `rdocx`, all gained public fields after `v0.13.1`. None
  is `#[non_exhaustive]`.

The HLD already names these as intentional pre-1.0 source breaks at
`docs/hld/10-bindings-spec.md:129` to `docs/hld/10-bindings-spec.md:130`
(`ChartData`), `docs/hld/10-bindings-spec.md:524` to
`docs/hld/10-bindings-spec.md:525` (`CT_SectPr`),
`docs/hld/10-bindings-spec.md:928` to `docs/hld/10-bindings-spec.md:929`
(`field_source`), and `docs/hld/10-bindings-spec.md:1261` to
`docs/hld/10-bindings-spec.md:1268` (numbering types, including removed `Copy`
impls). The changelog has disclosed this kind of change before, at
`CHANGELOG.md:1100` to `CHANGELOG.md:1102` (`rpptx-v0.8.0`) and
`CHANGELOG.md:1184` to `CHANGELOG.md:1187` (`v0.11.0`). Both S73 Rust sections
should list the struct-literal breaks and say that callers must set the new
fields or use `Default` where it now exists.

## Smells

### S1, the Word PNG straight-alpha fix from F-X117 ships in `v0.14.0` and `py-rdocx-v0.14.0` but is unmapped and unnoted there
`scripts/test_sprint_workflow.py:5376`

F-X117 (Issue 119) changed the shared raster image path to premultiply
straight-alpha pixels at `crates/oxml-pdf/src/raster.rs:1063`. Word PNG
rendering reaches that path too: `Document::render_page_to_png_with_options`
calls `oxml_pdf::render_page_to_png` at `crates/rdocx/src/document.rs:20889`,
which draws every `PositionedElement::Image` through `render_image` at
`crates/oxml-pdf/src/raster.rs:380`. The Python `render_page_to_png` and
`render_pages` methods at `crates/rdocx-py/python/rdocx/_rdocx.pyi:434` to
`crates/rdocx-py/python/rdocx/_rdocx.pyi:440` expose the same path. So DOCX
pictures with an alpha channel composite differently in 0.14.0. The fix is
noted only for presentations at `CHANGELOG.md:48` to `CHANGELOG.md:51`. F-X117
is absent from the `v0.14.0` map at `scripts/test_sprint_workflow.py:5376` and
the `py-rdocx-v0.14.0` map at `scripts/test_sprint_workflow.py:5386`. Compare
F-X092, whose shared `oxml-pdf` fix is noted in both Rust families. Issue 119
is still notified through the rpptx families, so no contributor goes
unnotified. That makes this a smell and not a defect. The 64 MiB ceiling part
of F-X117 is rpptx-only and correctly stays out.

## Nitpicks

- `scripts/test_sprint_workflow.py:5539`: "PRs 77 through 80" yields only 77
  and 80 because the `through` range is not expanded. No mapped story depends
  on the AS_BUILT form today, because the F-X095 backlog entry links all four.
- `CHANGELOG.md:344` to `CHANGELOG.md:347`: "comparison with locally declared
  drawing namespaces" inverts F-X097, which keeps namespace bindings inherited
  from ancestor scopes. Issue 115 is also grouped under that phrase, but it
  concerns appended terminal paragraphs (F-X113).

## Not found

- **Version carriers.** `3043785e` touches no carrier. Re-grepping manifests,
  `pyproject.toml` files, workflows, and `scripts/readme_doctests.py` finds only
  the registry pins `sha1` and `sha2` at 0.11.0. Pass 2's carrier inventory
  still holds.
- **Release-note validity.** `release-notes --check` passes for `v0.14.0`,
  `rpptx-v0.12.0`, `py-rdocx-v0.14.0`, and `py-rpptx-v0.12.0`. The new Issue 75
  entry in `py-rdocx-v0.14.0` is backed by F-X097, which cites Issue 75 and
  landed after `py-rdocx-v0.13.2`. `gh` confirms `hadim` as the author.
- **Python family maps.** Both Python maps match what their bindings ship.
  F-X101, F-263, and F-X121 change no behaviour reachable from `rpptx`. Their
  `rpptx-render` and `rpptx` edits only fill or pass along the new
  `forced_break_after` and `field_source` fields. Neither stub file exposes
  charts. F-X111 is CLI-only.
- **Contributor credit.** Apart from D1, every linked record appears in its
  section's `Contributors` text, and the handle sets match the inventory.
- **HLD and plan consistency.** HLD 03, 10, 12, 14, and 15 and the plan still
  agree on versions, tags, incubating-first order, and the through-F-X121
  inventory. The new HLD 12 paragraph matches the test.
- **Voice rules.** `prose_check` reports 0 violations for the tree and for the
  `3043785e` commit message.
