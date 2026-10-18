# F-X259, Repair reviewed consistency defects and strengthen acceptance gates

**Status**: approved
**Sprint**: S95.3
**Size**: M
**Depends on**: F-283, F-284, F-X254

## Problem

The independent evaluation at `.claude/reviews/S95.11-S95.2-code-quality-evaluation.md` reproduces STYLEREF evaluator/render disagreement, grammar1 empty implicit-loop failure and contradictory completed capability owners. It also identifies an overly dense XML traversal predicate. The current source is 67423b6b on sprint/s95.3. All prerequisites are done. The user explicitly requests the fixes, current contribution intake and stronger shared model instructions, while reserving remaining sprint implementation for Grok and final sprint review for Codex.

## Spec reference

- `docs/hld/08-rendering-spec.md`, Numbering-aware heading placement, defines unnumbered STYLEREF r and the distinct REF context.
- `docs/hld/03-architecture.md`, structured template evaluation, defines lexical lookup and one staged publication.
- `docs/hld/02-scope-and-non-goals.md`, Modern DOCX capability matrix, defines capability status and owner semantics.
- `docs/hld/12-testing-strategy.md`, Numbering-aware navigation regression and scalar/structural template controls, defines local regression observations.
- `.claude/WORKFLOW.md`, The design plan is a machine-consumed contract and Git workflow, defines canonical instruction sources and completion evidence.

## Approach

Put the missing STYLEREF default caller policy in existing ResolvedNumbering::numbered_reference_text. Both facade and renderer already call it. Remove the facade-only fallback. REF with absent caller retains full context.

Make grammar1 lexical resolution return the existing ResolvedValue::Deferred when an empty-loop implicit item cannot be inspected, retaining the legacy distinction between unknown item properties and known invalid outer/root containers. Use the same resolver for structural paths and scalar leaves. Keep the existing expression parser and filters. Do not introduce another scope engine or public type.

After product regressions pass, reconcile the completed bounded DOCX-050 and DOCX-051 capabilities with evidence and no active owner. Extend metadata adoption to validate the final capability-owner invariant rather than trusting a pre-completion policy result. Add a real temporary-history regression for the adoption failure.

Name local traversal cases in the existing rpptx frontier scanner without changing its predicates. Strengthen existing commands and reference skills. Add prepare-only planning and requested independent reviewer handoff semantics to run-sprint. Regenerate adapters. Keep process and behavior changes independently reviewable, with separate commits if later committed. No commit, push or full sprint completion is requested by this preparation.

The user's follow-up extends this instruction work to detailed design validation. Require pinned factual compatibility evidence, exact applicable API/numeric/schema contracts, complete decision aggregation and authentic oracle provenance. Separate provider-contract approval from actual dependency readiness and correct draft mode to always write its plan. Preserve Grok's current draft plans and decisions for explicit revision and approval.

## Consumer map

| Rule | Consumers | Shared authority | Required agreement |
|---|---|---|---|
| STYLEREF number context | facade evaluation/cache update and deterministic layout | ResolvedNumbering::numbered_reference_text | r/t/n/w with unnumbered and numbered callers |
| Deferred template lookup | scalar leaves, nested loops and if paths in both grammars | existing lexical scopes and ResolvedValue | empty item lookup defers, known invalid root paths still refuse |
| Completion ownership | capability matrix, backlog and metadata adoption | current tracked files | incomplete rows have live owners, complete rows have none |
| Workflow instructions | all implementing models and generated adapters | canonical commands/skills | same requirements and honest reviewer identity |

## Acceptance criteria

