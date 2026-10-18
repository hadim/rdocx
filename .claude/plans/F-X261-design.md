# F-X261, Resolve Word package preservation and inspection contributions

**Status**: approved
**Sprint**: S95.3
**Size**: L
**Estimate**: 6 to 9 developer-days, provisional
**Depends on**: F-X238, F-X220, F-X241

## Problem

Issues 387, 390 and 391 are open against main `7496a81c`. No candidate covers Issue 387. `CustomProperties` in `crates/rdocx-py/python/rdocx/_rdocx.pyi` supports `get`, `set`, `delete`, `__setitem__` and `__contains__`. It has no `__iter__`, `__len__`, `items` or name lookup. PowerPoint already iterates `PyCustomProperties` from `crates/rpptx-py/src/presentation.rs`.

Issue 390 has six rows. `set_style` and `add_style` drop `w:latentStyles` and other unmodelled style-root children. Removing a picture paragraph leaves its image relationship and media part. A plain TrueType part with content type `application/x-font-ttf` and a null `w:fontKey` opens and saves, then `fonts`, `render_pages` and `to_pdf` fail. A header relationship to a non-XML target is ignored by `validate`. `Comment.resolved` returns `bool` from `crates/rdocx/src/comments.rs` and the Python stub, and a missing `commentsExtended` entry becomes `false`.

The issue also names a control-owner failure that the reporter could not reduce, and PR396 discloses an existing remove, reinsert and `set_header_image` regression. Those two observations are not in the six reduced rows.

Issue 391 has five rows. `replace_xml` rejects a declared `w14` root attribute unless the prefix is written before the declaration is considered. `StoryItem.xml` is not standalone, while `Paragraph.xml` is. There is no read-only part XML accessor. Nothing reads or writes `w:pPrDefault`. A text-box story item can be listed and its text replaced, but no `Paragraph` handle reaches it.

PR395 head `5d481eb53f00926896f9de632115f4cb245fc941` and PR396 head `c71708541397f96128dea2dc8b9c1a21c7da498a` are the 2026-10-18 identities. Each is one archive-measurement commit ahead of functional parents `c36bc80c1c23a7fa32c02ba7f1d26ef53b666939` and `7d573cc67c6ca7e7884608224a89bba9bbb4a587`. The measurement commits touch README files and `scripts/readme_doctests.py` only. Both pulls are candidates. They overlap on `crates/rdocx/src/document.rs`, `crates/rdocx-py/src/document.rs` and `docs/hld/10-bindings-spec.md`, so they are one implementation, not two waves.

Current source is `67423b6b` plus the uncommitted F-X259 diff, which already edits `crates/rdocx/src/document.rs`. This story starts after that diff is integrated.

## Spec reference

- `docs/hld/04-opc-and-packaging.md`, checked Word source ownership, relationship reachability and embedded fonts.
- `docs/hld/03-architecture.md`, shared Word operations and owned publication.
- `docs/hld/10-bindings-spec.md`, Word properties, paragraph handles, comments and checked XML.
- `docs/hld/12-testing-strategy.md`, Binding tests, Word checked XML qualification and embedded font acceptance.
- `docs/hld/14-development-backlog.md`, F-X261.

## Approach

Inspect the functional parents and reimplement the accepted rows on the integrated tree. Do not merge either pull.

Style mutation keeps unmodelled root children, including `w:latentStyles`, in schema order ahead of `w:style`. Empty style records and foreign attributes on those children round-trip byte for byte through `set_style`, `add_style` and `insert_document`.

Picture removal drops a media part and its relationship only when the candidate package proves the part unreachable. Shared targets, pop then reinsert, and caller-embedded unused media stay. The disclosed header-image reinsert sequence gets its own constructed package and must pass. The unreduced control-owner report stays unqualified. The story adds a diagnostic that names the owner route it could prove, and it does not claim that private document repaired.

Plain supported font parts are listed, removed and rendered through the existing embedded-font provider. An unsupported MIME still refuses, and the error names the part and the content type. Native fallback is measured at the final gate. It is not assumed from a valid header.

`validate` reports a header or footer relationship whose target is not a header or footer part, including the part name and content type. Supported variants stay usable and still save.

