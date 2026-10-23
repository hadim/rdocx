import sys
sys.argv = ["x"]
sys.path.insert(0, "/private/tmp/claude-501/-Users-hadim-Code-libs-rdocx/08057ee4-afc5-4751-aca1-2668752380de/scratchpad/rv407b")
import guard407 as g
import rdocx
body, root = g.SHAPES[sys.argv[1] if len(sys.argv) > 1 else "sdt_w14_checkbox"]
doc = g.build(body, root)
for name, fn in g.OPS[:5]:
    fn(doc)
data = doc.to_bytes()
xml = rdocx.Document.from_bytes(data).part_xml("/word/document.xml")
print(xml[xml.index("<w:body>"):])
