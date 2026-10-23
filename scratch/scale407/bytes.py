"""Save bytes after each #407 call on an edit_cost_docx document, to compare two builds."""
import hashlib, sys
import rdocx
path = sys.argv[1]
def para(doc, k):
    return next(doc.paragraphs[i] for i in range(len(doc.paragraphs) - 1, -1, -1) if doc.paragraphs[i].text == f"alpha beta {k}")
calls = {
    "insert_paragraph": lambda doc: doc.insert_paragraph(10, "x"),
    "add_paragraph": lambda doc: doc.add_paragraph("x"),
    "add_table": lambda doc: doc.add_table(2, 2),
    "insert_table": lambda doc: doc.insert_table(10, 2, 2),
    "remove_content": lambda doc: doc.remove_content(doc.find_content_index(para(doc, 0))),
    "remove_content_range": lambda doc: doc.remove_content_range(doc.find_content_index(para(doc, 1)), doc.find_content_index(para(doc, 1)) + 1),
    "insert_paragraph_before": lambda doc: para(doc, 2).insert_paragraph_before("x"),
}
for name, call in calls.items():
    doc = rdocx.Document.open(path)
    call(doc)
    data = doc.to_bytes()
    again = rdocx.Document.from_bytes(data)
    print(name, hashlib.sha256(data).hexdigest()[:16], len(again.paragraphs), [p.text for p in again.paragraphs][-3:])
