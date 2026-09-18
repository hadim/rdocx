# F-X108, all aspects, pass 1

**Reviewed**: working diff against `HEAD`, 13 files, 502 changed lines
**Verdict**: 1 defect, 0 smells, 0 nitpicks

## Defects

### D1, orphan cleanup can leave an authored content-type default behind

`crates/rdocx/src/document.rs:12089`

When a format change moves the last image using an authored extension, the
cleanup removes the old part and a part-specific override but never removes an
authored default for that extension. The design requires the old content-type
entry to disappear when the old part is no longer referenced. The existing
story-image cleanup already applies the necessary guard: remove a default only
when no remaining part without an override uses it, and only when the facade
authored that default. Without the same reconciliation here, replacing the
last PNG with JPEG can leave an orphan `png` default in `[Content_Types].xml`.

## Smells

None.

## Nitpicks

None.

## Not found

No additional correctness, contract, panic, OOXML, test, or structure findings.
The selected relationship identifier remains stable, shared package targets
use relationship-local copy-on-write, owner validation rejects wrong scope and
mode, unsupported signatures fail before staging, and publication follows a
successful serialized reopen. The native and Python gates exercise the public
surface and fail at the pre-feature baseline.
