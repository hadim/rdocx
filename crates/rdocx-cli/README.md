# rdocx-cli

`rdocx-cli` turns DOCX files into inspectable and scriptable shell artifacts.
It extracts content, reports structure, validates packages, applies changes,
and produces fixed or flow output without an Office host.

## Capabilities

- Human-readable or JSON structure and metadata inspection.
- Accepted-view plain text or schema-1 rich extraction with typed nested paths.
- Deterministic point-space body layout fragments for shell automation.
- PDF, HTML, Markdown, PNG, JPEG, and multi-page TIFF conversion.
- Page-range rendering, guarded literal replacement, diffing, and validation
  verdicts.
- Comment thread inspection and mutation with explicit body run ranges and
  optional RFC 3339 dates.
- Tracked revision inspection, filtered resolution, table-of-contents rebuilds,
  and document comparison at run, word, or character granularity with ignore
  options.
- Package-preserving edits cover the final S74 paragraph, run, typography,
  table, section, settings, field, form, equation, and drawing surface.

## Measured footprint and speed

| Measurement | Value | Version | Platform | Build mode | Input | Command | Statistic | Measured on |
|---|---|---|---|---|---|---|---|---|
| Crates.io archive: rdocx-cli | 46,413 compressed bytes, 203,342 member bytes, 8 members | 0.14.0 | macOS 26.6.2, Apple M5 Max, arm64 | `cargo package --locked --no-verify` | Tracked `rdocx-cli` package inventory | `python3 scripts/readme_doctests.py --record-measurements` | gzip archive bytes, tar member bytes, tar member count | 2026-09-30 |

## Use it when

Use this crate for shell automation. Use the
[`rdocx`](https://docs.rs/rdocx) library when these operations need to run
inside a Rust application.

## Relationship

The binary delegates document behavior to `rdocx` and shares path and JSON
conventions with `rpptx-cli` through `oxml-cli-support`.

## Example

```sh
cargo install rdocx-cli --version '^0.14.0'

rdocx inspect report.docx
rdocx text report.docx
rdocx text report.docx --json
rdocx layout report.docx --json
rdocx replace template.docx -p TOKEN -v ready --expect 1 -o report.docx
rdocx convert report.docx --to pdf -o report.pdf
rdocx validate report.docx
rdocx render report.docx --page 0 -o rendered
rdocx comment list report.docx --json
rdocx comment add report.docx --start-paragraph 0 --start-run 0 \
  --end-paragraph 0 --end-run 1 --author Reviewer --text 'Check this' \
  --date 2026-09-13T12:00:00Z -o commented.docx
rdocx revision accept reviewed.docx --author Reviewer -o accepted.docx
rdocx compare original.docx edited.docx --author Reviewer \
  --timestamp 2026-09-13T12:00:00Z -o redline.docx
rdocx compare original.docx edited.docx --author Reviewer \
  --timestamp 2026-09-13T12:00:00Z --granularity word --ignore-comments \
  --ignore-story header -o redline.docx
rdocx toc rebuild report.docx -o refreshed.docx
```

Comment `add` ranges use zero-based body paragraph and run boundaries. The
start is inclusive and the end is exclusive. Run boundaries count the runs that
`text --json` lists, including the runs inside inline content controls and
tracked insertions. A range that cannot be anchored exactly, such as one that
crosses the edge of an inline content control, is refused. Comment replies,
resolution, and removal select a decimal comment id. Comment `add` and `reply`
write an optional `--date` RFC 3339 timestamp as the comment date. An invalid
timestamp exits unsuccessfully without creating the output, and without
`--date` the comment stays undated.

`compare` replaces each changed run whole by default (`--granularity run`),
like the Rust and Python APIs. `--granularity word` marks only the changed
words, and `--granularity character` marks only the changed characters.
`--ignore-formatting`, `--ignore-whitespace`, `--ignore-fields`,
and `--ignore-comments` keep the original side of those differences.
`--ignore-comments` keeps the original's comments and anchors and drops the
comments of the edited file. The repeatable `--ignore-story KIND` keeps one
story of the original, where `KIND` is a Python `Story.kind` name: `body`,
`header`, `footer`, `comment`, `text_box`, `footnote`, or `endnote`. Ignoring
the `comment` story excludes only the comments part, so a pair whose comments
differ needs `--ignore-comments`. An unknown granularity or story name is a
usage error, a repeated story fails, and neither creates the output. The
`--json` record states the options that ran.

Revision `list` reports the main story. Revision `accept` and `reject` operate
across every supported story and accept at most one selector: `--id`,
`--author`, or the paired `--start-date` and `--end-date` RFC 3339 bounds.
Omitting a selector resolves all modeled revisions. Every mutation, comparison,
and TOC rebuild requires `-o/--output`, publishes only a complete validated
DOCX, and supports a schema-1 record through `--json`. A `.docx`, `.docm`,
`.dotx`, or `.dotm` output extension selects the package class the output
declares, so a template edited into `report.docx` is written as a document. An
input that carries a VBA project cannot change to a macro-free extension.

`text` prints each paragraph with the same accepted-view text as `text --json`:
tracked insertions and move destinations are included, and tracked deletions
and move sources are left out. Plain `text` covers body paragraphs and the
paragraphs inside content controls, table cells, and nested tables in document
order.

`text --json` reports accepted-view paragraphs in source order. Each paragraph
has a zero-based `body_index`, a typed zero-based path within that body item,
its direct style and numbering, and accepted-view runs. Its `text` also holds
the text inside smart tags, inline custom XML elements, and simple fields,
which `runs` does not list. Run `formatting` is
`null` when no direct run properties exist. Otherwise it records nullable
direct bold, italic, strike, underline, font, point size, colour, highlight,
language, and character style values.

`layout --json` uses bundled deterministic fonts. It lists every direct body
item, including preserved items that have no fragments. Each laid-out fragment
uses points from the top-left page origin and records one-based physical and
displayed page numbers. A body item that crosses a page boundary has one
fragment on each occupied page.

`replace --expect N` publishes only when the run-aware replacement count is
exactly `N`. A mismatch exits unsuccessfully without creating or replacing the
requested output.

The count covers the text a reader sees in the body, tables,
content controls, headers, footers, footnotes, endnotes, and tracked insertions,
in the text boxes of the body, headers, and footers, in the labels of the
charts in the body, and inside smart tags, inline custom XML elements, and the
cached results of simple fields. Deleted text is not counted, except in a text
box inside a deleted run, which is replaced like any other text box. A text box
inside a note and the separators of the notes parts are not counted.

`convert` and `render` refuse an output file that already exists unless
`--force` is given. Even with `--force` they refuse their own input file under
any spelling of its path, and an output that is not a regular file, such as a
directory, a symbolic link, a FIFO, or a device like `/dev/null`. A run checks
every file it would write before it writes the first one, and publishes each
file only once it is complete, so a failed run leaves no truncated output.

`validate` exits unsuccessfully on a structural error: a relationship to a
missing part, a part without a content type, or a prefix that `mc:Ignorable`
or `mc:MustUnderstand` lists without a namespace declaration. Empty
paragraphs, heading level gaps, and missing metadata are warnings only.


Run `rdocx --help` or `rdocx <command> --help` for the complete option set.
