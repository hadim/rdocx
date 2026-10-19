# F-X262, Resolve PowerPoint authoring and table-render contributions

**Status**: approved
**Sprint**: S95.3
**Size**: L
**Estimate**: 4 to 6 developer-days, provisional
**Depends on**: F-X205, F-X206, F-X258

## Problem

Issue 392 reports nine authoring or render gaps on main `7496a81c`.

Placeholder iteration stops at the first missing `idx`. After idx 1 is removed, `slide.placeholders` yields `[0]` while `slide.shapes` filtered by `is_placeholder` yields `[0, 2]`. Layouts and masters have the same gap. `fill.transparency = 0.8` writes `a:alpha` `19999` because the percentage is truncated. `MSO_SHAPE.UP_ARROW` and the preset name `upArrow` are absent from the generated table. A table row whose text wraps keeps the stored `a:tr/@h`, so the text is painted over the next row. `TextFrame.clear` does not exist. Assigning empty text writes an empty run, so the next `add_run` follows that run. A table-cell run refuses `hyperlink.address`. `bullet_size` accepts only a percentage float from 0.25 to 4.0, so `Pt(17.85)` is rejected. Nothing removes the `p:style` that `add_shape` and `add_connector` write.

PR394 head `e9f9fbe7b86388b8702af5757bdfa7e4e4fc6596` is the refreshed identity. It is one archive-measurement commit ahead of functional parent `f9d165e77392b57a58223046d8cd551220c61a3a`. The extra commit edits README files and `scripts/readme_doctests.py` only. The functional diff is about twenty files, including `tools/gen-presets/generate.py`, `crates/oxml-drawing/src/preset_shape_data.rs`, `crates/rpptx-render/src/lib.rs` and `crates/rpptx/src/lib.rs`. It is a candidate. Its PowerPoint corpus and wasm gate were not run here.

F-X259 is integrated at `4879e58a`, which is an ancestor of this sprint branch. That commit edits `crates/rpptx/src/lib.rs` and rendering docs. This story still shares `docs/hld/10-bindings-spec.md` with F-X260 and F-X261, so it follows them.

## Spec reference

- `docs/hld/06-presentationml-model.md`, shape, text and table models.
- `docs/hld/08-rendering-spec.md`, Preset geometry and Tables.
- `docs/hld/10-bindings-spec.md`, PowerPoint Python formatting and live owners.
- `docs/hld/12-testing-strategy.md`, Binding tests and the render fidelity gate.
- `docs/hld/15-build-and-toolchain.md`, deterministic font mode and generated preset provenance.
- `docs/hld/14-development-backlog.md`, F-X262.

## Approach

Reimplement the nine rows from the functional parent. Do not merge the pull. Regenerate the preset table with the existing `tools/gen-presets` entry and keep its upstream pin. The `upArrow` geometry is accepted only when the generator output and the checked-in table match, and the plan records the source of the two adjustment values.

Percentage conversion matches pinned python-pptx 1.0.2. Transparency, gradient-stop position, percentage bullet size and run baseline use ties-to-even rounding, the same rule the existing crop setter already uses. They do not use half-away-from-zero. EMU, twips, time and adjustment handles keep their existing truncation. `0.8` transparency writes alpha `20000`. The tests include one genuine halfway value that lands even and one genuine halfway value that half-away-from-zero would send odd. The expected tie integers are measured against python-pptx 1.0.2. They are not invented here. `0.57` is exact and is not a rounding proof. Crop behaviour itself stays as it is.

Placeholder iteration visits every placeholder. Slide iteration uses ascending `idx`. Layout and master iteration use document order. Shape-tree order stays on `shapes`. Lookup by `idx` stays the existing lookup, and that compatibility decision is documented separately from iteration. A1 includes a fixture whose tree order differs from `idx` order.

Table growth is a render change. Stored `a:tr/@h` stays. The row's painted height grows to fit wrapped text, margins, spans and vertical text, and the following row moves with it. Fills, borders, background and text are checked together. This is an intentional PowerPoint paint delta and this story is the only baseline owner in its wave. The Word 49 hash stays still.

`TextFrame.clear` leaves one paragraph, keeps that paragraph's properties, and removes runs. Setting the frame text to empty writes a paragraph with no runs. Held runs of the cleared frame become stale. Other shapes stay live.

