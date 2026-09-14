# Current Sprint, S72

**Milestone**: M23 From-scratch business documents.

**Goal**: expose every document section in stable order and provide one
relationship-safe content model for the body and every related story required
by from-scratch generation. The user-approved issue wave also restores
producer-package compatibility, searchable PDF text, comparison with drawings,
CLI automation, Python access, and a reviewed Python release path. Public
mutations must preserve schema order, unmodelled XML, story ownership, and
cross-part relationships while committing only complete validated candidates.

## Spec references

- `docs/hld/02-scope-and-non-goals.md`, for the DOCX-015 through DOCX-021
  capability boundaries and the precise partial surfaces S72 completes.
- `docs/hld/03-architecture.md`, for WordprocessingML grammar ownership, one
  facade-owned story model, source identity, and staged package mutation.
- `docs/hld/04-opc-and-packaging.md`, for relationship-scoped story parts,
  header and footer ownership, fragment dependency closure, and atomic commit.
- `docs/hld/05-drawingml-model.md`, for schema-ordered picture and DrawingML
  payload semantics that asset movement and fragment import must preserve.
- `docs/hld/08-rendering-spec.md`, for section geometry, story isolation,
  header and footer selection, and source-preserving layout behavior.
- `docs/hld/09-charts-spec.md`, for Word chart, editable-workbook, drawing, and
  relationship closure ownership that import must remap atomically.
- `docs/hld/10-bindings-spec.md`, for the native Rust facade boundary and the
  public section, story, relationship, and fragment-import surface.
- `docs/hld/12-testing-strategy.md`, for synthetic round-trip, differential,
  relationship-scope, fragment-remapping, and atomic-failure gates.
- `docs/hld/13-risks-and-open-questions.md`, for the rule that cross-part Word
  invariants are staged and validated as one operation.
- `docs/hld/14-development-backlog.md`, for the F-250 through F-256 acceptance
  contracts, the F-X090 through F-X094f issue contracts, dependencies, sizes,
  and S72 sequencing.
- `docs/hld/15-build-and-toolchain.md`, for Python wheel construction, trusted
  publication authority, artifact verification, and the separate release gate.

## The wave

| F-ID | Title | Size | Status | Owner |
|------|-------|------|--------|-------|
| F-253 | Container-neutral story editing | L | done | - |
| F-250 | Ordered mutable section facade | L | done | - |
| F-251 | Complete section and page geometry | L | done | - |
| F-254 | Generic insert, move, clone, and remove operations | L | done | - |
| F-255 | Part-scoped assets, links, and relationships | M | done | - |
| F-X090 | Accept part-local producer drawing identities | S | done | - |
| F-X091 | Serialize unused root default namespaces safely | M | done | - |
| F-252 | Rich per-section headers and footers | L | done | - |
| F-X093 | Preserve drawings through document comparison staging | M | done | - |
| F-256 | Transactional cross-document fragment import | L | done | - |
| F-X094a | Expose Word collaboration and redline commands in rdocx-cli | M | done | - |
| F-X094b | Structured CLI text and layout plus guarded replacement | L | done | - |
| F-X094c | Priority rdocx Python collaboration, comparison, layout, and TOC | L | done | - |
| F-X094d | rdocx Python sections, styles, rich stories, and hyperlinks | L | done | - |
| F-X094e | rpptx Python rendering, comments, and notes | L | done | - |
| F-X094f | Prepare the py-v0.13.1 release path | M | in-progress | codex |
| F-X092 | Preserve logical reading order in generated PDFs | L | done | - |

## Sequencing note

Rows are listed in dependency order, not F-ID order.

F-253 and F-250 establish the independent story and section ownership
foundations. F-251 may proceed once ordered section mutation exists, while
F-254 and F-255 build generic mutation and part-scoped relationships on the
container-neutral story model. F-X090 and F-X091 repair producer admission and
namespace serialization before F-252 expands related stories. F-252 converges
the section and story tracks so each header and footer variant can own rich
related content safely. F-X093 then protects drawings through comparison, and
F-256 remaps the complete package dependency graph. F-X094a through F-X094e
expose the reviewed native capabilities through the CLI and Python bindings.
F-X094f prepares the Python release contract without publishing. F-X092 lands
last because it exclusively owns the declared PDF baseline change.

## Definition of done for this sprint

- Ordered lookup, insertion, removal, and mutation preserve portrait,
  landscape, portrait section order and never orphan a related story.
- Authored size, orientation, margins, gutter, columns, page numbering, header
  and footer distance, title-page state, and break type match pinned Word page
  geometry and numbering while unsupported children remain visible.
- Default, first, and even headers and footers can be created, linked,
  inherited, replaced, and removed per section with rich content that survives
  save, reopen, replacement, and inheritance changes.
- One public location and traversal model visits supported body, cell, header,
  footer, note, comment, and text-box content with identical mutation errors
  and without introducing a second document tree.
- Arbitrary-position insert, move, clone, and remove operations preserve exact
  content order, references, and untouched raw XML, with invalid operations
  leaving the document unchanged.
- Images, hyperlinks, charts, and other related content resolve only through
  the relationships of their owning body or related story part.
- A dependency-rich cross-document fragment can be imported repeatedly with
  deterministic remapping and no collisions, while any unsupported dependency
  aborts without changing the destination.
- Producer drawing identifiers are validated per physical part, unused ambient
  default namespaces do not block safe saves, and comparison preserves valid
  drawings and namespace ownership.
- Generated Word and PowerPoint PDFs preserve complete logical text lines for
  pinned extraction while retaining deterministic visual geometry.
- CLI and Python users can reach the issue-defined collaboration, comparison,
  layout, text, story, hyperlink, rendering, comment, and note capabilities
  through exact typed or schema-versioned contracts.
- The `py-v0.13.1` release path validates twelve wheels and two source
  distributions, but creates no tag and performs no publication without a
  separate `/release` approval.
- Every operation preserves unmodelled XML and schema child order, publishes
  only a complete staged document and package candidate, remains deterministic,
  and leaves the hash harness unchanged unless a separately labelled and
  reviewed behavior change declares the expected delta.
