# rdocx-py

Native rdocx for Python

`rdocx` gives Python applications a native DOCX workflow for creating,
opening, editing, comparing, laying out, and rendering Word documents. It
works directly with OOXML packages and does not require Microsoft Word,
LibreOffice, a conversion service, or a Java or .NET runtime.

The package uses the Rust `rdocx` document engine. Unsupported safe producer
XML remains available to the package rather than being discarded during an
ordinary open and save cycle.

## Installation

Install the current release from PyPI:

```sh
python -m pip install rdocx
```

`rdocx` requires CPython 3.9 or newer. Its platform wheels use the Python 3.9
stable ABI, so one wheel supports multiple compatible Python versions. Source
installation requires a Rust toolchain and maturin.

## Quick start

```python
from rdocx import Document

doc = Document()
doc.add_paragraph("Hello from Python")
doc.save("hello.docx")

pdf = doc.to_pdf()
with open("report.pdf", "wb") as output:
    output.write(pdf)
```

## Capabilities

- File and byte-based DOCX input and output.
- Paragraphs, runs, fonts, tables, rows, cells, sections, and styles.
- Rich headers, footers, related stories, and resolved hyperlinks.
- Tracked comparison, main-body comment threads, revision resolution, and TOC
  rebuilding.
- Deterministic layout fragments, page geometry, and PDF, PNG, JPEG, and TIFF
  output through the native document engine.
- Python collections with negative indexes, slices, iteration, and explicit
  stale-handle errors after structural changes.

## Use it when

Use `rdocx` when a Python service, desktop application, or automation task
needs to work with Word documents locally. It is suited to template updates,
document assembly, review workflows, comparison, and deterministic output.

## Relationship

The Python API is intentionally focused. It is source compatible with the
documented `rdocx` surface, but it is not a drop-in replacement for every
python-docx method or private lxml object.

## Example

Open and update an existing document:

```python
from rdocx import Document

document = Document("template.docx")
document.add_paragraph("Approved")
document.save("approved.docx")
```

## Type checking

The distribution includes hand-written extension stubs and a `py.typed`
marker. Editors and type checkers can inspect concrete document, paragraph,
run, table, section, story, comment, comparison, and layout types.

```sh
python -m pip install mypy
python -m mypy your_application.py
```

The release gate validates the installed package with strict mypy and
`stubtest` in addition to its runtime suite.

## Project links

- [Source repository](https://github.com/tensorbee/rdocx)
- [Issue tracker](https://github.com/tensorbee/rdocx/issues)
- [Changelog](https://github.com/tensorbee/rdocx/blob/main/CHANGELOG.md)
- [Binding specification](https://github.com/tensorbee/rdocx/blob/main/docs/hld/10-bindings-spec.md)

## License

Licensed under either the
[MIT License](https://github.com/tensorbee/rdocx/blob/main/LICENSE) or the
[Apache License 2.0](https://www.apache.org/licenses/LICENSE-2.0), at your
option.
