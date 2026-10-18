# F-287, Custom XML stores and data binding authoring

**Status**: approved
**Sprint**: S95.3
**Size**: L
**Depends on**: F-285

## Problem

Bound controls can already update a store that exists. `docs/hld/04-opc-and-packaging.md` says the document custom XML relationship resolves the item part, and that item's properties relationship resolves the `ds:itemID`. The evaluator accepts namespace-aware absolute child paths with optional one-based indexes. Functions, wildcards, descendant axes and general predicates are out of contract. `resolve_custom_xml_part` and `custom_xml_store_item_count` in `crates/rdocx/src/content_control.rs` implement that lookup. `CT_DataBinding` holds `prefix_mappings`, `xpath` and `store_item_id`.

Nothing public creates a store, item properties, schema references or a binding on a control. `crates/rdocx/src/bibliography.rs` writes one `ds:datastoreItem` for its own identity. That is the existing package sequence to copy from, not a second writer with different relationship rules. Unknown bytes in an existing item are already retained by the value updater. A new API must not become raw package surgery.

F-286 does not have to land first. The binding sits on `CT_SdtPr`, which F-285 also edits, so the drafted wave still runs this story after F-286. The dependency edge itself is only F-285. DOCX-054 is unsupported today.

## Spec reference

- `docs/hld/14-development-backlog.md`, F-287.
- `docs/hld/02-scope-and-non-goals.md`, DOCX-054.
- `docs/hld/04-opc-and-packaging.md`, the document-owned custom XML graph.
- `docs/hld/03-architecture.md`, data binding and staged publication.
- `docs/hld/10-bindings-spec.md`, bound value updates.
- `docs/hld/12-testing-strategy.md`, round-trip of bound custom XML and byte-identical unknown stores.

## Approach

Add store and binding methods on `Document`. Do not add a package-surgery API, a crate or a file.

`add_custom_xml_store(item_xml, properties) -> Result<String>` stages a new item part, a properties part, both relationships and the content-type overrides. The item id is a new GUID. The properties root is `ds:datastoreItem` with that `ds:itemID`, schema references from the argument, and no other invented children. Part names use the existing suffix allocator. A conventional name that is already taken is not overwritten. The method returns the store item id.

`set_content_control_binding(location, binding) -> Result<()>` writes `w:dataBinding` on an existing control from F-285. `binding` carries the store item id, an absolute xpath and optional prefix mappings. The xpath is rejected when it contains a function, a wildcard, a descendant axis or a predicate other than a one-based index. The store must already resolve. The selected element must exist or be creatable by the current updater's empty-element rule. A failed check does not publish.

Value updates stay on `set_content_control_value_by_tag` and `bind_content_controls`. This story does not grow a second evaluator. After a public bind, those methods must update the new store and the display together, and a failed control must roll back both.

Unknown sibling bytes in the item stay byte-identical across a value update and across save and reopen. Schema references round-trip and are not resolved against a network or a second schema engine. Prefix mappings are stored and used by the existing path walker.

Bibliography keeps its private writer unless both call sites would copy the same relationship steps. A shared helper is justified only then, because bibliography exists today and this API would be the second caller. If the steps are not the same, this story inlines the public sequence and leaves bibliography alone.

Python adds `Document.add_custom_xml_store` and `Document.set_content_control_binding` and keeps the frozen content-control snapshot from F-285. WASM and CLI gain none.

## Consumer map

| Rule | Entry points and outputs | Shared provider and caller policy | Lifecycle stages compared |
|---|---|---|---|
| Store identity follows the package graph | `add_custom_xml_store` | The relationship walk in `resolve_custom_xml_part`. No fixed filename | Create, reopen, resolve by id |
| Binding uses the existing path subset | `set_content_control_binding` and the value setters | `CT_DataBinding` and the current updater | Bind, fill, refused xpath, rollback |
| Unknown item bytes survive | Value update on a store with an unmodelled sibling | Byte compare of the unrelated span | Update, save, reopen |
| Display and store commit together | Existing tag fill on a newly bound control | One staged package | Success and one rejected sibling control |

