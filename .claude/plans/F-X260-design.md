# F-X260, Restore live handles and bound structural edit cost

**Status**: approved
**Sprint**: S95.3
**Size**: L
**Estimate**: 4 to 6 developer-days, provisional
**Depends on**: F-X211c, F-X237

## Problem

Issue 388 is a live-owner regression on main `7496a81c`. Setting Word cell shading, width, vertical alignment, borders, margins, column width or grid widths drops the cell's paragraph and run owners. A fresh `cell.paragraphs[0].runs` then raises `ValueError: element has no current checked physical owner`. The same failure happens through a handle taken before the edit, including a nested table. On the PowerPoint side, a second `Paragraph.add_run` or `Paragraph.add_field` on one handle retires that handle, and `TextFrame.add_paragraph` retires earlier run handles.

The current Python publication writes the edited document back without moving owner addresses. Creating `w:tcPr` shifts the raw child index of every item in that cell. `PyDocument::adopt_property_edit` does not exist on this branch. PowerPoint append proof currently drops every paragraph and run owner whose path starts at the appended owner. That code is `appended_text_owners` in `crates/rpptx-py/src/presentation.rs`.

Issue 389 is separate. On a release build, seven operations cost more as the unrelated package grows, and the reporter fails when the large package is more than three times the small one. The operations are `add_textbox`, alt text, `left`, `fill.solid`, PowerPoint `replace_xml`, Word `replace_xml` and Word `remove_content`. The same report asks for one call that removes a range of body blocks, the counterpart of `insert_document`. There is no such method today. `Document::remove_content` removes one direct body child.

PR393 head `beb9c448d0d99636a799d95f718acccb9ea55646` is unchanged by the 2026-10-18 refresh. It is a candidate, not accepted code. Its Word half relocates owners when the candidate inventory pairs one to one with the current owners and otherwise keeps the conservative publication. Its PowerPoint half keeps earlier paragraph and run owners and marks only the appended owner's text views changed. The PR body says the Word repair still walks the whole document on each property setter. That walk is not acceptance for Issue 389.

Integrated sprint source includes F-X259 at `4879e58a`. The claim base is `c776041c`. F-X259 does not edit the Python owner files this story owns. It does edit `crates/rdocx/src/document.rs` and `crates/rpptx/src/lib.rs`, and that diff is already an ancestor of this claim.

## Spec reference

- `docs/hld/03-architecture.md`, Owned publication and causal paths.
- `docs/hld/10-bindings-spec.md`, Python handle lifetime and checked XML mutation.
- `docs/hld/12-testing-strategy.md`, Binding tests and Integrated Python compatibility qualification.
- `docs/hld/14-development-backlog.md`, F-X260.

## Approach

Reimplement the accepted owner repair on the integrated sprint source. Do not merge PR393.

For Word property edits, stage the existing table, row and cell setter on a candidate. If every current owner relocates to exactly one candidate address with the same story, item kind, path depth and cell coordinates, rewrite those addresses and keep the handles. Any other inventory change, including a real insert, removal or replacement, keeps today's conservative retirement. Fresh handles taken after the edit must see the same runs. This covers the setters named by Issue 388 and the related setters PR393 lists: shading, width, vertical alignment, borders, margins, column width, grid widths, vertical merge and grid span.

For PowerPoint append, `add_run`, `add_field` and `add_paragraph` keep earlier owners. Only the appended-to owner's cached text views retire. An already-materialised paragraph collection keeps its earlier members and is not silently refreshed. A stale handle that was already invalid stays invalid. Word's documented run retirement is not changed to match this rule.

For cost, profile the seven named operations in a release build before choosing a bounded path. The target is affected-owner work using the current publication and OPC transaction. Do not add a second owner registry. Do not skip validation, namespace custody or stale-handle checks to win the ratio. A deterministic visit or allocation observation must fail if the implementation still scans unrelated slides or body blocks.

