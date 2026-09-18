# S73 sprint review, pass 8

**Reviewed**: `sprint/s73` at `8fe5b008` against
`029065ca3546559f982900260878eab4ac73f1a4`, 258 files and 34,574 changed
lines, crates: oxml-chart, oxml-cli-support, oxml-core, oxml-drawing,
oxml-layout, oxml-media, oxml-opc, oxml-pdf, oxml-sml, rdocx, rdocx-cli,
rdocx-html, rdocx-layout, rdocx-opc, rdocx-oxml, rdocx-pdf, rdocx-py,
rdocx-wasm, rpptx, rpptx-chart, rpptx-cli, rpptx-layout, rpptx-oxml,
rpptx-py, rpptx-render, rpptx-wasm
**Verdict**: 0 blocking, 0 should-fix, 2 nice-to-have

Bound extension: exact release recovery boundary. Pass 8 is required because
the immutable failed `rpptx-v0.12.0` tag forced a reviewed 0.12.1 recovery
story after pass 7. `/release` requires a clean sprint review at the new exact
head before any external mutation.

## Scope of this pass

The delta after pass 7 adds the pass 7 review record, F-X122 planning and
completion, the F-X122 microscope record, and the F-X112 recovery release
review. Production behavior remains unchanged except for CRLF-to-LF
normalization when comparing the two reviewed prose members of CLI archives.
The complete shared and PowerPoint family, Python and WASM carriers, release
notes, tests, and notification mappings move coherently from the unavailable
0.12.0 version to 0.12.1. This pass rechecked those changes against every
cross-feature release dependency and retained pass 7's two nice-to-have
findings at their prior severity.

## Blocking

None.

## Should-fix

None.

## Nice-to-have

### N1, The F-X114 Word records measure preservation rather than Word's TOC generation
`crates/rdocx/tests/regression_test.rs:8498`

Retained from pass 7 at the same severity. No Word TOC behavior or evidence
changed in the recovery delta. The ignored capture still proves that Word
preserves the rebuilt entries, while the contracted generation behavior is
proved directly by the regression gate.

### N2, A created canonical TOC style uses an uppercase built-in name
`crates/rdocx/src/field.rs:5762`

Retained from pass 7 at the same severity. No field or style source changed.
The selector remains case-insensitive, so a repeated rebuild creates no
duplicate style and no user-visible defect was found.

## Milestone gate

The M23 gate at `docs/hld/14-development-backlog.md:2214` requires all five
private references to be generated from a blank public facade, reopen without
repair, match package semantics and reviewed visual thresholds, reproduce
identical DOCX bytes, and report no unexplained preservation-only fallback.

The gate holds. `m23_private_from_scratch_corpus_passes_required_mode` at
`crates/rdocx/tests/integration_test.rs:2768` is the required-private gate and
was recorded passing for the integrated F-263 result. The recovery delta does
not change any generator, public authoring path, corpus validator, layout or
rendering implementation, private evidence, or hash baseline. The current
review-side harness regenerated every public sample and matched all 49 entries.
This pass did not rerun the private corpus and claims no publication outcome.

## Not found

- Interaction: stable 0.14.0 packaged dependencies now require the complete
  shared 0.12.1 registry family, and the reviewed release order publishes that
  family first. No package can resolve to the failed 0.12.0 attempt.
- Release gate: the 15 incubating crates, seven stable crates, both Python
  distributions, four tag names, four note sections, CI literals, lock entries,
  examples, and 51-record notification inventory agree. All 22 Rust versions
  and both PyPI versions were absent during review, as were the four requested
  tags on origin.
- Asset validation: newline normalization is limited to README and licence
  equality. Exact archive names, member sets, executable mode, nonempty binary
  payloads, reviewed text content, and checksums remain mandatory. Mutation
  tests reject removal of either prose comparison.
- Duplication: no second release validator, version carrier, package allowlist,
  notification inventory, or recovery story was introduced.
- Layering: no `oxml-*` manifest gained an `rdocx-*` or `rpptx-*` dependency.
- Harness: `scripts/hash_baseline.json` still changes only in sprint commit
  `295672fa` for F-X102. F-X122 declares and observes no additional delta, and
  the current check reports 49 of 49 entries matching.
- Gate consistency: all 122 workflow tests pass with two registry-only skips
  under the reviewed environment. Prose reports zero violations and all 26
  generated agent skills are in sync.
- Documentation: HLD 03, 10, 12, 14, and 15, both release plans, the sprint
  plan, current sprint, backlog, AS_BUILT, and tracker agree on the immutable
  failure, recovery versions, publication order, and remaining F-X112 state.
- Delivery records: F-X122 is done with no owner in both trackers and has one
  AS_BUILT and one sprint-tracker entry. F-X112 remains in progress with owner
  `codex`, is reviewed in run state, and correctly has no completion entry
  before external publication and notification verification.
- Dependencies and surface: the recovery introduces no third-party dependency,
  trait, generic, crate, module, feature flag, wrapper, or public API.
- Backlog counts: 450 total stories comprise 377 done, one in progress, 68
  pending, and four archived. The X milestone has 132 stories with 127 done,
  one in progress, and four archived.
