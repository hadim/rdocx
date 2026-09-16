# F-X108, Replace an existing picture atomically

**Status**: completed
**Sprint**: S73
**Size**: M
**Depends on**: F-255, F-X106c

## Problem

The facade reads existing image bytes and embeds new images, but it cannot
replace the payload referenced by an existing drawing. Directly replacing the
target part can change other drawings that share it and cannot safely handle a
format or extension change.

## Spec reference

- `docs/hld/03-architecture.md`, staged package mutation and story ownership.
- `docs/hld/04-opc-and-packaging.md`, relationships and content types.
- `docs/hld/10-bindings-spec.md`, native and Python media operations.
- `docs/hld/12-testing-strategy.md`, package graph and binding gates.
- `docs/hld/14-development-backlog.md`, F-X108.

## Approach

Add `Document::replace_image_for_story(story, relationship_id, bytes) ->
Result<()>` plus `Document::replace_image(relationship_id, bytes)` for the body
and matching Python methods. Resolve the exact internal image relationship in
its owner part and use the existing byte sniffer to select the supported image
type and canonical extension. Reuse the target only when it is unshared and
its extension remains compatible. Otherwise allocate a deterministic new
media part, update the same relationship ID, install the correct content type,
and remove the old part and content-type entry only when no relationship
references it. Publish through a reopened staged candidate.

## Rejected alternatives

- Mutate the existing part unconditionally. Shared targets would change
  unrelated pictures.
- Change the drawing's `r:embed` identifier. The requested API preserves the
  drawing and relationship identity.
- Trust the filename extension. Use the existing media sniffer and explicit
  fallback rules.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| binding | `replace_image_preserves_drawings_and_story_relationship_ownership` | Body, header, and footer replacements retain drawing XML and work through Rust and Python. |
| regression | shared target copy-on-write | Replacing one shared occurrence leaves every other occurrence byte-identical. |
| round-trip | format and content type | PNG to JPEG and JPEG to PNG update target names and content types and leave no orphan. |
| failure | atomic rejection | Missing, external, non-image, malformed, or exhausted relationships publish no byte change. |

The **test gate** is the binding test named in the backlog.

## HLD impact

- `docs/hld/03-architecture.md`
- `docs/hld/04-opc-and-packaging.md`
- `docs/hld/10-bindings-spec.md`
- `docs/hld/12-testing-strategy.md`
- `docs/hld/14-development-backlog.md`

## Risk routing

- Public API of a published crate. State the additive API, run rustdoc, inspect
  the API diff, and run package dry-runs and archive-size checks.
- WASM or PyO3 bindings. Run both WASM checks, pytest, strict mypy, stubtest,
  and clean abi3 installation.

## Hash harness

Expected to be unchanged because the samples do not invoke image replacement.

## Implementation checklist

- [x] Add failing body and related-story replacement tests.
- [x] Implement checked owner and target resolution with copy-on-write.
- [x] Reconcile media names, content types, orphans, and identifiers atomically.
- [x] Bind the operation in Python and verify lifecycle behavior.
- [x] Run package, binding, API, hash harness, full verification, and microscope gates.

## Open questions

None. Shared media uses copy-on-write so relationship-local replacement cannot
surprise another drawing.
