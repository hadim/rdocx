# F-X212, Align native accessibility with visible paragraph content

**Status**: draft
**Sprint**: S91
**Size**: M
**Depends on**: F-X196

## Problem

Native Document::audit_accessibility at crates/rdocx/src/document.rs:29098
counts plain text only, so picture paragraphs contribute to consecutive
empty warnings. PR317 adds native model visibility and fixes this audit,
beyond the already implemented read-only F-X196 CLI contract. Its immutable
head is3d056f46e97e9cc1caf2969b5096eeb6775a5fa0.

Contributor rdocx-oxml/text.rs:2316 treats every unknown raw run child as
visible except a local-name list. That is insufficient for foreign lookalikes,
properties and opaque producer extensions. Its local-name filter at11924
cannot establish qualified Word identity. Reuse accepted typed traversal and
regression scaffolding after proving those boundaries, not the blanket raw
predicate or recorded baseline.

## Spec reference

- docs/hld/14-development-backlog.md, F-X212 and PR317 reconciliation.
- docs/hld/10-bindings-spec.md, CLI validation advisory findings and native
  Word facade stability.
- docs/hld/04-opc-and-packaging.md, preserved XML and package ownership.
- docs/hld/12-testing-strategy.md, Test taxonomy and deterministic hash gate.
- docs/hld/08-rendering-spec.md, deterministic font and output contracts.
- docs/hld/15-build-and-toolchain.md, Packaging and existing archive carriers.

## Approach

Complete F-X196 before claiming this source wave. Preserve its reviewed
namespace-qualified scan, typed control admission, first expanded content
slot and recursive facade inventory. Add the native semantic predicate in
existing rdocx-oxml/src/text.rs and use it from the existing audit. No new
module, file, crate, generic parameter, trait or dependency is needed.

Typed accepted-view runs and equations are primary. Reuse concrete content
and existing accepted traversal. Text needs a non-whitespace display value.
Deleted text and comment references do not count. A soft hyphen alone does
not prove visible content. Ordinary drawings, fields with display semantics,
symbols, break/tab/special characters and note references do count.

For raw pict/object/AlternateContent and preserved Word wrappers, prove the
actual namespace and accepted physical path. Preserve inherited bindings,
alias/default/rebound cases and native content-control slot admission.
Unknown XML is retained, not assumed visible. Do not turn local-name matching
into a second namespace parser. Finish the minimal existing-helper design
against actual model prefix provenance before approving the plan.

A CT_P::has_visible_content method is useful to the actual audit consumer.
Only add a ParagraphRef projection if an actual facade/CLI consumer uses it.
Do not add an unused forwarding surface. Retain audit body inventory and
other accessibility checks. No layout, source mutation or traversal expansion
is implied by changing visibility.

Native and CLI public controls share the core semantics. Where PR317 differs
from approved F-X196, such as loose field markers, establish the display
behavior with a source-built discriminator and review an explicit small CLI
refinement under this owner. Do not replace the complete scanner with a
predicate that loses opaque/raw context. F-X203 consumes the integrated
validation contract without a duplicate audit implementation.

## Rejected alternatives

- Copy the contributor baseline without deriving accepted-source deltas.
- Reopen F-X196 and repeat its completed scanner implementation.
- Treat unknown non-whitespace XML as visible content.
- Add an unused facade forwarding method or a second generic parser.

## Test plan

**Test gate**: regression.

- Add the contributor actual-picture audit regression to the existing native
  entrypoint and run against unchanged claimed Base before production edits.
- Qualified aliases/default namespace count actual pict/object content.
  Foreign/rebound same-local-name raw content remains excluded and exact.
- Duplicate SDT slots, invalid opaque controls, deleted/moveFrom content,
  loose field/instruction markers, producer hints and soft hyphens do not
  hide true empty paragraphs. Accepted wrappers and equations do count.
- Actual pictures followed by true empty paragraphs produce correct distinct
  consecutive-empty findings, preserving every other audit finding.
- Source serializes identically before/after native audit and CLI validation.
- Run shared native/CLI discriminators and applicable changed-crate checks,
  tests, prose/drift/repository policy and verified packaging riders.
- Derive all49 hash keys locally. Revert only the audit behavior in a bounded
  proof to recover old hashes, then restore accepted source. Never record
  against system fonts or permit unrelated deltas.

## HLD impact

- docs/hld/10-bindings-spec.md
- docs/hld/12-testing-strategy.md
- docs/hld/14-development-backlog.md

State the actual native audit visibility and any reviewed CLI refinement.
Describe deterministic delta attribution, not historical change prose.

## Risk routing

- Parser/serializer: HLD04 and HLD06 conventions, qualified names, retained
  opaque source, alias/default/rebound and schema child-order controls.
- Visible behavior/hash: HLD08/HLD12, exclusive baseline owner, labelled
  intentional audit commit and independent expected-key review.
- Published native API if exposed: HLD10/HLD15, actual affected-package
  patched dry runs, membership/size checks and actual archive measurements.
- No binding API, new asset, dependency, version or structural file change.

## Hash harness

Expected intentional delta is feature_showcase:word/document.xml,
feature_showcase:pdf/pages and feature_showcase:pdf/bytes because its generated
report loses the false consecutive-empty finding. All46 other entries must
match. Confirm this exact set from accepted source before any baseline write.
Contributor hashes are evidence to investigate, not expected values to copy.
Review the generated finding count and reverse-control proof independently.

## Implementation checklist

- [ ] Complete F-X196 and approve the qualified native predicate design.
- [ ] Claim this story and observe real failing public audit regression.
- [ ] Reuse/adapt native predicate and audit wiring in existing files.
- [ ] Prove raw/typed admission and unchanged source controls.
- [ ] Review any explicit CLI semantic refinement before implementation.
- [ ] Derive and independently review exact three-key audit delta.
- [ ] Commit intentional behavior separately with reviewed expected delta.
- [ ] Run scoped/risk gates and clean independent microscope.
- [ ] Update listed HLD sections and validated worker handoff.

## Open questions

Resolve actual stored-prefix provenance for raw CT_R/CT_P fragments before
choosing minimal helpers. Establish loose field display semantics against
qualified source controls before changing the CLI contract. These are
technical evidence gaps, not additional user permission requirements.
This draft does not authorize source or baseline changes.
