---
description: Implement a started feature against its design plan and test stubs.
---

# /implement-feature F-XXX

Turn the failing stubs from `/start-feature` into a passing implementation,
against the design plan as the contract.

## Steps

1. **Re-read the contract.** `.claude/plans/F-XXX-design.md` and its cited
   `docs/hld/` sections. Then `.claude/scratch/F-XXX-progress.md` if resuming.
   Use the exact approved signatures, numeric conversions and XML placements.
   If source inspection contradicts a compatibility claim or exposes an invalid
   content model, stop the affected change, revise the contract and resolve
   material decisions before proceeding. Do not code around a flawed plan or
   silently weaken its acceptance gate.

   On resume, compare the local plan and applicable canonical commands/skills
   with the current canonical sprint revisions. Read the authoritative files
   from the canonical worktree or Git objects. Record their commit or content
   identity, the material differences and applicability to existing work in
   the existing progress file. A claim Base alone does not identify a later
   approved contract. An unresolved semantic difference pauses only affected
   work. Do not reset, force-checkout or rebase a dirty worker, copy sprint
   trackers into it, or edit another owner's worktree. Its owner reconciles
   source safely through the normal workflow. Reuse unaffected local evidence.

2. **Confirm the state.** The F-ID is `in-progress`, the branch is `sprint/sNN`,
   and the stubs exist and fail.

3. **Work the checklist in order.** Implement the numbered acceptance rows,
   preserving the consumer map and test matrix. Inspect every caller of shared
   policy: defaults, context, owner selection and scope must agree where the
   contract says so. Compare actual relevant outputs, including positioned
   rendering, before/after update and save/reopen. Add a missing discriminating
   control when a new interaction is found. Keep unchanged-input consumers and
   refusal/atomicity controls valid. Use scoped `cargo check` between edits.
   Batch related focused regressions sharing a test binary when practical,
   rather than compiling the same large binary separately for each added case.
   Retain individual outcomes and rerun the cases affected by each source fix.
   Run the narrowest useful test target, not the workspace. Never defer a check
   needed to resolve an actual implementation uncertainty merely to batch it.

4. **Keep the progress file current.** Update
   `.claude/scratch/F-XXX-progress.md` whenever you stop, using the shape below.
   This is what makes a handoff or a resume cheap.

5. **Stay inside the plan.** If the implementation needs something the plan did
   not anticipate:
   - A small clarification: proceed and record it under "Deviations" for the
     AS_BUILT entry.
   - A change of approach, a new dependency, or a new public type: **stop and
     revise the plan** through `/design F-XXX --revise`. Do not let the code and
     the contract diverge silently.

6. **Update the HLD before freezing.** Execute exactly the approved plan's
   `## HLD impact` list during implementation. Describe implemented intent,
   without claiming pending oracle qualification passed. Source and HLD edits
   must be present for microscope and the final sprint gate.

7. **When the implementation checklist is done**, consume existing applicable
   local observations and run only missing `/verify --scoped F-XXX` checks.
   Obtain `/microscope F-XXX --working`. Do not run a whole suite or external
   oracle here. Record the final sprint checks still due. Source changes after
   review require affected checks and a review of the new material.

## Progress file shape

```markdown
# F-XXX progress notes

## Current state
What works, what does not, what is half-done.

## Changed areas
Files touched so far.

## Last green check
The exact command that last passed, and when.

## Blockers
What is stopping progress, if anything.

## Next action
The single next thing to do. Specific enough to act on without re-deriving it.
```

## Refused situations

- **The F-ID is not `in-progress`.** Run `/start-feature` first.
- **No design plan.** Run `/design`.
- **Changing approach without revising the plan.** The plan is a contract.
- **Committing.** That is `/complete-feature`.
- **Marking a checklist item done while its test fails.**
- **Updating the hash baseline to silence a delta.** Explain it or revert it.