`CommentRef::resolved` and the Python and CLI projections grow an unknown state. Rust returns `Option<bool>`. Python returns `bool | None`. A freshly added comment that writes an extension entry stays explicitly unresolved, which is `False`. A package with that part removed reports unknown, which is `None`. CLI treats only `Some(true)` as resolved. This is a public break on the pre-1.0 family.

`replace_xml` binds a root attribute when that prefix is declared on the same root, in either declaration order. An unbound prefix still refuses, and the target bytes do not change. Table and cell `replace_xml` use the same rule. `StoryItem.xml` gains the namespace declarations `Paragraph.xml` already writes, for every supported physical owner. The existing native exact-slice API stays.

Part inspection is read-only. Python `part_names() -> tuple[str, ...]` and `part_xml(part_name: str) -> str`. Rust `part_names(&self) -> Result<Vec<String>>` and `part_xml(&self, part_name: &str) -> Result<String>`. Names are package paths with a leading slash. Missing and binary parts return an error that names the part and content type. The read sees unsaved staged parts and returns no mutable handle. There is no WASM part API.

Default paragraph format uses Python `DefaultParagraphFormat` with `space_before`, `space_after` and `line_spacing`, carried by the existing `Length` type. Rust exposes `default_paragraph_properties(&self) -> Option<&CT_PPr>` and `set_default_paragraph_properties(&mut self, Option<CT_PPr>) -> Result<()>`. An omitted Python keyword preserves the existing value. A keyword set to `None` clears only that property. Clearing the entire native paragraph-default object preserves `w:rPrDefault` and unrelated opaque source. Malformed lengths do not publish.

A story-item paragraph handle writes the physical text box and its qualified alternate in one transaction. If either copy cannot be updated, neither changes. A stale handle still fails. Custom properties gain `__iter__`, `__len__`, `items` and `__getitem__(name)` on the existing class, in native part order. An absent part has length 0. There is no second collection type.

No new crate, module or file. Issue 387 extends the existing `CustomProperties` class. It does not add a second collection.

## Consumer map

| Rule | Entry points and outputs | Shared provider and caller policy | Lifecycle stages compared |
|---|---|---|---|
| Custom properties can be listed | Python `Document.custom_properties` iteration, length, items and name lookup | Existing native `custom_properties()` order. PowerPoint is the behaviour reference, not a shared type | Absent part, imported part, edit then iterate, save and reopen |
| Unmodelled style root bytes survive | `set_style`, `add_style`, `insert_document`, package writer | Existing styles raw slots and schema order | Import, mutate, save, reopen |
| Picture media follows reachability | `remove_content`, pop and reinsert, header image | One pruning rule, shared with F-X260 range removal | Shared target, orphan, caller-embedded unused part |
| Comment resolved distinguishes unknown | Rust `CommentRef`, Python `Comment`, CLI comment listing | One optional value. CLI collapses only for display | Authored comment, stripped extension, explicit true and false |
| Checked XML and part reads agree | `replace_xml`, `StoryItem.xml`, `part_xml`, default paragraph format, text-box paragraph | Existing checked physical source. No second XML owner | Unsaved edit, standalone parse, save and reopen, paired text boxes |

## Acceptance criteria

