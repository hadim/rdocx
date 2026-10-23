import io, sys, zipfile, hashlib
sys.path.insert(0, sys.argv[1])
import rdocx
GRID = '<w:tblGrid><w:gridCol w:w="1000"/><w:gridCol w:w="1000"/></w:tblGrid>'
ROW = lambda s: f'<w:tr><w:tc><w:p><w:r><w:t>{s}0</w:t></w:r></w:p></w:tc><w:tc><w:p><w:r><w:t>{s}1</w:t></w:r></w:p></w:tc></w:tr>'
TAILS = {
    "tail_sdt_row": f'<w:tbl>{GRID}{ROW("b")}<w:sdt><w:sdtPr/><w:sdtContent>{ROW("a")}</w:sdtContent></w:sdt></w:tbl>',
    "tail_customxml_row": f'<w:tbl>{GRID}{ROW("b")}<w:customXml w:element="r">{ROW("a")}</w:customXml></w:tbl>',
    "tail_sdt_cell": f'<w:tbl>{GRID}<w:tr><w:tc><w:p><w:r><w:t>b0</w:t></w:r></w:p></w:tc><w:sdt><w:sdtPr/><w:sdtContent><w:tc><w:p><w:r><w:t>a1</w:t></w:r></w:p></w:tc></w:sdtContent></w:sdt></w:tr></w:tbl>',
}
def build(tbl, after):
    doc = rdocx.Document(); doc.add_paragraph("first")
    z = zipfile.ZipFile(io.BytesIO(doc.to_bytes())); out = io.BytesIO()
    with zipfile.ZipFile(out, "w") as a:
        for i in z.infolist():
            d = z.read(i.filename)
            if i.filename == "word/document.xml":
                t = d.decode(); at = t.index("<w:sectPr"); d = (t[:at] + "<w:p><w:r><w:t>x</w:t></w:r></w:p>" + tbl + after + t[at:]).encode()
            a.writestr(i, d)
    return rdocx.Document.from_bytes(out.getvalue())
def out(fn):
    try: return repr(fn())[:70]
    except BaseException as e: return f"{type(e).__name__}: {str(e)[:160]}"
for name, tbl in TAILS.items():
    for op in ("add_table", "insert_table_end", "insert_table_before_y"):
        doc = build(tbl, "<w:p><w:r><w:t>y</w:t></w:r></w:p>")
        def go():
            if op == "add_table": t = doc.add_table(1, 2)
            elif op == "insert_table_end": t = doc.insert_table(3, 1, 2)
            else: t = doc.insert_table(2, 1, 2)
            t.cell(0, 0).text = "NEW00"; t.cell(0, 1).text = "NEW01"
            return [t.cell(0, 0).text, t.cell(0, 1).xml[-40:]]
        r = out(go)
        res = [out(lambda: [[c.text for row in t.rows for c in row.cells] for t in doc.tables])]
        res.append(out(lambda: [p.text for p in doc.paragraphs]))
        res.append(out(lambda: hashlib.sha1(doc.to_bytes()).hexdigest()))
        res.append(out(lambda: "NEW00" in zipfile.ZipFile(io.BytesIO(doc.to_bytes())).read("word/document.xml").decode().split("<w:sdt>")[1] if "<w:sdt>" in zipfile.ZipFile(io.BytesIO(doc.to_bytes())).read("word/document.xml").decode() else "nosdt"))
        print(name, op, r, "|", " | ".join(res))
