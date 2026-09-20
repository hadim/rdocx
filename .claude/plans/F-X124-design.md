# F-X124, Make content cloning linear and explicit

**Status**: completed
**Sprint**: S74
**Size**: M
**Depends on**: F-X106a, F-X116

## Problem

Issue 126 measures super-quadratic Python `clone_content` growth from 50 to 400
paragraphs. Issue 132 shows that invalid calls name neither `source` nor the
integer `destination` contract. Both failures sit on the same binding path.

## Spec reference

- `docs/hld/04-opc-and-packaging.md`, transactional generic content mutation.
- `docs/hld/10-bindings-spec.md`, Python handle validation and error mapping.
- `docs/hld/12-testing-strategy.md`, regression and binding gates.
- `docs/hld/14-development-backlog.md`, "F-X124, Make content cloning linear and explicit".

## Approach

Resolve the direct source and destination locations once in the binding. Make
the native clone transaction perform one package preparation and one reopen,
without repeated whole-story projection. Retain identity freshening and
relationship checks. Give the Python parameters explicit signature names and
map invalid values to messages that name `source` or `destination`.

## Rejected alternatives

- Add a cache to every paragraph handle. The call can avoid repeated discovery.
- Skip package reopen. Atomic validation is part of the public mutation contract.
- Accept destination handles. That silently changes the shipped API.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| regression | `clone_content_scales_linearly_and_names_invalid_arguments` | Bounded growth and exact Python messages. |
| integration | direct paragraph and table clones | Content, relationships, identities, and stale handles remain correct. |
| regression | rejected clone matrix | Wrong owner, stale path, and bad destination remain atomic. |

The **test gate** is the named regression.

## HLD impact

- `docs/hld/04-opc-and-packaging.md`
- `docs/hld/10-bindings-spec.md`
- `docs/hld/12-testing-strategy.md`
- `docs/hld/14-development-backlog.md`

## Risk routing

- Public PyO3 API. Run typing, Python 3.9 and 3.12 priority suites, strict mypy,
  stubtest, workspace exclusions, and both WASM checks.
- Package mutation. Prove one successful reopen and byte-identical rollback.

## Hash harness

Expected unchanged. Clone performance and binding errors do not alter samples.

## Implementation checklist

- [x] Pin native and Python scaling before changing the path.
- [x] Remove repeated story discovery from one clone call.
- [x] Name both invalid arguments in stable Python errors.
- [x] Run focused Rust, binding, package, and hash verification gates.

## Open questions

None. The accepted signature remains `(source, destination: int)`.
