# F-X106c, Expose story mutation, hyperlinks, revisions, fields, and XML in Python

**Status**: completed
**Sprint**: S73
**Size**: L
**Depends on**: F-X103, F-X106b

## Problem

Python now reads sections, stories, links, comments, comparison, layout, and TOC
reports, but it cannot replace story text, set simple headers or footers, create
story links, inspect or resolve revisions, update field caches, or read one
story item's exact XML. These native capabilities are the remaining cohesive
read and mutation boundary from Issue 94. Contributor testing also found that
story replacement retains empty hyperlinks, cloned content duplicates comment
anchors, stale StoryItem snapshots can target different content, and text
lookup prefers an enclosing TOC control over the direct heading.

## Spec reference

- `docs/hld/03-architecture.md`, story ownership and staged mutation.
- `docs/hld/10-bindings-spec.md`, immutable snapshots and relationship-scoped mutations.
- `docs/hld/12-testing-strategy.md`, Python runtime, GIL, and typing gates.
- `docs/hld/14-development-backlog.md`, F-X106c.

## Approach

Add frozen revision snapshots and bind the existing native revision filters and
accept or reject methods. Add `set_story_text`, `set_header`, `set_footer`, and
`add_hyperlink_to_story` using frozen Story identifiers. Expose `update_fields`
with typed outcomes or counts from the native method and expose StoryItem `xml`
as immutable bytes. Sanitize cloned comment anchors, remove empty hyperlinks
created by story replacement, carry the document revision in every StoryItem,
and make content lookup prefer direct paragraph matches while exposing all
matches when ambiguity remains. Release the GIL for package-wide field or
revision work and bump binding revision exactly once after a successful
mutation.

## Rejected alternatives

- Return live native revision or XML references. They would outlive the
  document revision that owns them.
- Add mutable raw XML. Issue 94 requests a read-only escape hatch, and mutation
  would bypass package invariants.
- Reimplement story or field logic in Python. The native staged facade remains
  the single authority.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| binding | `python_story_revision_field_and_xml_operations_are_typed_and_atomic` | Story text, headers, footers, hyperlinks, revisions, fields, exact item XML, clean clone anchors, and direct-match lookup reopen correctly. |
| lifecycle | revision and GIL | Mutations bump once after success, failures bump none, and long native work permits Python thread progress. |
| regression | contributor Issue 94 cases | Empty links are pruned, stale StoryItems fail loudly, comment anchors do not duplicate, and a heading wins over enclosing TOC text. |
| typing | installed mypy and stubtest | Frozen revisions, Story inputs, field results, bytes, and filters match runtime. |

The **test gate** is the binding test named in the backlog.

## HLD impact

- `docs/hld/03-architecture.md`
- `docs/hld/10-bindings-spec.md`
- `docs/hld/12-testing-strategy.md`
- `docs/hld/14-development-backlog.md`

## Risk routing

- WASM or PyO3 bindings. Run both WASM checks, build the mixed package, and run
  pytest, strict mypy, stubtest, GIL checks, and clean abi3 installation.

## Hash harness

Expected to be unchanged because the sample generator does not use Python
story, revision, or field mutations.

## Implementation checklist

- [x] Add frozen revision and field-result snapshots where existing values are insufficient.
- [x] Bind story text, header, footer, and story hyperlink mutations.
- [x] Bind revision inspection and resolution filters.
- [x] Bind field updates and read-only StoryItem XML.
- [x] Sanitize cloned anchors, empty links, stale StoryItems, and ambiguous lookup.
- [x] Verify GIL release, revision bumps, runtime, typing, WASM, full verification, and microscope.

## Open questions

None.
