# F-264, all, pass 2

**Reviewed**: the uncommitted working tree on `work/f-264-claude`, 18 files,
2572 insertions and 140 deletions, plus `.claude/scratch/F-264-progress.md`
**Verdict**: 0 defects, 0 smells, 5 nitpicks

## Defects

None.

## Smells

None.

### S1 is fixed
`crates/rdocx-oxml/src/paragraph_properties.rs:75`,
`crates/rdocx-oxml/src/borders.rs:36`

Both public retention fields now state that the value is serialized verbatim
and that a caller storing one owns the escaping. The border field also states
that the retained attributes are written ahead of the modeled ones, which is
what F-269 needs to know before it writes a theme reference through them.

### S2 is fixed
`crates/rdocx-oxml/src/paragraph_properties.rs:2095`

`newly_modeled_paragraph_toggles_replay_their_source_carrier` now asserts that
every entry of `PPR_MODELED_TOGGLES` resolves through `ppr_modeled_slot` to the
same slot before it exercises the carriers. A toggle added to one table and not
the other fails that assertion instead of underflowing
`occurrences[slot as usize] - 1` at parse time.

### S3 is fixed
`.claude/scratch/F-264-progress.md`

The two storage-type corrections are recorded under "Deviations" with the
measured sizes, the two tests that forced them, the bisected stack thresholds
before and after, the F-084 precedent, and the semver statement. The entry is
ready for AS_BUILT at integration.

## Nitpicks

Unchanged from pass 1 and deliberately left. Each one matches an existing
convention in the file it lives in.

- `crates/rdocx-oxml/src/paragraph_properties.rs:795`, the four new typed
  `w:val` children and `w:framePr` are typed from the `Event::Empty` arm only,
  as `w:shd` already is.
- `crates/rdocx-oxml/src/paragraph_properties.rs:805`, `w:divId` fails the
  document open on a malformed value, as `w:outlineLvl` does.
- `crates/rdocx/src/paragraph.rs:1779`, `set_border_value` writes `space: 1`,
  the same replacement semantics `set_border_all` has.
- `crates/rdocx/src/paragraph.rs:2072`, `Paragraph::mark` materialises an empty
  `w:rPr`, so `mark()` with no setter leaves an empty `w:pPr`.
- `crates/rdocx/src/paragraph.rs:1393`, `border_all` and
  `set_border_bottom_with_space` replace a whole edge and drop the retained
  attributes, which the plan pins so no existing caller breaks.

## Re-checked after the pass 1 fixes

The three fixes are a doc comment on each of two fields, six lines of assertion
inside an existing test, and one scratch file. No behaviour changed, and the
gates were rerun rather than assumed. `cargo fmt --all --check` passes,
`cargo clippy -p rdocx-oxml -p rdocx --all-targets --all-features -- -D
warnings` is clean, `cargo test -p rdocx-oxml` is 510 passing, `cargo test -p
rdocx` is 464 plus 255 plus 492 plus 2 passing with none failing,
`python3 scripts/hash_harness.py --check` reports 49 entries match, and
`python3 scripts/prose_check.py` reports zero violations.

## Not found

- `correctness`, `contract`, `panics`, `ooxml`, `tests` and `structure` all
  produced nothing beyond the nitpicks above. Pass 1 records what each aspect
  covered, and the pass 1 fixes touched documentation, one test and one scratch
  file, so none of those aspects changed.
