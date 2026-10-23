"""Differential handle trace for #407: prints every held handle's state after each edit."""
import io
import zipfile

import rdocx

W14 = "http://schemas.microsoft.com/office/word/2010/wordml"
BODY = "".join([
    '<w:p><w:pPr><w:pStyle w:val="Heading1"/></w:pPr><w:r><w:rPr><w:b/></w:rPr><w:t>heading</w:t></w:r></w:p>',
    '<w:p><w:r><w:t xml:space="preserve">see </w:t></w:r><w:hyperlink w:anchor="target"><w:r><w:t>target</w:t></w:r></w:hyperlink></w:p>',
    '<w:p><w:bookmarkStart w:id="1" w:name="target"/><w:fldSimple w:instr=" PAGE "><w:r><w:t>1</w:t></w:r></w:fldSimple><w:bookmarkEnd w:id="1"/></w:p>',
    '<w:tbl><w:tblGrid><w:gridCol w:w="2000"/><w:gridCol w:w="2000"/></w:tblGrid><w:tr><w:tc><w:p><w:r><w:t>cell</w:t></w:r></w:p>'
    '<w:tbl><w:tblGrid><w:gridCol w:w="1000"/></w:tblGrid><w:tr><w:tc><w:p><w:r><w:t>nested</w:t></w:r></w:p></w:tc></w:tr></w:tbl>'
    '<w:p/></w:tc><w:tc><w:p><w:r><w:t>right</w:t></w:r></w:p></w:tc></w:tr></w:tbl>',
    '<w:sdt><w:sdtPr><w:alias w:val="block"/></w:sdtPr><w:sdtContent><w:p><w:r><w:t>block one</w:t></w:r></w:p><w:p><w:r><w:t>block two</w:t></w:r></w:p></w:sdtContent></w:sdt>',
    *(f'<w:p><w:r><w:rPr><w:i/></w:rPr><w:t>{text}</w:t></w:r><w:r><w:t>tail</w:t></w:r></w:p>' for text in "abcdefgh"),
    '<w:p><w:r><w:t>last</w:t></w:r></w:p>',
])


def build(root_w14):
    document = rdocx.Document()
    document.add_paragraph("first")
    document.section(0).header.add_paragraph("header text")
    output = io.BytesIO()
    with zipfile.ZipFile(io.BytesIO(document.to_bytes())) as source, zipfile.ZipFile(output, "w") as result:
        for info in source.infolist():
            data = source.read(info.filename)
            if info.filename == "word/document.xml":
                text = data.decode()
                at = text.index("<w:sectPr")
                text = text[:at] + BODY + text[at:]
                if root_w14:
                    text = text.replace("<w:document ", f'<w:document xmlns:w14="{W14}" ', 1)
                data = text.encode()
            result.writestr(info, data)
    return rdocx.Document.from_bytes(output.getvalue())


def probe(value):
    try:
        return repr(value())
    except Exception as error:  # noqa: BLE001
        return f"{type(error).__name__}: {error}"


def run(root_w14):
    doc = build(root_w14)
    held = []

    def grab(step):
        for index in (0, 3, 6, 9, len(doc.paragraphs) - 1):
            if index < len(doc.paragraphs):
                paragraph = doc.paragraphs[index]
                held.append((f"{step} p{index}", paragraph))
                if len(paragraph.runs):
                    held.append((f"{step} p{index}.r0", paragraph.runs[0]))
        table = doc.tables[0]
        held.append((f"{step} t0", table))
        cell = table.cell(0, 0)
        held.append((f"{step} c00", cell))
        held.append((f"{step} c00.p0", cell.paragraphs[0]))
        for extra in range(1, len(doc.tables)):
            held.append((f"{step} t{extra}.c00", doc.tables[extra].cell(0, 0)))
        host = next(t.cell(0, 0) for t in doc.tables if len(t.cell(0, 0).tables))
        nested = host.tables[0]
        held.append((f"{step} nested", nested))
        held.append((f"{step} nested.p0", nested.cell(0, 0).paragraphs[0]))

    def report(step):
        print(f"== {step}: {[p.text for p in doc.paragraphs]}")
        for name, handle in held:
            line = [name, probe(lambda: handle.text)]
            line.append(probe(lambda: handle.xml))
            print(" | ".join(line))

    steps = [
        ("insert_paragraph(3)", lambda: doc.insert_paragraph(3, "ins")),
        ("bold run", lambda: setattr(doc.paragraphs[8].runs[0], "bold", True)),
        ("remove_content(8)", lambda: doc.remove_content(8)),
        ("paragraph.text", lambda: setattr(doc.paragraphs[8], "text", "changed")),
        ("add_paragraph", lambda: doc.add_paragraph("appended")),
        ("add_run", lambda: doc.paragraphs[9].add_run("more")),
        ("remove_content_range(9, 11)", lambda: doc.remove_content_range(9, 11)),
        ("cell.text", lambda: setattr(doc.tables[0].cell(0, 1), "text", "cell edit")),
        ("add_row", lambda: doc.tables[0].add_row()),
        ("insert_paragraph_before", lambda: doc.paragraphs[7].insert_paragraph_before("before")),
        ("replace_xml runs", lambda: doc.paragraphs[8].replace_xml(
            '<w:p xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:r><w:t>x</w:t></w:r><w:r><w:t>y</w:t></w:r><w:r><w:t>z</w:t></w:r></w:p>')),
        ("insert after local runs", lambda: doc.insert_paragraph(2, "after local")),
        ("replace_xml link", lambda: doc.paragraphs[9].replace_xml(
            '<w:p xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:hyperlink w:anchor="target"><w:r><w:t>linked</w:t></w:r></w:hyperlink></w:p>')),
        ("header append", lambda: doc.section(0).header.add_paragraph("h2")),
        ("insert_paragraph(0)", lambda: doc.insert_paragraph(0, "zero")),
        ("add_table", lambda: doc.add_table(1, 1)),
        ("remove_content(1)", lambda: doc.remove_content(1)),
        ("insert at end", lambda: doc.insert_paragraph(doc.paragraphs.__len__() + 1, "end")),
        ("insert_table before table", lambda: doc.insert_table(4, 2, 2)),
        ("cell text after insert_table", lambda: setattr(doc.tables[1].cell(0, 1), "text", "moved")),
        ("insert_table after table", lambda: doc.insert_table(7, 1, 3)),
        ("add_table", lambda: doc.add_table(2, 1)),
        ("nested cell edit", lambda: setattr(doc.tables[1].cell(0, 0).tables[0].cell(0, 0), "text", "deep")),
        ("insert_paragraph after tables", lambda: doc.insert_paragraph(5, "between")),
        ("remove_content(5)", lambda: doc.remove_content(5)),
        ("insert_table(0)", lambda: doc.insert_table(0, 1, 1)),
    ]
    grab("open")
    report("open")
    for step, edit in steps:
        print(f"-- {step}: {probe(lambda: type(edit()).__name__)}")
        report(step)
        grab(step)
    print(doc.to_bytes() == rdocx.Document.from_bytes(doc.to_bytes()).to_bytes())


for root_w14 in (False, True):
    print(f"#### root_w14={root_w14}")
    run(root_w14)
