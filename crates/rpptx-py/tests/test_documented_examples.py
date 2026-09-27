import importlib.metadata
import io
import math
import re
import struct
import sys
import threading
import zipfile
import zlib

import pytest


def _png_chunk(kind, data):
    return struct.pack(">I", len(data)) + kind + data + struct.pack(">I", zlib.crc32(kind + data))


def _tiny_png(red=0x20, green=0x80, blue=0xE0):
    signature = b"\x89PNG\r\n\x1a\n"
    header = struct.pack(">IIBBBBB", 1, 1, 8, 2, 0, 0, 0)
    pixels = zlib.compress(bytes((0, red, green, blue)))
    return (
        signature
        + _png_chunk(b"IHDR", header)
        + _png_chunk(b"IDAT", pixels)
        + _png_chunk(b"IEND", b"")
    )


def _write_tiny_png(path):
    path.write_bytes(_tiny_png())


def test_python_pptx_getting_started_examples_run_with_global_revision_refetches(tmp_path, monkeypatch):
    monkeypatch.chdir(tmp_path)
    _write_tiny_png(tmp_path / "monty-truth.png")

    # Hello World
    from rpptx import Presentation

    prs = Presentation()
    title_slide_layout = prs.slide_layouts[0]
    slide = prs.slides.add_slide(title_slide_layout)
    title = slide.shapes.title
    subtitle = slide.placeholders[1]

    title.text = "Hello, World!"
    subtitle = prs.slides[0].placeholders[1]
    subtitle.text = "python-pptx was here!"

    prs.save("test.pptx")

    # Bullet slide
    from rpptx import Presentation

    prs = Presentation()
    bullet_slide_layout = prs.slide_layouts[1]

    slide = prs.slides.add_slide(bullet_slide_layout)
    shapes = slide.shapes

    title_shape = shapes.title
    body_shape = shapes.placeholders[1]

    title_shape.text = "Adding a Bullet Slide"

    body_shape = prs.slides[0].shapes.placeholders[1]
    tf = body_shape.text_frame
    tf.text = "Find the bullet slide layout"

    tf = prs.slides[0].shapes.placeholders[1].text_frame
    p = tf.add_paragraph()
    p.text = "Use _TextFrame.text for first bullet"
    p = prs.slides[0].shapes.placeholders[1].text_frame.paragraphs[1]
    p.level = 1

    tf = prs.slides[0].shapes.placeholders[1].text_frame
    p = tf.add_paragraph()
    p.text = "Use _TextFrame.add_paragraph() for subsequent bullets"
    p = prs.slides[0].shapes.placeholders[1].text_frame.paragraphs[2]
    p.level = 2

    prs.save("test.pptx")

    # add_textbox()
    from rpptx import Presentation
    from rpptx.util import Inches, Pt

    prs = Presentation()
    blank_slide_layout = prs.slide_layouts[6]
    slide = prs.slides.add_slide(blank_slide_layout)

    left = top = width = height = Inches(1)
    txBox = slide.shapes.add_textbox(left, top, width, height)
    tf = txBox.text_frame

    tf.text = "This is text inside a textbox"

    tf = prs.slides[0].shapes[-1].text_frame
    p = tf.add_paragraph()
    p.text = "This is a second paragraph that's bold"
    p = prs.slides[0].shapes[-1].text_frame.paragraphs[1]
    p.font.bold = True

    tf = prs.slides[0].shapes[-1].text_frame
    p = tf.add_paragraph()
    p.text = "This is a third paragraph that's big"
    p = prs.slides[0].shapes[-1].text_frame.paragraphs[2]
    p.font.size = Pt(40)

    prs.save("test.pptx")

    # add_picture()
    from rpptx import Presentation
    from rpptx.util import Inches

    img_path = "monty-truth.png"

    prs = Presentation()
    blank_slide_layout = prs.slide_layouts[6]
    slide = prs.slides.add_slide(blank_slide_layout)

    left = top = Inches(1)
    pic = slide.shapes.add_picture(img_path, left, top)

    slide = prs.slides[0]
    left = Inches(5)
    height = Inches(5.5)
    pic = slide.shapes.add_picture(img_path, left, top, height=height)

    prs.save("test.pptx")

    # add_shape()
    from rpptx import Presentation
    from rpptx.enum.shapes import MSO_SHAPE
    from rpptx.util import Inches

    prs = Presentation()
    title_only_slide_layout = prs.slide_layouts[5]
    slide = prs.slides.add_slide(title_only_slide_layout)
    shapes = slide.shapes

    shapes.title.text = "Adding an AutoShape"
    shapes = prs.slides[0].shapes

    left = Inches(0.93)
    top = Inches(3.0)
    width = Inches(1.75)
    height = Inches(1.0)

    shape = shapes.add_shape(MSO_SHAPE.PENTAGON, left, top, width, height)
    shape.text = "Step 1"

    left = left + width - Inches(0.4)
    width = Inches(2.0)

    for n in range(2, 6):
        shapes = prs.slides[0].shapes
        shape = shapes.add_shape(MSO_SHAPE.CHEVRON, left, top, width, height)
        shape.text = "Step %d" % n
        left = left + width - Inches(0.4)

    prs.save("test.pptx")

    # add_table()
    from rpptx import Presentation
    from rpptx.util import Inches

    prs = Presentation()
    title_only_slide_layout = prs.slide_layouts[5]
    slide = prs.slides.add_slide(title_only_slide_layout)
    shapes = slide.shapes

    shapes.title.text = "Adding a Table"
    shapes = prs.slides[0].shapes

    rows = cols = 2
    left = top = Inches(2.0)
    width = Inches(6.0)
    height = Inches(0.8)

    table = shapes.add_table(rows, cols, left, top, width, height).table

    table.columns[0].width = Inches(2.0)
    table.columns[1].width = Inches(4.0)

    table.cell(0, 0).text = "Foo"
    table.cell(0, 1).text = "Bar"

    table.cell(1, 0).text = "Baz"
    table.cell(1, 1).text = "Qux"

    prs.save("test.pptx")

    # Extract all text from slides
    from rpptx import Presentation

    path_to_presentation = "test.pptx"
    prs = Presentation(path_to_presentation)

    text_runs = []

    for slide in prs.slides:
        for shape in slide.shapes:
            if not shape.has_text_frame:
                continue
            for paragraph in shape.text_frame.paragraphs:
                for run in paragraph.runs:
                    text_runs.append(run.text)

    assert text_runs == ["Adding a Table"]
    assert (tmp_path / "test.pptx").is_file()


def test_lazy_collections_and_stale_handles_are_loud():
    import rpptx

    prs = rpptx.Presentation()
    first = prs.slides.add_slide(prs.slide_layouts[6])
    held = first.shapes.add_textbox(0, 0, 100, 100)
    prs.slides.add_slide(prs.slide_layouts[6])

    assert len(prs.slides) == 2
    assert list(prs.slides[:1])[0].shapes[-1].text == ""
    with pytest.raises(rpptx.StaleElementError, match=r"revision 2.*revision 3"):
        _ = held.text


def test_omitted_placeholder_index_resolves_as_default_zero():
    import rpptx

    prs = rpptx.Presentation()
    slide = prs.slides.add_slide(prs.slide_layouts[0])
    slide.placeholders[0].text = "default zero"
    assert prs.slides[0].shapes.title.text == "default zero"


def _assert_stale_after_exactly_one_bump(rpptx, operation):
    with pytest.raises(rpptx.StaleElementError) as raised:
        operation()
    revisions = re.search(
        r"revision (\d+).*revision (\d+)", str(raised.value)
    )
    assert revisions is not None
    captured, current = map(int, revisions.groups())
    assert current == captured + 1


def _assert_exact_stale(rpptx, operation, kind, captured, current, recovery):
    expected = (
        f"{kind} handle was created at document revision {captured}, but the "
        f"document is now at revision {current} (a structural change "
        f"invalidated it). {recovery}"
    )
    with pytest.raises(rpptx.StaleElementError) as raised:
        operation()
    assert str(raised.value) == expected


def test_structural_append_invalidates_every_preexisting_path_handle():
    import rpptx

    prs = rpptx.Presentation()
    layouts = prs.slide_layouts
    layout = layouts[6]
    slides = prs.slides
    slide = slides.add_slide(layout)

    _assert_exact_stale(
        rpptx,
        lambda: len(slides),
        "slide collection",
        0,
        1,
        "Re-fetch it with prs.slides.",
    )
    _assert_exact_stale(
        rpptx,
        lambda: len(layouts),
        "slide layout collection",
        0,
        1,
        "Re-fetch it with prs.slide_layouts.",
    )
    _assert_exact_stale(
        rpptx,
        lambda: layout.name,
        "slide layout",
        0,
        1,
        "Re-fetch it with prs.slide_layouts[6].",
    )
    shapes = slide.shapes
    shapes.add_textbox(0, 0, 100, 100)

    _assert_exact_stale(
        rpptx,
        lambda: slide.shapes,
        "slide",
        1,
        2,
        "Re-fetch it with prs.slides[0].",
    )
    _assert_exact_stale(
        rpptx,
        lambda: len(shapes),
        "shape collection",
        1,
        2,
        "Re-fetch it with prs.slides[0].shapes.",
    )


def test_stale_shape_text_and_table_paths_report_exact_recovery():
    import rpptx

    prs = rpptx.Presentation()
    prs.slides.add_slide(prs.slide_layouts[6])
    shape = prs.slides[0].shapes.add_textbox(0, 0, 100, 100)
    shape.text = "before"
    prs.slides[0].shapes.add_table(1, 1, 0, 0, 100, 100)

    slide = prs.slides[0]
    shapes = slide.shapes
    placeholders = slide.placeholders
    shape = shapes[0]
    frame = shape.text_frame
    paragraphs = frame.paragraphs
    paragraph = paragraphs[0]
    runs = paragraph.runs
    run = runs[0]
    font = paragraph.font
    table = shapes[1].table
    columns = table.columns
    column = columns[0]
    cell = table.cell(0, 0)
    prs.slides.add_slide(prs.slide_layouts[6])

    cases = (
        (lambda: slide.shapes, "slide", "Re-fetch it with prs.slides[0]."),
        (lambda: len(shapes), "shape collection", "Re-fetch it with prs.slides[0].shapes."),
        (lambda: placeholders[0], "placeholder collection", "Re-fetch it with prs.slides[0].placeholders."),
        (lambda: shape.text, "shape", "Re-fetch it with prs.slides[0].shapes[0]."),
        (lambda: frame.text, "text frame", "Re-fetch it with prs.slides[0].shapes[0].text_frame."),
        (lambda: len(paragraphs), "paragraph collection", "Re-fetch it with prs.slides[0].shapes[0].text_frame.paragraphs."),
        (lambda: paragraph.text, "paragraph", "Re-fetch it with prs.slides[0].shapes[0].text_frame.paragraphs[0]."),
        (lambda: len(runs), "run collection", "Re-fetch it with prs.slides[0].shapes[0].text_frame.paragraphs[0].runs."),
        (lambda: run.text, "run", "Re-fetch it with prs.slides[0].shapes[0].text_frame.paragraphs[0].runs[0]."),
        (lambda: font.bold, "font", "Re-fetch it with prs.slides[0].shapes[0].text_frame.paragraphs[0].font."),
        (lambda: table.columns, "table", "Re-fetch it with prs.slides[0].shapes[1].table."),
        (lambda: len(columns), "column collection", "Re-fetch it with prs.slides[0].shapes[1].table.columns."),
        (lambda: column.width, "column", "Re-fetch it with prs.slides[0].shapes[1].table.columns[0]."),
        (lambda: cell.text, "cell", "Re-fetch it with prs.slides[0].shapes[1].table.cell(0, 0)."),
    )
    for operation, kind, recovery in cases:
        _assert_exact_stale(rpptx, operation, kind, 4, 5, recovery)