## Behavior contracts and evidence

| Surface or placement | Exact behavior and boundary examples | Primary source identity and location | Local discriminator or pending evidence |
|---|---|---|---|
| Store creation | `add_custom_xml_store(item_xml, schema_uris) -> Result<String>` returns the new item id. Part names come from the existing preferred-name allocator. An occupied conventional name is not overwritten. The properties root is `ds:datastoreItem` with `ds:itemID`, then `ds:schemaRefs`, then one `ds:schemaRef` per URI. Schema URIs are stored and not fetched. The id shape is the braced uppercase form already allocated by bibliography, `{00000000-0000-4000-8000-############}`. | Q18. `create_collection` in `crates/rdocx/src/bibliography.rs` lines 1795 to 1846 writes the same relationship and content-type steps and an empty `ds:schemaRefs`. | A1. |
| Shared helper | The relationship steps match bibliography, so one private helper in `rdocx` allocates the item part, properties part, relationships and content types. Bibliography keeps its own item bytes. The helper does not move into `oxml-*`. | Q18, resolved by reading both call shapes. | C5. Bibliography bytes stay unchanged. |
| Binding xpath | Absolute child paths only. A leading `//`, an empty step, a function, a wildcard, a descendant axis or a predicate other than a one-based index refuses and publishes nothing. `/ns:root/ns:child` and `/ns:root/ns:child[1]` are accepted when the prefix is mapped. | `parse_xpath` and `parse_xpath_segment` at `crates/rdocx/src/content_control.rs` lines 910 to 990. | A4. |
| Value update | Existing tag and bind setters update the store and the display together. A failed sibling rolls both back. Unknown sibling bytes stay byte-identical. An ambiguous store id and an other-namespace decoy refuse. | Q18. | A2, A3, A5 and A6. |

## Dependency readiness

The formal edge is F-285. Its provider contract is approved. `BACKLOG.md` still records F-285 as `pending`, so this story is not implementation-ready until the F-285 prefix checkpoint is integrated. The file lease also waits until F-286 leaves `content_control.rs`. Q18's helper fact is resolved and is not an open choice.

## Acceptance criteria

| ID | Input or setup | Expected observation or refusal | Consumers | Local test | Final gate or pending evidence |
|---|---|---|---|---|---|
| A1 | Create a store with one element and one schema reference | Reopen resolves the same item id. The schema reference is unchanged. A second create does not overwrite the first part | Native `Document` and Python `Document` | `custom_xml_store_round_trips_item_and_schema` | Hash harness unchanged for documents that do not call the API |
| A2 | Bind a rich-text control to that store with prefix mappings and `/ns:root/ns:child` | Fill updates the element text and the control display. Reopen shows both | Binding setter and existing fill | `custom_xml_binding_updates_the_addressed_store` | Round-trip suite |
| A3 | The same item with an unknown sibling and a comment in the unmodelled span | After fill, that span is byte-identical | Store writer | `custom_xml_unknown_bytes_stay_identical` | Round-trip suite |
| A4 | XPath with a function, a wildcard, a descendant axis, and a one-based index | The first three refuse and publish nothing. The index form updates that child | Binding setter | `custom_xml_xpath_stays_inside_the_existing_subset` | None beyond local refusal |
| A5 | Two controls, one path missing | The successful control does not remain half-updated. Document and item match the input | Existing staged fill | `custom_xml_bind_failure_rolls_back_display_and_store` | Round-trip suite |
| A6 | A store id that resolves to two items, and a same-local-name decoy in another namespace | The ambiguous id refuses. The decoy is not metadata | Existing resolver | `custom_xml_identity_matching_stays_namespace_aware` | Existing decoy test stays green |

## Test matrix

