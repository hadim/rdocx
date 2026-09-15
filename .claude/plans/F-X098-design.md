# F-X098, Preserve content-control type payloads

**Status**: completed
**Sprint**: S73
**Size**: M
**Depends on**: F-253

## Problem

`crates/rdocx-oxml/src/content_control.rs:169` recognizes a nonempty supported
type element and skips its subtree. `CT_SdtPr::to_xml` later writes only an empty
element for the discriminator at `crates/rdocx-oxml/src/content_control.rs:261`,
discarding producer attributes and children.

## Spec reference

- `docs/hld/03-architecture.md`, "Low-level content-control traversal is recursive and ordered".
- `docs/hld/04-opc-and-packaging.md`, "Story items are projections over the existing typed and retained package sources".
- `docs/hld/10-bindings-spec.md`, "Native Word callers can also inspect content controls".

## Approach

Retain the unmodelled attributes and children of the first supported type
element with the type value observed when it was parsed. Serialization writes
those payload bytes under the fixed-prefix type element only while the public
`control_type` remains equal to the parsed value. A changed type writes the
existing canonical empty type element at the original slot. Duplicate type
elements stay preserved as raw children under the existing first-modeled rule.

## Rejected alternatives

- Treating every type element as raw would remove the existing typed
  discriminator.
- Modeling each extension payload would expand scope and parse content the
  product does not edit.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| round-trip | `content_control_type_payload_round_trips_until_type_changes` | Supported nonempty type elements retain exact payload until the discriminator changes, then serialize one canonical replacement in the same slot. |
| regression | Existing content-control grammar tests | Duplicate and unsupported children remain ordered raw payloads. |

The test gate is the backlog round-trip test named above.

## HLD impact

- `docs/hld/03-architecture.md`
- `docs/hld/04-opc-and-packaging.md`
- `docs/hld/12-testing-strategy.md`
- `docs/hld/14-development-backlog.md`

## Risk routing

- Any parser or serialiser. Re-read `docs/hld/04-opc-and-packaging.md` and
  `docs/hld/06-presentationml-model.md`. Verify schema order, prefix-tolerant
  reads, fixed-prefix writes after mutation, and byte-identical raw subtree
  preservation while unchanged.

## Hash harness

Expected to be unchanged. Existing samples do not carry nonempty modeled
content-control type payloads.

## Implementation checklist

- [x] Add the failing type-payload round-trip regression.
- [x] Capture the first supported type element payload.
- [x] Reuse it only while the discriminator is unchanged.
- [x] Run focused low-level tests, hash harness, full verification, and microscope.

## Open questions

None. The user approved the proposed preservation contract.
