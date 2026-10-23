"""rpptx: cost of replace_image, slides.remove, add_picture and add_slide as a deck grows (neutral, synthetic deck).

    python package_calls.py

Builds decks of N slides, each with one picture (a random-noise PNG of about SIZE_KB, so it does not compress) and a
few text boxes, saves and reopens them, then times replace_image on 5 pictures, slides.remove on 5 slides, add_picture of 5 pictures and add_slide of 5
slides.
"""

import io
import os
import tempfile
import time

import numpy as np
import rpptx
from PIL import Image
from rpptx import Inches


def png(seed: int, side: int) -> bytes:
    rng = np.random.default_rng(seed)
    buffer = io.BytesIO()
    Image.fromarray(rng.integers(0, 255, (side, side, 3), dtype=np.uint8)).save(buffer, format="PNG")
    return buffer.getvalue()


def build(path: str, slides: int, side: int) -> None:
    prs = rpptx.Presentation()
    layout = prs.slide_layouts[6]
    for index in range(slides):
        slide = prs.slides.add_slide(layout)
        slide.shapes.add_picture(io.BytesIO(png(index, side)), Inches(1), Inches(1), Inches(4), Inches(4))
        for box in range(10):
            slide.shapes.add_textbox(Inches(5), Inches(0.5 * box), Inches(3), Inches(0.4)).text_frame.text = f"text {box}"
    prs.save(path)


def timed(path: str, side: int) -> tuple[float, float]:
    prs = rpptx.Presentation(path)
    pictures = [shape for slide in list(prs.slides)[:5] for shape in slide.shapes if shape.shape_type == rpptx.enum.shapes.MSO_SHAPE_TYPE.PICTURE]
    start = time.perf_counter()
    for index, picture in enumerate(pictures):
        picture.replace_image(io.BytesIO(png(10_000 + index, side)))
    replace = (time.perf_counter() - start) / len(pictures)
    prs = rpptx.Presentation(path)
    start = time.perf_counter()
    for slide in list(prs.slides)[:5]:
        prs.slides.remove(slide)
    remove = (time.perf_counter() - start) / 5
    prs = rpptx.Presentation(path)
    slide = prs.slides.add_slide(prs.slide_layouts[6])
    start = time.perf_counter()
    for index in range(5):
        slide.shapes.add_picture(io.BytesIO(png(20_000 + index, side)), Inches(1), Inches(1), Inches(2), Inches(2))
    add = (time.perf_counter() - start) / 5
    start = time.perf_counter()
    for _ in range(5):
        prs.slides.add_slide(prs.slide_layouts[6])
    new_slide = (time.perf_counter() - start) / 5
    return replace, remove, add, new_slide


print("rpptx", __import__("importlib.metadata").metadata.version("rpptx"))
print(f"{'slides':>6} {'png KB':>7} {'deck MB':>8} {'replace_image s':>16} {'slides.remove s':>16} {'add_picture s':>14} {'add_slide s':>12}")
with tempfile.TemporaryDirectory() as folder:
    for slides, side in ((10, 300), (40, 300), (80, 300), (40, 600)):
        path = os.path.join(folder, f"deck_{slides}_{side}.pptx")
        build(path, slides, side)
        kb = len(png(0, side)) / 1e3
        replace, remove, add, new_slide = timed(path, side)
        print(f"{slides:>6} {kb:>7.0f} {os.path.getsize(path) / 1e6:>8.1f} {replace:>16.3f} {remove:>16.3f} {add:>14.3f} {new_slide:>12.3f}")
