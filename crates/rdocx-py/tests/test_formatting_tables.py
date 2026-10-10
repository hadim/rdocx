from io import BytesIO
from zipfile import ZIP_DEFLATED, ZipFile

import pytest

from rdocx import RGBColor


def _document_xml(document_bytes):
    with ZipFile(BytesIO(document_bytes)) as package:
        return package.read("word/document.xml").decode()


def _replace_document_xml(document_bytes, old, new):
    source_bytes = BytesIO(document_bytes)
    output_bytes = BytesIO()
    with ZipFile(source_bytes) as source, ZipFile(
        output_bytes, "w", compression=ZIP_DEFLATED
    ) as output:
        for member in source.infolist():
            contents = source.read(member.filename)
            if member.filename == "word/document.xml":
                assert old in contents
                contents = contents.replace(old, new)
            output.writestr(member, contents)
    return output_bytes.getvalue()


def test_unset_run_bold_is_none():
    from rdocx import Document

    document = Document()
    run = document.add_paragraph("").add_run("plain")

    assert run.font.bold is None


def test_run_bool_tristate_round_trips():
    from rdocx import Document

    document = Document()
    font = document.add_paragraph("").add_run("formatted").font
    font.bold = False
    font.italic = True
    font.strike = False

    reopened = Document.from_bytes(document.to_bytes())
    font = reopened.paragraphs[0].runs[0].font
    assert font.bold is False
    assert font.italic is True
    assert font.strike is False


def test_none_clears_direct_formatting():
    from rdocx import Document, Inches

    document = Document()
    font = document.add_paragraph("").add_run("formatted").font
    font.bold = True
    font.italic = False
    font.underline = True
    font.bold = None
    font.italic = None
    font.underline = None
    paragraph_format = document.paragraphs[0].paragraph_format
    paragraph_format.keep_with_next = True
    paragraph_format.keep_together = True
    paragraph_format.page_break_before = True
    paragraph_format.widow_control = True
    paragraph_format.first_line_indent = Inches(-0.25)
    paragraph_format.keep_with_next = None
    paragraph_format.keep_together = None
    paragraph_format.page_break_before = None
    paragraph_format.widow_control = None
    paragraph_format.first_line_indent = None

    reopened = Document.from_bytes(document.to_bytes())
    font = reopened.paragraphs[0].runs[0].font
    paragraph_format = reopened.paragraphs[0].paragraph_format
    assert font.bold is None
    assert font.italic is None
    assert font.underline is None
    assert paragraph_format.keep_with_next is None
    assert paragraph_format.keep_together is None
    assert paragraph_format.page_break_before is None
    assert paragraph_format.widow_control is None
    assert paragraph_format.first_line_indent is None


def test_font_and_paragraph_format_values_round_trip():
    from rdocx import (
        Document,
        Inches,
        Pt,
        RGBColor,
        WD_ALIGN_PARAGRAPH,
        WD_UNDERLINE,
    )

    document = Document()
    paragraph = document.add_paragraph("")
    run = paragraph.add_run("formatted")
    run.font.name = "Aptos"
    run.font.size = Pt(12)
    run.font.color = RGBColor(0x12, 0x34, 0x56)
    run.font.underline = WD_UNDERLINE.DOT_DASH
    paragraph = document.paragraphs[0]
    paragraph.alignment = WD_ALIGN_PARAGRAPH.RIGHT
    paragraph.paragraph_format.space_before = Pt(3)
    paragraph.paragraph_format.space_after = Pt(6)
    paragraph.paragraph_format.left_indent = Inches(0.5)
    paragraph.paragraph_format.right_indent = Inches(0.25)
    paragraph.paragraph_format.first_line_indent = Inches(-0.25)
    paragraph.paragraph_format.line_spacing = Pt(18)
    paragraph.paragraph_format.keep_with_next = False
    paragraph.paragraph_format.keep_together = True
    paragraph.paragraph_format.page_break_before = False
    paragraph.paragraph_format.widow_control = True

    reopened = Document.from_bytes(document.to_bytes())
    paragraph = reopened.paragraphs[0]
    font = paragraph.runs[0].font
    paragraph_format = paragraph.paragraph_format
    assert font.name == "Aptos"
    assert font.size == Pt(12)
    assert font.color.rgb == RGBColor(0x12, 0x34, 0x56)
    assert font.underline == WD_UNDERLINE.DOT_DASH
    assert paragraph.alignment == WD_ALIGN_PARAGRAPH.RIGHT
    assert paragraph_format.alignment == WD_ALIGN_PARAGRAPH.RIGHT
    assert paragraph_format.space_before == Pt(3)
    assert paragraph_format.space_after == Pt(6)
    assert paragraph_format.left_indent == Inches(0.5)
    assert paragraph_format.right_indent == Inches(0.25)
    assert paragraph_format.first_line_indent == Inches(-0.25)
    assert paragraph_format.line_spacing == Pt(18)
    assert paragraph_format.keep_with_next is False
    assert paragraph_format.keep_together is True
    assert paragraph_format.page_break_before is False
    assert paragraph_format.widow_control is True


def test_all_approved_underline_codes_round_trip():
    from rdocx import Document, WD_UNDERLINE

    for style in (WD_UNDERLINE.DOT_DASH, WD_UNDERLINE.DOT_DOT_DASH):
        document = Document()
        font = document.add_paragraph("").add_run("underlined").font
        font.underline = style
        reopened = Document.from_bytes(document.to_bytes())
        assert reopened.paragraphs[0].runs[0].font.underline == style


def test_format_subhandles_become_stale_after_structure_change():
    from rdocx import Document, StaleElementError

    document = Document()
    paragraph = document.add_paragraph("")
    font = paragraph.add_run("held").font
    paragraph_format = document.paragraphs[0].paragraph_format

    document.add_paragraph("invalidates both")

    with pytest.raises(StaleElementError):
        _ = font.bold
    with pytest.raises(StaleElementError):
        _ = paragraph_format.space_after


def test_nested_paragraph_stale_error_names_complete_recovery_path():
    from rdocx import Document, StaleElementError

    document = Document()
    cell = document.add_table(rows=1, cols=1).rows[0].cells[0]
    nested = cell.add_paragraph("held")

    document.add_paragraph("invalidates nested paragraph")

    with pytest.raises(
        StaleElementError,
        match=(
            r"doc\.tables\[0\]\.rows\[0\]\.cells\[0\]"
            r"\.paragraphs\[1\]"
        ),
    ):
        _ = nested.text


def test_cell_text_replacement_invalidates_nested_run_and_font():
    from rdocx import Document, StaleElementError

    document = Document()
    document.add_table(rows=1, cols=1)
    document.tables[0].rows[0].cells[0].text = "before"
    run = document.tables[0].rows[0].cells[0].paragraphs[0].runs[0]
    font = run.font

    document.tables[0].rows[0].cells[0].text = "after"

    with pytest.raises(StaleElementError, match=r"revision 2.*revision 3"):
        _ = run.text
    with pytest.raises(StaleElementError, match=r"revision 2.*revision 3"):
        _ = font.bold