Range removal is one half-open direct-body operation on the existing fragment transaction. `Document::remove_content_range(start: usize, end: usize) -> Result<usize>` and Python `Document.remove_content_range(start, end) -> int`. An empty range returns 0. A reversed or out-of-bounds range returns an error and does not publish. The call does not recurse into tables. Relationship cleanup follows the same owned-unreachable rule F-X261 defines for pictures, so the two stories must not invent two pruning policies. This story calls the policy. It does not own style or font preservation.

No new crate, module or file. Python changes stay in `crates/rdocx-py/src/document.rs`, `crates/rdocx-py/src/table.rs` and `crates/rpptx-py/src/presentation.rs`. The native range method lands on `Document` in the existing `document.rs`.

## Consumer map

| Rule | Entry points and outputs | Shared provider and caller policy | Lifecycle stages compared |
|---|---|---|---|
| Non-structural Word cell edits keep owners | Python cell, row and table property setters. Fresh and held paragraph and run handles | `PyDocument` owner inventory. Native table edits do not grow a second policy | In-memory handle, fresh lookup, save and reopen |
| PowerPoint append keeps earlier owners | `Paragraph.add_run`, `Paragraph.add_field`, `TextFrame.add_paragraph` | Existing generation proof in `appended_text_owners` | Held handle, earlier collection, new run identity, stale view |
| Local edits do not scan the unrelated package | The seven Issue 389 operations | Existing publication and OPC transaction | Release timing plus a visit or allocation bound |
| One range removal is one transaction | Native `Document` and Python `Document.remove_content_range` | Existing direct-body fragment boundary. Picture pruning policy shared with F-X261 | Empty, one, many, invalid, save and reopen |

## Behavior contracts and evidence

| Surface or placement | Exact behavior and boundary examples | Primary source identity and location | Local discriminator or pending evidence |
|---|---|---|---|
| Word cell, row and table property setters | A property edit that keeps the same cells rewrites owner addresses and keeps fresh and held paragraph and run handles. A real insert, removal or replacement keeps conservative retirement. Flat and nested cells use the same rule. | Issue 388 on main `7496a81c`. Current publication assigns the candidate without an owner revision. `PyDocument::adopt_property_edit` is not on this branch. | A1 and A2. Old source raises `element has no current checked physical owner`. |
| PowerPoint append | `add_run`, `add_field` and `add_paragraph` keep earlier paragraph and run handles. Only the appended owner's cached text view is retired. A list already taken from `frame.paragraphs` does not grow. A later iteration sees the new paragraph. | Issue 388. `appended_text_owners` in `crates/rpptx-py/src/presentation.rs`. Paragraph collection capture binds the text scope. | A3 and A4. |
| Seven release operations | Ratio below 3. PowerPoint small package is an empty deck. Large package is 40 slides with 40 text boxes. Operations are `add_textbox`, alt text, `left`, `fill.solid` and `replace_xml`. Word small package is 50 paragraphs. Large package is 800 paragraphs. Operations are `replace_xml` and `remove_content`. A visit counter fails when the edit reads an unrelated slide or body block. Debug timings do not pass. | Issue 389 body, the release-build table on main `7496a81c`. | A5. The ratio is final-sprint evidence. The visit counter is the local gate. |
| Direct body range removal | `Document::remove_content_range(start: usize, end: usize) -> Result<usize>` and Python `Document.remove_content_range(start, end) -> int`. Half-open over direct body children. Empty range returns 0 and does not stage. `start > end` or `end > len` returns `Error::Other("content range is reversed or out of range")` and does not publish. Python surfaces that as `RdocxError`. The call does not enter a table. Picture pruning calls the F-X261 reachability rule. | Q8. The error variant matches `Error::Other` already used by checked story edits. `try_remove_content` is `crates/rdocx/src/document.rs` around line 26748. | A6 and A7. |

## Dependency readiness

`BACKLOG.md` records F-X211c and F-X237 as `done`. Those are the formal edges. F-X259 is not a formal dependency. Its integrated head `4879e58a` is an ancestor of claim base `c776041c`. This story is implementation-ready on those edges. The file lease still places it before F-X261 and F-X262. Approval of this plan does not complete the story. The running worker `work/f-x260-grok` predates the behavior-contract revision. Its owner records that revision in the worker progress file before further implementation. Dirty worker files stay in the worker. A5, the HLD update and the microscope stay open.

