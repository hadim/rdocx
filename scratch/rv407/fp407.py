import io, sys, zipfile
sys.path.insert(0, sys.argv[1])
import rdocx
sys.argv = sys.argv[:1]
sys.path.insert(0, "/private/tmp/claude-501/-Users-hadim-Code-libs-rdocx/08057ee4-afc5-4751-aca1-2668752380de/scratchpad/rv407")
import diff407
P, TBL = diff407.P, diff407.TBL
MERGED = ('<w:tbl><w:tblGrid><w:gridCol w:w="1000"/><w:gridCol w:w="1000"/></w:tblGrid><w:tr><w:tc><w:tcPr><w:gridSpan w:val="2"/></w:tcPr><w:p/></w:tc></w:tr>'
          '<w:tr><w:tc><w:tcPr><w:vMerge w:val="restart"/></w:tcPr><w:p/></w:tc><w:tc><w:p/></w:tc></w:tr><w:tr><w:tc><w:tcPr><w:vMerge/></w:tcPr><w:p/></w:tc><w:tc><w:p/></w:tc></w:tr></w:tbl>')
def build():
    doc = rdocx.Document(); doc.add_paragraph("first")
    z = zipfile.ZipFile(io.BytesIO(doc.to_bytes())); out = io.BytesIO()
    with zipfile.ZipFile(out, "w") as a:
        for i in z.infolist():
            d = z.read(i.filename)
            if i.filename == "word/document.xml":
                t = d.decode(); at = t.index("<w:sectPr"); d = (t[:at] + P("x") + TBL("t", nested=True) + P("y") + MERGED + P("z") + t[at:]).encode()
            a.writestr(i, d)
    return rdocx.Document.from_bytes(out.getvalue())
def out(fn):
    try: return repr(fn())[:80]
    except BaseException as e: return f"{type(e).__name__}: {str(e)[:120]}"
EDITS = {"none": lambda d: None, "insert_paragraph": lambda d: d.insert_paragraph(1, "i"), "remove_content": lambda d: d.remove_content(0),
         "insert_table": lambda d: d.insert_table(1, 1, 1), "add_paragraph": lambda d: d.add_paragraph("a"), "settext": lambda d: setattr(d.paragraphs[0], "text", "T")}
for name, edit in EDITS.items():
    doc = build(); edit(doc)
    res = []
    for t in range(len(doc.tables)):
        for (r, c) in [(0, 0), (0, 1), (1, 0), (1, 1), (2, 0), (2, 1), (5, 5)]:
            res.append(f"t{t}({r},{c})=" + out(lambda: doc.tables[t].cell(r, c).text))
        res.append(f"t{t} nested=" + out(lambda: [n.cell(0, 0).text for n in doc.tables[t].cell(0, 0).tables]))
        res.append(f"t{t} add_row=" + out(lambda: len(doc.tables[t].add_row().cells)))
        res.append(f"t{t} merge=" + out(lambda: doc.tables[t].cell(0, 0).merge(doc.tables[t].cell(1, 0)).text))
    res.append("story_items=" + out(lambda: len(doc.story_items)))
    res.append("find=" + out(lambda: doc.find_content_index(doc.paragraphs[-1])))
    res.append("replace_text=" + out(lambda: doc.paragraphs[-1].replace_text("z", "Z")))
    res.append("bold=" + out(lambda: setattr(doc.paragraphs[-1].runs[0].font, "bold", True)))
    res.append("style=" + out(lambda: setattr(doc.paragraphs[-1], "style", "Heading1")))
    res.append("hyperlink=" + out(lambda: doc.paragraphs[-1].add_hyperlink("https://e.x", "link").text))
    res.append("pop=" + out(lambda: doc.insert_content(0, doc.pop_content(len(doc.paragraphs)))))
    res.append("bytes=" + out(lambda: __import__("hashlib").sha1(doc.to_bytes()).hexdigest()))
    print(name, "\n  " + "\n  ".join(res))
