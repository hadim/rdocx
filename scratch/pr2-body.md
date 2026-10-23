## Context: why a change of this size

This PR and #413 are much larger than the PRs usually sent here, and that is deliberate.

Every public edit of the Python bindings goes through the same safety design. The binding copies the whole document, applies the edit to the copy, and revalidates the result. For package-level edits it validates by writing and rereading the whole ZIP. Then it rebuilds the inventory of every live Python handle from scratch and matches the old handles against it. A failed edit therefore leaves nothing half done, and a handle never silently points at a stale element. These guarantees are worth keeping. Their implementation, though, recomputes the whole document on every call. The staging and ZIP reopen date from August. The per-edit handle inventory came with F-X211a and F-X260 in October.

On a real deck this made 159 of the 248 public mutating calls of rpptx cost a time that grows with the deck (#403). Per-call fast paths (0.14.3, #389) fixed five of them and could not close the family. #413 made the candidate copy and the handle inventory proportional to the parts an edit changes. This PR removes the last whole-package step: the commit that wrote and reread the whole ZIP, at about 22 ms per MB, on every package-level edit.

**This is an exceptional change, not a pattern.** It retrofits one property, cost proportional to what an edit touches, across a whole edit path that 50 to 100 call sites share. That is why it is large. We do not expect to send changes of this size routinely. The guarantees above are kept, not traded: under `debug_assertions`, every edit also runs the previous full path and asserts the same result. CI builds the binding in debug for its pytest suite, so every test checks the equivalence. Random edit sequences replayed on decks with and without the parse snapshot (read from bytes, built in memory, reloaded after every step) gave the same bytes and errors.

Second of two PRs for #403, **stacked on #413**: this branch starts at #413's head (bdf22890), so until #413 merges the diff here shows both. The commits of this PR are the fourteen after bdf22890.

#413 made the edit candidate cheap and the Python owner inventory local. What remained is the issue's second cause: every package-bound edit (properties, notes, comments, removing a shape or a slide, replacing an image, cell fills, themes, layouts, charts, fonts...) committed by writing the whole package as a ZIP, every part deflated again, media included, and then parsing all of it back. That costs about 22 ms per MB.

## What changes

- **oxml-opc**:
  - `OpcPackage::reread` (hidden) gives what `write_to` followed by `from_reader` would give, or the same error, without a ZIP.
  - It checks every entry name. It runs the writer's checks (identities, relationship ids, content types, characters XML cannot carry) only on the entries that changed, and reparses only the relationships and content types that changed.
  - It falls back to the ZIP for names a write would change (a trailing backslash, too long for an entry, a part of 4 GiB or more) and for packages holding malformed producer bytes.
  - `Relationships` share their source bytes between clones.
- **rpptx**:
  - A presentation read from bytes keeps a snapshot of what it parsed. A commit reopens in memory, reusing every slide, layout and master record whose part, relationships, notes and comments bytes are unchanged, and reparsing only the rest.
  - Media are hashed only for new bytes. `staged_package` skips the parts that still match the snapshot.
  - `Presentation::saves_as` (hidden) replaces the binding's 18 calls that compared `to_bytes()` twice (flips, connections, align, distribute, chart legend and data, theme name, notes text, fonts, `replace_all`). `insert_picture` validates through the same reread.
  - `deterministic_part_key` no longer allocates.
- The snapshot keeps the text field ids of each part as read, so choosing an unused field id (slide header and footer, `Paragraph.add_field`, a slide on a layout with header/footer placeholders) rescans only the parts that changed. The binding's footer and transition proofs match only the parts an edit changed.
- **Debug oracle**:
  - Under `debug_assertions`, every commit runs a second time on a copy without the snapshot, through the old full staging and ZIP reopen.
  - It asserts the same error, or the same package, records, media store and saved bytes.
  - CI runs the binding's pytest suite on a debug build, so every commit site of that suite checks the equivalence.

## Three labelled behaviour changes

1. **Commits keep untouched slides' bytes** (23c7837a). `commit_candidate` and `commit_embedded_candidate` wrote every slide and notes part again from its model. They now stage like `to_bytes` and `commit_owned_parts` already did, so an untouched slide keeps its source bytes through a package-bound edit and the save. The M21 representative deck keeps its two hand-written notes slides as written, so its pinned digest changes, and those two parts are its only change.
2. **`fit_text` measures only its own slide** (85d35fe8). An edit of one slide no longer fails because another slide cannot render: for example, slide 3 without its layout relationship, or a notes master without its theme. `fit_report` and `refresh_autofit` still fail on such a deck.
3. **Slide transition and slide header/footer edits ignore other slides** (71594cbb). They used to match the owners of every slide, so a deck with two owners of one identity on another slide (two `mc:AlternateContent` shapes, for example) refused them with "footer owner identity is ambiguous". They now match only the parts the edit changed, and succeed. A header/footer edit adds or removes placeholder shapes, so like `add_shape` it is still refused when a kept slide repeats a shape id, now with "structural owner proof has an ambiguous shape id".

The hash harness (rdocx samples) matches on all 49 entries.

## Measured (release build, Apple M3 Max)

`tests/test_pptx_scaling.py` of hadim/rdocx-skills#61 with `--runxfail`: **all 248 calls pass**, plus the completeness test. On #413's head, the 107 package-bound calls failed.

| call, 5 to 20 slides (1 MB picture a slide) | #413 head | this PR |
|---|---:|---:|
| `core_properties.title` | 205 to 662 ms | 0.2 to 0.3 ms |
| `shapes.remove` | 187 to 735 ms | 1.0 to 1.1 ms |
| `replace_image` | 193 to 693 ms | 0.5 to 0.6 ms |
| table cell `fill.solid` | 186 to 713 ms | 0.8 to 1.0 ms |
| `Shape.flip_h` | 150 to 562 ms | 0.6 to 0.8 ms |
| `slides.remove` | 113 to 406 ms | 1.1 to 1.4 ms |

The five slide `SlideTransition` setters and the four slide `HeaderFooter` setters, which still grew by about 1 ms a slide on CI after the first version of this PR, now take 0.3 to 1.6 ms at 5, 20 and 40 slides (9 to 14 ms at 40 slides before).

The issue's `package_calls.py` (s per call at 10, 40 and 80 slides, then 40 slides with 1 MB pictures):

| call | main | this PR |
|---|---|---|
| replace_image | .085 / .256 / .487 / .752 | .006 / .006 / .006 / .021 |
| slides.remove | .067 / .251 / .504 / .693 | .001 / .002 / .002 / .002 |
| add_picture | .012 / .029 / .057 / .055 | .005 / .005 / .005 / .020 |
| add_slide | .004 / .014 / .036 / .018 | .000 |

The 1 MB row pays for decoding the new image, not for the deck.

The issue's `shape_calls.py --media` (ms per call at 10, 40 and 80 slides):

| call | main | this PR |
|---|---|---|
| build 10 slides (s) | 6.6 / 91.7 / 348 | 0.3 / 0.3 / 0.3 |
| cell.fill.solid | 166 / 637 / 1422 | 7.4 / 7.5 / 7.9 |
| notes_text_frame | 170 / 652 / 1435 | 1.5 / 1.9 / 2.4 |
| replace_image | 158 / 532 / 1012 | 3.4 / 3.6 / 3.9 |
| shapes.remove | 185 / 698 / 1566 | 9.7 / 10.0 / 10.3 |
| slides.remove | 178 / 693 / 1584 | 1.3 / 1.9 / 2.7 |
| add_slide | 18.9 / 158 / 527 | 0.1 / 0.2 / 0.2 |

The other rows are 0.0 to 0.9 ms, flat.

**What still grows a little**:
- Removing a slide renumbers the owners of the later slides in the binding (#413's addresses carry the slide ordinal): the counted inventory work goes from 35 at 3 slides to 314 at 12, about 0.3 ms at 20 slides.
- `slides.remove` updates the slide list and scans every slide's relationships for jumps to the removed slide.
- Notes and picture edits keep per-part metadata work: package clones and identity checks.
- A slide edited in place since the last commit is serialized again at staging. So a deck built in memory and never committed still grows on such calls: `flip_h` takes 1.7 to 5.3 ms from 5 to 20 slides. Removing that needs a per-record serialization cache.

**Still on the whole-package path**, none of them in the scaling test: embedded-content commits (`commit_embedded_candidate`), signed decks, and `refresh_autofit`, which works on the whole deck by design.

## Tests

- rpptx:
  - `issue403_package_edits_do_the_same_commit_work_at_n_and_4n_slides` covers ten edits, one per commit family, plus a `saves_as` call. Each does the same counted parsing, serializing, hashing and ZIP work at 3 and 12 slides, with zero ZIP writes, and gives the same bytes as the ZIP path. It fails if `saves_as` goes back to comparing two `to_bytes()`.
  - `issue403_a_commit_reuses_unchanged_records_and_keeps_edits_made_in_place`.
  - `issue403_fit_text_does_not_fail_on_another_broken_slide`.
  - The counted test also covers a slide transition and a slide header/footer edit after a field was added on another slide, so a stale carried field id would change the bytes.
- oxml-opc: reread tests, including a stray root `.rels` (the same `DuplicatePartName` as the ZIP path) and a name ending with a backslash.
- rpptx-py: a pytest guard of the commit and inventory work at 3 and 12 slides (debug builds), next to #413's inventory guard, and `test_issue403_footer_and_transition_edits_ignore_another_slide`.
- Debug builds also check the field ids chosen against a full scan, and the footer proofs against a match of every part.

Fixes #403.
