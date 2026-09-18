# F-X120, Accept fractional DOCX line spacing values

**Status**: completed
**Sprint**: S73
**Size**: S
**Depends on**: none

## Problem

`CT_PPr::from_xml` requires `w:spacing/@w:line` to parse directly as `i32`.
Some real DOCX producers write decimal values such as `257.1432`, so the
public document open path rejects otherwise valid packages before the facade
can expose or render their paragraphs. Contributor PR 122 supplies the
observed producer cases and an initial bounded parser.

## Spec reference

- `docs/hld/01-glossary.md`, Word integer twips and unit conversion rules.
- `docs/hld/04-opc-and-packaging.md`, Word paragraph property parsing and canonical serialization.
- `docs/hld/08-rendering-spec.md`, effective Word line spacing and deterministic pagination.
- `docs/hld/12-testing-strategy.md`, parser, round-trip, and output-stability gates.
- `docs/hld/14-development-backlog.md`, F-X120.

## Approach

Integrate PR 122 and harden its decimal normalization so it does not depend on
binary floating-point approximation. Keep the existing integer parse as the
first path. For a signed plain decimal, validate the complete lexical form,
derive the nearest integer twip with exact decimal arithmetic, round an exact
half away from zero, and reject values whose numeric value lies outside the
signed 32-bit range. Serialize the modeled property through the existing
integer writer. Do not broaden decimal acceptance to any other OOXML measure.

## Rejected alternatives

- Parse every twip measurement as a decimal. The reported producer deviation
  is specific to paragraph line spacing, and widening every parser would hide
  unrelated malformed input.
- Retain `f64` rounding from the initial patch. Long decimal fractions near a
  half can cross the rounding boundary when represented in binary.
- Preserve the decimal spelling on save. Once modeled, the property has one
  signed integer twip value and the existing writer provides the canonical
  form.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| regression | `word_fractional_line_spacing_opens_with_nearest_twip_values` | Observed producer decimals open through the public facade, normalize to the expected twips, save canonically, and reopen identically. |
| unit | exact decimal boundaries | Positive and negative halves, long fractions around a half, signs, zeros, signed bounds, malformed values, exponent notation, and overflow have exact outcomes. |
| layout | normalized paragraph spacing | Equivalent decimal and integer inputs produce identical deterministic layout and raster bytes. |
| round-trip | namespace and sibling preservation | Aliased Word prefixes and unrelated spacing attributes survive normalization without changing parser scope or schema order. |

The **test gate** is the regression test named in the backlog.

## HLD impact

- `docs/hld/04-opc-and-packaging.md`
- `docs/hld/08-rendering-spec.md`
- `docs/hld/12-testing-strategy.md`
- `docs/hld/14-development-backlog.md`

## Risk routing

- Unit conversion. Keep constructor truncation unchanged, test exact positive
  and negative decimal rounding separately, and require an explained hash
  result.
- Any parser or serializer. Verify namespace aliases, canonical integer write,
  sibling attribute preservation, and save-reopen behavior.
- Layout and pagination. Compare decimal and normalized integer documents in
  deterministic font mode and require byte-identical rendered pages.

## Hash harness

Expected to be unchanged because checked-in samples do not contain fractional
paragraph line spacing. Any delta blocks completion until separately explained.

## Implementation checklist

- [x] Add the contributor's observed public-open regression as a failing gate.
- [x] Normalize signed plain decimals with exact nearest-twip arithmetic.
- [x] Cover half boundaries, long fractions, overflow, aliases, and canonical reopen.
- [x] Prove normalized integer and fractional inputs render identically.
- [x] Run parser, round-trip, layout, hash harness, full verification, and microscope gates.

## Open questions

None. The user asked to include every new relevant pull request in S73 and
approved the sprint work.
