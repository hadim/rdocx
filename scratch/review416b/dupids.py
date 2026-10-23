"""Footer-owner proofs on a deck where another slide repeats a shape id."""
import io, re, sys, zipfile
import rpptx

def deck_bytes():
    p = rpptx.Presentation()
    for _ in range(3):
        s = p.slides.add_slide(p.slide_layouts[6])
        for _ in range(2):
            s.shapes.add_textbox(0, 0, 100, 100).text_frame.text = "t"
    return p.to_bytes()

def with_duplicate_ids(data, part="ppt/slides/slide3.xml"):
    src = zipfile.ZipFile(io.BytesIO(data))
    out = io.BytesIO()
    with zipfile.ZipFile(out, "w", zipfile.ZIP_DEFLATED) as dst:
        for info in src.infolist():
            body = src.read(info.filename)
            if info.filename == part:
                xml = body.decode()
                ids = re.findall(r'<p:cNvPr id="(\d+)"', xml)
                # make every shape id equal to the first shape's id
                first = ids[1]
                xml = re.sub(r'(<p:cNvPr id=")(\d+)(")', lambda m: m.group(1) + (m.group(2) if m.group(2) == ids[0] else first) + m.group(3), xml)
                body = xml.encode()
            dst.writestr(info, body)
    return out.getvalue()

data = with_duplicate_ids(deck_bytes())
cases = {
    "transition.type": lambda s, p: setattr(s.transition, "type", "fade"),
    "transition.duration": lambda s, p: setattr(s.transition, "duration", 1.5),
    "header_footer.footer": lambda s, p: setattr(s.header_footer, "footer", "F"),
    "header_footer.slide_number": lambda s, p: setattr(s.header_footer, "slide_number", True),
}
for name, edit in cases.items():
    p = rpptx.Presentation.from_bytes(data)
    s = p.slides[0]
    keep = p.slides[2].shapes[0]
    try:
        edit(s, p)
        print(name, "OK", "kept handle text:", keep.text_frame.text)
    except Exception as e:
        print(name, "ERROR", type(e).__name__, e)