| Case | Modes or state combined | Expected difference or agreement | Wrong implementation rejected |
|---|---|---|---|
| C1 | No store, one store, two stores | Ids do not collide and names are not reused | `count + 1` naming that overwrites a hole |
| C2 | Empty selected element versus an element with text | Both end with the filled text. Siblings stay put | A rewriter that serialises the whole item from a tree |
| C3 | Implicit default namespace versus an explicit prefix | The same expanded name selects the same element | A string compare on the prefix |
| C4 | Bind, save, reopen, fill | The binding attributes survive reopen | A binding that exists only on the live object |
| C5 | Bibliography store left untouched | Its item id and bytes stay | A public helper that rewrites the bibliography part |

CLI and WASM stay unchanged. Layout does not read custom XML item bytes except through the control display, which A2 checks.

## Rejected alternatives

- Exposing `OpcPackage` mutation as the public API.
- A general XPath engine. The packaging spec already bounds the path language.
- A new custom-XML crate. The resolver and the bibliography writer are the existing owners.
- Starting this story in parallel with F-286. The file lease on `content_control.rs` is shared even though the dependency edge is only F-285.

## Test plan

**Test gate**: round-trip. Bound controls resolve, update and reopen against the correct store, and unknown store content stays byte-identical.

| Timing | Category | Test | Asserts |
|---|---|---|---|
| local scoped | round-trip | A1 through A6 in the existing `rdocx` integration entrypoint | Id, path, rollback and byte identity |
| local scoped | regression | Existing binding and decoy tests | The evaluator did not grow a wildcard or a function |
| final sprint | rider | full hash harness and installed suite | No delta on packages that do not gain a store |

No new test binary. No network schema fetch.

## HLD impact

- `docs/hld/02-scope-and-non-goals.md`, DOCX-054
- `docs/hld/03-architecture.md`
- `docs/hld/04-opc-and-packaging.md`
- `docs/hld/10-bindings-spec.md`
- `docs/hld/12-testing-strategy.md`
- `docs/hld/14-development-backlog.md`

## Risk routing

- Any parser or serialiser. Properties child order, and a byte-identical unknown span.
- Public API of a published crate. Additive store and binding methods. Final publish dry run.
- WASM or PyO3 bindings. Python is included. Final `rdocx-wasm` check. Python tests stay excluded from the workspace binary.
- Shared semantic behavior with multiple consumers. Bind, fill, display and reopen.
- Grammar, scope, compatibility mode or empty collection. C1 and the empty selected element.
- Crate boundary. No `oxml-*` dependency on `rdocx-*`. The helper, if extracted, stays in `rdocx`.

No external oracle. The test gate is round-trip.

## Hash harness

Unchanged.

## Implementation checklist

- [ ] Wait for the F-285 prefix checkpoint and for F-286 to leave the shared files.
- [ ] Add a RED test that a missing public create still cannot bind a new store.
- [ ] Implement the store and binding methods on the existing resolver.
- [ ] Update the HLD list and obtain a zero-finding microscope.
- [ ] Leave the hash harness for the final union.

## Open questions

| ID | Affected F-IDs and A-IDs | Decision and recommendation | Status and answer/source |
|---|---|---|---|
| Q18 | F-287 A1 to A6 | `add_custom_xml_store` returns a new item id through the existing graph. Binding accepts only the current absolute child-path subset. Unknown bytes stay byte-identical. Bibliography shares a private `rdocx` helper because the relationship steps match. The helper does not live in `oxml-*`. | Answered 2026-10-18 by the user. The helper fact was confirmed from `bibliography.rs` lines 1795 to 1846. |
| Q19 | F-285, F-286 and F-287 | This story starts after F-286, which starts after the F-285 prefix checkpoint. | Answered 2026-10-18 by the user. |
| Q21 | F-285, and this story's approval | The F-285 method names. This plan has no separate product choice. | Answered 2026-10-18 on F-285. This plan adopts those names. |

No external issue closes from this plan.
