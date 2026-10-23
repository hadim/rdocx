"""The commit-work guard with remove of slide 1 and of the last slide, split by counter."""
import io, struct, zlib
import rpptx
from rpptx import _rpptx

def _tiny_png(r, g, b):
    def chunk(kind, data):
        return struct.pack(">I", len(data)) + kind + data + struct.pack(">I", zlib.crc32(kind + data) & 0xFFFFFFFF)
    raw = b"\x00" + bytes((r % 256, g % 256, b % 256))
    return b"\x89PNG\r\n\x1a\n" + chunk(b"IHDR", struct.pack(">IIBBBBB", 1, 1, 8, 2, 0, 0, 0)) + chunk(b"IDAT", zlib.compress(raw)) + chunk(b"IEND", b"")

commit_work, inventory_work = _rpptx._commit_work, _rpptx._inventory_work

def work(slides, which):
    built = rpptx.Presentation()
    for index in range(slides):
        slide = built.slides.add_slide(built.slide_layouts[6])
        for _ in range(4):
            slide.shapes.add_textbox(0, 0, 100, 100).text_frame.text = "before"
        slide.shapes.add_table(2, 2, 0, 0, 900, 900)
        slide.shapes.add_picture(io.BytesIO(_tiny_png(index, 0, 0)), 0, 0, 100, 100)
        slide.notes_slide.notes_text_frame.text = "notes"
    presentation = rpptx.Presentation.from_bytes(built.to_bytes())
    commit_work(); inventory_work()
    presentation.slides.remove(presentation.slides[which])
    return commit_work(), inventory_work()

for which in (1, -1):
    print("remove", which, "N=3", work(3, which), "N=12", work(12, which))
