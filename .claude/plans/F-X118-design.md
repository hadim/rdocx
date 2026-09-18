# F-X118, Make notes rendering and replacement safe

**Status**: completed
**Sprint**: S73
**Size**: L
**Depends on**: F-X105, F-X111

## Problem

Notes PDF rendering rejects Google Slides exports whose notes slide omits a
reverse relationship to its owning slide even though the presentation graph
already identifies that owner. The `rpptx replace` command overwrites existing
outputs, publishes a file after zero matches, cannot enforce an expected count,
and ignores speaker notes. Notes text is also read-only in native and Python
APIs.

## Spec reference

- `docs/hld/03-architecture.md`, presentation graph ownership and staged mutation.
- `docs/hld/10-bindings-spec.md`, CLI replacement and notes mutation.
- `docs/hld/12-testing-strategy.md`, Google Slides, CLI, and binding gates.
- `docs/hld/14-development-backlog.md`, F-X118.

## Approach

Pass the already resolved owning slide into notes rendering and use it when a
reverse relationship is absent. Continue to reject conflicting or multiple
reverse owners. Add staged native notes text mutation that preserves the notes
placeholder and first effective run formatting. Include notes in presentation
replacement counts. Route the CLI through the shared guarded-output helper,
add `--expect`, reject zero matches unless explicitly expected, and publish only
after all validation succeeds.

## Rejected alternatives

- Synthesize a reverse relationship during open. Rendering has the owner and
  should not mutate an otherwise valid producer package.
- Permit in-place overwrite through a temporary file. The shared CLI policy
  deliberately refuses existing destinations.
- Leave notes replacement as a Python-only loop. Native and CLI counts must
  agree on one atomic operation.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| regression | `notes_render_without_a_reverse_slide_relationship` | Control and Google-style graphs produce equivalent notes pages, while conflicting owners fail closed. |
| CLI | `rpptx_replace_is_guarded_counted_and_includes_notes` | Existing output, input equality, zero matches, expected counts, notes text, and atomic publication match the contract. |
| binding | notes mutation | Native and Python notes edits preserve placeholder identity and first-run formatting through reopen. |

The **test gate** is the CLI test named in the backlog.

## HLD impact

- `docs/hld/03-architecture.md`
- `docs/hld/10-bindings-spec.md`
- `docs/hld/12-testing-strategy.md`
- `docs/hld/14-development-backlog.md`

## Risk routing

- Any parser or serializer. Preserve valid producer relationship graphs and
  verify notes child order after mutation.
- CLI behavior. Test destination refusal, staging cleanup, exit codes, stdout,
  and exact counted replacement before publication.
- WASM or PyO3 bindings. Run both WASM checks, installed Python runtime and
  typing gates, and clean abi3 installation.

## Hash harness

Expected to be unchanged because Word samples do not exercise presentation
notes or the rpptx CLI.

## Implementation checklist

- [x] Add Google-style notes graph and guarded CLI failures.
- [x] Render missing reverse relationships from the known owning slide.
- [x] Add formatting-preserving native and Python notes mutation.
- [x] Count slide and notes replacement through one staged native operation.
- [x] Add output guard and `--expect` to the CLI.
- [x] Run notes, CLI, binding, hash harness, full verification, and microscope gates.

## Open questions

None.
