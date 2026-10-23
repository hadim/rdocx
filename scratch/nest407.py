import rdocx
doc = rdocx.Document()
p = doc.add_paragraph("keep")
cell = doc.add_table(1, 1).cell(0, 0)
depth = 1
try:
    for depth in range(2, 40):
        cell = cell.add_table(1, 1).cell(0, 0)
except Exception as error:
    print("add_table refused at depth", depth, type(error).__name__, str(error)[:120])
else:
    print("reached depth", depth)
try:
    print("xml ok", len(doc.paragraphs[0].xml))
except Exception as error:
    print("xml error", type(error).__name__, str(error)[:120])
try:
    doc.to_bytes(); print("save ok")
except Exception as error:
    print("save error", type(error).__name__, str(error)[:120])
