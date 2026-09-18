# F-X105, Separate slide-owned placeholders from master header flags

**Status**: completed
**Sprint**: S73
**Size**: M
**Depends on**: F-226

## Problem

`ResolveCtx::flatten` applies one `LatentPolicy` derived from layout and master
`p:hf` flags to slide, layout, and master placeholders. This removes an
occupied slide-owned slide number when the master flag is false and can expose
an occupied layout date when only `sldNum` was intended. The current HLD states
the same broad rule, so the code and specification must change together if the
oracle confirms the reported precedence.

## Spec reference

- `docs/hld/07-inheritance-and-resolution.md`, flattening order and latent placeholders.
- `docs/hld/12-testing-strategy.md`, placeholder differential testing.
- `docs/hld/13-risks-and-open-questions.md`, oracle-backed semantic decisions.
- `docs/hld/14-development-backlog.md`, F-X105.

## Approach

Record the ECMA master-scoped wording and pinned LibreOffice result as the
approved decision evidence. Treat occupied slide-owned latent placeholders as
direct slide content, outside the master and layout policy. Apply `p:hf` only
while deciding whether to inherit an occupied latent placeholder from a layout
or master. Preserve deeper-source suppression so a slide-owned placeholder
continues to replace its inherited counterpart. A later PowerPoint observation
may confirm the decision but is not required to start this story.

## Rejected alternatives

- Keep the existing rule because the code matches the HLD. The HLD is a
  decision record, not independent evidence, and the reported output conflicts
  with LibreOffice and the master-scoped wording.
- Always draw occupied layout placeholders. That reproduces the stale date
  defect and defeats latent inheritance.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| differential | `slide_owned_latent_placeholders_ignore_master_header_flags` | The three reported `p:hf` shapes match the recorded PowerPoint or approved secondary oracle decision. |
| regression | placeholder source matrix | Occupied and empty slide, layout, and master date, footer, and number placeholders obey source-specific policy. |
| regression | draw order and deduplication | Direct slide content stays in source order and suppresses the same inherited latent type once. |

The **test gate** is the differential test named in the backlog.

## HLD impact

- `docs/hld/07-inheritance-and-resolution.md`
- `docs/hld/12-testing-strategy.md`
- `docs/hld/13-risks-and-open-questions.md`
- `docs/hld/14-development-backlog.md`

## Risk routing

- Layout. Use deterministic fonts for every render and inspect the complete
  placeholder matrix before accepting a baseline change.
- External oracle comparison. Pin the oracle identity, record whether
  PowerPoint was performed, and classify any disagreement explicitly.

## Hash harness

Expected to be unchanged unless an existing deck contains direct latent
placeholders under disabled master flags. Any delta is intentional only after
the oracle decision is recorded.

## Implementation checklist

- [x] Record the standards and application-oracle decision.
- [x] Add the failing direct and inherited placeholder matrix.
- [x] Split direct-slide and inherited-latent policy in the flattener.
- [x] Update the HLD rule and run deterministic rendering, hash harness, full verification, and microscope.

## Open questions

None. The user approved the ECMA master-scoped wording and pinned LibreOffice
result as the decision boundary when PowerPoint is unavailable.
