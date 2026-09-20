# F-264, Complete paragraph property authoring

**Status**: completed
**Sprint**: S74
**Size**: L
**Depends on**: F-253

## Problem

Line citations below are against the pre-split
`crates/rdocx-oxml/src/properties.rs`, because that is where the state was read.
The sprint splits that file before wave 1 claims, so the implementing session
reads the same code at its new path. See the first note in Approach.

`CT_PPr` already carries a complete schema slot table for every `w:pPr` child,
`crates/rdocx-oxml/src/properties.rs:2538` to `2578`, so unmodelled children
survive at their correct `xsd:sequence` position. Only eighteen of those
thirty-six slots are typed, `crates/rdocx-oxml/src/properties.rs:2591` to
`2611`. The untyped remainder includes `w:framePr` (slot 4),
`w:suppressLineNumbers` (slot 7), `w:adjustRightInd` (slot 19),
`w:contextualSpacing` (slot 23), `w:mirrorIndents` (slot 24),
`w:suppressOverlap` (slot 25), `w:textDirection` (slot 27),
`w:textAlignment` (slot 28) and `w:textboxTightWrap` (slot 29). A caller can
open a document that uses them and save it without loss, but cannot create,
read, mutate or remove any of them.

The facade gap is larger than the grammar gap. `Paragraph` at
`crates/rdocx/src/paragraph.rs:293` has no setter for the logical indentation
`ind_start` and `ind_end` that `CT_PPr` already models at
`crates/rdocx-oxml/src/properties.rs:118` to `121`, none for the automatic
spacing at `:108` to `:111`, none for `suppress_auto_hyphens` at `:132`, none
for `bidi` at `:134`, and none for the paragraph-mark run properties at `:142`.
Border authoring covers only "all four edges" and "bottom",
`crates/rdocx/src/paragraph.rs:856` and `:897`, so `w:left`, `w:right`,
`w:top`, `w:between` and `w:bar` are unreachable even though `CT_PBdr` models
all six, `crates/rdocx-oxml/src/borders.rs:126`. `CT_BorderEdge` at
`crates/rdocx-oxml/src/borders.rs:15` types only `val`, `sz`, `space` and
`color`, so a producer's `w:shadow`, `w:frame`, `w:themeColor`, `w:themeTint`
and `w:themeShade` are dropped on write. Shading authoring hard-codes
`val="clear"` and `color="auto"`, `crates/rdocx/src/paragraph.rs:841`. Tabs can
only be appended, `crates/rdocx/src/paragraph.rs:924`, never read back by
index, replaced or removed, and `ParagraphRef` offers only `tab_stop_count`,
`crates/rdocx/src/paragraph.rs:1460`. `ParagraphRef` has no reader for hanging
indent, logical indentation, outline level, hyphenation, direction or the
paragraph mark. That is what makes `DOCX-030` a `partial` row with `P` in
Create, Mutate, Remove, Save-reopen and Native at
`docs/hld/02-scope-and-non-goals.md:237`.

## Spec reference

- `docs/hld/02-scope-and-non-goals.md`, "Modern DOCX capability matrix", the
  `DOCX-030` row and the rule that public OXML types do not count as facade
  authoring and that save-reopen means modeled state returns through
  `Document`.
- `docs/hld/03-architecture.md`, "What stays put", for `rdocx-oxml` owning the
  WordprocessingML property grammar, expanded names and schema positions
  deciding typed meaning, every unmodelled property staying in its original
  schema slot, and the facade computing effective paragraph properties without
  a second reader model.
- `docs/hld/03-architecture.md`, "Facade conventions", for the borrow-handle
  idiom, consuming builders for formatting, `&mut self` methods returning a
  nested handle, and index-based `Option`-returning accessors that never panic.
- `docs/hld/04-opc-and-packaging.md`, "Package integrity", the paragraph on
  selecting attributes by the bound WordprocessingML namespace and serializing
  canonical values with fixed `w` attributes in schema order while unmodelled
  content retains its stored bytes.
- `docs/hld/12-testing-strategy.md`, "Test taxonomy", for the `round-trip`
  category definition and the no-binary-fixture rule.
- `docs/hld/14-development-backlog.md`, "F-264, Complete paragraph property
  authoring (L)", for the scope list and the named round-trip gate.

