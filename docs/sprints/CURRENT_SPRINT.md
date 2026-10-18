# Current Sprint, S95.3

**Milestone**: M24, Modern DOCX authoring completeness.

**Goal**: Author the content control lifecycle before typed values and custom XML
bindings. F-285 creates, inspects, edits, moves, clones and removes controls.
F-286 and F-287 then add typed controls and custom XML stores. The original
three stories retain their 12 to 15 developer-day estimate. The user added
review repairs and the live Issue387 through Issue392 and PR393 through PR396
resolution union on 2026-10-18. The expanded seven-story estimate is 28 to 39
developer-days, not an unchanged sprint-capacity promise.

## Spec references

- `docs/hld/14-development-backlog.md`, F-285, F-286 and F-287 define the creation, typed-control and custom-XML scope, their dependencies and the round-trip or differential gates. M24's complete stability boundary remains F-310 in S99.
- `docs/hld/02-scope-and-non-goals.md`, DOCX-052, DOCX-053 and DOCX-054 name the form rows these stories close.
- `docs/hld/03-architecture.md`, content-control order, shell identity, data binding and fragment ownership define the structures public creation must reuse.
- `docs/hld/04-opc-and-packaging.md`, the document-owned custom XML graph, item properties and exact retention of unrelated store bytes govern F-287 publication.
- `docs/hld/10-bindings-spec.md`, the present content-control read and replacement surfaces define what authoring may expose and what stays internal.
- `docs/hld/12-testing-strategy.md`, round-trip and differential controls for content controls, bound custom XML and byte-identical unknown stores define the integrated observations.

## The wave

| F-ID | Title | Size | Status | Owner |
|------|-------|------|--------|-------|
| F-X259 | Repair reviewed consistency defects and strengthen acceptance gates | M | in-progress | codex |
| F-X260 | Restore live handles and bound structural edit cost | L | in-progress | grok |
| F-X261 | Resolve Word package preservation and inspection contributions | L | pending | - |
| F-X262 | Resolve PowerPoint authoring and table-render contributions | L | pending | - |
| F-285 | Content control creation and lifecycle | L | pending | - |
| F-286 | Rich, repeating, and typed content controls | L | pending | - |
| F-287 | Custom XML stores and data binding authoring | L | pending | - |

## Sequencing note

Rows are listed in intended execution order, not F-ID order. F-X259 prepares
the reviewed corrections and shared workflow now. Its final integrated checks
remain part of this sprint. Then prioritize F-X260, F-X261 and F-X262 before
new authoring. Their source and binding carriers overlap, so they stay serial:
F-X260, then F-X261, then F-X262. PR heads are candidates and have not been
merged or accepted. F-X260 must not trade owner correctness for speed.

The user requests Grok implementation through the shared commands followed by
an independent Codex sprint review. Use `/run-sprint --reviewer codex` and stop
at the review handoff if Codex cannot be invoked in that environment. The Grok
implementer cannot record its own review as the requested Codex review.

Within the authoring group, F-285 comes first. Complete and integrate F-285
through its scoped dependency-prefix checkpoint before F-286 starts. F-287
starts only after F-286 leaves the shared content-control files. The three
stories stay serial. F-X249's bounded existing-control fill repair does not
replace these authoring contracts.

## Definition of done for this sprint

- Integrate F-285 through its scoped dependency-prefix checkpoint before starting F-286 or F-287.
- F-285 public creation and mutation of block, row, cell, paragraph and inline controls survives round-trip with no raw-XML fallback.
- F-286 typed controls match the pinned Word structure and behaviour under replacement and repetition. Authentic Word provenance, version and input identity are required. Generated output is not that oracle. Missing evidence stays pending.
- F-287 bound controls resolve, update and reopen against the correct store, and unknown store content stays byte-identical.
- One integrated verification and a clean sprint review. Do not start S96. M24's complete stability gate remains F-310 in S99.
- Issues 264 and 281 stay open. No release tag is prepared by opening this sprint.


## Live contribution acceptance inventory

Read-only intake on 2026-10-18 found six new issues and four open PRs. Source
bodies and PR identities are retained locally in
`.claude/scratch/S95.3-github-intake.json` and
`.claude/scratch/S95.3-pr-intake.json`, with SHA-256
`320da690d0469135e1515db51d3855e47156120749ffd4b52b389f065e3bb8e0` and
`fd0fc0346fbf7ce2cd713fc24cd1a3051d08ef5c6264c14d8b9ad76e72154d43`.
Refresh and compare remote bodies/heads before using them. The durable contracts
below and HLD14 retain the scope if local intake files are unavailable.

