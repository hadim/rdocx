# F-266, International and vertical typography

**Status**: completed
**Sprint**: S74
**Size**: L
**Depends on**: F-264, F-265

## Problem

The draft design round found that DOCX-033 in
`docs/hld/02-scope-and-non-goals.md:240` covers nine distinct behaviours across
six independent work groups, three crates and the bundled font inventory. That
is too much for one F-ID. `.claude/WORKFLOW.md`, "Sub-IDs when a story splits",
gives the mechanism, and the S74 consolidated design round approved the split.

This file is now the parent record. It carries no implementation checklist of
its own. Each child carries a complete contract.

## Spec reference

- `docs/hld/02-scope-and-non-goals.md`, "Modern DOCX capability matrix", the
  DOCX-033 row and the `Y`, `P`, `N`, `PV`, `NA`, `B` legend preceding it.
- `docs/hld/14-development-backlog.md`, "F-266, International and vertical
  typography (L)", whose acceptance text the three children divide.
- `.claude/WORKFLOW.md`, "Sub-IDs when a story splits".

## Approach

The split follows the work-group boundaries the draft identified, cut where the
dependencies are real rather than where the line count is even.

| Child | Title | Work groups | Depends on |
|---|---|---|---|
| F-266a | Script identity and font slot resolution | A, B, plus the bundled subset faces | F-264, F-265 |
| F-266b | Ruby and emphasis marks | C, D | F-266a |
| F-266c | Character grid and vertical text | E, F | F-266a, F-269 |

F-266a comes first because both siblings need Hangul and Kana to have a script
identity and both need a font that can draw them. F-266b and F-266c are
independent of each other and may run in parallel once F-266a lands.

Each child owns one named golden test in a shared deterministic fixture module.
F-266a records its digest, and neither sibling moves it. Each sibling adds its
own page and its own digest, so a moved baseline is always attributable to one
child.

## Rejected alternatives

- **Leave F-266 whole.** Rejected. Six work groups spanning `oxml-layout`,
  `rdocx-oxml`, `rdocx-layout`, `rdocx` and the published font inventory is a
  diff no single `/microscope` pass can review honestly.
- **Split by crate rather than by behaviour.** Rejected. Every work group
  crosses at least two crates, so a crate-shaped cut would leave each child
  unable to prove anything end to end.
- **Split ruby out alone and keep the other five together.** Rejected. It
  leaves the second child as large as the original problem.
- **Four children, separating the bundled fonts from script identity.**
  Rejected. The font faces exist to make the script identity provable, and a
  child that adds fonts nothing exercises has no test gate of its own.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| golden | `mixed_script_page_matches_the_pinned_geometry_and_reading_order` | Owned by F-266a |
| golden | `ruby_and_emphasis_page_matches_the_pinned_geometry_and_reading_order` | Owned by F-266b |
| golden | `grid_and_vertical_page_matches_the_pinned_geometry_and_reading_order` | Owned by F-266c |

The **test gate** for the parent is the union of the three, and the parent
closes only when all three pass. No test is stubbed against this file. Each
child's plan is what `/start-feature` reads.

## HLD impact

None from this file. Each child records its own, and
`docs/hld/02-scope-and-non-goals.md` DOCX-033 reaches its final classification
in F-266c, the last child to close.

## Risk routing

None from this file. It describes no diff. Each child routes its own risk, and
`/run-sprint` takes the union across the three when it builds the consolidated
gate.

## Hash harness

Unchanged from this file, which contains no code change. Each child carries its
own prediction. The bundled-font addition, which is the only change in the
family with a credible path to moving a sample, lands in F-266a under an
explicit containment rule.

## Implementation checklist

- [x] F-266a complete.
- [x] F-266b complete.
- [x] F-266c complete.
- [x] DOCX-033 in `docs/hld/02-scope-and-non-goals.md` reaches its earned
      classification, which F-266c records.
- [x] `docs/hld/14-development-backlog.md` and `docs/sprints/BACKLOG.md` both
      carry the three children, per `.claude/WORKFLOW.md`.

**The parent closes only when every child closes.** This file is never started
directly.

## Open questions

None. Resolved in the S74 consolidated design round.
