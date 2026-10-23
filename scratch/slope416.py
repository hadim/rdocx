"""Slope of the nine transition and header-footer setters: python slope416.py SLIDES [CALL --sample OUT SECONDS].

Builds the prof403 deck (25 text boxes, 12 rectangles, a 4x4 table, a ~1 MB picture and notes a slide), opens it
with rpptx, and times each setter on fresh new slides (best of 3), as the rdocx-skills scaling test does."""
import os
import subprocess
import sys
import tempfile
import time

sys.argv_saved = list(sys.argv)
sys.path.insert(0, os.path.dirname(__file__))
import importlib.util
spec = importlib.util.spec_from_file_location("prof", os.path.join(os.path.dirname(__file__), "prof403.py"))
source = open(spec.origin).read().replace("\nmain()\n", "\n")
prof = {}
exec(compile(source, spec.origin, "exec"), prof)
import rpptx


def transition(kind=None):
    def make(prs):
        t = prs.slides.add_slide(prs.slide_layouts[6]).transition
        if kind:
            t.type = kind
        return t
    return make


def header_footer(auto=False):
    def make(prs):
        h = prs.slides.add_slide(prs.slide_layouts[6]).header_footer
        if auto:
            h.date = "auto"
        return h
    return make


CALLS = {
    "SlideTransition.type": (transition(), "type", "fade"),
    "SlideTransition.direction": (transition("push"), "direction", "up"),
    "SlideTransition.duration": (transition("fade"), "duration", 1.5),
    "SlideTransition.advance_on_click": (transition(), "advance_on_click", False),
    "SlideTransition.advance_after": (transition(), "advance_after", 2.0),
    "HeaderFooter.slide_number": (header_footer(), "slide_number", True),
    "HeaderFooter.footer": (header_footer(), "footer", "footer"),
    "HeaderFooter.date": (header_footer(), "date", "1 October 2026"),
    "HeaderFooter.date_format": (header_footer(True), "date_format", "datetime2"),
}


def main():
    slides = int(sys.argv[1])
    only = sys.argv[2] if len(sys.argv) > 2 else None
    out = sys.argv[sys.argv.index("--sample") + 1] if "--sample" in sys.argv else None
    with tempfile.TemporaryDirectory() as folder:
        path = os.path.join(folder, "deck.pptx")
        prof["build"](path, slides)
        for name, (make, attr, value) in CALLS.items():
            if only and name != only:
                continue
            prs = rpptx.Presentation(path)
            best = []
            for _ in range(3):
                target = make(prs)
                start = time.perf_counter()
                setattr(target, attr, value)
                best.append(time.perf_counter() - start)
            print(f"{name} @ {slides} slides: {min(best) * 1000:.1f} ms")
            if out:
                seconds = int(sys.argv[-1])
                targets = [make(prs) for _ in range(400)]
                sampler = subprocess.Popen(["sample", str(os.getpid()), str(seconds), "1", "-mayDie", "-file", out],
                                           stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
                i = 0
                while sampler.poll() is None and i < len(targets):
                    setattr(targets[i], attr, value)
                    i += 1
                sampler.wait()
                print(f"{i} calls sampled")


main()
