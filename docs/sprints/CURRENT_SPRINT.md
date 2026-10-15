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
| F-X227 | Execute the unchanged first-draft compatibility corpus in owner CI | L | in-progress | codex |

## Sequencing note

All prerequisite features are already done in S91. F-X224 through F-X226 implemented in parallel in separate worktrees with disjoint source leases. F-X227 is implemented and independently reviewed, with actual external acceptance pending. ROOT owns shared records, integration and final artifact qualification. No S92 feature work or new contributor intake is included.

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

The original corpus-in-CI criterion for Issues314/316 and PR333 remains
undelivered. F-X227 is implemented and independently reviewed at
215715b3b733307832202147243738f5b7b6279d. The reviewed sprint ref is
available on GitHub solely for external acceptance. Local policy180 tests
with two platform skips and all49 hashes pass. Primary hosted CI38696954013
is running. Exact-source build-only rehearsal38696969789 passed27 jobs,
with four publication jobs intentionally skipped. Its12 wheels, two source
distributions and12 CLI archives were authenticated. Fresh installed wheels
passed156 priority checks across Python3.9/3.12 and four typing checks.

The owner-only corpus CI38702668217 passed at the same source, including
its aggregate CI gate. All100 original cases ran once with safe cleanup and
no timeout. Independent output/source qualification and separate admissions
produced95 qualified correct files, all42 hard gates and51 native clauses,
five preserved no-credit cases and no unexpected failures. All114 unaffected
saved outputs match their authenticated historical bytes. The three cases
using the revised native inputs have fresh package observations. Historical
native and app witnesses retain their original producer identities.
Three preceding assignments were refused before any corpus case ran. Their
failed runs and finite cleanup receipts remain retained. The successful
runner was removed after its single job, with API404 and reaped process groups.

Current215 integrated verification has passed all22 packaging dry runs,
formatting, Clippy,6463 Rust tests,49 hashes, prose and adapter checks.
All15 local core stages and supply-chain checks passed. Primary hosted
CI38696954013 completed with a golden-PNG failure after its full Rust,
Python, MSRV, animation and fidelity consumers passed. Only three old raster
expectations failed. They predate the approved F-X210 Georgia-to-Gelasio
change, whose12 product hashes were updated without the separate raster
manifest. Mac and Linux independently reproduce the same three new hashes.
F-X224 now includes the attributed correction, independent review and fresh
actual acceptance. No final full-gate or sprint-review completion is claimed.
Issue360's external
skill PR37 remains open. Its owner reviewed it and asked to combine its
merge with a package pin update after our release is available. The
user approved closing Issue360 after publication, counting the reviewed
submission as delivery and retaining upstream merge as an external
follow-up. Fresh PyPI management checks confirm both package owners and
trusted publishers. No release or contributed-record closure is claimed.

## Definition of done for this sprint

- Preserve original main CI38662802341 failures and exact Node24.11.1 reproduction.
- Pass existing native font/animation consumers and exact executable fractional release-tag controls.
- Pass the unchanged real PowerPoint Node facade under the default1MiB stack, with public APIs, opaque XML, child order and parser bounds intact.
- Pass the original public chart alias edit in debug and optimized builds, preserving validation, sibling graphs and atomic refusal.
- Complete normal scoped lifecycle, zero-finding independent microscopes, fresh integrated full verification and clean sprint review.
- Qualify actual current paired Rust/CLI/Python artifacts,39/312 installed runtime/typing, font104/native-eight riders, primary hosted CI and exact-SHA build-only rehearsal.
- Run the unchanged first-draft corpus in actual owner CI with honest per-area rates under the approved F-X227 contract.
- Close normally without changing S91's tag. Publish both prepared families only after exact-main checks, then reconcile the70 humanized comments. Discussions264 and281 remain open, and all existing S92 work remains deferred.
