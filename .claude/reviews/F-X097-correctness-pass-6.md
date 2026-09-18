# F-X097, correctness, pass 6

**Reviewed**: F-X097 portions of the four-file, 630-line combined working diff
**Verdict**: 1 defect, 1 smell, 0 nitpicks

## Defects

### D1, dirty typed input can lose producer drawing bindings before comparison

`crates/rdocx/src/comparison.rs:1406`

The pre-flush namespace closure runs only when the complete typed document is
equal to the package parse. If a caller opens a producer document whose drawing
uses bindings inherited from `w:drawing`, then makes an unrelated typed edit,
the equality check is false. `prepare_staged_package` at line 1411 serializes
the dirty model before its drawing wrapper is closed. The later closure cannot
recover bindings that have already disappeared. Comparison can therefore emit
an invalid drawing or lose its typed relationship after an ordinary API edit.

## Smells

### S1, comparison publishes an internal raw XML helper from rdocx-oxml

`crates/rdocx-oxml/src/text.rs:6887`

The implementation changes `raw_with_external_bindings` from crate-private to
public solely so the facade crate can use it. `#[doc(hidden)]` does not remove
the additive public API or its compatibility obligation. The same package-side
namespace closure can remain internal to `rdocx`, which would keep F-X097 out
of the low-level public surface.

## Nitpicks

None.

## Not found

No additional correctness, contract, panic, OOXML order, test, or structure
findings were found. The complex-field owner projection covers shared physical
runs and rejects unsafe shared-owner mutation. The focused regression exercises
main and header drawing namespaces plus accept and reject paths.
