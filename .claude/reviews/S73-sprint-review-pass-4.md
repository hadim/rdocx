# S73 sprint review, pass 4

**Reviewed**: `sprint/s73` against `029065ca3546559f982900260878eab4ac73f1a4`, 197 files, 31,284 changed lines, crates: oxml-chart, oxml-drawing, oxml-layout, oxml-pdf, rdocx, rdocx-cli, rdocx-layout, rdocx-oxml, rdocx-py, rpptx, rpptx-cli, rpptx-layout, rpptx-py, rpptx-render
**Verdict**: 0 blocking, 1 should-fix, 2 nice-to-have

Bound extension: scheduled dependency-prefix boundary. Passes 1 to 3 closed
the earlier dependency-prefix boundaries clean at `a3be3d1f`, and this pass
reviews the F-X114 record boundary at `6694f268` before the F-X112 release
boundary.

## Blocking

None.

## Should-fix

### S1, BACKLOG summary counts disagree with the status rows
`docs/sprints/BACKLOG.md:43`

The AUTOGEN summary reports the cross-cutting milestone as 131 F-IDs with 124
done and 3 pending, and the total at `docs/sprints/BACKLOG.md:44` as 374 done
and 71 pending. The status rows at HEAD contain 126 done, 4 archived, and 1
pending F-X row, and only F-X112 at `docs/sprints/BACKLOG.md:661` is pending.
The correct counts are 126 done and 1 pending for the X row, and 376 done and
69 pending for the total. The drift began when the F-X104 and F-X105
completion commits `96f7935f` and `06724d97` changed their rows at
`docs/sprints/BACKLOG.md:651` and `docs/sprints/BACKLOG.md:652` without
regenerating the counts. Every later ledger commit, including the F-X114 record
at `6694f268`, adjusted the stale numbers incrementally, so the error survived
passes 1 to 3. The file states the counts are regenerated from the rows, and
`/sync-status` check 4 would report the mismatch. A fix must recompute the X
and total rows from the table rather than apply another delta.

## Nice-to-have

### N1, The F-X114 Word records measure preservation rather than Word's TOC generation
`crates/rdocx/tests/regression_test.rs:8498`

The ignored capture opens the rdocx-rebuilt document in Word and saves it at
`crates/rdocx/tests/regression_test.rs:8500` without updating the TOC field.
The pinned records therefore prove that Word accepts and retains rdocx's style
ids, tab stops, and structural suffix tabs. They cannot detect a difference from
the entries Word itself would generate. The plan's differential row expects
Word to agree on style selection and tab placement, and
`docs/hld/12-testing-strategy.md:381` describes the records as pinned Word
differential evidence. The regression gate itself holds. The choice at
`docs/hld/08-rendering-spec.md:1085` that a style-owned right tab suppresses the
direct tab is the behavior an update-fields capture would validate. Either
record the evidence as a Word round trip or add a field update step to a later
oracle.

### N2, A created canonical TOC style uses an uppercase built-in name
`crates/rdocx/src/field.rs:5762`

When no style carries the name `toc N` and no `TOCN` id exists, the rebuild
creates the style with the name `TOC N`. The repository writes Word built-in
style names in lowercase elsewhere, such as `heading 1` at
`crates/rdocx-oxml/src/styles.rs:713`, and the F-X114 contract selects by
`w:name="toc N"`. rdocx's own lookup is case-insensitive, so a repeated rebuild
reuses the style and no duplicate appears. Writing `toc N` would keep created
styles consistent with the built-in naming convention and the documented
selector.

## Milestone gate

The M23 gate at `docs/hld/14-development-backlog.md:2214`:

```text
all five private references are generated from a blank public facade, reopen
without repair, match the required package semantics, and meet the reviewed
deterministic visual thresholds. Repeated generation produces identical DOCX
bytes, and no generated document reports an unexplained preservation-only
fallback.
```

The gate holds for this boundary. Pass 3 recorded the required-private test at
`crates/rdocx/tests/integration_test.rs:2768` passing at `a809a244`. F-X114
changes only `rebuild_toc()` and its private helpers in
`crates/rdocx/src/field.rs`. A search of the five local ignored generators
found no call to `rebuild_toc`, so F-X114 cannot change their output, and the
private corpus evidence remains applicable. This pass did not rerun the private
corpus. The sprint run state records a passing full verification at
`e82dac08` with the hash harness unchanged at 49 of 49, which agrees with the
F-X114 AS_BUILT entry and the ledger commit trailer. F-X112 publication remains
intentionally pending, so this verdict does not claim final sprint closure.

## Not found

- Interaction: F-X114 consumes F-X103's span, source, and bookmark discovery
  without changing it. Paragraph indices for section lookup at
  `crates/rdocx/src/field.rs:5783` come from the same flattened body order used
  by `discover_toc_sources`. Styles are resolved and flushed inside the staged
  candidate before bookmark staging, so the bookmark rename path that reopens
  and recurses finds the created style by name and creates no duplicate. A
  failed rebuild discards the candidate. The structural suffix at
  `crates/rdocx/src/field.rs:6396` keeps F-260 ordered run content intact, and
  the PAGEREF placeholder substitution, F-263 layout-backed fields, F-X120 line
  spacing, and F-259 measurement are untouched.
- Duplication: the pre-sprint `text_width_twips` at
  `crates/rdocx/src/document.rs:19903` serves the legacy TOC insertion path.
  The new section-aware helper is not a second helper written within the
  sprint, and no other sprint-local style resolver or section geometry helper
  was added.
- Layering: F-X114 touches only `rdocx`, and no `oxml-*` crate gained an
  `rdocx-*` or `rpptx-*` dependency.
- Harness: no baseline moved in the F-X114 delta. The declared F-X102
  feature-showcase PDF delta remains the only sprint baseline movement.
- Docs: the F-X114 HLD impact list matches the edits to HLD 04, 08, 12, and
  14. The plan is completed with every checklist item ticked. AS_BUILT, the
  tracker row, the BACKLOG row, CURRENT_SPRINT with owner `-`, and the run
  state all record F-X114 done, and F-X112 remains the only pending S73 row.
  The two microscope passes agree with the AS_BUILT remediation note. The
  summary count drift is recorded as S1.
- Dependencies: no manifest changed after pass 3. The CLI manifest changes keep
  the named consumers reviewed in pass 2.
- Surface: F-X114 adds no public native, Python, WASM, or CLI API. Every new
  item is a private helper or struct in `crates/rdocx/src/field.rs`.
