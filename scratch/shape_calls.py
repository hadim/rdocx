"""rpptx: per-call cost of shape-level edits as one in-memory deck grows (neutral, synthetic deck).

    python shape_calls.py [--media] [SLIDES ...]   # 10, 40 and 80 slides by default

Without --media, slides of text boxes, autoshapes and a table; with it, also a picture and speaker notes a slide.

The issue 389 build (slides of 25 text boxes with an alt text and 12 autoshapes with a solid fill) plus a 4x4 table a
slide, grown in one Presentation. With --media each slide also holds a random-noise PNG of about 270 KB (it does not
compress) and a notes slide. At each size, each call below is timed 10 times on a fresh shape, cell or slide,
and its mean printed, with the time the last 10 slides took to build. Every timed call changes something: the
same call repeated on an unchanged target returns early and would measure nothing.
"""

import argparse
import io
import time

import numpy as np
import rpptx
from PIL import Image
from rpptx import Inches
from rpptx.dml.color import RGBColor
from rpptx.enum.shapes import MSO_SHAPE


def png(seed: int, side: int) -> bytes:
    rng = np.random.default_rng(seed)
    buffer = io.BytesIO()
    Image.fromarray(rng.integers(0, 255, (side, side, 3), dtype=np.uint8)).save(buffer, format="PNG")
    return buffer.getvalue()


def add_content(prs, layout, index: int, media: bool) -> None:
    slide = prs.slides.add_slide(layout)
    for box_index in range(25):
        box = slide.shapes.add_textbox(Inches(0.2 + 0.5 * (box_index % 5)), Inches(0.2 + 0.5 * (box_index // 5)), Inches(2), Inches(0.4))
        box.text_frame.text = f"text {box_index}"
        box.alt_text = f"tag {box_index}"
    for shape_index in range(12):
        shape = slide.shapes.add_shape(MSO_SHAPE.RECTANGLE, Inches(4 + 0.4 * shape_index), Inches(4), Inches(0.3), Inches(0.3))
        shape.fill.solid()
        shape.fill.fore_color.rgb = RGBColor(0x1F, 0x3A, 0x5F)
    table = slide.shapes.add_table(4, 4, Inches(0.5), Inches(5), Inches(6), Inches(1.5)).table
    for row in range(4):
        for column in range(4):
            table.cell(row, column).text_frame.text = f"{row}.{column}"
    if media:
        slide.shapes.add_picture(io.BytesIO(png(index, 300)), Inches(7), Inches(0.5), Inches(2.5), Inches(2.5))
        slide.notes_slide.notes_text_frame.text = f"notes {index}"


def mean(call, count: int = 10) -> float:
    start = time.perf_counter()
    for index in range(count):
        call(index)
    return (time.perf_counter() - start) / count * 1000


def probe(prs, layout) -> dict[str, float]:
    slide = prs.slides.add_slide(layout)
    shapes = [slide.shapes.add_shape(MSO_SHAPE.RECTANGLE, Inches(1), Inches(1), Inches(1), Inches(1)) for _ in range(10)]
    for shape in shapes:
        shape.text_frame.text = "a"
    boxes = [slide.shapes.add_textbox(Inches(1), Inches(3), Inches(2), Inches(0.4)) for _ in range(10)]
    table = slide.shapes.add_table(4, 4, Inches(1), Inches(5), Inches(3), Inches(1)).table
    small = [png(1_000 + index, 32) for index in range(20)]
    pictures = []
    fresh = [prs.slides.add_slide(layout) for _ in range(10)]
    added = []
    cell = lambda i: table.cell(i // 4, i % 4)  # noqa: E731, ten distinct cells
    results = {
        "add_textbox": mean(lambda i: slide.shapes.add_textbox(Inches(5), Inches(5), Inches(1), Inches(0.4))),
        "add_shape": mean(lambda i: slide.shapes.add_shape(MSO_SHAPE.OVAL, Inches(5), Inches(1), Inches(1), Inches(1))),
        "add_picture": mean(lambda i: pictures.append(slide.shapes.add_picture(io.BytesIO(small[i]), Inches(6), Inches(1)))),
        "text_frame.text": mean(lambda i: setattr(boxes[i].text_frame, "text", "b")),
        "cell.text_frame.text": mean(lambda i: setattr(cell(i).text_frame, "text", "b")),
        "fill.solid": mean(lambda i: shapes[i].fill.solid()),
        "fore_color.rgb": mean(lambda i: setattr(shapes[i].fill.fore_color, "rgb", RGBColor(0x1F, 0x3A, 0x5F))),
        "fill.background": mean(lambda i: shapes[i].fill.background()),
        "line.fill.background": mean(lambda i: shapes[i].line.fill.background()),
        "cell.fill.solid": mean(lambda i: cell(i).fill.solid()),
        "cell.fill.background": mean(lambda i: cell(i).fill.background()),
        "alt_text": mean(lambda i: setattr(shapes[i], "alt_text", "x")),
        "left": mean(lambda i: setattr(shapes[i], "left", Inches(2))),
        "replace_xml (same XML)": mean(lambda i: shapes[i].element.replace_xml(shapes[i].element.xml)),
        "replace_xml (new text)": mean(lambda i: shapes[i].element.replace_xml(shapes[i].element.xml.replace(">a<", ">c<"))),
        "add_run": mean(lambda i: boxes[i].text_frame.paragraphs[0].add_run()),
        "notes_text_frame": mean(lambda i: fresh[i].notes_slide.notes_text_frame),
        "replace_image": mean(lambda i: pictures[i].replace_image(io.BytesIO(small[10 + i]))),
        "shapes.remove": mean(lambda i: slide.shapes.remove(shapes[i])),
        "add_slide": mean(lambda i: added.append(prs.slides.add_slide(layout))),
        "slides.remove": mean(lambda i: prs.slides.remove(added[i])),
    }
    for extra in [slide, *fresh]:
        prs.slides.remove(extra)
    return results


parser = argparse.ArgumentParser()
parser.add_argument("--media", action="store_true", help="add a picture and speaker notes to each slide")
parser.add_argument("slides", nargs="*", type=int, default=[10, 40, 80], help="deck sizes to probe")
args = parser.parse_args()
sizes = args.slides

version = __import__("importlib.metadata").metadata.version("rpptx")
print("rpptx", version, "- mean ms a call -", "with a picture and notes a slide" if args.media else "no picture, no notes")
prs = rpptx.Presentation()
layout = prs.slide_layouts[6]
rows = {}
built = 0
for target in sizes:
    start = time.perf_counter()
    while built < target:
        add_content(prs, layout, built, args.media)
        built += 1
        if built == target - 10:
            start = time.perf_counter()
    last_ten = time.perf_counter() - start
    rows[target] = (last_ten, probe(prs, layout))
    print(f"{target} slides done", flush=True)

names = list(rows[sizes[0]][1])
print(f"{'call':<24}" + "".join(f"{f'{n} slides':>12}" for n in rows))
print(f"{'build 10 slides (s)':<24}" + "".join(f"{rows[n][0]:>12.1f}" for n in rows))
for name in names:
    print(f"{name:<24}" + "".join(f"{rows[n][1][name]:>12.1f}" for n in rows))
