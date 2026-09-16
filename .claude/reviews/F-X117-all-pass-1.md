# F-X117, all aspects, pass 1

**Reviewed**: uncommitted worker diff, 10 files and 446 changed lines
**Verdict**: 1 defect, 0 smells, 0 nitpicks

## Defects

### D1, render failures break existing public struct literals

`crates/rpptx-layout/src/lib.rs:32`

Adding `render_failures` as a required public field of `ScopedMediaIds` makes
every downstream struct literal that names the existing fields fail to compile.
The story only needs an internal way to carry render rejections into layout, so
this source compatibility break is outside the approved contract. Keep the
existing public struct shape intact and pass failures through an additive API or
an internal resolution path.

## Smells

None.

## Nitpicks

None.

## Not found

Correctness, panics, OOXML, tests, and structure produced no other findings.
The checked decoded bounds reject malformed, overflowing, encoded-over-limit,
and decoded-over-limit inputs before unbounded allocation. The named raster
regression fails against straight-alpha input and the presentation regression
proves both reported image sizes plus the visible over-limit fallback.
