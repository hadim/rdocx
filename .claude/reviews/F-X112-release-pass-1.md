# F-X112, release, pass 1

**Reviewed**: `git diff aba94e0c..564471a7`, 60 files, 829 insertions and 222
deletions. Version carriers for the stable 0.14.0 and incubating 0.12.0
families, both Python projects, four `CHANGELOG.md` sections, the new S73
release contract test, renamed publish preflight tests, and HLD 03, 10, 12,
14, 15, `CLAUDE.md`, and the design plan.
**Verdict**: 5 defects, 3 smells, 5 nitpicks

## Defects

### D1, `rpptx-v0.12.0` omits Issue 76 and the native PNG helpers it added
`CHANGELOG.md:17`

F-X094e is linked to Issue 76 (`docs/hld/14-development-backlog.md`, F-X094e
`**GitHub issue**`) and landed in commit `4f564c9b` on 2026-09-14, after the
`rpptx-v0.11.0` tag at `0b6bd622`. It added public native API to the `rpptx`
crate, `Presentation::slide_png_deterministic` and `slide_pngs_deterministic`
at `crates/rpptx/src/lib.rs:1179` and `crates/rpptx/src/lib.rs:1186`. Those
methods first reach crates.io in 0.12.0. The Added section at
`CHANGELOG.md:17` to `CHANGELOG.md:40` does not mention them, and Issue 76 is
absent from the section and from the `rpptx-v0.12.0` inventory at
`scripts/test_sprint_workflow.py:5350`. `/release-notes` requires every
included issue to appear as a direct link with contributor credit, so the
`@hadim` credit at `CHANGELOG.md:82` is also incomplete.

### D2, `v0.14.0` omits Issue 121 and its native Word APIs
`CHANGELOG.md:111`

F-X119 (Issue 121, commit `be1fbea7`) added public native `rdocx` API:
`Document::insert_picture_to_story` at `crates/rdocx/src/document.rs:12965`,
`add_story_comment` and `add_story_comment_with_date` at
`crates/rdocx/src/comments.rs:514` and `crates/rdocx/src/comments.rs:530`, and
the exported `StoryRunPosition` and `StoryRunRange` types. The stable Added
section at `CHANGELOG.md:111` to `CHANGELOG.md:148` does not mention them, and
Issue 121 is absent from the `v0.14.0` inventory at
`scripts/test_sprint_workflow.py:5343`. The treatment is inconsistent with
`rpptx-v0.12.0`, which credits Issue 121 for the analogous native presentation
APIs from the same F-ID at `CHANGELOG.md:29` to `CHANGELOG.md:31`. The same
classification question applies to Issue 94 and PRs 108 to 111, whose F-X106a,
F-X106b, and F-X106c commits also added public native helpers such as
`set_highlight_value`, `highlight_color`, `find_paragraph_indices`, and
`content_index_of_paragraph`. Either credit them in `v0.14.0` or record why
they belong only to the Python family.

### D3, `v0.14.0` Compatibility misstates the `rdocx replace` change
`CHANGELOG.md:210`

The section says no migration action is required and describes only the
expected-count check. In published `v0.13.1`, `rdocx replace` wrote with
`doc.save(output)` (`git show v0.13.1:crates/rdocx-cli/src/commands.rs`, line
336), which overwrote an existing output, including the input file. It now
publishes through `publish_document` at `crates/rdocx-cli/src/commands.rs:959`,
which calls `stage_and_publish` at `crates/rdocx-cli/src/commands.rs:1134`.
`ensure_output_paths_available` at `crates/oxml-cli-support/src/lib.rs:121`
refuses any path that already exists. A script that ran
`rdocx replace in.docx ... -o in.docx`, or reran into an existing output, now
fails. The same change is disclosed for `rpptx replace` at
`CHANGELOG.md:72` to `CHANGELOG.md:75` but not for the stable CLI.

### D4, `py-rpptx-v0.12.0` claims a Python replacement API that does not exist
`CHANGELOG.md:394`

