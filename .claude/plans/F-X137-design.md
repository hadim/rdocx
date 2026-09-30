# F-X137, Package and CLI safety contribution wave

**Status**: completed
**Sprint**: S76
**Size**: L
**Depends on**: F-X136

## Problem

The five contribution heads address output refusal, atomic saves, namespace
bindings, comments roots and package class selection. They were built against
the S75 `main` base and overlap at `crates/rdocx/src/document.rs`, the CLI
commands and existing integration tests. The current save entry points are at
`crates/oxml-opc/src/package.rs:251` and
`crates/rdocx/src/document.rs:11991`. Replaying whole heads would also replay
shared documentation and archive measurement commits without reviewing their
incremental behavior.

## Spec reference

- `docs/hld/04-opc-and-packaging.md`, "The package" and "Package integrity".
- `docs/hld/03-architecture.md`, "Facade conventions".
- `docs/hld/10-bindings-spec.md`, "CLIs".
- `docs/hld/12-testing-strategy.md`, "The hash harness".

## Approach

Review the incremental diffs of PRs 174, 178, 182, 185 and 197 against each
PR's base. Replay their behavior in that order on the S76 prefix, resolving
overlap against the approved package, save and CLI contracts. Preserve producer
part bytes and namespace declarations when a typed part is unchanged. Keep
output-path refusal before any mutation, stage writes atomically, and derive
the saved package class from the selected output extension. Keep the public
entry points additive and do not create a second package writer.
The semver impact is additive on pre-1.0 public APIs: path-specific byte
serialisation and CLI force options gain new entry points, while save behavior
changes intentionally under the existing methods.

## Rejected alternatives

- Merge each whole PR head. The heads share README measurements and source
  files, so this would obscure which behavior produced a change.
- Update the hash baseline. This wave does not own rendering output changes.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| regression | Existing CLI integration entrypoints plus Issue 156 refusal cases | Input-as-output and existing output without force leave bytes unchanged, and a closed stdout reader exits cleanly. |
| regression | Existing package and rdocx regression entrypoints plus Issues 157 and 164 cases | Save errors leave the destination intact, symlink and mode behavior is retained, and unchanged producer parts stay byte-identical. |
| round-trip | Namespace and package-class cases from PRs 182, 185 and 197 | Edited roots bind every used prefix, no-op comments retain source bytes, and DOCX, DOCM, DOTX and DOTM save with the selected class. |

**Test gate**: regression, as stated in the backlog. Run focused tests for
`oxml-opc`, `rdocx`, `rdocx-cli` and `rpptx-cli`, then the scoped gate.

## HLD impact

- `docs/hld/04-opc-and-packaging.md`
- `docs/hld/03-architecture.md`
- `docs/hld/06-presentationml-model.md`
- `docs/hld/10-bindings-spec.md`

## Risk routing

- Parser or serialiser, per `.claude/skills/risk-routing.md`: read
  `docs/hld/04-opc-and-packaging.md` and `06-presentationml-model.md`. Check
  schema child order, prefix-tolerant reads and a byte-preserving round-trip
  for unmodelled subtrees.
- Public API of published crates: read `docs/hld/10-bindings-spec.md`, state
  semver impact, run `cargo publish --dry-run` for touched published crates
  and check the `.crate` size ceiling.

## Hash harness

Expected unchanged. Any delta stops integration until attributed.

## Implementation checklist

- [x] Review each PR's incremental commits and record accepted or rejected changes.
- [x] Reconcile save, CLI and namespace changes on the S76 prefix.
- [x] Keep behavior changes separately labelled from measurement or documentation changes.
- [x] Run the stated regression, round-trip and risk checks.
- [x] Run microscope passes until zero defects and zero smells.

## Open questions

None. The S76 contribution inventory and package contracts define the wave.
