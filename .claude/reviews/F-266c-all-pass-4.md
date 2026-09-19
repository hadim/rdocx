# F-266c, all, pass 4

**Reviewed**: the uncommitted working tree on `work/f-266c-claude` after the
pass 3 remediation, 20 tracked files.
**Verdict**: 1 defect, 1 smell, 2 nitpicks

Aspects reviewed: correctness, contract, panics, ooxml, tests, structure.

Every pass 3 finding was re-read against the current tree.

- **D1** is closed for every realistic input. `crates/rdocx-layout/src/table.rs:1172`
  adds `VERTICAL_AUTO_MEASURE` and the three-arm match at `table.rs:702` sends
  an auto-height rotated cell to it. Measured on a 720 twip column running
  72.0 to 108.0 points, the autofit case now paints at 94.42, the declared
  narrow grid at 99.75 and the declared `w:trHeight` at 99.75, one stacked line
  each, against 11.33 and 16.65 before the fix.
- **S1** is closed. `crates/rdocx-layout/src/paginator.rs:4329` builds the
  upright band once per cell and `paginator.rs:4416` borrows it or the cell
  geometry with no clone on either path.
- Both pass 3 nitpicks are closed.

## Defects

### D1, a revised paragraph after an unrevised one in a rotated cell loses its change bar
`crates/rdocx-layout/src/paginator.rs:4439`, with the guard at
`crates/rdocx-layout/src/paginator.rs:4429`

The flag is set unconditionally after every paragraph, while
`render_change_bar` returns early for a paragraph with no visible revision. So
it records that a paragraph was processed rather than that a bar was drawn, and
the guard then suppresses every later paragraph. Measured under the tracked
revision view with a two-paragraph cell carrying one `w:pPrChange`, a `tbRl`
cell draws one bar for revised then plain, **none** for plain then revised, and
one for both revised, while an `lrTb` cell draws one, one and two. The
de-duplication itself is correct, and the bar for the both-revised rotated cell
does span the row band.

## Smells

### S1, the auto-height measure is a bound and the bound is undocumented
`crates/rdocx-layout/src/table.rs:706`

Nothing checks that the resulting stack fits the column, so the fix moves the
threshold from text longer than the column to text longer than ten thousand
points rather than removing it. Measured on the same column, four hundred words
paint inside it, four thousand words put 2827 runs outside it and reach
negative page x, and twenty thousand words reach -505. A horizontal cell
overflows rightward, which is ordinary, while a rotated stack grows leftward
off the page edge. The row height is capped by the same constant, which becomes
visible geometry and is written down nowhere.

## Nitpicks

- `crates/rdocx-layout/src/paginator.rs:4439`, the flag is also set for a
  non-rotated cell, where the guard short-circuits before reading it.
- `docs/hld/08-rendering-spec.md:699`, `w:topLinePunct` defaults to off while
  `w:kinsoku` and `w:wordWrap` default to on, so the sentence covering all
  three at their defaults is true of the third only in the weak sense that an
  off toggle asks for nothing.

## Carried forward

The DOCX-033 evidence cell is now honest, and `complete` is defensible against
the matrix's own definition at `docs/hld/02-scope-and-non-goals.md:182`. The
`Render` column is not: the legend gives it the same meanings as the operation
columns, and the row's own clause says the default behaviour of three
properties is unapplied, which is `P`. Reaching `P` means `partial`, which the
guard requires a live non-F-266 owner for, and creating one means editing
`BACKLOG.md`, which this story forbids the worker. The integrator ratifies the
`Render` value or opens the follow-up F-ID.

## Not found

- **prose integrity**. The whole of `docs/hld/08-rendering-spec.md` from the
  run properties to the tables section reads as coherent blocks with no text
  mangled by the scripted edits and no duplicated or dropped sentence. Every
  claim checks out against the code.
- **test vacuity**. Every `.all()` in both new modules is preceded by a length
  or non-empty guard, the upright-stacking `.any()` sits inside an equality
  against the expected bool so it proves absence too, and the in-column test
  carries its own painted count.
- **module placement**. Each new module is the single last top-level `mod` in
  its file.
- **borrow and ownership**, **panics**, **ooxml child order**, **structure**
  and **cache correctness**, all unchanged from pass 3 and still correct.
- **gates**. Formatting, workspace clippy, the hash harness at 49 of 49 and the
  prose rules are clean.
