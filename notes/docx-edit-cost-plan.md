# Docx counterpart of #403: profiles, plan, PR1 design draft (2026-10-23)

## Measurement

rdocx-skills `tests/test_docx_scaling.py` (on rdocx-skills main) times every mutating call of the rdocx Python binding
on documents of N = 25 and 4N = 100 body paragraphs built by `edit_cost_docx` (`tests/builders.py:318`). Each call
runs on a freshly opened document. A GROWING call counts as fixed when its 4N cost is under 1.5 x its N cost + 5 ms.
On #415's head (c025a48f), 135 calls still grow. `scratch/scale407/final/scaling.txt` holds the per-call numbers.

## Profiles (release build with symbols, `sample`, 4N, % of the call)

The document copy (`clone_for_staging`) is never a real cost: at most 2.1% of a call, median 0.4%.
Five whole-document steps:

1. **Binding inventory**:
   - `word_owner_locations` serializes, scans and reparses every story item.
   - `WordOwnerIndexes::prepare` reruns `paragraph_story_locations` and `cell_paragraph_story_locations`, which run
     `stories()` and the body scans again.
   - Returned handles are bound by native whole-story scans, once per cell owner, so binding is quadratic:
     `bind_prepared` -> `cell_paragraph_story_location` -> `modeled_main_cell_route`.
2. **Save compare**: `publish_api_candidate`, `publish_owner_candidate` and `publish_form_candidate` detect a no-op
   by comparing two `to_bytes`. The same pattern sits at document.rs 1683, 1724, 9686, 10825 and 12771.
3. **Flush and staging**:
   - `flush_document_to_package` reparses document.xml only to test for a change (about 11 ms at 4N, where the
     value compare would cost 0.05 ms), then replays nested-namespace owners. It runs several times per call.
   - Staging adds `canonicalize_drawing_ids` -> `opaque_relationship_ids_from_serialized_story`, and
     `reconcile_comment_removal`.
4. **Reopen**: `prepare_and_reopen_staged` writes the ZIP and runs `from_bytes`. About 108 native call sites.
5. **Addressing and proofs**:
   - whole-story scans for one item: `story_item_source`, `story_source_and_owner`, `pictures()`,
     `modeled_main_cell_route`, comment anchors, `content_control::index_elements`, `story_range_*`;
   - `validate_numbering_graph` -> `document_numbering_references` clones the document, flushes it and scans every
     story part;
   - `prove_scoped_main_source` reparses the main part.

   These are the superlinear calls: `Cell.add_paragraph` 7.9x, `Cell.replace_text` 4.6x, `move_comment` 4.4x
   (317 -> 1408 ms).

Representative shares at 4N:

| call | 4N ms | inventory | save compare | staging | reopen | addressing |
|---|---:|---:|---:|---:|---:|---:|
| Table.indent | 66 | 100 | 0 | 0 | 0 | 0 |
| Cell.add_paragraph | 387 | 100 (84 binding, quadratic) | 0 | 0 | 0 | 0 |
| Cell.text | 216 | 34 | 0 | 53 | 12 | 0 |
| Run.text | 202 | 31 | 68 | 0 | 0 | 2 |
| Document.set_header | 215 | 29 | 0 | 58 | 12 | 0 |
| Document.set_font | 190 | 33 | 67 | 0 | 0 | 0 |
| Document.move_comment | 1408 | 4 | 0 | 16 | 2 | 77 |
| Cell.replace_text | 713 | 9 | 0 | 18 | 4 | 68 |
| Document.replace_image (cause B) | 84 | 0 | 0 | 68 | 31 | 0 |
| Document.link_style_to_numbering (cause B) | 69 | 0 | 0 | 99 | 0 | 0 |

The full per-call profiles and results were in the old machine's scratchpad, `docxplan/` (pr25.txt, pr100.txt,
split-100.txt and the scripts). The scripts are copied under `scratch/docxplan/`.

## Plan (5 PRs, each with a debug oracle against the full path)

| PR | content | calls fixed (cumulative) | size | risk | depends on |
|---|---|---:|---|---|---|
| 1 | Word owner inventory by body block and story part | 42 | L, 1.5 to 2.5k lines | medium | #415 |
| 2 | reuse an unchanged main part at staging, `saves_as` instead of two `to_bytes` | about 52 | M | low to medium | #416 `same_package_as` |
| 3 | stage the main part by blocks (block provenance, incremental flush, numbering references) | about 58 | L/XL | high: nested-namespace replay must stay byte for byte | |
| 4 | reopen committed documents in memory | about 86 | M/L | medium-high | #416 `reread` |
| 5 | address story content by block (cell routes, pictures, comment anchors, content controls, story ranges) | about 133 | L | medium | |