def test_table_handles_write_through_and_reopen():
    from rdocx import (
        Document,
        Inches,
        Pt,
        WD_CELL_VERTICAL_ALIGNMENT,
        WD_TABLE_ALIGNMENT,
    )

    document = Document()
    table = document.add_table(rows=2, cols=2)
    table.style = "TableGrid"
    table.alignment = WD_TABLE_ALIGNMENT.CENTER
    table.width = Inches(4)
    document.tables[0].rows[0].cells[0].text = "alpha"
    document.tables[0].rows[1].cells[1].text = "omega"
    document.tables[0].rows[1].cells[1].vertical_alignment = (
        WD_CELL_VERTICAL_ALIGNMENT.BOTTOM
    )
    document.tables[0].rows[1].cells[1].width = Inches(2)
    document.tables[0].rows[0].cells[0].add_paragraph("nested")
    nested = document.tables[0].rows[0].cells[0].paragraphs[-1]
    nested.paragraph_format.space_after = Pt(6)
    nested.paragraph_format.left_indent = Inches(0.25)

    reopened = Document.from_bytes(document.to_bytes())
    table = reopened.tables[0]
    assert len(reopened.tables) == 1
    assert len(list(reopened.tables)) == 1
    assert len(table.rows) == 2
    assert [len(row.cells) for row in table.rows] == [2, 2]
    assert len(table.rows[0].cells) == 2
    assert table.style == "TableGrid"
    assert table.alignment == WD_TABLE_ALIGNMENT.CENTER
    assert table.width == Inches(4)
    assert table.rows[0].cells[0].text == "alpha\nnested"
    assert table.rows[1].cells[1].text == "omega"
    assert (
        table.rows[1].cells[1].vertical_alignment
        == WD_CELL_VERTICAL_ALIGNMENT.BOTTOM
    )
    assert table.rows[1].cells[1].width == Inches(2)
    nested = table.rows[0].cells[0].paragraphs[-1]
    assert [paragraph.text for paragraph in table.rows[0].cells[0].paragraphs] == [
        "alpha",
        "nested",
    ]
    assert nested.paragraph_format.space_after == Pt(6)
    assert nested.paragraph_format.left_indent == Inches(0.25)


def test_unrepresentable_table_justify_reads_as_none_after_reopen():
    from rdocx import Document, WD_TABLE_ALIGNMENT

    document = Document()
    table = document.add_table(rows=1, cols=1)
    table.alignment = WD_TABLE_ALIGNMENT.CENTER
    with pytest.raises(ValueError, match="unsupported table alignment"):
        table.alignment = 3

    justified = _replace_document_xml(
        document.to_bytes(),
        b'<w:jc w:val="center"/>',
        b'<w:jc w:val="both"/>',
    )
    reopened = Document.from_bytes(justified)

    assert reopened.tables[0].alignment is None


def test_automatic_font_color_reads_as_none_after_reopen():
    from rdocx import Document, RGBColor

    document = Document()
    font = document.add_paragraph("").add_run("automatic").font
    font.color = RGBColor(0x12, 0x34, 0x56)
    automatic = _replace_document_xml(
        document.to_bytes(),
        b'<w:color w:val="123456"/>',
        b'<w:color w:val="auto"/>',
    )
    reopened = Document.from_bytes(automatic)

    assert reopened.paragraphs[0].runs[0].font.color.rgb is None


def test_table_rows_are_cloned_with_their_formatting_and_removed():
    from rdocx import (
        Document,
        Inches,
        RdocxError,
        StaleElementError,
        WD_CELL_VERTICAL_ALIGNMENT,
    )

    document = Document()
    document.add_table(rows=2, cols=2)
    document.tables[0].rows[0].cells[0].text = "header"
    document.tables[0].rows[1].cells[0].text = "entry"
    document.tables[0].rows[1].cells[0].paragraphs[0].runs[0].font.bold = True
    template = document.tables[0].rows[1].cells[1]
    template.width = Inches(2)
    template.vertical_alignment = WD_CELL_VERTICAL_ALIGNMENT.BOTTOM

    table = document.tables[0]
    held = table.rows[0]
    copied = table.clone_row(-1)
    with pytest.raises(StaleElementError):
        _ = held.cells
    copied.cells[1].text = "new entry"

    reopened = Document.from_bytes(document.to_bytes())
    rows = reopened.tables[0].rows
    assert [row.cells[0].text for row in rows] == ["header", "entry", "entry"]
    assert rows[2].cells[0].paragraphs[0].runs[0].font.bold is True
    assert rows[2].cells[1].text == "new entry"
    assert rows[2].cells[1].width == Inches(2)
    assert rows[2].cells[1].vertical_alignment == WD_CELL_VERTICAL_ALIGNMENT.BOTTOM

    reopened.tables[0].clone_row(0, at=0)
    reopened.tables[0].remove_row(1)
    reopened.tables[0].remove_row(-1)
    assert [row.cells[0].text for row in reopened.tables[0].rows] == [
        "header",
        "entry",
    ]
    with pytest.raises(IndexError):
        reopened.tables[0].remove_row(2)
    with pytest.raises(IndexError):
        reopened.tables[0].clone_row(0, at=3)
    reopened.tables[0].remove_row(0)
    with pytest.raises(RdocxError, match="at least one row"):
        reopened.tables[0].remove_row(0)
    assert [row.cells[0].text for row in reopened.tables[0].rows] == ["entry"]


def test_table_borders_margins_and_grid_widths_round_trip():
    from rdocx import Document, Inches, Pt

    document = Document()
    table = document.add_table(rows=2, cols=3)
    assert table.border("top") is None
    assert table.cell_margins is None
    table.set_borders("single", size=4, color="000000")
    table.set_border("insideV", "dashed", size=8, color="FF0000")
    table.set_cell_margins(top=Pt(1), right=Pt(2), bottom=Pt(3), left=Pt(4))
    table.grid_widths = [Inches(1), Inches(2), Inches(3)]
    table.set_column_width(-1, Inches(1.5))

    xml = _document_xml(document.to_bytes())
    assert '<w:top w:val="single" w:sz="4" w:space="0" w:color="000000"/>' in xml
    assert '<w:insideV w:val="dashed" w:sz="8" w:space="0" w:color="FF0000"/>' in xml
    assert "<w:tblCellMar>" in xml
    assert '<w:top w:w="20" w:type="dxa"/>' in xml
    assert '<w:right w:w="40" w:type="dxa"/>' in xml
    assert '<w:gridCol w:w="1440"/>' in xml
    assert '<w:gridCol w:w="2160"/>' in xml

    table = Document.from_bytes(document.to_bytes()).tables[0]
    assert table.border("top") == ("single", 4, RGBColor.from_string("000000"))
    assert table.border("insideH") == ("single", 4, RGBColor.from_string("000000"))
    assert table.border("insideV") == ("dashed", 8, RGBColor.from_string("FF0000"))
    assert table.cell_margins == (Pt(1), Pt(2), Pt(3), Pt(4))
    assert table.grid_widths == (Inches(1), Inches(2), Inches(1.5))
    assert table.width == Inches(4.5)
    assert [cell.width for cell in table.rows[1].cells] == [
        Inches(1),
        Inches(2),
        Inches(1.5),
    ]