def test_whole_text_replacement_stales_descendant_handles_once():
    import rpptx

    prs = rpptx.Presentation()
    slide = prs.slides.add_slide(prs.slide_layouts[6])
    shape = slide.shapes.add_textbox(0, 0, 100, 100)
    shape.text = "before"
    shape = prs.slides[0].shapes[-1]
    frame = shape.text_frame
    paragraph = frame.paragraphs[0]
    run = paragraph.runs[0]
    shape.text = "after"
    for operation in (
        lambda: shape.text,
        lambda: frame.text,
        lambda: paragraph.text,
        lambda: run.text,
    ):
        _assert_stale_after_exactly_one_bump(rpptx, operation)
    assert prs.slides[0].shapes[-1].text == "after"

    prs = rpptx.Presentation()
    slide = prs.slides.add_slide(prs.slide_layouts[6])
    frame = slide.shapes.add_textbox(0, 0, 100, 100).text_frame
    frame.text = "before"
    frame = prs.slides[0].shapes[-1].text_frame
    paragraph = frame.paragraphs[0]
    run = paragraph.runs[0]
    frame.text = "after"
    for operation in (lambda: frame.text, lambda: paragraph.text, lambda: run.text):
        _assert_stale_after_exactly_one_bump(rpptx, operation)
    assert prs.slides[0].shapes[-1].text_frame.text == "after"

    prs = rpptx.Presentation()
    slide = prs.slides.add_slide(prs.slide_layouts[6])
    frame = slide.shapes.add_textbox(0, 0, 100, 100).text_frame
    frame.text = "before"
    frame = prs.slides[0].shapes[-1].text_frame
    paragraph = frame.paragraphs[0]
    run = paragraph.runs[0]
    paragraph.text = "after"
    for operation in (lambda: paragraph.text, lambda: run.text):
        _assert_stale_after_exactly_one_bump(rpptx, operation)
    assert prs.slides[0].shapes[-1].text_frame.paragraphs[0].text == "after"


def test_nested_group_paths_report_exact_recovery(tmp_path):
    if importlib.util.find_spec("pptx") is None:
        pytest.skip("python-pptx oracle is installed only for the differential gate")

    import rpptx
    from pptx import Presentation as OraclePresentation

    source = tmp_path / "nested-groups.pptx"
    oracle = OraclePresentation()
    slide = oracle.slides.add_slide(oracle.slide_layouts[6])
    outer = slide.shapes.add_group_shape()
    inner = outer.shapes.add_group_shape()
    textbox = inner.shapes.add_textbox(0, 0, 100, 100)
    textbox.text = "nested"
    oracle.save(source)

    prs = rpptx.Presentation(source)
    inner = prs.slides[0].shapes[0].shapes[0]
    nested_shapes = inner.shapes
    shape = nested_shapes[0]
    frame = shape.text_frame
    paragraphs = frame.paragraphs
    paragraph = paragraphs[0]
    runs = paragraph.runs
    run = runs[0]
    font = paragraph.font
    prs.slides.add_slide(prs.slide_layouts[6])

    cases = (
        (lambda: len(nested_shapes), "shape collection", "Re-fetch it with prs.slides[0].shapes[0].shapes[0].shapes."),
        (lambda: shape.text, "shape", "Re-fetch it with prs.slides[0].shapes[0].shapes[0].shapes[0]."),
        (lambda: frame.text, "text frame", "Re-fetch it with prs.slides[0].shapes[0].shapes[0].shapes[0].text_frame."),
        (lambda: len(paragraphs), "paragraph collection", "Re-fetch it with prs.slides[0].shapes[0].shapes[0].shapes[0].text_frame.paragraphs."),
        (lambda: paragraph.text, "paragraph", "Re-fetch it with prs.slides[0].shapes[0].shapes[0].shapes[0].text_frame.paragraphs[0]."),
        (lambda: len(runs), "run collection", "Re-fetch it with prs.slides[0].shapes[0].shapes[0].shapes[0].text_frame.paragraphs[0].runs."),
        (lambda: run.text, "run", "Re-fetch it with prs.slides[0].shapes[0].shapes[0].shapes[0].text_frame.paragraphs[0].runs[0]."),
        (lambda: font.bold, "font", "Re-fetch it with prs.slides[0].shapes[0].shapes[0].shapes[0].text_frame.paragraphs[0].font."),
    )
    for operation, kind, recovery in cases:
        _assert_exact_stale(rpptx, operation, kind, 0, 1, recovery)


def test_unit_constructors_truncate_fractional_values_toward_zero():
    import rpptx
    from rpptx.enum.shapes import MSO_SHAPE
    from rpptx.util import Inches, Length, Pt

    for constructor, factor in ((Inches, 914_400.0), (Pt, 12_700.0)):
        assert constructor(1.75 / factor) == 1
        assert constructor(-1.75 / factor) == -1
    assert Length(914_400).inches == 1.0
    assert Length(12_700).pt == 1.0
    assert Length(-1).emu == -1
    assert MSO_SHAPE.PENTAGON == 51
    assert MSO_SHAPE.CHEVRON == 52
    assert issubclass(rpptx.PackageError, rpptx.RpptxError)
    assert issubclass(rpptx.XmlError, rpptx.RpptxError)
    assert issubclass(rpptx.StaleElementError, rpptx.RpptxError)


def test_missing_package_raises_the_named_package_error(tmp_path):
    import rpptx

    with pytest.raises(rpptx.PackageError):
        rpptx.Presentation(tmp_path / "missing.pptx")


def _author_documented_decks(Presentation, Inches, Pt, MSO_SHAPE, root, image_path):
    root.mkdir()
    paths = {name: root / f"{name}.pptx" for name in (
        "hello", "bullet", "textbox", "picture", "shapes", "table"
    )}

    prs = Presentation()
    slide = prs.slides.add_slide(prs.slide_layouts[0])
    title = slide.shapes.title
    subtitle = slide.placeholders[1]
    title.text = "Hello, World!"
    subtitle = prs.slides[0].placeholders[1]
    subtitle.text = "python-pptx was here!"
    prs.save(paths["hello"])

    prs = Presentation()
    slide = prs.slides.add_slide(prs.slide_layouts[1])
    shapes = slide.shapes
    shapes.title.text = "Adding a Bullet Slide"
    shapes = prs.slides[0].shapes
    frame = shapes.placeholders[1].text_frame
    frame.text = "Find the bullet slide layout"
    frame = prs.slides[0].shapes.placeholders[1].text_frame
    paragraph = frame.add_paragraph()
    paragraph.text = "Use _TextFrame.text for first bullet"
    paragraph = prs.slides[0].shapes.placeholders[1].text_frame.paragraphs[1]
    paragraph.level = 1
    frame = prs.slides[0].shapes.placeholders[1].text_frame
    paragraph = frame.add_paragraph()
    paragraph.text = "Use _TextFrame.add_paragraph() for subsequent bullets"
    paragraph = prs.slides[0].shapes.placeholders[1].text_frame.paragraphs[2]
    paragraph.level = 2
    prs.save(paths["bullet"])

    prs = Presentation()
    slide = prs.slides.add_slide(prs.slide_layouts[6])
    frame = slide.shapes.add_textbox(Inches(1), Inches(1), Inches(1), Inches(1)).text_frame
    frame.text = "This is text inside a textbox"
    frame = prs.slides[0].shapes[-1].text_frame
    paragraph = frame.add_paragraph()
    paragraph.text = "This is a second paragraph that's bold"
    paragraph = prs.slides[0].shapes[-1].text_frame.paragraphs[1]
    paragraph.font.bold = True
    frame = prs.slides[0].shapes[-1].text_frame
    paragraph = frame.add_paragraph()
    paragraph.text = "This is a third paragraph that's big"
    paragraph = prs.slides[0].shapes[-1].text_frame.paragraphs[2]
    paragraph.font.size = Pt(40)
    prs.save(paths["textbox"])

    prs = Presentation()
    slide = prs.slides.add_slide(prs.slide_layouts[6])
    slide.shapes.add_picture(str(image_path), Inches(1), Inches(1))
    slide = prs.slides[0]
    slide.shapes.add_picture(
        str(image_path), Inches(5), Inches(1), height=Inches(5.5)
    )
    prs.save(paths["picture"])

    prs = Presentation()
    slide = prs.slides.add_slide(prs.slide_layouts[5])
    shapes = slide.shapes
    shapes.title.text = "Adding an AutoShape"
    shapes = prs.slides[0].shapes
    left = Inches(0.93)
    top = Inches(3.0)
    width = Inches(1.75)
    height = Inches(1.0)
    shape = shapes.add_shape(MSO_SHAPE.PENTAGON, left, top, width, height)
    shape.text = "Step 1"
    left = left + width - Inches(0.4)
    width = Inches(2.0)
    for number in range(2, 6):
        shapes = prs.slides[0].shapes
        shape = shapes.add_shape(MSO_SHAPE.CHEVRON, left, top, width, height)
        shape.text = f"Step {number}"
        left = left + width - Inches(0.4)
    prs.save(paths["shapes"])

    prs = Presentation()
    slide = prs.slides.add_slide(prs.slide_layouts[5])
    shapes = slide.shapes
    shapes.title.text = "Adding a Table"
    shapes = prs.slides[0].shapes
    table = shapes.add_table(
        2, 2, Inches(2), Inches(2), Inches(6), Inches(0.8)
    ).table
    table.columns[0].width = Inches(2)
    table.columns[1].width = Inches(4)
    for row, values in enumerate((("Foo", "Bar"), ("Baz", "Qux"))):
        for column, value in enumerate(values):
            table.cell(row, column).text = value
    prs.save(paths["table"])
    return paths


def _normalized_table(table):
    columns = len(table.columns)
    rows = []
    row = 0
    while True:
        try:
            rows.append(tuple(table.cell(row, column).text for column in range(columns)))
        except IndexError:
            break
        row += 1
    return tuple(int(column.width) for column in table.columns), tuple(rows)


def _normalized_presentation(prs):
    slides = []
    for slide in prs.slides:
        shapes = []
        for shape in slide.shapes:
            paragraphs = ()
            if shape.has_text_frame:
                paragraphs = tuple(
                    (
                        paragraph.text,
                        paragraph.level,
                        paragraph.font.bold,
                        int(paragraph.font.size) if paragraph.font.size is not None else None,
                        tuple(run.text for run in paragraph.runs),
                    )
                    for paragraph in shape.text_frame.paragraphs
                )
            shapes.append(
                (
                    shape.has_text_frame,
                    paragraphs,
                    shape.has_table,
                    _normalized_table(shape.table) if shape.has_table else None,
                )
            )
        slides.append(tuple(shapes))
    return tuple(slides)


def _normalized_text_extraction(prs):
    return tuple(
        run.text
        for slide in prs.slides
        for shape in slide.shapes
        if shape.has_text_frame
        for paragraph in shape.text_frame.paragraphs
        for run in paragraph.runs
    )


def _normalized_documented_records(Presentation, paths):
    records = {
        name: _normalized_presentation(Presentation(path))
        for name, path in paths.items()
    }
    records["extract"] = _normalized_text_extraction(Presentation(paths["table"]))
    return records


def _oracle_writer_contract(Presentation, paths):
    def auto_shape_type(shape):
        if int(shape.shape_type) != 1:
            return None
        try:
            return int(shape.auto_shape_type)
        except (AttributeError, TypeError, ValueError):
            return None

    records = {}
    for name, path in paths.items():
        prs = Presentation(path)
        records[name] = tuple(
            tuple(
                (
                    int(shape.shape_type),
                    (int(shape.left), int(shape.top), int(shape.width), int(shape.height)),
                    auto_shape_type(shape),
                    int(shape.placeholder_format.idx) if shape.is_placeholder else None,
                )
                for shape in slide.shapes
            )
            for slide in prs.slides
        )
    return records


def test_pinned_python_pptx_bidirectional_seven_example_records(tmp_path):
    if importlib.util.find_spec("pptx") is None:
        pytest.skip("python-pptx oracle is installed only for the differential gate")
    assert importlib.metadata.version("python-pptx") == "1.0.2"

    import rpptx
    from rpptx.enum.shapes import MSO_SHAPE as RpptxShape
    from rpptx.util import Inches as RpptxInches
    from rpptx.util import Pt as RpptxPt
    from pptx import Presentation as OraclePresentation
    from pptx.enum.shapes import MSO_SHAPE as OracleShape
    from pptx.util import Inches as OracleInches
    from pptx.util import Pt as OraclePt

    image_path = tmp_path / "monty-truth.png"
    _write_tiny_png(image_path)
    authored = (
        _author_documented_decks(
            rpptx.Presentation,
            RpptxInches,
            RpptxPt,
            RpptxShape,
            tmp_path / "rpptx-authored",
            image_path,
        ),
        _author_documented_decks(
            OraclePresentation,
            OracleInches,
            OraclePt,
            OracleShape,
            tmp_path / "python-pptx-authored",
            image_path,
        ),
    )
    normalized = []
    writer_contracts = []
    for paths in authored:
        rpptx_records = _normalized_documented_records(rpptx.Presentation, paths)
        oracle_records = _normalized_documented_records(OraclePresentation, paths)
        assert rpptx_records == oracle_records
        normalized.append(rpptx_records)
        writer_contracts.append(_oracle_writer_contract(OraclePresentation, paths))
    assert normalized[0] == normalized[1]
    assert writer_contracts[0] == writer_contracts[1]
    assert normalized[0]["extract"] == ("Adding a Table",)


