# F-286, Rich, repeating, and typed content controls

**Status**: approved
**Sprint**: S95.3
**Size**: L
**Depends on**: F-285

## Problem

`SdtType` already names rich text, plain text, picture, checkbox, combo box, drop-down, date, group, repeating section and repeating item. The value facade can fill a control that already has the right payload. `CT_SdtPr::prepare_text_value` accepts a Gregorian date in `yyyy-MM-dd` or `dd/MM/yyyy`, and a drop-down value that matches one list item. `checkbox_state` maps the existing true and false vocabulary to a glyph. Picture and group refuse a text value. None of that creates the type element, the list items, the date metadata or a repeating item.

F-285 creates only a rich or plain shell and refuses every other kind. This story adds the missing payloads. It cannot start until F-285 has passed its dependency-prefix checkpoint. F-287 starts only after this story leaves the shared files.

DOCX-053 is still partial. The differential gate is a final observation against pinned Word structure and behaviour under replacement and repetition. It is not a worker oracle. A package this workspace generates is not that observation. Missing authentic Word provenance, version and input identity stays pending.

## Spec reference

- `docs/hld/14-development-backlog.md`, F-286.
- `docs/hld/02-scope-and-non-goals.md`, DOCX-053.
- `docs/hld/03-architecture.md`, content-control type and shell identity.
- `docs/hld/10-bindings-spec.md`, value replacement.
- `docs/hld/12-testing-strategy.md`, differential controls for content controls.

## Approach

Add `Document::set_content_control_type(location, payload)` on a shell from F-285. F-285's create signature does not grow a payload enum.

`ContentControlTypePayload` is one of:

- `RichText` and `PlainText`, with no children.
- `Date { format }`, where `format` is `yyyy-MM-dd` or `dd/MM/yyyy`, calendar is Gregorian, and storage is the existing `storeMappedDataAs` text mapping.
- `CheckBox`, using the existing checked and unchecked glyph slots.
- `ComboBox { items }` and `DropDownList { items }`, each item a display and value pair.
- `Picture { relationship_id }`, one image relationship the document already owns.
- `Group`.
- `RepeatingSection`, which requires one child repeating item that holds the template content.

The setter replaces the type element and keeps tag, alias, id, lock, appearance, placeholder and unmodelled raw slots. An invalid payload, a duplicate list value, an unknown date format or a picture relationship the part does not own refuses and does not publish.

Replacement uses the existing value methods. This story adds the cases those methods cannot see until the payload exists. Repeating a section clones the template item with a fresh id and appends it inside the section. Zero items refuses. One item repeats to two. Many items append one more. Removing the last item refuses, so a section never becomes empty.

Picture replacement, if any, changes the relationship id and refuses a text string. Group still refuses text. Those rules stay in `prepare_control_value`.

No new file. Python adds the payload argument on the same document methods F-285 exposes and keeps the frozen `ContentControl` snapshot. WASM and CLI stay out.

Local tests construct the replacement and repetition observations in source. They do not add a binary fixture. Those observations are not the Word differential. Differential evidence needs authentic Word provenance, the Word version and the input identity. A package this workspace writes, including a checked-in copy of its own output, is not a Word oracle. Until that evidence exists, the differential gate stays pending.

## Consumer map

| Rule | Entry points and outputs | Shared provider and caller policy | Lifecycle stages compared |
|---|---|---|---|
| Type payload and value fill agree | `set_content_control_type` and the existing tag, alias and bind setters | `CT_SdtPr::prepare_text_value` and `checkbox_state` | Set type, fill, save, reopen |
| Lists reject an unknown choice | Combo and drop-down fill | One item table. Display and value are both accepted as the caller key | Known value, unknown value, ambiguous value |
| Repeating section grows by cloning the template | Section and item shells from F-285's transaction | Fresh id, dropped `w14:paraId` | Zero, one and many items |
| Picture uses an owned relationship | Picture payload and the existing image embed | No new media allocator | Present relationship, missing relationship, text value |

## Behavior contracts and evidence