def test_row_height_split_and_header_round_trip():
    from rdocx import Document, Pt, WD_ROW_HEIGHT_RULE

    document = Document()
    table = document.add_table(rows=2, cols=1)
    row = table.rows[0]
    assert row.height is None
    assert row.height_rule is None
    assert row.cant_split is None
    assert row.is_header is None
    row.height = Pt(20)
    assert row.height_rule == WD_ROW_HEIGHT_RULE.AT_LEAST
    row.height_rule = WD_ROW_HEIGHT_RULE.EXACTLY
    row.height = Pt(30)
    row.cant_split = True
    row.is_header = True
    table.rows[1].cant_split = False

    xml = _document_xml(document.to_bytes())
    assert '<w:trHeight w:val="600" w:hRule="exact"/>' in xml
    assert "<w:cantSplit/>" in xml
    assert "<w:tblHeader/>" in xml

    rows = Document.from_bytes(document.to_bytes()).tables[0].rows
    assert rows[0].height == Pt(30)
    assert rows[0].height_rule == WD_ROW_HEIGHT_RULE.EXACTLY
    assert rows[0].cant_split is True
    assert rows[0].is_header is True
    assert rows[1].cant_split is False
    assert rows[1].is_header is None

    rows[0].is_header = None
    rows[0].cant_split = None
    assert rows[0].is_header is None
    assert rows[0].cant_split is None


def test_cell_shading_borders_and_margins_round_trip():
    from rdocx import Document, Pt

    document = Document()
    cell = document.add_table(rows=1, cols=2).cell(0, 1)
    assert cell.shading is None
    assert cell.border("bottom") is None
    assert cell.margins is None
    cell.shading = "D9D9D9"
    cell.set_border("bottom", "double", size=6, color="auto")
    cell.set_margins(top=0, right=Pt(5), bottom=0, left=Pt(5))

    xml = _document_xml(document.to_bytes())
    assert '<w:shd w:val="clear" w:color="auto" w:fill="D9D9D9"/>' in xml
    assert '<w:bottom w:val="double" w:sz="6" w:space="0" w:color="auto"/>' in xml

    cell = Document.from_bytes(document.to_bytes()).tables[0].cell(0, 1)
    assert cell.shading == RGBColor.from_string("D9D9D9")
    assert cell.border("bottom") == ("double", 6, "auto")
    assert cell.border("top") is None
    assert cell.margins == (0, Pt(5), 0, Pt(5))


def test_invalid_table_formatting_changes_nothing_and_keeps_handles_live():
    from rdocx import Document, RdocxError, WD_ROW_HEIGHT_RULE

    document = Document()
    table = document.add_table(rows=1, cols=2)
    row = table.rows[0]
    cell = row.cells[0]
    before = document.to_bytes()

    with pytest.raises(ValueError, match="border style"):
        table.set_borders("bogus", size=4, color="000000")
    with pytest.raises(ValueError, match="border edge"):
        cell.set_border("middle", "single", size=4, color="000000")
    with pytest.raises(RdocxError, match="border width"):
        table.set_border("top", "single", size=97, color="000000")
    with pytest.raises(ValueError, match=r"Cell\.set_border\(color\).*six ASCII hexadecimal digits"):
        cell.set_border("top", "single", size=4, color="red")
    with pytest.raises(RdocxError, match="cannot be negative"):
        table.set_cell_margins(top=-635, right=0, bottom=0, left=0)
    with pytest.raises(RdocxError, match="cannot be negative"):
        cell.set_margins(top=0, right=0, bottom=0, left=-635)
    with pytest.raises(ValueError, match=r"Cell\.shading.*six ASCII hexadecimal digits"):
        cell.shading = "yellow"
    with pytest.raises(RdocxError, match="exactly 2 positive column widths"):
        table.grid_widths = [914400]
    with pytest.raises(RdocxError, match="must be positive"):
        table.grid_widths = [914400, 0]
    with pytest.raises(IndexError):
        table.set_column_width(2, 914400)
    with pytest.raises(ValueError, match="nonnegative"):
        table.set_column_width(0, -635)
    with pytest.raises(RdocxError, match="cannot be negative"):
        row.height = -635
    with pytest.raises(ValueError, match="set height first"):
        row.height_rule = WD_ROW_HEIGHT_RULE.EXACTLY
    with pytest.raises(ValueError, match="unsupported row height rule"):
        row.height_rule = 0

    assert document.to_bytes() == before
    table.set_borders("single", size=4, color="auto")
    row.cant_split = True
    cell.shading = "FFFFFF"
    assert (row.cant_split, cell.shading) == (True, RGBColor.from_string("FFFFFF"))


def test_insert_table_at_a_body_index_returns_the_new_table():
    from rdocx import Document, StaleElementError

    seed = Document()
    seed.add_table(rows=1, cols=1).cell(0, 0).text = "in control"
    seed.add_paragraph("after")
    document = Document.from_bytes(
        _replace_document_xml(
            _replace_document_xml(seed.to_bytes(), b"<w:tbl>", b"<w:sdt><w:sdtContent><w:tbl>"),
            b"</w:tbl>",
            b"</w:tbl></w:sdtContent></w:sdt>",
        )
    )
    held = document.tables[0]
    before = document.to_bytes()
    with pytest.raises(IndexError, match="content index out of range"):
        document.insert_table(3, 1, 1)
    assert document.to_bytes() == before

    inserted = document.insert_table(1, 2, 2)
    with pytest.raises(StaleElementError):
        _ = held.rows
    assert document.find_content_index(inserted) == 1
    assert len(inserted.rows) == 2
    inserted.cell(1, 1).text = "inserted"

    reopened = Document.from_bytes(document.to_bytes())
    body = [
        (item.kind, item.text)
        for item in reopened.story_items
        if item.direct_body_index is not None
    ]
    assert body == [("content_control", None), ("table", None), ("paragraph", "after")]
    assert [table.cell(0, 0).text for table in reopened.tables] == ["in control", ""]
    assert reopened.tables[1].cell(1, 1).text == "inserted"


def test_cells_merge_through_the_checked_table_operations():
    from rdocx import Document, RdocxError, StaleElementError

    document = Document()
    table = document.add_table(rows=3, cols=3)
    held = table.cell(0, 2)
    table.set_cell_grid_span(0, 0, 2)
    with pytest.raises(StaleElementError):
        _ = held.text

    table = document.tables[0]
    assert [len(row.cells) for row in table.rows] == [2, 3, 3]
    assert [cell.grid_span for cell in table.rows[0].cells] == [2, 1]
    last = table.cell(0, -1)
    table.set_cell_vertical_merge(0, -1, "restart")
    table.set_cell_vertical_merge(1, -1, "continue")
    assert last.vertical_merge == "restart"
    assert [row.cells[-1].vertical_merge for row in table.rows] == [
        "restart",
        "continue",
        None,
    ]

    xml = _document_xml(document.to_bytes())
    assert '<w:gridSpan w:val="2"/>' in xml
    assert '<w:vMerge w:val="restart"/>' in xml
    assert "<w:vMerge/>" in xml

    table.cell(1, 1).text = "full"
    table = document.tables[0]
    before = document.to_bytes()
    with pytest.raises(RdocxError, match="nonempty cell"):
        table.set_cell_grid_span(1, 0, 2)
    with pytest.raises(RdocxError, match="no matching cell above"):
        table.set_cell_vertical_merge(1, 0, "continue")
    with pytest.raises(RdocxError, match="exceeds the row grid"):
        table.set_cell_grid_span(0, 0, 4)
    with pytest.raises(ValueError, match="vertical merge"):
        table.set_cell_vertical_merge(1, 0, "sideways")
    with pytest.raises(IndexError):
        table.set_cell_grid_span(3, 0, 2)
    assert document.to_bytes() == before

    table.set_cell_vertical_merge(1, -1, None)
    table.set_cell_grid_span(0, 0, None)
    reopened = Document.from_bytes(document.to_bytes()).tables[0]
    assert [len(row.cells) for row in reopened.rows] == [3, 3, 3]
    assert reopened.cell(1, -1).vertical_merge is None


