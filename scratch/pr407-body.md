## Context: why a change of this size

This PR is larger than the fixes usually sent here, and that is deliberate.

Every public edit of the Python bindings goes through the same safety design. The binding copies the whole document, applies the edit to the copy, and revalidates the result. For package-level edits it validates by writing and rereading the whole ZIP. Then it rebuilds the inventory of every live Python handle from scratch and matches the old handles against it. A failed edit therefore leaves nothing half done, and a handle never silently points at a stale element. These guarantees are worth keeping. Their implementation, though, recomputes the whole document on every call. The staging and ZIP reopen date from August. The per-edit handle inventory came with F-X211a and F-X260 in October.

#407 is the Word side of the same family as #403 (rpptx, fixed by #413 and #416). Reading one paragraph's XML serialized and rescanned the whole body. Inserting or removing one body paragraph cloned the document and reparsed every paragraph to rebuild the handle inventory. Both cost about 0.5 s at 1000 paragraphs of real-world runs. Here the reads serialize only the item read, and the plain body edits update the handle inventory in place (a splice), next to the full rebuild they used to need.

**This is an exceptional change, not a pattern.** It retrofits one property, cost proportional to what an edit touches, to the body reads and the plain body edits. That is why it is large. We do not expect to send changes of this size routinely. The guarantees above are kept, not traded: under `debug_assertions`, every edit also runs the previous full path and asserts the same result. CI builds the binding in debug for its pytest suite, so every test checks the equivalence. A differential fuzz against main found no difference in outcomes, errors, saved bytes or handle validity.

`Paragraph.xml`, `insert_paragraph` and `remove_content` cost a time in proportion to the document body (#407). So did `Run.xml`, `Table.xml`, `Cell.xml`, `add_paragraph`, `Paragraph.insert_paragraph_before`, `remove_content_range`, `insert_table` and `add_table`, which take the same routes. A read serialized and scanned the whole body to find one item, after a linear search of the owner inventory and an enumeration of every story. A body edit cloned the document, then rebuilt the Python owner inventory from a body serialization, story scans and a reparse of every paragraph.

## What changes

- **Reads.** A direct body paragraph, run, table or cell is serialized alone under the main part root and found through the inventory index.
- **Edits.** `insert_paragraph`, `insert_paragraph_before`, `add_paragraph`, `insert_table` and `add_table` without a style, and `remove_content` / `remove_content_range` on plain direct paragraphs (with or without a comments part), splice the owner inventory in place. The kept owners shift their body routes (and their cell story owner indexes after an inserted table), the inserted owners are born and the removed ones retire, with the retirements, births and order the full rebuild gives. The publication is prepared before the native edit, which then runs in place without a staged clone.
- **Debug checks.** Under `debug_assertions`, every splice is checked against the full rebuild and every direct read against the old whole-source read. CI runs the binding's pytest suite on a debug build, so both checks cover the whole suite. A debug-only counter of full rebuilds and whole-source reads, made outside those checks, lets `test_issue407_direct_routes_do_no_whole_body_work` prove that each call took its direct route.
- `RevisionCounter::prepare` (oxml-py-support) compared retired scopes pairwise, so retiring a whole inventory was quadratic. It now keeps a set.

## One visible byte change in the XML reads

The main part root declares `xmlns:w14` whenever any byte of the part uses it, so a fragment used to carry `xmlns:w14` whenever any other body item did. A paragraph, table or cell fragment now declares w14 only when the part root does or the fragment itself uses it, by name or in a markup compatibility list such as `Requires="w14"`. Every other declaration is unchanged. A fragment whose `mc:Choice Requires="w14"` used to come out with an unbound prefix (when nothing else in the body used `w14:`) now declares it. Saved documents are unchanged (hash harness: 49 match).

## Measured (release build, Apple M3 Max, ms per call)

| call | before, 100 / 1000 / 4000 paragraphs | after |
|---|---:|---:|
| `Paragraph.xml` | 12.7 / 126 / 524 | 0.11 / 0.11 / 0.11 |
| `Run.xml` | 13.0 / 127 / 524 | 0.24 / 0.11 / 0.12 |
| `Table.xml` | 6.9 / 69 / 282 | 0.006 / 0.007 / 0.007 |
| `Cell.xml` | 11.4 / 113 / 455 | 0.12 / 0.07 / 0.07 |
| `insert_paragraph` | 45 / 471 / 2297 | 0.14 / 1.28 / 5.51 |
| `remove_content` | 38 / 400 / 1993 | 0.12 / 1.16 / 4.89 |
| `insert_paragraph_before` | 45 / 466 / 2298 | 0.14 / 1.17 / 4.92 |
| `remove_content_range` | 115 / 1142 / 4992 | 0.19 / 1.16 / 5.20 |
| `add_paragraph` | 44 / 437 / 1807 | 0.07 / 0.39 / 1.52 |
| `insert_table` | 46 / 462 / 2293 | 0.33 / 1.39 / 5.33 |
| `add_table` | 49 / 450 / 1859 | 0.21 / 0.59 / 1.96 |

The edits stay linear in the number of owners, at about 0.2 us each: a conservative publication still retires and rebirths the scopes of every owner. Going further needs an inventory scope shared by every handle, a larger change left out of this PR.

Still on the full path: `add_table` with a style (it edits styles), a table inserted in a main part that has a text box story (45 / 431 ms at 100 / 1000 paragraphs), XML reads of nested tables, of tables inside a content control and of paragraphs that are not direct body children.

## Repro

The issue's script now passes (every run, insert_paragraph at 1000 paragraphs 1.3 to 1.8 ms against its 3 ms bar):

```text
rdocx 0.16.3
document.xml: 241042 and 2410942 bytes (100 and 1000 paragraphs of 6 runs)
PASS Paragraph.xml: 0.1 ms with 100 paragraphs, 0.1 ms with 1000
PASS insert_paragraph: 0.2 ms with 100 paragraphs, 1.3 ms with 1000
PASS remove_content: 0.1 ms with 100 paragraphs, 1.2 ms with 1000
```

#389's script still passes.

## Tests

- rdocx: `issue407_direct_body_items_read_as_their_story_source` checks the read contract (equal to the whole-source read up to the root w14 declaration, w14 declared exactly when needed) over paragraphs with runs, properties, a hyperlink, fields, a bookmark, content controls and tracked changes, and over a table with merged cells and a nested table, in five root and w14 cases. `issue407_direct_body_reads_and_plain_edits_do_not_grow_with_the_body` finds the same counted work at 100 and 400 paragraphs, with no body serialization.
- rdocx-py: `test_issue407_direct_routes_do_no_whole_body_work` (debug builds) and the existing handle lifetime tests.
- A differential fuzz against main (410 seeds, about 4,100 operations over ten document shapes: Word-like roots, nested and merged tables, block content controls, custom XML, alternate content, text boxes, a section in the last paragraph, headers, footnotes and comments) found no difference in handle validity, stale messages, text, saved bytes or XML beyond the w14 contract.

Fixes #407.