| ID | Observable criterion | Evidence |
|---|---|---|
| A1 | Multilevel r and r+t evaluate and render Clause1 and1, before/after update and reopen | new unit regression using actual field_source positioned text, n/w controls |
| A2 | Empty, single and multi-record implicit/explicit loops work under both grammars | new unit regression with exact counts/output and nested structural paths |
| A3 | Missing known root members, wrong intermediate kinds and malformed syntax still refuse atomically | companion negative controls with before/after bytes |
| A4 | Final capability records agree with completed owners | existing matrix tests and new metadata adoption regression |
| A5 | XML acceptance remains identical after naming traversal cases | existing seven fx254 frontier tests |
| A6 | Canonical instructions require measurable design, applicable API/numeric/schema contracts, pinned compatibility facts, complete decisions, authentic oracle evidence and discriminating cross-consumer/final-state checks | reviewed command/skill diff, prose and generated-adapter checks, dependency/draft-mode consistency inspection |
| A7 | Current sprint records every new issue/PR criterion and Grok/Codex handoff | CURRENT_SPRINT, HLD14 and draft contribution plans |

## Rejected alternatives

- Copying the renderer fallback repeats the policy that already diverged.
- Skipping empty-loop validation hides malformed templates and known root errors.
- Marking every capability complete solely to satisfy a test invents acceptance. Reconcile the actual bounded contracts after their defects are fixed.
- Repeating workspace/native gates per small edit adds cost without proving missing combinations.
- A new general agent framework, scope registry or generic policy engine is unnecessary.

## Test plan

**Test gate**: regression.

| Timing | Category | Test | Asserts |
|---|---|---|---|
| local scoped | regression | reviewed_styleref_relative_fields_render_the_evaluated_number | actual old RED and repaired GREEN across evaluation/layout/update/reopen |
| local scoped | regression | reviewed_template_empty_implicit_scopes_match_populated_scopes | empty/single/multi and both grammars, nested paths, source atomicity |
| local scoped | regression | metadata adoption temporary-history test | completed incomplete-owner status refuses actual metadata adoption |
| local scoped | regression | existing matrix and fx254 tests | status consistency and unchanged finite XML admission |
| local scoped | unit | existing template and numbering-resolver cases | parser/filter/ref context and refusal controls |
| final sprint | integration | verify full and approved final rider union | actual integrated workspace, installed binding, WASM, docs, package and deterministic49/pixels7 checks |

Local checks use scoped cargo check/clippy, fmt, named tests, prose and adapter checks. Preserve old-source failures in the existing progress record. External/native claims remain pending for the final union.

## HLD impact

- `docs/hld/02-scope-and-non-goals.md`
- `docs/hld/03-architecture.md`
- `docs/hld/08-rendering-spec.md`
- `docs/hld/12-testing-strategy.md`
- `docs/hld/14-development-backlog.md`

## Risk routing

Layout/output: HLD08, deterministic scoped positioned-text comparisons and final49hash/sevenpixel gate. Parser/serialization: HLD04 preservation and HLD06 sequence conventions, existing grammar/refusal tests. Process/state: metadata adoption temporary-history controls. Published provider behavior changes without a new public API, final affected package/archive and binding/WASM applicability remains in the sprint union. No new dependency, module, trait or feature flag. User explicitly authorizes normal plan/review artifacts and shared instruction updates.

## Hash harness

Expected unchanged for all49documents and sevenpixel cases. These inputs do not use the repaired opt-in multilevel STYLEREF or template combination. Unexpected deltas block final acceptance and cannot be absorbed into the refactor.

## Implementation checklist

- [x] Preserve old-source failing regressions.
- [x] Repair the shared numbering and deferred scope policies.
- [x] Reconcile capability metadata and guard adoption.
- [x] Simplify XML traversal without behavior changes.
- [x] Update shared instructions, current intake and draft contribution plans.
- [x] Execute the HLD impact list and scoped checks.
- [ ] Obtain independent feature review and final integrated acceptance.

## Open questions

None for the bounded reviewed fixes and instruction updates. Contribution API choices belong to the separate draft F-X260 through F-X262 plans. The user already requests Grok implementation and independent Codex sprint review. No remaining sprint authoring or contribution implementation starts in this preparation.
