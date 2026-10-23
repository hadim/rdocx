import hashlib, io, json, sys, zipfile
import rpptx
sys.path.insert(0, sys.argv[3])
from probe_helpers import deck_bytes, rezip, parts

def spaced(entries):
    for name in list(entries):
        if name.startswith(("ppt/slides/slide", "ppt/notesSlides/notesSlide")):
            entries[name] = entries[name].replace(b"><", b">\n  <")

source = rezip(deck_bytes(), spaced)
results = {}
for label, edit in [
    ("core title", lambda d: setattr(d.core_properties, "title", "edited")),
    ("remove slide 2", lambda d: d.slides.remove(d.slides[1])),
    ("notes slide 1", lambda d: setattr(d.slides[0].notes_slide.notes_text_frame, "text", "edited")),
]:
    deck = rpptx.Presentation.from_bytes(source)
    edit(deck)
    out = parts(deck.to_bytes())
    results[label] = {name: hashlib.sha256(data).hexdigest()[:12] for name, data in sorted(out.items())}
    src = parts(source)
    print(label, "changed vs source:", sorted(n for n in out if src.get(n) != out[n]))
json.dump(results, open(sys.argv[2], "w"), indent=0, sort_keys=True)