def test_table_acceptance_workflow_writes_the_native_body():
    # table_acceptance_workflow_writes_the_body_the_python_binding_pins in
    # crates/rdocx/tests/integration_test.rs pins the same body for the native
    # calls, so CI checks that these calls write what the native ones write.
    from rdocx import Document, WD_ROW_HEIGHT_RULE

    document = Document()
    document.add_paragraph("before")
    document.add_paragraph("after")
    table = document.insert_table(1, 3, 3)
    table.set_cell_grid_span(0, 0, 2)
    table = document.tables[0]
    table.set_cell_vertical_merge(1, 2, "restart")
    table.set_cell_vertical_merge(2, 2, "continue")
    table.set_borders("single", size=4, color="000000")
    table.set_border("insideV", "dashed", size=8, color="FF0000")
    table.set_cell_margins(top=0, right=63500, bottom=12700, left=127000)
    table.grid_widths = [1371600, 1828800, 1828800]
    table.set_column_width(-1, 914400)
    cell = table.cell(1, 0)
    cell.shading = "D9D9D9"
    cell.set_margins(top=12700, right=0, bottom=25400, left=6350)
    cell.set_border("bottom", "double", size=6, color="auto")
    row = table.rows[0]
    row.height = 254000
    row.cant_split = True
    row.is_header = True
    table.rows[1].height = 381000
    table.rows[1].height_rule = WD_ROW_HEIGHT_RULE.EXACTLY

    xml = _document_xml(document.to_bytes())
    body = xml[xml.index("<w:body>") : xml.index("</w:body>") + len("</w:body>")]
    assert "".join(line.strip() for line in body.splitlines()) == (
        '<w:body><w:p><w:r><w:t>before</w:t></w:r></w:p><w:tbl><w:tblPr>'
        '<w:tblW w:w="6480" w:type="dxa"/><w:tblBorders>'
        '<w:top w:val="single" w:sz="4" w:space="0" w:color="000000"/>'
        '<w:left w:val="single" w:sz="4" w:space="0" w:color="000000"/>'
        '<w:bottom w:val="single" w:sz="4" w:space="0" w:color="000000"/>'
        '<w:right w:val="single" w:sz="4" w:space="0" w:color="000000"/>'
        '<w:insideH w:val="single" w:sz="4" w:space="0" w:color="000000"/>'
        '<w:insideV w:val="dashed" w:sz="8" w:space="0" w:color="FF0000"/>'
        '</w:tblBorders><w:tblCellMar><w:top w:w="0" w:type="dxa"/>'
        '<w:left w:w="200" w:type="dxa"/><w:bottom w:w="20" w:type="dxa"/>'
        '<w:right w:w="100" w:type="dxa"/></w:tblCellMar></w:tblPr>'
        '<w:tblGrid><w:gridCol w:w="2160"/><w:gridCol w:w="2880"/>'
        '<w:gridCol w:w="1440"/></w:tblGrid><w:tr><w:trPr><w:cantSplit/>'
        '<w:trHeight w:val="400" w:hRule="atLeast"/><w:tblHeader/></w:trPr>'
        '<w:tc><w:tcPr><w:tcW w:w="5040" w:type="dxa"/>'
        '<w:gridSpan w:val="2"/></w:tcPr><w:p/></w:tc><w:tc><w:tcPr>'
        '<w:tcW w:w="1440" w:type="dxa"/></w:tcPr><w:p/></w:tc></w:tr>'
        '<w:tr><w:trPr><w:trHeight w:val="600" w:hRule="exact"/></w:trPr>'
        '<w:tc><w:tcPr><w:tcW w:w="2160" w:type="dxa"/><w:tcBorders>'
        '<w:bottom w:val="double" w:sz="6" w:space="0" w:color="auto"/>'
        '</w:tcBorders>'
        '<w:shd w:val="clear" w:color="auto" w:fill="D9D9D9"/><w:tcMar>'
        '<w:top w:w="20" w:type="dxa"/><w:left w:w="10" w:type="dxa"/>'
        '<w:bottom w:w="40" w:type="dxa"/><w:right w:w="0" w:type="dxa"/>'
        '</w:tcMar></w:tcPr><w:p/></w:tc><w:tc><w:tcPr>'
        '<w:tcW w:w="2880" w:type="dxa"/></w:tcPr><w:p/></w:tc><w:tc>'
        '<w:tcPr><w:tcW w:w="1440" w:type="dxa"/>'
        '<w:vMerge w:val="restart"/></w:tcPr><w:p/></w:tc></w:tr><w:tr>'
        '<w:tc><w:tcPr><w:tcW w:w="2160" w:type="dxa"/></w:tcPr><w:p/>'
        '</w:tc><w:tc><w:tcPr><w:tcW w:w="2880" w:type="dxa"/></w:tcPr>'
        '<w:p/></w:tc><w:tc><w:tcPr><w:tcW w:w="1440" w:type="dxa"/>'
        '<w:vMerge/></w:tcPr><w:p/></w:tc></w:tr></w:tbl><w:p><w:r>'
        '<w:t>after</w:t></w:r></w:p><w:sectPr>'
        '<w:pgSz w:w="12240" w:h="15840"/>'
        '<w:pgMar w:top="1440" w:right="1440" w:bottom="1440" w:left="1440"'
        ' w:gutter="0" w:header="720" w:footer="720"/></w:sectPr></w:body>'
    )


def test_python_paragraph_and_run_formatting_matches_native_facades():
    from rdocx import Document

    document = Document()
    paragraph = document.add_paragraph("Title")
    assert paragraph.style is None
    assert paragraph.numbering is None
    paragraph.style = "Heading1"
    paragraph.numbering = (1, 2)
    assert paragraph.text == "Title"

    reopened = Document.from_bytes(document.to_bytes())
    assert reopened.paragraphs[0].style == "Heading1"
    assert reopened.paragraphs[0].numbering == (1, 2)

    with pytest.raises(ValueError, match="numbering level"):
        paragraph.numbering = (1, 9)
    paragraph.style = None
    paragraph.numbering = None
    reopened = Document.from_bytes(document.to_bytes())
    assert reopened.paragraphs[0].style is None
    assert reopened.paragraphs[0].numbering is None

    document = Document()
    run = document.add_paragraph("").add_run("marked")
    assert run.style_id is None
    assert run.font.highlight is None
    assert run.font.shading is None
    run.style_id = "Strong"
    run.font.highlight = "yellow"
    run.font.shading = "FFFF00"

    reopened = Document.from_bytes(document.to_bytes()).paragraphs[0].runs[0]
    assert reopened.style_id == "Strong"
    assert reopened.font.highlight == "yellow"
    assert reopened.font.shading == RGBColor.from_string("FFFF00")

    run.font.shading = "AUTO"
    reopened = Document.from_bytes(document.to_bytes()).paragraphs[0].runs[0]
    assert reopened.font.shading == "auto"

    with pytest.raises(ValueError, match="highlight"):
        run.font.highlight = "FFFF00"
    with pytest.raises(ValueError, match="hexadecimal"):
        run.font.shading = "yellow"
    run.style_id = None
    run.font.highlight = None
    run.font.shading = None
    reopened = Document.from_bytes(document.to_bytes()).paragraphs[0].runs[0]
    assert reopened.style_id is None
    assert reopened.font.highlight is None
    assert reopened.font.shading is None


