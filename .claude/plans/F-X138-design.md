# F-X138, Word story and content contribution wave

**Status**: completed
**Sprint**: S76
**Size**: L
**Depends on**: F-X137

## Problem

Eight contribution heads address story visibility, direct body indexes,
replacement, comments and text boxes. The current public body and replacement
paths include `crates/rdocx/src/document.rs:14636`, `:14680`, `:14996` and
`:21220`. Several PRs are stacked and include ancestor commits from other
waves. PR 191 includes PR 179. PR 195 includes changes from PRs 180 and 184.
PRs 202, 210 and 211 extend PR 195. Applying whole heads would duplicate
behavior and blur attribution.

## Spec reference

- `docs/hld/03-architecture.md`, "What stays put" and "Facade conventions".
- `docs/hld/04-opc-and-packaging.md`, "The package".
- `docs/hld/05-drawingml-model.md`, "What already exists", text boxes.
- `docs/hld/10-bindings-spec.md`, "Native Word facade stability" and "CLIs".
- `docs/hld/12-testing-strategy.md`, "Test taxonomy" and "Binding tests".

## Approach

After F-X137's completed checkpoint, review and replay only the unique
incremental changes in PRs 177, 179, 180, 191, 195, 202, 210 and 211. Keep
the explicit stack edges 195 before 202 and 210, and 202 before 211. Map
public run and body indexes to direct body coordinates, traverse modeled
content controls and related stories once, and preserve every text-box child
and inherited namespace scope during replacement. Anchor comments using the
same run space exposed by the public paragraph view. Keep unknown XML opaque.
The semver impact is additive for pre-1.0 native and Python comment and story
entry points. Existing body-index behavior is corrected to its documented
direct-child contract.

## Rejected alternatives

- Cherry-pick complete stacked PR branches. They contain ancestor commits
  assigned to other F-IDs and repeat common README changes.
- Flatten nested stories into body indexes. Their physical ownership and
  relationship scopes are distinct.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| regression | Issue 163 direct-index cases in the existing rdocx regression entrypoint | `find_content_index`, `split_run`, bookmark and comment coordinates agree after a table or block control. |
| regression | Issue 172 comment entrypoints | Paragraph, story and text-range comments anchor the requested runs and survive save and reopen. |
| regression | Issue 160 content-control location matrix | Text, replacement and counts reach modeled controls, tables, notes, insertions and text boxes exactly once. |
| round-trip | Producer text-box and wrapper cases | Every unmodelled child and inherited namespace survives a targeted replacement. |
| integration | Existing CLI and Python entrypoints | Rust, CLI and Python project the same supported story content and indexes. |

**Test gate**: regression, as stated in the backlog. Run focused tests for
`rdocx-oxml`, `rdocx`, `rdocx-cli` and the Python binding, then the scoped
gate.

## HLD impact

- `docs/hld/03-architecture.md`
- `docs/hld/04-opc-and-packaging.md`
- `docs/hld/05-drawingml-model.md`
- `docs/hld/10-bindings-spec.md`
- `docs/hld/12-testing-strategy.md`

## Risk routing

- Parser or serialiser: read `docs/hld/04-opc-and-packaging.md` and
  `06-presentationml-model.md`. Verify schema order, prefix-tolerant reads
  and byte-identical unmodelled subtree preservation.
- Public API of published crates: read `docs/hld/10-bindings-spec.md`, state
  semver impact, run `cargo publish --dry-run` for touched published crates
  and check the `.crate` size ceiling.
- PyO3 bindings: read `docs/hld/10-bindings-spec.md`, run the existing Python
  smoke tests and the WASM target check where the native surface changes.

## Hash harness

Expected unchanged. A changed deterministic output must be explained before
the F-X138 checkpoint.

## Implementation checklist

- [x] Review each incremental PR diff and identify ancestor commits already accepted.
- [x] Reconcile body coordinates, story traversal, replacement and text boxes.
- [x] Add source-built regression cases to existing test binaries.
- [x] Run focused and risk checks, then microscope to zero findings.

## Open questions

None. The stack order and acceptance matrices are stated in the S76 plan.
