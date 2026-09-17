# S73 sprint review, pass 5

**Reviewed**: `sprint/s73` at `fada4915` against `029065ca3546559f982900260878eab4ac73f1a4`, 198 files, 31,401 changed lines, crates: oxml-chart, oxml-drawing, oxml-layout, oxml-pdf, rdocx, rdocx-cli, rdocx-layout, rdocx-oxml, rdocx-py, rpptx, rpptx-cli, rpptx-layout, rpptx-py, rpptx-render
**Verdict**: 0 blocking, 0 should-fix, 2 nice-to-have

Bound extension: scheduled dependency-prefix boundary. Passes 1 to 3 closed
the earlier dependency-prefix boundaries clean at `a3be3d1f`, and passes 4 and
5 review the F-X114 record boundary, with this pass being its first
remediation pass after the pass 4 should-fix.

The delta since pass 4 is two commits. `bb5188c2` records the pass 4 review
file and `fada4915` changes only the two summary rows in
`docs/sprints/BACKLOG.md`. This pass still reviewed the whole integrated
sprint diff, with the closest reading on F-X114 at `e82dac08` and its ledger
records at `6694f268`.

## Blocking

None.

## Should-fix

None open.

### S1, BACKLOG summary counts disagree with the status rows, resolved
`docs/sprints/BACKLOG.md:43`

Resolved by `fada4915`. This pass recounted every milestone summary row from
its AUTOGEN table independently of the committed numbers, counting archived
rows in the total only. All 25 milestone rows match their tables. The X row at
`docs/sprints/BACKLOG.md:43` reports 131 F-IDs, 126 done, 0 in progress, and 1
pending, and its table holds 126 done, 4 archived, and 1 pending row. The four
archived rows are F-X030, F-X055, F-X060, and F-X078 at
`docs/sprints/BACKLOG.md:572`, `docs/sprints/BACKLOG.md:597`,
`docs/sprints/BACKLOG.md:602`, and `docs/sprints/BACKLOG.md:620`. The only
pending X row is F-X112 at `docs/sprints/BACKLOG.md:661`. The total at
`docs/sprints/BACKLOG.md:44` reports 449, 376, 0, and 69, which equals the sum
of the 25 recomputed rows. The table holds 449 unique F-ID rows with no
duplicate. M23 reports 24 of 24 done, which agrees with the S73 M23 rows. The
fix commit changes nothing else, and its message records the F-X104 and F-X105
cause that pass 4 identified.

## Nice-to-have

### N1, The F-X114 Word records measure preservation rather than Word's TOC generation
`crates/rdocx/tests/regression_test.rs:8498`

Re-evaluated and retained at the same severity. The ignored capture still opens
the rdocx-rebuilt document at `crates/rdocx/tests/regression_test.rs:8498` and
saves it at `crates/rdocx/tests/regression_test.rs:8500` without updating the
TOC field, so the pinned records show that Word accepts and retains rdocx's
style ids, tab stops, and structural suffix tabs. They cannot detect a
difference from entries Word would generate itself, which the plan's
differential row and `docs/hld/12-testing-strategy.md:381` describe. No
concrete user-visible defect follows. The regression gate proves the
contracted behavior directly, and the evidence wording is the only gap.

### N2, A created canonical TOC style uses an uppercase built-in name
`crates/rdocx/src/field.rs:5762`

Re-evaluated and retained at the same severity. A style created when no
`toc N` name and no `TOCN` id exist is still named `TOC N`, while the default
styles in `crates/rdocx-oxml/src/styles.rs:713` use lowercase built-in names
such as `heading 1`. The selector at `crates/rdocx/src/field.rs:5751` compares
names case-insensitively, so a repeated rebuild or the bookmark-rename
recursion reuses the created style and no duplicate appears. Before F-X114 the
entry paragraphs referenced an undefined `TOCN` id outright, so the created
style is not a regression. No user-visible defect was found.

## Milestone gate

The M23 gate at `docs/hld/14-development-backlog.md:2214`:

