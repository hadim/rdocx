"""The cost of one structural edit grows with the whole package (rpptx shapes, rdocx replace_xml / remove_content).

Builds its own input. Prints the cost per edit in a small and a large package, and FAIL when the large one is more
than 3x the small one (the edited part is the same size in both). Exits 1 if any case fails.
"""

import importlib.metadata
import sys
import time

import rdocx
import rpptx
from rpptx import Inches
from rpptx.enum.shapes import MSO_SHAPE

RATIO = 3.0
results = []


def per_call(op, n=5):
    start = time.perf_counter()
    for _ in range(n):
        op()
    return (time.perf_counter() - start) / n * 1000


def pptx_costs(extra_slides):
    prs = rpptx.Presentation()
    for _ in range(extra_slides):
        slide = prs.slides.add_slide(prs.slide_layouts[6])
        for _ in range(40):
            slide.shapes.add_textbox(0, 0, Inches(1), Inches(1))
    slide = prs.slides.add_slide(prs.slide_layouts[6])
    shape = slide.shapes.add_shape(MSO_SHAPE.OVAL, 0, 0, Inches(1), Inches(1))
    return {
        "add_textbox": per_call(lambda: slide.shapes.add_textbox(0, 0, Inches(1), Inches(1))),
        "alt_text": per_call(lambda: setattr(shape, "alt_text", "x")),
        "left": per_call(lambda: setattr(shape, "left", Inches(1))),
        "fill.solid": per_call(lambda: shape.fill.solid()),
        "replace_xml": per_call(lambda: shape.element.replace_xml(shape.element.xml)),
    }


def docx_costs(n):
    x = '<w:p xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:r><w:t>x</w:t></w:r></w:p>'
    doc = rdocx.Document()
    for i in range(n):
        doc.add_paragraph(f"paragraph {i}")
    p = doc.insert_paragraph(n // 2, "")
    replace = per_call(lambda: p.replace_xml(x))
    k = iter(range(n, 0, -1))
    remove = per_call(lambda: doc.remove_content(next(k)))
    return {"replace_xml": replace, "remove_content": remove}


print("rdocx", importlib.metadata.version("rdocx"), "| rpptx", importlib.metadata.version("rpptx"))
small, large = pptx_costs(0), pptx_costs(40)
for name in small:
    ok = large[name] <= max(RATIO * small[name], 5.0)
    results.append(ok)
    print(f"{'PASS' if ok else 'FAIL'} rpptx {name}: {small[name]:.1f} ms in an empty deck, "
          f"{large[name]:.1f} ms with 40 slides x 40 text boxes")
small, large = docx_costs(50), docx_costs(800)
for name in small:
    ok = large[name] <= max(RATIO * small[name], 5.0)
    results.append(ok)
    print(f"{'PASS' if ok else 'FAIL'} rdocx {name}: {small[name]:.1f} ms with 50 paragraphs, "
          f"{large[name]:.1f} ms with 800")
has_range = any(n in dir(rdocx.Document) for n in ("remove_range", "remove_content_range", "pop_range"))
results.append(has_range)
print(f"{'PASS' if has_range else 'FAIL'} rdocx: a call removes a range of body blocks at once")
sys.exit(0 if all(results) else 1)
