# Current Sprint, S95.3

**Milestone**: M24, Modern DOCX authoring completeness.

**Goal**: Author the content control lifecycle before typed values and custom XML
bindings. F-285 creates, inspects, edits, moves, clones and removes controls.
F-286 and F-287 then add typed controls and custom XML stores. The three
stories total 12 to 15 developer-days.

## Spec references

- `docs/hld/14-development-backlog.md`, F-285, F-286 and F-287 define the creation, typed-control and custom-XML scope, their dependencies and the round-trip or differential gates. M24's complete stability boundary remains F-310 in S99.
- `docs/hld/02-scope-and-non-goals.md`, DOCX-052, DOCX-053 and DOCX-054 name the form rows these stories close.
- `docs/hld/03-architecture.md`, content-control order, shell identity, data binding and fragment ownership define the structures public creation must reuse.
- `docs/hld/04-opc-and-packaging.md`, the document-owned custom XML graph, item properties and exact retention of unrelated store bytes govern F-287 publication.
- `docs/hld/10-bindings-spec.md`, the present content-control read and replacement surfaces define what authoring may expose and what stays internal.
- `docs/hld/12-testing-strategy.md`, round-trip and differential controls for content controls, bound custom XML and byte-identical unknown stores define the integrated observations.

## The wave

| F-ID | Title | Size | Status | Owner |
|------|-------|------|--------|-------|
| F-285 | Content control creation and lifecycle | L | pending | - |
| F-286 | Rich, repeating, and typed content controls | L | pending | - |
| F-287 | Custom XML stores and data binding authoring | L | pending | - |

## Sequencing note

Rows are listed in dependency order, not F-ID order.

F-285 comes first. F-286 and F-287 both depend on F-285 and do not depend on
each other. Complete and integrate F-285 through its scoped dependency-prefix
checkpoint before either of them starts. Their shared source and resource
leases may still require serial implementation after that prefix. F-X249's
bounded existing-control fill repair does not replace these authoring
contracts.

## Definition of done for this sprint

- Integrate F-285 through its scoped dependency-prefix checkpoint before starting F-286 or F-287.
- F-285 public creation and mutation of block, row, cell, paragraph and inline controls survives round-trip with no raw-XML fallback.
- F-286 typed controls match the pinned Word structure under replacement and repetition.
- F-287 bound controls resolve, update and reopen against the correct store, and unknown store content stays byte-identical.
- One integrated verification and a clean sprint review. Do not start S96. M24's complete stability gate remains F-310 in S99.
- Issues 264 and 281 stay open. No release tag is prepared by opening this sprint.
