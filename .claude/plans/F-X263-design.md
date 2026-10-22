# F-X263, Prepare Word 0.16.3 and PowerPoint 0.14.3 patch releases

**Status**: approved
**Sprint**: S95.3
**Size**: L
**Depends on**: F-X259, F-X260, F-X261, F-X262, F-285, F-286, F-287

## Problem

The user requests Rust and PyPI publication only after every sprint story and
verification is complete, retaining Word 0.16.x and PowerPoint 0.14.x. Current
carriers prepare 0.16.2 and 0.14.2. Fresh inventories of22 crate registries and
two PyPI projects select 0.16.3 and 0.14.3, absent across both exact families.
The receipt is `.claude/scratch/S95.3-patch-release-inventory.json`. Recheck
mutable registries and remote tags before publication. Earlier proposed
0.17.0/0.15.0 targets are superseded, not an approved version choice.

This is preparation, not publication. F-X263 is the eighth story added by the
explicit release request. Allow3 to 5 additional developer-days provisionally,
for34 to 49 total. This expands the normal seven-story cadence rather than
pretending release preparation was included in the earlier estimate.

## Spec reference

- `docs/hld/14-development-backlog.md`, F-X263. Test gate: release regression.
- `docs/hld/10-bindings-spec.md`, Packaging and CI.
- `docs/hld/15-build-and-toolchain.md`, Packaging, Publishing, Release process and CI job matrix.
- `.claude/commands/release.md`, Family contract, Preconditions and Final approval.
- `.claude/WORKFLOW.md`, Atomic feature rhythm and External CI source availability.

## Approach

Use the existing release machinery and exact allowlists. No new runtime API,
module, crate, flag, dependency, renderer behavior or release framework.

Set workspace.package.version to 0.16.3 and preserve ten inherited manifest
carriers. Change sixteen explicit shared/PowerPoint and binding versions to
0.14.3. The seventeenth explicit manifest is the separate rpptx-wasm 0.12.1
boundary and stays. Align 24 path dependency pins, applicable Cargo.lock
records, both Python projects, CLI/install/README current examples and current
version assertions. Keep publish=false on binding/support/WASM packages.
The initial manifest/source inventory is
`.claude/scratch/S95.3-release-carrier-intake.json`. Reconcile it with actual
source at start. External gif 0.14.2 and other dependencies are not release
carriers. Do not globally replace version strings.

The exact Word crates.io set is rdocx-opc, rdocx-oxml, rdocx-layout,
rdocx-html, rdocx-pdf, rdocx and rdocx-cli, plus PyPI rdocx from rdocx-py.
The exact shared/PowerPoint set is oxml-core, oxml-opc, oxml-media, oxml-layout,
oxml-drawing, oxml-pdf, oxml-sml, oxml-cli-support, oxml-chart, rpptx-oxml,
rpptx-chart, rpptx-layout, rpptx-render, rpptx and rpptx-cli, plus PyPI rpptx
from rpptx-py. No binding or WASM crate is uploaded to crates.io.

Add an exact target-specific release regression to the existing
scripts/test_sprint_workflow.py. Run it against pre-preparation carriers for
an actual RED. Preserve immutable previous-release notes and contributor
inventories. Update only assertions that consume current manifests/examples.
Maintain meaningful wrong-version and wrong-family negative controls.

Prepare family-only CHANGELOG sections v0.16.3 and rpptx-v0.14.3 with the
existing five required subsections. The actual prior published family tags
both resolve to 57f4d7438483e109cb8e45097f3498fe1c223b23. Verify the tag
identities before deriving the complete published-baseline-to-final inventory,
including S95.11, S95.2 and S95.3. Do not credit only the latest repair slice,
or recount outcomes already included in 0.16.2/0.14.2.

Authenticate every included issue/PR and contributor from the pinned intake
and reviewed implementation. Current intake includes Issues 387 through 392,
398 and 400, and candidate PRs 393 through 397, 399 and 401. Separate direct patch
provenance, adapted implementation and closed-but-unmerged contributions.
Include only applicable families, meaningful outcome descriptions and actual
links. Keep Issue 390's unavailable private unreduced input explicit. Preserve
Issues 264/281 and later form programme boundaries. Disclose actual API/source
compatibility changes rather than implying patch numbering guarantees every
source contract. Never claim unresolved native parity or fidelity acceptance.

Run genuine locally patched 22-package cargo publish dryrun with verification
enabled and locked resolution. Retain exact target versions, source and archive
identities, 22 Verifying records, unique members and the 10 MiB ceiling. Only
actual measurements update the existing README/archive catalog. No-verify
packaging is measurement support, not a passed publication verification.

Build fresh ordinary both-family wheels and CLIs from the final prepared
source. Authenticate METADATA, full RECORD and imported native hashes. Use
clean Python 3.9/3.12 environments for exact 42/44 priority inventories and 3.12
for strict mypy and stubtest. Real imported distributions must supply the
executed API. Keep the already qualified native Word/font inputs and pinned
viewers as independent final riders. Source changes invalidate affected prior
measurements. Do not rerun unaffected native authoring merely for version text.

