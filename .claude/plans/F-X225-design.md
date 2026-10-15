# F-X225, Keep DrawingML text-style parsing within the default WASM stack

**Status**: approved
**Sprint**: S91.1
**Size**: M
**Depends on**: F-X214, F-X221

Both dependencies are done. ROOT approves this bounded representation repair under the existing instruction to finish the sprint and its release blockers. No product question is outstanding.

## Problem

The exact main Node regression `wasm_presentation_uses_the_real_facade_in_node` fails under Node24.11.1, wasm-pack0.15.0 and wasm-bindgen0.2.126. The original fault is `CT_TextBody::from_element`, not the later console-error or externref allocation traps. The ordinary constructor parses the bundled template through real PresentationML shape and text-body owners. `crates/oxml-drawing/src/text/mod.rs:50` stores the ten-level list-style model inline. Its parser retains inline list-style results at229 and240. `crates/oxml-drawing/src/text/list_style.rs:135` and146 construct and return additional inline style values.

The reproduced main Base is `eb79c31155d8e38f9f7f0c6d3b2753c0b66ea83e`. The unchanged executable code reserves193024 bytes for `CT_TextBody::from_element`, with large live callers including77072 bytes for the wrapper,82864 for shape-child capture,55136 for shape reading,117872 for shape-child parsing,77984 for group-child capture and81040 for shape-frame parsing. The initial linear stack pointer is1048576. At the original fault it is negative49520, an aggregate1098096-byte reservation. An export-only diagnostic copy exposed the existing stack global without changing executable instructions. The diagnosis receipt is `/private/tmp/S91-main-WASM-diagnosis-1/diagnosis-receipt.json`, SHA256 `271de40d4a56f69ddce90f335d99be681d01538d31c6aa1fd88579feb9292419`. All original failed attempts remain failed.

## Spec reference

- `docs/hld/04-opc-and-packaging.md`, The package and Package integrity: unchanged package identity, raw XML and relationship preservation.
- `docs/hld/06-presentationml-model.md`, The shape tree and Preservation strategy: small owned projections, bounded parser frames, schema child order and verbatim unsupported source.
- `docs/hld/10-bindings-spec.md`, WASM: the `rpptx::Presentation` facade, constructor, addSlide, fromBytes and toBytes, complete graph round-trip, unchanged renderer-free default and deterministic render profile.
- `docs/hld/15-build-and-toolchain.md`, A WASM target and Node job and Pinned corpus-test runtime: exact Node24.11.1/wasm-pack0.15.0, both target graphs, and the distinction between approved native corpus8MiB and the actual default WASM1MiB stack.
- `docs/hld/12-testing-strategy.md`, regression, round-trip and binding tests: genuine Base failure, unchanged assertions and exact installed artifact provenance.

## Approach

Change the private `CT_TextBody::list_style` storage to `Option<Box<CT_TextListStyle>>`. Keep its absence allocation-free. Preserve `list_style(&self) -> Option<&CT_TextListStyle>` through `as_deref`, and preserve `has_list_style`, clone, equality, set_text, paragraph mutation, empty_like, content movement and serialization semantics. External callers receive the same references and values as before. Do not box or change the public ten paragraph-property fields of `CT_TextListStyle`.

Move list-style parsing and intermediate ownership onto the heap before large values accumulate in callers. Start with boxed owned storage and parsed locals in the existing text module. If that alone leaves the deciding test red, use a concrete private boxed parse path in the existing list_style module, so internal from_start/from_element calls and the text-body consumer need not materialize multiple full inline results. Public `from_xml`, `from_xml_with_inherited_namespaces`, `with_inherited_defaults` and other existing public signatures remain unchanged. Any private helper performs actual parsing or ownership work, rather than forwarding without reducing live storage.

Keep actual namespace scopes, captured root attributes, opaque child bytes, ordered raw slots and schema boundaries intact. Preserve error variants, invalid-input behavior and no-op source serialization. Do not replace parsing with a miniature model or omit source fields to reduce size. Do not change depth32, decoded-input limits, font policy, package identities, versions, feature defaults or renderer behavior.

Only the existing `crates/oxml-drawing/src/text/mod.rs` and, if needed, `text/list_style.rs` carry product changes. Add regression cases inside existing test modules. An existing `rpptx-wasm/src/lib.rs` inline test may be strengthened only if an additional real caller is required, keeping the original complete facade test intact. No new Rust file, module, crate, trait, generic parameter, dependency, feature or asset is proposed. If verified archive tuples move, update only the existing affected README observations and numeric catalog in `scripts/readme_doctests.py` from actual measurements, retaining the existing64-byte normalization tolerance and10MiB ceiling. Those metadata updates do not change versions or runtime behavior.

**Semver impact**: none. This changes private representation and parser storage, with no public signature, source-literal field type or behavior change. No version bump belongs to this story.

## Rejected alternatives

- Raising the WASM linker stack, Node stack or test-only allowance hides the real default-stack failure.
- A global native stack increase does not change the WASM linear stack and is not this repair.
- Boxing public list-style fields breaks native source callers unnecessarily.
- Removing retained XML, skipping the real facade or weakening its assertions abandons existing contracts.
- Applying the earlier broad experimental parser rewrite exceeds this focused storage defect. Extend only when the unchanged deciding test demonstrates a remaining failure.