def test_paragraph_style_accepts_a_defined_id_or_name_and_rejects_the_rest():
    from rdocx import Document

    source = BytesIO(Document().to_bytes())
    output = BytesIO()
    with ZipFile(source) as archive, ZipFile(
        output, "w", compression=ZIP_DEFLATED
    ) as rewritten:
        for member in archive.infolist():
            contents = archive.read(member.filename)
            if member.filename == "word/styles.xml":
                assert b"</w:styles>" in contents
                contents = contents.replace(
                    b"</w:styles>",
                    b'<w:style w:type="character" w:styleId="Strong">'
                    b'<w:name w:val="Strong"/></w:style>'
                    b'<w:style w:type="character" w:styleId="Emphasis">'
                    b'<w:name w:val="Emphasis Char"/></w:style>'
                    b'<w:style w:type="paragraph" w:styleId="EmphPara">'
                    b'<w:name w:val="Emphasis"/></w:style></w:styles>',
                )
            rewritten.writestr(member, contents)
    document = Document.from_bytes(output.getvalue())
    document.add_paragraph("Title")
    paragraph = document.paragraphs[0]

    for value, expected in (
        ("Heading 1", "Heading1"),
        ("Normal", "Normal"),
        ("heading 1", "Heading1"),
        ("Emphasis", "EmphPara"),
        ("Heading1", "Heading1"),
    ):
        paragraph.style = value
        assert paragraph.style == expected

    before = document.to_bytes()
    for value, error, message in (
        ("NoSuchStyle", KeyError, "no style with ID or name 'NoSuchStyle'"),
        ("Heading 10", KeyError, "no style"),
        ("Strong", ValueError, "character style"),
        ("emphasis char", ValueError, "character style"),
    ):
        with pytest.raises(error, match=message):
            paragraph.style = value
    assert document.to_bytes() == before
    assert paragraph.style == "Heading1"
    assert Document.from_bytes(before).paragraphs[0].style == "Heading1"


def _part(document, name):
    with ZipFile(BytesIO(document.to_bytes())) as archive:
        return archive.read(name).decode()


def test_new_document_defines_the_common_word_styles_and_opened_ones_keep_theirs():
    from rdocx import Document

    document = Document()
    styles = {style.style_id: style for style in document.styles}
    expected = {f"Heading{level}" for level in range(1, 10)} | {
        "Normal",
        "Title",
        "Subtitle",
        "NoSpacing",
        "Quote",
        "ListParagraph",
        "Caption",
        "TableGrid",
    }
    assert set(styles) == expected
    assert styles["Heading2"].name == "heading 2"
    assert styles["Heading2"].based_on == "Normal"
    assert styles["TableGrid"].style_type == "table"
    assert [style.is_default for style in document.styles].count(True) == 1

    document.add_paragraph("Chapter")
    document.paragraphs[0].style = "Heading 2"
    assert document.paragraphs[0].style == "Heading2"
    document.paragraphs[0].style = "Caption"
    assert document.paragraphs[0].style == "Caption"

    trimmed = BytesIO()
    with ZipFile(BytesIO(document.to_bytes())) as source, ZipFile(
        trimmed, "w", compression=ZIP_DEFLATED
    ) as output:
        for member in source.infolist():
            contents = source.read(member.filename)
            if member.filename == "word/styles.xml":
                contents = (
                    b'<w:styles xmlns:w="http://schemas.openxmlformats.org/'
                    b'wordprocessingml/2006/main"><w:style w:type="paragraph" '
                    b'w:default="1" w:styleId="Normal"><w:name w:val="Normal"/>'
                    b"</w:style></w:styles>"
                )
            output.writestr(member, contents)
    opened = Document.from_bytes(trimmed.getvalue())
    assert [style.style_id for style in opened.styles] == ["Normal"]


def test_add_style_derives_the_word_id_and_writes_its_formatting(tmp_path):
    from rdocx import Document, Inches, Pt, RGBColor, Style

    document = Document()
    document.add_paragraph("body")
    held = document.paragraphs[0]

    note = document.add_style("Note", style_type="paragraph", based_on="Normal")
    assert note == Style(
        style_id="Note",
        name="Note",
        based_on="Normal",
        style_type="paragraph",
        linked_style=None,
        next_style=None,
        priority=None,
        auto_redefine=None,
        hidden=None,
        semi_hidden=None,
        unhide_when_used=None,
        quick_format=None,
        locked=None,
        is_default=False,
    )
    boxed = document.add_style(
        "Boxed Note",
        based_on="note",
        next_style="Normal",
        font_name="Arial",
        font_size=Pt(12),
        bold=True,
        italic=False,
        color=RGBColor(0x11, 0x22, 0x33),
        space_before=Pt(6),
        space_after=Pt(3),
        left_indent=Inches(0.5),
        right_indent=Inches(0.25),
        first_line_indent=-Inches(0.25),
    )
    assert (boxed.style_id, boxed.based_on, boxed.next_style) == (
        "BoxedNote",
        "Note",
        "Normal",
    )
    assert document.add_style("heading x", style_id="Custom").style_id == "Custom"
    assert document.add_style("Callout", "character", bold=True).style_type == "character"
    assert document.add_style("Plain Grid", "table", based_on="Table Grid").based_on == (
        "TableGrid"
    )
    assert held.text == "body"
    held.style = "Boxed Note"
    assert held.style == "BoxedNote"

    path = tmp_path / "styles.docx"
    document.save(path)
    reopened = {style.style_id: style for style in Document(path).styles}
    assert reopened["BoxedNote"] == boxed
    xml = _part(document, "word/styles.xml")
    start = xml.index('w:styleId="BoxedNote"')
    boxed_xml = xml[start : xml.index("</w:style>", start)]
    for fragment in (
        '<w:spacing w:before="120" w:after="60"/>',
        '<w:ind w:left="720" w:right="360" w:hanging="360"/>',
        '<w:rFonts w:ascii="Arial" w:hAnsi="Arial" w:eastAsia="Arial" w:cs="Arial"/>',
        '<w:i w:val="false"/>',
        '<w:color w:val="112233"/>',
        '<w:sz w:val="24"/>',
    ):
        assert fragment in boxed_xml, fragment

    docx = pytest.importorskip("docx")
    oracle = docx.Document(str(path)).styles["Boxed Note"]
    assert oracle.style_id == "BoxedNote"
    assert oracle.base_style.name == "Note"
    assert (oracle.font.name, oracle.font.size, oracle.font.bold) == ("Arial", Pt(12), True)
    assert oracle.font.italic is False
    assert str(oracle.font.color.rgb) == "112233"
    assert oracle.paragraph_format.space_before == Pt(6)
    assert oracle.paragraph_format.left_indent == Inches(0.5)
    assert oracle.paragraph_format.first_line_indent == -Inches(0.25)


def test_add_style_derives_word_ids_that_survive_save_and_reopen(tmp_path):
    from rdocx import Document

    expected = {
        "Q&A": "QA",
        "Note (draft)": "Notedraft",
        "My-Style.v2": "My-Stylev2",
        "\u00dcberschrift Eigen": "berschriftEigen",
        'A "q" <x>': "Aqx",
        "\u898b\u51fa\u3057 \u30ab\u30b9\u30bf\u30e0": "a",
        "\u5225\u306e\u30b9\u30bf\u30a4\u30eb": "a0",
    }
    document = Document()
    for name, style_id in expected.items():
        assert document.add_style(name, based_on="Normal").style_id == style_id
        document.add_paragraph(name).style = style_id

    path = tmp_path / "ids.docx"
    document.save(path)
    reopened = Document(path)
    assert set(expected.values()) <= {style.style_id for style in reopened.styles}
    assert [paragraph.style for paragraph in reopened.paragraphs] == list(
        expected.values()
    )
    docx = pytest.importorskip("docx")
    oracle = {style.style_id for style in docx.Document(str(path)).styles}
    assert set(expected.values()) <= oracle


