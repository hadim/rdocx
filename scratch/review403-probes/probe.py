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


base = deck_bytes()

# A. Source with a stray root ".rels" entry.
stray = rezip(base, lambda entries: entries.__setitem__(".rels", entries["_rels/.rels"]))


def stray_commit():
    deck = rpptx.Presentation.from_bytes(stray)
    deck.core_properties.title = "edited"
    return "commit ok"


def stray_save():
    deck = rpptx.Presentation.from_bytes(stray)
    deck.core_properties.title = "edited"
    return len(deck.to_bytes())


attempt("A root .rels: commit", stray_commit)
attempt("A root .rels: commit then save", stray_save)

# B. Source slide bytes that differ from the rpptx serialization.
def pad(entries):
    for name in list(entries):
        if name.startswith("ppt/slides/slide") or name.startswith("ppt/notesSlides/notesSlide"):
            entries[name] = entries[name].replace(b"?>", b"?>\n<!-- producer -->", 1)


padded = rezip(base, pad)


def labelled():
    deck = rpptx.Presentation.from_bytes(padded)
    deck.core_properties.title = "edited"
    out = parts(deck.to_bytes())
    src = parts(padded)
    changed = sorted(name for name in out if src.get(name) != out[name])
    return changed


attempt("B labelled change, parts differing from source after core title", labelled)


def labelled_digest():
    deck = rpptx.Presentation.from_bytes(padded)
    deck.core_properties.title = "edited"
    return {name: hashlib.sha256(data).hexdigest()[:12] for name, data in sorted(parts(deck.to_bytes()).items())}


import json
with open(sys.argv[2], "w") as handle:
    json.dump(labelled_digest(), handle, indent=0, sort_keys=True)

# C. fit_text on slide 1 while slide 3 has no layout relationship.
def drop_layout(entries):
    name = "ppt/slides/_rels/slide3.xml.rels"
    xml = entries[name].decode()
    import re
    xml = re.sub(r'<Relationship [^>]*slideLayout[^>]*/>', "", xml)
    entries[name] = xml.encode()


broken = rezip(base, drop_layout)


def fit_other_slide():
    deck = rpptx.Presentation.from_bytes(broken)
    frame = deck.slides[0].shapes[0].text_frame
    return frame.fit_text(max_size=18)


attempt("C fit_text slide 1, slide 3 without layout", fit_other_slide)
