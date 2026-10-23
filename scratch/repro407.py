"""rdocx: one Paragraph.xml, insert_paragraph or remove_content costs a time that grows with the size of the document's
XML (its body), not with the element. Builds its own input: body paragraphs whose runs carry the verbose run
properties Google Docs writes on every run.

Prints the cost per call in a small and a large body (the same paragraph read, the same edit), and FAIL when the large
one is more than 3 times the small one. Exits 1 if any case fails.
"""

import importlib.metadata
import io
import sys
import time
import zipfile

import rdocx

RATIO = 3.0
RPR = (
    '<w:rPr><w:rFonts w:ascii="Arial" w:cs="Arial" w:eastAsia="Arial" w:hAnsi="Arial"/><w:b w:val="0"/>'
    '<w:i w:val="0"/><w:smallCaps w:val="0"/><w:strike w:val="0"/><w:color w:val="000000"/><w:sz w:val="20"/>'
    '<w:szCs w:val="20"/><w:u w:val="none"/><w:shd w:fill="auto" w:val="clear"/><w:vertAlign w:val="baseline"/>'
    '<w:rtl w:val="0"/></w:rPr>'
)


def build(paragraphs: int) -> rdocx.Document:
    doc = rdocx.Document()
    doc.add_paragraph("first")
    z = zipfile.ZipFile(io.BytesIO(doc.to_bytes()))
    parts = {i.filename: z.read(i.filename) for i in z.infolist()}
    xml = parts["word/document.xml"].decode()
    body = "".join(
        f"<w:p>{''.join(f'<w:r>{RPR}<w:t xml:space=\"preserve\">run {k} of paragraph {i} </w:t></w:r>' for k in range(6))}</w:p>"
        for i in range(paragraphs)
    )
    at = xml.index("<w:sectPr")
    parts["word/document.xml"] = (xml[:at] + body + xml[at:]).encode()
    out = io.BytesIO()
    with zipfile.ZipFile(out, "w") as archive:
        for name, data in parts.items():
            archive.writestr(name, data)
    return rdocx.Document.from_bytes(out.getvalue())


def per_call(op, n=3):
    start = time.perf_counter()
    for _ in range(n):
        op()
    return (time.perf_counter() - start) / n * 1000


def costs(doc):
    p = doc.paragraphs[20]
    return {
        "Paragraph.xml": per_call(lambda: p.xml),
        "insert_paragraph": per_call(lambda: doc.insert_paragraph(10, "x")),
        "remove_content": per_call(lambda: doc.remove_content(10)),
    }


print("rdocx", importlib.metadata.version("rdocx"))
small, large = build(100), build(1000)
sizes = [len(zipfile.ZipFile(io.BytesIO(d.to_bytes())).read("word/document.xml")) for d in (small, large)]
print(f"document.xml: {sizes[0]} and {sizes[1]} bytes (100 and 1000 paragraphs of 6 runs)")
a, b = costs(small), costs(large)
failed = False
for name in a:
    bad = b[name] > RATIO * max(a[name], 1.0)
    failed |= bad
    print(f"{'FAIL' if bad else 'PASS'} {name}: {a[name]:.1f} ms with 100 paragraphs, {b[name]:.1f} ms with 1000")
sys.exit(1 if failed else 0)
