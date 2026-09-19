# F-264, all, pass 1

**Reviewed**: the uncommitted working tree on `work/f-264-claude`, 18 files,
2559 insertions and 140 deletions
**Verdict**: 0 defects, 3 smells, 5 nitpicks

## Defects

None.

`correctness`, `panics` and `ooxml` produced no defect. The sections below say
what was checked.

## Smells

### S1, the two public attribute-retention fields do not state their escaping contract
`crates/rdocx-oxml/src/paragraph_properties.rs:73`,
`crates/rdocx-oxml/src/borders.rs:33`

`CT_FramePr::extra_attributes` and `CT_BorderEdge::extra_attributes` store the
attribute value exactly as the source spelled it, and the writers replay it
through `Attribute { key, value: Cow::Borrowed(..) }`, which does not escape.
That is correct for a parsed value and it is what `num_pr_extra_attributes`
already does, but both new fields are public and documented only as keeping
"their stored spelling". F-269 is the named consumer of the border field and
will populate it in code. A caller that stores a value containing `&`, `<` or a
quote gets invalid XML out with no diagnostic. The contract has to be on the
field.

### S2, two tables must agree or a parse underflows
`crates/rdocx-oxml/src/paragraph_properties.rs:1762`,
`crates/rdocx-oxml/src/paragraph_properties.rs:1721`

`PPR_MODELED_TOGGLES` names the six toggles that retain an attribute carrier,
and `ppr_modeled_slot` separately names every modeled child. Both parse arms
read `occurrences[slot as usize] - 1` after `record_ppr_modeled` has
incremented that slot. A future toggle added to the first table but not the
second makes that subtraction underflow on the first occurrence, which panics
in a debug build and wraps in release. Today the two lists agree, so nothing is
wrong, and nothing in the file proves they stay that way.

### S3, the storage-type corrections are a deviation and are recorded nowhere
`crates/rdocx-oxml/src/paragraph_properties.rs:228`,
`crates/rdocx-oxml/src/paragraph_properties.rs:253`

The design plan asks for `pub frame: Option<CT_FramePr>` and leaves
`pub borders: Option<CT_PBdr>` alone. The implementation stores both behind a
`Box`. The reason is real and measured, and it follows the F-084 precedent for
`CT_ShapeProperties`, but a deviation that only lives in the code cannot reach
the AS_BUILT entry. `/implement-feature` says a small clarification proceeds
and is recorded under "Deviations".

## Nitpicks

- `crates/rdocx-oxml/src/paragraph_properties.rs:793`, `w:framePr`,
  `w:textDirection`, `w:textAlignment`, `w:textboxTightWrap` and `w:divId` are
  typed from the `Event::Empty` arm only. The non-empty spelling falls through
  to raw retention at the same schema slot, which is exactly what `w:shd`
  already does in this file.
- `crates/rdocx-oxml/src/paragraph_properties.rs:803`, `w:divId` parses with
  `val.parse()?`, so a malformed value fails the document open rather than
  dropping the element. That matches `w:outlineLvl` three lines below and the
  "fail explicitly instead of becoming zero" posture in
  `docs/hld/04-opc-and-packaging.md`.
- `crates/rdocx/src/paragraph.rs:1779`, `set_border_value` writes `space: 1` on
  every authored edge, so mutating an edge that carried another gap resets it.
  That is the same replacement semantics `set_border_all` has, and
  `set_border_bottom_with_space` remains the explicit-gap door.
- `crates/rdocx/src/paragraph.rs:2072`, `Paragraph::mark` materialises an empty
  `w:rPr`, and `CT_PPr::is_empty` treats a present `rpr` as non-empty, so
  calling `mark()` and setting nothing leaves `<w:pPr></w:pPr>` behind.
  `clear_mark` removes it again.
- `crates/rdocx/src/paragraph.rs:1393`, `border_all` and
  `set_border_bottom_with_space` still replace the whole edge, so they drop the
  attributes `set_border_value` now retains. The plan keeps both as they are so
  no existing caller breaks.

## Contract

