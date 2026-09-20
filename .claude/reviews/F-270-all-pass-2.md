# F-270, all, pass 2

**Reviewed**: the uncommitted working tree on `work/f-270-claude` after pass 1
remediation, 20 tracked files plus `crates/rdocx-oxml/src/web_settings.rs`,
roughly 4600 added and 800 removed lines.
**Verdict**: 0 defects, 0 smells, 3 nitpicks

## Defects

None.

D1 is fixed at `crates/rdocx-oxml/src/settings.rs:604`. A group entry is
cleared immediately when the element that opened it was self-closing, so its
prefix scope cannot reach a sibling.
`a_self_closing_group_does_not_leak_its_namespace_scope` fails if the guard is
removed.

D2 is fixed at `crates/rdocx-oxml/src/web_settings.rs:110`. The loop now tests
`depth == 1` first and only enters the division branch for a depth below a live
`w:divs`. `a_self_closing_divs_element_does_not_swallow_later_children` fails if
the ordering is restored.

## Smells

None.

S1 is fixed. `to_xml` in both models now emits through a local `emit` closure
over `Writer::get_mut`, and the three group elements go through `wrap_group`,
which builds them with `BytesStart` and `BytesEnd` like every other element in
the file. No hand-written element literal remains.

## Nitpicks

- `crates/rdocx-oxml/src/settings.rs:1461`, `remove_remove_personal_information`
  and `remove_remove_date_and_time` read badly, and are kept because they are
  the honest mechanical result of `remove_` applied to the spec names
  `w:removePersonalInformation` and `w:removeDateAndTime`.
- `crates/rdocx-oxml/src/settings.rs:3183`, `rewrite_child_in_raw` sets
  `inserted = true` in the depth-one `End` arm, where the value is never read
  again. It matches the shape of `rewrite_ordered_child` beside it.
- `crates/rdocx/src/document.rs:18745`, `mail_merge_settings` carries a suffix
  only because `Document::mail_merge` already names the field-merge operation
  in `crates/rdocx/src/field.rs:1435`.

## Not found

- **correctness**: the occurrence tally is the single source for both
  diagnostics and the removal ambiguity guard, so "absent", "duplicated" and
  "malformed" cannot disagree between them. `assign` clears a slot on the second
  occurrence and keeps it clear, so a third well-formed occurrence cannot
  resurrect a value. `default_tab_interval_pt` filters a zero interval, so no
  division by zero can reach `resolve_tab_width`.
- **contract**: every checklist item is present and nothing beyond the plan was
  added. The five accessors F-269 consumes exist as getter, setter and remover
  at both the model and facade levels. No `CT_PPr` field and no paragraph
  accessor were added, which is F-264's half.
  `scripts/docx_authoring_conformance.py` is unchanged and
  `new_word_compatible_package` is untouched, so fresh profiles gain no web
  settings part.
- **panics**: no new `unwrap` or `expect` on parsed input outside
  `#[cfg(test)]`. No new indexing, slicing or unchecked arithmetic on untrusted
  input.
- **ooxml**: `xsd:sequence` order holds at the settings, `w:compat`,
  `w:mailMerge` and `w:webSettings` levels. Reads accept in-scope Word aliases
  and reject foreign lookalikes. Writes use the fixed `w:` prefix. Every
  unmodelled subtree is replayed from `source_xml` rather than reconstructed.
- **structure**: no new trait, no `Box<dyn>`, no new feature flag, no new crate.
  One new module, approved in the S74 consolidated design round. The three
  duplicated order tables collapsed into one `SETTINGS_ORDER` and one
  `element_follows` predicate, and `rewrite_automatic_hyphenation` and
  `setting_follows_math_properties` were deleted in favour of the shared
  `rewrite_ordered_child` walk.
- **tests**: the hash harness reports 49 entries match. Reverting either parser
  fix, the closed-set constant, the diagnostics, the removal guard, the schema
  ordering, or the layout wiring fails a named test.

## Gate results at this pass

- `cargo fmt --all --check`: clean.
- `cargo clippy --workspace --all-targets --all-features --exclude rdocx-py
  --exclude rpptx-py -- -D warnings`: clean.
- `cargo test -p rdocx-oxml`: 520 passed, 0 failed.
- `cargo test -p rdocx`: 463, 257, 492 and 2 passed, 0 failed.
- `cargo test -p oxml-layout`: 105 and 3 passed, 0 failed.
- `cargo test -p oxml-layout --no-default-features`: 103 and 3 passed, 0 failed.
- `python3 scripts/hash_harness.py --check`: 49 entries match.
- `python3 scripts/prose_check.py`: 0 violations.
