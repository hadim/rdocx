import sys
shape, ops = sys.argv[1], sys.argv[2].split(",")
sys.argv = ["x"]
sys.path.insert(0, "/private/tmp/claude-501/-Users-hadim-Code-libs-rdocx/08057ee4-afc5-4751-aca1-2668752380de/scratchpad/rv407b")
import guard407 as g
import rdocx
body, root = g.SHAPES[shape]
doc = g.build(body, root)
table = dict(g.OPS)
for name in ops:
    if name == "a2x":
        g.replace(doc, "s-a2", "s-a2x")
    else:
        table[name](doc)
xml = rdocx.Document.from_bytes(doc.to_bytes()).part_xml("/word/document.xml")
print(xml[xml.index("<w:body>"):])
