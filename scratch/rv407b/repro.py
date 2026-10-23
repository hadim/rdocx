"""Minimal repro: a plain removal next to an unreconcilable nested owner. Main refuses the edit, the branch accepts it
and then cannot save."""
import io, zipfile, sys
import rdocx

W = "http://schemas.openxmlformats.org/wordprocessingml/2006/main"
W14 = "http://schemas.microsoft.com/office/word/2010/wordml"
W15 = "http://schemas.microsoft.com/office/word/2012/wordml"
ROWS = (f'<w:sdt xmlns:w="{W}" xmlns:w15="{W15}"><w:sdtPr><w:tag w:val="rows"/><w15:repeatingSection/></w:sdtPr>'
        '<w:sdtContent><w:sdt><w:sdtPr><w15:repeatingSectionItem/></w:sdtPr><w:sdtContent><w:p><w:r><w:t>row</w:t>'
        '</w:r></w:p></w:sdtContent></w:sdt></w:sdtContent></w:sdt>')
BOX = (f'<w:sdt xmlns:w14="{W14}"><w:sdtPr><w14:checkbox><w14:checked w14:val="0"/></w14:checkbox></w:sdtPr>'
       '<w:sdtContent><w:p><w:r><w:t>box</w:t></w:r></w:p></w:sdtContent></w:sdt>')
ALIAS = f'<q:sdt xmlns:q="{W}"><q:sdtPr/><q:sdtContent><q:p><q:r><q:t>in q</q:t></q:r></q:p></q:sdtContent></q:sdt>'
CASES = {
    "two identical python-docx repeating sections": ROWS + ROWS,
    "two distinct repeating sections": ROWS + ROWS.replace('w:val="rows"', 'w:val="rows2"'),
    "two identical w14 checkboxes": BOX + BOX,
    "one block control under an alias prefix": ALIAS,
}


def document(owners):
    doc = rdocx.Document()
    doc.add_paragraph("keep")
    out = io.BytesIO()
    with zipfile.ZipFile(io.BytesIO(doc.to_bytes())) as src, zipfile.ZipFile(out, "w") as dst:
        for info in src.infolist():
            data = src.read(info.filename)
            if info.filename == "word/document.xml":
                text = data.decode()
                at = text.index("<w:sectPr")
                data = (text[:at] + "<w:p><w:r><w:t>drop</w:t></w:r></w:p>" + owners + text[at:]).encode()
            dst.writestr(info, data)
    return rdocx.Document.from_bytes(out.getvalue())


def attempt(fn):
    try:
        return f"ok {fn()!r}"
    except Exception as error:
        return f"{type(error).__name__}: {error}"


for name, owners in CASES.items():
    for label, edit in (("remove_content", lambda d: d.remove_content(1)),
                        ("replace_xml", lambda d: d.paragraphs[1].replace_xml(d.paragraphs[1].xml.replace(">drop<", ">x<"))),
                        ("insert_paragraph", lambda d: d.insert_paragraph(1, "new").text)):
        doc = document(owners)
        print(f"{name:46s} {label:16s} edit: {attempt(lambda: edit(doc))[:60]:60s} save: {attempt(lambda: len(doc.to_bytes()) > 0)[:70]}")