After local scoped checks and a clean independent microscope, use the existing
source-availability checkpoint to push only the reviewed clean canonical
sprint ref. Leave this feature in-progress and hosted gates explicitly pending.
Dispatch wheels.yml with its build-only no-publication contract. Authenticate
actual source SHA, all 27 deciding jobs and four skipped publication jobs.
Inspect 26 unique raw payloads: 12 wheels, two sdists and 12 CLIs. Aggregated
copies do not create extra artifacts. Validate each family's exact six wheels
and one sdist with the existing python-release-artifacts contract, including
name, version, metadata and platform/ABI tags.

Install actual downloaded host wheels into fresh 3.9/3.12 environments. Run
priority inventories under both interpreters and strict typing/stub checks
under 3.12. Bind receipts to downloaded artifacts, not prior local builds.
Failed or missing hosted evidence keeps the story in-progress. No registry
upload, tag, main merge or GitHub release is part of preparation.

After actual preparation acceptance, complete ordinary feature records. Final
integrated verification and an authorized clean independent sprint review
still gate closure. The existing exhausted review bound is not reset by adding
this feature. Only close-sprint may merge/tag the sprint. Only release may
publish from the exact reviewed clean main SHA, after separate final approval
for each family, shared/PowerPoint first.

## Consumer map

| Contract | Consumers | Actual evidence owner |
|---|---|---|
| Exact version and family | Cargo manifests, path pins, lock records, Python projects, CLI and current examples | Target-specific release regression and unified-release-family checks |
| Published bytes match source |22 archives, two local wheels and both CLIs | Root-owned verified dryrun, archive inventory and imported artifact receipts |
| Hosted bytes match reviewed source | wheels.yml build-only jobs, family artifact validators and downloaded host installs | Actual run SHA, job conclusions and payload hashes |
| Accurate release claims | CHANGELOG, GitHub contribution inventory and future release body | Baseline-to-final reviewed inventory and release-notes check |
| Publication boundary | close-sprint and release commands | Completed preparation, full verification, independent review and separate final approval |

## Behavior contracts and evidence

| Surface | Required behavior | Discriminating evidence |
|---|---|---|
| Family selection | Exact 7 Word and 15 shared/PowerPoint crates, one matching Python project per tag | Both canonical family validators, wrong-family/version controls |
| Version lines |0.16.3 and 0.14.3 current carriers, separate WASM remains0.12.1 | Parsed manifest, pin, lock and packaged metadata checks |
| Historical evidence | Old release notes, benchmark versions and compatibility receipts stay immutable | Historical regression inventories and reviewed semantic diffs |
| Artifact provenance | Source, wheel, native module and downloaded host artifacts have distinct actual identities | Full RECORD checks, imported paths/hashes and actual successful child exits |
| No publication | Build-only source checkpoint and dispatch create no tag/upload/release | Four publication jobs skipped, unchanged tag and registry namespaces |
| Final approval | Separate exact reviewed clean main SHA approval per family | release.md final approval after sprint closure, never inferred from keep-going |

Public signatures, XML/numeric policies and object lifetimes are unchanged.
Those contracts remain owned by the seven provider stories. This story cannot
alter them to make a release check pass.

## Dependency readiness

F-285 is done through its actual provider prefix. The other six provider
stories remain in-progress in delivery records, reviewed in run state. Their
plans are approved. Native Q24, final qualification and the exhausted sprint
review bound remain explicit. Research and this design may proceed, but carrier
implementation waits for actual dependency completion or an approved genuine
local prefix checkpoint retaining final rider debt. Reviewed alone is not done.

## Acceptance criteria

| ID | Input | Expected observation | Evidence |
|---|---|---|---|
| A1 | Pre-preparation carriers, then prepared tree | Actual target regression RED, then exact 0.16.3/0.14.3 GREEN across families, bindings, pins and locks | Existing scripts/test_sprint_workflow.py module |
| A2 | Complete baseline-to-final contribution set | Family-only truthful notes with every included issue/PR linked and authenticated specific credit, limitations retained | release-notes --check and independently reviewed inventory |
| A3 | Final prepared source and all 22 local patches | 22 verified target-version archives, correct members/source and each below 10 MiB | Actual cargo publish dryrun and retained archive catalog |
| A4 | Fresh local wheels/CLIs | Exact installed versions, 42/44 priorities on3.9/3.12, strict 3.12 typing/stubs | Clean-environment execution and full artifact provenance |
| A5 | Reviewed clean source on GitHub | Actual build-only run at that SHA, 27 deciding jobs passed, four publication jobs skipped, 26 unique raw payloads | Hosted run/job/artifact receipts |
| A6 | Downloaded host wheels and both full family artifact sets | Exact metadata/ABI sets and host priority/typing outcomes under required interpreters | Existing validators and actual downloaded installed producers |
| A7 | Current product paint and platform graphs | Both WASM configurations and unchanged 49 hash/seven pixel baselines | Actual affected checks in integrated union |
| A8 | Preparation complete but sprint still open | No main merge, tag, registry upload or release. Pending gates remain explicit | Git/registry/workflow state and delivery reconciliation |

