# F-X123, Accept producer TOC style variants

**Status**: completed
**Sprint**: S74
**Size**: S
**Depends on**: F-X114

## Problem

Issues 124 and 125 show two producer variants that prevent TOC rebuilding. A
custom-style list with one trailing comma is rejected as an odd pair list, and
duplicate style identifiers make TOC source discovery invoke the strict style
mutation validator even though the rest of the read surface accepts the file.

## Spec reference

- `docs/hld/04-opc-and-packaging.md`, lossless Word package parsing.
- `docs/hld/12-testing-strategy.md`, dynamic TOC differential and regression gates.
- `docs/hld/14-development-backlog.md`, "F-X123, Accept producer TOC style variants".

## Approach

Drop exactly one final empty custom-style component before validating pairs.
Reject interior empties and all other malformed lists as before. Build TOC
style lookup from the first definition of each identifier, add an ordered
diagnostic for later duplicates, and leave the strict style-graph validator
unchanged for public style mutations.

## Rejected alternatives

- Weaken `validate_style_graph`. Mutations still require an unambiguous graph.
- Remove duplicate style XML. Rebuild must not rewrite producer definitions.
- Ignore every empty component. That would accept malformed interior pairs.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| regression | `toc_rebuild_accepts_trailing_style_separator_and_duplicate_style_ids` | Both producer variants rebuild with stable entries and diagnostics. |
| round-trip | duplicate style package save and reopen | Every source style remains present and ordered. |
| regression | malformed custom-style lists | Interior empties and missing levels retain stored display with diagnostics. |

The **test gate** is the named regression.

## HLD impact

- `docs/hld/04-opc-and-packaging.md`
- `docs/hld/12-testing-strategy.md`
- `docs/hld/14-development-backlog.md`

## Risk routing

- Parser and serializer behavior. Prove prefix tolerance, exact malformed
  rejection, retained duplicate XML, and unchanged strict mutation validation.

## Hash harness

Expected unchanged. The checked samples do not contain either producer variant.

## Implementation checklist

- [x] Add source-built regressions for both reports and malformed controls.
- [x] Accept one trailing empty custom-style component.
- [x] Use deterministic first-definition TOC style lookup with diagnostics.
- [x] Run focused field, style, round-trip, and hash verification gates.

## Open questions

None. The reporter accepts deterministic first or last lookup. First definition
matches normal source-order lookup and avoids rewriting the package.
