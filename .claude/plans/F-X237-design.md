# F-X237, Restore linear Python table owner inventory

**Status**: approved
**Sprint**: S92
**Size**: L
**Depends on**: none

## Problem

[Issue378](https://github.com/tensorbee/rdocx/issues/378), reported by Hadrien
Mary, describes quadratic Python table open and single-edit cost and cubic
cell-by-cell fill after0.16.1. His240-cell fill is133.53s versus1.89s on
release0.16.0. Those are reporter measurements, not local results.
`word_owner_locations` enumerates coordinates and invokes the scalar native
cell provider for every cell (`crates/rdocx-py/src/document.rs:3968`,4008,4017).
That provider reparses the table, materializes source, scans owners and restarts
cell traversal (`crates/rdocx/src/document.rs:22201`). Separately,
`prepare_word_revision` searches every old owner for each candidate owner
(`crates/rdocx-py/src/document.rs:4854`). Batching alone leaves this quadratic.

## Spec reference

- `docs/hld/14-development-backlog.md`, "F-X237, Restore linear Python table owner inventory".
- `docs/hld/03-architecture.md`, "Owned publication and causal paths".
- `docs/hld/10-bindings-spec.md`, "Word checked XML owner authority" and "Word tables, physical pictures and internal links".
- `docs/hld/12-testing-strategy.md`, "Binding tests", "Word story inventory controls", "Word table, picture and paired-owner controls" and "Word checked XML qualification".
- `docs/hld/04-opc-and-packaging.md`, "The package", source ownership and namespace preservation.
- `docs/hld/06-presentationml-model.md`, "Parts", qualified XML preservation rules.

## Approach

Production changes stay in existing native and Python `document.rs` files.
Tests use their existing entrypoints and `crates/rdocx-py/tests/test_core.py`.
No new product file, module, trait, generic, feature or dependency is authorized.
One additive hidden public native API serves an existing cross-crate consumer:

```rust
#[doc(hidden)]
pub fn story_table_cell_snapshots(
    &self,
) -> Result<Vec<(ContentLocation, usize, usize, StoryId)>>;
```

Each tuple binds an actual physical table, physical row and cell coordinates,
and checked source cell identity. Materialize and qualify each physical source
once. Match source-ordered owner spans against one physical modeled traversal,
including nested tables and interleaved controls. Parse each outer modeled
table once and traverse its already parsed nested models, or use equivalent
one-pass source events. Never reparse each nested subtree or revisit all
descendant cells for each ancestor table. Count all such production work.
Visit each physical table and cell once. Do not filter all owners anew per table, restart
`nth_cell_in_table`, infer physical identity from logical grid columns, or return
before complete correspondence and namespace/fingerprint validation. Preserve
existing refusal boundaries and source admission depth limits. Keep the scalar
provider as a small-fixture equivalence oracle, without using it per production
result. The binding consumes the batch while retaining its current table/cell,
paragraph, run, hyperlink and section order and actual parent joins.

Replace the nested old-owner search with an ephemeral physical-route index.
Do not treat `same_inventory_route` as an equivalence relation. Its branch
precedence depends on both operands' body-slot and parent classes. Partition
old root indexes by Some-slot, None-slot fallback and projection identity.
A candidate Body Some route queries matching Some-slot routes and admissible
None-slot projection routes. A candidate None route queries projection routes
across old classes. Preserve the body paragraph path-length and trailing-control
suffix conditions. Related roots and independent section/hyperlink roots retain
exact current identity. Normalize hyperlink removal indices through the same
checked index shift as the current special case.

For descendants, retain every admissible old parent candidate until child
matching completes, then choose the earliest matching old child index. Never
choose only the first parent early. Index child routes under their actual old
parent index using the current local relation: cell row/physical-cell, accepted
run path, or paragraph/table projection ordinal with exact parent-class branch
selection. Validate the assembled bounded candidates with the existing predicate.
Preserve earliest old match, including synthetic duplicate-route controls.
Do not change matching to last-match, injective matching or a new refusal.

The production bound must be proved by actual generator invariants: per-story
projection ordinals, direct-body slot plus control suffix, physical row/cell per
table, and accepted run path per paragraph are unique. At most two old root
candidates arise from slot versus fallback projection, and each physical parent
contributes at most one child match, so the candidate set does not grow with
depth. `commit_word_revision` replaces the current owner vector, rather than
accumulating historical owners (`crates/rdocx-py/src/document.rs:5000`). Verify
these facts with actual inventories before claiming linear cost. Small fabricated
duplicates must remain legacy-correct, with their exceptional multiplicity
reported separately from the valid generated-inventory bound. Use a bounded
iterative dependency stack for forward parent references, without recursion,
growing copied ancestor keys or repeated whole-list retries.

Keep selected-owner lookup, descendant and ancestry scope rules, regular Run
epoch, first stale-operation cause, no-op behavior, staged revision/allocation
and one success-only publication unchanged. No persistent inventory cache or
validation bypass is needed. Any residual nonlinear loop on the admitted open
or single-edit path must be investigated rather than hidden behind timings.

## Rejected alternatives

- Batch native cells while leaving quadratic revision-route reconciliation.
- Reuse stale inventory across edits or retire every owner to avoid proof.
- Substitute logical grid coordinates for actual merged or nested cell ownership.
- Add a new Python batch-edit API instead of repairing existing operations.
- Use only timing ratios or collapse the non-equivalence matcher into one key.

## Test plan

The story **test gate** is **regression**. Add genuine tests first and execute
actual Base RED before implementation. Existing test files are the only carriers.

| Category | Test | Asserts |
|---|---|---|
| unit | `fx237_table_cell_snapshots_scan_each_source_and_visit_each_cell_once` | Actual source/owner/namespace counters and new test-only table/cell visit counters prove one source materialization and qualified scan per physical source and one table/cell traversal at60,120,240 cells, multiple tables and nested tables. No toy implementation or timing-only complexity claim. |
| regression | `fx237_batched_cell_routes_equal_checked_single_cell_routes` | Every batched result equals the existing checked scalar route on small body, control, header/footer, note, comment and text-box fixtures. Include alias/foreign prefixes, merges, omitted cells, nested tables, row/cell controls, empty owners and correspondence refusal. |
| unit | `fx237_indexed_routes_match_legacy_first_match` | Compare production index choices with the test-only original matcher for all publication variants, shifted hyperlinks, duplicate routes, mixed body Some/None and parent-cell/non-cell classes, unsupported body paths and unequal ancestry. Earliest match remains exact. |
| unit | `fx237_revision_route_work_is_linear` | Actual generated inventories satisfy uniqueness, at most two parent probes, one old-entry indexing pass and one new-entry pass. Scale60,120,240 cells and repeated equal-looking tables. Account for admitted route depth and retain arbitrary small duplicate correctness controls. |
| regression | `test_fx237_table_owner_inventory_preserves_physical_routes_and_generations` | Held same-cell/table/unrelated paragraph handles, nested and identical-looking siblings, physical merges and omitted cells, related stories, ordinary text and replace_text preserve current survival/stale causes through repeated edits and save/reopen. |
| regression | `test_fx237_owner_inventory_refusal_is_atomic` | Foreign/stale/manual owners, unsupported opaque source, invalid grid/merge and existing late-allocation/revision failure seams leave bytes, generations and held handles unchanged. No-op publishes nothing. |
| regression | `test_fx237_table_open_edit_and_fill_costs` | Exact Issue37860/120/240-cell fill/open/edit, including40-by6, preserves final content through save/reopen. Default functional check remains bounded. Explicit installed release measurement enforces the declared same-host timing gates. |
| round-trip | existing checked-owner and F-X211 Word controls | Exact unmodelled payload, relationships, source order, physical owner provenance and reopened resolved document remain unchanged. |

Pure binding Rust tests use an existing supported non-extension test build.
Never pretend the workspace extension-module test binary links. If a private
helper test needs a carrier, use the existing binding file without a new module.

Timing qualification uses authenticated release0.16.0 and a fresh candidate
release wheel on the same interpreter, hardware and process configuration.
Pin exact wheel filename and SHA against release SHA256SUMS. Record source/Base,
compiler/profile, wheel/native identities, OS/CPU and interpreter. Warm up once,
then five open/edit trials and three fill trials per60,120,240 cells. Retain all
samples and medians, with seed generation excluded from open/edit timing.
Include no-table and unrelated-paragraph-replace_text controls. Candidate
medians must satisfy open <= max(3 * baseline,0.010s), single edit <=
max(3 * baseline,0.030s), and fill <= max(3 * baseline,0.500s) at every size.
The exact240-cell fill is required. Deterministic work counts are the asymptotic
gate, not a flaky ratio alone. Authenticate one actual broken Base reproduction
and its work counts. Do not repeat three expensive broken240 fills without a
new ambiguity. If the same-host baseline cannot run, timing acceptance remains
incomplete. Never relabel the reporter's M3 Max measurements as local evidence.

Build separate debug-guarded opt-level2 Word qualification and release timing
wheels when their profile contracts differ. Retain both exact wheel identities
and never substitute the release measurements for debug-guarded acceptance.

Run scoped native check/tests, existing preservation and atomicity matrices,
changed-crate clippy and fresh installed complete Word Python suite. Preserve
all collected Word tests and its existing authenticated runtime prerequisites,
including font nodes, when that full suite is executed. The PowerPoint full
font matrix is not newly earned. Run independent microscope and scoped verify.

## HLD impact

- `docs/hld/03-architecture.md`
- `docs/hld/10-bindings-spec.md`
- `docs/hld/12-testing-strategy.md`

Describe current batched physical inventory, exact indexed matching and cost
qualification. The intake story in14 is already canonical. Do not add history to
the mechanism documents.

## Risk routing

- **Public API of a published crate**: one additive hidden native method, no
  breaking change or Python API expansion. Read bindings and structural rules.
  Denied-warning rustdoc, actual verified workspace-patched publication dry run
  and nonempty archives below10MiB apply. Scoped dirty-source packaging may use
  the explicit verify exception with the flag recorded. Final clean full gate
  must omit allow-dirty. Remeasure actual source archive carriers with unchanged
  compression tolerance and limits, then independently reconcile combined source.
- **PyO3 bindings**: both WASM graphs and fresh installed complete Word Python
  suite at the scoped gate, alongside changed-native-crate checks. Full workspace
  tests excluding both extension crates remain the final integrated floor, not
  an added worker full-workspace gate. Bind exact
  source, wheels, native imports, compiler/CLI/oracle inputs and collected nodes.
  Full Word suite retains its actual existing runtime prerequisites. No new
  PowerPoint font/full-suite rider absent a concrete interaction or failure.
- **Qualified XML traversal**: OPC/model reading, exact opaque subtree and
  namespace/physical relationship round trips and atomic refusal. Writes and
  schema child order remain unchanged. Reuse existing source admission limits.
- **External baseline comparison**: pin actual0.16.0 release artifact and hashes,
  interpreter and environment, original Base and current source, all repetitions
  and raw outcomes. No new Office or LibreOffice appearance baseline is earned.

## Hash harness

Expected unchanged, all49 entries. An unexplained delta blocks completion.
No font or rendering baseline changes are authorized.

## Implementation checklist

- [ ] Establish canonical intake, approved plan and isolated claim.
- [ ] Record actual Base RED, work counts and broken timing reproduction.
- [ ] Implement complete batched physical correspondence.
- [ ] Prove exact bounded route matching and first-match compatibility.
- [ ] Pass preservation, generations, atomic refusal and installed timing gates.
- [ ] Pass fresh complete Word Python suite and all scoped risk riders.
- [ ] Pass independent microscope and prepare a validating handoff without push.

## Open questions

No unresolved product choice. Root independently approves the explicit small-factor
thresholds as a concrete reading of Issue378 acceptance. Exact index correctness
and valid-inventory bounds are proof obligations, not an assumption that the
legacy relation is an equivalence. Failure to prove them blocks implementation
acceptance and requires a design revision. Original five S92 stories remain intact.