The note says Python can "count notes matches in presentation replacement". The
`rpptx` Python surface has no presentation text replacement. A search for
`replace` in `crates/rpptx-py/python/rpptx/_rpptx.pyi` and
`crates/rpptx-py/src` finds no match. The only notes mutation is the
`Slide.notes_text` setter at `crates/rpptx-py/python/rpptx/_rpptx.pyi:118` to
`crates/rpptx-py/python/rpptx/_rpptx.pyi:120`. The counted replacement lives
only in native `Presentation::replace_text` and the `rpptx` CLI, which belong
to `rpptx-v0.12.0`. `/release-notes` refuses a note that belongs only to the
other family.

### D5, `py-rpptx-v0.12.0` credits PR 123 chart defaults with no Python chart surface
`CHANGELOG.md:412`

PR 123 (F-X121, commit `97bb2198`) changed only `crates/oxml-chart/src/lib.rs`
and one `crates/rdocx/src/document.rs` line. Neither Python stub file exposes
chart authoring. A case-insensitive search for `chart` in
`crates/rpptx-py/python/rpptx/_rpptx.pyi` and `crates/rpptx-py/src` finds no
match. The `py-rpptx-v0.12.0` Fixed entry at `CHANGELOG.md:412` to
`CHANGELOG.md:414`, the `@chevinbrown` credit at `CHANGELOG.md:435` to
`CHANGELOG.md:436`, and the inventory at `scripts/test_sprint_workflow.py:5362`
pin a native-only outcome into the Python family. `py-rdocx-v0.14.0` correctly
leaves PR 123 out for the same reason.

## Smells

### S1, `rpptx-v0.12.0` Compatibility omits the widened `Presentation::replace_text` scope
`CHANGELOG.md:69`

Native `Presentation::replace_text` at `crates/rpptx/src/lib.rs:1678` to
`crates/rpptx/src/lib.rs:1699` now also rewrites speaker-notes shapes and adds
their matches to the returned count. At `rpptx-v0.11.0` it touched only the
slide shape tree. Existing Rust callers get changed notes content and larger
counts without any code change. The Added bullet at `CHANGELOG.md:24` mentions
the counting, but the Compatibility paragraph says the new APIs are additive
and no migration action is required, which does not cover an existing method
whose behaviour changed.

### S2, The test gate's inventory is self-referential and cannot catch an omitted record
`scripts/test_sprint_workflow.py:5342`

`S73_RELEASE_INVENTORY` is a hand-written copy of the links now in the notes,
and `assert_s73_release_notes_truth_contract` only proves that the notes equal
that copy. The completeness loop at `scripts/test_sprint_workflow.py:5489` to
`scripts/test_sprint_workflow.py:5493` checks the same constant against
literals in the same test, so it is tautological. Nothing ties the inventory
to repository evidence, for example the `**GitHub issue**` and
`**GitHub pull request**` links of the F-IDs listed in F-X112 `Depends on`, so
D1 and D2 pass the gate. The HLD statement at
`docs/hld/12-testing-strategy.md:2563` to `docs/hld/12-testing-strategy.md:2568`
that the sets "equal the reviewed contribution inventory" is only as strong as
this constant. The test does fail against the pre-change tree, because the
workspace version assertion and the missing `CHANGELOG.md` headings both fail.

### S3, The plan's scope still says through F-X119 while the backlog and notes include F-X120 and F-X121
`.claude/plans/F-X112-design.md:6`

The plan's `Depends on` at `.claude/plans/F-X112-design.md:6`, its notification
scope at `.claude/plans/F-X112-design.md:39`, and its test-plan row at
`.claude/plans/F-X112-design.md:55` all stop at F-X119.
`docs/hld/14-development-backlog.md:5348` depends on F-X113 through F-X121.
The notes and inventory include PR 122 (F-X120) and PR 123 (F-X121). The new
test pins the stale phrase with `assertIn("through F-X119", ...)` at
`scripts/test_sprint_workflow.py:5484` while also requiring PRs 122 and 123 at
`scripts/test_sprint_workflow.py:5491`. This diff edited the plan but left the
contract contradicting its own inventory.

