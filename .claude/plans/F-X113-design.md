# F-X113, Preserve appended paragraphs in document comparison

**Status**: completed
**Sprint**: S73
**Size**: M
**Depends on**: F-X097

## Problem

`Document::compare` rejects a plain-text edit when two or more paragraphs are
appended after the original final paragraph. Rejecting the staged revisions
leaves one extra empty terminal paragraph, so the comparison self-check no
longer reconstructs the original story.

## Spec reference

- `docs/hld/03-architecture.md`, comparison staging and revision ownership.
- `docs/hld/12-testing-strategy.md`, accept and reject reconstruction gates.
- `docs/hld/14-development-backlog.md`, F-X113.

## Approach

Keep terminal paragraph-mark ownership explicit while lowering a trailing
insertion run from the comparison alignment. The original final paragraph mark
must remain the rejection boundary, while every appended paragraph belongs to
the inserted range. Preserve the existing invariant that both accepting and
rejecting the staged comparison reproduce their respective source stories.

## Rejected alternatives

- Remove the final empty paragraph after rejection. That would hide a bad
  revision shape and could delete an intentional empty paragraph.
- Weaken the reconstruction self-check. It is the gate that prevented a
  corrupt redline from being published.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| regression | `comparison_appends_multiple_terminal_paragraphs_without_residue` | One, two, and three appended paragraphs accept to the edited document and reject exactly to the original. |
| differential | insertion positions | Start, middle, and end insertions retain equivalent revision ownership without synthetic empty paragraphs. |
| preservation | mixed terminal content | Empty final paragraphs, fields, and drawings remain unchanged outside the inserted range. |

The **test gate** is the regression test named in the backlog.

## HLD impact

- `docs/hld/03-architecture.md`
- `docs/hld/12-testing-strategy.md`
- `docs/hld/14-development-backlog.md`

## Risk routing

- Comparison and revision semantics. Prove both accept and reject
  reconstruction for every new fixture and retain exact unrelated package
  parts.
- Any parser or serializer. Verify schema order and preservation of terminal
  unmodelled content.

## Hash harness

Expected to be unchanged because samples do not run document comparison.

## Implementation checklist

- [x] Add the failing two-paragraph terminal append regression.
- [x] Correct terminal paragraph-mark ownership in comparison lowering.
- [x] Cover start, middle, empty, field, and drawing boundaries.
- [x] Run comparison, preservation, hash harness, full verification, and microscope gates.

## Open questions

None.