def _add_speaker_notes(source, target, text):
    with zipfile.ZipFile(source) as archive:
        parts = {name: archive.read(name) for name in archive.namelist()}

    content_types = parts["[Content_Types].xml"].decode()
    parts["[Content_Types].xml"] = content_types.replace(
        "</Types>",
        '<Override PartName="/ppt/notesSlides/notesSlide1.xml" '
        'ContentType="application/vnd.openxmlformats-officedocument.presentationml.notesSlide+xml"/>'
        "</Types>",
    ).encode()
    slide_rels = parts["ppt/slides/_rels/slide1.xml.rels"].decode()
    parts["ppt/slides/_rels/slide1.xml.rels"] = slide_rels.replace(
        "</Relationships>",
        '<Relationship Id="rId2" '
        'Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/notesSlide" '
        'Target="../notesSlides/notesSlide1.xml"/>'
        "</Relationships>",
    ).encode()
    parts["ppt/notesSlides/notesSlide1.xml"] = (
        '<?xml version="1.0" encoding="UTF-8" standalone="yes"?>'
        '<p:notes xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" '
        'xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main">'
        "<p:cSld><p:spTree><p:nvGrpSpPr><p:cNvPr id=\"1\" name=\"\"/>"
        "<p:cNvGrpSpPr/><p:nvPr/></p:nvGrpSpPr><p:grpSpPr/>"
        "<p:sp><p:nvSpPr><p:cNvPr id=\"2\" name=\"Notes Placeholder\"/>"
        "<p:cNvSpPr/><p:nvPr><p:ph type=\"body\" idx=\"1\"/></p:nvPr>"
        "</p:nvSpPr><p:spPr/><p:txBody><a:bodyPr/><a:lstStyle/><a:p>"
        '<a:r><a:rPr b="1"><a:extLst>'
        '<a:ext uri="{6D487B31-4C56-4F45-AED3-4C741AC43E77}">'
        '<x:payload xmlns:x="urn:rdocx:test"/></a:ext></a:extLst></a:rPr>'
        f"<a:t>{text}</a:t></a:r></a:p></p:txBody></p:sp>"
        "</p:spTree></p:cSld><p:clrMapOvr><a:masterClrMapping/>"
        "</p:clrMapOvr></p:notes>"
    ).encode()
    parts["ppt/notesSlides/_rels/notesSlide1.xml.rels"] = (
        '<?xml version="1.0" encoding="UTF-8" standalone="yes"?>'
        '<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">'
        '<Relationship Id="rId1" '
        'Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/notesMaster" '
        'Target="../notesMasters/notesMaster1.xml"/>'
        '<Relationship Id="rId2" '
        'Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/slide" '
        'Target="../slides/slide1.xml"/></Relationships>'
    ).encode()
    with zipfile.ZipFile(target, "w", zipfile.ZIP_DEFLATED) as archive:
        for name, data in parts.items():
            archive.writestr(name, data)


def test_presentation_render_comments_and_notes_match_native_snapshots(tmp_path):
    import rpptx

    author_id = "{11111111-1111-1111-1111-111111111111}"
    comment_id = "{22222222-2222-2222-2222-222222222222}"
    reply_id = "{33333333-3333-3333-3333-333333333333}"
    second_reply_id = "{44444444-4444-4444-4444-444444444444}"
    second_comment_id = "{55555555-5555-5555-5555-555555555555}"
    created = "2026-09-14T10:30:00Z"

    presentation = rpptx.Presentation()
    slide = presentation.slides.add_slide(presentation.slide_layouts[6])
    source = tmp_path / "source.pptx"
    with_notes = tmp_path / "with-notes.pptx"
    presentation.save(source)
    _add_speaker_notes(source, with_notes, "F-X094e speaker note")
    presentation = rpptx.Presentation(with_notes)
    slide = presentation.slides[0]

    pdf = presentation.to_pdf()
    slide_png = presentation.render_slide_to_png(0, dpi=72.0)
    assert pdf.startswith(b"%PDF-")
    assert slide_png is not None
    assert slide_png.startswith(b"\x89PNG\r\n\x1a\n")
    assert presentation.render_all_slides(dpi=72.0) == [slide_png]
    assert presentation.to_notes_pdf().startswith(b"%PDF-")
    assert presentation.render_all_notes(dpi=72.0)[0].startswith(
        b"\x89PNG\r\n\x1a\n"
    )
    assert slide.notes_text == "F-X094e speaker note"

    with pytest.raises(rpptx.RpptxError, match="GUID"):
        presentation.add_comment_author(
            id="not-a-guid",
            name="Invalid",
            user_id="invalid@example.com",
            provider_id="local",
        )
    assert slide.notes_text == "F-X094e speaker note"
    with pytest.raises(rpptx.RpptxError, match="unknown comment author"):
        slide.add_comment(
            id=comment_id,
            author_id="{99999999-9999-9999-9999-999999999999}",
            created=created,
            text="Must remain atomic",
        )
    assert slide.notes_text == "F-X094e speaker note"

    presentation.add_comment_author(
        id=author_id,
        name="Ada Lovelace",
        initials="AL",
        user_id="ada@example.com",
        provider_id="local",
    )
    author = presentation.comment_authors[0]
    assert (
        author.id,
        author.name,
        author.initials,
        author.user_id,
        author.provider_id,
    ) == (author_id, "Ada Lovelace", "AL", "ada@example.com", "local")
    with pytest.raises(AttributeError):
        author.name = "mutated"

    slide = presentation.slides[0]
    slide.add_comment(
        id=comment_id,
        author_id=author_id,
        created=created,
        text="Review this slide",
    )
    slide = presentation.slides[0]
    slide.reply_to_comment(
        comment_id,
        id=reply_id,
        author_id=author_id,
        created=created,
        text="First reply",
    )
    slide = presentation.slides[0]
    slide.reply_to_comment(
        comment_id,
        id=second_reply_id,
        author_id=author_id,
        created=created,
        text="Second reply",
    )
    slide = presentation.slides[0]
    slide.add_comment(
        id=second_comment_id,
        author_id=author_id,
        created=created,
        text="Move me first",
    )
    slide = presentation.slides[0]
    slide.move_comment(1, 0)
    slide = presentation.slides[0]
    slide.move_reply(comment_id, 1, 0)

    comments = presentation.slides[0].comments
    assert tuple(comment.id for comment in comments) == (second_comment_id, comment_id)
    comment = comments[1]
    assert (comment.id, comment.author_id, comment.status, comment.created, comment.text) == (
        comment_id,
        author_id,
        None,
        created,
        "Review this slide",
    )
    assert tuple(reply.id for reply in comment.replies) == (second_reply_id, reply_id)
    assert comment.replies[0].text == "Second reply"
    with pytest.raises(AttributeError):
        comment.text = "mutated"

    output = tmp_path / "comments.pptx"
    presentation.save(output)
    reopened = rpptx.Presentation(output)
    assert reopened.comment_authors == presentation.comment_authors
    assert reopened.slides[0].comments == presentation.slides[0].comments


def _text_layout_deck(tmp_path, width, text, body_properties=None):
    import rpptx

    presentation = rpptx.Presentation()
    slide = presentation.slides.add_slide(presentation.slide_layouts[6])
    shape = slide.shapes.add_textbox(rpptx.Inches(1), rpptx.Inches(1), width, rpptx.Pt(36))
    shape.text = text
    if body_properties is None:
        return presentation
    source = tmp_path / "text-layout-source.pptx"
    target = tmp_path / "text-layout.pptx"
    presentation.save(source)
    with zipfile.ZipFile(source) as archive:
        parts = {name: archive.read(name) for name in archive.namelist()}
    slide_xml = parts["ppt/slides/slide1.xml"]
    assert slide_xml.count(b"<a:bodyPr/>") == 1
    parts["ppt/slides/slide1.xml"] = slide_xml.replace(b"<a:bodyPr/>", body_properties)
    with zipfile.ZipFile(target, "w", zipfile.ZIP_DEFLATED) as archive:
        for name, data in parts.items():
            archive.writestr(name, data)
    return rpptx.Presentation(target)


def test_text_layout_reports_the_renderer_fit_at_full_and_reduced_width(tmp_path):
    import rpptx

    assert rpptx.Presentation().text_layout() == ()
    presentation = _text_layout_deck(tmp_path, rpptx.Inches(4), "Fits in one line")
    shape = presentation.slides[0].shapes[0]
    (frame,) = presentation.text_layout()
    assert isinstance(frame, rpptx.TextFrameLayout)
    assert (frame.slide_index, frame.shape_id, frame.name) == (0, shape.shape_id, shape.name)
    assert (frame.autofit, frame.font_scale, frame.overflow) == ("none", 1.0, False)
    assert isinstance(frame.frame, rpptx.BoundingBox)
    assert (frame.frame.x, frame.frame.y, frame.frame.width, frame.frame.height) == (
        72.0,
        72.0,
        288.0,
        36.0,
    )
    assert (frame.usable.x, frame.usable.y) == pytest.approx((79.2, 75.6))
    assert (frame.usable.width, frame.usable.height) == pytest.approx((273.6, 28.8))
    (line,) = frame.lines
    assert isinstance(line, rpptx.TextLineLayout)
    assert (line.paragraph_index, line.text, line.font_size) == (0, "Fits in one line", 18.0)
    assert line.bounds.x == pytest.approx(79.2)
    assert frame.usable.y <= line.bounds.y < line.baseline < line.bounds.y + line.bounds.height
    assert frame.height == pytest.approx(line.bounds.height)
    assert presentation.text_layout() == (frame,)
    with pytest.raises(AttributeError):
        frame.overflow = True  # type: ignore[misc]

    presentation.slides[0].shapes[0].text_frame.add_paragraph().text = "One line too many"
    (two_lines,) = presentation.text_layout()
    assert [(line.paragraph_index, line.text) for line in two_lines.lines] == [
        (0, "Fits in one line"),
        (1, "One line too many"),
    ]
    assert two_lines.overflow
    assert two_lines.height > two_lines.usable.height

    measured = _text_layout_deck(tmp_path, rpptx.Inches(8), "Fits only at full width")
    natural = measured.text_layout()[0].lines[0].bounds.width
    tight = _text_layout_deck(
        tmp_path, math.ceil((natural / 0.975 + 14.4) * 12_700), "Fits only at full width"
    )
    (full,) = tight.text_layout()
    (narrower,) = tight.text_layout(width_factor=0.95)
    assert (len(full.lines), full.overflow) == (1, False)
    assert (len(narrower.lines), narrower.overflow) == (2, True)
    assert narrower.usable.width == pytest.approx(full.usable.width * 0.95)
    assert "".join(line.text for line in narrower.lines) == "Fits only at full width"
    with pytest.raises(rpptx.RpptxError, match="width factor"):
        tight.text_layout(width_factor=0.0)
    with pytest.raises(TypeError):
        tight.text_layout(0.95)  # type: ignore[misc]

    scaled = _text_layout_deck(
        tmp_path,
        rpptx.Inches(4),
        "Stored scale",
        b'<a:bodyPr lIns="0" tIns="0" rIns="0" bIns="0"><a:normAutofit fontScale="50000"/></a:bodyPr>',
    )
    (scaled_frame,) = scaled.text_layout()
    assert (scaled_frame.autofit, scaled_frame.font_scale) == ("normal", 0.5)
    assert (scaled_frame.usable.x, scaled_frame.usable.height) == (72.0, 36.0)
    assert scaled_frame.lines[0].font_size == 9.0
    grown = _text_layout_deck(
        tmp_path, rpptx.Inches(4), "Stored extent", b"<a:bodyPr><a:spAutoFit/></a:bodyPr>"
    )
    assert grown.text_layout()[0].autofit == "shape"

    assert _assert_rpptx_releases_gil(lambda: tight.text_layout()) == (full,)


def test_notes_mutation_preserves_text_through_save_and_reopen(tmp_path):
    import rpptx

    presentation = rpptx.Presentation()
    presentation.slides.add_slide(presentation.slide_layouts[6])
    source = tmp_path / "source.pptx"
    with_notes = tmp_path / "with-notes.pptx"
    output = tmp_path / "updated-notes.pptx"
    presentation.save(source)
    _add_speaker_notes(source, with_notes, "Draft speaker note")

    presentation = rpptx.Presentation(with_notes)
    held = presentation.slides[0]
    held.notes_text = "Final speaker note"
    assert presentation.slides[0].notes_text == "Final speaker note"
    with pytest.raises(rpptx.StaleElementError):
        _ = held.notes_text

    presentation.save(output)
    assert rpptx.Presentation(output).slides[0].notes_text == "Final speaker note"
    with zipfile.ZipFile(output) as archive:
        notes_xml = archive.read("ppt/notesSlides/notesSlide1.xml").decode()
    assert '<p:ph type="body" idx="1"' in notes_xml
    assert 'b="1"' in notes_xml
    assert "x:payload" in notes_xml

    without_notes = rpptx.Presentation()
    held = without_notes.slides.add_slide(without_notes.slide_layouts[6])
    assert held.notes_text is None
    held.notes_text = "Created speaker note"
    assert without_notes.slides[0].notes_text == "Created speaker note"
    with pytest.raises(rpptx.StaleElementError):
        _ = held.notes_text