## Test matrix

| Case | Modes compared | Wrong implementation rejected |
|---|---|---|
| C1 | Inherited Word, explicit PowerPoint, nonpublished WASM and external dependency | Global replacement or mixed family carriers |
| C2 | Current assertions versus immutable historical notes | Editing history to satisfy a new target |
| C3 | Local build versus hosted/downloaded installed artifact | Relabeling an older wheel as a new producer |
| C4 | Exact family allowlists and duplicate aggregates | Binding/WASM publication or double-counted payloads |
| C5 | Successful wrapper versus actual child/semantic result | Exit0 with failed build, wrong metadata or unpublished deciding jobs |
| C6 | Build-only dispatch versus release tag workflow | Publication during preparation or inferred final approval |

## Test plan

**Test gate**: release regression.

Run target-specific actual RED/GREEN and complete sprint-workflow policy tests.
Run affected version-contract native checks without PyO3 test binaries, normal
scoped lint/fmt/prose/drift, actual verified 22 archives and README examples.
Qualify ordinary local and hosted downloaded wheels on the stated interpreters,
priority inventories and strict typing/stub checks. Record actual hosted SHA,
job and artifact outcomes. Consume unchanged applicable product/oracle evidence
with its real producer, refreshing invalidated measurements. Final full suites,
reporter ratios, hash/pixels, all earned riders and independent sprint review
remain the integrated union, not repeated per package or worker.

## HLD impact

- `docs/hld/10-bindings-spec.md`
- `docs/hld/15-build-and-toolchain.md`

## Risk routing

- Release scripting/version strings: release.md family contract, semantic carrier diffs, actual packages and clean full gate, separate final approval.
- Published family/public API packaging: HLD10 packaging, exact allowlists, truthful compatibility notes, verified archives and 10 MiB limit.
- PyO3/WASM: actual installed 3.9/3.12 priorities, strict 3.12 typing/stubs, both WASM configurations, no binding Rust test binaries.
- Contribution acceptance: baseline-to-final source-pinned issue/PR inventory, bounded claims and authenticated specific credit.
- Delivery metadata: rerun affected status/capability/release-policy checks after completion, retain original product producer attribution.

## Hash harness

Expected unchanged 49 semantic/render hashes and seven deterministic pixel
buffers. No baseline lease, font change or intended behavioral delta.

## Rejected alternatives

0.17.0/0.15.0 contradict the user's retained version lines.
Global replacement corrupts history and unrelated dependencies. Old wheels or
no-verify archives do not qualify current preparation. Build-only execution is
not publication. Deferring preparation completion until registry publication
creates a circular prerequisite. Pending gates are not accepted by instruction
refinement or by adding this story.

## Contract closure

The family allowlists match release.md and the existing validators. Fresh
registry receipts select both absent patches. Manifest intake distinguishes
ten inherited carriers, sixteen explicit target carriers, the separate WASM
version and 24 internal pins. Native product APIs and numerical/XML policies
remain unchanged. Every promised preparation outcome maps to A1 through A8 and
a real consumer, producer and check. Historical contracts and new current
assertions are separate. The exact five note headings remain Highlights,
Added, Fixed, Compatibility and Contributors. Provider contracts are approved,
but actual implementation readiness remains pending. No product decision
remains in this plan. Approval authorizes no carrier edit before readiness.

## Implementation checklist

- [x] Pin fresh registry inventories and current manifest carrier map.
- [x] Close the design contract against exact family/carrier inventories and existing release commands.
- [ ] Confirm all actual provider checkpoints before start.
- [ ] Establish actual target-specific RED and preserve historical controls.
- [ ] Align carriers, current assertions, family notes and HLD impact.
- [ ] Qualify local archives, wheels/CLIs, installed consumers and scoped checks.
- [ ] Obtain clean independent microscope and reviewed source-availability checkpoint.
- [ ] Qualify actual hosted build-only jobs and downloaded artifacts.
- [ ] Complete preparation records and applicable final metadata checks.

## Open questions

No new product choice is needed for version lines or families. The user already
specified both. Candidate patches come from actual registry inventories and
must be rechecked before publication. Q24 belongs to F-286 and remains pending.
The exhausted sprint-review bound needs explicit replanning before another
sprint review. Neither is answered or bypassed by this design. Q25 asks whether to permit
at most two further independent sprint-review passes after all required
checks, preserving the exhausted three and all findings. It remains pending. Registry roles
and trusted-publisher configuration remain fresh final release preflights.
