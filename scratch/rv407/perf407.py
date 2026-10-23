"""Which document shapes take the #407 fast paths: time each call on a shape padded with 600 plain paragraphs."""
import io
import json
import sys
import time
import zipfile

sys.path.insert(0, sys.argv[1])
import rdocx  # noqa: E402

sys.argv = sys.argv[:1] + ["0", "0"]
sys.path.insert(0, "/private/tmp/claude-501/-Users-hadim-Code-libs-rdocx/08057ee4-afc5-4751-aca1-2668752380de/scratchpad/rv407")
import diff407  # noqa: E402

RPR = '<w:rPr><w:rFonts w:ascii="Arial" w:hAnsi="Arial"/><w:b w:val="0"/><w:color w:val="000000"/><w:sz w:val="20"/></w:rPr>'
FILL = list(
    f"<w:p>{''.join(f'<w:r>{RPR}<w:t xml:space=\"preserve\">fill {i} run {k} </w:t></w:r>' for k in range(4))}</w:p>"
    for i in range(600)
)


def build(name, extra):
    body, root = diff407.shape(name)
    doc = rdocx.Document()
    doc.add_paragraph("first")
    if extra == "header":
        doc.section(0).header.add_paragraph("h")
        doc.section(0).header.add_table(1, 2)
    if extra == "footnote":
        doc.add_paragraph("noted")
        doc.add_footnote(doc.paragraphs[-1], "foot")
    if extra == "comment":
        doc.add_comment_on_text("first", author="a", text="c")
    z = zipfile.ZipFile(io.BytesIO(doc.to_bytes()))
    out = io.BytesIO()
    with zipfile.ZipFile(out, "w") as archive:
        for info in z.infolist():
            data = z.read(info.filename)
            if info.filename == "word/document.xml":
                text = data.decode()
                at = text.index("<w:sectPr")
                text = text[:at] + "".join(FILL[:300]) + body + "".join(FILL[300:]) + text[at:]
                if root:
                    text = text.replace("<w:document ", "<w:document" + root + " ", 1)
                data = text.encode()
            archive.writestr(info, data)
    return rdocx.Document.from_bytes(out.getvalue())


def ms(fn, n=3):
    best = None
    for _ in range(n):
        t = time.perf_counter()
        fn()
        e = (time.perf_counter() - t) * 1000
        best = e if best is None else min(best, e)
    return round(best, 2)


def measure(doc):
    out = {}
    slot = doc.find_content_index("fill 450 run 0 fill 450 run 1 fill 450 run 2 fill 450 run 3 ".strip()) if False else None
    paragraphs = doc.paragraphs
    target = next(i for i, p in enumerate(paragraphs) if p.text.startswith("fill 450 "))
    p = paragraphs[target]
    slot = doc.find_content_index(p)
    out["Paragraph.xml"] = ms(lambda: p.xml)
    run = p.runs[1]
    out["Run.xml"] = ms(lambda: run.xml)
    if len(doc.tables):
        t = doc.tables[0]
        out["Table.xml"] = ms(lambda: t.xml)
        c = t.cell(0, 0)
        out["Cell.xml"] = ms(lambda: c.xml)

    def ins_rm():
        doc.insert_paragraph(slot, "x")
    out["insert_paragraph"] = ms(lambda: doc.insert_paragraph(slot, "x"))
    out["remove_content"] = ms(lambda: doc.remove_content(slot))
    out["remove_content_range"] = ms(lambda: doc.remove_content_range(slot, slot + 1))
    out["insert_table"] = ms(lambda: doc.insert_table(slot, 2, 2))
    out["add_paragraph"] = ms(lambda: doc.add_paragraph("tail"))
    out["add_table"] = ms(lambda: doc.add_table(1, 2))
    out["to_bytes(ref)"] = ms(doc.to_bytes, 1)
    return out


cases = [("plain", None), ("plain", "header"), ("plain", "footnote"), ("plain", "comment"), ("word", None),
         ("tables", None), ("sdt", None), ("w14inside", None), ("customxml", None), ("altchunk", None),
         ("textbox", None), ("lastsect", None)]
result = {}
for name, extra in cases:
    key = f"{name}+{extra}" if extra else name
    try:
        result[key] = measure(build(name, extra))
    except Exception as error:
        result[key] = {"error": f"{type(error).__name__}: {error}"}
print(json.dumps(result))
