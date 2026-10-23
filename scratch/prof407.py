"""Profile one rdocx call in a loop: python prof407.py PARAGRAPHS CALL SECONDS OUT (repro407's document build)."""
import io
import os
import subprocess
import sys
import time
import zipfile

import rdocx

RPR = (
    '<w:rPr><w:rFonts w:ascii="Arial" w:cs="Arial" w:eastAsia="Arial" w:hAnsi="Arial"/><w:b w:val="0"/>'
    '<w:i w:val="0"/><w:smallCaps w:val="0"/><w:strike w:val="0"/><w:color w:val="000000"/><w:sz w:val="20"/>'
    '<w:szCs w:val="20"/><w:u w:val="none"/><w:shd w:fill="auto" w:val="clear"/><w:vertAlign w:val="baseline"/>'
    '<w:rtl w:val="0"/></w:rPr>'
)


def build(paragraphs):
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


def main():
    count, name, seconds, out = int(sys.argv[1]), sys.argv[2], float(sys.argv[3]), sys.argv[4]
    doc = build(count)
    p = doc.paragraphs[20]
    ops = {
        "xml": lambda: p.xml,
        "insert_paragraph": lambda: doc.insert_paragraph(10, "x"),
        "remove_content": lambda: (doc.insert_paragraph(10, "x"), doc.remove_content(10)),
    }
    op = ops[name]
    sampler = subprocess.Popen(["sample", str(os.getpid()), str(int(seconds)), "1", "-mayDie", "-file", out],
                               stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    n = 0
    while sampler.poll() is None:
        op()
        n += 1
    print(f"{name}: {n} calls in {seconds + 1:.0f} s, sampled into {out}")


main()
