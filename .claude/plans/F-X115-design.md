# F-X115, Preserve modern comment metadata and identity

**Status**: completed
**Sprint**: S73
**Size**: M
**Depends on**: F-X109

## Problem

The modern comments part uses a non-standard content type, rdocx renumbers
comment ids during an ordinary save, and authored comments cannot carry the
optional `w:date`. A saved id can therefore address a different thread and
other consumers can ignore replies or resolved state.

## Spec reference

- `docs/hld/03-architecture.md`, comment identity and staged mutation.
- `docs/hld/04-opc-and-packaging.md`, content types and part preservation.
- `docs/hld/10-bindings-spec.md`, comment mutation inputs and snapshots.
- `docs/hld/12-testing-strategy.md`, collaboration and package gates.
- `docs/hld/14-development-backlog.md`, F-X115.

## Approach

Use the Open XML commentsExtended content type and preserve an accepted source
override on a no-op save. Retain comment ids and parent links through rdocx
save and reopen, allocating new ids from the unused nonnegative space instead
of document-order renumbering. Add an optional validated RFC 3339 date to native
and Python comment and reply creation. Default to no date for deterministic
behavior. Document that a third-party editor may independently renumber ids.

## Rejected alternatives

- Keep renumbering and document it as the only behavior. The returned mutation
  id is useful only if rdocx itself keeps it stable.
- Default dates to the wall clock. That makes identical authoring inputs
  produce different packages.
- Preserve the non-standard content type once opened. It can make supported
  thread metadata invisible to strict consumers.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| regression | `comments_keep_standard_content_type_ids_dates_and_threads` | Standard content type, stable ids, parent links, optional dates, and resolved state survive save and reopen. |
| preservation | no-op modern comments save | Accepted producer paths and unrelated sidecar XML remain unchanged. |
| binding | installed comment mutation | Python dates validate, ids remain stable, and stubs match runtime. |

The **test gate** is the regression test named in the backlog.

## HLD impact

- `docs/hld/03-architecture.md`
- `docs/hld/04-opc-and-packaging.md`
- `docs/hld/10-bindings-spec.md`
- `docs/hld/12-testing-strategy.md`
- `docs/hld/14-development-backlog.md`

## Risk routing

- Any parser or serializer. Verify content type, relationship, comment, and
  extended-comment ordering with exact unrelated XML preservation.
- Public API of a published crate. Record additive optional date inputs and run
  rustdoc, package dry-runs, and archive checks.
- WASM or PyO3 bindings. Run both WASM checks, pytest, strict mypy, stubtest,
  and a clean abi3 installation.

## Hash harness

Expected to be unchanged because samples contain no modern comments.

The dedicated SHA-bound Word comment candidate is an intentional behavioural
delta. Its SHA-256 changes from
`a5ad0e8eb2d1a676daa07431deb2a0f11ee32e8bb92d099d14d5d16d43708adb` to
`d5b38f5ebbf3279cb3b77215ba667aaa149f0ddd4d29e66e13518201acf483cf`
because `[Content_Types].xml` now carries the standard commentsExtended
content type. The candidate's document content and thread model are unchanged.

## Implementation checklist

- [x] Add failing content-type, stable-id, thread, and date regressions.
- [x] Correct modern comment content type and no-op preservation.
- [x] Preserve ids and allocate new stable identities without collision.
- [x] Add optional validated native and Python dates.
- [x] Run collaboration, package, binding, hash harness, full verification, and microscope gates.

## Open questions

None.
