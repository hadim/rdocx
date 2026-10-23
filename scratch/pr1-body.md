## Context: why a change of this size

This PR and its follow-up #416 are much larger than the PRs usually sent here, and that is deliberate.

Every public edit of the Python bindings goes through the same safety design. The binding copies the whole document, applies the edit to the copy, and revalidates the result. For package-level edits it validates by writing and rereading the whole ZIP. Then it rebuilds the inventory of every live Python handle from scratch and matches the old handles against it. A failed edit therefore leaves nothing half done, and a handle never silently points at a stale element. These guarantees are worth keeping. Their implementation, though, recomputes the whole document on every call. The staging and ZIP reopen date from August. The per-edit handle inventory came with F-X211a and F-X260 in October.

On a real deck this made 159 of the 248 public mutating calls of rpptx cost a time that grows with the deck (#403): up to several seconds per call at 80 slides, and a deck builder ported from python-pptx took 902 s instead of 106 s. rpptx 0.14.3 added fast paths for the five calls #389 measured, but fixing calls one at a time cannot close a family of 159. So these two PRs change the shared mechanism instead:
- **#413 (this PR):** the edit candidate shares every part and record it does not change (copy on write), and the handle inventory is rebuilt only for the parts the edit changed.
- **#416:** a commit rereads in memory only the parts that changed, instead of round-tripping the whole ZIP.

**This is an exceptional change, not a pattern.** It retrofits one property, cost proportional to what an edit touches, across a whole edit path that 50 to 100 call sites share. That is why it is large. We do not expect to send changes of this size routinely. The guarantees above are kept, not traded: under `debug_assertions`, every edit also runs the previous full path and asserts the same result. CI builds the binding in debug for its pytest suite, so every test checks the equivalence. A differential fuzz against main found no difference in outcomes, errors, saved bytes or handle validity.

First of two PRs for #403. It removes the first of the two whole-deck steps the issue describes: the binding copied the whole `Presentation` for every edit and rebuilt and re-matched the owner inventory of every slide, with a quadratic matching. The second step, the native commit that writes and reparses the whole ZIP, is the next PR, stacked on this branch.

## What changes

- **oxml-opc**: `OpcPackage.parts` holds `Arc<Vec<u8>>`, so cloning a package copies pointers, not bytes. `set_part` takes `impl Into<Arc<Vec<u8>>>` (every existing call compiles unchanged) and `remove_part` returns the shared buffer. This is a source-level change of a public field and two signatures of a crate both families use, to be listed under Compatibility.
- **rpptx**: slide, layout and master records and the notes and handout masters sit behind `Arc` and are copied on write, so a candidate clone costs one pointer per part. The `MediaStore` holds the package's own buffers instead of a second copy. Two `#[doc(hidden)]` methods tell the binding which records a candidate still shares with the live deck. Text replacement copies only the slides it changes.
- **rpptx-py**: the owner inventory is kept per part (one per slide, one for layouts and masters). An edit walks only the parts whose record the candidate no longer shares, carries the others over (remapping slide and layout ordinals on add, move, remove and duplicate), and matches survival proofs through maps instead of linear searches. Under `debug_assertions`, every prepare that carries a part is checked against a walk of every part, and the indexes are checked after each commit. The pytest suite, which CI runs on a debug build, therefore checks the equivalence on every edit.
- `Hyperlink.address` and `Hyperlink.target_slide` rescanned and hashed every medium after each edit. The rescan now runs only when a part was actually pruned.

Which Python handles stay valid or retire after an edit, error messages and saved bytes are unchanged (hash harness: 49 match). One visible difference: after an `add_textbox` append, later generation numbers can differ from main's. Generations are only compared for equality, so no handle changes validity.

## Measured (release build, Apple M3 Max)

`tests/test_pptx_scaling.py` of hadim/rdocx-skills#61 (decks of 5 and 20 slides, 25 text boxes, 12 rectangles, a 4x4 table, a 1 MB picture and notes a slide), run with `--runxfail`: the 52 calls of the issue's GROWING set that follow the shapes now pass, and every call outside GROWING still passes. The 107 calls that follow the package bytes still grow, and they are the next PR's.

| call, 5 to 20 slides | main | this branch |
|---|---:|---:|
| `Paragraph.add_run` | 5.9 to 41.5 ms | 0.1 to 0.1 ms |
| `ShapeCollection.add_shape` | 6.1 to 44.5 ms | 0.1 to 0.2 ms |
| `TextFrame.text` | 5.2 to 44.3 ms | 0.1 to 0.1 ms |
| `SlideCollection.add_slide` | 10.5 to 64.0 ms | 0.2 to 0.2 ms |

The issue's `shape_calls.py`, ms per call at 10, 40 and 80 slides:

| call | 10 | 40 | 80 |
|---|---:|---:|---:|
| add_shape | 0.5 | 0.5 | 0.3 |
| add_picture | 1.0 | 0.9 | 1.0 |
| text_frame.text | 0.3 | 0.3 | 0.3 |
| cell.text_frame.text | 0.4 | 0.4 | 0.3 |
| replace_xml (new text) | 1.1 | 0.7 | 0.8 |
| add_run | 0.4 | 0.4 | 0.4 |
| add_slide | 0.4 | 0.3 | 0.4 |

On main a57be365 the same rows went from 16 to 537 ms (add_shape) and 18 to 536 ms (add_run) between 10 and 80 slides.

## Tests

- rpptx: `issue403_a_clone_shares_records_and_bytes_until_an_edit_copies_one` checks that records, part buffers and media stay shared after a clone and that an edit copies only what it changes.
- rpptx-py: `test_issue403_shape_edits_do_the_same_inventory_work_at_n_and_4n_slides` (pytest, debug builds) asserts the same counted inventory work at 3 and 12 slides for ten shape edits. It fails when every part is forced to be walked. The Rust test of the same name in `fx260_visit_tests` adds the same check for local runs.
- A seeded differential run of 120 random edits (shapes, slides, layouts, imports, pictures, tables, `replace_xml`, `replace_text`, notes, headers and footers, hyperlinks) compared main with this branch on 50 seeds: identical outcomes, error messages, saved bytes and held-handle validity.

Part of #403.
