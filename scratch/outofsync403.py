"""Out-of-sync staging cost: python outofsync403.py SLIDES.

Builds a deck of SLIDES slides in memory through the binding, with no edit that commits (25 text boxes and a table a
slide), so every slide record differs from what its part bytes give, then times Shape.flip_h on slide 0 (best of 3)."""
import sys
import time

import rpptx
from rpptx import Inches

slides = int(sys.argv[1])
prs = rpptx.Presentation()
for k in range(slides):
    slide = prs.slides.add_slide(prs.slide_layouts[6])
    for i in range(25):
        slide.shapes.add_textbox(Inches(0.2 + 0.5 * (i % 5)), Inches(0.2 + 0.5 * (i // 5)), Inches(2), Inches(0.4)).text_frame.text = f"text {i}"
    slide.shapes.add_table(4, 4, Inches(0.5), Inches(5), Inches(6), Inches(1.5))
shape = prs.slides[0].shapes[0]
best = []
for i in range(4):
    start = time.perf_counter()
    shape.flip_h = i % 2 == 0
    best.append(time.perf_counter() - start)
print(f"flip_h @ {slides} in-memory slides: {min(best[1:]) * 1000:.1f} ms (best of 3)")
