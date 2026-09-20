# F-266c, all, pass 5

**Reviewed**: the uncommitted working tree on `work/f-266c-claude` after the
pass 4 remediation, 20 tracked files.
**Verdict**: 0 defects, 1 smell, 2 nitpicks

Aspects reviewed: correctness, contract, panics, ooxml, tests, structure.

Both pass 4 findings and both pass 4 nitpicks are closed.

- **D1** is closed. `crates/rdocx-layout/src/paginator.rs:4441` sets the flag
  inside the guard and only when the paragraph carried a visible revision. The
  bar counts were re-derived across three directions and six orderings: a
  rotated cell gives one bar for revised then plain, one for plain then
  revised, one for both revised and none for neither, while a horizontal cell
  gives one, one, two and none. The rotated bar spans the whole row band in
  every non-zero case.
- **S1** is closed. `docs/hld/08-rendering-spec.md:772` states the bound, why
  the column width is the wrong axis, and that the stack grows past the column
  beyond it.
- The flag is now inert for a horizontal cell, and
  `docs/hld/08-rendering-spec.md:703` separates `w:topLinePunct` out and says
  it defaults to off.

## Defects

None.

## Smells

### S1, the change-bar test pins only the rotated half of a two-sided rule
`crates/rdocx/tests/regression_test.rs:32163`

All three assertions use `tbRl`. The guard encodes two claims, that a rotated
cell draws one bar for the whole row band and that a horizontal cell draws one
for every revised paragraph, and only the first is pinned. The `!rotated ||`
term reads as redundant and is a plausible future simplification, and dropping
it leaves all three assertions at one and the suite green while every
horizontal bar after the first disappears.

## Nitpicks

- `docs/hld/08-rendering-spec.md:718` is 98 columns where the surrounding new
  prose wraps at about 78, a wrap artifact of a scripted edit. The sentence is
  complete and correct.
- `crates/rdocx-layout/src/engine.rs:8881`, "so an right-to-left run's first
  advance" should read "so a right-to-left run's first advance". It is a code
  comment, so the prose scanner does not see it.

## Not found

- **prose integrity**. `docs/hld/08-rendering-spec.md` from the run properties
  to the tables section reads end to end as coherent blocks with nothing
  mangled, duplicated or dropped. Every claim checks against the code: the
  breaker is UAX#14, none of the six toggles is consumed in `rdocx-layout`,
  one base-character advance is the em, `w:vertCompress` never widens, and
  `w:combine` returns before the `w:vert` path.
- **the horizontal cell path**. With no rotation the transposed box reproduces
  the base expressions textually, with identical association, for the vertical
  offset, the content origin, the cell geometry, the nested table origin, the
  anchored placement and the change bar. The furniture vector is provably
  empty and its append a no-op, and rotate then append then clip leaves the
  furniture inside the clip group exactly as before.
- **test vacuity**. Every `.all()` is preceded by a length or non-empty guard,
  both `.any()` uses are a presence assertion and an equality against the
  expected bool, and the in-column test carries its own painted count.
- **module placement**. Each new module is the single last top-level `mod` in
  its file, running to the end of the file.
- **gates**. Formatting, workspace clippy excluding the Python crates, the
  hash harness at 49 of 49 and the prose rules are all clean, and every
  F-266c test passes.