## Test plan

**Test gate**: regression.

| Category | Test | Asserts |
|---|---|---|
| regression | existing `wasm_presentation_uses_the_real_facade_in_node` | Exact pinned Node24.11.1 and wasm-pack0.15.0, default1MiB WASM stack, real constructor/addSlide/fromBytes/toBytes and complete original graph assertions fail on genuine Base and pass after the storage repair. No linker stack override or capture bypass in accepted GREEN |
| regression | `fx225_list_style_parse_with_real_caller_state` in the existing rpptx-wasm inline tests | Use the original nontrivial-deck constructor and real template/layout/shape caller path, exercise source-like paragraph defaults and all levels, consume the retained state afterward, and pass within the existing default budget. Measure actual type sizes if useful, without inventing an ABI baseline or artificial padding |
| round-trip | `fx225_text_style_boxing_preserves_source_and_public_callers` in existing tests | Absent, empty and populated list styles, aliased/inherited namespaces, root attributes, comments, foreign lookalikes and opaque children retain exact payload and child order through parse, clone, getter, mutation, serialization and reopen |
| unit | existing and focused list-style inheritance, getters and body mutation controls | `Option<&CT_TextListStyle>` callers compile unchanged, all ten public property slots stay native values, equality and deep clone hold, set_text and empty_like retain intended styles, direct public style mutation serializes correctly, invalid and duplicate children keep their existing errors |
| regression | existing focused PresentationML parser and F214 bound controls | Ordinary template, layout, master and group consumers still parse, depth and resource limits do not change, no hidden capture/reparse or new recursion is introduced |
| integration | both WASM target checks and existing Node tests | `cargo check --target wasm32-unknown-unknown -p rdocx-wasm -p rpptx-wasm`, plus rpptx-wasm render target check, exact Word Node2 and PPT default Node1, and existing render-profile Node check where selected, retain facade boundaries |
| golden | unchanged hash harness49 | Same deterministic output bytes and no baseline update |
| integration | verified affected publishable archives | At least oxml-drawing and every additional package whose archived source changes verify against exact local dependency patches, with current member/source correspondence and each archive below10MiB |

Start with `cargo check -p oxml-drawing --all-targets`, then affected strict Clippy, existing text/list-style unit tests and the original exact Node regression in an exclusively owned target. Add native facade/PresentationML checks earned by actual source callers. Preserve the genuine pinned failure and diagnostic artifacts. Normal scoped format, repository policy, prose/adapters, rustdoc, MSRV, dependency policy and no-default checks remain required where the actual diff routes them. Do not rerun unrelated Office, full Python or the frozen100 merely to establish this focused regression. The sprint's final integrated gate still runs its mandatory union.

Acceptance must record the actual resulting frame/storage reduction, the fixed default stack and unchanged complete graph assertions. A compile check, enlarged-stack pass or release-only pass is insufficient. A populated real caller supplements the original default-template test and prevents a tiny empty fixture from hiding live model storage.

## HLD impact

- `docs/hld/06-presentationml-model.md`, The shape tree: describe the existing large list-default projection as privately heap-owned, preserving borrowed access and raw ownership.
- `docs/hld/10-bindings-spec.md`, WASM: require the real default facade constructor and round-trip to operate at the pinned default stack without a test-only allowance.

HLD04's preservation contract is unchanged and is read and tested, not rewritten. F-X224 owns HLD15 CI/runtime routing. Its unchanged default WASM budget is read and tested here, with no build-budget change. Backlog registration belongs to ROOT before claim, using the accompanying story-entry proposal. HLD prose describes current intent, not failure history or counts of completed runs.

## Risk routing

Parser/serializer: read HLD04/06, preserve exact unmodelled subtrees, root attributes, namespace scope and schema child order, with parse/save/reopen tests. Native published-crate API: read HLD10 and structural rules, preserve every public signature and field type, state no semver impact, verify affected archives and10MiB limits. WASM: exact default-stack Node regression plus both wasm32 graphs and the existing render feature. The dependency graph, feature defaults, shaping, layout and font assets do not change. Hash49 must remain exact, with deterministic bundled fonts. No new external rendering oracle or manual Office gate is earned by a representation-only change.

## Hash harness

All49 outputs are expected unchanged. Any output delta blocks completion pending separate attribution and review. Do not record new baselines or use ambient fonts.

## Implementation checklist

- [x] ROOT registers the story and approves this plan before claim.
- [ ] Retain the genuine default1MiB Base failure and immutable source/tool receipts.
- [ ] Implement only private storage and necessary private parser-temporary changes.
- [ ] Pass the original pinned default-stack Node regression and real populated caller test.
- [ ] Prove public caller compatibility, mutation semantics, opaque XML and child order.
- [ ] Pass affected native/WASM/scoped package and hash gates.
- [ ] Obtain zero-finding independent microscope and update only listed HLD mechanisms.

## Open questions

None. If an actual fix requires a public signature, public field type, resource-bound or observable behavior change, stop and revise the plan before making it. Only the actual diagnosed frames and original real caller are established evidence. Any later type-size measurement is diagnostic, not a new portable API or acceptance waiver.