PR1's 42 calls:
- the 23 Table, Row and Cell property setters;
- `Cell.add_paragraph`, `Paragraph.text`, `add_run`, `add_hyperlink`, `add_equation`;
- `Run.remove`, `Run.add_field`, `Document.split_run`;
- `add_comment`, `reply_to`, `resolve_comment`;
- `add_chart`, `set_text_watermark`, `set_image_watermark`, `set_note_policy`;
- `Settings.odd_and_even_pages_header_footer`.

**User decisions (2026-10-23):**
- only PR1 for now; PR2 to PR5 are decided after PR1's review and the #413/#416 merge;
- detection by value comparison (derived `PartialEq` on blocks), not `Arc` sharing.

Open questions not yet asked:
- PR3 byte fidelity (recommended: fix any oracle difference toward the old bytes);
- a heavy-media variant of the skills test (its 30 KB pictures hide package-size costs);
- after `set_font` every paragraph handle goes stale, kept as is (in scope).

## PR1 design draft (checked against the code, nothing written yet)

Branch `fix/docx-edit-cost-inventory`, at c025a48f (#415's head before its fix pass). Rebase it onto #415's final head
first.

- **Owner order.** `word_owner_addresses` (rdocx-py document.rs about 4755) builds one flat list by family: tables,
  each followed by its cells, then paragraphs with their runs, then sections, then hyperlinks. Within a family the
  order is story-source order: the main part (body story first, then cell stories in document order), then headers
  and footers, footnotes, endnotes, comments. Births and generations follow that order, so an incremental build must
  reproduce it exactly.
- **The live owner list can be out of that order** (`word_inventory_is_canonical`, about 5164: a local edit appends
  the runs it creates). Do not cut kept owners out of `self.owners`.
- **A cache of pieces on `PyDocument`.** Keep one piece per body block and one per story part:
  - a block's piece holds its four snapshot families with block-relative routes, plus the parsed facts (accepted
    text, run paths, cell paragraph counts), and the `BodyContent` value it was built from;
  - a part's piece holds the same families for that part, and the part's bytes;
  - a piece is reused only when the candidate's block or part equals the stored value, so it cannot go stale.
- **Detection.** Align the common prefix and suffix of the body blocks with `PartialEq`. Any change of the root or
  body namespace declarations, or of `plain_local_edit_blocked`, rebuilds everything.
- **Changed blocks.** A new hidden native helper generalizes `detached_table_owner_snapshots` and
  `standalone_body_item` (rdocx document.rs about 24232-24268 and 23620) to any `BodyContent`.
- **Assembly.** Walk the pieces with offsets: body item index, direct-body slot, cell-story `owner_index`, and the
  paragraph, table and hyperlink ordinals of each story. Recompute the flat parent indexes. This replaces
  `word_owner_locations(candidate)` in `prepare_word_revision` and `adopt_property_edit`.
- **Indexes and binding.** `WordOwnerIndexes::prepare` takes `body_paragraphs`, `cell_paragraphs` and `body_tables`
  from the pieces. Returned handles are bound from the inventory: `bind_prepared`, `TableLocation::native`,
  `cell_paragraph_story_location`.
- **Fallback to the full rebuild**: a text-box story is present, a FormValues or Signing publication, or any
  construct spanning blocks.
- The Conservative and local publication logic stays as it is, fed with the assembled list.

Check first:
- whether a `StoryId` fingerprint (`structural_fingerprint` over owner XML spans, document.rs about 12318-12333 and
  12827) built on one block equals the whole-source one (#415's oracle compares without fingerprints);
- how `scan_story_items` numbers body items against `body.content` slots: raw XML blocks, preserved nodes, the
  `sectPr` exclusion, `complex_field` items;
- the hyperlink snapshot order.

Guards to write:
- a debug oracle (full rebuild plus full `prepare_word_revision`, asserting the same owners, parents, slots,
  ordinals, read facts, index projections, retired set and birth order; #415's `unfingerprinted_*` helpers can be
  reused);
- a debug counter `_word_*_work` read by a pytest at K vs 4K blocks (it must fail when piece reuse is forced off);
- Rust N vs 4N counted tests in existing test files.
