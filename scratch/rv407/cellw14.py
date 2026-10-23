import io, zipfile, rdocx
W14 = "http://schemas.microsoft.com/office/word/2010/wordml"
doc = rdocx.Document(); doc.add_paragraph("first")
z = zipfile.ZipFile(io.BytesIO(doc.to_bytes())); out = io.BytesIO()
body = ('<w:tbl><w:tblGrid><w:gridCol w:w="1000"/><w:gridCol w:w="1000"/></w:tblGrid><w:tr>'
        f'<w:tc><w:p xmlns:w14="{W14}" w14:paraId="0000AAAA"><w:r><w:t>a</w:t></w:r></w:p></w:tc>'
        '<w:tc><w:p><w:r><w:t>b</w:t></w:r></w:p></w:tc></w:tr></w:tbl><w:p/>')
with zipfile.ZipFile(out, "w") as a:
    for i in z.infolist():
        d = z.read(i.filename)
        if i.filename == "word/document.xml":
            t = d.decode(); at = t.index("<w:sectPr"); d = (t[:at] + body + t[at:]).encode()
        a.writestr(i, d)
doc = rdocx.Document.from_bytes(out.getvalue())
print("cell(0,1) declares w14:", "xmlns:w14" in doc.tables[0].cell(0, 1).xml, " uses:", "w14:" in doc.tables[0].cell(0, 1).xml.replace(f'xmlns:w14="{W14}"', ""))
print("paragraph 0 declares w14:", "xmlns:w14" in doc.paragraphs[0].xml)
