# F-251, Complete section and page geometry

**Status**: approved
**Sprint**: S72
**Size**: L
**Depends on**: F-250

## Problem

`CT_SectPr` models most ordinary page geometry, but the facade setters at
`crates/rdocx/src/document.rs:9988` address only the final section and omit
page-number start and format. The parser stores unsupported section children in
one side vector at `crates/rdocx-oxml/src/document.rs:100`, so a modeled edit
must keep those children visible and in valid schema order.

## Spec reference

- `docs/hld/02-scope-and-non-goals.md`, "Modern DOCX capability matrix",
  DOCX-016.
- `docs/hld/03-architecture.md`, "Facade conventions".
- `docs/hld/08-rendering-spec.md`, Word section geometry and page numbering.
- `docs/hld/14-development-backlog.md`, "F-251, Complete section and page geometry".

## Approach

Extend the typed section model with the M23 page-number properties needed for
start and supported number format while preserving unknown attributes and
children. Expose complete getters and setters on the F-250 section handles for
page size, orientation, margins, gutter, columns, page numbering, header and
footer distance, title-page state, and break type. Serialize modeled children
in `CT_SectPr` schema order and replay unsupported children at stable positions.
Keep the existing final-section document conveniences as delegating entry
points into the same concrete implementation.

## Rejected alternatives

- A facade-only geometry snapshot would lose unknown producer state on write.
- Replacing `CT_SectPr` wholesale would erase unsupported M24 properties.
- Floating-point storage would weaken the exact integer-twip contract.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| differential | `mixed_orientation_sections_match_word_geometry_and_page_numbers` | Source-built mixed sections match the pinned Word page geometry and page-number sequence. |
| round-trip | `section_geometry_round_trips_with_unsupported_children_in_order` | Every authored M23 property reopens and untouched M24 children retain bytes and position. |
| regression | `rejected_section_geometry_is_atomic` | Invalid columns, dimensions, and numbering values leave bytes unchanged. |

The story test gate is differential: mixed-orientation source-built sections
match the pinned Word page geometry and page-number sequence.

## HLD impact

- `docs/hld/02-scope-and-non-goals.md`
- `docs/hld/01-glossary.md`
- `docs/hld/03-architecture.md`
- `docs/hld/08-rendering-spec.md`
- `docs/hld/10-bindings-spec.md`
- `docs/hld/12-testing-strategy.md`
- `docs/hld/14-development-backlog.md`

## Risk routing

- Unit conversion. Read the units section of `docs/hld/01-glossary.md` and the
  deliberately wrong note in `CLAUDE.md`. Keep `as i64` truncation and declare
  any harness delta.
- Any parser or serialiser. Read `docs/hld/04-opc-and-packaging.md` and
  `docs/hld/06-presentationml-model.md`. Prove fixed write prefixes, schema
  order, prefix-tolerant reads, and byte-exact retained subtrees.
- Layout and pagination. Read `docs/hld/08-rendering-spec.md`. Use deterministic
  fonts for every baseline and differential render.
- Public API of a published crate. Read `docs/hld/10-bindings-spec.md` and the
  structural rules in `CLAUDE.md`. State the additive pre-1.0 semver impact,
  run `cargo publish --dry-run -p rdocx`, and assert the packaged crate size.
- External oracle comparison. Read `.claude/skills/differential-testing.md`.
  Pin and record the exact Word oracle version.

## Hash harness

Expected to be unchanged for existing fixtures. Newly authored section
geometry has no baseline entry unless separately reviewed.

## Implementation checklist

- [ ] Model page-number start and supported number format with raw retention.
- [ ] Preserve exact `CT_SectPr` child order during parse and serialization.
- [ ] Add complete geometry accessors to ordered section handles.
- [ ] Validate ranges before staged publication.
- [ ] Add round-trip, differential, and atomic rejection tests.

## Open questions

None. M23 authors `w:pgNumType/@w:start` and equal-width count and spacing.
Format, chapter style, separator, variable widths, and separators remain
preserved for M24. Zero columns and nonpositive page dimensions fail without
changing the document.