## Acceptance criteria

| ID | Input or setup | Expected observation or refusal | Consumers | Local test | Final gate or pending evidence |
|---|---|---|---|---|---|
| A1 | Flat and nested Word cell. Shading, width and column width. Held run and a fresh cell taken after the edit | `runs[0].text` reads the cell text. A later run edit survives save and reopen. The old source raises the missing-owner error | Python cell handles. Native package bytes | `issue388_cell_property_edits_preserve_live_owners` | Installed `rdocx` product suite at final sprint |
| A2 | The related setters listed above, plus a setter that really replaces the cell content | Property edits keep the same owner. A genuine replacement still retires the removed owner | Python table, row and cell setters | `issue388_property_edit_owner_boundaries` | Same installed suite |
| A3 | One PowerPoint paragraph. Two `add_run` calls and one `add_field` on the held handle | The paragraph handle and the earlier run stay usable. The new run's handle addresses that run | Python paragraph and run handles | `issue388_ppt_append_owner_identity` | Installed `rpptx` product suite |
| A4 | `TextFrame.add_paragraph` while an earlier run handle and an already-materialised paragraph collection exist | Earlier owners stay live. The collection keeps its earlier members and gains no silent refresh. Callers iterate again to see the appended paragraph | Python text frame, paragraph collection, run | `issue388_ppt_append_collection_contract` | Same installed suite |
| A5 | The reporter's small and large packages for all seven named operations | Each large/small ratio is below 3 under the pinned release workload. A visit counter shows the unrelated slides or blocks were not scanned | Python Word and PowerPoint editors. Native publication | `issue389_local_edits_do_not_scale_with_unrelated_package` | Release-build samples recorded with the final sprint. Not a worker-only debug timing |
| A6 | Empty, one-item, many-item and invalid direct body ranges | The approved half-open call stages once. Invalid input returns the old package bytes. Empty removes zero items | Native `Document` and Python `Document` | `issue389_range_removal_is_one_owned_transaction` | Installed product suite and hash harness unchanged |
| A7 | A refused XML edit, a nested table, and a package with shared media | The faster path still refuses atomically and does not drop a shared image | Python checked mutation and native package graph | `issue389_edit_cost_preserves_checked_mutation_contract` | Final hash harness unchanged |

Issue 388 rows map to A1 through A4. Issue 389's seven timing rows map to A5. Its range-removal request maps to A6. A7 is the control that forbids buying A5 by skipping checks. PR393 does not satisfy A5 or A6.

## Test matrix

| Case | Modes or state combined | Expected difference or agreement | Wrong implementation rejected |
|---|---|---|---|
| C1 | Zero, one and many runs, flat and nested, held and fresh | Nested `w:tcPr` shifts indexes the same way a top-level cell does | A repair that only rewrites the top-level inventory |
| C2 | Property edit versus a real content replacement | Only the property edit keeps the owner | A blanket "never retire" change |
| C3 | PowerPoint append versus Word run retirement | Earlier PowerPoint runs survive. Word retirement outside this story stays as documented | Copying the PowerPoint rule into every Word setter |
| C4 | Small package versus large package, same edited element | Ratio below 3 and a bounded visit count | A timer-only pass that still walks every slide |
| C5 | Current source, PR393 candidate, integrated successor | Same inputs. Old evidence is not relabelled as a new run | Merging the candidate without re-running it on `67423b6b` plus F-X259 |

CLI and WASM are unaffected. They do not hold these Python owner inventories. Native Word table mutation already has its own handles and is only a byte-level control for A1 and A6.

## Rejected alternatives

- Merging PR393. It leaves the whole-document walk in place and adds no range API.
- A second owner registry. The current inventory already has the addresses.
- Disabling stale checks on large packages. That fails A7.
- Treating Issue 389 as solved because Issue 388's handles work.

## Test plan

**Test gate**: regression. Each A-ID needs an old-source semantic RED and a repaired GREEN. A missing range method is a compile failure and is recorded separately from the timing RED.

