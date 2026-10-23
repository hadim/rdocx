"""Differential run of seeded edit sequences: handle validity, op outcomes and saved bytes.

    python diffops.py LIBDIR SEED STEPS OUT
"""

import hashlib
import io
import json
import struct
import zlib
import random
import sys

sys.path.insert(0, sys.argv[1])
import rpptx  # noqa: E402

SEED = int(sys.argv[2])
STEPS = int(sys.argv[3])
OUT = sys.argv[4]


def png(seed):
    def chunk(kind, data):
        body = kind + data
        return struct.pack(">I", len(data)) + body + struct.pack(">I", zlib.crc32(body) & 0xFFFFFFFF)

    raw = b"".join(b"\x00" + bytes([seed % 256, (seed * 7) % 256, 3]) * 4 for _ in range(4))
    return (
        b"\x89PNG\r\n\x1a\n"
        + chunk(b"IHDR", struct.pack(">IIBBBBB", 4, 4, 8, 2, 0, 0, 0))
        + chunk(b"IDAT", zlib.compress(raw))
        + chunk(b"IEND", b"")
    )


def err(exc):
    return f"{type(exc).__name__}: {exc}"


def probe(kind, handle):
    try:
        if kind == "slide":
            return ("ok", handle.slide_id, len(handle.shapes))
        if kind == "shape":
            text = handle.text_frame.text if handle.has_text_frame else None
            return ("ok", handle.shape_id, handle.name, text)
        if kind in ("frame", "para", "run", "cell"):
            return ("ok", handle.text)
        if kind == "layout":
            return ("ok", handle.name)
        if kind == "element":
            return ("ok", hashlib.sha256(handle.xml.encode()).hexdigest()[:16])
        if kind == "layout_count":
            return ("ok", handle())
        if kind == "table":
            return ("ok", len(handle.rows), len(handle.columns))
    except BaseException as exc:  # noqa: BLE001 - PanicException is a BaseException
        if isinstance(exc, KeyboardInterrupt):
            raise
        return ("err", err(exc))
    return ("?",)


