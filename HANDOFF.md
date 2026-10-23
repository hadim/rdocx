# Handoff: integration snapshot of 2026-10-23 and the next rdocx-skills release (state on 2026-10-23, evening)

This orphan branch of hadim/rdocx only carries notes. It is not meant for a PR. Code lives on the branches below.

## Goal

The user's plan: carry every open pull request of theirs on tensorbee/rdocx on an integration branch of hadim/rdocx,
pin it in rdocx-skills (PR hadim/rdocx-skills#63) with the docs and tests pass, wait until **every** pull request is
final (the user said: "on attend que tout soit fini avant"), then merge #63 and release the plugin. Both the merge and
the release are asked to the user first.

## State of the integration branches (all pushed to hadim/rdocx)

| branch | head | content |
|---|---|---|
| `integration/open-prs-2026-10-23` | caf70fe4 | v0.16.3 (`a57be365`) + #409 (7df1c7e0), #410 (4070a788), #411 (3c9c296c), #412 (38fefb17), #413 (bdf22890) |
| `integration/open-prs-2026-10-23-2` | 076e7ed3 | -1 + #414 (3d3d42db, issue #402) + #415 (0fbf3e96, issue #407) |
| `integration/open-prs-2026-10-23-3` | b6196403 | -2 + #416 (48b5edea, #403 part 2, stacked on #413). **Pinned by skills PR #63** |
| `integration/open-prs-2026-10-23-4` | b2ad60dc | -3 + #416 at 768f8c10 (force-pushed over 48b5edea, only its archive re-record commit replaced, two new commits fix the nine slide setters). Not pinned yet |

Each merge commit names the PR head it took. Never rewrite or delete an integration branch: the skills lock fetches
the pinned commit from it. A new snapshot is a new `-N` branch.

Conflicts met: only archive rows (`README.md`, `crates/*/README.md`, `scripts/readme_doctests.py`: take either side;
archive rows were not re-recorded on any -10-23 snapshot, the fork runs no CI), except one real one: **#416 over #409
in `crates/oxml-opc/src/package.rs`**. #409 makes `write_to` write the entries of a new pub `zip_entries()` and drops
`write_serialized`; #416 keeps `write_serialized` and adds `reread` and `same_package_as`. Resolution: keep #409's
`write_to` and `zip_entries`, add #416's two functions unchanged (`write_serialized` has no other caller). The git
rerere cache with these resolutions stays on the old machine.

## PR heads

| PR | issue | branch on hadim/rdocx | head | state |
|---|---|---|---|---|
| #409 #410 #411 #412 | #406 #408 #404 #405 | see the table above | | final |
| #413 | #403 part 1 | `fix/403-binding-owner-inventory` | bdf22890 | final |
| #414 | #402 | `fix/402-default-line-spacing-multiple` | 3d3d42db | final |
| #415 | #407 | `fix/407-paragraph-body-cost` | 0fbf3e96 | **will move**: its extension (fast path for the `edit_cost_docx` document shape) is WIP on `wip/407-fix-pass`, not reviewed: do not integrate that branch. The new head comes from the #403/#407 session, whose notes are on `handoff/403-407-2026-10-23` |
| #416 | #403 part 2 | `fix/403-native-commit` | 768f8c10 | final, in -4 |

## rdocx-skills PR #63

Branch `try/rdocx-open-prs-2026-10-23`, head d38c52f, lock on `integration/open-prs-2026-10-23-3`. CI green on the four
platforms (534 passed; the tree is marked tested). What it holds:

- 6 gaps closed, their tests in `tests/test_docx_carried.py` (each names its PR; delete once upstream main has it):
  docx-part-xml-without-relationships (#409), docx-insert-document-duplicate-style-ids and
  docx-insert-document-new-document-refused (#410), docx-insert-document-reused-picture-swapped (#411),
  docx-comments-range-across-cells (#412), docx-default-line-spacing-multiple (#414). `test_docx_everyday.py` deleted.
- Docs: `Comment.anchor_issue` (python-api.md, cli.md; not in the docx SKILL.md, which has 4 bytes of its 16,000-byte
  budget left), `part_names`/`part_xml` with rels and content types, `insert_document` workarounds removed, line
  spacing as a number of lines (python-api.md, recipes.md).
- `test_pptx_scaling.py`: GROWING holds the nine slide transition and header/footer setters, **non-strict** xfails
  (`pytest.mark.gap(GROWS, strict=False)`, conftest reads `mark.kwargs["strict"]`), because on -3 they grew by about
  1 ms a slide, within the noise of the bound (they flipped both ways across runs). The user has not approved this
  exception yet. #416 at 768f8c10 is said to fix the nine (249 of 249 locally in the #403 session).
- `test_docx_scaling.py`: GROWING lost `Paragraph.xml` (#415); 144 calls left.
- CI fix in `plugin/scripts/rdocx_env.py`: the export gets the commit's time only over a build folder whose last build
  was this commit (`target/<platform>/built-commit`). A cache fallback restored from a build that finished after the
  new commit's committer time made cargo link stale crates (Windows, "no method reread on OpcPackage").
- Skills `CLAUDE.md` "State on 23/10/2026" describes the -3 pin.

## Next steps, in order

1. Wait for #415's new head (`gh pr view 415 -R tensorbee/rdocx --json headRefOid`) and the #403/#407 session's word
   that it is final and reviewed.
2. Integrate it. If 0fbf3e96 is an ancestor of the new head: merge it into a new `-5` branched from -4. If the branch
   was force-pushed over 0fbf3e96: build `-5` from -1 (caf70fe4) + #414 + new #415 + #416 (768f8c10), in that order,
   redoing the package.rs resolution above. Name the head in each merge message.
3. Checks on the new snapshot, own `CARGO_TARGET_DIR` per worktree:
   - `cargo clippy --workspace --all-targets --all-features -- -D warnings`
   - `pixi exec -s poppler -- env -u CONDA_PREFIX RUST_MIN_STACK=8388608 cargo test --workspace --all-features
     --exclude rdocx-py --exclude rpptx-py --no-fail-fast`: only environmental failures expected on the old Mac (19:
     corpus tests of rpptx, rpptx-oxml, rpptx-layout, rpptx-cli, oxml-chart fx215; pinned Poppler, LibreOffice and
     HarfBuzz versions). `pixi exec` sets CONDA_PREFIX, which breaks the pyo3 build scripts: hence `env -u`.
   - Python suites on a **debug** build (`maturin develop --uv -m crates/<crate>/Cargo.toml` in a scratch venv with
     maturin, pytest, lxml, pillow, python-pptx): rpptx-py `tests/test_documented_examples.py` (53 environmental
     failures: 48 fx195, LibreOffice pins 158/169/217, 2 raster pins; any `checked_commit` panic is a real drift),
     and rdocx-py, where #415's debug oracles check every splice against the full rebuild. Delete the
     `_r*.abi3.so` maturin leaves in the package folder afterwards.
4. Push the snapshot, then on the skills branch: `python3 plugin/scripts/rdocx_env.py bump <full hash>`, set the
   lock's `ref` to the branch name, `upstream` stays `https://github.com/hadim/rdocx`, rename the pin in the
   `test_docx_carried.py` docstring. Push once and let ci.yml build (ci.yml cancels a run in progress on a new push,
   so batch commits).
5. From the CI logs of the four platforms (`gh api --allow-escape-sequences repos/hadim/rdocx-skills/actions/jobs/<id>/logs`,
   count the `[XPASS(strict)]` lines and the FAILED ids per scaling file):
   - docx: remove from GROWING only calls that XPASS on **every** platform (a union of one run's XPASSes was wrong
     on 2026-10-23: borderline calls flip). Update the docx `gaps.md` row (count, which calls) and `tests/gaps.py`.
   - pptx: if the nine pass everywhere, GROWING is empty: delete the gap entry, its `gaps.md` row, the `strict` kwarg
     in conftest and its line in `tests/README.md`, and the non-strict paragraph in `test_pptx_scaling.py`.
   - Docs for #416's labelled behaviour changes: commits keep the source bytes of untouched slides; `fit_text` no
     longer fails because another slide is broken; transition and header/footer edits no longer fail because of
     duplicate owners on another slide. Mention where an agent would rely on it.
6. Update the skills `CLAUDE.md` state (pin, gaps, suite) and the PR #63 body (`gh api -X PATCH
   repos/hadim/rdocx-skills/pulls/63 -F body=@file`: `gh pr edit` failed with a GraphQL error). After the last green
   run, push nothing more: release.yml reuses the files ci.yml attested only for that exact tree.
7. Brief the user in French (what changes, gaps closed, numbers, open points), then ask, with the question tool:
   merge PR #63 (the classifier needs the explicit wording "merge la PR #63 de rdocx-skills"), and the release
   (`release.yml`, asked each time). After the release: `git sync-beliver` (mirror to beliver-bio), tell the user to
   run "Rechercher des mises à jour" for the personal marketplace on claude.ai.

## Old machine

Worktrees `~/Code/libs/rdocx/worktrees/int-1023`, `int-1023-2`, `int-1023-3`, `int-1023-4`, `skills-1023` and
`handoff-int`: everything in them is pushed. The memory of the old machine
(`~/.claude/projects/-Users-hadim-Code-libs-rdocx/memory/`, file `integration-branches-and-skills-pin.md`) has the
same story in short.
