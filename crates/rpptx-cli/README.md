# rpptx-cli

`rpptx-cli` makes complete PPTX workflows available to shell scripts. It
inspects and extracts deck content, validates package invariants, compares or
replaces text, and produces deterministic fixed output.

## Capabilities

- Human-readable and JSON inspection.
- Slide-order text extraction and recursive outlines.
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
rpptx convert deck.pptx --to pdf -o deck.pdf
rpptx thumbnail deck.pptx -o thumbnail.png
```

Run `rpptx --help` or `rpptx <command> --help` for the complete command surface.
