# Current Sprint, S91.1

**Milestone**: M24, X, release runtime and CI repairs.

**Goal**: Fix the native CI setup, default-WASM parser and checked chart relationship failures found after S91 merged, then deliver the original first-draft corpus CI criterion. Requalify the still-unpublished Word0.16.1 and shared/PowerPoint0.14.1 families, preserving the frozen40-issue/30-PR scope and open discussions264/281.

## Spec references

- `docs/hld/04-opc-and-packaging.md`, package identity and verbatim opaque XML.
- `docs/hld/06-presentationml-model.md`, shape-tree parser storage and source preservation.
- `docs/hld/10-bindings-spec.md`, actual default-stack WASM facade and package round-trip.
- `docs/hld/12-testing-strategy.md`, regression, binding and exact source/oracle verification.
- `docs/hld/15-build-and-toolchain.md`, pinned tools, CI job matrix and closed-main releases.

## The wave

| F-ID | Title | Size | Status | Owner |
|------|-------|------|--------|-------|
| F-X224 | Repair hosted release tool and stack routing | M | in-progress | codex |
| F-X225 | Keep DrawingML text-style parsing within the default WASM stack | M | in-progress | codex |
| F-X226 | Release superseded relationships after checked PowerPoint XML edits | M | in-progress | codex |
| F-X227 | Execute the unchanged first-draft compatibility corpus in owner CI | L | pending | |

## Sequencing note

All prerequisite features are already done in S91. F-X224 through F-X226 implemented in parallel in separate worktrees with disjoint source leases. F-X227 is approved for implementation. ROOT owns shared records, integration and final artifact qualification. No S92 feature work or new contributor intake is included.

## Current qualification

F-X224, F-X225 and F-X226 are implemented, independently reviewed and
integrated atb3b2163482ae42f68a66594ab2064481bd7b406b. Current local full
verification passes:6463 Rust tests,778 Word and854 PowerPoint Python tests,
78 priority cases on each of Python3.9/3.12, four typing checks,104 font
cases, eight native SVG owners, two F216 controls and Node2/2/3 at the
unchanged1MiB stack. Literal24 documentation cases retain34 byte-identical
outputs. Selected XML controls retain nine byte-identical successes and one
documented non-success, without a new100 score. All49 hashes match.

Original main CI38662802341 is terminal and failed overall. Its Word binding
job passed all778 cases in5982.92 seconds. Dot-only logs isolate a slow parity
block but do not establish individual case timings. This is evidence for
reviewing dev-profile optimization, not a measured speedup or a corrective
hosted pass. Original failed jobs remain retained.

The original corpus-in-CI criterion for Issues314/316 and PR333 is still
undelivered. F-X227 is approved for implementation, including the separate native input
revision and one-job runner proposal. Actual runner admission still needs
qualification before registration or launch. Primary hosted CI and exact-source build-only
rehearsal remain due for the final corrective source. Issue360's external
skill PR37 remains open. Its owner reviewed it and asked to combine its
merge with a package pin update after our release is available. The
delivery-boundary question remains pending. No
release or contributed-record closure is claimed.

## Definition of done for this sprint

- Preserve original main CI38662802341 failures and exact Node24.11.1 reproduction.
- Pass existing native font/animation consumers and exact executable fractional release-tag controls.
- Pass the unchanged real PowerPoint Node facade under the default1MiB stack, with public APIs, opaque XML, child order and parser bounds intact.
- Pass the original public chart alias edit in debug and optimized builds, preserving validation, sibling graphs and atomic refusal.
- Complete normal scoped lifecycle, zero-finding independent microscopes, fresh integrated full verification and clean sprint review.
- Qualify actual current paired Rust/CLI/Python artifacts,39/312 installed runtime/typing, font104/native-eight riders, primary hosted CI and exact-SHA build-only rehearsal.
- Run the unchanged first-draft corpus in actual owner CI with honest per-area rates under the approved F-X227 contract.
- Close normally without changing S91's tag. Publish both prepared families only after exact-main checks, then reconcile the70 humanized comments. Discussions264 and281 remain open, and all existing S92 work remains deferred.
