# F-X139, Word identity and comparison contribution wave

**Status**: completed
**Sprint**: S76
**Size**: L
**Depends on**: F-X138

## Problem

PRs 183, 184, 190, 193, 198 and 205 repair producer-prefix handling,
identity preservation and comparison behavior. The present comparison entry
point is `crates/rdocx/src/comparison.rs:226`. Field and TOC parsing reaches
`crates/rdocx-oxml/src/text.rs`. The PR heads share ancestor changes and
source files with F-X138. Replaying a whole head could silently replace that
feature's story and replacement semantics.

## Spec reference

- `docs/hld/03-architecture.md`, "What stays put" and "Facade conventions".
- `docs/hld/04-opc-and-packaging.md`, "The package".
- `docs/hld/10-bindings-spec.md`, "Native Word facade stability" and "CLIs".
- `docs/hld/12-testing-strategy.md`, "Test taxonomy" and "The hash harness".

## Approach

After F-X138's completed checkpoint, review and replay each PR's unique
incremental diff. Carry inherited namespace scope into TOC and text-box run
parsing. Preserve producer identity attributes on modeled edits and remove
copied paragraph or row identities where duplication would be invalid.
Normalize only comparison noise that the contract declares equivalent. Report
unsupported metadata differences as diagnostics rather than refusing a valid
pair. Keep marker order tied to the accepted run projection. Expose CLI
comparison options and comment dates through the existing command surface.
The semver impact is additive for pre-1.0 CLI comparison flags and comment
date output, with corrected comparison equivalence and producer identity
preservation in existing native APIs.

## Rejected alternatives

- Compare serialized XML bytes. Producer attributes and run segmentation
  would produce false redlines.
- Import all shared ancestor commits from PR 190 or 193. F-X138 owns their
  overlapping story behavior.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| regression | Issue 159 identity rows in the existing rdocx regression entrypoint | Identity-only changes create no redline, valid producer identities survive edits and copied identities are renewed. |
| regression | PR 183 TOC and text-box producer cases | Rebuild accepts inherited run prefixes and retains unchanged producer XML. |
| regression | Issue 161 comparison option and marker cases | CLI and native options agree, accepted text and comment markers are ordered, and unsupported differences yield diagnostics. |
| round-trip | PR 193 table row identity case | Modeled table and paragraph edits retain unrelated producer attributes. |

**Test gate**: regression, as stated in the backlog. Run focused `rdocx`,
`rdocx-oxml` and CLI tests, then the scoped gate.

## HLD impact

- `docs/hld/03-architecture.md`
- `docs/hld/04-opc-and-packaging.md`
- `docs/hld/10-bindings-spec.md`
- `docs/hld/12-testing-strategy.md`

## Risk routing

- Layout cache safety: read `docs/hld/08-rendering-spec.md`. Keep
  deterministic font mode for the row-identity cache test and verify the
  cached table path preserves rendered output.
- Parser or serialiser: read `docs/hld/04-opc-and-packaging.md` and
  `06-presentationml-model.md`. Verify schema order, prefix-tolerant reads
  and a byte-preserving unmodelled subtree round-trip.
- Public API of published crates: read `docs/hld/10-bindings-spec.md`, state
  semver impact, run `cargo publish --dry-run` for touched published crates
  and check the `.crate` size ceiling.

## Hash harness

Expected unchanged. A changed baseline is not part of S76.

## Implementation checklist

- [x] Review each incremental PR diff against its actual parent.
- [x] Reconcile namespace, identity and comparison changes with F-X138.
- [x] Prove native and CLI options and producer cases in existing test binaries.
- [x] Run focused and risk checks, then microscope to zero findings.

## Open questions

None. Remaining Issue 159 and 161 matrix completion belongs to S78.