def test_python_round_three_authoring_and_inspection_is_typed_and_lossless(tmp_path):
    import rpptx

    source = tmp_path / "round-three-source.pptx"
    rich = tmp_path / "round-three-rich.pptx"
    with_notes = tmp_path / "round-three-notes.pptx"
    output = tmp_path / "round-three-output.pptx"
    presentation = rpptx.Presentation()
    slide = presentation.slides.add_slide(presentation.slide_layouts[6])
    shape = slide.shapes.add_textbox(
        rpptx.Inches(1), rpptx.Inches(2), rpptx.Inches(3), rpptx.Inches(1)
    )
    shape.text = "formatted"
    presentation.save(source)

    with zipfile.ZipFile(source) as archive:
        parts = {name: archive.read(name) for name in archive.namelist()}
    slide_xml = parts["ppt/slides/slide1.xml"]
    slide_xml = slide_xml.replace(b"<a:bodyPr/>", b"<a:bodyPr><a:spAutoFit/></a:bodyPr>", 1)
    slide_xml = slide_xml.replace(
        b"<a:r>",
        b'<a:r><a:rPr sz="1800"><a:solidFill><a:srgbClr val="123456"/>'
        b'</a:solidFill><a:latin typeface="Aptos"/><a:extLst><a:ext uri="round-three">'
        b'<x:payload xmlns:x="urn:rdocx:test"/></a:ext></a:extLst></a:rPr>',
        1,
    )
    parts["ppt/slides/slide1.xml"] = slide_xml
    with zipfile.ZipFile(rich, "w", zipfile.ZIP_DEFLATED) as archive:
        for name, data in parts.items():
            archive.writestr(name, data)
    _add_speaker_notes(rich, with_notes, "draft note")

    presentation = rpptx.Presentation(with_notes)
    shape = presentation.slides[0].shapes[0]
    assert (shape.left, shape.top, shape.width, shape.height) == (
        rpptx.Inches(1),
        rpptx.Inches(2),
        rpptx.Inches(3),
        rpptx.Inches(1),
    )
    assert shape.shape_id is not None
    assert shape.name is not None
    assert shape.text_frame.autofit == "shape"
    run = shape.text_frame.paragraphs[0].runs[0]
    assert run.font.name == "Aptos"
    assert run.font.size == rpptx.Pt(18)
    assert run.font.color == "123456"
    run.text = "updated"
    presentation.slides[0].notes_text = "final note"
    presentation.save(output)

    reopened = rpptx.Presentation(output)
    assert reopened.slides[0].shapes[0].text == "updated"
    assert reopened.slides[0].notes_text == "final note"
    with zipfile.ZipFile(output) as archive:
        preserved = archive.read("ppt/slides/slide1.xml")
    assert b"urn:rdocx:test" in preserved
    assert b'sz="1800"' in preserved
    assert b"updated" in preserved


def _assert_rpptx_releases_gil(operation):
    gate = threading.Lock()
    gate.acquire()
    ready = threading.Event()
    progressed = threading.Event()

    def wait_for_detached_call():
        ready.set()
        gate.acquire()
        progressed.set()

    old_switch_interval = sys.getswitchinterval()
    sys.setswitchinterval(60.0)
    worker = threading.Thread(target=wait_for_detached_call)
    try:
        worker.start()
        assert ready.wait(timeout=5.0)
        gate.release()
        result = None
        for _ in range(64):
            result = operation()
            if progressed.is_set():
                break
        progressed_during_call = progressed.is_set()
    finally:
        if not worker.is_alive() and gate.locked():
            gate.release()
        worker.join(timeout=5.0)
        if gate.locked():
            gate.release()
        sys.setswitchinterval(old_switch_interval)

    assert not worker.is_alive()
    assert progressed_during_call, "Python worker made no progress during native call"
    assert result is not None
    return result


def test_slide_and_notes_rendering_release_the_gil():
    import rpptx

    presentation = rpptx.Presentation()
    for _ in range(3):
        presentation.slides.add_slide(presentation.slide_layouts[6])

    slides = _assert_rpptx_releases_gil(
        lambda: presentation.render_all_slides(dpi=72.0)
    )
    notes = _assert_rpptx_releases_gil(
        lambda: presentation.render_all_notes(dpi=72.0)
    )

    assert len(slides) == 3
    assert len(notes) == 3


def _textbox_presentation(rpptx):
    presentation = rpptx.Presentation()
    slide = presentation.slides.add_slide(presentation.slide_layouts[6])
    shape = slide.shapes.add_textbox(
        rpptx.Inches(1), rpptx.Inches(1), rpptx.Inches(4), rpptx.Inches(2)
    )
    shape.text = "hello"
    return presentation


def _text_body_xml(path):
    with zipfile.ZipFile(path) as archive:
        slide_xml = archive.read("ppt/slides/slide1.xml").decode()
    return slide_xml[slide_xml.index("<p:txBody>") : slide_xml.index("</p:txBody>")]


def _replace_in_slide(source, target, old, new):
    with zipfile.ZipFile(source) as archive:
        parts = {name: archive.read(name) for name in archive.namelist()}
    slide_xml = parts["ppt/slides/slide1.xml"].decode()
    assert old in slide_xml
    parts["ppt/slides/slide1.xml"] = slide_xml.replace(old, new, 1).encode()
    with zipfile.ZipFile(target, "w", zipfile.ZIP_DEFLATED) as archive:
        for name, data in parts.items():
            archive.writestr(name, data)


def test_text_frame_margins_anchor_wrap_and_auto_size_round_trip_and_clear(tmp_path):
    import rpptx
    from rpptx import MSO_ANCHOR, MSO_AUTO_SIZE

    presentation = _textbox_presentation(rpptx)
    frame = presentation.slides[0].shapes[0].text_frame
    assert (
        frame.margin_left,
        frame.margin_right,
        frame.margin_top,
        frame.margin_bottom,
        frame.vertical_anchor,
        frame.word_wrap,
        frame.auto_size,
    ) == (None, None, None, None, None, None, None)

    frame.margin_left = rpptx.Inches(0.25)
    frame.margin_right = 0
    frame.margin_top = rpptx.Pt(3)
    frame.margin_bottom = rpptx.Pt(4)
    frame.vertical_anchor = MSO_ANCHOR.BOTTOM
    frame.word_wrap = False
    frame.auto_size = MSO_AUTO_SIZE.SHAPE_TO_FIT_TEXT
    assert isinstance(frame.margin_left, rpptx.Length)
    output = tmp_path / "frame.pptx"
    presentation.save(output)

    assert _text_body_xml(output).startswith(
        '<p:txBody><a:bodyPr lIns="228600" tIns="38100" rIns="0" bIns="50800" '
        'anchor="b" wrap="none"><a:spAutoFit/></a:bodyPr>'
    )
    presentation = rpptx.Presentation(output)
    frame = presentation.slides[0].shapes[0].text_frame
    assert (
        frame.margin_left,
        frame.margin_right,
        frame.margin_top,
        frame.margin_bottom,
    ) == (rpptx.Inches(0.25), 0, rpptx.Pt(3), rpptx.Pt(4))
    assert frame.vertical_anchor is MSO_ANCHOR.BOTTOM
    assert frame.word_wrap is False
    assert frame.auto_size is MSO_AUTO_SIZE.SHAPE_TO_FIT_TEXT
    assert frame.autofit == "shape"

    for name in (
        "margin_left",
        "margin_right",
        "margin_top",
        "margin_bottom",
        "vertical_anchor",
        "word_wrap",
        "auto_size",
    ):
        setattr(frame, name, None)
    presentation.save(output)
    assert _text_body_xml(output).startswith("<p:txBody><a:bodyPr/>")


def test_text_frame_margins_read_universal_measures_and_the_justified_anchors(tmp_path):
    import rpptx
    from rpptx import MSO_ANCHOR

    source = tmp_path / "source.pptx"
    measured = tmp_path / "measured.pptx"
    _textbox_presentation(rpptx).save(source)
    _replace_in_slide(
        source,
        measured,
        "<a:bodyPr/>",
        '<a:bodyPr lIns="0.1in" rIns="1pc" tIns="2mm" bIns="3pt" anchor="dist"/>',
    )

    frame = rpptx.Presentation(measured).slides[0].shapes[0].text_frame
    assert (
        frame.margin_left,
        frame.margin_right,
        frame.margin_top,
        frame.margin_bottom,
    ) == (91440, 152400, 72000, 38100)
    assert frame.vertical_anchor is MSO_ANCHOR.DISTRIBUTE


def test_paragraph_alignment_spacing_and_indents_round_trip_and_clear(tmp_path):
    import rpptx
    from rpptx import PP_ALIGN

    presentation = _textbox_presentation(rpptx)
    paragraph = presentation.slides[0].shapes[0].text_frame.paragraphs[0]
    names = (
        "alignment",
        "line_spacing",
        "space_before",
        "space_after",
        "left_indent",
        "right_indent",
        "first_line_indent",
    )
    assert [getattr(paragraph, name) for name in names] == [None] * len(names)

    paragraph.alignment = PP_ALIGN.JUSTIFY
    paragraph.line_spacing = 1.25
    paragraph.space_before = rpptx.Pt(6)
    paragraph.space_after = 0.5
    paragraph.left_indent = rpptx.Inches(0.5)
    paragraph.right_indent = rpptx.Inches(0.25)
    paragraph.first_line_indent = -rpptx.Inches(0.25)
    output = tmp_path / "paragraph.pptx"
    presentation.save(output)

    assert (
        '<a:pPr marL="457200" marR="228600" indent="-228600" algn="just">'
        '<a:lnSpc><a:spcPct val="125000"/></a:lnSpc>'
        '<a:spcBef><a:spcPts val="600"/></a:spcBef>'
        '<a:spcAft><a:spcPct val="50000"/></a:spcAft></a:pPr>'
    ) in _text_body_xml(output)
    presentation = rpptx.Presentation(output)
    paragraph = presentation.slides[0].shapes[0].text_frame.paragraphs[0]
    assert paragraph.alignment is PP_ALIGN.JUSTIFY
    assert paragraph.line_spacing == 1.25 and isinstance(paragraph.line_spacing, float)
    assert paragraph.space_before == rpptx.Pt(6)
    assert isinstance(paragraph.space_before, rpptx.Length)
    assert paragraph.space_after == 0.5 and isinstance(paragraph.space_after, float)
    assert paragraph.left_indent == rpptx.Inches(0.5)
    assert paragraph.right_indent == rpptx.Inches(0.25)
    assert paragraph.first_line_indent == -rpptx.Inches(0.25)

    paragraph.line_spacing = rpptx.Pt(18)
    paragraph.space_after = rpptx.Pt(3)
    assert paragraph.line_spacing == rpptx.Pt(18)
    assert isinstance(paragraph.line_spacing, rpptx.Length)
    assert paragraph.space_after == rpptx.Pt(3)

    for name in names:
        setattr(paragraph, name, None)
    presentation.save(output)
    assert "<a:pPr/>" in _text_body_xml(output)


def test_a_deck_with_line_spacing_above_two_lines_opens_and_reads_back(tmp_path):
    import rpptx

    source = tmp_path / "source.pptx"
    spaced = tmp_path / "spaced.pptx"
    _textbox_presentation(rpptx).save(source)
    _replace_in_slide(
        source,
        spaced,
        "<a:p><a:r>",
        '<a:p><a:pPr><a:lnSpc><a:spcPct val="250000"/></a:lnSpc>'
        '<a:spcBef><a:spcPct val="13200000"/></a:spcBef></a:pPr><a:r>',
    )

    paragraph = rpptx.Presentation(spaced).slides[0].shapes[0].text_frame.paragraphs[0]
    assert paragraph.line_spacing == 2.5
    assert paragraph.space_before == 132.0


