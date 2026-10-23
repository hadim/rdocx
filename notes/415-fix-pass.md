# #415 (fixes #407): extension and fix pass, state at handoff (2026-10-23)

**Branches:**
- PR #415's head on the fork: `fix/407-paragraph-body-cost` = 0fbf3e96. Reviewed and pushed. It covers the issue's
  repro only.
- WIP on the fork: `wip/407-fix-pass` = b6fb9d6d, on top of:
  - c5ff353b, the squashed implementation including the extension to realistic documents;
  - c025a48f, the archive re-record;
  - 1260cd06 and b6fb9d6d, WIP of the fix pass after the extension's review.

## Extension (c5ff353b, reviewed): reach rdocx-skills' review-shaped documents

Diagnosis on `edit_cost_docx`. Reads were already direct. The 7 edits each fell back to a full inventory rebuild,
for three reasons:
- the paragraph inside a nested w15 repeatingSection has no story route, which left a `None` in `body_paragraphs`;
- the python-docx `parse_xml` snippets (`w:sdt`, `w:ins`) declare `w` again, which set `plain_local_edit_blocked`;
- an insertion before a slot that owns no routed item fell back too.

What changed:
- rdocx-oxml `paragraph_count_before(slot)`;
- the planner keeps unrouted facade paragraphs in place and handles insertion before an unowned slot;
- `plain_local_edit_is_blocked` was relaxed. That caused the blocker below.

Results on test_docx_scaling.py (N = 25 / 4N = 100), these calls now pass:
- `insert_paragraph`, `remove_content`, `insert_paragraph_before`, `remove_content_range`;
- `add_paragraph`, `insert_table`, `add_table`, `add_heading`, `add_page_break`;
- `Paragraph`, `Run`, `Table` and `Cell` `.xml`.

## Review blocker B1 and the fix pass (WIP)

**B1.** With nested `sdt` owners that replay cannot tell apart once serialized, the relaxed guard let these edits
succeed and leave a document no save can write:
- the shapes: two identical w15 repeating sections or w14 checkboxes on a root without that prefix, or a block
  `q:sdt` with `xmlns:q`=W and a `q:tag`;
- the edits: `remove_content`, `remove_content_range`, `replace_xml`.

On main they refuse atomically, or save. Design note: `.claude/plans/F-X260-design.md:447-450`.

**Fix in WIP (rdocx document.rs plus the binding):**
- `plain_local_edit_facts` puts back main's guard for replace: `Paragraph.replace_xml` keeps the staged source
  splice.
- A lazy save proof: `nested_owner_replay_facts` serializes the body once and replays the nested owners. It is cached
  in a new `nested_owner_replay: OnceLock` field, runs on first use and only when nested owners exist, and resets when
  the main part is re-sourced.
- Plain removal stays local only when every owner is a `w:sdt` and the replay succeeds.
- When the replay fails:
  - removals take the staged path and refuse atomically, as on main;
  - the binding refuses `insert_paragraph`, `insert_paragraph_before`, `add_paragraph`, `add_heading`,
    `add_page_break`, `insert_table` and `add_table` before any change, through
    `PyDocument::refuse_unsavable_body_edit` and the hidden `Document::unsavable_body_edit()`. That is stricter than
    main, which left those documents unsavable.
- m1 fixed: documents with an owner on `w:tbl` or `w:tc` take main's route again.
- m2 done: docs/hld 03 and 10 and the guard doc comment updated.
- The nit is done.
- Tests:
  - pytest `test_issue407_nested_owner_body_edits_never_publish_unsavable_bytes` (32 cases);
  - Rust `issue407_plain_removal_stays_local_only_when_nested_owners_replay`;
  - `pytest -k "issue407 or issue389"`: 59 passed.
- Numbers:
  - open is unchanged (83 ms at 100 paragraphs);
  - the lazy proof costs 5, 17 and 160 ms at 25, 100 and 1000 paragraphs, once after open or after a staged edit,
    on documents with nested owners;
  - the 13 timed calls of test_docx_scaling.py pass.
- New byte delta, `sdt_w14_checkbox`: a local removal followed by a staged replace on a root without w14 writes
  redundant namespace declarations on the replaced `w:p`. Text is equal. Not yet stated in a commit body.

## What remains on #415

1. **Decide the seed 6 gap**, and ask the user if needed. In fz2 seed 6, setting text on a paragraph inside a
   repeating section already leaves the document unsavable on main. A later plain range removal is then accepted
   where main refuses, because the cached proof was taken before. The document is unsavable on both builds, before and
   after. There is no clean invalidation point for edits inside an owner. Options: accept it and state it, or redesign
   the invalidation.
2. **Rerun on the head:**
   - fz2 (`scratch/rv407b/fz2.py`) and the extended fuzz (`scratch/rv407/diff407.py`, `scratch/fuzz407/run.sh`),
     with a fresh debug build; only the checkbox delta is expected;
   - the full `test_docx_scaling.py --runxfail`, repro407 and repro389;
   - clippy, `cargo test` for rdocx, rdocx-oxml, oxml-py-support and rdocx-py (the last two need
     `DYLD_LIBRARY_PATH` set to the Python lib dir), the full debug pytest, readme_doctests.
3. **Squash** 1260cd06 and b6fb9d6d into c5ff353b. Update its body: the B1 approach, refused versus saved cases, m1,
   m2, the checkbox delta, the open and proof costs. The draft body is `scratch/commit407.txt`.
4. **Re-record** the rdocx and rdocx-oxml archive measurements as the single last commit.
5. **Push** onto `fix/407-paragraph-body-cost` (force, since c025a48f replaced 0fbf3e96). Then:
   - update the PR body: start from `scratch/pr407-body.md`, which already has the "Context" section; add the
     extension, the B1 rule and the test_docx_scaling.py numbers;
   - send the new head to the integration session.
6. **Residual siblings to report, not fixed:**
   - `Paragraph.text` on a plain paragraph of a twin, alias or shadowed document still leaves it unsavable, as on
     main;
   - edits inside an owner (`Paragraph.add_run` inside a block control) are unsavable on both builds;
   - the Rust API's inserts are unguarded; only the binding refuses.