## Approach

Three layers, grammar first.

### The starting base

**This story does not move any file.** Before wave 1 claims, the sprint splits
the existing `crates/rdocx-oxml/src/properties.rs` into separate
paragraph-property and run-property modules as its own byte-identical labelled
commit, with re-exports preserving every existing public path. F-264 starts
from that moved base. Everything below that this plan calls **the paragraph
property module** is whatever the split names the module holding `CT_PPr`.
Because the split is a pure file move, the hash harness is byte identical
across it, and that obligation belongs to the split commit rather than to this
story. No behaviour change of F-264 may be folded into the move.

### The paragraph property module

Add nine typed fields to `CT_PPr`, each already owning a schema slot:

```rust
pub frame: Option<CT_FramePr>,            // slot 4
pub suppress_line_numbers: Option<bool>,  // slot 7
pub adjust_right_ind: Option<bool>,       // slot 19
pub contextual_spacing: Option<bool>,     // slot 23
pub mirror_indents: Option<bool>,         // slot 24
pub suppress_overlap: Option<bool>,       // slot 25
pub text_direction: Option<String>,       // slot 27
pub text_alignment: Option<String>,       // slot 28
pub textbox_tight_wrap: Option<String>,   // slot 29
```

`text_direction`, `text_alignment` and `textbox_tight_wrap` stay `String` for
the same reason `CT_Shd::val` and `CT_PPr::line_rule` do, which is that a
producer token outside the `ST_*` enumeration must survive unchanged. The
facade enums convert at the boundary.

`CT_FramePr` is a new struct in the same module, attributes only, mirroring the
`CT_Shd` shape:

```rust
pub struct CT_FramePr {
    pub drop_cap: Option<String>,
    pub lines: Option<u32>,
    pub w: Option<Twips>,
    pub h: Option<Twips>,
    pub v_space: Option<Twips>,
    pub h_space: Option<Twips>,
    pub wrap: Option<String>,
    pub h_anchor: Option<String>,
    pub v_anchor: Option<String>,
    pub x: Option<Twips>,
    pub x_align: Option<String>,
    pub y: Option<Twips>,
    pub y_align: Option<String>,
    pub h_rule: Option<String>,
    pub anchor_lock: Option<bool>,
    pub extra_attributes: Vec<(String, String)>,
}
```

`extra_attributes` keeps producer and foreign attributes in source order, the
same retention `num_pr_extra_attributes` already gives `w:numPr`. It is written
back before the modeled attributes are appended, so an unmodelled `w:framePr`
attribute is never dropped by typing the element.

Parsing joins the existing `Event::Empty` and `Event::Start` arms of
`from_xml_with_prefixes_and_owner_bindings`, uses `is_word_element` and
`is_word_attribute` so it stays prefix tolerant, and adds the nine local names
to `ppr_modeled_slot`. Serialization inserts each field into `to_xml` at its
slot position, which is the only correct place because
`write_ppr_with_positioned_raw` replays retained raw children against
`ppr_slot_for_name` over the generated bytes.

The toggle members join the carrier machinery `w:bidi` already uses.
`record_modeled_toggle_candidate` and `remove_redundant_modeled_toggle_candidate`
keep the source bytes when the element carries an attribute the model does not
own, so a parse-then-save of an untouched document stays byte identical. The
single hard-coded bidi test becomes one helper:

```rust
fn ppr_modeled_toggle_present(ppr: &CT_PPr, slot: u8) -> bool
```

returning `true` for any slot that is not a modeled toggle, which keeps the
existing behaviour for every other slot and adds the new ones by one match arm
each. That is a generalisation of one condition, not a new abstraction.

`is_empty` and `merge_from` gain one clause per new field. `merge_from` is what
`resolve_paragraph_properties` at `crates/rdocx/src/style.rs:585` walks, so
the new fields inherit through `docDefaults` and the `basedOn` chain with no
further change.

### `crates/rdocx-oxml/src/borders.rs`

**F-264 owns `CT_BorderEdge` attribute retention.** F-269 needs `w:shadow`,
`w:frame`, `w:themeColor`, `w:themeTint` and `w:themeShade` retained for its
page borders, and this story touches the same type first, so the change lands
here once and F-269 consumes it rather than duplicating it.