def test_run_font_setters_round_trip_and_none_clears_each_attribute(tmp_path):
    import rpptx
    from rpptx import MSO_UNDERLINE, RGBColor

    presentation = _textbox_presentation(rpptx)
    font = presentation.slides[0].shapes[0].text_frame.paragraphs[0].runs[0].font
    names = ("italic", "underline", "strike", "all_caps", "name", "color", "size", "bold")
    assert [getattr(font, name) for name in names] == [None] * len(names)

    font.italic = True
    font.underline = True
    font.strike = True
    font.all_caps = True
    font.name = "Georgia"
    font.color = RGBColor(0x12, 0x34, 0x56)
    font.size = rpptx.Pt(20)
    font.bold = False
    output = tmp_path / "font.pptx"
    presentation.save(output)

    assert (
        '<a:rPr sz="2000" b="0" i="1" cap="all" u="sng" strike="sngStrike">'
        '<a:solidFill><a:srgbClr val="123456"/></a:solidFill>'
        '<a:latin typeface="Georgia"/></a:rPr>'
    ) in _text_body_xml(output)
    presentation = rpptx.Presentation(output)
    font = presentation.slides[0].shapes[0].text_frame.paragraphs[0].runs[0].font
    assert [getattr(font, name) for name in names] == [
        True,
        True,
        True,
        True,
        "Georgia",
        "123456",
        rpptx.Pt(20),
        False,
    ]

    for value, expected, written in (
        (MSO_UNDERLINE.DOTTED_LINE, MSO_UNDERLINE.DOTTED_LINE, 'u="dotted"'),
        (MSO_UNDERLINE.WORDS, MSO_UNDERLINE.WORDS, 'u="words"'),
        (MSO_UNDERLINE.SINGLE_LINE, True, 'u="sng"'),
        (MSO_UNDERLINE.NONE, False, 'u="none"'),
        (False, False, 'u="none"'),
    ):
        font.underline = value
        assert font.underline is expected
        presentation.save(output)
        assert written in _text_body_xml(output)

    for name in names:
        setattr(font, name, None)
    presentation.save(output)
    assert "<a:r><a:rPr/><a:t>hello</a:t></a:r>" in _text_body_xml(output)


def test_font_color_takes_triples_or_hex_and_keeps_transforms_it_does_not_name(tmp_path):
    import rpptx

    source = tmp_path / "source.pptx"
    colored = tmp_path / "colored.pptx"
    output = tmp_path / "output.pptx"
    presentation = _textbox_presentation(rpptx)
    presentation.slides[0].shapes[0].text_frame.paragraphs[0].add_run(" theme")
    presentation.save(source)
    _replace_in_slide(
        source,
        colored,
        "<a:r><a:t>hello</a:t></a:r>",
        '<a:r><a:rPr><a:solidFill><a:srgbClr val="FF0000"><a:alpha val="50000"/>'
        "</a:srgbClr></a:solidFill></a:rPr><a:t>hello</a:t></a:r>",
    )
    _replace_in_slide(
        colored,
        colored,
        "<a:r><a:t",
        '<a:r><a:rPr><a:solidFill><a:schemeClr val="accent1"><a:lumMod val="75000"/>'
        "</a:schemeClr></a:solidFill></a:rPr><a:t",
    )

    presentation = rpptx.Presentation(colored)
    paragraph = presentation.slides[0].shapes[0].text_frame.paragraphs[0]
    alpha, theme = paragraph.runs[0].font, paragraph.runs[1].font
    assert alpha.color == "FF0000"
    assert theme.color is None
    alpha.color = "00ff00"
    theme.color = (1, 2, 3)
    assert (alpha.color, theme.color) == ("00FF00", "010203")
    assert paragraph.font.color is None
    paragraph.font.color = rpptx.RGBColor.from_string("ABCDEF")
    assert paragraph.font.color == "ABCDEF"
    presentation.save(output)

    body = _text_body_xml(output)
    assert '<a:srgbClr val="00FF00"><a:alpha val="50000"/></a:srgbClr>' in body
    assert '<a:solidFill><a:srgbClr val="010203"/></a:solidFill>' in body
    assert "schemeClr" not in body
    assert '<a:defRPr><a:solidFill><a:srgbClr val="ABCDEF"/></a:solidFill></a:defRPr>' in body

    alpha.color = None
    presentation.save(output)
    assert "00FF00" not in _text_body_xml(output)


def test_font_strike_caps_and_name_keep_the_variants_they_cannot_name(tmp_path):
    import rpptx

    source = tmp_path / "source.pptx"
    variants = tmp_path / "variants.pptx"
    output = tmp_path / "output.pptx"
    _textbox_presentation(rpptx).save(source)
    _replace_in_slide(
        source,
        variants,
        "<a:r><a:t>",
        '<a:r><a:rPr cap="small" strike="dblStrike"><a:latin typeface="Aptos" '
        'panose="020B0004020202020204" charset="0"/></a:rPr><a:t>',
    )

    presentation = rpptx.Presentation(variants)
    font = presentation.slides[0].shapes[0].text_frame.paragraphs[0].runs[0].font
    assert font.strike is True
    assert font.all_caps is None
    font.strike = font.strike
    font.all_caps = None
    font.name = "Georgia"
    presentation.save(output)
    body = _text_body_xml(output)
    assert 'strike="dblStrike"' in body
    assert 'cap="small"' in body
    assert '<a:latin typeface="Georgia" panose="020B0004020202020204" charset="0"/>' in body

    font.strike = False
    font.all_caps = True
    presentation.save(output)
    body = _text_body_xml(output)
    assert 'strike="noStrike"' in body
    assert body.count("cap=") == 1 and 'cap="all"' in body
    reopened = rpptx.Presentation(output).slides[0].shapes[0].text_frame.paragraphs[0]
    assert reopened.runs[0].font.all_caps is True


def test_paragraph_bullet_reads_each_choice_and_a_character_replaces_a_picture_bullet(tmp_path):
    import rpptx

    presentation = _textbox_presentation(rpptx)
    paragraph = presentation.slides[0].shapes[0].text_frame.paragraphs[0]
    assert paragraph.bullet is None
    paragraph.bullet = "•"
    assert paragraph.bullet == "•"
    paragraph.bullet = False
    assert paragraph.bullet is False
    source = tmp_path / "source.pptx"
    presentation.save(source)
    assert '<a:pPr><a:buNone/></a:pPr>' in _text_body_xml(source)
    paragraph.bullet = None
    assert paragraph.bullet is None

    styled = tmp_path / "styled.pptx"
    numbered = tmp_path / "numbered.pptx"
    picture = tmp_path / "picture.pptx"
    output = tmp_path / "output.pptx"
    _replace_in_slide(
        source,
        styled,
        "<a:buNone/>",
        '<a:buClr><a:srgbClr val="FF0000"/></a:buClr><a:buSzPct val="80000"/>'
        '<a:buFont typeface="Wingdings"/><a:buChar char="§"/>',
    )
    _replace_in_slide(source, numbered, "<a:buNone/>", '<a:buAutoNum type="arabicPeriod"/>')
    _replace_in_slide(
        source,
        picture,
        "<a:buNone/>",
        '<a:buFontTx/><a:buBlip><a:blip r:embed="rId9"/></a:buBlip>',
    )

    presentation = rpptx.Presentation(styled)
    paragraph = presentation.slides[0].shapes[0].text_frame.paragraphs[0]
    assert paragraph.bullet == "§"
    paragraph.bullet = "-"
    presentation.save(output)
    assert (
        '<a:buClr><a:srgbClr val="FF0000"/></a:buClr><a:buSzPct val="80000"/>'
        '<a:buFont typeface="Wingdings"/><a:buChar char="-"/>'
    ) in _text_body_xml(output)
    paragraph.bullet = None
    presentation.save(output)
    assert "<a:bu" not in _text_body_xml(output)

    assert rpptx.Presentation(numbered).slides[0].shapes[0].text_frame.paragraphs[0].bullet is True
    presentation = rpptx.Presentation(picture)
    paragraph = presentation.slides[0].shapes[0].text_frame.paragraphs[0]
    assert paragraph.bullet is True
    paragraph.bullet = "-"
    presentation.save(output)
    body = _text_body_xml(output)
    assert "buBlip" not in body
    assert '<a:buFontTx/><a:buChar char="-"/>' in body


def test_add_run_returns_a_live_run_and_stales_the_paragraph_handle_once():
    import rpptx

    presentation = _textbox_presentation(rpptx)
    paragraph = presentation.slides[0].shapes[0].text_frame.paragraphs[0]
    run = paragraph.add_run(" world")
    assert run.text == " world"
    run.font.bold = True
    assert run.font.bold is True
    _assert_stale_after_exactly_one_bump(rpptx, lambda: paragraph.text)

    paragraph = presentation.slides[0].shapes[0].text_frame.paragraphs[0]
    assert paragraph.text == "hello world"
    assert [current.font.bold for current in paragraph.runs] == [None, True]
    empty = paragraph.add_run()
    assert empty.text == ""
    assert len(presentation.slides[0].shapes[0].text_frame.paragraphs[0].runs) == 3


def test_invalid_text_property_values_raise_before_changing_the_package():
    import rpptx

    presentation = _textbox_presentation(rpptx)
    frame = presentation.slides[0].shapes[0].text_frame
    paragraph = frame.paragraphs[0]
    font = paragraph.runs[0].font
    before = presentation.to_bytes()
    for error, operation in (
        # 1800 EMU is 14 hundredths of a point, which the schema rejects.
        (ValueError, lambda: setattr(font, "size", 1800)),
        (ValueError, lambda: setattr(font, "size", rpptx.Pt(4001))),
        (ValueError, lambda: setattr(font, "color", "GG0000")),
        (ValueError, lambda: setattr(font, "color", "FF00")),
        (ValueError, lambda: setattr(font, "color", (256, 0, 0))),
        (TypeError, lambda: setattr(font, "color", 3.5)),
        (ValueError, lambda: setattr(font, "name", "")),
        (ValueError, lambda: setattr(font, "underline", 18)),
        (ValueError, lambda: setattr(paragraph, "alignment", 0)),
        (TypeError, lambda: setattr(paragraph, "alignment", "ctr")),
        (ValueError, lambda: setattr(paragraph, "line_spacing", 132.5)),
        (ValueError, lambda: setattr(paragraph, "line_spacing", -1)),
        (ValueError, lambda: setattr(paragraph, "space_before", rpptx.Pt(1585))),
        (ValueError, lambda: setattr(paragraph, "space_after", -0.5)),
        (ValueError, lambda: setattr(paragraph, "left_indent", -1)),
        (ValueError, lambda: setattr(paragraph, "first_line_indent", 51_206_401)),
        (ValueError, lambda: setattr(paragraph, "bullet", True)),
        (ValueError, lambda: setattr(paragraph, "bullet", "")),
        (ValueError, lambda: setattr(frame, "margin_left", 2**31)),
        (ValueError, lambda: setattr(frame, "vertical_anchor", 2)),
        (ValueError, lambda: setattr(frame, "auto_size", 3)),
    ):
        with pytest.raises(error):
            operation()
        assert presentation.to_bytes() == before


def test_clearing_absent_text_properties_inserts_no_empty_property_elements():
    import rpptx

    presentation = _textbox_presentation(rpptx)
    frame = presentation.slides[0].shapes[0].text_frame
    paragraph = frame.paragraphs[0]
    before = presentation.to_bytes()
    for target, names in (
        (frame, ("margin_left", "vertical_anchor", "word_wrap", "auto_size")),
        (paragraph, ("alignment", "line_spacing", "space_after", "left_indent", "bullet")),
        (paragraph.font, ("bold", "italic", "underline", "name", "color", "size")),
        (paragraph.runs[0].font, ("bold", "strike", "all_caps", "name", "color", "size")),
    ):
        for name in names:
            setattr(target, name, None)
    assert presentation.to_bytes() == before


