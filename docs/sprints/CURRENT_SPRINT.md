# Current Sprint, S91.1

**Milestone**: M24, X, release runtime and CI repairs.

**Goal**: Fix the concrete native CI setup and default-WASM parser failures found after S91 merged. Requalify the still-unpublished Word0.16.1 and shared/PowerPoint0.14.1 families, preserving the frozen40-issue/30-PR scope and open discussions264/281.

## Spec references

- `docs/hld/04-opc-and-packaging.md`, package identity and verbatim opaque XML.
- `docs/hld/06-presentationml-model.md`, shape-tree parser storage and source preservation.
- `docs/hld/10-bindings-spec.md`, actual default-stack WASM facade and package round-trip.
- `docs/hld/12-testing-strategy.md`, regression, binding and exact source/oracle verification.
- `docs/hld/15-build-and-toolchain.md`, pinned tools, CI job matrix and closed-main releases.

## The wave

| F-ID | Title | Size | Status | Owner |
|------|-------|------|--------|-------|
| F-X224 | Repair hosted release tool and stack routing | M | pending | - |
| F-X225 | Keep DrawingML text-style parsing within the default WASM stack | M | pending | - |

## Sequencing note

Both dependencies are already done in S91. The corrective stories run in parallel in separate worktrees with disjoint source leases. ROOT owns shared records, integration and final artifact qualification. No S92 feature work or new contributor intake is included.

## Definition of done for this sprint

- Preserve original main CI38662802341 failures and exact Node24.11.1 reproduction.
- Pass existing native font/animation consumers and exact executable fractional release-tag controls.
- Pass the unchanged real PowerPoint Node facade under the default1MiB stack, with public APIs, opaque XML, child order and parser bounds intact.
- Complete normal scoped lifecycle, zero-finding independent microscopes, fresh integrated full verification and clean sprint review.
- Qualify actual current paired Rust/CLI/Python artifacts,39/312 installed runtime/typing, font104/native-eight riders, primary hosted CI and exact-SHA build-only rehearsal.
- Close normally without changing S91's tag. Publish both prepared families only after exact-main checks, then reconcile the70 humanized comments. Discussions264 and281 remain open, and all existing S92 work remains deferred.