Add ordered attribute retention to `CT_BorderEdge` in the shape
`CT_FramePr::extra_attributes` uses, an
`extra_attributes: Vec<(String, String)>` filled in source order by the
existing `from_xml_attrs_with_prefixes` path and replayed before the modeled
`w:val`, `w:sz`, `w:space` and `w:color` are appended. That covers the five
attributes F-269 names and every other producer or foreign attribute in one
mechanism, rather than five typed fields that would still drop the sixth.
`CT_BorderEdge::new` and the existing struct-literal call sites keep working
because the field defaults to empty.

### `crates/rdocx/src/paragraph.rs`

Follow the file's existing triple convention exactly: a consuming builder
`fn name(mut self, ..) -> Self`, an in-place `fn set_name(&mut self, ..)`, and
a clearing `fn set_name_value(&mut self, Option<..>)` that returns early when
the value is `None` and `self.inner.properties` is already `None`.

New facade types, all in `paragraph.rs`, none in a new file:

```rust
pub enum ParagraphBorderEdge { Top, Bottom, Left, Right, Between, Bar }
pub enum ParagraphTextDirection { /* the six ST_TextDirection tokens */ }
pub enum ParagraphTextAlignment { Top, Center, Baseline, Bottom, Auto }
pub enum TextboxTightWrap { None, AllLines, FirstAndLastLine, FirstLineOnly, LastLineOnly }
pub enum DropCap { None, Drop, Margin }
pub enum FrameWrap { Auto, NotBeside, Around, Tight, Through, None }
pub enum FrameAnchor { Margin, Page, Text }
pub struct ParagraphFrame { /* checked mirror of CT_FramePr */ }
pub struct TabStopRef<'a> { /* alignment, position, leader */ }
pub struct ParagraphMark<'a> { inner: &'a mut CT_RPr }
pub struct ParagraphMarkRef<'a> { inner: Option<&'a CT_RPr> }
```

`ParagraphBorderEdge` mirrors the existing `CellBorderEdge` at
`crates/rdocx/src/table.rs:76`, and `ParagraphTextDirection` is a parallel enum
beside `CellTextDirection` at `crates/rdocx/src/table.rs:101`. The sprint
decided to add the parallel enum and **not** rename `CellTextDirection`, since
renaming is a breaking change to public API and a cell-named type reads wrong
on a paragraph.

`ParagraphMark` is a nested handle reached through `&mut self`, which is the
idiom `docs/hld/03-architecture.md` "Facade conventions" names, and the sprint
confirmed that shape over flat `mark_*` methods on `Paragraph`.

Setters added to `Paragraph`, grouped by the backlog's own scope list:

- Logical indentation. `indent_start`, `indent_end`, `mirror_indents`,
  `adjust_right_indent`, and the missing `set_hanging_indent_value`.
- Automatic spacing. `space_before_auto`, `space_after_auto`,
  `contextual_spacing`, and `line_spacing_rule` clearing.
- Borders. `border(edge, ..)` and `set_border_value(edge, Option<..>)` for all
  six edges, plus `clear_borders`. `border_all` and `border_bottom` stay as
  they are, so no existing caller breaks.
- Shading. `shading_pattern(pattern, fill, color)` and
  `set_shading_value(Option<..>)` for removal. `shading` stays.
- Tabs. `tab_stop(index) -> Option<TabStopRef>`, `set_tab_stop(index, ..)`,
  `remove_tab_stop(index) -> bool`, `clear_tab_stops`. Index accessors return
  `Option` and never panic.
- Pagination. `suppress_line_numbers`, `suppress_auto_hyphens`, and the
  `_value` clearers for the four toggles that already have setters.
- Frames. `frame(ParagraphFrame)`, `set_frame_value(Option<ParagraphFrame>)`,
  `suppress_overlap`, `textbox_tight_wrap`.
- Outline. `set_outline_level_value(Option<u32>)`, accepting 0 to 9 inclusive
  and rejecting above, because Word writes 0 to 8 for heading levels and 9 for
  body text.