def test_text_properties_agree_with_python_pptx_in_both_directions(tmp_path):
    if importlib.util.find_spec("pptx") is None:
        pytest.skip("python-pptx oracle is installed only for the differential gate")
    assert importlib.metadata.version("python-pptx") == "1.0.2"

    import rpptx
    from pptx import Presentation as OraclePresentation
    from pptx.dml.color import RGBColor as OracleRGBColor
    from pptx.enum.text import MSO_ANCHOR as OracleAnchor
    from pptx.enum.text import MSO_AUTO_SIZE as OracleAutoSize
    from pptx.enum.text import MSO_UNDERLINE as OracleUnderline
    from pptx.enum.text import PP_ALIGN as OracleAlign
    from pptx.util import Inches as OracleInches
    from pptx.util import Pt as OraclePt

    oracle_path = tmp_path / "python-pptx.pptx"
    oracle = OraclePresentation()
    slide = oracle.slides.add_slide(oracle.slide_layouts[6])
    frame = slide.shapes.add_textbox(
        OracleInches(1), OracleInches(1), OracleInches(4), OracleInches(2)
    ).text_frame
    frame.margin_left = OracleInches(0.3)
    frame.margin_bottom = OraclePt(2)
    frame.vertical_anchor = OracleAnchor.MIDDLE
    frame.word_wrap = True
    frame.auto_size = OracleAutoSize.TEXT_TO_FIT_SHAPE
    paragraph = frame.paragraphs[0]
    paragraph.alignment = OracleAlign.RIGHT
    paragraph.line_spacing = 2.5
    paragraph.space_before = OraclePt(12)
    paragraph.space_after = OraclePt(3)
    run = paragraph.add_run()
    run.text = "oracle"
    run.font.italic = True
    run.font.underline = OracleUnderline.DOUBLE_LINE
    run.font.name = "Georgia"
    run.font.color.rgb = OracleRGBColor(0x11, 0x22, 0x33)
    run.font.size = OraclePt(24)
    paragraph.add_run().font.underline = True
    oracle.save(oracle_path)

    frame = rpptx.Presentation(oracle_path).slides[0].shapes[0].text_frame
    assert (frame.margin_left, frame.margin_right, frame.margin_bottom) == (
        rpptx.Inches(0.3),
        None,
        rpptx.Pt(2),
    )
    assert frame.vertical_anchor is rpptx.MSO_ANCHOR.MIDDLE
    assert frame.word_wrap is True
    assert frame.auto_size is rpptx.MSO_AUTO_SIZE.TEXT_TO_FIT_SHAPE
    paragraph = frame.paragraphs[0]
    assert paragraph.alignment is rpptx.PP_ALIGN.RIGHT
    assert paragraph.line_spacing == 2.5
    assert (paragraph.space_before, paragraph.space_after) == (rpptx.Pt(12), rpptx.Pt(3))
    font = paragraph.runs[0].font
    assert (font.italic, font.underline, font.name, font.color, font.size) == (
        True,
        rpptx.MSO_UNDERLINE.DOUBLE_LINE,
        "Georgia",
        "112233",
        rpptx.Pt(24),
    )
    assert paragraph.runs[1].font.underline is True

    rpptx_path = tmp_path / "rpptx.pptx"
    presentation = _textbox_presentation(rpptx)
    frame = presentation.slides[0].shapes[0].text_frame
    frame.margin_top = rpptx.Pt(5)
    frame.vertical_anchor = rpptx.MSO_ANCHOR.BOTTOM
    frame.word_wrap = False
    frame.auto_size = rpptx.MSO_AUTO_SIZE.NONE
    paragraph = frame.paragraphs[0]
    paragraph.alignment = rpptx.PP_ALIGN.CENTER
    paragraph.line_spacing = rpptx.Pt(20)
    paragraph.space_before = rpptx.Pt(4)
    font = paragraph.runs[0].font
    font.bold = True
    font.italic = False
    font.underline = rpptx.MSO_UNDERLINE.WAVY_LINE
    font.name = "Arial"
    font.color = rpptx.RGBColor(0xAA, 0xBB, 0xCC)
    font.size = rpptx.Pt(28)
    presentation.save(rpptx_path)

    frame = OraclePresentation(rpptx_path).slides[0].shapes[0].text_frame
    assert frame.margin_top == OraclePt(5)
    assert frame.vertical_anchor == OracleAnchor.BOTTOM
    assert frame.word_wrap is False
    assert frame.auto_size == OracleAutoSize.NONE
    paragraph = frame.paragraphs[0]
    assert paragraph.alignment == OracleAlign.CENTER
    assert paragraph.line_spacing == OraclePt(20)
    assert paragraph.space_before == OraclePt(4)
    font = paragraph.runs[0].font
    assert (font.bold, font.italic, font.underline, font.name, font.size) == (
        True,
        False,
        OracleUnderline.WAVY_LINE,
        "Arial",
        OraclePt(28),
    )
    assert font.color.rgb == OracleRGBColor(0xAA, 0xBB, 0xCC)


def test_text_enums_match_python_pptx_member_values_and_xml_tokens():
    if importlib.util.find_spec("pptx") is None:
        pytest.skip("python-pptx oracle is installed only for the differential gate")
    assert importlib.metadata.version("python-pptx") == "1.0.2"

    import rpptx
    from pptx.enum import text as oracle
    from rpptx.enum import text

    presentation = _textbox_presentation(rpptx)

    def written(attribute):
        body = presentation.to_bytes()
        with zipfile.ZipFile(io.BytesIO(body)) as archive:
            slide_xml = archive.read("ppt/slides/slide1.xml").decode()
        return re.search(f' {attribute}="([^"]+)"', slide_xml).group(1)

    for ours, theirs, attribute, assign in (
        (
            text.MSO_TEXT_UNDERLINE_TYPE,
            oracle.MSO_TEXT_UNDERLINE_TYPE,
            "u",
            lambda value: setattr(
                presentation.slides[0].shapes[0].text_frame.paragraphs[0].runs[0].font,
                "underline",
                value,
            ),
        ),
        (
            text.PP_PARAGRAPH_ALIGNMENT,
            oracle.PP_PARAGRAPH_ALIGNMENT,
            "algn",
            lambda value: setattr(
                presentation.slides[0].shapes[0].text_frame.paragraphs[0], "alignment", value
            ),
        ),
        (
            text.MSO_VERTICAL_ANCHOR,
            oracle.MSO_VERTICAL_ANCHOR,
            "anchor",
            lambda value: setattr(
                presentation.slides[0].shapes[0].text_frame, "vertical_anchor", value
            ),
        ),
        (text.MSO_AUTO_SIZE, oracle.MSO_AUTO_SIZE, None, None),
    ):
        members = {member.name: member for member in theirs if member.name != "MIXED"}
        extensions = {"JUSTIFY", "DISTRIBUTE"} if ours is text.MSO_VERTICAL_ANCHOR else set()
        assert {member.name for member in ours} == set(members) | extensions
        for name, member in members.items():
            assert int(ours[name]) == int(member)
            if assign is not None:
                assign(ours[name])
                assert written(attribute) == member.xml_value
def _python_pptx_deck(path, build):
    pptx = pytest.importorskip("pptx", reason="python-pptx is the differential oracle")
    deck = pptx.Presentation()
    build(deck)
    deck.save(path)
    return path


def _package_parts(data):
    with zipfile.ZipFile(io.BytesIO(data)) as archive:
        return {name: archive.read(name) for name in archive.namelist()}


def _package_bytes(parts):
    output = io.BytesIO()
    with zipfile.ZipFile(output, "w", zipfile.ZIP_DEFLATED) as archive:
        for name, data in parts.items():
            archive.writestr(name, data)
    return output.getvalue()


def _template_jpeg():
    import rpptx

    return _package_parts(rpptx.Presentation().to_bytes())["docProps/thumbnail.jpeg"]


def test_slide_size_reads_and_writes_keep_the_other_dimension(tmp_path):
    import rpptx

    prs = rpptx.Presentation()
    assert (prs.slide_width, prs.slide_height) == (12_192_000, 6_858_000)
    assert isinstance(prs.slide_width, rpptx.Length)
    held = prs.slides
    prs.slide_width = rpptx.Inches(10)
    assert (prs.slide_width, prs.slide_height) == (rpptx.Inches(10), 6_858_000)
    assert len(held) == 0
    before = prs.to_bytes()
    with pytest.raises(rpptx.RpptxError):
        prs.slide_height = 0
    assert prs.to_bytes() == before
    path = tmp_path / "sized.pptx"
    prs.save(path)
    pptx = pytest.importorskip("pptx", reason="python-pptx is the differential oracle")
    oracle = pptx.Presentation(path)
    assert (oracle.slide_width, oracle.slide_height) == (rpptx.Inches(10), 6_858_000)

    parts = _package_parts(prs.to_bytes())
    parts["ppt/presentation.xml"] = re.sub(
        rb"<p:sldSz [^>]*/>", b"", parts["ppt/presentation.xml"]
    )
    unsized = tmp_path / "unsized.pptx"
    unsized.write_bytes(_package_bytes(parts))
    prs = rpptx.Presentation(unsized)
    assert (prs.slide_width, prs.slide_height) == (None, None)
    prs.slide_height = rpptx.Inches(5)
    assert (prs.slide_width, prs.slide_height) == (12_192_000, rpptx.Inches(5))


def test_slide_layout_hidden_and_background_round_trip_through_python_pptx(tmp_path):
    import rpptx
    from rpptx.dml.color import RGBColor
    from rpptx.enum.dml import MSO_FILL_TYPE

    source = _python_pptx_deck(
        tmp_path / "slides.pptx",
        lambda deck: [deck.slides.add_slide(deck.slide_layouts[index]) for index in (5, 1)],
    )
    prs = rpptx.Presentation(source)
    first, second = prs.slides[0], prs.slides[1]
    assert first.slide_layout == prs.slide_layouts[5]
    assert first.slide_layout != prs.slide_layouts[1]
    assert first.slide_layout.name == "Title Only"
    assert prs.slide_layouts.index(second.slide_layout) == 1
    with pytest.raises(ValueError, match="layout not in this SlideLayouts collection"):
        prs.slide_layouts.index(rpptx.Presentation().slide_layouts[1])

    assert first.hidden is False
    first.hidden = True
    assert prs.slides[0].hidden is True

    background = first.background
    assert first.follow_master_background is True
    assert background.fill.type is None
    assert first.follow_master_background is True
    background.fill.solid()
    background.fill.fore_color.rgb = RGBColor(0x12, 0x34, 0x56)
    assert background.fill.type == MSO_FILL_TYPE.SOLID
    assert first.follow_master_background is False
    second.follow_master_background = False
    assert second.background.fill.type == MSO_FILL_TYPE.BACKGROUND
    output = tmp_path / "slides-out.pptx"
    prs.save(output)

    pptx = pytest.importorskip("pptx", reason="python-pptx is the differential oracle")
    oracle = pptx.Presentation(output)
    assert oracle.slides[0]._element.get("show") == "0"
    assert oracle.slides[0].follow_master_background is False
    assert oracle.slides[0].background.fill.fore_color.rgb == pptx.dml.color.RGBColor(0x12, 0x34, 0x56)
    assert oracle.slides[1].follow_master_background is False

    prs.slides[0].follow_master_background = True
    prs.slides[0].hidden = False
    prs.save(output)
    oracle = pptx.Presentation(output)
    assert oracle.slides[0].follow_master_background is True
    assert oracle.slides[0]._element.get("show") in (None, "1")


def test_shape_geometry_name_and_rotation_setters_match_python_pptx(tmp_path):
    import rpptx

    prs = rpptx.Presentation()
    slide = prs.slides.add_slide(prs.slide_layouts[6])
    shape = slide.shapes.add_textbox(1, 2, 3, 4)
    held = shape
    shape.left, shape.top = rpptx.Inches(1), rpptx.Inches(2)
    shape.width, shape.height = rpptx.Inches(3), rpptx.Inches(4)
    shape.name = "Renamed box"
    shape.rotation = 45.5
    assert (held.left, held.top, held.width, held.height) == (
        rpptx.Inches(1),
        rpptx.Inches(2),
        rpptx.Inches(3),
        rpptx.Inches(4),
    )
    assert (held.name, held.rotation) == ("Renamed box", 45.5)
    for value, expected in ((-90, 270.0), (360, 0.0), (720.25, 0.25)):
        shape.rotation = value
        assert shape.rotation == expected
    shape.rotation = 30
    with pytest.raises(ValueError):
        shape.width = -1
    with pytest.raises(ValueError):
        shape.rotation = float("nan")

    group = prs.slides[0].shapes.add_group_shape()
    assert (group.left, group.width, group.rotation) == (None, None, 0.0)
    group.width = 100
    assert (group.left, group.width, group.height) == (None, 100, 0)
    output = tmp_path / "geometry.pptx"
    prs.save(output)

    pptx = pytest.importorskip("pptx", reason="python-pptx is the differential oracle")
    oracle = pptx.Presentation(output).slides[0].shapes[0]
    assert (oracle.left, oracle.top, oracle.width, oracle.height) == (
        rpptx.Inches(1),
        rpptx.Inches(2),
        rpptx.Inches(3),
        rpptx.Inches(4),
    )
    assert (oracle.name, oracle.rotation) == ("Renamed box", 30.0)


def test_shape_type_reports_the_python_pptx_member_for_every_shape_kind(tmp_path):
    import rpptx
    from rpptx.enum.shapes import MSO_SHAPE_TYPE

    def build(deck):
        pptx = pytest.importorskip("pptx")
        slide = deck.slides.add_slide(deck.slide_layouts[1])
        slide.shapes.add_textbox(0, 0, 10, 10)
        slide.shapes.add_shape(pptx.enum.shapes.MSO_SHAPE.ROUNDED_RECTANGLE, 0, 0, 10, 10)
        slide.shapes.add_picture(io.BytesIO(_tiny_png()), 0, 0)
        slide.shapes.add_table(1, 1, 0, 0, 10, 10)
        slide.shapes.add_connector(pptx.enum.shapes.MSO_CONNECTOR.ELBOW, 0, 0, 10, 10)
        slide.shapes.add_group_shape()
        builder = slide.shapes.build_freeform(0, 0)
        builder.add_line_segments([(10, 10), (0, 10)])
        builder.convert_to_shape()

    source = _python_pptx_deck(tmp_path / "kinds.pptx", build)
    pptx = pytest.importorskip("pptx", reason="python-pptx is the differential oracle")
    expected = [int(shape.shape_type) for shape in pptx.Presentation(source).slides[0].shapes]
    shapes = rpptx.Presentation(source).slides[0].shapes
    actual = [shape.shape_type for shape in shapes]
    assert actual == expected
    assert all(isinstance(value, MSO_SHAPE_TYPE) for value in actual)
    assert actual[0] == MSO_SHAPE_TYPE.PLACEHOLDER
    assert actual[-1] == MSO_SHAPE_TYPE.FREEFORM