| Surface or placement | Exact behavior and boundary examples | Primary source identity and location | Local discriminator or pending evidence |
|---|---|---|---|
| Type setter | `set_content_control_type(location, payload)` replaces the type element and keeps tag, alias, id, lock, appearance, placeholder and unmodelled raw slots. An invalid payload does not publish. Citation, equation, bibliography and document-part kinds stay refused. | Q17. Backlog F-286 sentence. `SdtType` names in `crates/rdocx-oxml/src/content_control.rs` lines 55 to 77. | A1. Local tests do not claim a Word match. |
| Checkbox vocabulary | `true`, `1` and `yes` check. `false`, `0` and `no` uncheck. `maybe` refuses and does not publish. | `prepare_control_value` at `crates/rdocx/src/content_control.rs` lines 81 to 88. | A3. |
| Date, lists, picture and group | Date uses the existing Gregorian `yyyy-MM-dd` and `dd/MM/yyyy` forms. Lists reject duplicate, unknown and ambiguous values. Picture requires an already-owned relationship and refuses text. Group refuses text. | Q17. Existing `prepare_text_value` and the picture and group refusal at lines 102 to 106. | A2, A4, A5 and A6. |
| Repeating section and item | A repeating section carries one template item. Zero items refuses. One item repeats to two. Removing the last item refuses. `w15:repeatingSectionItem` is authored as that template child. | Q17. The element names are `w15:repeatingSection` and `w15:repeatingSectionItem`. | A7. |
| Word differential | Every supported control must match pinned Word structure and behaviour under replacement and repetition. A package this workspace generates is not that evidence. | Backlog test gate. `.claude/skills/differential-testing.md`, native provenance rules. | Pending. No authentic Word producer, version or input identity is in the repo. |

## Dependency readiness

The formal edge is F-285. Its provider contract is approved. `BACKLOG.md` still records F-285 as `pending`, so this story is not implementation-ready until the F-285 prefix checkpoint is integrated. Reviewed status on F-285 does not unblock this story. The Word differential stays pending after implementation and is not a local gate.

## Acceptance criteria

| ID | Input or setup | Expected observation or refusal | Consumers | Local test | Final gate or pending evidence |
|---|---|---|---|---|---|
| A1 | Set each supported kind on an F-285 shell, then reopen | The type element and its children follow the schema order this story writes. Tag and raw slots remain. The Word structure comparison stays pending until authentic provenance exists | Native setter and Python `Document` | `typed_content_control_payload_round_trips` | Authentic Word structure diff. Pending while provenance, version and input identity are missing |
| A2 | Date `2026-10-18` with both formats, plus `31/02/2026` and a non-Gregorian calendar marker | The valid date stores the existing display and full date. The two invalid inputs do not publish | Existing date preparer | `typed_date_control_rejects_bad_calendar_values` | Authentic Word date comparison. Pending while provenance is missing |
| A3 | Checkbox `true`, `false`, `yes`, `no`, `1`, `0`, and `maybe` | The first six follow today's glyph mapping. `maybe` refuses | Existing checkbox path | `typed_checkbox_control_uses_existing_vocabulary` | Authentic Word checkbox comparison. Pending while provenance is missing |
| A4 | Drop-down and combo with two items, a repeated value, and a fill that names the display | Unique items round-trip. The repeated value refuses. Fill selects the matching item and writes its value | List payload and `prepare_text_value` | `typed_list_control_rejects_unknown_and_ambiguous_values` | Authentic Word list comparison. Pending while provenance is missing |
| A5 | Picture with an embedded image relationship, a missing id, and a text fill | The owned id round-trips. The other two refuse | Picture payload and value facade | `typed_picture_control_requires_an_owned_image` | No render change claimed |
| A6 | Group, then a text fill | The group shell round-trips and the text fill refuses | Value facade | `typed_group_control_refuses_text` | Authentic Word group comparison. Pending while provenance is missing |
| A7 | Repeating section with zero, one and three items. Repeat once. Remove down to one and then try the last | Zero refuses. One becomes two. Three becomes four. The last item cannot be removed | Section transaction | `repeating_section_clones_the_template_item` | Authentic Word repeating comparison. Pending while provenance is missing |

Citation, equation, bibliography, document-part list and document-part object stay readable and are not authored here. A setter call for those kinds refuses. That keeps the backlog list exact.

## Test matrix

| Case | Modes or state combined | Expected difference or agreement | Wrong implementation rejected |
|---|---|---|---|
| C1 | Rich and plain shells converted to each typed kind | Unmodelled `sdtPr` children survive the conversion | A setter that rebuilds properties and drops raw slots |
| C2 | Empty item list, one item, many items | Only one and many can be filled or repeated | A repeating section with no template |
| C3 | Display key versus value key on a list | Both select the same item when they are unique | A filler that compares display only |
| C4 | Save and reopen between set and fill | The payload is still there, and fill does not reset it | A fill that depends on in-memory state the package lost |
| C5 | Local replacement and repetition versus the Word differential | Local tests observe child order, refusal and repetition counts. They do not certify Word. Missing authentic provenance stays pending | A test that treats generated XML as the Word oracle |

