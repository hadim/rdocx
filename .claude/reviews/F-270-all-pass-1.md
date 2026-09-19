# F-270, all, pass 1

**Reviewed**: the uncommitted working tree on `work/f-270-claude`, 20 tracked
files plus the new `crates/rdocx-oxml/src/web_settings.rs`, roughly 4540 added
and 790 removed lines.
**Verdict**: 2 defects, 1 smell, 3 nitpicks

## Defects

### D1, a self-closing settings group leaks its namespace scope onto its siblings
`crates/rdocx-oxml/src/settings.rs:600`

`absorb_top_level` records a group entry for `w:compat`, `w:docVars` and
`w:mailMerge` without checking whether the element was a `Start` or an `Empty`.
An `Empty` group such as `<w:compat xmlns:z="...w NS..."/>` never raises the
depth, so the matching `End` arm never runs and the group entry stays live.
Every following top-level sibling then inherits the group's prefix scope, so a
`<z:view z:val="print"/>` written after the self-closing group is accepted as
the modeled `w:view` even though `z` is out of scope there. The correct reading
is that the alias is unbound outside the element that declared it.

### D2, a self-closing `w:divs` swallows every later web settings child
`crates/rdocx-oxml/src/web_settings.rs:112`

The parse loop tests `divs_depth.is_some()` before it tests `depth == 1`. A
self-closing `<w:divs/>` sets `divs_depth` and, having no `End` at that depth,
never clears it. Every subsequent top-level child then takes the division
branch instead of `absorb`, so a part shaped
`<w:webSettings><w:divs/><w:allowPNG/></w:webSettings>` projects no
`allowPNG` at all. The existing division test used a non-empty `w:divs`, so
nothing caught it.

## Smells

### S1, `to_xml` writes group elements as raw byte literals
`crates/rdocx-oxml/src/settings.rs:1735`

Holding `let out = writer.get_mut()` across the whole serializer forces
`out.extend_from_slice(b"<w:compat>")` and three more hand-written element
literals. Every other element in the file goes through `BytesStart` and
`Writer`, so a reader scanning for emitted elements misses these, and a future
attribute on one of the three groups would be written by hand. The same shape
appears in `crates/rdocx-oxml/src/web_settings.rs`.

## Nitpicks

- `crates/rdocx-oxml/src/settings.rs:1447`, `remove_remove_personal_information`
  and `remove_remove_date_and_time` read badly, but they are the honest
  mechanical result of `remove_` applied to `w:removePersonalInformation`.
  Renaming either half would break the one-to-one mapping to the spec name.
- `crates/rdocx-oxml/src/settings.rs:3097`, `rewrite_child_in_raw` sets
  `inserted = true` in the depth-one `End` arm, where the value is never read
  again.
- `crates/rdocx/src/document.rs:18745`, `mail_merge_settings` carries a suffix
  only because `Document::mail_merge` already names the field-merge operation.

## Not found

- **contract**: every `## Implementation checklist` item is present. The closed
  `SUPPORTED_SETTINGS` has the thirty-one names the plan lists, in the plan's
  order. `CompatibilityOption` covers the complete closed `CT_Compat` on-off
  set. `w:dataSource`, `w:headerSource`, `w:odso`, `w:frameset` and `w:divs`
  stay preservation-only. No Python, WASM or CLI entry point was added.
  `new_word_compatible_package` is untouched and
  `scripts/docx_authoring_conformance.py:350` is unchanged. No `CT_PPr` field
  and no paragraph accessor were added, which is F-264's half.
- **panics**: no new `unwrap` or `expect` on parsed input outside `#[cfg(test)]`.
  The three `expect` calls in the facade staging helpers are on a candidate this
  code just populated. No new indexing or slicing on untrusted input. Integer
  parsing is `parse::<T>().ok()`, never unchecked.
- **ooxml**: `xsd:sequence` order is written at all four levels, proved by
  `settings_children_serialize_in_schema_sequence_order`. Reads are
  prefix-tolerant and writes use the fixed `w:` prefix. Unmodelled subtrees stay
  in `source_xml` and are never re-serialized from the typed model.
- **structure**: no new trait, no new generic parameter beyond `stage_*_removal`
  and `assign`, which are instantiated more than twenty ways today. No
  `Box<dyn>`. No new feature flag. One new module, explicitly approved in the
  S74 consolidated design round.
- **tests**: reverting either parser change fails a named test. The hash
  harness reports 49 entries match.
