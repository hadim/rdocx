# Current Sprint, S73

**Milestone**: M23 From-scratch business documents.

**Goal**: close M23 with complete corpus authoring, resolve Issues 75, 83
through 100, and 115 through 121, and incorporate PR 101 through PR 123 through
verified native, binding, rendering, and release outcomes.
Five generators must start from `Document::new()`, use only the public `rdocx`
facade, and reproduce the required private references without templates, raw
XML injection, or post-processing. The reviewed result prepares and publishes
both Rust package families and both version-aligned Python distributions.

## Spec references

- `docs/hld/02-scope-and-non-goals.md`, for the DOCX-022 through DOCX-028
  capability boundaries and the public-authoring classifications S73 closes.
- `docs/hld/03-architecture.md`, for facade ownership, ordered content,
  package-wide staged mutation, layout ownership, and renderer-only watermark
  projection.
- `docs/hld/04-opc-and-packaging.md`, for exact table measurement values,
  relationship-scoped assets, schema-ordered serialization, and atomic package
  publication.
- `docs/hld/05-drawingml-model.md`, for WordprocessingDrawing wrappers, anchor
  and wrapping semantics, text-box bodies, and preserved compatibility
  branches.
- `docs/hld/08-rendering-spec.md`, for table grid and merge lowering,
  caller-width measurement, story-aware drawings and watermarks, and
  post-pagination field substitution.
- `docs/hld/10-bindings-spec.md`, for the native Rust public facade boundary
  used by the blank-document generators and the deliberate binding limits.
- `docs/hld/12-testing-strategy.md`, for sanitized differential fixtures,
  deterministic rendering, private-artifact leak prevention, and the required
  five-document conformance mode.
- `docs/hld/13-risks-and-open-questions.md`, for the closed M23 capability
  vocabulary, private-corpus confidentiality boundary, and atomic cross-part
  invariant rule.
- `docs/hld/14-development-backlog.md`, for the F-257 through F-263 and
  F-X097 through F-X122 acceptance contracts, dependencies, sizes, issue
  mappings, and release gate.
- `docs/hld/15-build-and-toolchain.md`, for package-family allowlists, portable
  CLI assets, version alignment, trusted publication, and release approval.

## The wave

| F-ID | Title | Size | Status | Owner |
|------|-------|------|--------|-------|
| F-257 | Complete M23 table authoring | L | done | - |
| F-260 | Ordered run content authoring | L | done | - |
| F-X097 | Preserve namespace-scoped drawings and complex fields in comparison | M | done | - |
| F-X098 | Preserve content-control type payloads | M | done | - |
| F-X099 | Expose direct body ownership for story items | M | done | - |
| F-X100 | Preserve explicit false table toggles | S | done | - |
| F-X102 | Resolve header and footer pictures in their story scope | M | done | - |
| F-X104 | Render DrawingML picture transparency | M | done | - |
| F-X105 | Separate slide-owned placeholders from master header flags | M | done | - |
| F-X110 | Control field updates on document open | S | done | - |
| F-X111 | Attach portable CLI binaries to Rust releases | L | done | - |
| F-258 | Complete M23 row and cell authoring | L | done | - |
| F-X101 | Honor run-level page breaks during pagination | M | done | - |
| F-X103 | Accept standard TOC switches and report rebuild diagnostics | M | done | - |
| F-X106a | Expose indexed content mutation and counted replacement in Python | L | done | - |
| F-X106b | Expose paragraph and run formatting mutations in Python | M | done | - |
| F-X106c | Expose story mutation, hyperlinks, revisions, fields, and XML in Python | L | done | - |
| F-259 | Container measurement and equal-height layout | M | done | - |
| F-261 | Rich HTML fragments in arbitrary containers | L | done | - |
| F-262 | Corpus drawings, text boxes, and watermarks | L | done | - |
| F-X107 | Clone and remove existing table rows | M | done | - |
| F-X108 | Replace an existing picture atomically | M | done | - |
| F-X109 | Split text runs at Unicode character offsets | M | done | - |
| F-263 | Layout-backed fields and M23 corpus gate | L | done | - |
| F-X113 | Preserve appended paragraphs in document comparison | M | done | - |
| F-X114 | Rebuild TOC entries with document styles and geometry | M | done | - |
| F-X115 | Preserve modern comment metadata and identity | M | done | - |
| F-X116 | Make Python story reads linear and complete | L | done | - |
| F-X117 | Render transparent and large raster pictures safely | M | done | - |
| F-X118 | Make notes rendering and replacement safe | L | done | - |
| F-X119 | Complete round-three Python authoring and inspection | L | done | - |
| F-X120 | Accept fractional DOCX line spacing values | S | done | - |
| F-X121 | Adopt PR 123 authored line-chart portability | S | done | - |
| F-X122 | Recover the immutable rpptx 0.12.0 release attempt | M | done | - |
| F-X112 | Publish the complete S73 package families | L | done | - |

## Sequencing note

Rows are listed in dependency order, not F-ID order.

