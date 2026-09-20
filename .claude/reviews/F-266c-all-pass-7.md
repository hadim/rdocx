# F-266c, all, pass 7

**Reviewed**: the uncommitted working tree on `work/f-266c-claude` after the
pass 6 remediation, 21 tracked files, 2920 insertions and 112 deletions.
**Verdict**: 0 defects, 0 smells, 3 nitpicks

Aspects reviewed: correctness, contract, panics, ooxml, tests, structure.

This is the closing pass. The exit condition is met.

## Defects

None.

The pass 6 defect is closed. `crates/rdocx-layout/src/block.rs:391` carries the
pitch on the reflow, `crates/rdocx-layout/src/engine.rs:6959` fills it from the
same value the non-reflow path uses with the exact-rule filter, and
`crates/rdocx-layout/src/paginator.rs:2871` re-applies the snap over the
re-broken lines. The advances were re-derived rather than taken on trust: with
no float the fixture paints one line per paragraph at 44.0, which is one 36
point grid row plus 8 points of spacing, and with a floating table the first
paragraph wraps to three lines at exactly 36.0 each. An exact `w:lineRule` is
still exempt after a reflow, at 28.0 without the float and 20.0 with it. The
vertical-clearance offset derives only from the paragraph top and the wrap
geometry, never from line heights, so the snap cannot perturb it.

Both pass 6 nitpicks are closed. A duplicate `w:docGrid` now goes to the slot
after the typed element at `crates/rdocx-oxml/src/document.rs:804`, matching
`w:paperSrc`, and no line this story adds to
`docs/hld/08-rendering-spec.md` exceeds 86 columns.

## Smells

None.

## Nitpicks

- `crates/rdocx-layout/src/paginator.rs:4835` and
  `crates/rdocx-layout/src/paginator.rs:3722`, neither cache-bytes function
  counts the retained attribute strings of a key's `w:docGrid`, so an entry
  whose grid carries foreign attributes under-counts by a few dozen bytes.
- `docs/hld/02-scope-and-non-goals.md:240`, carried forward. The row is
  `complete` with every fidelity column `Y` and the owner cleared, the two
  toggle clauses are split so the accurate claim and the shortfall are stated
  separately, and `scripts/test_sprint_workflow.py` adds `266` in the same
  shape F-267, F-268 and F-270 used without weakening the guard. The `Render`
  value is the integrator's to ratify.
- `crates/rdocx/tests/integration_test.rs`, the vertical run test hard-codes
  the line start at 72 points.

## Not found

- **the cached reflow pitch**. The cache entry sizing reaches the reflow
  through `size_of_val`, so the new field is counted without a change, and the
  source rebinding does not touch it.
- **pitch against cache key**. The pitch is a pure function of the grid and the
  effective paragraph properties, and the key carries the whole paragraph and
  the grid, so a hit implies the same derived pitch. No path can disagree.
- **test vacuity**. Both `.all()` sites are guarded, the upright-stacking
  equality proves absence as well as presence, and the reflow test states
  explicitly that the float must change the geometry or the test proves
  nothing.
- **structure**. The newly `pub(crate)` tolerance sits on a private module, so
  nothing leaks from the crate, and there is still no new trait, generic,
  `Box<dyn>`, wrapper, feature flag, crate, module or file.
- **module placement**. Each new test module is the single last top-level
  `mod` in its file, running to the end of the file.
- **gates**. Formatting, workspace clippy excluding the Python crates, the hash
  harness at 49 of 49, the prose rules and the sprint workflow tests are all
  clean, and every F-266c test passes.