- Direction. **`right_to_left` for `w:bidi` is a named deliverable of this
  story, not an incidental.** F-266a depends on the public setter and the
  matching `ParagraphRef` reader to build its RTL golden fixture through public
  API, so both ship here with their own round-trip coverage. Alongside it,
  `text_direction` and `text_alignment`.
- Paragraph mark. `mark(&mut self) -> ParagraphMark<'_>`, whose setters cover
  the mark family that `Run` already exposes over the same `CT_RPr`, and
  `clear_mark`.

`ParagraphRef` gains the matching reader for every one of the above, including
`right_to_left` for F-266a and the four that are missing today for
already-modeled state: `hanging_indent`, `indent_start`, `indent_end` and
`outline_level`.

### What this story does not take

- **`CT_Shd` theme attributes belong to F-265**, which is doing the full theme
  sweep across `w:rFonts`, `w:color` and `w:shd`. F-264 does not need them.
  Paragraph shading authoring here reads and writes `val`, `color` and `fill`
  only, adds no field to `CT_Shd`, and leaves the existing parse and write
  paths for that type untouched, so F-265's change applies cleanly on top.
- **No layout change is required.** `crates/rdocx-layout/src/engine.rs:6461` to
  `:6480` already folds `ind_start` and `ind_end` into the effective left and
  right indent under the resolved text direction, and
  `crates/rdocx-layout/src/paginator.rs:2269` and `:2282` already lay out
  paragraph shading and borders.

### The DOCX-030 row

Decided in the consolidated round. F-264 closes `Create`, `Read`, `Mutate`,
`Remove`, `Save-reopen` and `Native` to `Y`. `Layout` and `Render` stay `P`,
because this story's gate is round-trip and positioned paragraph frames are a
flow-breaking layout feature it does not build. The M24 end-of-milestone gate
forbids an *unexplained* partial row rather than a partial row, so the row's
evidence cell must name the owner for positioned frame placement explicitly.
The row is therefore explained rather than bare, and the classification stays
`partial` until that owner lands.

## Rejected alternatives

- Expose `&mut CT_PPr` from the facade. `docs/hld/02-scope-and-non-goals.md`
  states that public OXML types do not count as facade authoring, so the row
  would stay `partial`.
- Add `mark_bold`, `mark_italic` and the rest directly onto `Paragraph`. It
  needs no new type but adds roughly a dozen prefixed methods to a handle that
  already has more than sixty, and it reads worse than one nested handle.
- Refactor `Run`'s formatting setters onto a shared handle over `&mut CT_RPr`
  and have `Run` delegate. It removes duplication but touches every run setter
  in a story whose gate is paragraph round-trip, and F-265 owns the run
  surface next.
- Model `w:kinsoku`, `w:wordWrap`, `w:overflowPunct`, `w:topLinePunct`,
  `w:autoSpaceDE`, `w:autoSpaceDN` and `w:snapToGrid` here. They are the
  `DOCX-033` East Asian family, and the sprint assigned them to **F-266c**,
  character grid and vertical text. Typing them without the typography
  behaviour would claim a capability the renderer does not have.
- Model `w:cnfStyle` here. It is table conditional formatting owned by
  **F-267**, and stays raw-preserved at slot 32.

`w:divId` is the exception to the hand-off pattern above, and it is typed
here. It is a `w:pPr` child, so the paragraph property grammar owns it under
the sprint's ownership rule, and F-264 is the only wave 1 story that edits
`CT_PPr`. Typing it in **F-270** instead would put two wave 1 workers in the
same struct, the same parse arm and the same write order. F-264 therefore
types `w:divId` at slot 31 and ships its paragraph facade accessor. **F-270**
owns the other half, `CT_WebSettings` and the read-only `div_ids()`
projection that says whether a reference resolves.
- Type the five border attributes F-269 needs as five named fields on
  `CT_BorderEdge`. Ordered attribute retention covers those five and every
  other producer attribute in one mechanism, so it removes a whole class of
  loss rather than five instances of it.
- Rename `CellTextDirection` to a neutral shared name. It is public API and the
  rename breaks callers for no behavioural gain.
