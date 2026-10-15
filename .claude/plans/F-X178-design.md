# F-X178, Clearable direct run formatting setters

**Status**: approved
**Sprint**: S92
**Size**: S
**Depends on**: F-X200

## Problem

Issue364 identifies missing nullable reset forms for Run setters. The remaining
native gaps are width scale and baseline position in `crates/rdocx/src/run.rs:991`
and `crates/rdocx/src/run.rs:1002`. Both currently allocate direct properties and
assign Some. Earlier formatting work already supplies the other reset forms.
The Python Font surface does not currently expose these two properties.

## Spec reference

- `docs/hld/10-bindings-spec.md`, "Python Word formatting and checked list publication", absent-source clearing and native setter delegation.
- `docs/hld/12-testing-strategy.md`, "Test taxonomy", unit and round-trip gates.
- `docs/hld/14-development-backlog.md`, F-X178.

## Approach

Add `Run::set_width_scale_value(&mut self, percent: Option<u32>)` and
`Run::set_position_value(&mut self, half_points: Option<i32>)` beside the
existing setters. Each returns without allocating properties when both the
argument and properties are absent. Otherwise assign only the selected field.
Retain the existing builders and setters, delegating setters to Some.
Do not add validation or reinterpret existing units or legal values.
This is additive native API. Preserve the established Python surface because
there is no corresponding exposed width-scale or position property to repair.

## Rejected alternatives

- Clearing with 100 percent or zero position leaves an explicit override and does not restore inheritance.
- Adding Python properties expands Issue364 beyond its existing Rust setter contract.
- A generic setter helper adds indirection for two concrete assignments.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| unit | `clearing_width_scale_and_position_does_not_create_run_properties` | Both None setters on a property-free run leave properties absent |
| unit | `clearing_width_scale_and_position_preserves_other_direct_properties` | Some values, including zero and signed position, persist in the typed fields, selected clears leave the sibling, bold false and opaque XML intact |
| round-trip | `run_width_scale_and_position_can_restore_inheritance_after_reopen` | Existing setters and new Some forms serialize correct w:w and w:position, clearing one leaves the other, clearing both retains unrelated properties and opaque XML through save/reopen |

**Test gate**: unit and round trip. Each property writes its explicit value,
removes only its own element on None, and adds no w:rPr when absent.
Retain explicit-false controls for boolean setters. Values survive save and
reopen, with all existing deterministic hashes unchanged.

Use the existing unit module in `crates/rdocx/src/run.rs` and existing
`crates/rdocx/tests/integration_test.rs`. No new module or source file.
Run cargo check -p rdocx --all-targets while iterating, then scoped verification
including cargo fmt, rdocx all-target/all-feature clippy, cargo test -p rdocx,
hash harness and repository policy checks.

## HLD impact

- `docs/hld/10-bindings-spec.md`, native nullable setter contract beside the formatting section.

## Risk routing

- Public API of a published crate: additive methods with existing signatures preserved. Run patched-local workspace cargo publish --dry-run --allow-dirty under the scoped gate and assert every archive is under 10 MiB. Follow the exact patch list in .claude/commands/verify.md.

No unit conversion, parser, serializer, binding, dependency or baseline change.
The round-trip regression still proves preservation of unrelated raw XML.

## Hash harness

Unchanged. Existing callers retain identical assignment and serialization.
New clear operations are exercised only in source-built tests.

## Implementation checklist

- [ ] Add failing unit and save/reopen tests in existing test entrypoints.
- [ ] Implement the two nullable methods and delegate existing setters.
- [ ] Prove absent-property, independent clears, explicit false, zero, negative position and opaque payload preservation.
- [ ] Pass scoped verification and additive API packaging riders.
- [ ] Obtain independent zero-defect, zero-smell microscope review.

## Open questions

None. Keep this repair on the existing native Run surface.