A table-cell run hyperlink uses the slide relationship scope that a text-box run already uses. Shared and invalid targets follow that existing policy. Save and reopen keep one relationship.

Bullet size accepts the current percentage float and a `Length` that writes `buSzPts`. The getter returns the stored kind. `None` clears the size element.

`remove_style` removes `p:style` from a shape or connector. An unsupported kind raises. Absence is not an error. Rust returns `bool`, true when an element was removed. Python returns `None`.

No new crate or module. The preset generator is the existing tool. New public names are `bullet_size` with a length, `TextFrame.clear` and `remove_style`.

## Consumer map

| Rule | Entry points and outputs | Shared provider and caller policy | Lifecycle stages compared |
|---|---|---|---|
| Placeholder iteration sees gaps | `slide`, layout and master `placeholders` | Idx lookup stays on the existing format. Shape order stays on `shapes` | Import, idx deletion, iteration, lookup |
| Percentage fields round together | Transparency, gradient stop, bullet percent, baseline | One conversion helper. EMU and twips constructors stay truncating | Set, XML, save and reopen |
| `upArrow` is a generated preset | Enum, preset name, import, render | `tools/gen-presets` output checked in | Generate, author, reopen, paint |
| Table rows grow only when painted | `rpptx-render` row layout and PDF or PNG | Stored `h` is not rewritten | Short text, wrapped text, span, vertical text |
| Clear and empty text agree | `TextFrame.clear`, `text` setter, `add_run` | Existing run owner generations | Held run, following append, reopen |
| Cell hyperlinks use the slide | Table-cell run and text-box run | Existing slide relationship owner | Add, replace, clear, invalid target |
| Bullet size keeps both units | Getter and setter on the paragraph | Percentage range stays 0.25 to 4.0 | Points, percent, `None`, invalid |
| Style removal is kind-checked | Shape and connector `remove_style` | Theme style element only | Present, absent, unsupported kind |

## Behavior contracts and evidence

| Surface or placement | Exact behavior and boundary examples | Primary source identity and location | Local discriminator or pending evidence |
|---|---|---|---|
| Placeholder iteration | Slide iteration is ascending `idx` and does not stop at a gap. Layout, master and notes iteration is document order. `placeholders[idx]` stays idx lookup on every owner. `shapes` stays tree order. | python-pptx 1.0.2 `src/pptx/shapes/shapetree.py`. `SlidePlaceholders.__iter__` sorts by `ph_idx` at lines 793 to 796. Layout and master inherit `_BaseShapes.__iter__` document order at lines 97 to 100. `NotesSlidePlaceholders` subclasses `MasterPlaceholders`. Our `PyPlaceholderCollection.__getitem__` is idx lookup at `crates/rpptx-py/src/shape.rs` lines 2399 to 2422, and the class has no `__iter__`, so Python stops at the first gap. Layout `__getitem__` in python-pptx is positional. Q4 keeps our idx lookup. | A1. Fixture tree order differs from idx order. |
| Percentage fields | Storage is thousandths of a percent. The helper is `int(round(value * 100000.0))`, Python 3 ties-to-even. Transparency stores `(1 - value) * 100000`. `0.8` writes alpha `20000`. `0.998085` writes `192`, and half-away-from-zero also writes `192`. `0.994255` writes `574`, while half-away-from-zero writes `575`. Gradient position `0.000015` writes `2` and `0.000025` writes `2` against half-away `3`. Bullet percent `0.250015` writes `25002` and `0.250025` writes `25002` against half-away `25003`. Baseline `-0.000015` writes `-2` and `-0.000025` writes `-2` against half-away `-3`. `0.57` scales to `56999.99999999999` and both rules write `57000`, so it is not a tie proof. Non-finite values fail the existing range checks. These setters do not accept Python `bool`. | python-pptx 1.0.2 `ST_Percentage.convert_to_xml` and `ST_TextSpacingPercentOrPercentString.convert_to_xml` in `src/pptx/oxml/simpletypes.py`. Current transparency truncates at `crates/rpptx-py/src/dml.rs` line 676. Crop already uses `round_ties_even` at `crates/rpptx-py/src/shape.rs` line 420. Adjustments still truncate at line 2469. | A2. Measured with Python 3 `round` on 2026-10-18. EMU, twips and time stay truncating. |
| `upArrow` | `MSO_SHAPE.UP_ARROW` is python-pptx value 35. The generator output and the checked-in table gain `upArrow` and lose the duplicate that displaced it. Any other preset name emitted by python-pptx 1.0.2 and dropped by the generator is the same row. Adjustment defaults are the generator output, not hand-written numbers. | Issue 392. `crates/rpptx-py/python/rpptx/enum/shapes.py` has no `upArrow`. | A3. |
| Table row paint | Stored `a:tr/@h` stays. Painted height grows to the tallest cell, including wrap, spans, margins and vertical text. Fills, borders, background and text are observed together. The following row moves with that height. | Issue 392. Q7. | A4. This story owns the PowerPoint paint baseline. |
| Clear and empty text | `clear` leaves one paragraph, keeps its properties and drops runs. Held runs of that frame go stale. Other shapes stay live. Empty text writes a paragraph with no runs. | Q5 and Issue 392. | A5 and A6. |
| Bullet size | Float from 0.25 to 4.0 writes `buSzPct` through the percentage helper. `Length` writes `buSzPts`, and `Pt(17.85)` is `1785`. `None` removes the size element. The getter returns the stored kind. `0.1` still refuses. | Q2 and Issue 392. | A8. |
| Style removal | Shapes and connectors only. Rust returns `bool`, `false` when `p:style` is already absent. Python returns `None`. An unsupported kind raises and does not publish. | Q6. | A9. |