## Nitpicks

- `CHANGELOG.md:316`, a `py-rdocx-v0.14.0` Fixed entry reports a
  `Table.clone_row` namespace defect, but `clone_row` is new in this release, so
  users never saw a broken version. It reads better folded into the Added
  bullet at `CHANGELOG.md:290`.
- `CHANGELOG.md:408`, "pictures larger than 16 MiB render" overstates F-X117.
  Decoding is capped at a checked 64 MiB, as `CHANGELOG.md:45` to
  `CHANGELOG.md:46` states correctly for the Rust family.
- `CHANGELOG.md:313`, `Paragraph.runs` now includes runs inside tracked
  insertions and inline content controls. That can shift run indexes in
  existing Python code, while Compatibility at `CHANGELOG.md:340` says only
  that calls remain source compatible.
- `docs/hld/03-architecture.md:979` is 108 columns and
  `docs/hld/10-bindings-spec.md:1635` is 83, left unwrapped by the edit. The
  same applies to `CHANGELOG.md:24`, `CHANGELOG.md:82`, `CHANGELOG.md:212`, and
  `CHANGELOG.md:260`.
- `docs/hld/03-architecture.md:995`, "at the same reviewed SHA" now follows the
  inserted sentence about the superseded 0.13.2 crates.io train, so its
  antecedent, the `py-rdocx-v0.13.2` SHA two sentences earlier, is harder to
  find.

## Not found

- **Version carrier completeness.** All 27 lockfile entries moved (10 stable,
  17 incubating). All 17 explicit incubating manifests, 16 incubating and 8
  stable `[workspace.dependencies]` pins, `[workspace.package].version`, both
  `pyproject.toml` files, every crate README install line, the root README, the
  CI WASM literals, `scripts/readme_doctests.py`, all Rust source manifest
  assertions, and the renamed `publish.yml` test identifiers agree. No
  current-state 0.13.2 or 0.11.0 carrier remains outside history.
- **Historical records.** The immutable v0.11.0, `rpptx-v0.11.0`,
  `py-rdocx-v0.13.2`, and `py-rpptx-v0.11.0` tests, older `CHANGELOG.md`
  sections, and HLD history paragraphs were correctly left unchanged.
- **Authenticated authors.** `gh` confirms `hadim` for Issues 72 to 76, 83 to
  86, 88 to 100, and 115 to 121 and for PRs 101 to 114, `emptinessform` for
  Issue 69, `chevinbrown` for PRs 71 and 123, and `pedroassumpcao` for PRs 77
  to 80 and 122.
- **Excluded candidates.** PR 70 was superseded by PR 71. PR 81 is test and CI
  infrastructure, whose diagnosis was addressed in `88a9f253`. Issues 65 to 67
  belong to earlier stable releases. Issue 68 is an unanswered question and
  Issue 87 is a thank-you. None is a missing user-visible record.
- **`rpptx replace` Compatibility claim.** It matches
  `crates/rpptx-cli/src/commands.rs:285` to
  `crates/rpptx-cli/src/commands.rs:311`.
- **API names.** `update_page_fields`, `update_layout_backed_fields`,
  `update_fields_on_open`, `Table.clone_row`, `story_items`, `hyperlinks`,
  `RunPosition`, and `split_run` exist in the native source and
  `crates/rdocx-py/python/rdocx/_rdocx.pyi`.
- **Release notes format.** `release-notes --check` passes for all four tags,
  and the three family preparation tests plus the new gate pass.
- **HLD consistency.** The release order, incubating first, is stated
  consistently in the plan, HLD 10, 12, 14, and 15. The 03 and 15 carrier
  counts match the manifests.
- **Voice rules.** No em-dash, en-dash, or prose semicolon in any changed gated
  Markdown or in the new `CHANGELOG.md` sections.
