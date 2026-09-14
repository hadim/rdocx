# rdocx-cli

`rdocx-cli` turns DOCX files into inspectable and scriptable shell artifacts.
It extracts content, reports structure, validates packages, applies changes,
and produces fixed or flow output without an Office host.

## Capabilities

- Human-readable or JSON structure and metadata inspection.
- Text extraction across body paragraphs and table cells.
- PDF, HTML, Markdown, PNG, JPEG, and multi-page TIFF conversion.
- Page-range rendering, literal replacement, diffing, and validation verdicts.
- Comment thread inspection and mutation with explicit body run ranges.
- Tracked revision inspection, filtered resolution, document comparison, and
  table-of-contents rebuilds.

## Use it when

Use this crate for shell automation. Use the
[`rdocx`](https://docs.rs/rdocx) library when these operations need to run
inside a Rust application.

## Relationship

The binary delegates document behavior to `rdocx` and shares path and JSON
conventions with `rpptx-cli` through `oxml-cli-support`.

## Example

```sh
cargo install rdocx-cli --version '^0.13.1'

rdocx inspect report.docx
rdocx text report.docx
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

Run `rdocx --help` or `rdocx <command> --help` for the complete option set.