```text
all five private references are generated from a blank public facade, reopen
without repair, match the required package semantics, and meet the reviewed
deterministic visual thresholds. Repeated generation produces identical DOCX
bytes, and no generated document reports an unexplained preservation-only
fallback.
```

The gate holds for this boundary, on recorded evidence rather than a rerun.
Pass 3 recorded
`m23_private_from_scratch_corpus_passes_required_mode` at
`crates/rdocx/tests/integration_test.rs:2768` passing at `a809a244`. This pass
confirmed that the five local ignored generators contain no `rebuild_toc` call,
so F-X114 cannot change their output. `fada4915` changes only tracker
Markdown. No private corpus path appears in the sprint diff and no file under
the private corpus root is tracked. The sprint run state records a passing full
verification at `e82dac08` with the hash harness unchanged at 49 of 49, which
matches the F-X114 AS_BUILT entry and the `6694f268` trailer. This pass did not
rerun the private corpus or the workspace gate, because the shared target
directory was in use by a running verification. F-X112 publication remains
intentionally pending, so this verdict does not claim final sprint closure.

## Not found

- Interaction: `ensure_toc_entry_styles` at `crates/rdocx/src/field.rs:1060`
  runs on the staged candidate after source discovery and before bookmark
  staging. The flush writes styles inside the candidate, and a failed rebuild
  discards it. The bookmark-rename recursion at
  `crates/rdocx/src/field.rs:1131` reopens with the created style and reuses
  it by name. The section lookup at `crates/rdocx/src/field.rs:1188` indexes
  the reopened provisional body with `collect_body_paragraphs`, the same
  flattened order F-X103's span scan and source discovery use, and bookmark
  insertion adds no paragraph. The structural suffix at
  `crates/rdocx/src/field.rs:6391` keeps F-260 ordered run content. F-263
  layout-backed fields, F-X101 page breaks, F-X120 line spacing, and F-259
  measurement are untouched. No other generation path in `rdocx`,
  `rdocx-layout`, or `rdocx-py` still writes a fixed `TOCN` id or the old 9350
  twip stop.
- Duplication: `toc_section_text_width` at `crates/rdocx/src/field.rs:5783`
  and the legacy `text_width_twips` at `crates/rdocx/src/document.rs:19903`
  compute similar widths. Blame places the legacy helper before the merge
  base, so the sprint did not write the same helper twice.
- Layering: no `oxml-*` manifest names an `rdocx-*` or `rpptx-*` dependency.
  The only matches are release `tag-name` metadata strings.
- Harness: `scripts/hash_baseline.json` changed once in the sprint, in
  `295672fa` for F-X102, moving the three feature-showcase PDF fingerprints
  with a declared reason that matches its AS_BUILT entry. F-X114 and every
  later commit leave the baseline unchanged.
- Docs: the F-X114 HLD impact list matches the edits to HLD 04, 08, 12, and
  14, and the HLD 08 text at `docs/hld/08-rendering-spec.md:1083` describes
  the checked-geometry fallback the code implements. The plan is completed
  with every item ticked. AS_BUILT at `docs/sprints/AS_BUILT.md:15063`, the
  tracker row, the BACKLOG row at `docs/sprints/BACKLOG.md:663`, and
  CURRENT_SPRINT at `docs/sprints/CURRENT_SPRINT.md:72` all record F-X114
  done with owner `-`. The run state records it completed with integration
  commit `e82dac08`. F-X112 remains pending at
  `docs/sprints/CURRENT_SPRINT.md:80` and approved but not started in the run
  state, as intended. The microscope passes agree with the AS_BUILT
  remediation note.
- Dependencies: the only manifest changes in the sprint are the
  `crates/rdocx-cli/Cargo.toml` and `crates/rpptx-cli/Cargo.toml` binstall
  metadata and `system-fonts` default features, whose consumer is F-X111. No
  manifest changed after pass 3.
- Surface: F-X114 adds no public native, Python, WASM, or CLI item. Every new
  function and struct in `crates/rdocx/src/field.rs` is private.
