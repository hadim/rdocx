# F-X094a, Expose Word collaboration and redline commands in rdocx-cli

**Status**: completed
**Sprint**: S72
**Size**: M
**Depends on**: F-148, F-150, F-234, F-235

## Problem

`rdocx-cli` exposes seven commands but none of the shipped comment,
revision-resolution, comparison, or TOC operations requested in GitHub Issue
76. Rust users can automate those workflows, while CLI users cannot.

## Spec reference

- `docs/hld/10-bindings-spec.md`, CLI and native Word facade boundaries.
- `docs/hld/12-testing-strategy.md`, compiled CLI integration contracts.
- `docs/hld/14-development-backlog.md`, F-X094a.

## Approach

Add nested `comment list|add|reply|resolve|remove`, `revision list|accept|reject`,
`compare`, and `toc rebuild` commands in the existing CLI files. Comment input
uses explicit zero-based half-open body paragraph and run coordinates.
Revision list declares `scope: main`, while resolution declares its supported
all-story scope. Every mutation requires `-o/--output`, validates in memory,
publishes through the existing staged output set, and emits a schema-1 JSON
envelope when requested. Date bounds are paired and selectors are mutually
exclusive.

## Rejected alternatives

- Expose raw XML. That bypasses the facade and duplicates validation.
- Add a new CLI module. The existing two-file command structure remains clear.
- Reuse `diff` for compare. A structural diff does not create Word revisions.
- Claim complete revision or comment authoring. F-291, F-293, and F-295 own it.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| integration | `cli_collaboration_commands_are_schema_stable_and_atomic` | Exact schema-1 comment, revision, compare, and TOC records plus no partial output on failure. |
| integration | `comment_commands_round_trip_one_resolved_thread` | Add, reply, resolve, list, remove, save, and reopen. |
| integration | `revision_filters_change_only_matching_revisions` | Id, author, and paired date filters accept or reject only matches. |
| integration | `compare_accept_and_reject_reproduce_each_input` | CLI redline reopens and native accept or reject reaches each input. |

The test gate is `cli_collaboration_commands_are_schema_stable_and_atomic`.

## HLD impact

- `docs/hld/10-bindings-spec.md`
- `docs/hld/12-testing-strategy.md`
- `docs/hld/14-development-backlog.md`

## Risk routing

- Public API of a published CLI crate. Preserve existing syntax, document every
  additive command, run its package dry-run, and enforce the archive-size gate.

## Hash harness

Expected unchanged across all 49 entries.

## Implementation checklist

- [x] Add the nested Clap graph and exact validation.
- [x] Add schema-1 serializers with explicit scope fields.
- [x] Route every mutation through staged output.
- [x] Document coordinates, filters, JSON, and output rules.
- [x] Add exact compiled-binary and atomic-failure tests.
- [x] Run the full gate and unchanged hash harness.
- [x] Update exactly the listed HLD files.

## Open questions

None.