| ID | Input or setup | Expected observation or refusal | Consumers | Local test | Final gate or pending evidence |
|---|---|---|---|---|---|
| A1 | Imported properties in package order, an empty document, then one `set` | Iteration, `len`, `items` and `[name]` agree. Absence iterates empty. Order is the part order | Python `CustomProperties`. Native slice is the source | `issue387_custom_properties_are_enumerable` | Installed `rdocx` suite |
| A2 | Styles root with `latentStyles`, an opaque child and an empty style | `set_style`, `add_style` and `insert_document` keep the raw bytes and the schema order | Native styles writer and Python style API | `issue390_style_edits_preserve_unmodelled_root_source` | Hash harness unchanged |
| A3 | Shared image, unique image, orphan media, pop then reinsert | Only a proved-unreachable owned part is dropped | `remove_content` and package relationships | `issue390_removed_picture_media_has_proved_reachability` | Final package corpus |
| A4 | Plain TTF and OTF, a key variant, an unsupported MIME | Listing, removal and the layout reader agree. The bad MIME names the part | `Document.fonts`, render and PDF entry | `issue390_plain_fonts_share_decoding_and_diagnostics` | Deterministic font render at final sprint |
| A5 | Header relationship to `application/octet-stream`, plus a valid header | `validate` names the bad target. The valid header still renders | `validate` and header stories | `issue390_invalid_story_targets_are_diagnosed` | Final validate suite |
| A6 | Authored comment, extension removed, explicit open, explicit resolved | Unknown, false and true stay distinct in Rust and Python. CLI lists only explicit true as resolved | Rust, Python, CLI | `issue390_comment_resolution_preserves_unknown_state` | Installed suite and CLI integration |
| A7 | `w14:paraId` declared before or after the attribute, on paragraph, table and cell. A second document with an unbound prefix | The declared prefix binds. The unbound edit refuses and the bytes match the input | Python `replace_xml` | `issue391_root_attributes_use_declared_namespace_scope` | Hash harness unchanged |
| A8 | Story items for paragraph, table, cell, header and text box | `StoryItem.xml` parses alone. The native exact slice stays available | Python story item and native snapshot | `issue391_story_item_snapshots_are_namespace_complete` | Installed suite |
| A9 | Unsaved style edit, a missing name, a binary part, a header part | Names use one normalized form. Binary and missing names refuse. XML matches the staged part | Python `part_names` and `part_xml` | `issue391_part_inspection_reflects_current_package` | No WASM part API |
| A10 | An omitted keyword, one keyword set to `None`, a typed spacing value, a clear of the whole native paragraph-default object, and a malformed length | The omitted keyword keeps its value. `None` clears only that property. The whole-object clear preserves `rPrDefault` and unrelated opaque source. The typed value writes that property. Malformed input does not publish | Python default paragraph format and Rust `CT_PPr` | `issue391_default_paragraph_format_is_owned_and_atomic` | Hash harness unchanged |
| A11 | VML text box, DrawingML text box, and a qualified pair | The approved handle updates both copies or refuses both. A stale handle still fails | Python `Paragraph` on a story item | `issue391_textbox_paragraph_handles_preserve_physical_custody` | Installed suite |
| A12 | Constructed remove, reinsert and `set_header_image`. The unreduced control report has no fixture | The constructed sequence keeps a reachable image. The control row records the diagnostic and stays unqualified | Package graph and the progress record | `issue390_asset_reinsertion_and_unreduced_control_diagnostic` | No closure of the private report |

Issue 387 is A1. Issue 390's six reduced rows are A2 through A6, and A12 holds the two disclosures. Issue 391's five rows are A7 through A11.

## Test matrix

| Case | Modes or state combined | Expected difference or agreement | Wrong implementation rejected |
|---|---|---|---|
| C1 | Empty, one and many properties or style children | Length and order agree before and after one edit | A snapshot that ignores a later `set` |
| C2 | Shared versus unique media, valid versus binary parts | Shared bytes stay. Binary part XML refuses | Deleting every image part, or decoding a font as XML |
| C3 | Explicit false versus missing extension | Python `False` and `None` differ | Mapping both to `False` |
| C4 | Prefix declared first versus attribute first | Both bind. An unbound prefix still refuses | Accepting every `w14` attribute |
| C5 | One text box versus a qualified pair | Both copies change, or neither does | A silent edit of one copy |
| C6 | Current tree, candidate parent, measurement head | Same fixtures. The measurement commit is not a behaviour change | Treating the new head as an unreviewed feature commit |

## Rejected alternatives

- Merging PR395 and PR396 onto the dirty F-X259 tree.
- A new ZIP walker for part reads. The package already has part bytes.
- Closing Issue 390 because six reduced tests pass.
- A binding-only comment flag that leaves Rust returning `bool`.

## Test plan

**Test gate**: regression. Each reduced A-ID has an old-source semantic RED and a repaired GREEN. A12's unreduced input has no RED fixture and stays unqualified.

| Timing | Category | Test | Asserts |
|---|---|---|---|
| local scoped | regression | A1 through A11 in the existing Python and native entrypoints | The row observation, including atomic refusal |
| local scoped | regression | A12 constructed package | Reachable header image survives. Diagnostic text is stored for the missing fixture |
| final sprint | rider | installed products, deterministic font render, full hash harness, both WASM checks | Word 49 hash and seven pixel baselines unchanged |