- Perform the properties module split inside this story. A file move must be
  byte identical across the harness, and folding a behaviour change into it is
  forbidden by `.claude/WORKFLOW.md`.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| round-trip | `every_public_paragraph_property_reopens_and_preserves_unrelated_xml` | Authors every new property through `Paragraph`, saves, reopens, reads each value back through `ParagraphRef`, and asserts the producer's unmodelled `w:pPr` children and root attributes are byte identical |
| round-trip | `paragraph_right_to_left_authored_publicly_reopens_as_modeled_state` | The named `w:bidi` deliverable F-266a depends on, set and cleared through `Paragraph` and read back through `ParagraphRef` after save and reopen |
| unit | `frame_properties_round_trip_every_modeled_attribute` | `CT_FramePr` parse and write preserve each typed attribute and every unmodelled attribute in source order |
| unit | `border_edge_retains_shadow_frame_and_theme_attributes` | `CT_BorderEdge` ordered retention keeps `w:shadow`, `w:frame`, `w:themeColor`, `w:themeTint`, `w:themeShade` and an unrelated foreign attribute across parse and write, which is what F-269 consumes |
| unit | `modeled_paragraph_children_serialize_in_schema_sequence_order` | The nine new children land at their `ppr_slot_for_name` positions among retained raw children |
| unit | `newly_modeled_paragraph_toggles_replay_their_source_carrier` | An untouched toggle carrying an unowned attribute survives a parse and save byte for byte |
| unit | `paragraph_property_parsing_accepts_alias_and_foreign_namespaces` | Prefix-tolerant read of the new children, and a foreign same-local element stays unmodelled |
| unit | `new_paragraph_properties_inherit_through_the_style_chain` | `merge_from` and `resolve_paragraph_properties` carry each new field from `docDefaults` and `basedOn` |
| integration | `paragraph_border_edges_author_read_and_clear_individually` | Each of the six edges is settable, readable and removable without disturbing the others, and retained attributes survive an edge mutation |
| integration | `tab_stops_read_mutate_and_remove_by_index` | Index accessors return `Option`, an out-of-range index is `None` rather than a panic, and removal preserves order |
| integration | `paragraph_mark_formatting_authors_reads_and_clears` | `Paragraph::mark` writes `w:pPr/w:rPr`, `ParagraphRef` reads it, and `clear_mark` removes the element |
| integration | `outline_level_accepts_zero_through_nine_and_rejects_above` | The decided bound, with no panic on the rejected path |
| regression | `a_paragraph_that_only_gains_a_modeled_frame_keeps_its_producer_attribute_order` | Typing `w:framePr` does not reorder or drop producer attributes |
| regression | `logical_indentation_set_through_the_facade_survives_save_and_reopen` | `w:ind/@w:start` and `@w:end` reopen as modeled state, not as raw bytes |
| regression | `clearing_the_last_paragraph_property_removes_the_empty_ppr` | A `_value(None)` call that empties the properties leaves no empty `w:pPr` behind |

The **test gate** is the round-trip test
`every_public_paragraph_property_reopens_and_preserves_unrelated_xml`.

Every test above runs on this machine with no Word GUI automation and no human
in the loop, which matches the sprint's evidence policy. The gate is
round-trip, so that policy adds no obligation here.

Unit tests are inline `#[cfg(test)]` modules in the paragraph property module,
`crates/rdocx-oxml/src/borders.rs` and `crates/rdocx/src/style.rs`. Integration
tests are new modules inside the existing
`crates/rdocx/tests/integration_test.rs`, and regression tests inside the
existing `crates/rdocx/tests/regression_test.rs`. No new file under any
`tests/` directory, and no binary fixture.

## HLD impact

- `docs/hld/02-scope-and-non-goals.md`
- `docs/hld/03-architecture.md`
- `docs/hld/04-opc-and-packaging.md`
- `docs/hld/12-testing-strategy.md`
- `docs/hld/14-development-backlog.md`

## Risk routing

Three rows match.

- **Any parser or serialiser.** Read `docs/hld/04-opc-and-packaging.md`. The
  extra checks are an `xsd:sequence` assertion over the nine new children
  against `ppr_slot_for_name`, prefix-tolerant reads with a fixed `w` prefix on
  write, and a round-trip test proving the unmodelled subtree that
  `capture_element` retained is byte identical after the newly typed siblings
  are serialized. Typing an element that used to replay verbatim is the
  specific hazard here, so the toggle-carrier path is asserted for each new
  toggle and not only for `w:bidi`. The same obligation covers the
  `CT_BorderEdge` retention change, where an attribute that used to be dropped
  now round-trips in source order.