## Dependency readiness

`BACKLOG.md` records F-X205, F-X206 and F-X258 as `done`. The formal edges are met. This story still waits for the F-X260 and F-X261 file lease on `crates/rpptx/src/lib.rs` and `docs/hld/10-bindings-spec.md`. Its own questions are answered, so the plan stays approved. Approval is not implementation readiness while those files are leased.

## Acceptance criteria

| ID | Input or setup | Expected observation or refusal | Consumers | Local test | Final gate or pending evidence |
|---|---|---|---|---|---|
| A1 | Slide, layout and master with an idx gap, plus one fixture whose tree order differs from idx order | Slide iteration yields every placeholder in ascending idx. Layout and master iteration yield document order. `placeholders[idx]` still addresses that idx | Python placeholders and shapes | `issue392_placeholder_iteration_crosses_index_gaps` | Installed `rpptx` suite |
| A2 | Transparency `0.8`, `0.998085` and `0.994255`. Gradient position `0.000015` and `0.000025`. Bullet percent `0.250015` and `0.250025`. Baseline `-0.000015` and `-0.000025`. `0.57` is present only to show it does not separate the tie rule | Alpha `20000`, `192` and `574`. Positions `2` and `2`. Bullet values `25002` and `25002`. Baselines `-2` and `-2`. Half-away-from-zero differs on `0.994255`, `0.000025`, `0.250025` and `-0.000025`. EMU, twips, time and adjustments still truncate | Fill, gradient, bullet percent, baseline | `issue392_percentage_rounding_has_explicit_boundaries` | Final wasm check. No Word hash delta |
| A3 | `MSO_SHAPE.UP_ARROW`, `add_shape("upArrow")`, and an imported `upArrow` | The geometry matches the regenerated table. Reopen keeps the type. Adjustments are the generated pair | Enum, preset parser, render | `issue392_up_arrow_has_reproducible_geometry` | Generator rerun is local. Paint of a new shape is not a corpus delta unless a fixture already draws it |
| A4 | Short text, wrapped text, a span, and vertical text | Painted row height fits the text. The next row does not overlap. Stored `h` is unchanged | Render geometry, PDF and PNG | `issue392_table_rows_grow_without_overlapping_following_rows` | Pinned viewer or pixel baseline, owned by this story |
| A5 | Text box and table cell with paragraph properties and two runs | `clear` leaves one empty paragraph and those properties. Held runs of that frame are stale | Python text frame | `issue392_text_frame_clear_preserves_approved_properties` | Installed suite |
| A6 | `text = ""` then `add_run` | The paragraph has one new run and no leading empty run | Text setter and run collection | `issue392_empty_text_then_append_has_one_new_run` | Installed suite |
| A7 | Table-cell run hyperlink add, change, clear, and an unknown target | The relationship is on the slide. Invalid input does not publish. Save and reopen keep the address | Cell run and text-box run | `issue392_cell_run_hyperlinks_use_the_actual_slide_scope` | Installed suite |
| A8 | `Pt(17.85)`, percentage `0.29`, `None`, and `0.1` | Points write `buSzPts` `1785`. Percent writes `buSzPct`. `None` removes it. `0.1` still refuses | Getter and setter | `issue392_bullet_size_distinguishes_points_from_percent` | Installed suite and stub check |
| A9 | Shape, connector, a shape with no `p:style`, and a kind that has no style | Style element is removed. Absence does not raise. The unsupported kind raises and does not publish | Python and native shape | `issue392_style_removal_is_kind_checked` | Installed suite |

