# F-X112, release, pass 5

**Reviewed**: `git diff aba94e0c..c9bf8ab8`, 64 files, 1682 insertions and 233
deletions. Excluding review files, 60 files, 978 insertions and 233 deletions.
Remediation commit `c9bf8ab8` changes only `CHANGELOG.md` (54 lines) and adds
the pass 4 review. Every concrete API claim in both rewritten Rust
Compatibility sections was checked against HEAD, `v0.13.1`, and
`rpptx-v0.11.0`. The Python stub was diffed against `py-rdocx-v0.13.2`.
**Verdict**: 1 defect, 0 smells, 2 nitpicks

## Pass 4 findings

### D1, fixed except for one false claim, reopened as D1 below

`CHANGELOG.md:240` now says the release contains source-incompatible Rust
changes and uses "including" language. Verified at HEAD against `v0.13.1`:

- `TocRebuildReport` swaps `diagnostic_count` for `diagnostics` and drops
  `Copy` at `crates/rdocx/src/field.rs:673` to `crates/rdocx/src/field.rs:677`.
  True.
- `CommentRangeStart` and `CommentRangeEnd` are struct variants, and both
  bookmark variants gained `has_child_content`, at
  `crates/rdocx/src/paragraph.rs:42` to `crates/rdocx/src/paragraph.rs:65`.
  True.
- `ListNumberFormat` drops `Copy`, keeps `PartialEq` and `Eq`, has no
  `#[non_exhaustive]`, and grows from 7 to 61 variants at
  `crates/rdocx/src/document.rs:21995`. True.
- `rdocx::Error::Story` at `crates/rdocx/src/error.rs:58`, on an enum without
  `#[non_exhaustive]`. True.
- `CT_Anchor::from_xml` and `CT_Inline::from_xml` take `NsReader` at
  `crates/rdocx-oxml/src/drawing.rs:506` and
  `crates/rdocx-oxml/src/drawing.rs:969`, in the public `drawing` module. True.
  The omitted `parse_alternate_content` parameter is covered by "including".
- `ListLevel` losing `PartialEq` is false. See D1.

### S1, fixed, with one listed item overstated

`CHANGELOG.md:78` to `CHANGELOG.md:83` now names `Blip` and
`OpcError::DuplicatePartName` under "including" language. `OpcError` gained the
variant at `crates/oxml-opc/src/error.rs:23` and has no `#[non_exhaustive]` at
either tag. `ChartData` fields and `Default` at `crates/oxml-chart/src/lib.rs:139`,
`LayoutLine.forced_break_after` at `crates/oxml-layout/src/line.rs:258`,
`field_source` on `TextSegment`, `GlyphRun`, and `MultilingualGlyphRun` at
`crates/oxml-layout/src/line.rs:176`, `crates/oxml-layout/src/output.rs:235`,
and `crates/oxml-layout/src/output.rs:263`, and `ResolvedImage.opacity` at
`crates/rpptx-layout/src/lib.rs:226` are all absent at `rpptx-v0.11.0` and
present at HEAD on structs without `#[non_exhaustive]`. The pass 4 premise for
`Blip` was wrong, which is recorded as a nitpick below.

### Pass 4 nitpicks

- Issue 93 now sits between 92 and 100 at `CHANGELOG.md:105`. Fixed.
- `CHANGELOG.md:398` to `CHANGELOG.md:402` scopes the sentence to document
  calls and names the constructor change separately. Fixed, with the residual
  wording noted below.

## Defects

### D1, `v0.14.0` Compatibility says `ListLevel` is no longer `PartialEq`, but it is, and omits the real `ListLevel` break
`CHANGELOG.md:248`

The bullet says "`ListLevel` is no longer `Copy`, `PartialEq`, or `Eq`." At
HEAD the derive at `crates/rdocx/src/document.rs:22354` drops `PartialEq`, but
F-247 adds a manual `impl PartialEq for ListLevel` at
`crates/rdocx/src/document.rs:22374`. Only `Copy` and `Eq` were removed. A
caller who compares `ListLevel` values with `==` is told that code breaks when
it still compiles, and may rewrite working code.

The concrete break the bullet misses is on construction. At `v0.13.1`,
`ListLevel` had only the public fields `format` and `start`, so
`ListLevel { format, start }` compiled. At HEAD twelve private fields follow
them at `crates/rdocx/src/document.rs:22360` to
`crates/rdocx/src/document.rs:22371`, so that struct literal and an exhaustive
destructure without `..` no longer compile. The caller action is
`ListLevel::new(format)` with the `start` builder, which exists at both tags.
`docs/hld/10-bindings-spec.md:1260` to `docs/hld/10-bindings-spec.md:1263`
already records that extending `ListLevel` breaks full struct literals. Say
that `ListLevel` is no longer `Copy` or `Eq`, and that it must be built with
`ListLevel::new` and its builders rather than a struct literal.

This is a defect rather than a nitpick because it is a false statement about a
facade type in the section callers read to plan an upgrade, not an omission
covered by "including".

## Smells

None.

## Nitpicks

- `CHANGELOG.md:80` to `CHANGELOG.md:83`: `Blip` has private fields at both
  `rpptx-v0.11.0` and HEAD (`crates/oxml-drawing/src/fill.rs:614` to
  `crates/oxml-drawing/src/fill.rs:615`), so no external caller can build it
  with a full struct literal or destructure it without `..`. The new public
  field is not a source break. Listing it overstates impact but misleads no
  caller into a wrong change of consequence.
- `CHANGELOG.md:400`: "The one signature change is the `TocRebuildReport`
  constructor" is literally false. `add_comment` widened `range` and gained an
  optional `date`, and `reply_to` gained an optional `date`, in
  `crates/rdocx-py/python/rdocx/_rdocx.pyi:466` and
  `crates/rdocx-py/python/rdocx/_rdocx.pyi:475`. Both are compatible. "The one
  incompatible signature change" would be exact.

## Not found

- **Other Rust compatibility claims.** Every other named item in both Rust
  sections is true at HEAD and absent or different at the previous tag. A
  comparison of every `pub fn` signature under `crates/rdocx/src` between
  `v0.13.1` and HEAD, excluding test modules, finds none removed or changed, so
  no headline facade method break is unlisted.
- **Release-note validity.** `release-notes --check` reports ok for `v0.14.0`,
  `rpptx-v0.12.0`, `py-rdocx-v0.14.0`, and `py-rpptx-v0.12.0` at HEAD.
  `s73_release_contract_requires_four_version_aligned_families` passes.
- **Release-note truth against AS_BUILT and backlog.** `c9bf8ab8` changes no
  Added or Fixed bullet and no family inventory. The story maps confirmed in
  pass 4 are unchanged.
- **Family scope.** The 15-package and seven-package selected sets in both
  Compatibility sections are unchanged and match the manifests.
- **Contributor credit.** Issue ordering is now ascending in the rpptx list.
  Handle sets and linked records are unchanged from pass 4.
- **Test gate strength.** No test changed in `c9bf8ab8`. The inventory gate
  accepted in pass 4 still applies.
- **HLD and plan consistency.** `c9bf8ab8` touches no HLD or plan file. HLD 03,
  10, 12, 14, and 15 and the plan still agree on versions, tags, publication
  order, and inventory. HLD 10 agrees with the notes on the numbering breaks
  apart from D1.
- **Version carriers.** Unchanged since pass 4 and still consistent.
- **Voice rules.** `prose_check` reports 0 violations for the tree, and the
  `c9bf8ab8` commit message contains no em-dash or prose semicolon.
