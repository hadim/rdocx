"""Probes for the PR2 review. argv[1] = 'head' or 'base'."""
import hashlib
import io
import sys
import zipfile

import rpptx


def deck_bytes(slides=3):
    deck = rpptx.Presentation()
    for index in range(slides):
        slide = deck.slides.add_slide(deck.slide_layouts[6])
        slide.shapes.add_textbox(0, 0, 3000000, 1000000).text_frame.text = f"text {index} " * 20
        slide.notes_slide.notes_text_frame.text = "notes"
    return deck.to_bytes()


def rezip(data, edit):
    source = zipfile.ZipFile(io.BytesIO(data))
    entries = {info.filename: source.read(info) for info in source.infolist()}
    edit(entries)
    out = io.BytesIO()
    with zipfile.ZipFile(out, "w", zipfile.ZIP_DEFLATED) as target:
        for name, payload in entries.items():
            target.writestr(name, payload)
    return out.getvalue()


def parts(data):
    archive = zipfile.ZipFile(io.BytesIO(data))
    return {info.filename: archive.read(info) for info in archive.infolist()}


def attempt(label, run):
    try:
        print(label, "->", run())
    except BaseException as error:  # PanicException derives from BaseException
        print(label, "->", type(error).__name__, str(error).splitlines()[0][:160])