def test_add_style_rejects_bad_input_without_changing_the_document():
    from rdocx import Document

    document = Document()
    document.add_style("Callout", "character")
    before = document.to_bytes()
    for arguments, keywords, error, message in (
        (("Heading 2",), {}, ValueError, "ID 'Heading2' already exists"),
        (("normal",), {"style_id": "Other"}, ValueError, "named 'normal' already exists"),
        (("Note",), {"style_id": " "}, ValueError, "cannot be blank"),
        (("Note", "numbering"), {}, ValueError, "style_type must be"),
        (("Note",), {"based_on": "Missing"}, KeyError, "no style with ID or name 'Missing'"),
        (("Note",), {"based_on": "Callout"}, ValueError, "is a character style, not a paragraph"),
        (("Note",), {"next_style": "Table Grid"}, ValueError, "is a table style, not a paragraph"),
        (("Note", "character"), {"next_style": "Normal"}, ValueError, "only a paragraph style"),
        (("Note", "character"), {"space_after": 12700}, ValueError, "no paragraph formatting"),
    ):
        with pytest.raises(error, match=message):
            document.add_style(*arguments, **keywords)
    assert document.to_bytes() == before


def test_a_style_with_a_numbering_level_renders_its_paragraphs_numbered():
    from rdocx import Document, ListLevel, Pt

    def recipe(style_numbering, direct_numbering):
        document = Document()
        definition = document.add_numbering_definition(
            [ListLevel(format="decimal", text="%1."), ListLevel(format="lowerLetter")]
        )
        instance = document.add_numbering_instance(definition)
        document.add_style(
            "Step",
            based_on="Normal",
            font_name="Arial",
            font_size=Pt(14),
            bold=True,
            space_before=Pt(12),
            space_after=Pt(6),
        )
        if style_numbering:
            document.link_style_to_numbering("Step", instance, 0)
        for text in ("Mix", "Bake"):
            paragraph = document.add_paragraph(text)
            paragraph.style = "Step"
            if direct_numbering:
                paragraph.numbering = (instance, 0)
        return document

    document = recipe(style_numbering=True, direct_numbering=False)
    assert document.paragraphs[0].numbering is None
    styles_xml = _part(document, "word/styles.xml")
    step = styles_xml[styles_xml.index('w:styleId="Step"') :]
    assert '<w:numId w:val="1"/>' in step[: step.index("</w:style>")]
    numbering_xml = _part(document, "word/numbering.xml")
    assert '<w:pStyle w:val="Step"/>' in numbering_xml
    assert '<w:numFmt w:val="lowerLetter"/>' in numbering_xml
    assert '<w:lvlText w:val="%2."/>' in numbering_xml

    rendered = document.render_page_to_png(0, 72)
    assert rendered == recipe(False, True).render_page_to_png(0, 72)
    assert rendered != recipe(False, False).render_page_to_png(0, 72)
    reopened = Document.from_bytes(document.to_bytes())
    assert reopened.render_page_to_png(0, 72) == rendered


def test_list_levels_and_numbering_calls_check_their_input():
    import rdocx
    from rdocx import Document, Inches, ListLevel, RdocxError

    level = ListLevel(format="bullet", text="-", start=3, left_indent=Inches(1))
    assert (level.format, level.text, level.start) == ("bullet", "-", 3)
    assert (level.left_indent, level.hanging_indent) == (Inches(1), None)
    assert level == ListLevel(format="bullet", text="-", start=3, left_indent=Inches(1))
    assert ListLevel().format == "decimal"
    with pytest.raises(ValueError, match="'Decimal' is not a Word numbering format"):
        ListLevel(format="Decimal")
    with pytest.raises(ValueError, match="hanging_indent"):
        ListLevel(hanging_indent=-1)
    with pytest.raises(AttributeError):
        level.start = 1  # type: ignore[misc]

    document = Document()
    definition = document.add_numbering_definition(
        [level, ListLevel(format="upperRoman", hanging_indent=Inches(0.5))]
    )
    numbering_xml = _part(document, "word/numbering.xml")
    for fragment in (
        '<w:start w:val="3"/>',
        '<w:lvlText w:val="-"/>',
        '<w:ind w:left="1440" w:hanging="360"/>',
        '<w:ind w:left="1440" w:hanging="720"/>',
    ):
        assert fragment in numbering_xml, fragment
    instance = document.add_numbering_instance(definition)
    assert instance != document.add_numbering_instance(definition)

    before = document.to_bytes()
    for call in (
        lambda: document.add_numbering_definition([]),
        lambda: document.add_numbering_definition([ListLevel(text="%2.")]),
        lambda: document.add_numbering_instance(definition + 100),
        lambda: document.link_style_to_numbering("Normal", instance + 100, 0),
        lambda: document.link_style_to_numbering("Normal", instance, 5),
    ):
        with pytest.raises(RdocxError):
            call()
    with pytest.raises(ValueError, match="is a table style, not a paragraph"):
        document.link_style_to_numbering("TableGrid", instance, 0)
    assert document.to_bytes() == before
    assert rdocx.ListLevel is ListLevel


def test_styles_can_be_removed_and_made_the_default():
    from rdocx import Document, RdocxError

    document = Document()
    document.add_style("Note", based_on="Normal")
    document.add_style("Aside", based_on="Note")
    document.add_paragraph("aside").style = "Aside"

    assert document.remove_style("No Such Style") is False
    before = document.to_bytes()
    with pytest.raises(RdocxError, match="referenced by style 'Aside'"):
        document.remove_style("Note")
    with pytest.raises(RdocxError, match="referenced by document content"):
        document.remove_style("Aside")
    assert document.to_bytes() == before

    document.set_default_style("note")
    defaults = [style.style_id for style in document.styles if style.is_default]
    assert defaults == ["Note"]
    with pytest.raises(KeyError, match="no style with ID or name 'Missing'"):
        document.set_default_style("Missing")

    assert document.remove_style("Subtitle") is True
    assert "Subtitle" not in {style.style_id for style in document.styles}


def test_word_highlight_keywords_round_trip_and_clear():
    from rdocx import Document, RGBColor

    document = Document()
    document.add_paragraph("").add_run("marked").font.color = RGBColor(0x12, 0x34, 0x56)
    keyword = _replace_document_xml(
        document.to_bytes(),
        b'<w:color w:val="123456"/>',
        b'<w:color w:val="123456"/><w:highlight w:val="darkBlue"/>',
    )
    reopened = Document.from_bytes(keyword)
    font = reopened.paragraphs[0].runs[0].font
    assert font.highlight == "darkBlue"

    font.highlight = "yellow"
    reopened = Document.from_bytes(reopened.to_bytes())
    assert reopened.paragraphs[0].runs[0].font.highlight == "yellow"
    reopened.paragraphs[0].runs[0].font.highlight = None
    assert Document.from_bytes(reopened.to_bytes()).paragraphs[0].runs[0].font.highlight is None


