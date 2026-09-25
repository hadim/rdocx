# rpptx-cli

`rpptx-cli` makes complete PPTX workflows available to shell scripts. It
inspects and extracts deck content, validates package invariants, compares or
replaces text, and produces deterministic fixed output.

## Capabilities

- Human-readable inspection, and JSON inspection with per-shape kind,
  placeholder, geometry, rotation, autofit, and text detail.
- Slide-order text extraction and recursive outlines, as plain text or schema-1
  JSON, with speaker notes on request.
- PDF, PNG, JPEG, and multi-page TIFF conversion.
- Selected-slide rendering, thumbnails, text diff, replacement, and validation.
- Scriptable output covers slide order, notes, comments, recursive group
  content, relationship-backed media, and deterministic rendering diagnostics.

## Measured footprint and speed

| Measurement | Value | Version | Platform | Build mode | Input | Command | Statistic | Measured on |
|---|---|---|---|---|---|---|---|---|
| Crates.io archive: rpptx-cli | 27,236 compressed bytes, 108,831 member bytes, 8 members | 0.12.1 | macOS 26.6.2, Apple M5 Max, arm64 | `cargo package --locked --no-verify` | Tracked `rpptx-cli` package inventory | `python3 scripts/readme_doctests.py --record-measurements` | gzip archive bytes, tar member bytes, tar member count | 2026-09-19 |

## Use it when

Use the CLI for shell automation. Use `rpptx` when the same operations belong inside a Rust application.

## Relationship

It uses `oxml-cli-support` for shared command conventions and the real `rpptx` facade for document behavior.

## Example

```sh
cargo install rpptx-cli --version '^0.12.1'
rpptx inspect deck.pptx --json
rpptx text deck.pptx --json
rpptx outline deck.pptx --notes
rpptx convert deck.pptx --to pdf -o deck.pdf
rpptx thumbnail deck.pptx -o thumbnail.png
```

`inspect --json` keeps its existing keys and adds `shape_details` beside each
slide's shape count. Each shape reports its z-order index, id, name, kind,
placeholder type and index, direct position and size in EMU, rotation in
degrees, direct autofit mode, paragraphs, table size, and children. A
placeholder that inherits its geometry from its layout reports `null` geometry,
and a placeholder without an explicit type reports a `null` type.

`text --json` reports each slide's one-based number, slide id, paragraphs, and
speaker notes, or `null` notes when the slide has no notes part. Each paragraph
has a typed zero-based `path` of shape, table row, table cell, and paragraph
positions, the id of the shape that owns it, its level, its visible text, and
its regular runs. Run indexes are the facade's run positions, so fields and line
breaks appear only in the paragraph text, where a line break is U+000B. Run
`formatting` is `null` when a run has no direct properties. Otherwise it records
nullable direct bold, italic, underline, font, point size, and sRGB colour
values.

`outline --json` reports each slide's title, its outline items with their
levels, and its speaker notes. Notes are the plain text of the notes body, with
paragraphs and line breaks both written as newlines. `--notes` prints one
`Notes:` line for each non-empty notes line after a slide's plain text or
outline. JSON output always carries the notes.

Run `rpptx --help` or `rpptx <command> --help` for the complete command surface.
