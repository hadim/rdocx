# F-X112, release, pass 4

**Reviewed**: `git diff aba94e0c..6dcb8bf0`, 63 files, 1503 insertions and 233
deletions. This covers the whole feature diff again, including remediation
commit `6dcb8bf0` (3 files, 214 insertions, 25 deletions, of which 168 are the
pass 3 review file). Review files are excluded from the scope. The Rust and
Python public surfaces were diffed against `v0.13.1`, `rpptx-v0.11.0`,
`py-rdocx-v0.13.2`, and `py-rpptx-v0.11.0`.
**Verdict**: 1 defect, 1 smell, 2 nitpicks

## Pass 3 findings

### D1, confirmed fixed

`scripts/test_sprint_workflow.py:5379` now maps F-263 into `rpptx-v0.12.0`, and
the rpptx inventory at `scripts/test_sprint_workflow.py:5351` includes Issue 93.
The Added bullet at `CHANGELOG.md:43` to `CHANGELOG.md:46` names `FieldSource`
and links Issue 93, and the `@hadim` credit links it at `CHANGELOG.md:98`. In a
temporary export of `6dcb8bf0`, removing Issue 93 from both the rpptx inventory
and the rpptx section fails on `tag='rpptx-v0.12.0', story='F-263'`.

### D2, partially fixed

The items pass 3 named are now disclosed. `TocRebuildReport` losing
`diagnostic_count` and the `ChartData` and `CT_*` field additions appear at
`CHANGELOG.md:236` to `CHANGELOG.md:240`. `ChartData`, `LayoutLine`,
`field_source`, and `ResolvedImage` appear at `CHANGELOG.md:75` to
`CHANGELOG.md:79`. The independent API diff below finds further source breaks
of the same kind that are still not disclosed. They are recorded as D1 and S1 of
this pass.

### S1, confirmed fixed

F-X117 is mapped at `scripts/test_sprint_workflow.py:5376` and
`scripts/test_sprint_workflow.py:5386`, and Issue 119 is in both inventories.
The claim holds. `Document` PNG rendering calls `oxml_pdf::render_page_to_png`
at `crates/rdocx/src/document.rs:20889`, and the raster path premultiplies at
`crates/oxml-pdf/src/raster.rs:1063`. The notes say so at `CHANGELOG.md:197`
and `CHANGELOG.md:361`, with credit at `CHANGELOG.md:273` and
`CHANGELOG.md:412`. Removing Issue 119 from the `v0.14.0` inventory and section
fails on `story='F-X117'`. Removing it from the `py-rdocx-v0.14.0` inventory
fails too.

### Pass 3 nitpicks

- `scripts/test_sprint_workflow.py:5541` to
  `scripts/test_sprint_workflow.py:5543` expands `through` ranges. The same
  parsing applied to "PRs 77 through 80 and 82" yields 77, 78, 79, 80, and 82.
  Fixed.
- `CHANGELOG.md:356` to `CHANGELOG.md:360` now says namespaces declared on
  ancestor elements and moves Issue 115 to appended paragraphs. Fixed.

## Defects

### D1, the `v0.14.0` Compatibility section still omits concrete source breaks in the facade and `rdocx-oxml`
`CHANGELOG.md:236`

The section says "Most native Rust APIs are additive, with a few pre-1.0 source
changes" and lists only `TocRebuildReport.diagnostics` and new struct fields.
`/release-notes` requires Compatibility to say what a caller must change.
Diffing the seven selected crates against `v0.13.1` finds these undisclosed
breaks:

- `ParagraphItemRef::CommentRangeStart(i32)` and `CommentRangeEnd(i32)` became
  struct variants with `id` and `has_child_content` at
  `crates/rdocx/src/paragraph.rs:42` and `crates/rdocx/src/paragraph.rs:49`.
  `BookmarkStart` and `BookmarkEnd` gained `has_child_content` at
  `crates/rdocx/src/paragraph.rs:56` and `crates/rdocx/src/paragraph.rs:65`.
  The enum is `#[non_exhaustive]`, but that does not protect variant patterns.
  A caller matching `ParagraphItemRef::CommentRangeStart(id)` or
  `BookmarkEnd { id }` no longer compiles. This came from F-X095.
- `ListLevel` lost `Copy`, `PartialEq`, and `Eq` at
  `crates/rdocx/src/document.rs:22354`, and `ListNumberFormat` lost `Copy` and
  grew from 7 to 61 variants without `#[non_exhaustive]` at
  `crates/rdocx/src/document.rs:21995` (F-247). Implicit copies, equality
  comparisons, and exhaustive matches break.
- `TocRebuildReport` also lost `Copy` at `crates/rdocx/src/field.rs:673`. The
  notes mention only the field swap.
- `rdocx::Error` gained `Story` at `crates/rdocx/src/error.rs:58` and is not
  `#[non_exhaustive]` (F-253).