The plan's implementation checklist is covered item for item. The nine typed
`CT_PPr` members plus `CT_FramePr` parse, write at their schema slots, and take
part in `is_empty` and `merge_from`. The hard-coded bidi carrier test is now
`ppr_modeled_toggle_present`. `CT_BorderEdge` carries ordered attribute
retention covering the five attributes F-269 named. `w:divId` is typed at slot
31 with a paragraph accessor and paragraph-side assertions only. The facade
carries every named type, every setter triple, `right_to_left` for F-266a, and
the matching `ParagraphRef` readers including the four for already-modeled
state. `crates/rdocx/src/lib.rs:101` exports the new public names. Only the
five HLD files the plan lists are touched.

Two contract points are worth stating because they read as gaps and are not.
"`line_spacing_rule` clearing" is already served by the existing
`Paragraph::clear_line_spacing`, which clears `line_spacing` and `line_rule`
together, so nothing was added. The "`_value` clearers for the four toggles
that already have setters" already existed at
`crates/rdocx/src/paragraph.rs:1266` onwards. Nothing beyond the plan's scope
list was added, and the story takes none of the work the plan hands to F-265,
F-266c, F-267, F-269 or F-270.

## Tests

Every test the plan names exists and is named as the plan names it. The gate
`every_public_paragraph_property_reopens_and_preserves_unrelated_xml` authors
all twenty-two new properties, saves, reopens, reads each one back through
`ParagraphRef`, and requires the four unmodelled `w:pPr` children to survive
byte identical. It cannot pass against reverted code, because the setters and
readers it calls do not exist there, and it fails on a wrong write order
because the retained children are matched byte for byte after serialization.

The unit tests carry their own weight. `frame_properties_round_trip_every_modeled_attribute`
compares the whole serialized element, not a substring.
`modeled_paragraph_children_serialize_in_schema_sequence_order` interleaves
four retained raw children at slots 12, 20, 32 and 16 among the eleven new
typed children and asserts the full order.
`newly_modeled_paragraph_toggles_replay_their_source_carrier` asserts byte
equality for each of the five new toggles and for the reparse.

## Panics

No `unwrap`, `expect`, slice index or unchecked arithmetic was added outside
`#[cfg(test)]`. `Paragraph::tab_stop`, `ParagraphRef::tab_stop`,
`set_tab_stop`, `remove_tab_stop` and `ParagraphRef::border` all return
`Option` or `bool` for an out-of-range index, and
`tab_stops_read_mutate_and_remove_by_index` proves index 3 of a three-element
collection returns `None` and `false` rather than panicking.
`set_outline_level_value` rejects above nine, including `u32::MAX`, without
mutation. The only new arithmetic is `occurrences[slot as usize] - 1`, which
S2 covers.

## OOXML

Child order, prefixes and retention were checked against
`docs/hld/04-opc-and-packaging.md`. Reads go through `is_word_element` and
`is_word_attribute`, so any prefix bound to the Word namespace is accepted and
a foreign same-local element stays unmodelled, which
`paragraph_property_parsing_accepts_alias_and_foreign_namespaces` proves for
both. Writes use the fixed `w` prefix. Every new child is written at its
`ppr_slot_for_name` position, and `write_ppr_with_positioned_raw` still replays
retained raw children against that same table. Typing an element that used to
replay verbatim is the stated hazard, and the five new toggles join the carrier
machinery rather than losing an unowned attribute.

## Structure

No new trait, generic parameter, `Box<dyn>`, forwarding wrapper, feature flag,
module or file. `PPR_MODELED_TOGGLES` plus `ppr_toggle_slot`,
`ppr_toggle_field_mut` and `ppr_modeled_toggle_present` replace a hard-coded
`w:bidi` special case in three places with one table, which reduces the cases a
reader must hold. The facade enums are the usual "make the invalid state
unrepresentable" trade the repository already takes for `CellTextDirection`.
No new file under any `tests/` directory: the integration tests are one module
inside the existing entry point and the regression tests are flat, matching
each file's own convention.

## Not found

- `correctness`. No wrong slot, no inverted condition, no off-by-one in the
  occurrence bookkeeping. The `effective_ppr_raw_position` generalisation keeps
  the carrier comparison inside one slot.
- `panics`. Nothing beyond S2's latent case.
- `ooxml`. No dropped subtree, no prefix leak, no schema-order violation.
- `contract`. No surface beyond the plan's scope list.
