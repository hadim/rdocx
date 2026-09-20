# rdocx-cli

`rdocx-cli` turns DOCX files into inspectable and scriptable shell artifacts.
It extracts content, reports structure, validates packages, applies changes,
and produces fixed or flow output without an Office host.

## Capabilities

- Human-readable or JSON structure and metadata inspection.
- Plain text or schema-1 rich accepted-view extraction with typed nested paths.
- Deterministic point-space body layout fragments for shell automation.
- PDF, HTML, Markdown, PNG, JPEG, and multi-page TIFF conversion.
- Page-range rendering, guarded literal replacement, diffing, and validation
  verdicts.
- Comment thread inspection and mutation with explicit body run ranges.
- Tracked revision inspection, filtered resolution, document comparison, and
  table-of-contents rebuilds.
- Package-preserving edits cover the final S74 paragraph, run, typography,
  table, section, settings, field, form, equation, and drawing surface.

## Measured footprint and speed

| Measurement | Value | Version | Platform | Build mode | Input | Command | Statistic | Measured on |
|---|---|---|---|---|---|---|---|---|
| Crates.io archive: rdocx-cli | 33,805 compressed bytes, 145,256 member bytes, 8 members | 0.14.0 | macOS 26.6.2, Apple M5 Max, arm64 | `cargo package --locked --no-verify` | Tracked `rdocx-cli` package inventory | `python3 scripts/readme_doctests.py --record-measurements` | gzip archive bytes, tar member bytes, tar member count | 2026-09-19 |

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
rdocx revision accept reviewed.docx --author Reviewer -o accepted.docx
rdocx compare original.docx edited.docx --author Reviewer \
  --timestamp 2026-09-13T12:00:00Z -o redline.docx
rdocx toc rebuild report.docx -o refreshed.docx
```

Comment `add` ranges use zero-based body paragraph and run boundaries. The
start is inclusive and the end is exclusive. Comment replies, resolution, and
removal select a decimal comment id.

Revision `list` reports the main story. Revision `accept` and `reject` operate
across every supported story and accept at most one selector: `--id`,
`--author`, or the paired `--start-date` and `--end-date` RFC 3339 bounds.
Omitting a selector resolves all modeled revisions. Every mutation, comparison,
and TOC rebuild requires `-o/--output`, publishes only a complete validated
DOCX, and supports a schema-1 record through `--json`.

`text --json` reports accepted-view paragraphs in source order. Each paragraph
has a zero-based `body_index`, a typed zero-based path within that body item,
its direct style and numbering, and accepted-view runs. Run `formatting` is
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

Run `rdocx --help` or `rdocx <command> --help` for the complete option set.
