# S73 sprint review, pass 9

**Reviewed**: `sprint/s73` at `e4bb3cff` against
`029065ca3546559f982900260878eab4ac73f1a4`, 259 files and 34,886 changed
lines, crates: oxml-chart, oxml-cli-support, oxml-core, oxml-drawing,
oxml-layout, oxml-media, oxml-opc, oxml-pdf, oxml-sml, rdocx, rdocx-cli,
rdocx-html, rdocx-layout, rdocx-opc, rdocx-oxml, rdocx-pdf, rdocx-py,
rdocx-wasm, rpptx, rpptx-chart, rpptx-cli, rpptx-layout, rpptx-oxml,
rpptx-py, rpptx-render, rpptx-wasm
**Verdict**: 0 blocking, 0 should-fix, 2 nice-to-have

Bound extension: scheduled final closure boundary. Pass 9 is required after
verified publication, notification, closure, and F-X112 delivery finalization.
Earlier global pass numbers belong to completed dependency-prefix and release
boundaries, so this pass does not extend the remediation limit for this
boundary.

## Scope of this pass

The delta after pass 8 adds its clean review record and the completed F-X112
delivery evidence. Production source, manifests, workflows, release notes, and
package contents are unchanged from reviewed release SHA `58ca5a27`. The new
ledger commit records the four verified releases, all 51 contribution comments,
the authorized record closures, current registry state, and completed tracker
status. This pass rechecked that evidence against the complete sprint contract
and retained pass 8's two nice-to-have findings at their prior severity.

## Blocking

None.

## Should-fix

None.

## Nice-to-have

### N1, The F-X114 Word records measure preservation rather than Word's TOC generation
`crates/rdocx/tests/regression_test.rs:8498`

Retained from pass 8 at the same severity. The ignored Word operation opens and
saves the already rebuilt document. It proves preservation in Word while the
contracted generation behavior is proved directly by the regression gate.

### N2, A created canonical TOC style uses an uppercase built-in name
`crates/rdocx/src/field.rs:5762`

Retained from pass 8 at the same severity. The selector is case-insensitive, so
a repeated rebuild creates no duplicate style and no user-visible defect was
found.

## Milestone gate

The M23 gate at `docs/hld/14-development-backlog.md:2214` requires all five
private references to be generated from a blank public facade, reopen without
repair, match package semantics and reviewed deterministic visual thresholds,
reproduce identical DOCX bytes, and report no unexplained preservation-only
fallback.

The gate holds. `m23_private_from_scratch_corpus_passes_required_mode` at
`crates/rdocx/tests/integration_test.rs:2768` passed for the integrated F-263
result at `a809a244` in 781.83 seconds. It ran all five pure Rust generators and
their package, semantic, schema, deterministic render, visual threshold,
repair, and no-fallback checks. No generator, authoring path, corpus validator,
layout implementation, renderer, or private evidence changed after that
recorded run. The final full verification at `e4bb3cff` regenerated every
public sample and matched all 49 hash entries.

## Not found

- Interaction: all four immutable release tags target `58ca5a27`. The 15-crate
  shared 0.12.1 family published before the seven-crate stable 0.14.0 family,
  so every registry dependency was available before its consumer.
- Release gate: workflow runs `35285117271`, `35311114834`, `35314444102`, and
  `35317913616` succeeded. The Rust and PyPI inventories, owners, CLI assets,
  wheel sets, installed runtime suites, typing, stubs, and byte-identical
  release bodies are recorded at `docs/sprints/AS_BUILT.md:15193`.
- Notifications and closure: the F-X112 ledger contains exactly 51 distinct
  issue or pull-request rows and exactly 51 comment URLs beginning at
  `docs/sprints/AS_BUILT.md:15220`. The prepared machine inventory has the same
  51 records, and the verified final state is recorded at
  `docs/sprints/AS_BUILT.md:15206`.
- Duplication: the finalization delta adds no second release validator,
  inventory, version carrier, owner record, or delivery tracker.
- Layering: no manifest changed after pass 8, and no `oxml-*` crate gained an
  `rdocx-*` or `rpptx-*` dependency anywhere in the sprint delta.
- Harness: `scripts/hash_baseline.json` still changes only in the reviewed
  F-X102 behavior commit. The final check at `e4bb3cff` reports 49 of 49
  entries matching.
- Gate consistency: format, clippy, the complete workspace tests, workflow
  tests, no-default-font tests, WASM checks, rustdoc, README examples, all 22
  package dry-runs, archive size checks, and cargo-deny pass. The Python rider
  passes both bindings on Python 3.9 and 3.12 with strict mypy 2.3.0 and
  stubtest.
- Documentation: all five HLD files named by the F-X112 plan describe current
  published state. The plan, AS_BUILT entry, sprint tracker, backlog, current
  sprint, and run state all mark F-X112 completed with no owner.
- Dependencies and surface: finalization adds no dependency, trait, generic,
  crate, module, feature flag, wrapper, or public API.
- Backlog counts: 450 total stories comprise 378 done, zero in progress, 68
  pending, and four archived. The X milestone has 132 stories with 128 done,
  zero in progress, and four archived.
