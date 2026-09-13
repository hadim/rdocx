# F-X094e, rpptx Python rendering, comments, and notes

**Status**: approved
**Sprint**: S72
**Size**: L
**Depends on**: F-136, F-217, F-226

## Problem

`rpptx-py` exposes construction, save, bytes, slides, and layouts, but none of
the native deterministic PDF or PNG rendering, speaker-note output and text,
or modern comment operations identified in GitHub Issue 76.

## Spec reference

- `docs/hld/08-rendering-spec.md`, PowerPoint and notes rendering.
- `docs/hld/10-bindings-spec.md`, Python and native PowerPoint conventions.
- `docs/hld/12-testing-strategy.md`, binding and render gates.
- `docs/hld/14-development-backlog.md`, F-217, F-226, and F-X094e.

## Approach

Add additive native deterministic one-slide and all-slide PNG conveniences over
the existing resolved layout and raster path. In existing Python files, add
frozen comment-author, comment, and reply snapshots. Expose presentation PDF,
slide PNG, notes PDF and PNG, notes text, author inspection and creation, and
current slide comment, reply, and ordered move operations. Accept native GUID
and RFC 3339 strings. Release the GIL for rendering and advance binding revision
only after successful collaboration mutation.

## Rejected alternatives

- Bind directly to private render crates. The public facade owns validation.
- Return comment dictionaries. That weakens exact typing.
- Add remove or resolve operations. No native behavior exists today.
- Create a new binding module. Existing presentation and slide files suffice.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| binding | `presentation_render_comments_and_notes_match_native_snapshots` | Slide and notes output, notes text, comment thread mutation, reopen, typing, and atomic invalid identity failure. |
| rendering | native and Python parity | Exact deterministic PDF and PNG bytes match the native facade. |
| concurrency | binding render GIL release | Another Python thread progresses during slide and notes rendering. |
| typing | installed mypy and stubtest | Frozen identities, timestamps, tuples, and optional values match runtime. |

The test gate is `presentation_render_comments_and_notes_match_native_snapshots`.

## HLD impact

- `docs/hld/08-rendering-spec.md`
- `docs/hld/10-bindings-spec.md`
- `docs/hld/12-testing-strategy.md`
- `docs/hld/14-development-backlog.md`

## Risk routing

- Layout. Use deterministic fonts and exact native versus binding output.
- WASM or PyO3 bindings. Run binding exclusions, both WASM checks, pytest,
  strict mypy, stubtest, mixed-package build, and clean abi3 installation.
- Public API of `rpptx`. Record additive impact, run the patched package dry-run,
  and enforce the archive-size gate.

## Hash harness

Expected unchanged across all 49 entries.

## Implementation checklist

- [ ] Add native deterministic slide PNG conveniences.
- [ ] Add frozen collaboration values in existing binding files.
- [ ] Expose slide, notes, and comment operations.
- [ ] Release the GIL and advance revision only on success.
- [ ] Update exports, stubs, README, and installed tests.
- [ ] Run native, render, binding, wheel, WASM, and full gates.
- [ ] Update exactly the listed HLD files.

## Open questions

None.
