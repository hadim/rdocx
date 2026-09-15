# F-256, Transactional cross-document fragment import

**Status**: completed
**Sprint**: S72
**Size**: L
**Depends on**: F-246, F-247, F-248, F-249, F-250, F-251, F-252, F-253, F-254, F-255

## Problem

`Document::insert_document` at `crates/rdocx/src/document.rs:11116` copies an
entire body selection and remaps only styles and numbering. The staged helper
at `crates/rdocx/src/document.rs:11125` has no selected subtree abstraction or
caller conflict policy, and it does not close bookmarks, comments, media,
drawings, charts, embedded parts, fields, and part-local relationships.

## Spec reference

- `docs/hld/02-scope-and-non-goals.md`, "Modern DOCX capability matrix",
  DOCX-021.
- `docs/hld/03-architecture.md`, staged package mutation and fragment closure.
- `docs/hld/04-opc-and-packaging.md`, fragment dependency closure and atomic
  publication.
- `docs/hld/05-drawingml-model.md`, relationship-bearing drawing preservation.
- `docs/hld/09-charts-spec.md`, chart and embedded workbook closure.
- `docs/hld/10-bindings-spec.md`, "Native Word facade stability".
- `docs/hld/14-development-backlog.md`, "F-256, Transactional cross-document fragment import".

## Approach

Add concrete `DocumentFragment` and non-exhaustive `FragmentConflictPolicy`
values. A source document creates a fragment from one valid F-253 content range
and records its source owner. Import first discovers and validates the complete
supported dependency closure, then reserves every destination style,
numbering, bookmark, comment, drawing, media, chart, embedded part,
relationship, and content-type identity. Apply deterministic reuse or rename
policy per supported conflict class. Rewrite typed and preserved
relationship-bearing XML only after all maps exist. Publish only after the
staged candidate serializes, reopens, and validates. Unsupported or external
dependencies fail without changing the destination.

## Rejected alternatives

- Reusing whole-document insertion would copy unrelated content and parts.
- Lazy allocation during copy could leave half a graph after a late failure.
- Raw string replacement cannot distinguish namespaces, attributes, or owner
  scopes safely.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| regression | `dependency_rich_fragment_imports_twice_without_collisions` | Two imports reopen with styles, numbering, bookmarks, comments, media, drawings, charts, embeddings, fields, and relationships resolved. |
| round-trip | `fragment_conflict_policies_are_deterministic` | Equivalent inputs and policies produce identical maps and output bytes. |
| regression | `unsupported_fragment_dependency_aborts_without_mutation` | Unsupported, dangling, external, malformed, and exhausted closures leave destination bytes unchanged. |

The story test gate is regression: a dependency-rich fragment imports twice
without collisions and reopens with every reference resolved.

## HLD impact

- `docs/hld/02-scope-and-non-goals.md`
- `docs/hld/03-architecture.md`
- `docs/hld/04-opc-and-packaging.md`
- `docs/hld/05-drawingml-model.md`
- `docs/hld/09-charts-spec.md`
- `docs/hld/10-bindings-spec.md`
- `docs/hld/12-testing-strategy.md`
- `docs/hld/13-risks-and-open-questions.md`
- `docs/hld/14-development-backlog.md`

## Risk routing

- Any parser or serialiser. Read `docs/hld/04-opc-and-packaging.md` and
  `docs/hld/06-presentationml-model.md`. Prove schema order, prefix-tolerant
  reads, fixed-prefix writes, and byte-exact retained subtrees.
- Public API of a published crate. Read `docs/hld/10-bindings-spec.md` and the
  structural rules in `CLAUDE.md`. State additive pre-1.0 semver impact, run
  `cargo publish --dry-run -p rdocx`, and assert the packaged crate size.

## Hash harness

Expected to be unchanged. Existing whole-document insertion behavior stays
compatible and no baseline is re-recorded.

## Implementation checklist

- [x] Add owned fragment selection and explicit conflict policy values.
- [x] Discover the complete supported dependency closure before mutation.
- [x] Reserve deterministic destination identities and build complete maps.
- [x] Rewrite typed and retained references with owner scope preserved.
- [x] Serialize, reopen, validate, and publish the staged candidate atomically.
- [x] Test repeated import, every supported dependency class, and rejection.

## Open questions

None. F-256 covers main-body ranges plus their owned closure. Callers choose
conflict policy independently for styles, numbering, and related parts.
Equivalent reuse compares modeled semantics plus exact preserved payloads.
Final body section properties import only in explicit section-inclusive mode.