All nine issue rows map in order to A1 through A9. A2 covers transparency, gradient-stop position, percentage bullet size and baseline. It does not change unrelated truncation.

## Test matrix

| Case | Modes or state combined | Expected difference or agreement | Wrong implementation rejected |
|---|---|---|---|
| C1 | Gap at the start, middle and end. Slide, layout and master. One fixture whose tree order differs from idx order | Slide order is ascending idx. Layout and master order is document order. Idx lookup is unchanged | A slide-only fix, an idx sort on a layout or master, or a lookup that became positional |
| C2 | Truncation value `0.8`, exact `0.57`, one even tie and one tie that half-away-from-zero would send odd | Only the non-exact values show ties-to-even. At least one tie differs from half-away-from-zero | A test that uses `0.57` as proof of rounding, or a half-away-from-zero helper |
| C3 | Short row versus wrapped row, with and without a span | Stored XML matches. Painted bounds differ | Rewriting `a:tr/@h` and calling that a render fix |
| C4 | Held run across `clear` versus a run on another shape | The cleared run is stale. The other shape stays live | Retiring every run on the slide |
| C5 | Functional parent versus measurement head | Same product bytes | Treating the README commit as a render change |

CLI export uses the same renderer as the library, so A4 covers it. WASM has no separate table painter. The final wasm check is a compile rider, not a second paint oracle.

## Rejected alternatives

- Merging PR394, including its README measurement commit, as the implementation.
- Changing stored row heights. Later editors would save the grown value.
- Rounding every integer conversion. Unit constructors are pinned.
- Adding `upArrow` by hand in the generated table without a generator change.

## Test plan

**Test gate**: regression. Each A-ID has an old-source semantic RED where the behaviour already exists, and a repaired GREEN. Missing methods are compile failures and are logged separately.

| Timing | Category | Test | Asserts |
|---|---|---|---|
| local scoped | regression | A1, A2, A3, A5 through A9 in `crates/rpptx-py/tests/test_documented_examples.py` | The issue values above |
| local scoped | render | A4 geometry in `crates/rpptx-render` | Painted row bounds and unchanged stored `h` |
| final sprint | rider | deterministic PowerPoint pixels, pinned viewer, full hash harness, wasm check, preset generator check | Word hash unchanged. PowerPoint delta matches A4 only |

Do not add a test binary. The generator output is reviewed in the same commit as the generator change.

## HLD impact

- `docs/hld/06-presentationml-model.md`
- `docs/hld/08-rendering-spec.md`
- `docs/hld/10-bindings-spec.md`
- `docs/hld/12-testing-strategy.md`
- `docs/hld/14-development-backlog.md`
- `docs/hld/15-build-and-toolchain.md`, only if the preset pin text changes

## Risk routing

- Contributed PR or new issue acceptance. Pin the refreshed head and the functional parent. All nine rows stay mapped.
- Layout, pagination, line breaking or text shaping. A4 uses deterministic font mode. The pixel baseline is re-recorded only for the declared table-growth delta.
- Any parser or serialiser. Preset and text writers keep schema order. `p:style` removal must not reorder the remaining shape children.
- Public API of a published crate. `bullet_size`, `clear` and `remove_style` need the semver note for the approved signatures. Final publish dry run.
- WASM or PyO3 bindings. Final `rpptx-wasm` check. Python tests stay excluded from the workspace binary.
- A generated artefact. Rerun `tools/gen-presets` and commit only the `upArrow` delta it produces.
- Shared semantic behavior with multiple consumers. Cell and text-box hyperlinks, and slide, layout and master placeholders.
- Unit conversion. EMU, twips, time and adjustment values still truncate. The percentage helper is ties-to-even and is a different function.
- Compatibility, numeric conversion or collection ordering. The percentage rows and the per-class placeholder order above are the check.

