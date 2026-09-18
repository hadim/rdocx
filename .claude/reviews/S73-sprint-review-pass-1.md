# S73 sprint review, pass 1

**Reviewed**: `sprint/s73` against `029065ca3546559f982900260878eab4ac73f1a4`, 176 files, 27,309 changed lines, crates: oxml-chart, oxml-drawing, oxml-layout, oxml-pdf, rdocx, rdocx-cli, rdocx-layout, rdocx-oxml, rdocx-py, rpptx, rpptx-cli, rpptx-layout, rpptx-py, rpptx-render
**Verdict**: 0 blocking, 1 should-fix, 0 nice-to-have

## Blocking

None.

## Should-fix

### S1, completed rows retain active owners
`docs/sprints/CURRENT_SPRINT.md:47`

Twenty-five completed S73 rows still name `codex` in the `Owner` column, while
the five stories finalised at this checkpoint correctly clear the column. The
run-sprint finalisation contract requires a completed story to clear its owner.
Leaving historical owners mixed with active ownership makes the shared delivery
record ambiguous and disagrees with the completed-state convention already used
by F-X115 through F-X121. Clear the owner cell for every completed row and clear
the corresponding completed feature owners in the resumable run state.

## Nice-to-have

None.

## Milestone gate

The M23 gate requires the private-corpus mode to generate all five references
from public `Document::new()` programs without LibreOffice post-processing, raw
XML, or source-template access. That final gate does not hold yet because F-263
is still in progress at this scheduled dependency-prefix checkpoint. The
completed prefix is supported by the passing public authoring conformance,
layout, corpus, binding, package, deterministic viewer, and full workspace
tests recorded at `b5b32cafc280070c8282974beff5293d7511f3ce`. F-263 must supply
the five-document private acceptance evidence before final sprint closure.

## Not found

- Interaction: no jointly incorrect behavior was found across the completed
  table, run, story, drawing, comment, notes, image, and chart changes.
- Duplication: no competing sprint-local implementation of a shared operation
  was found.
- Layering: the full dependency-direction tests passed, and no `oxml-*` crate
  gained an `rdocx-*` or `rpptx-*` dependency.
- Harness: F-X102 declares and records the three changed feature-showcase PDF
  fingerprints, while every later checkpoint matches the resulting 49-entry
  baseline.
- Gate: no false claim that the unfinished five-document M23 gate already
  passed was found.
- Dependencies: no third-party dependency was added. The two CLI manifest
  changes expose named system-font feature routing and cargo-binstall metadata.
- Surface: every added public type and operation maps to an approved S73 story.
- Docs beyond S1: the HLD sections named by completed plans describe current
  behavior and the completed AS_BUILT records cite their exact gates.
