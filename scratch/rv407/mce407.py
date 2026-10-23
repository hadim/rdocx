import io, sys, zipfile
import rdocx
W14 = "http://schemas.microsoft.com/office/word/2010/wordml"
def build(body):
    doc = rdocx.Document(); doc.add_paragraph("first")
    z = zipfile.ZipFile(io.BytesIO(doc.to_bytes())); out = io.BytesIO()
    with zipfile.ZipFile(out, "w") as a:
        for i in z.infolist():
            d = z.read(i.filename)
            if i.filename == "word/document.xml":
                t = d.decode(); at = t.index("<w:sectPr"); d = (t[:at] + body + t[at:]).encode()
            a.writestr(i, d)
    return rdocx.Document.from_bytes(out.getvalue())
body = ('<w:p><w:r><mc:AlternateContent><mc:Choice Requires="w14"><w:t>choice</w:t></mc:Choice><mc:Fallback><w:t>fallback</w:t></mc:Fallback></mc:AlternateContent></w:r></w:p>'
        f'<w:p xmlns:w14="{W14}" w14:paraId="1234ABCD"><w:r><w:t>id</w:t></w:r></w:p>')
doc = build(body)
for i, p in enumerate(doc.paragraphs):
    x = p.xml
    print(i, repr(p.text), "xmlns:w14" in x, 'Requires="w14"' in x)
    if 'Requires="w14"' in x:
        print(x)
        try:
            p.replace_xml(x); print("replace ok", repr(p.text))
        except Exception as e:
            print("replace err", type(e).__name__, e)
        try:
            rdocx.Document().add_paragraph("")
        except Exception: pass