def test_table_indent_accepts_negative_zero_positive_and_none():
    from rdocx import Document, Inches

    document = Document()
    table = document.add_table(rows=1, cols=1)
    assert table.indent is None
    for value in (Inches(-0.25), 0, Inches(0.25), None):
        table.indent = value
        assert table.indent == value


def test_invalid_table_indent_leaves_source_and_handles_unchanged():
    from rdocx import Document, Inches
    document = Document()
    table = document.add_table(rows=1, cols=1)
    table.indent = Inches(-0.25)
    cell = table.cell(0, 0)
    before = document.to_bytes()
    from rdocx import RdocxError
    for value in ("0.25", 0.25, (2**31) * 635, (-(2**31) - 1) * 635):
        with pytest.raises((TypeError, RdocxError)):
            table.indent = value
        assert document.to_bytes() == before
        assert cell.text == ""
    reopened = Document.from_bytes(document.to_bytes())
    assert reopened.tables[0].indent == Inches(-0.25)
    table.indent = -636
    assert table.indent == -635
    assert Document.from_bytes(document.to_bytes()).tables[0].indent == -635
    table.indent = None
    assert b"tblInd" not in _document_xml(document.to_bytes()).encode()


def _f_x197_word_colour_surface(document, surface):
    if surface == "style-create":
        return lambda value: document.add_style("Colour Control", color=value), None
    if surface == "style-update":
        document.add_style("Colour Control")
        return lambda value: document.set_style("Colour Control", color=value), None
    if surface.startswith("font"):
        font = document.add_paragraph("").add_run("colour").font
        if surface == "font-direct":
            return lambda value: setattr(font, "color", value), lambda: font.color.rgb
        if surface == "font-rgb":
            return lambda value: setattr(font.color, "rgb", value), lambda: font.color.rgb
        return lambda value: setattr(font, "shading", value), lambda: font.shading
    table = document.add_table(1, 1)
    if surface == "table-borders":
        return lambda value: table.set_borders("single", size=4, color=value), lambda: (table.border("top") or (None, None, None))[2]
    if surface == "table-border":
        return lambda value: table.set_border("top", "single", size=4, color=value), lambda: (table.border("top") or (None, None, None))[2]
    cell = table.cell(0, 0)
    if surface == "cell-border":
        return lambda value: cell.set_border("left", "single", size=4, color=value), lambda: (cell.border("left") or (None, None, None))[2]
    assert surface == "cell-shading"
    return lambda value: setattr(cell, "shading", value), lambda: cell.shading


_F_X197_WORD_COLOUR_SURFACES = ["style-create", "style-update", "font-direct", "font-rgb", "font-shading", "table-borders", "table-border", "cell-border", "cell-shading"]


@pytest.mark.parametrize("surface", _F_X197_WORD_COLOUR_SURFACES)
@pytest.mark.parametrize("form", ["rgb", "triple", "hex", "hash"])
def test_every_word_colour_setter_accepts_rgb_and_optional_hash_hex(surface, form):
    from rdocx import Document, RGBColor
    from xml.etree import ElementTree as ET
    document = Document()
    expected = RGBColor(0xA1, 0xB2, 0xC3)
    assert RGBColor.from_string("#a1b2c3") == expected
    assert RGBColor.from_string("a1b2c3") == expected
    value = {"rgb": expected, "triple": tuple(expected), "hex": "a1b2c3", "hash": "#a1b2c3"}[form]
    setter, getter = _f_x197_word_colour_surface(document, surface)
    setter(value)
    if getter is not None:
        assert getter() == expected
        assert isinstance(getter(), RGBColor)
    saved = document.to_bytes()
    if surface != "style-create":
        setter(value)
        assert document.to_bytes() == saved
    reopened = Document.from_bytes(saved)
    with ZipFile(BytesIO(reopened.to_bytes())) as package:
        member = "word/styles.xml" if surface.startswith("style") else "word/document.xml"
        tree = ET.fromstring(package.read(member))
        namespace = "{http://schemas.openxmlformats.org/wordprocessingml/2006/main}"
        if surface.startswith("style"):
            tree = next(node for node in tree.iter(namespace + "style") if node.get(namespace + "styleId") == "ColourControl")
        assert any(node.get(namespace + name) == "A1B2C3" for node in tree.iter() for name in ("val", "fill", "color"))


def test_style_colour_inputs_are_validated_before_style_mutation():
    from rdocx import Document
    document = Document()
    document.add_style("Colour Control", color="#123456")
    before = document.to_bytes()
    with pytest.raises(ValueError, match="hex"):
        document.set_style("Colour Control", color="##123456")
    assert document.to_bytes() == before


def test_word_font_colour_format_supports_rgb_theme_and_direct_assignment():
    from rdocx import Document, RGBColor
    document = Document()
    font = document.add_paragraph("").add_run("colour").font
    colour = font.color
    assert colour.rgb is None
    assert colour.type is None
    font.color = "#123456"
    assert colour.rgb == RGBColor(0x12, 0x34, 0x56)
    colour.theme_color = "accent1"
    assert colour.theme_color == "accent1"
    font.color = None
    assert colour.rgb is None


def test_word_scalar_rgb_getters_return_rgbcolor_and_keep_auto():
    from rdocx import Document, RGBColor
    document = Document()
    cell = document.add_table(1, 1).cell(0, 0)
    cell.shading = "123456"
    assert cell.shading == RGBColor(0x12, 0x34, 0x56)
    cell.shading = "auto"
    assert cell.shading == "auto"


@pytest.mark.parametrize("surface", _F_X197_WORD_COLOUR_SURFACES)
@pytest.mark.parametrize("value,error_type", [(object(), TypeError), (7, TypeError), ((1, 2), TypeError), ((1.5, 2, 3), TypeError), ((10**100, 2, 3), ValueError), ((256, 2, 3), ValueError), ("##123456", ValueError), ("１２３４５６", ValueError), ("12345\ud800", ValueError), (" 123456", ValueError)])
def test_invalid_colour_inputs_name_the_parameter_and_leave_source_unchanged(surface, value, error_type):
    from rdocx import Document
    document = Document()
    setter, getter = _f_x197_word_colour_surface(document, surface)
    before = document.to_bytes()
    with pytest.raises(error_type) as error:
        setter(value)
    parameter = {"style-create": "Document.add_style(color)", "style-update": "Document.set_style(color)", "font-direct": "Font.color", "font-rgb": "Font.color.rgb", "font-shading": "Font.shading", "table-borders": "Table.set_borders(color)", "table-border": "Table.set_border(color)", "cell-border": "Cell.set_border(color)", "cell-shading": "Cell.shading"}[surface]
    assert parameter in str(error.value)
    assert "RGBColor" in str(error.value) or "hex" in str(error.value) or "channels" in str(error.value)
    if error_type is TypeError:
        assert type(value).__name__ in str(error.value)
    assert document.to_bytes() == before
    if getter is not None:
        assert getter() is None


def test_colour_format_handles_follow_structural_invalidation():
    from rdocx import Document, StaleElementError
    document = Document()
    colour = document.add_paragraph("").add_run("colour").font.color
    assert colour.rgb is None
    document.add_paragraph("invalidate")
    with pytest.raises(StaleElementError):
        _ = colour.rgb


