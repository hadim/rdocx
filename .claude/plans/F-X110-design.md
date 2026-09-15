# F-X110, Control field updates on document open

**Status**: completed
**Sprint**: S73
**Size**: S
**Depends on**: F-244, F-X100

## Problem

`w:updateFields` is preserved as unmodeled settings XML and the architecture
explicitly leaves it untouched. Pipelines that maintain caches cannot inspect,
set, clear, or remove the policy through Rust or Python.

## Spec reference

- `docs/hld/03-architecture.md`, field update policy.
- `docs/hld/04-opc-and-packaging.md`, settings schema order and raw preservation.
- `docs/hld/10-bindings-spec.md`, optional settings accessors.
- `docs/hld/12-testing-strategy.md`, settings and binding round trips.
- `docs/hld/14-development-backlog.md`, F-X110.

## Approach

Add an optional modeled `update_fields_on_open` value to `CT_Settings` using
the shared Word on-off parser and its schema slot. Add native
`update_fields_on_open() -> Option<bool>` and
`set_update_fields_on_open(Option<bool>) -> Result<()>`, plus matching Python
property access. `None` removes only the selected modeled occurrence. Duplicate
or malformed producer forms remain preserved and are not rewritten
ambiguously. Use the existing staged settings candidate.

## Rejected alternatives

- Return false for absence. Callers must distinguish an omitted policy from an
  explicit false value.
- Make `update_fields` change the setting. Cache computation and Word-open
  policy remain independent caller decisions.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| binding | `update_fields_on_open_is_typed_optional_and_schema_ordered` | Absent, true, false, set, clear, Rust, Python, and typing agree through reopen. |
| round-trip | aliases and duplicates | Prefix aliases parse, fixed output is ordered, and ambiguous duplicates remain raw. |
| failure | staged settings allocation | Relationship or part-name exhaustion publishes no mutation. |

The **test gate** is the binding test named in the backlog.

## HLD impact

- `docs/hld/03-architecture.md`
- `docs/hld/04-opc-and-packaging.md`
- `docs/hld/10-bindings-spec.md`
- `docs/hld/12-testing-strategy.md`
- `docs/hld/14-development-backlog.md`

## Risk routing

- Any parser or serialiser. Verify schema order, prefix-tolerant reads,
  fixed-prefix writes, duplicate handling, and exact raw-subtree preservation.
- Public API of a published crate. State the additive API, run rustdoc, inspect
  the API diff, and run package dry-runs and size checks.
- WASM or PyO3 bindings. Run both WASM checks, pytest, strict mypy, stubtest,
  and clean abi3 installation.

## Hash harness

Expected to be unchanged because no sample calls the new setting accessor.

## Implementation checklist

- [x] Add failing low-level, native, and Python optional-setting tests.
- [x] Model the setting with lossless duplicate and namespace behavior.
- [x] Add staged native and Python accessors.
- [x] Run parser, binding, API, hash harness, full verification, and microscope gates.

## Open questions

None.
