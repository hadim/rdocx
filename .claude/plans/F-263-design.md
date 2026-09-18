# F-263, Layout-backed fields and M23 corpus gate

**Status**: completed
**Sprint**: S73
**Size**: L
**Depends on**: F-241 through F-262

## Problem

`Document::update_fields` deliberately marks PAGE and NUMPAGES dirty because
field evaluation does not own pagination. The layout engine already computes
PAGE, NUMPAGES, PAGEREF, and TOC targets for rendering, but it cannot publish
those values back into field caches. The final five-document M23 gate also
cannot close until every generator uses only the public facade and all private
structural and visual evidence passes without fallback.

## Spec reference

- `docs/hld/03-architecture.md`, field evaluation, layout substitution, and staged mutation.
- `docs/hld/04-opc-and-packaging.md`, field cache and bookmark ownership.
- `docs/hld/08-rendering-spec.md`, Word page numbering, bookmark pagination, and TOC targets.
- `docs/hld/10-bindings-spec.md`, native and Python field operations.
- `docs/hld/12-testing-strategy.md`, private from-scratch DOCX conformance corpus.
- `docs/hld/13-risks-and-open-questions.md`, private corpus and cross-part invariant risks.
- `docs/hld/14-development-backlog.md`, F-263.

## Approach

Add a staged pagination-aware field update that computes one deterministic
layout snapshot, correlates each supported PAGE, NUMPAGES, PAGEREF, and TOC
cache with its story and physical page, rewrites only the typed cached result,
and clears dirty state after a successful reopen. For repeating header and
footer PAGE fields, use the value from the first physical page on which that
story variant appears, matching the issue request and documented limitation.
Expose the operation through the existing Python document owner with an owned
report and GIL release around layout. Keep pure `evaluate_fields` unchanged.

Complete `scripts/docx_authoring_conformance.py` so required-private mode
invokes all five public `Document::new()` generators, fails closed on missing
or changed private inputs, records only ignored local evidence, and checks
package, semantic, schema, render, determinism, repair, and no-fallback
criteria. Public CI retains the synthetic public-only fixture.

## Rejected alternatives

- Make ordinary field evaluation depend on layout. Evaluation remains pure and
  usable when pagination is unavailable.
- Store one PAGE result per rendered page in one header field. OOXML has one
  cache in the shared header part, so the first selected page is the only
  stable package value.
- Commit private fixtures, hashes, names, renders, or evidence. That would be
  an irreversible confidentiality failure.
- Use LibreOffice post-processing to refresh fields. The output must be owned
  and reproducible by rdocx itself.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| differential | `m23_private_from_scratch_corpus_passes_required_mode` | All five public-only generators match required private package, semantic, visual, deterministic, repair, and no-fallback evidence. |
| regression | `layout_backed_page_fields_update_cached_results` | Body, header, footer, table-cell, PAGEREF, and TOC caches receive deterministic layout values and reopen cleanly. |
| binding | pagination-aware field update | Rust and Python return the same typed report, release the GIL, and publish atomically. |
| security | private artifact scan | Tracked and staged paths reject private documents and derived identifiers without echoing them. |

The **test gate** is the differential test named in the backlog.

## HLD impact

- `docs/hld/02-scope-and-non-goals.md`
- `docs/hld/03-architecture.md`
- `docs/hld/04-opc-and-packaging.md`
- `docs/hld/08-rendering-spec.md`
- `docs/hld/10-bindings-spec.md`
- `docs/hld/12-testing-strategy.md`
- `docs/hld/13-risks-and-open-questions.md`
- `docs/hld/14-development-backlog.md`

## Risk routing

- Any parser or serialiser. Verify field run sequence, bookmark ownership,
  fixed-prefix writes, and exact unrelated cache and raw subtree preservation.
- Layout and pagination. Use deterministic fonts, one immutable layout result,
  and explicit expected field and raster deltas.
- Public API of a published crate. State the additive native and Python result,
  inspect rustdoc and API changes, and run package dry-runs and size checks.
- WASM or PyO3 bindings. Run both WASM checks, the mixed Python package,
  pytest, strict mypy, stubtest, and clean abi3 wheel installation.
- External oracle comparison. Pin Word, LibreOffice, and Poppler and keep all
  exact private identities and artifacts outside tracked state.

## Hash harness

Existing public samples should remain unchanged unless they explicitly invoke
the pagination-aware update. The five private outputs are not hash-harness
entries and their evidence remains outside the repository.

## Implementation checklist

- [x] Add failing body, related-story, table-cell, PAGEREF, and TOC cache updates.
- [x] Materialize page-dependent values from one deterministic layout snapshot.
- [x] Expose the atomic owned report through native Rust and Python.
- [x] Complete public-only and required-private conformance orchestration.
- [x] Run all five generators, private-leak scan, repair evidence, oracle comparison, hash harness, full verification, and microscope gates.

## Open questions

None.
