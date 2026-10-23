import io, zipfile
import rdocx
W = "http://schemas.openxmlformats.org/wordprocessingml/2006/main"
ROWS = (f'<w:sdt xmlns:w="{W}" xmlns:w15="http://schemas.microsoft.com/office/word/2012/wordml"><w:sdtPr><w:tag w:val="rows"/>'
        '<w15:repeatingSection/></w:sdtPr><w:sdtContent><w:sdt><w:sdtPr><w15:repeatingSectionItem/></w:sdtPr><w:sdtContent>'
        '<w:p><w:r><w:t>row</w:t></w:r></w:p></w:sdtContent></w:sdt></w:sdtContent></w:sdt>')
BLOCK = ('<w:sdt xmlns:x="urn:x"><w:sdtPr><x:meta/></w:sdtPr><w:sdtContent><w:p><w:r><w:t>blk</w:t></w:r></w:p>'
         '</w:sdtContent></w:sdt>')
def build(tail):
    doc = rdocx.Document(); doc.add_paragraph("keep")
    out = io.BytesIO()
    with zipfile.ZipFile(io.BytesIO(doc.to_bytes())) as src, zipfile.ZipFile(out, "w") as dst:
        for info in src.infolist():
            data = src.read(info.filename)
            if info.filename == "word/document.xml":
                t = data.decode(); at = t.index("<w:sectPr"); data = (t[:at] + tail + t[at:]).encode()
            dst.writestr(info, data)
    return rdocx.Document.from_bytes(out.getvalue())
for name, tail, target in (("repeating row", ROWS, "row"), ("block control", BLOCK, "blk")):
    for label, edit in (("Paragraph.text", lambda p: setattr(p, "text", "changed")),
                        ("Paragraph.add_run", lambda p: p.add_run("more")),
                        ("Run.text", lambda p: setattr(p.runs[0], "text", "changed"))):
        doc = build(tail)
        p = next(p for p in doc.paragraphs if p.text == target)
        try:
            edit(p); e = "ok"
        except Exception as error:
            e = f"{type(error).__name__}: {error}"[:70]
        try:
            doc.to_bytes(); s = "ok"
        except Exception as error:
            s = f"{type(error).__name__}: {error}"[:70]
        print(f"{name:14s} {label:18s} edit={e:30s} save={s}")
