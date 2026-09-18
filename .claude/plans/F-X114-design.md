# F-X114, Rebuild TOC entries with document styles and geometry

**Status**: completed
**Sprint**: S73
**Size**: M
**Depends on**: F-X103, F-263

## Problem

`rebuild_toc()` hard-codes `TOC1` through `TOC9`, writes a 9350 twip right tab
for every page, and places a numbered heading's tab suffix inside `w:t`. This
loses localized TOC styles, exceeds narrower section text widths, and can render
the literal tab as a missing glyph.

## Spec reference

- `docs/hld/04-opc-and-packaging.md`, style ownership and schema order.
- `docs/hld/08-rendering-spec.md`, section geometry, tabs, and field materialization.
- `docs/hld/12-testing-strategy.md`, field and Word comparison gates.
- `docs/hld/14-development-backlog.md`, F-X114.

## Approach

Resolve each built-in TOC level by its case-normalized `w:name` value `toc N`
and retain the producer's localized style id. Create the canonical style only
when no matching style exists. Prefer a right tab already supplied by the
effective TOC style. Otherwise derive the stop from the effective section text
width at the field location. Emit a numbering suffix tab as ordered run content
and serialize it as `w:tab`, never as a literal control character in `w:t`.

## Rejected alternatives

- Translate a fixed list of localized IDs. Style names are the stable package
  signal and producer IDs are not bounded to known languages.
- Keep 9350 twips as a compatibility default. It is outside ordinary A4 text
  width and ignores authored margins.
- Replace every tab suffix with a space. That changes numbering layout and
  discards an explicit numbering property.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| regression | `rebuilt_toc_uses_localized_styles_section_tabs_and_structural_suffixes` | Localized ids, A4 geometry, style tabs, and numbered heading suffixes reopen with no undefined style or literal tab. |
| differential | Word TOC entries | Pinned Word output agrees on style selection, tab placement, and visible numbered entry text. |
| preservation | unrelated styles | Existing TOC styles and unmodelled style children remain byte-preserved. |

The **test gate** is the regression test named in the backlog.

## HLD impact

- `docs/hld/04-opc-and-packaging.md`
- `docs/hld/08-rendering-spec.md`
- `docs/hld/12-testing-strategy.md`
- `docs/hld/14-development-backlog.md`

## Risk routing

- Layout and pagination. Use deterministic fonts and section-local geometry
  for every rendered assertion.
- Any parser or serializer. Verify style and paragraph child order plus exact
  preservation of unrelated producer XML.
- External oracle comparison. Pin Word and record structural and rendered
  tolerances.

## Hash harness

Expected to be unchanged because samples do not rebuild a TOC.

## Implementation checklist

- [x] Add localized-style, A4, style-tab, and numbered-heading failures.
- [x] Resolve TOC styles by built-in name without replacing producer ids.
- [x] Derive fallback tabs from effective section geometry.
- [x] Serialize numbering suffix tabs as ordered run content.
- [x] Run field, style, layout, oracle, hash harness, full verification, and microscope gates.

## Open questions

None.
