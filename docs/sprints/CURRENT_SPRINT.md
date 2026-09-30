# Current Sprint, S76

**Milestone**: X, contribution intake and issue repair.

**Goal**: review and integrate the first contribution prefix from the 40 open
GitHub PRs, starting with package and CLI safety, then reconciling Word story,
identity and comparison changes. Issue acceptance remains
open until the integrated S79 evidence checks every criterion.

## Spec references

- `docs/hld/03-architecture.md`, for story ownership and comparison behavior.
- `docs/hld/04-opc-and-packaging.md`, for package fidelity, atomic writes and
  namespace preservation.
- `docs/hld/10-bindings-spec.md`, for CLI and binding contracts affected by
  the contributions.
- `docs/hld/12-testing-strategy.md`, for source-built regressions, the hash
  harness, deterministic rendering and differential evidence.
- `docs/hld/14-development-backlog.md`, for F-X137 through F-X139
  dependencies, sizes and named test gates.

## The wave

| F-ID | Title | Size | Status | Owner |
|------|-------|------|--------|-------|
| F-X137 | Package and CLI safety contribution wave | L | done | - |
| F-X138 | Word story and content contribution wave | L | done | - |
| F-X139 | Word identity and comparison contribution wave | L | done | - |

## Sequencing note

F-X137 comes first because its save and namespace changes affect every later
reproduction. F-X138 then establishes the story and run index spaces that
F-X139 compares. PR 195 precedes 202 and 210, and 202 precedes 211.
F-X140 and its exclusive hash baseline update move to S77, after the Word
prefix, so the expected delta is attributable.
The full PR inventory, overlap and issue acceptance matrix are in
`docs/sprints/SPRINT_PLAN.md`, under Contribution intake evidence.

The four original unstarted S76 stories, F-X133 and F-271 through F-273,
resume in S80, the next feature sprint after the four repair boundaries.
S75's completed records and sprint tag retain their historical IDs.

## Definition of done for this sprint

- Every S76 PR is reviewed at its incremental diff, reconciled with prior
  integration, and has passing relevant focused checks after rebasing.
- The hash harness remains unchanged. Any output delta is explained before
  integration, with rendering work and PR 188's baseline update in S77.
- The integrated S76 result passes `/verify --full` and `/sprint-review`.
  Issues remain open until their full S78 acceptance evidence exists.