A read-only refresh on 2026-10-18 found issue bodies 264, 281, 377, 381 and
387 through 392 unchanged. PR393 remains
`beb9c448d0d99636a799d95f718acccb9ea55646`. PR394, PR395 and PR396 advanced by
one commit each, and that commit only records archive measurements in README
files and `scripts/readme_doctests.py`. Their refreshed heads are
`e9f9fbe7b86388b8702af5757bdfa7e4e4fc6596`,
`5d481eb53f00926896f9de632115f4cb245fc941` and
`c71708541397f96128dea2dc8b9c1a21c7da498a`. Functional parents remain the
previously pinned heads. No issue or pull request was closed, merged or
commented on.

| Issue or PR | Owner | Acceptance and disposition |
|---|---|---|
| [388](https://github.com/tensorbee/rdocx/issues/388), [393](https://github.com/tensorbee/rdocx/pull/393) | F-X260 | Word cell property edit handles and PPT append owners, including fresh/held/nested and stale-collection controls |
| [389](https://github.com/tensorbee/rdocx/issues/389) | F-X260 | All seven structural-edit scaling rows and range-removal API. PR393 alone does not resolve this issue |
| [387](https://github.com/tensorbee/rdocx/issues/387) | F-X261 | Word custom-property iteration, items and length against actual saved/read imported metadata |
| [390](https://github.com/tensorbee/rdocx/issues/390), [396](https://github.com/tensorbee/rdocx/pull/396) | F-X261 | Six reduced observations plus explicit disposition of the unreduced control-owner diagnostic and disclosed asset reinsert regression |
| [391](https://github.com/tensorbee/rdocx/issues/391), [395](https://github.com/tensorbee/rdocx/pull/395) | F-X261 | All five source/API rows, paired text-box authority and approved public compatibility |
| [392](https://github.com/tensorbee/rdocx/issues/392), [394](https://github.com/tensorbee/rdocx/pull/394) | F-X262 | All nine authoring/render rows, native table growth and intentional paint deltas |

No external issue/PR was closed, merged or commented on by this preparation.
`/close-sprint` reconciles them only after acceptance and the integrated main
push, retaining specific contributor credit for hadim. Issues264/281 remain
open, Issue381 stays with the S96 form programme, and Issue377's benchmark
invitation retains its existing disposition without new implementation credit.

## Additional acceptance and handoff

- F-X259 must retain individual old-source RED and fixed-source GREEN cases, a reviewed scoped checkpoint and named final rider debt. Its review report remains historical rather than rewritten as proof of repair.
- Each new contribution plan must map every issue row to an observable acceptance criterion, a test and a final evidence owner. Record partial/unreduced/unavailable criteria explicitly. Do not close a whole issue from PR titles or script exit codes.
- Complete detailed prepare-mode designs for the pending F-IDs before implementation. Resolve API/compatibility questions in one consolidated round. Budget expansion and overlapping source carriers must be visible in the wave plan.
- The six designs for F-285, F-286, F-287, F-X260, F-X261 and F-X262 were approved on 2026-10-18. Slide placeholders iterate by ascending idx. Layout and master placeholders iterate in document order, and idx lookup stays a separate compatibility decision. Percentage rounding matches pinned python-pptx 1.0.2 ties-to-even and leaves unrelated truncation in place. Omitted default-paragraph keywords preserve existing values. An individual None clears one property. Clearing the native paragraph-default object preserves run defaults and unrelated opaque source. Content-control creation validates each placement against its schema. Inline controls hold inline content. Unsupported placement and type combinations refuse atomically. F-286 keeps the full differential contract, and missing authentic Word evidence stays pending. The authoring order is the F-285 prefix checkpoint, then F-286, then F-287. F-X259 stays approved and running, with its repairs and final-gate debt intact.
- Checklist reconciliation on 2026-10-18 keeps Q1 through Q19 answered. Q20, Q21 and Q22 are answered the same day. `part_names` is lexicographic. Content controls use `insert_content_control`, the five setters, `remove_content_control` and `unwrap_content_control`. An imported auto line rule reads back as a float, and assigning a `Length` writes an exact rule. F-X261 and F-285 are approved on completed dependencies. F-286 and F-287 are approved against that provider contract and stay unready until the F-285 prefix checkpoint. F-X260 stays running and F-X262 stays approved. F-286 Word differential evidence remains pending.
- Design-quality pass 1 on 2026-10-19 closes R1 through R3 in the plans. F-286 names `set_content_control_type`, `repeat_content_control` and F-285 removal, and `RepeatingSectionItem` is a payload with a parent constraint. F-287 uses item bytes plus schema URIs, and binding takes the three `CT_DataBinding` strings. F-X261 projects every stored line rule through the existing paragraph getters. F-X262 no longer cites the pre-integration F-X259 tree. Q1 through Q22 stay answered. The Word differential stays pending. F-X260 A5, HLD and microscope stay open.
- Final integrated verification includes the new regression and policy union. After final completion edits, rerun affected metadata-dependent checks before adopting evidence. Codex performs the independent final sprint review, with the actual implementing and reviewing identities recorded.