- **Public API of a published crate.** Read `docs/hld/10-bindings-spec.md` and
  the `CLAUDE.md` structural rules. `CT_PPr` and `CT_BorderEdge` are public
  structs that are not `#[non_exhaustive]`, so adding fields breaks any
  struct-literal construction outside the workspace. On a pre-1.0 family that
  is a minor bump, and it is the same shape as the `CT_P` field additions
  F-X128 already landed. The extra checks are a stated semver impact of minor,
  a grep proving every in-workspace `CT_PPr` and `CT_BorderEdge` literal either
  ends in `..Default::default()` or is updated in this diff, and
  `cargo publish --dry-run` with the `.crate` size assertion for `rdocx` and
  `rdocx-oxml`. No surface beyond the backlog's scope list.
- **Unit conversion, `Twips`.** Read `docs/hld/01-glossary.md` units and the
  `CLAUDE.md` "Things that are deliberately wrong" entry. `ParagraphFrame`
  takes `Length` and converts through the pinned truncating `as_twips`. No new
  rounding, no change to an existing constructor.

The **file move or rename with no behaviour change** row is matched by the
separate properties module split commit, not by this story. F-264 starts from
the moved base and folds no behaviour change into the move.

The remaining rows do not match. No theme colour, no layout or pagination
change, no crate dependency change, no bundled asset, no binding change, no new
feature flag, no external oracle, no release scripting.

## Hash harness

Expected unchanged. `scripts/hash_harness.py` regenerates the seven samples
through `crates/rdocx/examples/generate_all_samples.rs` and none of them
authors a frame, logical indentation, automatic spacing, a text direction, a
retained border attribute or a paragraph-mark property. Every new `CT_PPr` and
`CT_BorderEdge` field defaults to `None` or empty and every new write path is
guarded, so the serialized bytes of the existing samples are identical. If the
implementation extends a sample to demonstrate a new setter, that is a separate
labelled commit with the re-recorded baseline and the stated delta, never
folded into this one. The properties module split has its own byte-identical
harness obligation in its own commit, ahead of this story.

## Implementation checklist

- [x] Start from the split base. Confirm the paragraph property module exists
      and its re-exports preserve every public path before editing.
- [x] Add `CT_FramePr` and the nine `CT_PPr` fields with their doc comments.
- [x] Extend `ppr_modeled_slot` and both parse arms, prefix tolerant.
- [x] Insert each field into `to_xml` at its schema slot.
- [x] Replace the hard-coded bidi carrier test with `ppr_modeled_toggle_present`.
- [x] Extend `is_empty` and `merge_from`.
- [x] Add ordered attribute retention to `CT_BorderEdge` in
      `crates/rdocx-oxml/src/borders.rs`, covering `w:shadow`, `w:frame`,
      `w:themeColor`, `w:themeTint` and `w:themeShade`, for F-269 to consume.
- [x] Type `w:divId` on `CT_PPr` at slot 31 and expose its paragraph facade
      accessor, with a round-trip test. F-270 owns the matching
      `CT_WebSettings::div_ids()` projection and the two halves meet at
      integration, so this story asserts only the paragraph side.
- [x] Add the facade enums, `ParagraphFrame`, `TabStopRef`, `ParagraphMark` and
      `ParagraphMarkRef` to `crates/rdocx/src/paragraph.rs`.
- [x] Add every `Paragraph` setter triple in the file's existing convention,
      including the named `right_to_left` deliverable for F-266a.
- [x] Add every matching `ParagraphRef` reader, including `right_to_left` and
      the four missing for already-modeled state.
- [x] Export the new public names from `crates/rdocx/src/lib.rs`.
- [x] Write the round-trip gate, then the unit, integration and regression tests.
- [x] Update the `DOCX-030` row to `Y` on the authoring columns and name the
      positioned-frame owner in its evidence cell.
- [x] Run the impacted crate tests, clippy, fmt, the hash harness and the prose
      gate.

## Open questions

None. Resolved in the S74 consolidated design round.