def _f_x197_word_pdf_pixels(pdf, tmp_path, name):
    import shutil
    import subprocess
    from PIL import Image
    executable = shutil.which("pdftoppm")
    assert executable is not None
    version = subprocess.run([executable, "-v"], capture_output=True, text=True, timeout=60)
    assert version.returncode == 0 and "26.01.0" in version.stdout + version.stderr
    source = tmp_path / (name + ".pdf")
    source.write_bytes(pdf)
    assert pdf.startswith(b"%PDF")
    prefix = tmp_path / name
    result = subprocess.run([executable, "-f", "1", "-singlefile", "-r", "72", "-png", str(source), str(prefix)], capture_output=True, text=True, timeout=60)
    assert result.returncode == 0, result.stdout + result.stderr
    return Image.open(prefix.with_suffix(".png")).convert("RGB")


def test_colour_inputs_and_nested_font_formats_reach_actual_rendering(tmp_path):
    from rdocx import Document, Pt
    from xml.etree import ElementTree as ET
    namespace = "{http://schemas.openxmlformats.org/drawingml/2006/main}"
    def render(value, theme=None):
        document = Document()
        font = document.add_paragraph("").add_run("COLOUR CONTROL").font
        font.name = "Liberation Sans"
        font.size = Pt(28)
        font.color.rgb = value
        if theme is not None:
            font.color.theme_color = theme
        output = BytesIO()
        with ZipFile(BytesIO(document.to_bytes())) as source, ZipFile(output, "w") as target:
            for member in source.infolist():
                data = source.read(member.filename)
                if member.filename == "word/theme/theme1.xml":
                    tree = ET.fromstring(data)
                    for token, rgb in (("accent1", "FF0000"), ("accent2", "0000FF")):
                        colour = tree.find(".//" + namespace + token + "/" + namespace + "srgbClr")
                        assert colour is not None
                        colour.set("val", rgb)
                    data = ET.tostring(tree)
                target.writestr(member, data)
        reopened = Document.from_bytes(output.getvalue())
        if theme is not None:
            colour = reopened.paragraphs[0].runs[0].font.color
            assert colour.theme_color == theme
            assert colour.rgb == RGBColor(0, 255, 0)
        return _f_x197_word_pdf_pixels(reopened.to_pdfa_deterministic(), tmp_path, (theme or value).replace("#", "hash"))
    red = render("#FF0000")
    blue = render("#0000FF")
    assert red.tobytes() == render("ff0000").tobytes()
    assert red.tobytes() != blue.tobytes()
    assert any(r > g + 40 and r > b + 40 for r, g, b in red.getdata())
    assert any(b > g + 40 and b > r + 40 for r, g, b in blue.getdata())
    assert render("00FF00", "accent1").tobytes() == red.tobytes()
    assert render("00FF00", "accent2").tobytes() == blue.tobytes()


def test_word_theme_colour_view_retains_cached_rgb_modifiers_and_opaque_source():
    from rdocx import Document, RGBColor, MSO_THEME_COLOR
    from xml.etree import ElementTree as ET
    document = Document()
    document.add_paragraph("").add_run("source").font.color = "123456"
    output = BytesIO()
    with ZipFile(BytesIO(document.to_bytes())) as source, ZipFile(output, "w") as target:
        for member in source.infolist():
            data = source.read(member.filename)
            if member.filename == "word/document.xml":
                assert data.count(b'w:val="123456"') == 1
                data = data.replace(b'w:val="123456"', b'w:val="123456" w:themeColor="accent1" w:themeTint="80" w:themeShade="40"')
                data = data.replace(b"</w:rPr>", b'<w:unknownColourMetadata value="retain"/></w:rPr>', 1)
            target.writestr(member, data)
    document = Document.from_bytes(output.getvalue())
    colour = document.paragraphs[0].runs[0].font.color
    before = document.to_bytes()
    assert colour.type == "theme" and colour.theme_color == "accent1"
    assert colour.rgb == RGBColor(0x12, 0x34, 0x56)
    assert document.to_bytes() == before
    colour.theme_color = MSO_THEME_COLOR.ACCENT_2
    reopened = Document.from_bytes(document.to_bytes())
    assert reopened.paragraphs[0].runs[0].font.color.theme_color == "accent2"
    with ZipFile(BytesIO(reopened.to_bytes())) as package:
        xml = package.read("word/document.xml")
        assert b'<w:unknownColourMetadata value="retain"/>' in xml
        root = ET.fromstring(xml)
        ns = "{http://schemas.openxmlformats.org/wordprocessingml/2006/main}"
        node = root.find(".//" + ns + "color")
        assert {key: node.get(ns + key) for key in ("val", "themeColor", "themeTint", "themeShade")} == {"val": "123456", "themeColor": "accent2", "themeTint": "80", "themeShade": "40"}
    before = document.to_bytes()
    for invalid in ("unknown", 0, -2, 17, True):
        with pytest.raises((TypeError, ValueError)):
            colour.theme_color = invalid
        assert document.to_bytes() == before
    colour.theme_color = None
    assert colour.type == "rgb" and colour.rgb == RGBColor(0x12, 0x34, 0x56)


@pytest.mark.parametrize("surface", ["font-shading", "table-borders", "table-border", "cell-border", "cell-shading"])
def test_word_shading_and_border_scalar_auto_remains_distinct_from_rgb(surface):
    from rdocx import Document
    document = Document()
    setter, getter = _f_x197_word_colour_surface(document, surface)
    setter("auto")
    assert getter() == "auto"
    assert '="auto"' in _document_xml(Document.from_bytes(document.to_bytes()).to_bytes())


@pytest.mark.parametrize("code,token", list(enumerate(["dark1", "light1", "dark2", "light2", "accent1", "accent2", "accent3", "accent4", "accent5", "accent6", "hyperlink", "followedHyperlink", "text1", "background1", "text2", "background2"], start=1)))
def test_word_supported_theme_enum_values_round_trip_without_invented_rgb(code, token):
    from rdocx import Document, MSO_THEME_COLOR
    document = Document()
    colour = document.add_paragraph("").add_run("theme").font.color
    colour.theme_color = MSO_THEME_COLOR(code)
    assert colour.rgb is None and colour.type == "theme"
    assert colour.theme_color == token
    assert Document.from_bytes(document.to_bytes()).paragraphs[0].runs[0].font.color.theme_color == token


def test_word_unset_colour_view_and_none_clear_preserve_absent_source():
    from rdocx import Document, ColorFormat
    document = Document()
    font = document.add_paragraph("").add_run("unset").font
    before = document.to_bytes()
    assert isinstance(font.color, ColorFormat)
    assert (font.color.rgb, font.color.type, font.color.theme_color) == (None, None, None)
    font.color.rgb = None
    font.color.theme_color = None
    font.color = None
    assert document.to_bytes() == before


def test_word_cell_font_colour_view_uses_accepted_run_paths_and_rejects_structural_staleness():
    from rdocx import Document, RGBColor, StaleElementError
    document = Document()
    cell = document.add_table(1, 1).cell(0, 0)
    cell.text = "cell colour"
    cell = document.tables[0].cell(0, 0)
    colour = cell.paragraphs[0].runs[0].font.color
    colour.rgb = "#123456"
    assert colour.rgb == RGBColor(0x12, 0x34, 0x56)
    assert Document.from_bytes(document.to_bytes()).tables[0].cell(0, 0).paragraphs[0].runs[0].font.color.rgb == colour.rgb
    document.add_paragraph("invalidate")
    before = document.to_bytes()
    with pytest.raises(StaleElementError):
        _ = colour.rgb
    with pytest.raises(StaleElementError):
        colour.rgb = "ABCDEF"
    assert document.to_bytes() == before
