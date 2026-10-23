"""Profile one rpptx call in a loop: python prof403.py SLIDES CALL [SECONDS] [--sample OUT].

Builds a python-pptx deck like rdocx-skills edit_cost_pptx (25 text boxes, 12 rectangles, a 4x4 table, a ~1 MB
noise PNG and notes a slide), opens it with rpptx, then calls CALL on fresh targets for SECONDS seconds. With
--sample, macOS `sample` records this process during the loop."""
import io
import os
import random
import subprocess
import sys
import tempfile
import time

import pptx
import rpptx
from PIL import Image
from pptx.enum.shapes import MSO_SHAPE
from pptx.util import Inches

SIDE = 580


def noise_png(side, seed):
    buffer = io.BytesIO()
    Image.frombytes("RGB", (side, side), random.Random(seed).randbytes(side * side * 3)).save(buffer, format="PNG")
    return buffer.getvalue()


def build(path, slides):
    prs = pptx.Presentation()
    for k in range(slides):
        slide = prs.slides.add_slide(prs.slide_layouts[6])
        for i in range(25):
            box = slide.shapes.add_textbox(Inches(0.2 + 0.5 * (i % 5)), Inches(0.2 + 0.5 * (i // 5)), Inches(2), Inches(0.4))
            box.text_frame.text = f"text {i}"
        for i in range(12):
            slide.shapes.add_shape(MSO_SHAPE.RECTANGLE, Inches(4 + 0.4 * i), Inches(4), Inches(0.3), Inches(0.3)).fill.solid()
        table = slide.shapes.add_table(4, 4, Inches(0.5), Inches(5), Inches(6), Inches(1.5)).table
        for r in range(4):
            for c in range(4):
                table.cell(r, c).text = f"{r}.{c}"
        slide.shapes.add_picture(io.BytesIO(noise_png(SIDE, k)), Inches(7), Inches(0.5), Inches(2.5))
        slide.notes_slide.notes_text_frame.text = f"notes {k}"
    prs.save(path)


def calls(prs):
    probe = prs.slides.add_slide(prs.slide_layouts[6])
    box = probe.shapes.add_textbox(Inches(1), Inches(1), Inches(2), Inches(0.5))
    box.text_frame.text = "a"
    para = box.text_frame.paragraphs[0]
    pic = probe.shapes.add_picture(io.BytesIO(noise_png(8, 1)), Inches(6), Inches(1))
    images = [noise_png(8, 2), noise_png(8, 3)]
    table = probe.shapes.add_table(2, 2, Inches(1), Inches(5), Inches(3), Inches(1)).table

    def remove_shape(i):
        shape = probe.shapes.add_textbox(Inches(1), Inches(1), Inches(1), Inches(1))
        probe.shapes.remove(shape)

    return {
        "add_run": lambda i: para.add_run().__setattr__("text", f"r{i}"),
        "add_shape": lambda i: probe.shapes.add_shape(MSO_SHAPE.RECTANGLE, Inches(1), Inches(1), Inches(1), Inches(1)),
        "text_frame.text": lambda i: box.text_frame.__setattr__("text", f"t{i}"),
        "title": lambda i: prs.core_properties.__setattr__("title", f"t{i}"),
        "replace_image": lambda i: pic.replace_image(io.BytesIO(images[i % 2])),
        "shapes.remove": remove_shape,
        "cell.fill": lambda i: table.cell(i % 2, (i // 2) % 2).fill.solid(),
        "add_slide": lambda i: prs.slides.add_slide(prs.slide_layouts[6]),
    }


def main():
    slides, name = int(sys.argv[1]), sys.argv[2]
    seconds = float(sys.argv[3]) if len(sys.argv) > 3 and not sys.argv[3].startswith("--") else 0
    out = sys.argv[sys.argv.index("--sample") + 1] if "--sample" in sys.argv else None
    with tempfile.TemporaryDirectory() as folder:
        path = os.path.join(folder, "deck.pptx")
        build(path, slides)
        prs = rpptx.Presentation(path)
        call = calls(prs)[name]
        call(0)
        best = min(timed(call, i) for i in range(1, 4))
        print(f"{name} @ {slides} slides: {best * 1000:.1f} ms (best of 3), deck {os.path.getsize(path) / 1e6:.1f} MB")
        if out:
            sampler = subprocess.Popen(["sample", str(os.getpid()), str(int(seconds)), "1", "-mayDie", "-file", out],
                                       stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
            i = 4
            while sampler.poll() is None:
                call(i)
                i += 1
            print(f"{i - 4} calls sampled into {out}")


def timed(call, i):
    start = time.perf_counter()
    call(i)
    return time.perf_counter() - start


main()
