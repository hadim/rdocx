# F-251, Complete section and page geometry

**Status**: completed
**Sprint**: S72
**Size**: L
**Depends on**: F-250

## Problem

`CT_SectPr` models most ordinary page geometry, but the legacy facade setters
address only the final section and omit a page-number restart. The parser also
stored unsupported section children without their schema positions, so a
modeled edit could move those children or hide them behind typed content.

## Spec reference

- `docs/hld/02-scope-and-non-goals.md`, "Modern DOCX capability matrix",
  DOCX-016.
- `docs/hld/03-architecture.md`, "Facade conventions".
- `docs/hld/08-rendering-spec.md`, Word section geometry and page numbering.
- `docs/hld/14-development-backlog.md`, "F-251, Complete section and page geometry".

## Approach

Extend the typed section model with M23 authoring for
`w:pgNumType/@w:start`. Preserve `fmt`, `chapStyle`, `chapSep`, and every other
unsupported page-number attribute or child for M24. Expose complete getters and
checked setters on the F-250 section handles for page size, orientation,
margins, gutter, equal-width columns, page-number start, header and footer
distance, title-page state, and break type. Serialize modeled children in
`CT_SectPr` schema order and replay unsupported children at stable schema or
repeated-reference boundaries. Distinguishable references follow their value,
while indistinguishable equal duplicates resolve deterministically by source
ordinal. Keep the existing final-section document conveniences explicitly
unchecked and infallible for source compatibility.

## Rejected alternatives

- A facade-only geometry snapshot would lose unknown producer state on write.
- Replacing `CT_SectPr` wholesale would erase unsupported M24 properties.
- Floating-point storage would weaken the exact integer-twip contract.
- Hidden object identities would break the published `Vec<HdrFtrRef>` mutation
  surface without making equal public values distinguishable to callers.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| differential | `mixed_orientation_sections_match_word_geometry_and_page_numbers` | Source-built mixed sections match the pinned Word page geometry and page-number sequence. |
| round-trip | `section_geometry_round_trips_with_unsupported_children_in_order` | Every authored M23 property reopens and untouched M24 children retain bytes and position. |
| regression | `rejected_section_geometry_is_atomic` | Invalid columns, dimensions, and numbering values leave bytes unchanged. |

The story test gate is differential: mixed-orientation source-built sections
match Microsoft Word 16.112.3 build 16.112.26083020 for exact physical page
geometry and displayed page-number sequence.

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
  structural rules in `CLAUDE.md`. The section accessors and setters, typed
  page-number state, and displayed page number are additive pre-1.0 Rust API.
  The public reference fields remain `Vec<HdrFtrRef>`. Run the workspace publish
  dry run with reviewed local patches and assert every packaged crate size.
- External oracle comparison. Read `.claude/skills/differential-testing.md`.
  Pin and record the exact Word oracle version.

## Hash harness

Expected to be unchanged for existing fixtures. Newly authored section
geometry has no baseline entry unless separately reviewed.

## Implementation checklist

- [x] Model page-number start while retaining unsupported M24 state.
- [x] Preserve exact `CT_SectPr` child order during parse and serialization.
- [x] Add complete geometry accessors to ordered section handles.
- [x] Validate ranges before staged publication.
- [x] Add round-trip, differential, and atomic rejection tests.

## Open questions

None. M23 authors `w:pgNumType/@w:start` and equal-width count and spacing.
Number format, chapter style, chapter separator, variable widths, and column
separators remain preserved for M24. Zero columns and nonpositive page
dimensions fail without changing the document.