def test_fill_and_line_formats_write_what_python_pptx_reads(tmp_path):
    import rpptx
    from rpptx.dml.color import RGBColor
    from rpptx.enum.dml import MSO_FILL, MSO_FILL_TYPE

    assert MSO_FILL is MSO_FILL_TYPE
    prs = rpptx.Presentation()
    slide = prs.slides.add_slide(prs.slide_layouts[6])
    shape = slide.shapes.add_shape("rect", 0, 0, 100, 100)
    fill = shape.fill
    assert fill.type is None
    with pytest.raises(TypeError, match="fill type _NoneFill has no foreground color"):
        _ = fill.fore_color
    fill.solid()
    assert fill.type == MSO_FILL_TYPE.SOLID
    assert fill.fore_color.rgb is None
    fill.fore_color.rgb = RGBColor(0xFF, 0x00, 0x00)
    fill.solid()
    assert shape.fill.fore_color.rgb == RGBColor(0xFF, 0x00, 0x00)
    assert str(shape.fill.fore_color.rgb) == "FF0000"
    with pytest.raises(ValueError, match="assigned value must be type RGBColor"):
        fill.fore_color.rgb = (1, 2, 3)

    line = shape.line
    assert (line.width, line.color.rgb, line.fill.type) == (0, None, None)
    line.color.rgb = RGBColor(0x00, 0x80, 0x00)
    assert line.fill.type == MSO_FILL_TYPE.SOLID
    line.width = rpptx.Pt(2)
    assert line.width == rpptx.Pt(2)
    with pytest.raises(ValueError):
        line.width = 20_116_801
    line.width = None
    assert line.width == 0
    line.width = rpptx.Pt(2)

    plain = prs.slides[0].shapes.add_shape("ellipse", 0, 0, 10, 10)
    _ = plain.line.color.rgb, plain.line.width, plain.fill.type
    plain.fill.background()
    assert plain.fill.type == MSO_FILL_TYPE.BACKGROUND
    with pytest.raises(ValueError, match="shape has no fill"):
        _ = prs.slides[0].shapes.add_group_shape().fill
    with pytest.raises(ValueError, match="shape has no line"):
        _ = prs.slides[0].shapes.add_table(1, 1, 0, 0, 10, 10).line

    held = prs.slides[0].shapes[0].fill
    prs.slides.add_slide(prs.slide_layouts[6])
    with pytest.raises(rpptx.StaleElementError):
        _ = held.type
    output = tmp_path / "fills.pptx"
    prs.save(output)
    parts = _package_parts(output.read_bytes())
    slide_xml = parts["ppt/slides/slide1.xml"].decode()
    assert slide_xml.count("<a:ln") == 1

    pptx = pytest.importorskip("pptx", reason="python-pptx is the differential oracle")
    oracle = pptx.Presentation(output).slides[0].shapes
    assert oracle[0].fill.type == pptx.enum.dml.MSO_FILL.SOLID
    assert oracle[0].fill.fore_color.rgb == pptx.dml.color.RGBColor(0xFF, 0x00, 0x00)
    assert oracle[0].line.color.rgb == pptx.dml.color.RGBColor(0x00, 0x80, 0x00)
    assert oracle[0].line.width == rpptx.Pt(2)
    assert oracle[1].fill.type == pptx.enum.dml.MSO_FILL.BACKGROUND


def test_colour_edits_keep_python_pptx_brightness_transforms(tmp_path):
    import rpptx
    from rpptx.dml.color import RGBColor

    def build(deck):
        pptx = pytest.importorskip("pptx")
        slide = deck.slides.add_slide(deck.slide_layouts[6])
        shape = slide.shapes.add_shape(pptx.enum.shapes.MSO_SHAPE.RECTANGLE, 0, 0, 10, 10)
        shape.fill.solid()
        shape.fill.fore_color.rgb = pptx.dml.color.RGBColor(0xFF, 0x00, 0x00)
        shape.fill.fore_color.brightness = -0.25

    source = _python_pptx_deck(tmp_path / "brightness.pptx", build)
    prs = rpptx.Presentation(source)
    prs.slides[0].shapes[0].fill.fore_color.rgb = RGBColor(0x00, 0x00, 0xFF)
    output = tmp_path / "brightness-out.pptx"
    prs.save(output)
    pptx = pytest.importorskip("pptx", reason="python-pptx is the differential oracle")
    color = pptx.Presentation(output).slides[0].shapes[0].fill.fore_color
    assert color.rgb == pptx.dml.color.RGBColor(0x00, 0x00, 0xFF)
    assert color.brightness == pytest.approx(-0.25)


def test_pictures_accept_bytes_and_file_objects_and_replace_their_image(tmp_path):
    import rpptx

    red, blue, jpeg = _tiny_png(0xFF, 0, 0), _tiny_png(0, 0, 0xFF), _template_jpeg()
    image_path = tmp_path / "red.png"
    image_path.write_bytes(red)
    stream = io.BytesIO(red)
    stream.read()
    prs = rpptx.Presentation()
    slide = prs.slides.add_slide(prs.slide_layouts[6])
    slide.shapes.add_picture(image_path, 0, 0)
    prs.slides[0].shapes.add_picture(stream, 0, 0, width=rpptx.Inches(1))
    prs.slides[0].shapes.add_picture(jpeg, 0, 0)
    prs.slides[0].shapes.add_textbox(0, 0, 10, 10)
    shapes = prs.slides[0].shapes
    assert [shape.image.blob for shape in shapes[:3]] == [red, red, jpeg]
    image = shapes[2].image
    assert (image.content_type, image.ext) == ("image/jpeg", "jpg")
    assert (shapes[0].image.content_type, shapes[0].image.ext) == ("image/png", "png")
    assert shapes[1].width == rpptx.Inches(1)
    with pytest.raises(ValueError, match="shape is not a picture"):
        _ = shapes[3].image

    held = shapes[0]
    held.replace_image(blue)
    assert held.image.blob == blue
    assert shapes[1].image.blob == red
    shapes[1].replace_image(io.BytesIO(jpeg))
    assert shapes[1].image.blob == jpeg
    before = prs.to_bytes()
    with pytest.raises(rpptx.RpptxError, match="not a supported image"):
        held.replace_image(b"not an image")
    assert prs.to_bytes() == before
    with pytest.raises(rpptx.RpptxError, match="unsupported image bytes"):
        prs.slides[0].shapes.add_picture(b"\x89PNG", 0, 0)
    output = tmp_path / "pictures.pptx"
    prs.save(output)
    media = [name for name in _package_parts(output.read_bytes()) if name.startswith("ppt/media/")]
    assert len(media) == 2

    pptx = pytest.importorskip("pptx", reason="python-pptx is the differential oracle")
    oracle = pptx.Presentation(output).slides[0].shapes
    assert [oracle[index].image.blob for index in range(3)] == [blue, jpeg, jpeg]


def test_shapes_and_slides_are_removed_and_reordered_with_stale_handles(tmp_path):
    import rpptx

    prs = rpptx.Presentation()
    for index in range(3):
        slide = prs.slides.add_slide(prs.slide_layouts[6])
        slide.shapes.add_textbox(0, 0, 10, 10).text = f"slide {index}"
    held = prs.slides[0]
    prs.slides.move(0, -1)
    assert [slide.shapes[0].text for slide in prs.slides] == ["slide 1", "slide 2", "slide 0"]
    with pytest.raises(rpptx.StaleElementError):
        _ = held.shapes
    with pytest.raises(IndexError):
        prs.slides.move(0, 3)
    prs.slides.remove(prs.slides[1])
    assert [slide.shapes[0].text for slide in prs.slides] == ["slide 1", "slide 0"]
    other = rpptx.Presentation()
    other_slide = other.slides.add_slide(other.slide_layouts[6])
    with pytest.raises(ValueError, match="slide is not in this collection"):
        prs.slides.remove(other_slide)

    shapes = prs.slides[0].shapes
    picture = shapes.add_picture(io.BytesIO(_tiny_png()), 0, 0)
    group = prs.slides[0].shapes.add_group_shape()
    assert len(prs.slides[0].shapes) == 3
    shapes = prs.slides[0].shapes
    with pytest.raises(ValueError, match="shape is not in this collection"):
        prs.slides[1].shapes.remove(shapes[0])
    with pytest.raises(rpptx.StaleElementError):
        shapes.remove(picture)
    shapes.remove(shapes[1])
    with pytest.raises(rpptx.StaleElementError):
        _ = group.name
    assert [shape.shape_type for shape in prs.slides[0].shapes] == [17, 6]
    output = tmp_path / "removed.pptx"
    prs.save(output)
    parts = _package_parts(output.read_bytes())
    assert not [name for name in parts if name.startswith("ppt/media/")]
    pptx = pytest.importorskip("pptx", reason="python-pptx is the differential oracle")
    oracle = pptx.Presentation(output)
    assert [len(slide.shapes) for slide in oracle.slides] == [2, 1]


def test_slide_duplicate_inserts_after_the_source_with_its_notes(tmp_path):
    import rpptx

    prs = rpptx.Presentation()
    for index in range(3):
        slide = prs.slides.add_slide(prs.slide_layouts[6])
        slide.shapes.add_textbox(0, 0, 10, 10).text = f"slide {index}"
    prs.slides[1].notes_text = "note 1"
    source = prs.slides[1]
    held = prs.slides[2].shapes[0]

    duplicate = prs.slides.duplicate(source)
    assert [slide.shapes[0].text for slide in prs.slides] == [
        "slide 0",
        "slide 1",
        "slide 1",
        "slide 2",
    ]
    assert [slide.notes_text for slide in prs.slides] == [None, "note 1", "note 1", None]
    assert (duplicate.shapes[0].text, duplicate.notes_text) == ("slide 1", "note 1")
    _assert_stale_after_exactly_one_bump(rpptx, lambda: held.text)
    _assert_stale_after_exactly_one_bump(rpptx, lambda: source.notes_text)
    duplicate.notes_text = "copy note"
    assert [slide.notes_text for slide in prs.slides] == [None, "note 1", "copy note", None]

    other = rpptx.Presentation()
    other_slide = other.slides.add_slide(other.slide_layouts[6])
    with pytest.raises(ValueError, match="slide is not in this collection"):
        prs.slides.duplicate(other_slide)

    prs.add_comment_author(
        id="{11111111-1111-1111-1111-111111111111}",
        name="Ada Lovelace",
        user_id="ada@example.com",
        provider_id="local",
    )
    prs.slides[0].add_comment(
        id="{22222222-2222-2222-2222-222222222222}",
        author_id="{11111111-1111-1111-1111-111111111111}",
        created="2026-09-14T10:30:00Z",
        text="Blocks duplication",
    )
    commented = prs.slides[0]
    before = prs.to_bytes()
    with pytest.raises(rpptx.RpptxError, match="modern comments"):
        prs.slides.duplicate(commented)
    assert prs.to_bytes() == before
    assert (len(prs.slides), commented.shapes[0].text) == (4, "slide 0")

    output = tmp_path / "duplicated.pptx"
    prs.save(output)
    pptx = pytest.importorskip("pptx", reason="python-pptx is the differential oracle")
    oracle = pptx.Presentation(output)
    assert [slide.shapes[0].text for slide in oracle.slides] == [
        "slide 0",
        "slide 1",
        "slide 1",
        "slide 2",
    ]
    assert [
        slide.notes_slide.notes_text_frame.text if slide.has_notes_slide else None
        for slide in oracle.slides
    ] == [None, "note 1", "copy note", None]


def test_presentation_from_bytes_opens_like_a_path_and_rejects_other_bytes(tmp_path):
    import rpptx

    prs = _textbox_presentation(rpptx)
    prs.slides[0].notes_text = "note"
    path = tmp_path / "source.pptx"
    prs.save(path)

    reopened = rpptx.Presentation.from_bytes(path.read_bytes())
    assert len(reopened.slides) == 1
    assert (reopened.slides[0].shapes[0].text, reopened.slides[0].notes_text) == (
        "hello",
        "note",
    )
    assert reopened.to_bytes() == rpptx.Presentation(path).to_bytes()
    with pytest.raises(rpptx.PackageError):
        rpptx.Presentation.from_bytes(b"not a package")


