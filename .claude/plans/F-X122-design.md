# F-X122, Recover the immutable rpptx 0.12.0 release attempt

**Status**: completed
**Sprint**: S73
**Size**: M
**Depends on**: F-X111, F-X119, F-X121

## Problem

The `rpptx-v0.12.0` release workflow built all six CLI archives, then the
aggregate validator rejected the Windows archive because Git converted the
packaged README from LF to CRLF. The README text itself was unchanged. No crate
was published and no GitHub release was created, but the tag is immutable and
cannot run revised workflow code.

## Spec reference

- `docs/hld/03-architecture.md`, "Versioning".
- `docs/hld/10-bindings-spec.md`, "Packaging" and "WASM".
- `docs/hld/12-testing-strategy.md`, release workflow and registry proofs.
- `docs/hld/14-development-backlog.md`, "F-X122, Recover the immutable rpptx 0.12.0 release attempt".
- `docs/hld/15-build-and-toolchain.md`, "Publishing" and "Release process".
- `.claude/commands/release.md`, immutable release tags and external verification.

## Approach

Normalize CRLF to LF only when comparing packaged README and licence payloads
with their reviewed checkout sources. Preserve all structural, executable,
nonempty, content, checksum, and workflow ordering checks. Move the complete
incubating source family, its unpublished WASM and Python carriers, workspace
pins, lock entries, examples, assertions, release notes, and notification
mapping from 0.12.0 to 0.12.1. Stable crates remain at 0.14.0 while their shared
dependency requirements move to the available 0.12.1 family.

Keep `rpptx-v0.12.0` at its original reviewed SHA. F-X112 will publish the
recovery as `rpptx-v0.12.1`, then `v0.14.0`, `py-rdocx-v0.14.0`, and
`py-rpptx-v0.12.1` through four separate release ceremonies.

## Rejected alternatives

- Move or delete `rpptx-v0.12.0`. Release tags are immutable evidence.
- Ignore README and licence content on Windows. That would weaken the asset
  contract beyond the observed newline representation difference.
- Publish only one patched crate. The incubating family and its Python
  distribution are version-aligned lockstep families.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| release regression | `rpptx_0_12_1_recovery_is_version_aligned_and_line_ending_safe` | Every recovery carrier is 0.12.1, four tags agree, CRLF text is accepted, and changed text plus weakened archive checks are rejected. |
| release preparation | `s73_release_contract_requires_four_version_aligned_families` | The exact stable and incubating allowlists, dependencies, notes, and notification inventories agree with the recovery tags. |
| package | patched workspace dry run | All 22 publishable archives stage under the recovery dependency graph and remain below the size limit. |

The **test gate** is the release regression named in the backlog.

## HLD impact

- `docs/hld/03-architecture.md`
- `docs/hld/10-bindings-spec.md`
- `docs/hld/12-testing-strategy.md`
- `docs/hld/14-development-backlog.md`
- `docs/hld/15-build-and-toolchain.md`

## Risk routing

- Release scripting and version strings. Inspect every manifest, lock entry,
  README, workflow literal, notes section, tag mapping, and registry absence.
  Require the full gate and fresh approval before any recovery tag.
- Crate dependency graph. Prove all stable shared requirements resolve to the
  published 0.12.1 family in dependency order.
- Public API of published crates. Run the exact package dry run and archive
  size gate for all 22 publishable crates.
- WASM or PyO3 bindings. Run both WASM graphs, the Python exclusions and rider,
  and build the selected Python wheel set before publication.
- Bundled fonts and assets. Preserve the complete `oxml-layout` legal and font
  inventory and validate all CLI archive members.

## Hash harness

Expected unchanged across all 49 entries. Version preparation and newline
normalization do not change document or rendering output.

## Implementation checklist

- [x] Add a failing regression for CRLF-safe text validation and the coherent
      0.12.1 recovery contract.
- [x] Normalize only README and licence newlines in the aggregate validator.
- [x] Move every incubating carrier, pin, lock entry, example, and assertion to
      0.12.1 while retaining stable 0.14.0.
- [x] Add reviewed 0.12.1 Rust and Python release notes without rewriting the
      immutable 0.12.0 record.
- [x] Update F-X112 notification and release mappings to the recovery tags.
- [x] Run every routed check, the full gate, and a clean microscope review.

## Open questions

None. The user approved ignoring Windows newline representation differences
and confirmed the 0.12.1 recovery tag set.