- In the public `rdocx_oxml::drawing` module, `CT_Anchor::from_xml` and
  `CT_Inline::from_xml` now take `&mut NsReader<&[u8]>` instead of
  `&mut Reader<&[u8]>` at `crates/rdocx-oxml/src/drawing.rs:506` and
  `crates/rdocx-oxml/src/drawing.rs:969`, and `parse_alternate_content` gained
  an `inherited_prefixes` parameter at `crates/rdocx-oxml/src/drawing.rs:345`
  (F-249). These are changed function signatures, not new fields.

The HLD already records the numbering breaks as intentional pre-1.0 source
breaks, including the removed `Copy` impls and exhaustive-match updates, at
`docs/hld/10-bindings-spec.md:1261` to `docs/hld/10-bindings-spec.md:1268`. The
release notes therefore understate what the spec itself says callers must
change. Name these breaks and the caller action for each, or summarise them by
kind (changed variant shapes, removed `Copy`, new enum variants, changed
low-level parser signatures) with the principal types named.

## Smells

### S1, the `rpptx-v0.12.0` Compatibility list omits `Blip.alpha_modulation_fix` and a new `OpcError` variant
`CHANGELOG.md:75`

The section lists the structs that gained fields (`ChartData`, `LayoutLine`,
the field glyph runs, `ResolvedImage`). Diffing the 15 selected crates against
`rpptx-v0.11.0` finds two more of the same kind:

- `oxml_drawing::fill::Blip` gained `pub alpha_modulation_fix` at
  `crates/oxml-drawing/src/fill.rs:613` (F-X104). It is not
  `#[non_exhaustive]`, so full struct literals break. It derives `Default`,
  which limits the impact.
- `OpcError` gained `DuplicatePartName` at `crates/oxml-opc/src/error.rs:23`
  (F-249). The enum is not `#[non_exhaustive]`, so exhaustive matches break.

The other fields found by the diff are already covered or harmless.
`PageFrame.displayed_page_number` sits on a `#[non_exhaustive]` struct at
`crates/oxml-layout/src/output.rs:417`. This is a smell rather than a defect
because both types are low-level and rarely constructed or matched by callers.
Still, the list reads as complete and is not.

## Nitpicks

- `CHANGELOG.md:98` to `CHANGELOG.md:99`: Issue 93 is inserted before Issues 91
  and 92, breaking the ascending order used in every other contributor list.
- `CHANGELOG.md:381` to `CHANGELOG.md:384`: "Existing 0.13.2 calls remain
  source compatible" is followed immediately by the `TocRebuildReport`
  constructor keyword change, which is a 0.13.2 call that no longer works.
  "Other existing 0.13.2 calls" would make the two sentences agree.

## Not found

- **Python compatibility.** The `_rdocx.pyi` diff against `py-rdocx-v0.13.2`
  removes or changes only the `TocRebuildReport` constructor, which is disclosed.
  `add_comment` widens `range` and `reply_to` adds an optional `date`, both
  compatible. The `_rpptx.pyi` diff against `py-rpptx-v0.11.0` removes nothing,
  and `rpptx` Python does not expose `replace_text`, so the notes-count change
  does not reach it.
- **CLI compatibility.** The `rdocx-cli` and `rpptx-cli` `main.rs` diffs add
  subcommands and flags and remove none. The guarded `replace` changes are
  disclosed in both Rust sections.
- **Family story maps.** For every story that touches a selected crate in each
  tag range, I checked backlog and AS_BUILT citations against the family
  inventory. Unmapped stories either cite only records already in the inventory
  (F-X094d), cite maintainer verification PR 82 (F-X094f, accepted in pass 3),
  or do not reach the family's output. F-X104 changes only
  `crates/oxml-drawing/src/fill.rs`, and no Word crate reads
  `alpha_modulation_fix`. F-X118 changes only `CT_TextParagraph::set_text` in
  `oxml-drawing`, and no Word crate calls it. F-X121 line-chart defaults and F-263
  and F-X101 are not reachable from the Python bindings they are left out of.
  F-X112 citing Issue 99 is stable-only and correct.
- **Release-note validity.** `release-notes --check` passes for `v0.14.0`,
  `rpptx-v0.12.0`, `py-rdocx-v0.14.0`, and `py-rpptx-v0.12.0`. The release
  contract test passes on `6dcb8bf0`.
- **Contributor credit.** Every linked record appears in its section's
  Contributors text. The handle sets match the inventories.
- **HLD and plan consistency.** `6dcb8bf0` touches no HLD or plan file. HLD 03,
  10, 12, 14, and 15 and the plan still agree on versions, tags,
  incubating-first order, and the through-F-X121 inventory.
- **Version carriers.** Manifests, READMEs, workflows, and
  `scripts/readme_doctests.py` carry 0.14.0 and 0.12.0. The only 0.11.0 pins
  left are the `sha1` and `sha2` registry dependencies at `Cargo.toml:101` and
  `Cargo.toml:102`.
- **Voice rules.** `prose_check` reports 0 violations for the tree and for the
  `6dcb8bf0` commit message.
