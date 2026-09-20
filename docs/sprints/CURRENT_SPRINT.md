# Current Sprint, S74

**Milestone**: M24 Modern DOCX authoring completeness.

**Goal**: expand the corpus-proven Word primitives into complete public
paragraph, run, table, section, and document-setting authoring. The sprint
closes the first seven capability gaps in the M24 matrix with modeled
round-trip ownership, deterministic layout and rendering, and pinned Word
differential evidence where the contract requires it. It then presents the
completed Rust and Python package depth with reproducible footprint and speed
evidence.

## Spec references

- `docs/hld/02-scope-and-non-goals.md`, for the DOCX-030 and DOCX-032 through
  DOCX-037 capability rows and the complete, partial, and unsupported
  classifications this sprint must close.
- `docs/hld/03-architecture.md`, for WordprocessingML grammar ownership,
  effective paragraph and run properties, facade mutation, and atomic package
  publication.
- `docs/hld/04-opc-and-packaging.md`, for namespace-aware modeled parts,
  schema-ordered property serialization, section ownership, and lossless
  settings and web-settings package updates.
- `docs/hld/08-rendering-spec.md`, for bidirectional and complex-script text,
  table-style resolution, advanced table geometry, and section-aware
  pagination.
- `docs/hld/12-testing-strategy.md`, for round-trip, differential, and golden
  gates using deterministic fonts and the pinned Word corpus.
- `docs/hld/14-development-backlog.md`, for the F-264 through F-270 and
  F-X130 acceptance contracts, dependencies, sizes, and named test gates.
- `docs/hld/15-build-and-toolchain.md`, for package READMEs, Python long
  descriptions, release artifacts, and reproducible size evidence.

## The wave

| F-ID | Title | Size | Status | Owner |
|------|-------|------|--------|-------|
| F-264 | Complete paragraph property authoring | L | done | - |
| F-265 | Complete run property and inline authoring | L | done | - |
| F-267 | Complete table style and conditional formatting authoring | L | done | - |
| F-269 | Complete section page semantics | L | done | - |
| F-270 | Complete settings and web settings authoring | L | done | - |
| F-266 | International and vertical typography | L | done | - |
| F-266a | Script identity and font slot resolution | L | done | - |
| F-266b | Ruby and emphasis marks | L | done | - |
| F-266c | Character grid and vertical text | L | done | - |
| F-268 | Floating and advanced table layout | L | done | - |
| F-268a | Advanced table authoring and geometry | L | done | - |
| F-268b | Floating table placement and wrap | M | done | - |
| F-X123 | Accept producer TOC style variants | S | done | - |
| F-X124 | Make content cloning linear and explicit | M | done | - |
| F-X125 | Compare table grid changes | M | done | - |
| F-X126 | Preserve drawings through comparison acceptance | M | done | - |
| F-X127 | Collapse adjacent page break requests | S | done | - |
| F-X128 | Preserve Word paragraph and revision identities | M | done | - |
| F-X129 | Tolerate unmatched notes placeholders | S | done | - |
| F-X130 | Show package depth, footprint, and speed | L | done | - |
| F-X131 | Retain only the namespace declarations a root attribute uses | S | done | - |
| F-X132 | Match a retained namespace owner by structure, not by identity | S | done | - |

## Sequencing note

Rows are listed in dependency order, not F-ID order.

F-264, F-265, F-267, F-269, and F-270 can begin from the completed M23
foundations. F-266 follows F-264 and F-265 because its typography behavior
requires both complete paragraph and run properties. F-268 follows the
completed table property and conditional-style surface so its floating,
bidirectional, autofit, and advanced grid behavior resolves the final authored
table state.

F-266 and F-268 were split at implementation in the S74 consolidated design
round. F-266a carries script identity, font slot resolution, and the bundled
deterministic subset faces. F-266b carries ruby and emphasis marks. F-266c
carries the character grid and vertical text, and follows F-269 because F-269
owns `w:sectPr/w:textDirection` authoring while F-266c owns only its render
projection. F-268a carries authoring and advanced geometry and follows F-267.
F-268b carries the paginator float and follows F-268a. Each parent closes only
after every child closes, and neither parent is started directly.

F-X123 through F-X129 are independent compatibility corrections reported in
Issues 124 through 132. F-X124 covers the two reports on the same clone API.
The correction stories precede the planned M24 authoring work so later stories
build on the repaired round-trip, comparison, layout, and binding contracts.

F-X130 follows F-264 through F-270. It documents the final S74 public surface
across every crate and both Python distributions, with dated and reproducible
evidence for differentiated capability, package footprint, and speed.

## Definition of done for this sprint

- Every supported paragraph property has a public reader and setter, reopens
  as modeled content, and preserves unrelated producer XML.
- Full run formatting, theme behavior, symbols, special characters, and
  ordered inline content match the pinned Word reference through save, reopen,
  and render.
- Mixed Arabic, Hebrew, Korean, Japanese, and Latin text authors and renders
  with pinned deterministic geometry and reading order.
- Table styles, conditional regions, band sizes, look values, and property
  layers resolve and render like the pinned Word-authored table.
- Fixed, autofit, nested, bidirectional, and floating tables match reviewed
  Word page geometry and pagination.
- Page borders, line numbering, columns, separators, vertical alignment,
  mirrored margins, book-fold settings, paper sources, and note configuration
  survive round-trip and change only their declared layout behavior.
- Public-authored settings and web settings report no unmodeled supported
  children, support typed removal and diagnostics, and preserve unknown
  extensions byte for byte.
- The root and all 26 crate READMEs, including both PyPI long descriptions,
  lead with their deepest implemented outcomes and carry reproducible,
  appropriately scoped capability, footprint, and speed evidence.
- The full workspace, deterministic hash harness, Word corpus, packaging,
  documentation, binding, and supply-chain gates pass without unexplained
  output changes.
