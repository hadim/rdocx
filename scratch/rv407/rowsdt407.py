import io, sys, zipfile
sys.path.insert(0, sys.argv[1])
import rdocx
def build(tbl):
    doc = rdocx.Document(); doc.add_paragraph("first")
    z = zipfile.ZipFile(io.BytesIO(doc.to_bytes())); out = io.BytesIO()
    with zipfile.ZipFile(out, "w") as a:
        for i in z.infolist():
            d = z.read(i.filename)
            if i.filename == "word/document.xml":
                t = d.decode(); at = t.index("<w:sectPr"); d = (t[:at] + "<w:p><w:r><w:t>x</w:t></w:r></w:p>" + tbl + "<w:p><w:r><w:t>y</w:t></w:r></w:p>" + t[at:]).encode()
            a.writestr(i, d)
    return rdocx.Document.from_bytes(out.getvalue())
GRID = '<w:tblGrid><w:gridCol w:w="1000"/><w:gridCol w:w="1000"/></w:tblGrid>'
ROW = lambda s: f'<w:tr><w:tc><w:p><w:r><w:t>{s}0</w:t></w:r></w:p></w:tc><w:tc><w:p><w:r><w:t>{s}1</w:t></w:r></w:p></w:tc></w:tr>'
TABLES = {
    "row_sdt": f'<w:tbl>{GRID}<w:sdt><w:sdtPr/><w:sdtContent>{ROW("a")}</w:sdtContent></w:sdt>{ROW("b")}</w:tbl>',
    "row_customxml": f'<w:tbl>{GRID}<w:customXml w:element="r">{ROW("a")}</w:customXml>{ROW("b")}</w:tbl>',
    "cell_sdt": f'<w:tbl>{GRID}<w:tr><w:sdt><w:sdtPr/><w:sdtContent><w:tc><w:p><w:r><w:t>c0</w:t></w:r></w:p></w:tc></w:sdtContent></w:sdt><w:tc><w:p/></w:tc></w:tr>{ROW("b")}</w:tbl>',
    "tracked_row": f'<w:tbl>{GRID}<w:tr><w:trPr><w:ins w:id="1" w:author="a" w:date="2026-01-01T00:00:00Z"/></w:trPr><w:tc><w:p/></w:tc><w:tc><w:p/></w:tc></w:tr>{ROW("b")}</w:tbl>',
}
def out(fn):
    try: return repr(fn())[:60]
    except BaseException as e: return f"{type(e).__name__}: {str(e)[:110]}"
for name, tbl in TABLES.items():
    for edit in ("none", "insert", "remove", "insert_table", "add_paragraph"):
        doc = build(tbl)
        r = out(lambda: {"none": None, "insert": lambda: doc.insert_paragraph(0, "i").text, "remove": lambda: doc.remove_content(0),
                         "insert_table": lambda: len(doc.insert_table(0, 1, 1).rows), "add_paragraph": lambda: doc.add_paragraph("a").text}[edit]() if edit != "none" else None)
        t = -1 if edit == "insert_table" else 0
        res = [out(lambda rc=rc: doc.tables[t].cell(*rc).text) for rc in [(0, 0), (0, 1), (1, 0), (1, 1)]]
        res += [out(lambda: doc.tables[t].xml[-30:]), out(lambda: doc.tables[t].cell(0, 0).xml[-30:]), out(lambda: len(doc.tables[t].rows))]
        res.append(out(lambda: doc.tables[t].cell(0, 0).paragraphs[0].text))
        print(name, edit, r, "|", " | ".join(res))
