"""Which of the #407 calls fall back to the whole-body route on an edit_cost_docx document (debug counters)."""
import sys
from pathlib import Path
sys.path.insert(0, sys.argv[1])  # skills tests folder
import rdocx
from builders import edit_cost_docx
from rdocx import _rdocx

out = Path(sys.argv[2])
path = edit_cost_docx(out / "n.docx", 25, picture_side=100)
work = _rdocx._word_whole_body_work


def para(doc, k):
    return next(doc.paragraphs[i] for i in range(len(doc.paragraphs) - 1, -1, -1)
                if doc.paragraphs[i].text == f"alpha beta {k}")


def measure(name, setup):
    doc = rdocx.Document.open(str(path))
    action = setup(doc)
    before = work()
    try:
        action()
        result = "ok"
    except Exception as error:  # noqa: BLE001
        result = f"{type(error).__name__}: {error}"
    after = work()
    print(f"{name:34s} rebuilds={after[0] - before[0]} whole_reads={after[1] - before[1]} {result}")


measure("open (first call baseline)", lambda doc: lambda: None)
measure("Paragraph.xml", lambda doc: (lambda p: lambda: p.xml)(para(doc, 0)))
measure("Run.xml", lambda doc: (lambda r: lambda: r.xml)(para(doc, 0).runs[1]))
measure("Table.xml", lambda doc: (lambda t: lambda: t.xml)(doc.tables[0]))
measure("Cell.xml", lambda doc: (lambda c: lambda: c.xml)(doc.tables[0].cell(0, 0)))
measure("Table.xml (last)", lambda doc: (lambda t: lambda: t.xml)(doc.tables[len(doc.tables) - 1]))
measure("Document.insert_paragraph(10)", lambda doc: lambda: doc.insert_paragraph(10, "x"))
measure("Document.add_paragraph", lambda doc: lambda: doc.add_paragraph("x"))
measure("Document.add_table", lambda doc: lambda: doc.add_table(2, 2))
measure("Document.insert_table(10)", lambda doc: lambda: doc.insert_table(10, 2, 2))
measure("Document.remove_content(alpha)", lambda doc: (lambda i: lambda: doc.remove_content(i))(doc.find_content_index(para(doc, 0))))
measure("Document.remove_content_range", lambda doc: (lambda i: lambda: doc.remove_content_range(i, i + 1))(doc.find_content_index(para(doc, 0))))
measure("Paragraph.insert_paragraph_before", lambda doc: (lambda p: lambda: p.insert_paragraph_before("x"))(para(doc, 0)))