F-257, F-260, and independent rendering, settings, and release-tooling
remediations can begin from the completed S72 model. F-X101 and F-X109 consume
F-260's ordered runs. F-X102 precedes the drawing corpus story. F-X103 consumes
F-X098's content-control preservation and precedes field materialization.
F-X106a through F-X106c are serialized because they share the Python document
surface. F-258 builds on F-257 and F-X100, then F-X107 adds row cloning and
removal. F-X108 follows the completed story binding and relationship surface.
F-259 and F-261 consume the complete table and run models. F-262 consumes
F-X102 and F-260. F-X113 and F-X117 are independent correction stories.
F-X114 follows the completed field and corpus layout path. F-X115 follows exact
comment anchoring, F-X116 follows the complete story binding and split-run
paths, and F-X118 establishes notes mutation before F-X119 completes the
round-three binding surface. F-263 starts after every behavior needed by the
private corpus and reported pagination fields is integrated. F-X121 is an
independent chart portability correction from PR 123. F-X122 repairs the
failed immutable incubating release attempt. F-X112 is the final release
boundary after every non-release story through F-X122. The five client
documents and all derived evidence remain private and uncommitted.

## Definition of done for this sprint

- Public table authoring covers the corpus width modes, alignment, indentation,
  layout, shading, borders, cell margins, grid, style look, and explicit
  invisible borders, with Word-equivalent geometry and pagination.
- Public row and cell authoring covers exact and minimum height, repeating
  headers, split policy, alignment, grid omissions, merges, borders, margins,
  width, shading, vertical alignment, text direction, conditional formatting,
  wrapping, and nested tables without unexplained fallback.
- Producer-written fractional paragraph line spacing opens, rounds to the
  nearest integer twip, saves canonically, and retains deterministic layout.
- Authored line charts declare non-overlay axis-title layout and explicit false
  marker and smoothing defaults in schema order for portable viewer behavior.
- Caller-width measurement uses the same deterministic fonts and rules as
  whole-document layout, reports height and diagnostics without mutation, and
  produces matching equal-height nested-table layout.
- One run can author text, tabs, line, page, and column breaks, drawings,
  fields, and symbols in stable mixed-content order without formatting setters
  replacing non-text children.
- The supported bounded HTML subset inserts equivalent rich content into body,
  cell, header, and footer containers, including lists, tables, data-URI
  images, links, resource resolution, and ordered diagnostics for every drop.
- Public drawing APIs author corpus-required inline and floating images,
  cropping, sizing, anchors, wrapping, rotated text boxes, text direction,
  section-aware watermarks, and modeled compatibility branches without raw
  header XML.
- PAGE, NUMPAGES, PAGEREF, and supported TOC caches materialize from
  deterministic layout, including numbered headings inside table cells.
- Required private-corpus mode generates all five documents from blank public
  facade programs. Each output reopens without repair, matches required package
  semantics and reviewed visual thresholds, is byte-identical on repetition,
  and reports no unexplained preservation-only fallback.
- No private document, filename, extracted content, render, digest, customer
  metadata, or identifying manifest enters tracked or staged repository state.
- Comparison preserves drawings whose required prefixes are declared on an
  ancestor and maps complex-field source runs to modeled comparison ownership.
- Content-control type elements preserve their attributes and children until a
  caller explicitly changes the type.
- Story-item snapshots expose a safe optional direct-body owner without
  changing the existing recursive path.
- Explicit false row-header, row-split, and cell-wrapping toggles remain false
  through save and reopen.
- Run-level page breaks create physical page boundaries used consistently by
  layout, renderers, page fields, and TOC targets.
- Header and footer pictures resolve through their owning story relationships
  and render in deterministic PNG and PDF output.
- Word-default TOC `\\z` rebuilds, and Rust and Python callers receive exact
  ordered rebuild diagnostics.
- PR 101's table-toggle and TOC `\\z` contributions land directly or through a
  reviewed hardened equivalent with contributor credit retained.
- rpptx applies `a:alphaModFix` to slide and inherited pictures across every
  rendering backend.
- Slide-owned date, footer, and number placeholders follow the recorded oracle
  decision independently from inherited `p:hf` policy.
- Python exposes the Issue 94 mutation, formatting, story, collaboration,
  field, and read-only XML operations with typed and atomic behavior.
- Existing table rows clone and remove safely, pictures replace with
  relationship-local copy-on-write behavior, and runs split at checked Unicode
  character offsets for exact comment anchoring.
- Terminal comparison insertions accept and reject without synthetic empty
  paragraphs, and rebuilt TOCs retain localized styles, section-width tabs, and
  structural numbering suffixes.
- Modern comments use the standard content type, stable rdocx ids, optional
  dates, and typed table-cell comment paths.
- Python story inventory is linear and exposes visible runs inside accepted
  insertions and inline controls through checked live handles.
- Transparent raster pixels compose correctly, ordinary large pictures render,
  and every over-limit image reports a stable visible failure.
- Notes render without a redundant reverse relationship, notes text is
  editable, and rpptx replacement is guarded, counted, and includes notes.
- Python can insert Word pictures and inspect or edit the requested PowerPoint
  geometry, identity, font, autofit, notes, and property-preserving run text.
- `w:updateFields` is a typed optional native and Python setting.
- Stable and incubating Rust releases attach six portable CLI archives and a
  verified checksum manifest with working cargo-binstall metadata.
- The exact 7-package stable and 15-package incubating Rust families plus PyPI
  rdocx and rpptx publish through four separately approved release actions at
  one reviewed S73 SHA.
- Every included open or closed issue and pull request receives a human result
  comment after its implementation, merge, or publication outcome is verified.
