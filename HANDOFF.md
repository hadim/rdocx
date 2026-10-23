# Handoff: tensorbee/rdocx #403 and #407, and the docx follow-up (state on 2026-10-23, evening)

This orphan branch of hadim/rdocx only carries notes and throwaway scripts from the previous machine. It is not
meant for a PR. Code lives on the branches listed below.

## Goal

Fix the shared causes of tensorbee/rdocx#403 (rpptx: most mutating calls of the Python binding cost a time that
grows with the deck) and #407 (rdocx: `Paragraph.xml`, `insert_paragraph` and `remove_content` cost a time
proportional to the body). Fix the cause, not each call. Keep every guarantee: atomic edits, handle validity and
retirement, stale messages, saved bytes, read results. Guard each change with a debug oracle that runs the old
full path and asserts the same result.

## State of the PRs

| PR | issue | branch (on hadim/rdocx) | head | state |
|---|---|---|---|---|
| #413 | #403 part 1 | `fix/403-binding-owner-inventory` | bdf22890 | final, reviewed. Copy on write of parts and records, owner inventory per part |
| #416 | #403 part 2, stacked on #413 | `fix/403-native-commit` | 768f8c10 | final, reviewed twice. In-memory commit, 249/249 on the pptx scaling test, 3 labelled behaviour changes |
| #415 | #407 | `fix/407-paragraph-body-cost` | 0fbf3e96 (pushed) | **in progress**: the WIP of its extension and fix pass is `wip/407-fix-pass` = b6fb9d6d. See `notes/415-fix-pass.md` |
| none | docx follow-up, PR1 of 5 | `fix/docx-edit-cost-inventory` (local only, at c025a48f, no code) | | approved, not started. See `notes/docx-edit-cost-plan.md` |

All three PR bodies have a "Context: why a change of this size" section, which the user asked for: why, how, and
that a change of this size is exceptional. Keep it in any new PR of this family. The current bodies are in
`scratch/pr1-body.md` (#413), `scratch/pr2-body.md` (#416) and `scratch/pr407-body.md` (#415, to update when pushing).

## Next steps, in order

1. **Finish #415** (`notes/415-fix-pass.md`, "What remains"):
   - decide the seed 6 gap with the user;
   - rerun the fuzzes and the gates;
   - squash, re-record, force-push onto `fix/407-paragraph-body-cost`;
   - update the PR body.

   Use a lean review on the fix pass before pushing.
2. **Tell the integration session.** It builds `integration/open-prs-2026-10-23-*` from v0.16.3 plus the open PRs
   #409 to #416, then repins rdocx-skills. It was named rdocx-03 on the old machine. It waits for #415's new head:
   ask the user where that session now runs.
3. **Docx PR1** (`notes/docx-edit-cost-plan.md`):
   - rebase `fix/docx-edit-cost-inventory` onto #415's final head;
   - implement the inventory by body block and story part, with detection by value comparison;
   - acceptance: its 42 calls flip on rdocx-skills `tests/test_docx_scaling.py --runxfail`.

   After PR1 is reviewed and #413/#416 merge, ask the user about PR2 to PR5.

## Known residuals, already stated in the PRs

- #416:
  - `slides.remove` renumbers the later slides' owners: inventory work 35 at 3 slides vs 314 at 12, about 0.3 ms
    at 20 slides;
  - a deck built in memory and never committed re-serializes the slides edited in place;
  - `commit_embedded_candidate`, signed decks and `refresh_autofit` still go through the whole package.
- #415: edits stay O(owners), about 0.2 us each (accepted by the user).

## Working conventions (the user's)

- **Chat:** French, concise, one line per step. Decisions go through the multiple-choice question tool, with the
  recommended option first and each option's consequence.
- **Repo:** `~/Code/libs/rdocx/rdocx`. `origin` = tensorbee/rdocx (read only), `fork` = hadim/rdocx (push there).
  Open PRs with `gh pr create --repo tensorbee/rdocx --base main --head hadim:<branch>`. Never touch the main
  checkout's branch.
- **Worktrees:** one git worktree per PR (the old machine used `~/Code/libs/rdocx/worktrees/<name>`), each with its
  own default `target/`. Never share a `CARGO_TARGET_DIR`.
- **Pipeline:** lean. One implementer agent runs the gates, one reviewer covers correctness and scope, and a fix
  pass happens only for a blocker or a major. Sibling gaps and review findings go into the same PR. State the agent
  count when above about 15.
- **Agent rules:** `scratch/agent-rules.md` (S there was the old scratchpad, so adapt the paths). It covers:
  - no new file or module without asking;
  - the commit style;
  - the archive re-record as the single last commit, via `scripts/readme_doctests.py`;
  - the gates;
  - the known environmental test failures;
  - the timing protocol for two agents on one Mac.
- **Large refactors:** profile first (release build with symbols, `sample` on a loop of one call), show a plan, and
  wait for the user's go.
- **Office apps:** LibreOffice headless only, never a Word or PowerPoint window.

## Rebuilding the old scratchpad on the new machine

- Python venvs: `uv venv`, then `uv pip install maturin pytest python-docx python-pptx pillow numpy`. Build a binding
  with `maturin develop --release --uv -m crates/<rdocx-py|rpptx-py>/Cargo.toml` from the worktree. Add
  `CARGO_PROFILE_RELEASE_STRIP=none CARGO_PROFILE_RELEASE_DEBUG=line-tables-only` for profiles. Delete the `.so`
  from the worktree afterwards.
- Baseline builds of main a57be365 (for the repros and fuzzes that compare against main): build main in its own
  worktree and copy `crates/<binding>/python/<pkg>` aside.
- rdocx-skills checkouts for the scaling tests:
  - `tests/test_pptx_scaling.py` lives on rdocx-skills main and on branch `gap/pptx-edit-cost-scaling` (PR #61);
  - `tests/test_docx_scaling.py` lives on rdocx-skills main (PR #63);
  - run either with `--runxfail`, one worker.
- Repros: `scratch/repro407.py` (#407), `scratch/repro389_0.py` (#389, needs rdocx and rpptx in one environment),
  and `scratch/shape_calls.py` and `scratch/package_calls.py` (#403).
- Profiling helpers: `scratch/prof403.py`, `scratch/prof407.py`, `scratch/summarize_sample.py`.
- Review probes and fuzzes:
  - `scratch/rv407/`, `scratch/rv407b/`, `scratch/review/`, `scratch/review403-probes/` and
    `scratch/review416b/`;
  - the docx profiling scripts are in `scratch/docxplan/`.
