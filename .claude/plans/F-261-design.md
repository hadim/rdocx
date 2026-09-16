# F-261, Rich HTML fragments in arbitrary containers

**Status**: completed
**Sprint**: S73
**Size**: L
**Depends on**: F-253 through F-260

## Problem

The native HTML importer creates a new document from HTML, but callers cannot
insert a fragment into an existing body, cell, header, or footer location. The
M23 generators require bounded rich fragments, including lists, tables,
images, links, and CSS, without raw XML or hidden network access.

## Spec reference

- `docs/hld/03-architecture.md`, inbound HTML ownership and staged document mutation.
- `docs/hld/04-opc-and-packaging.md`, HTML image resources and relationship ownership.
- `docs/hld/10-bindings-spec.md`, native-only HTML import boundary.
- `docs/hld/12-testing-strategy.md`, HTML diagnostics and M23 conformance.
- `docs/hld/14-development-backlog.md`, F-261.

## Approach

Add a native fragment insertion operation on `Document` that accepts a checked
`ContentLocation`, HTML text, and explicit resolver-provided image resources.
Reuse the existing HTML5 repair, CSS projection, list, table, image, link, and
diagnostic code to produce a concrete fragment. Reconcile numbering, media,
relationships, and identifiers into the selected story, then insert through
the existing staged package boundary. Return an owned result containing the
inserted direct range and ordered `HtmlDiagnostic` values for every dropped or
approximated construct. Keep external fetching absent.

## Rejected alternatives

- Import a temporary DOCX and use cross-document fragment transfer. That adds
  a package conversion and hides source HTML diagnostics.
- Accept a callback trait for resources. Only an explicit resource collection
  is needed today, so a trait has no second implementer.
- Add Python, WASM, or CLI entry points. The current product contract keeps
  inbound Word HTML native Rust only.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| differential | `rich_html_fragments_match_word_in_every_supported_container` | The supported subset produces equivalent Word content and rendering in body, cell, header, and footer locations. |
| regression | fragment package reconciliation | Lists, tables, data-URI and explicit images, links, numbering, and identifiers remain owner-correct after reopen. |
| unit | ordered fragment diagnostics | Every unsupported or dropped construct yields one stable location-aware diagnostic in source order. |
| failure | fragment insertion is atomic | Invalid location, resource, limit, or package graph leaves the document unchanged. |

The **test gate** is the differential test named in the backlog.

## HLD impact

- `docs/hld/02-scope-and-non-goals.md`
- `docs/hld/03-architecture.md`
- `docs/hld/04-opc-and-packaging.md`
- `docs/hld/10-bindings-spec.md`
- `docs/hld/12-testing-strategy.md`
- `docs/hld/14-development-backlog.md`

## Risk routing

- Any parser or serialiser. Verify generated Word child order and preserve
  unrelated raw content in the destination container.
- Public API of a published crate. State the additive native-only result and
  operation, run rustdoc and API inspection, and run package dry-runs and size
  checks.
- External oracle comparison. Pin Word, LibreOffice, and Poppler and record the
  supported-subset comparison and tolerance.

## Hash harness

Expected to be unchanged because existing samples do not insert HTML fragments
into an existing document. Any new sample output is reviewed as new coverage,
not a replacement baseline.

## Implementation checklist

- [x] Add failing body, cell, header, and footer fragment differentials.
- [x] Reuse the existing HTML parser and projection without a second model.
- [x] Reconcile container-local numbering, links, media, and identifiers atomically.
- [x] Return the inserted range and exact ordered diagnostics.
- [x] Run parser, package, public API, oracle, hash harness, full verification, and microscope gates.

## Open questions

None.
