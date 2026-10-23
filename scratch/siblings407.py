"""Cost per call of the #407 siblings at 100 and 1000 body paragraphs (repro407's input plus one 2x2 table at slot 30)."""
import importlib.metadata
import io
import sys
import time
import zipfile

import rdocx

RPR = (
    '<w:rPr><w:rFonts w:ascii="Arial" w:cs="Arial" w:eastAsia="Arial" w:hAnsi="Arial"/><w:b w:val="0"/>'
    '<w:i w:val="0"/><w:smallCaps w:val="0"/><w:strike w:val="0"/><w:color w:val="000000"/><w:sz w:val="20"/>'
    '<w:szCs w:val="20"/><w:u w:val="none"/><w:shd w:fill="auto" w:val="clear"/><w:vertAlign w:val="baseline"/>'
    '<w:rtl w:val="0"/></w:rPr>'
)
TABLE = ('<w:tbl><w:tblGrid><w:gridCol w:w="2000"/><w:gridCol w:w="2000"/></w:tblGrid>'
         + ('<w:tr>' + '<w:tc><w:p><w:r><w:t>cell</w:t></w:r></w:p></w:tc>' * 2 + '</w:tr>') * 2 + '</w:tbl>')


def build(paragraphs):
    doc = rdocx.Document()
    doc.add_paragraph("first")
    z = zipfile.ZipFile(io.BytesIO(doc.to_bytes()))
    parts = {i.filename: z.read(i.filename) for i in z.infolist()}
    xml = parts["word/document.xml"].decode()
    blocks = [
        f"<w:p>{''.join(f'<w:r>{RPR}<w:t xml:space=\"preserve\">run {k} of paragraph {i} </w:t></w:r>' for k in range(6))}</w:p>"
        for i in range(paragraphs)
    ]
    blocks.insert(30, TABLE)
    at = xml.index("<w:sectPr")
    parts["word/document.xml"] = (xml[:at] + "".join(blocks) + xml[at:]).encode()
    out = io.BytesIO()
    with zipfile.ZipFile(out, "w") as archive:
        for name, data in parts.items():
            archive.writestr(name, data)
    return rdocx.Document.from_bytes(out.getvalue())


def per_call(op, n=3, setup=None):
    total = 0.0
    for _ in range(n):
        if setup:
            setup()
        start = time.perf_counter()
        op()
        total += time.perf_counter() - start
    return total / n * 1000


def costs(doc):
    p = doc.paragraphs[20]
    run = p.runs[3]
    table = doc.tables[0]
    cell = table.cell(0, 0)
    variants = iter([p.xml.replace("run 0 of", f"run 0{k} of") for k in range(10)])
    result = {
        "Paragraph.xml": per_call(lambda: p.xml),
        "Run.xml": per_call(lambda: run.xml),
        "Table.xml": per_call(lambda: table.xml),
        "Cell.xml": per_call(lambda: cell.xml),
        "Paragraph.replace_xml": per_call(lambda: p.replace_xml(next(variants))),
        "add_paragraph": per_call(lambda: doc.add_paragraph("x")),
        "insert_paragraph_before": per_call(lambda: doc.paragraphs[10].insert_paragraph_before("x")),
        "remove_content_range": per_call(
            lambda: doc.remove_content_range(10, 12),
            setup=lambda: (doc.insert_paragraph(10, "a"), doc.insert_paragraph(10, "b"))),
        "insert_table": per_call(lambda: doc.insert_table(10, 2, 2)),
    }
    return result


print("rdocx", importlib.metadata.version("rdocx"), rdocx.__file__)
small, large = costs(build(100)), costs(build(1000))
for name in small:
    ratio = large[name] / max(small[name], 1.0)
    print(f"{name:26s} {small[name]:8.2f} ms with 100 paragraphs {large[name]:8.2f} ms with 1000  ratio {ratio:5.1f}")
