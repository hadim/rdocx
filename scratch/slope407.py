"""Cost per call at 100, 1000 and 4000 body paragraphs (repro407's input)."""
import sys
import time

sys.argv = [sys.argv[0]]
import importlib.util
spec = importlib.util.spec_from_file_location("siblings", __file__.replace("slope407.py", "siblings407.py"))
siblings = importlib.util.module_from_spec(spec)
sys.modules["siblings"] = siblings
src = open(spec.origin).read().split('print("rdocx"')[0]
exec(compile(src, spec.origin, "exec"), siblings.__dict__)

rows = {}
for count in (100, 1000, 4000):
    doc = siblings.build(count)
    p = doc.paragraphs[20]
    run = p.runs[3]
    table = doc.tables[0]
    cell = table.cell(0, 0)
    rows[count] = {
        "Paragraph.xml": siblings.per_call(lambda: p.xml, n=5),
        "Run.xml": siblings.per_call(lambda: run.xml, n=5),
        "Table.xml": siblings.per_call(lambda: table.xml, n=5),
        "Cell.xml": siblings.per_call(lambda: cell.xml, n=5),
        "insert_paragraph": siblings.per_call(lambda: doc.insert_paragraph(10, "x"), n=5),
        "remove_content": siblings.per_call(lambda: doc.remove_content(10), n=5),
        "insert_paragraph_before": siblings.per_call(lambda: doc.paragraphs[10].insert_paragraph_before("x"), n=5),
        "remove_content_range": siblings.per_call(
            lambda: doc.remove_content_range(10, 12),
            n=5, setup=lambda: (doc.insert_paragraph(10, "a"), doc.insert_paragraph(10, "b"))),
        "add_paragraph": siblings.per_call(lambda: doc.add_paragraph("x"), n=5),
        "insert_table": siblings.per_call(lambda: doc.insert_table(10, 2, 2), n=5),
        "add_table": siblings.per_call(lambda: doc.add_table(2, 2), n=5),
    }
for name in rows[100]:
    print(f"{name:24s} " + "  ".join(f"{rows[c][name]:9.3f} ms at {c}" for c in rows))
