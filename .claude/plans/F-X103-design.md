# F-X103, Accept standard TOC switches and report rebuild diagnostics

**Status**: completed
**Sprint**: S73
**Size**: M
**Depends on**: F-X098

## Problem

`evaluate_toc` treats the argument-free Word-default `\\z` switch as
unsupported, so `rebuild_toc` retains the stale stored result. Its report keeps
only a count, which makes the reason inaccessible through Rust and Python even
though field evaluation produced an exact message. PR 101 contributes the
focused `\\z` parser change and regression while explicitly leaving the report
diagnostic surface to this story.

## Spec reference

- `docs/hld/03-architecture.md`, field evaluation and atomic cache updates.
- `docs/hld/10-bindings-spec.md`, typed immutable Python snapshots.
- `docs/hld/12-testing-strategy.md`, TOC rebuild and binding gates.
- `docs/hld/14-development-backlog.md`, F-X103.
- GitHub PR 101, focused `\\z` acceptance commit and regression.

## Approach

Accept argument-free `\\z` as a supported no-op for paginated output while
retaining the instruction text. Change `TocRebuildReport` from a copyable count
record to an owned report with ordered `Vec<String>` diagnostics and keep
`diagnostic_count` as a derived accessor for compatibility. Accumulate exact
messages at every current count site and expose an immutable Python tuple with
matching stubs.

## Rejected alternatives

- Drop `\\z` from the saved instruction. It is valid producer intent and
  should round trip.
- Keep only the count and add logging. Library consumers need deterministic,
  programmatic diagnostics.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| binding | `word_default_toc_switch_rebuilds_and_reports_ordered_diagnostics` | Word-default TOC rebuilds with preserved control payload and exact Rust and Python diagnostics. |
| regression | malformed TOC controls | Every current diagnostic-count path supplies the corresponding ordered message. |
| typing | installed mypy and stubtest | Python diagnostics are an immutable tuple of strings. |

The **test gate** is the binding test named in the backlog.

## HLD impact

- `docs/hld/03-architecture.md`
- `docs/hld/10-bindings-spec.md`
- `docs/hld/12-testing-strategy.md`
- `docs/hld/14-development-backlog.md`

## Risk routing

- Public API of a published crate. State the additive accessor and loss of
  `Copy`, run rustdoc, inspect the API diff, and run package dry-runs and size
  checks.
- WASM or PyO3 bindings. Run both WASM checks, build the mixed package, and run
  pytest, strict mypy, stubtest, and clean abi3 wheel installation.

## Hash harness

Expected to be unchanged because the harness does not rebuild a TOC containing
`\\z`.

## Implementation checklist

- [x] Add a failing Word-default TOC reproduction.
- [x] Accept and retain argument-free `\\z`.
- [x] Collect exact ordered rebuild diagnostics in Rust.
- [x] Expose diagnostics and compatibility count through Python and stubs.
- [x] Run binding, public API, hash harness, full verification, and microscope gates.

## Open questions

None.
