"""Footer-owner proofs when another slide holds two mc:AlternateContent shapes."""
import io, re, zipfile
import rpptx

MC = "http://schemas.openxmlformats.org/markup-compatibility/2006"
def ac(inner_id):
    sp = ('<p:sp><p:nvSpPr><p:cNvPr id="%d" name="AC %d"/><p:cNvSpPr/><p:nvPr/></p:nvSpPr>'
          '<p:spPr/><p:txBody><a:bodyPr/><a:p><a:r><a:t>ac</a:t></a:r></a:p></p:txBody></p:sp>') % (inner_id, inner_id)
    return '<mc:AlternateContent xmlns:mc="%s"><mc:Choice Requires="p14">%s</mc:Choice><mc:Fallback>%s</mc:Fallback></mc:AlternateContent>' % (MC, sp, sp)

def deck(ids):
    p = rpptx.Presentation()
    for _ in range(3):
        p.slides.add_slide(p.slide_layouts[6]).shapes.add_textbox(0, 0, 100, 100).text_frame.text = "t"
    src = zipfile.ZipFile(io.BytesIO(p.to_bytes()))
    out = io.BytesIO()
    with zipfile.ZipFile(out, "w", zipfile.ZIP_DEFLATED) as dst:
        for info in src.infolist():
            body = src.read(info.filename)
            if info.filename == "ppt/slides/slide3.xml":
                xml = body.decode()
                xml = xml.replace("</p:spTree>", "".join(ac(i) for i in ids) + "</p:spTree>", 1)
                xml = xml.replace("<p:sld ", '<p:sld xmlns:p14="http://schemas.microsoft.com/office/powerpoint/2010/main" ', 1)
                body = xml.encode()
            dst.writestr(info, body)
    return out.getvalue()

for label, ids in (("distinct ids", (50, 51)), ("same id", (50, 50))):
    data = deck(ids)
    for name, edit in (("transition.type", lambda s: setattr(s.transition, "type", "fade")),
                       ("header_footer.slide_number", lambda s: setattr(s.header_footer, "slide_number", True))):
        p = rpptx.Presentation.from_bytes(data)
        try:
            edit(p.slides[0]); print(label, name, "OK", len(p.slides[2].shapes))
        except Exception as e:
            print(label, name, "ERROR", type(e).__name__, e)