def test_try_replace_text_checks_the_expected_count_before_publishing():
    import pickle

    import rpptx

    prs = _textbox_presentation(rpptx)
    prs.slides[0].shapes[0].text = "NAME and NAME"
    prs.slides[0].shapes.add_table(1, 1, 0, 0, 100, 100).table.cell(0, 0).text = "NAME"
    prs.slides[0].notes_text = "Notes for NAME"
    held = prs.slides[0].shapes[0]
    before = prs.to_bytes()

    with pytest.raises(rpptx.ReplacementCountError) as raised:
        prs.try_replace_text("NAME", "Ada", expect=3)
    error = raised.value
    assert isinstance(error, rpptx.RpptxError)
    assert str(error) == 'expected 3 replacement(s) of "NAME", found 4'
    assert (error.expected, error.found) == (3, 4)
    # Worker pools pickle exceptions, so the counts must survive the trip.
    copied = pickle.loads(pickle.dumps(error))
    assert (str(copied), copied.expected, copied.found) == (str(error), 3, 4)
    assert prs.to_bytes() == before
    assert held.text == "NAME and NAME"

    assert prs.try_replace_text("MISSING", "x") == 0
    assert prs.try_replace_text("MISSING", "x", expect=0) == 0
    with pytest.raises(rpptx.ReplacementCountError, match="found 0"):
        prs.try_replace_text("MISSING", "x", expect=1)
    with pytest.raises(rpptx.RpptxError, match="placeholder must not be empty"):
        prs.try_replace_text("", "x")
    assert prs.to_bytes() == before
    assert held.text == "NAME and NAME"

    assert prs.try_replace_text("NAME", "Ada", expect=4) == 4
    _assert_stale_after_exactly_one_bump(rpptx, lambda: held.text)
    shapes = prs.slides[0].shapes
    assert (shapes[0].text, shapes[1].table.cell(0, 0).text) == ("Ada and Ada", "Ada")
    assert prs.slides[0].notes_text == "Notes for Ada"
    held = prs.slides[0].shapes[0]
    assert prs.try_replace_text("Ada", "Grace") == 4
    _assert_stale_after_exactly_one_bump(rpptx, lambda: held.text)
    assert prs.slides[0].shapes[0].text == "Grace and Grace"


def test_slide_resolve_and_remove_comment_match_the_cli_operations():
    import rpptx

    author_id = "{11111111-1111-1111-1111-111111111111}"
    comment_id = "{22222222-2222-2222-2222-222222222222}"
    reply_id = "{33333333-3333-3333-3333-333333333333}"
    second_id = "{55555555-5555-5555-5555-555555555555}"
    unknown_id = "{99999999-9999-9999-9999-999999999999}"
    created = "2026-09-14T10:30:00Z"
    prs = rpptx.Presentation()
    for _ in range(2):
        prs.slides.add_slide(prs.slide_layouts[6])
    prs.add_comment_author(
        id=author_id, name="Ada Lovelace", user_id="ada@example.com", provider_id="local"
    )
    prs.slides[0].add_comment(id=comment_id, author_id=author_id, created=created, text="Thread")
    prs.slides[0].reply_to_comment(
        comment_id, id=reply_id, author_id=author_id, created=created, text="Reply"
    )
    prs.slides[0].add_comment(id=second_id, author_id=author_id, created=created, text="Second")

    slide = prs.slides[0]
    before = prs.to_bytes()
    for operation, comment in (
        (slide.resolve_comment, unknown_id),
        (slide.resolve_comment, reply_id),
        (slide.remove_comment, unknown_id),
        (prs.slides[1].remove_comment, comment_id),
    ):
        with pytest.raises(rpptx.RpptxError, match="unknown comment id"):
            operation(comment)
    assert prs.to_bytes() == before
    assert [comment.status for comment in slide.comments] == [None, None]

    slide.resolve_comment(comment_id)
    _assert_stale_after_exactly_one_bump(rpptx, lambda: slide.comments)
    thread, second = prs.slides[0].comments
    assert (thread.id, thread.status, second.status) == (comment_id, "resolved", None)
    assert [(reply.id, reply.status) for reply in thread.replies] == [(reply_id, None)]

    prs.slides[0].remove_comment(reply_id)
    assert prs.slides[0].comments[0].replies == ()
    prs.slides[0].remove_comment(comment_id)
    assert [comment.id for comment in prs.slides[0].comments] == [second_id]
    reopened = rpptx.Presentation.from_bytes(prs.to_bytes())
    assert reopened.slides[0].comments == prs.slides[0].comments

    # The facade keeps the emptied comments part, so duplicate still refuses.
    prs.slides[0].remove_comment(second_id)
    assert prs.slides[0].comments == ()
    before = prs.to_bytes()
    with pytest.raises(rpptx.RpptxError, match="modern comments"):
        prs.slides.duplicate(prs.slides[0])
    assert (prs.to_bytes(), len(prs.slides)) == (before, 2)


def test_add_shape_accepts_preset_names_and_every_mso_shape_member(tmp_path):
    import rpptx
    from rpptx.enum.shapes import MSO_AUTO_SHAPE_TYPE, MSO_CONNECTOR, MSO_CONNECTOR_TYPE, MSO_SHAPE, MSO_SHAPE_TYPE

    assert MSO_AUTO_SHAPE_TYPE is MSO_SHAPE and MSO_CONNECTOR is MSO_CONNECTOR_TYPE
    assert len(MSO_SHAPE) == 181
    assert (MSO_SHAPE.PENTAGON, MSO_SHAPE.CHEVRON) == (51, 52)
    assert MSO_SHAPE(5) is MSO_SHAPE.ROUNDED_RECTANGLE
    assert MSO_SHAPE.ROUNDED_RECTANGLE.xml_value == "roundRect"
    prs = rpptx.Presentation()
    prs.slides.add_slide(prs.slide_layouts[6])
    for member in MSO_SHAPE:
        prs.slides[0].shapes.add_shape(member, 0, 0, 10, 10)
    prs.slides[0].shapes.add_shape("roundRect", 0, 0, 10, 10)
    prs.slides[0].shapes.add_shape(1, 0, 0, 10, 10)
    with pytest.raises(ValueError, match="unsupported MSO_SHAPE value"):
        prs.slides[0].shapes.add_shape(9999, 0, 0, 10, 10)
    with pytest.raises(rpptx.RpptxError):
        prs.slides[0].shapes.add_shape("notAPreset", 0, 0, 10, 10)
    shapes = prs.slides[0].shapes
    shapes.add_connector(MSO_CONNECTOR.ELBOW, 100, 200, 10, 20)
    prs.slides[0].shapes.add_connector(1, 0, 0, 50, 50)
    with pytest.raises(ValueError, match="unsupported MSO_CONNECTOR value"):
        prs.slides[0].shapes.add_connector(MSO_CONNECTOR.MIXED, 0, 0, 1, 1)
    group = prs.slides[0].shapes.add_group_shape()
    assert (group.shape_type, len(group.shapes)) == (MSO_SHAPE_TYPE.GROUP, 0)
    output = tmp_path / "presets.pptx"
    prs.save(output)

    pptx = pytest.importorskip("pptx", reason="python-pptx is the differential oracle")
    oracle_members = {member.name: member for member in pptx.enum.shapes.MSO_AUTO_SHAPE_TYPE}
    assert set(oracle_members) - {member.name for member in MSO_SHAPE} == {"UP_ARROW"}
    for member in MSO_SHAPE:
        assert (oracle_members[member.name].value, oracle_members[member.name].xml_value) == (
            member.value,
            member.xml_value,
        )
    oracle = pptx.Presentation(output).slides[0].shapes
    def preset(shape):
        return shape._element.xpath("./p:spPr/a:prstGeom/@prst")[0]

    presets = [preset(shape) for shape in list(oracle)[: len(MSO_SHAPE) + 2]]
    assert presets == [member.xml_value for member in MSO_SHAPE] + ["roundRect", "rect"]
    elbow, straight = oracle[len(MSO_SHAPE) + 2], oracle[len(MSO_SHAPE) + 3]
    assert elbow.shape_type == pptx.enum.shapes.MSO_SHAPE_TYPE.LINE
    assert (elbow.begin_x, elbow.begin_y, elbow.end_x, elbow.end_y) == (100, 200, 10, 20)
    assert preset(straight) == "line"


def test_notes_text_creates_the_notes_slide_on_a_python_pptx_deck(tmp_path):
    import rpptx

    source = _python_pptx_deck(
        tmp_path / "no-notes.pptx",
        lambda deck: [deck.slides.add_slide(deck.slide_layouts[6]) for _ in range(2)],
    )
    assert not [name for name in _package_parts(source.read_bytes()) if "notes" in name]
    prs = rpptx.Presentation(source)
    prs.slides[1].notes_text = "Second slide note"
    assert [slide.notes_text for slide in prs.slides] == [None, "Second slide note"]
    output = tmp_path / "notes.pptx"
    prs.save(output)
    assert len(prs.render_all_notes(dpi=36.0)) == 2

    pptx = pytest.importorskip("pptx", reason="python-pptx is the differential oracle")
    oracle = pptx.Presentation(output)
    assert oracle.slides[0].has_notes_slide is False
    notes_slide = oracle.slides[1].notes_slide
    assert notes_slide.notes_text_frame.text == "Second slide note"
    assert [shape.name for shape in notes_slide.placeholders] == [
        "Slide Image Placeholder 1",
        "Notes Placeholder 2",
        "Slide Number Placeholder 3",
    ]


def test_shape_xml_is_a_self_contained_element():
    import xml.etree.ElementTree as ElementTree

    import rpptx

    prs = rpptx.Presentation()
    slide = prs.slides.add_slide(prs.slide_layouts[6])
    slide.shapes.add_textbox(0, 0, 10, 10).text = "xml"
    data = prs.slides[0].shapes[0].xml
    assert isinstance(data, bytes)
    element = ElementTree.fromstring(data)
    assert element.tag == "{http://schemas.openxmlformats.org/presentationml/2006/main}sp"
    assert b"<a:t>xml</a:t>" in data


def test_adjustments_match_python_pptx_defaults_reads_and_writes(tmp_path):
    import rpptx
    from rpptx.enum.shapes import MSO_SHAPE

    def build(deck):
        pptx = pytest.importorskip("pptx")
        slide = deck.slides.add_slide(deck.slide_layouts[6])
        for member in MSO_SHAPE:
            slide.shapes.add_shape(pptx.enum.shapes.MSO_AUTO_SHAPE_TYPE[member.name], 0, 0, 10, 10)
        arrow = slide.shapes.add_shape(pptx.enum.shapes.MSO_SHAPE.RIGHT_ARROW, 0, 0, 10, 10)
        arrow.adjustments[1] = 0.25

    source = _python_pptx_deck(tmp_path / "adjustments.pptx", build)
    pptx = pytest.importorskip("pptx", reason="python-pptx is the differential oracle")
    expected = [list(shape.adjustments) for shape in pptx.Presentation(source).slides[0].shapes]
    prs = rpptx.Presentation(source)
    actual = [list(shape.adjustments) for shape in prs.slides[0].shapes]
    differences = {
        member.name: (ours, theirs)
        for member, ours, theirs in zip(MSO_SHAPE, actual, expected)
        if ours != theirs
    }
    assert differences == {
        "FOLDED_CORNER": ([0.16667], []),
        "UP_DOWN_ARROW": ([0.5, 0.5], [0.5, 0.5, 0.5, 0.5]),
    }
    assert actual[-1] == expected[-1] == [0.5, 0.25]

    shapes = prs.slides[0].shapes
    rounded = shapes[list(MSO_SHAPE).index(MSO_SHAPE.ROUNDED_RECTANGLE)]
    adjustments = rounded.adjustments
    assert (len(adjustments), adjustments[0], adjustments[-1]) == (1, 0.16667, 0.16667)
    adjustments[0] = 0.3
    assert rounded.adjustments[0] == 0.3
    with pytest.raises(ValueError, match="adjustment value must be numeric"):
        adjustments[0] = "wide"
    with pytest.raises(IndexError):
        _ = adjustments[1]
    textbox = prs.slides[0].shapes.add_textbox(0, 0, 10, 10)
    assert len(textbox.adjustments) == 0
    with pytest.raises(ValueError, match="shape has no adjustments"):
        _ = prs.slides[0].shapes.add_group_shape().adjustments
    output = tmp_path / "adjustments-out.pptx"
    prs.save(output)
    oracle = pptx.Presentation(output).slides[0].shapes
    assert oracle[list(MSO_SHAPE).index(MSO_SHAPE.ROUNDED_RECTANGLE)].adjustments[0] == 0.3
