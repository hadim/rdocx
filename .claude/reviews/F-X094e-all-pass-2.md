# F-X094e, all aspects, pass 2

**Reviewed**: Final uncommitted working tree, 15 files, 765 additions and 22 deletions
**Verdict**: 1 defect, 0 smells, 0 nitpicks

## Defects

### D1, an out-of-range slide bypasses DPI validation

`crates/rpptx/src/lib.rs:6448`

`render_export_png` returns `None` for an out-of-range slide before calling the
shared validation helper. As a result, an invalid DPI such as `NaN` or `601.0`
is accepted whenever the index is outside the presentation, even though the
design and rendering specification require both slide PNG conveniences to
enforce the finite 600 DPI ceiling. Validate DPI before the index early return
and extend the native gate to cover the combined invalid-index and invalid-DPI
case.

## Smells

None.

## Nitpicks

None.

## Not found

Comment and reply identity handling, mutation atomicity, stale-handle behavior,
snapshot ordering and immutability, Python typing, GIL release, schema child
order, unmodelled XML preservation, panic paths, and structural-rule violations
produced no additional findings.