Theme tint stays on the naive Word helper. This story does not call it.

## Hash harness

Word 49 hash and the Word seven-pixel baseline stay unchanged. PowerPoint paint may change for wrapped table rows. That delta is labelled and this story is the only baseline writer in its wave. `upArrow` does not move an existing baseline unless a current fixture draws that preset.

## Implementation checklist

- [ ] Re-read the functional parent after F-X259, F-X260 and F-X261 have left the shared files.
- [x] Measure the genuine percentage ties against python-pptx 1.0.2 before writing expected integers.
- [ ] Record old-source RED for transparency, placeholders, empty text and bullet size.
- [x] Regenerate presets and implement the nine rows in the existing crates.
- [x] Update the HLD list and obtain a zero-finding microscope.
- [ ] Leave the pixel baseline, viewer and wasm check for the final union.

## Contract closure

R4's stale source sentence is corrected in Problem. F-X259 is integrated at `4879e58a`. This story's own questions stay answered. It still waits on the F-X260 and F-X261 file lease. No product contract changed in this pass.

## Deviations

The file lease named above is met. F-X260 is integrated at `883080bbe0277689ac48785f4a44ca24eb8db38c` and F-X261 at `6adb7258e7565969a58c55d3971d724519a59cd8`. That second commit is the parent of claim `2fdb6bf38aea54053096236bd837d5940fc45c64`. No product contract changed.

PR394 head `e9f9fbe7b86388b8702af5757bdfa7e4e4fc6596` and functional parent `f9d165e77392b57a58223046d8cd551220c61a3a` are not in this repository. `gh pr view 394` returned HTTP 401. The functional parent was not re-read. The approved plan stayed the source. The re-read checklist item stays open.

`set_cell_run_hyperlink` takes the cell as one `(row, column)` argument so the public method stays within the existing argument limit. The slide-scoped relationship behaviour is unchanged.

The layout and master lookup assertion checks each present idx. Comparing `placeholders[0]` with the first document-order idx would have required positional lookup, which Q4 keeps separate.

HLD 15 is unchanged. Its preset pin text does not name the generated count or the source digest.

Old-source RED was not recorded. The local tests are fixed-source GREEN only. The pixel baseline, pinned viewer and wasm check stay for the final union.

## Open questions

| ID | Affected F-IDs and A-IDs | Decision and recommendation | Status and answer/source |
|---|---|---|---|
| Q2 | F-X262 A8 | Bullet size accepts `float`, `Length` or `None`. A float from 0.25 to 4.0 writes `buSzPct`. A `Length` writes `buSzPts`. `Pt(17.85)` is `1785`. `None` clears the element. The getter returns the stored kind. | Answered 2026-10-18 by the user. |
| Q3 | F-X262 A2 | Percentage rounding matches python-pptx 1.0.2 ties-to-even for transparency, gradient-stop position, percentage bullet size and baseline. `0.8` writes `20000`. One tie lands even under both rules and one tie is sent odd by half-away-from-zero. Unrelated truncation stays. | Answered 2026-10-18 by the user. The integers above were measured from `int(round(value * 100000.0))` on 2026-10-18. |
| Q4 | F-X262 A1 | Slide placeholders iterate by ascending idx. Layout and master placeholders iterate in document order. Idx lookup stays a separate compatibility decision. Notes follow the master class, so they use document order. | Answered 2026-10-18 by the user. Notes order is the python-pptx 1.0.2 subclass fact, not a new choice. |
| Q5 | F-X262 A5 and A6 | `TextFrame.clear` leaves one paragraph, keeps its properties and drops runs. Empty text writes a paragraph with no runs. Held runs of that frame become stale. Other shapes stay live. | Answered 2026-10-18 by the user. |
| Q6 | F-X262 A9 | `remove_style` applies to shapes and connectors. Rust returns `bool`. Python returns `None`. A missing `p:style` returns `false`. An unsupported kind raises and does not publish. | Answered 2026-10-18 by the user. |
| Q7 | F-X262 A4 | Stored row height stays. Painted height grows so the next row does not overlap. This story owns the PowerPoint baseline. The Word 49 hash and seven-pixel baseline stay unchanged. | Answered 2026-10-18 by the user. |

No issue or pull request closes from this plan.