HTML export and layout already walk `sdtContent`. They are not given a second type policy. A picture control's image bytes are the existing drawing, so this story does not change the renderer.

## Rejected alternatives

- A second value engine beside `prepare_text_value`.
- Authoring citation, equation and bibliography in this story. They are outside the backlog sentence.
- Running Word on every worker iteration. The differential is final evidence, and only when provenance, version and input identity are recorded.
- Treating a source-built package, or a checked-in copy of this workspace's own output, as the Word oracle.
- Adding a binary fixture. The repository keeps fixtures in source, and the external corpus stays outside the published crates.
- Creating type payloads in F-285. That story refuses them.

## Test plan

**Test gate**: differential. Every supported control matches the pinned Word structure and behaviour under replacement and repetition.

Local tests construct those replacement and repetition observations in source and do not add a binary fixture. They are not the Word differential. Differential evidence must name authentic Word provenance, the Word version and the input identity. Generated output is not a Word oracle. Missing evidence stays pending.

| Timing | Category | Test | Asserts |
|---|---|---|---|
| local scoped | regression | A1 through A7 | Child order, refusal, repetition counts. These rows do not claim a Word match |
| local scoped | regression | Existing `prepare_text_value` and checkbox tests | Date and list rules did not move |
| final sprint | oracle | Authentic Word structure and behaviour under replacement and repetition | Provenance, version and input identity recorded. Pending until that evidence exists |

Tests join the F-285 module in the existing `rdocx` integration entrypoint.

## HLD impact

- `docs/hld/02-scope-and-non-goals.md`, DOCX-053
- `docs/hld/03-architecture.md`
- `docs/hld/10-bindings-spec.md`
- `docs/hld/12-testing-strategy.md`
- `docs/hld/14-development-backlog.md`

## Risk routing

- Any parser or serialiser. Type-element child order and raw-slot retention.
- XML creation in multiple parent contexts. A type payload uses the F-285 placement row. An unsupported pairing still publishes nothing.
- Public API of a published crate. Additive payload setter. Final publish dry run.
- WASM or PyO3 bindings. Python is included. Same rider as F-285.
- An external oracle comparison. Authentic Word provenance, version and input identity are required. Missing evidence stays pending. Generated output is not the oracle. Do not add a binary fixture.
- Shared semantic behavior with multiple consumers. Set, fill, save and reopen.
- Grammar, scope, compatibility mode or empty collection. C2 covers zero, one and many repeating items.
- F-285 dependency. Reviewed status is not enough. The prefix checkpoint must have completed.

No hash delta is expected. A fixture that only adds a new document does not move the existing baseline.

## Hash harness

Unchanged.

## Implementation checklist

- [ ] Wait for the F-285 dependency-prefix checkpoint.
- [ ] Keep the Word differential pending until provenance, version and input identity exist.
- [ ] Add RED coverage for one missing payload and one bad list value.
- [ ] Implement the setter on the existing property model.
- [ ] Update the HLD list and obtain a zero-finding microscope.
- [ ] Leave the live Word compare, if the pin must change, to the final union.

## Open questions

| ID | Affected F-IDs and A-IDs | Decision and recommendation | Status and answer/source |
|---|---|---|---|
| Q17 | F-286 A1 to A7 | `set_content_control_type(location, payload)` on an F-285 shell. The supported kinds are rich text, plain text, date, checkbox, picture, combo, drop-down, group, repeating section and repeating item. Citation, equation, bibliography and document-part kinds stay refused. Replacement and repetition observations stay required. Generated output is not a Word oracle. | Answered 2026-10-18 by the user. Checkbox mapping is the code at `content_control.rs` lines 81 to 88. |
| Q19 | F-285, F-286 and F-287 | This story starts after the F-285 prefix checkpoint and finishes before F-287. | Answered 2026-10-18 by the user. |
| Q21 | F-285, and this story's approval | The F-285 method names. This plan has no separate product choice. | Answered 2026-10-18 on F-285. This plan adopts those names. |

No external issue closes from this plan.
