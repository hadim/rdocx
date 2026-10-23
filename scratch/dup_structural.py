import sys
sys.path.insert(0, sys.argv[1])
import importlib.util
spec = importlib.util.spec_from_file_location("dupids_mod", sys.argv[1] + "/dupids.py")
src = open(spec.origin).read().split("data = with_duplicate_ids")[0]
ns = {}
exec(src, ns)
import rpptx
from rpptx.enum.shapes import MSO_SHAPE
data = ns["with_duplicate_ids"](ns["deck_bytes"]())
for name, edit in {
    "add_shape": lambda s, p: s.shapes.add_shape(MSO_SHAPE.RECTANGLE, 0, 0, 10, 10),
    "add_textbox": lambda s, p: s.shapes.add_textbox(0, 0, 10, 10),
    "shapes.remove": lambda s, p: s.shapes.remove(s.shapes[0]),
    "add_slide": lambda s, p: p.slides.add_slide(p.slide_layouts[6]),
}.items():
    p = rpptx.Presentation.from_bytes(data)
    try:
        edit(p.slides[0], p); print(name, "OK")
    except Exception as e:
        print(name, "ERROR", type(e).__name__, e)