Add tests to `crates/rdocx-py/tests/test_core.py`, `crates/rdocx-py/tests/test_formatting_tables.py`, `crates/rdocx/tests/regression_test.rs` and `crates/rdocx/tests/integration_test.rs`. Do not add a test binary. Font bytes used by A4 must already be licensed in the repo or be a checked-in minimal fixture with a licence note. Do not download a Google Docs sample.

## HLD impact

- `docs/hld/03-architecture.md`
- `docs/hld/04-opc-and-packaging.md`
- `docs/hld/08-rendering-spec.md`, only if the plain-font reader changes a documented fallback
- `docs/hld/10-bindings-spec.md`
- `docs/hld/12-testing-strategy.md`
- `docs/hld/14-development-backlog.md`

## Risk routing

- Contributed PR or new issue acceptance. Pin the refreshed heads and both functional parents. Keep A12 unreduced.
- Any parser or serialiser. Style child order, prefix-tolerant `replace_xml`, and byte-identical latent styles.
- Public API of a published crate. `resolved` becoming optional is the break. Part inspection and default paragraph format are additive. Final `cargo publish --dry-run` and crate size check.
- WASM or PyO3 bindings. Final wasm check for `rdocx-wasm`. Python tests stay excluded from the workspace test binary.
- Shared semantic behavior with multiple consumers. Rust, Python and CLI comment state, and both text-box copies.
- Grammar, scope, compatibility mode or empty collection. Empty properties, missing extension, absent part, binary part.
- Bundled fonts, only if A4 adds a font file. The licence file must match the bytes. Prefer an existing licensed face.
- Layout and deterministic font mode, only for the A4 render observation at the final gate.

Unit constructors stay truncating. This story does not round twips.

## Hash harness

Unchanged Word 49 hash and seven pixel baseline. A paint change is a stop, not a re-record. F-X262 owns PowerPoint paint.

## Implementation checklist

- [ ] Re-read both functional parents after F-X259 is integrated.
- [ ] Record old-source RED for every reduced row.
- [ ] Implement in the existing files and keep one pruning policy with F-X260.
- [ ] Update the HLD list and obtain a zero-finding microscope.
- [ ] Leave installed, font, wasm and hash evidence for the final union.

## Approved decisions

- Q1. Rust `CommentRef::resolved(&self) -> Option<bool>` and Python `bool | None`. Missing extension is `None`. An authored comment that still writes an extension stays `False`. CLI prints resolved only when the value is `Some(true)`.
- Q11. Python `part_names() -> tuple[str, ...]` and `part_xml(part_name: str) -> str`. Rust `part_names(&self) -> Result<Vec<String>>` and `part_xml(&self, part_name: &str) -> Result<String>`. Names are package paths with a leading slash. Missing and binary parts return an error that names the part and content type. The read sees unsaved staged parts and returns no mutable handle. There is no WASM part API.
- Q12. Python `DefaultParagraphFormat` exposes `space_before`, `space_after` and `line_spacing` through the existing `Length` carrier. Rust exposes `default_paragraph_properties(&self) -> Option<&CT_PPr>` and `set_default_paragraph_properties(&mut self, Option<CT_PPr>) -> Result<()>`. An omitted keyword preserves the existing value. A keyword set to `None` clears only that property. Clearing the entire native paragraph-default object preserves `w:rPrDefault` and unrelated opaque source. Malformed lengths do not publish.
- Q13. A story-item paragraph handle writes the physical text box and its qualified alternate in one transaction. If either copy cannot be updated, neither changes. A silent one-copy edit is refused. A stale handle still fails.
- Q14. A2 through A6 and the constructed A12 sequence can pass. The private unreduced control-owner document stays open and unqualified. A diagnostic is not a repair. Six reduced checks do not close Issue 390.
- Q15. The existing `CustomProperties` class gains `__iter__`, `__len__`, `items` and `__getitem__(name)` in native part order. There is no second type. An absent part has length 0. Empty, imported, post-edit, and save and reopen stay in A1.

No issue or pull request closes from this plan.
