import io, sys, zipfile
sys.path.insert(0, sys.argv[1])
import rdocx
sys.argv = sys.argv[:1]
sys.path.insert(0, "/private/tmp/claude-501/-Users-hadim-Code-libs-rdocx/08057ee4-afc5-4751-aca1-2668752380de/scratchpad/rv407")
import diff407
P, TBL = diff407.P, diff407.TBL
TAILS = {
    "sdt2": '<w:sdt><w:sdtPr/><w:sdtContent>' + P("i0") + P("i1") + P("i2") + '</w:sdtContent></w:sdt>',
    "sdt_tbl": '<w:sdt><w:sdtPr/><w:sdtContent>' + P("i0") + TBL("st") + P("i1") + '</w:sdtContent></w:sdt>',
    "nested_tbl": TBL("nt", nested=True),
    "inline_sdt": '<w:p><w:r><w:t>a</w:t></w:r><w:sdt><w:sdtPr/><w:sdtContent><w:r><w:t>in</w:t></w:r></w:sdtContent></w:sdt></w:p>',
    "hyperlink": '<w:p><w:hyperlink w:anchor="x"><w:r><w:t>l</w:t></w:r></w:hyperlink></w:p>',
    "raw": '<w:customXml w:element="z">' + P("cx") + '</w:customXml>',
    "bookmark_open": '<w:p><w:bookmarkStart w:id="3" w:name="b"/><w:r><w:t>b</w:t></w:r></w:p>' + '<w:bookmarkEnd w:id="3"/>',
    "sect_para": P("sp", "", "<w:pPr>" + diff407.SECT + "</w:pPr>"),
}
def build(tail):
    doc = rdocx.Document(); doc.add_paragraph("first")
    z = zipfile.ZipFile(io.BytesIO(doc.to_bytes())); out = io.BytesIO()
    with zipfile.ZipFile(out, "w") as a:
        for i in z.infolist():
            d = z.read(i.filename)
            if i.filename == "word/document.xml":
                t = d.decode(); at = t.index("<w:sectPr"); d = (t[:at] + P("x") + TBL("t") + P("y") + tail + t[at:]).encode()
            a.writestr(i, d)
    return rdocx.Document.from_bytes(out.getvalue())
def out(fn):
    try: return repr(fn())
    except BaseException as e: return f"{type(e).__name__}: {str(e)[:150]}"
for name, tail in TAILS.items():
    for op in ("add_paragraph", "add_table", "insert_end", "add_heading", "insert_table_end"):
        doc = build(tail)
        held = doc.paragraphs[-1]; held_t = doc.tables[0]
        n = len(doc.paragraphs) + len(doc.tables) + 4
        def go():
            if op == "add_paragraph": return doc.add_paragraph("new").text
            if op == "add_table": return len(doc.add_table(1, 2).rows)
            if op == "add_heading": return doc.add_heading("h", 1).text
            # find content_count: the largest index insert_paragraph accepts
            for k in range(n, -1, -1):
                try:
                    if op == "insert_end": r = doc.insert_paragraph(k, "end").text
                    else: r = len(doc.insert_table(k, 2, 2).rows)
                    return (k, r)
                except IndexError:
                    continue
        r = out(go)
        res = [out(lambda: held.text), out(lambda: held_t.xml[:30]), out(lambda: [p.text for p in doc.paragraphs]),
               out(lambda: [p.xml[-40:] for p in doc.paragraphs]), out(lambda: [t.xml[-20:] for t in doc.tables]),
               out(lambda: [[c.xml[-20:] for row in t.rows for c in row.cells] for t in doc.tables]),
               out(lambda: __import__("hashlib").sha1(doc.to_bytes()).hexdigest())]
        print(name, op, r, "|", " | ".join(res))
