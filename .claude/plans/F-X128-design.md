# F-X128, Preserve Word paragraph and revision identities

**Status**: completed
**Sprint**: S74
**Size**: M
**Depends on**: F-X115

## Problem

Issue 130 shows that a no-op save drops root attributes from modeled
paragraphs, runs, and section properties. This includes `w14:paraId`,
`w14:textId`, and the `w:rsid*` revision-session family.

## Spec reference

- `docs/hld/04-opc-and-packaging.md`, preserve unmodelled producer XML.
- `docs/hld/12-testing-strategy.md`, round-trip and deterministic serialization gates.
- `docs/hld/14-development-backlog.md`, "F-X128, Preserve Word paragraph and revision identities".

## Approach

Add ordered root-attribute retention to `CT_P`, `CT_R`, and `CT_SectPr` using
expanded-name-aware parsing. Preserve every attribute not already owned by a
typed field, including namespace aliases and foreign attributes. Writers emit
the retained attributes on the modeled root before children. Existing authored
paragraph identity injection overrides only the same expanded `paraId` name.

## Rejected alternatives

- Model only `paraId`. The same parser currently loses all producer root attributes.
- Generate missing identities. This report is lossless round-trip, not authoring policy.
- Preserve the whole element raw. Public mutations require the typed children.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| round-trip | `paragraph_run_and_section_identity_attributes_survive_noop_save` | Exact identity and rsid values survive save and reopen. |
| regression | alias and foreign attribute matrix | Expanded-name ownership is prefix tolerant and foreign values remain. |
| integration | edited identified paragraph | Typed edits retain unrelated identities and deterministic bytes. |

The **test gate** is the named round-trip test.

## HLD impact

- `docs/hld/04-opc-and-packaging.md`
- `docs/hld/12-testing-strategy.md`
- `docs/hld/14-development-backlog.md`

## Risk routing

- Parser and serializer behavior. Prove namespace aliases, duplicate expanded
  attribute rejection, deterministic order, child schema order, and raw retention.

## Hash harness

Expected unchanged. Existing generated samples do not carry producer identities.

## Implementation checklist

- [x] Add paragraph, run, and section root-attribute round trips.
- [x] Retain expanded-name-safe ordered root attributes.
- [x] Define authored `paraId` precedence without duplicates.
- [x] Run the impacted oxml, facade, round-trip, clippy, and hash gates.

## Open questions

None. Generation of missing identities is deliberately outside this correction.