| Timing | Category | Test | Asserts |
|---|---|---|---|
| local scoped | regression | A1 through A4 | Old missing-owner and stale-append failures, then live handles |
| local scoped | regression | A5 visit counter | Unrelated slides and body blocks are not visited. The ratio is the release observation |
| local scoped | regression | A6 and A7 | One transaction, atomic refusal, shared media kept |
| final sprint | risk rider | installed `rdocx` and `rpptx` suites, release ratio samples, full hash harness | No Word hash delta. No new public surface beyond the approved range method |

Local commands are the existing Python files `crates/rdocx-py/tests/test_formatting_tables.py` and `crates/rpptx-py/tests/test_documented_examples.py`, plus a native test in the existing `rdocx` integration entrypoint. No new test binary.

## HLD impact

- `docs/hld/03-architecture.md`
- `docs/hld/10-bindings-spec.md`
- `docs/hld/12-testing-strategy.md`
- `docs/hld/14-development-backlog.md`

`docs/hld/04-opc-and-packaging.md` is not edited. Relationship pruning stays the F-X261 contract.

## Risk routing

- Contributed PR or new issue acceptance. Pin the refreshed heads. Map every Issue 388 and Issue 389 row. PR393 is not whole-issue closure.
- Public API of a published crate. Range removal is additive. State the semver impact. `cargo publish --dry-run` and the crate size check run in the final union.
- WASM or PyO3 bindings. Final `cargo check --target wasm32-unknown-unknown` for both graphs. Workspace tests keep `--exclude rdocx-py --exclude rpptx-py`.
- Shared semantic behavior with multiple consumers. The consumer map is the check. Held, fresh and reopened handles see the same text.
- Grammar, scope, compatibility mode or empty collection. C1 and A6 cover zero, one, many and the invalid range.
- Any parser or serialiser, only for the range edit. Child order and atomic reopen. No new XML vocabulary.

Layout, theme colour, unit constructors and the hash baseline are not triggered. A5 must not change paint.

## Hash harness

Unchanged. A Word or PowerPoint byte change that is not required by the owner repair stops the story. F-X262 owns any intentional PowerPoint paint delta.

## Implementation checklist

- [ ] Inspect the integrated owner providers after F-X259 lands. Re-read PR393 at the pinned head.
- [ ] Record old-source RED for A1 through A5. The release ratio is a final observation, and debug timings do not pass A5.
- [ ] Implement through the existing inventories and publication path.
- [ ] Update the HLD impact list and obtain a zero-finding microscope.
- [ ] Leave the release ratio and installed suites for the final sprint union.

## Contract closure

R4 in `.claude/reviews/S95.3-design-quality-pass-1.md` does not change A1 through A7. The running worker acknowledges the canonical plan, including the behavior contracts added after claim base `c776041c`, in its progress file before further implementation. A5, the HLD update and the microscope stay explicit and unfinished. The release ratio of 3 stays final evidence.

## Open questions

| ID | Affected F-IDs and A-IDs | Decision and recommendation | Status and answer/source |
|---|---|---|---|
| Q8 | F-X260 A6 and A7 | Half-open `remove_content_range` on direct body children. Empty returns 0. Reversed or out-of-bounds does not publish. No table recursion. Picture pruning uses the F-X261 rule. | Answered 2026-10-18 by the user. Rust `Result<usize>`, Python `int`, error `Error::Other("content range is reversed or out of range")`. |
| Q9 | F-X260 A3 and A4 | Keep earlier PowerPoint paragraph and run handles across append. An already-materialised paragraph list is not refreshed. Word run retirement outside these setters stays as documented. | Answered 2026-10-18 by the user. |
| Q10 | F-X260 A5 | All seven Issue 389 release operations keep a large-to-small ratio below 3. A visit counter fails a whole-package scan. Debug timings do not pass. Range removal is not one of the seven rows. | Answered 2026-10-18 by the user. Workload sizes are the Issue 389 release table: empty deck versus 40 by 40, and 50 versus 800 paragraphs. |

No issue or pull request closes from this plan.