def main():
    rng = random.Random(SEED)
    prs = rpptx.Presentation()
    pool = []

    def keep(kind, handle):
        if len(pool) < 400:
            pool.append((kind, handle))
        return handle

    other = rpptx.Presentation()
    for index in range(3):
        oslide = other.slides.add_slide(other.slide_layouts[1 if index else 6])
        oshape = oslide.shapes.add_textbox(0, 0, 400, 400)
        oshape.text_frame.text = f"other {index} foo"
        if index == 2:
            oslide.shapes.add_picture(io.BytesIO(png(200)), 0, 0, 300, 300)

    class Layouts:
        def __getitem__(self, index):
            return prs.slide_layouts[index]

        def __len__(self):
            return len(prs.slide_layouts)

        def duplicate(self, layout):
            return prs.slide_layouts.duplicate(layout)

        def remove(self, layout):
            return prs.slide_layouts.remove(layout)

    layouts = Layouts()
    for index in range(6):
        slide = prs.slides.add_slide(layouts[6 if index % 2 else 1])
        keep("slide", slide)
        for box in range(3):
            shape = slide.shapes.add_textbox(100 * box, 100, 900, 300)
            shape.text_frame.text = f"s{index} box{box} foo"
            keep("shape", shape)
        rect = slide.shapes.add_shape("rect", 0, 0, 500, 500)
        rect.text_frame.text = f"rect {index} foo"
        if index % 3 == 0:
            table = slide.shapes.add_table(2, 2, 0, 0, 900, 900)
            table.table.cell(0, 0).text = f"cell {index} foo"
        if index % 2 == 0:
            slide.notes_slide.notes_text_frame.text = f"notes {index} foo"

    def rand_slide():
        return prs.slides[rng.randrange(len(prs.slides))]

    def rand_shape(slide=None):
        slide = slide or rand_slide()
        if len(slide.shapes) == 0:
            raise LookupError("empty slide")
        return slide.shapes[rng.randrange(len(slide.shapes))]

    def text_shape():
        for _ in range(10):
            shape = rand_shape()
            if shape.has_text_frame:
                return shape
        raise LookupError("no text shape")

    def table_shape():
        candidates = [
            shape for slide in prs.slides for shape in slide.shapes if shape.has_table
        ]
        if not candidates:
            raise LookupError("no table")
        return candidates[rng.randrange(len(candidates))]

    def op_add_textbox():
        shape = rand_slide().shapes.add_textbox(1, 2, 300, 300)
        keep("shape", shape)
        keep("frame", shape.text_frame)

    def op_add_textbox_text():
        shape = rand_slide().shapes.add_textbox(1, 2, 300, 300)
        shape.text_frame.text = f"new {rng.randrange(100)} foo"
        keep("shape", shape)

    def op_add_shape():
        keep("shape", rand_slide().shapes.add_shape("ellipse", 0, 0, 200, 200))

    def op_frame_text():
        shape = text_shape()
        keep("frame", shape.text_frame)
        shape.text_frame.text = f"t{rng.randrange(100)}\nline foo"

    def op_add_run():
        frame = text_shape().text_frame
        paragraph = frame.paragraphs[0]
        keep("para", paragraph)
        keep("run", paragraph.add_run(f"r{rng.randrange(100)}"))

    def op_add_paragraph():
        frame = text_shape().text_frame
        keep("frame", frame)
        keep("para", frame.add_paragraph())

    def op_run_text():
        frame = text_shape().text_frame
        paragraph = frame.paragraphs[0]
        if len(paragraph.runs) == 0:
            raise LookupError("no run")
        run = paragraph.runs[0]
        keep("run", run)
        run.text = f"rt{rng.randrange(100)}"

    def op_add_slide():
        keep("slide", prs.slides.add_slide(layouts[rng.choice([1, 6])]))

    def op_add_slide_at():
        position = rng.randrange(len(prs.slides) + 1)
        keep("slide", prs.slides.add_slide_at(layouts[6], position))

    def op_move_slide():
        count = len(prs.slides)
        prs.slides.move(rng.randrange(count), rng.randrange(count))

    def op_remove_slide():
        if len(prs.slides) < 3:
            raise LookupError("keep slides")
        prs.slides.remove(rand_slide())

    def op_duplicate_slide():
        keep("slide", prs.slides.duplicate(rand_slide()))

    def op_remove_shape():
        slide = rand_slide()
        slide.shapes.remove(rand_shape(slide))

    def op_move_shape():
        slide = rand_slide()
        count = len(slide.shapes)
        if count < 2:
            raise LookupError("few shapes")
        slide.shapes.move(rng.randrange(count), rng.randrange(count))

    def op_group():
        slide = rand_slide()
        count = len(slide.shapes)
        if count < 3:
            raise LookupError("few shapes")
        start = rng.randrange(count - 1)
        group = slide.shapes.group([slide.shapes[start], slide.shapes[start + 1]])
        keep("shape", group)

    def op_ungroup():
        groups = [
            shape for slide in prs.slides for shape in slide.shapes if shape.shape_type == 6
        ]
        if not groups:
            raise LookupError("no group")
        for child in groups[rng.randrange(len(groups))].ungroup():
            keep("shape", child)

    def op_cell_text():
        table = table_shape().table
        keep("table", table)
        cell = table.cell(rng.randrange(len(table.rows)), rng.randrange(len(table.columns)))
        keep("cell", cell)
        cell.text = f"c{rng.randrange(100)} foo"

    def op_add_row():
        table = table_shape().table
        keep("table", table)
        table.rows.add_row()

    def op_add_column():
        table = table_shape().table
        table.columns.add_column()

    def op_replace_xml():
        shape = text_shape()
        element = shape.element
        keep("element", element)
        xml = element.xml
        if "foo" not in xml:
            raise LookupError("no foo")
        element.replace_xml(xml.replace("foo", "baz", 1))

    def op_replace_xml_name():
        shape = rand_shape()
        element = shape.element
        xml = element.xml
        marker = f'name="{shape.name}"'
        if marker not in xml:
            raise LookupError("no name")
        element.replace_xml(xml.replace(marker, f'name="N{rng.randrange(1000)}"', 1))

    def op_notes():
        slide = rand_slide()
        slide.notes_slide.notes_text_frame.text = f"n{rng.randrange(100)}"

    def op_layout_duplicate():
        keep("layout", layouts.duplicate(layouts[rng.randrange(len(layouts))]))

    def op_layout_remove():
        if len(layouts) <= 11:
            raise LookupError("keep template layouts")
        layouts.remove(layouts[len(layouts) - 1])

    def op_replace_text():
        prs.replace_text("foo", f"q{rng.randrange(10)}")

    def op_hyperlink():
        frame = text_shape().text_frame
        paragraph = frame.paragraphs[0]
        if len(paragraph.runs) == 0:
            raise LookupError("no run")
        paragraph.runs[0].hyperlink.address = rng.choice([None, "https://example.com/a"])

    def op_shape_name():
        rand_shape().name = f"name{rng.randrange(100)}"

    def op_slide_layout():
        rand_slide().slide_layout = layouts[rng.choice([1, 5, 6])]

    def op_shape_text():
        text_shape().text = f"st{rng.randrange(100)}"

    def op_try_replace():
        rand_slide().try_replace_text("foo", "zz")

    def op_add_picture():
        keep("shape", rand_slide().shapes.add_picture(io.BytesIO(png(rng.randrange(50))), 0, 0, 100, 100))

    def pictures():
        return [shape for slide in prs.slides for shape in slide.shapes if shape.shape_type == 13]

    def op_remove_picture():
        holders = [slide for slide in prs.slides if any(shape.shape_type == 13 for shape in slide.shapes)]
        if not holders:
            raise LookupError("no picture")
        slide = holders[rng.randrange(len(holders))]
        found = [shape for shape in slide.shapes if shape.shape_type == 13]
        slide.shapes.remove(found[rng.randrange(len(found))])

    def op_replace_image():
        found = pictures()
        if not found:
            raise LookupError("no picture")
        found[rng.randrange(len(found))].replace_image(io.BytesIO(png(100 + rng.randrange(50))))

    def op_picture_then_hyperlink():
        slide = rand_slide()
        keep("shape", slide.shapes.add_picture(io.BytesIO(png(rng.randrange(50, 90))), 0, 0, 100, 100))
        shape = slide.shapes.add_textbox(0, 0, 100, 100)
        shape.text_frame.text = "link me"
        shape.text_frame.paragraphs[0].runs[0].hyperlink.address = "https://example.com/p"
        shape.text_frame.paragraphs[0].runs[0].hyperlink.address = None

    def op_import_other():
        keep("slide", prs.slides.import_slide(other.slides[rng.randrange(3)], index=rng.randrange(len(prs.slides) + 1)))

    def op_import_self():
        keep("slide", prs.slides.import_slide(rand_slide(), index=rng.randrange(len(prs.slides) + 1)))

    def op_import_many():
        slides, _ = prs.slides.import_slides(other, [0, 2], index=rng.randrange(len(prs.slides) + 1))
        for slide in slides:
            keep("slide", slide)

    def op_header_footer():
        footer = rand_slide().header_footer
        footer.footer = rng.choice([None, "foot"])

    def op_slide_number():
        rand_slide().header_footer.slide_number = rng.choice([True, False])

    def op_merge_split():
        table = table_shape().table
        cell = table.cell(0, 0)
        if cell.is_merge_origin:
            cell.split()
        else:
            cell.merge(table.cell(1, 1))

    def op_layout_dup_remove():
        dup = layouts.duplicate(layouts[rng.randrange(len(layouts))])
        keep("layout", dup)
        if rng.random() < 0.6:
            layouts.remove(dup)

    def op_text_marker():
        shape = text_shape()
        shape.text_frame.text = "marker foo"
        element = shape.element
        keep("element", element)
        element.replace_xml(element.xml.replace("marker foo", "marker bar", 1))

    def op_keep_handles():
        slide = rand_slide()
        keep("slide", slide)
        for shape in list(slide.shapes)[:4]:
            keep("shape", shape)
            if shape.has_text_frame:
                frame = shape.text_frame
                keep("frame", frame)
                for paragraph in list(frame.paragraphs)[:2]:
                    keep("para", paragraph)
                    for run in list(paragraph.runs)[:2]:
                        keep("run", run)

    ops = [value for name, value in sorted(locals().items()) if name.startswith("op_")]
    log = []
    for step in range(STEPS):
        op = rng.choice(ops)
        try:
            op()
            outcome = "ok"
        except BaseException as exc:  # noqa: BLE001
            if isinstance(exc, KeyboardInterrupt):
                raise
            outcome = err(exc)
        try:
            digest = hashlib.sha256(prs.to_bytes()).hexdigest()[:16]
        except BaseException as exc:  # noqa: BLE001
            digest = err(exc)
        probes = [probe(kind, handle) for kind, handle in pool]
        log.append({"step": step, "op": op.__name__, "outcome": outcome, "bytes": digest, "probes": probes})
    with open(OUT, "w") as handle:
        json.dump(log, handle, default=str)


main()
