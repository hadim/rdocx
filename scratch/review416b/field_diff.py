"""Differential check: field ids chosen with the carried per-part cache
(deck A, read from bytes, snapshot kept across commits), on an in-memory
deck (B, no snapshot at first), and on a deck reloaded from bytes after
every operation (C, fresh scan each time) must give the same bytes."""
import io, random, re, sys, zipfile
import rpptx

def base_bytes():
    p = rpptx.Presentation()
    for _ in range(4):
        s = p.slides.add_slide(p.slide_layouts[1])
        s.shapes.add_textbox(0, 0, 100, 100).text_frame.text = "a"
        s.shapes.add_textbox(0, 0, 100, 100).text_frame.text = "b"
    return p.to_bytes()

def textboxes(slide):
    return [s for s in slide.shapes if s.has_text_frame and s.text_frame.text in ("a", "b") or (s.has_text_frame and s.text_frame.text.startswith("x"))]

def ops(rng, n):
    out = []
    for _ in range(n):
        out.append((rng.choice(["field", "hf_num", "hf_footer", "hf_datefmt", "hf_all", "add", "inplace", "remove", "notes", "dup"]), rng.random(), rng.random()))
    return out

def apply(p, op):
    kind, r1, r2 = op
    n = len(p.slides)
    i = int(r1 * n)
    s = p.slides[i]
    if kind == "field":
        boxes = textboxes(s)
        if boxes:
            boxes[int(r2 * len(boxes))].text_frame.paragraphs[0].add_field("slidenum")
    elif kind == "hf_num":
        s.header_footer.slide_number = r2 < 0.7
    elif kind == "hf_footer":
        s.header_footer.footer = "F" if r2 < 0.7 else None
    elif kind == "hf_datefmt":
        s.header_footer.date_format = "datetime%d" % (1 + int(r2 * 13))
    elif kind == "hf_all":
        p.set_header_footer(slide_number=True, footer="All", date=None, hide_on_title=r2 < 0.5)
    elif kind == "add":
        p.slides.add_slide(p.slide_layouts[1 if r2 < 0.5 else 0])
    elif kind == "inplace":
        boxes = textboxes(s)
        if boxes:
            boxes[0].text_frame.text = "x%d" % int(r2 * 100)
    elif kind == "remove":
        if n > 2:
            p.slides.remove(s)
    elif kind == "notes":
        s.notes_slide.notes_text_frame.text = "n%d" % int(r2 * 100)
    elif kind == "dup":
        p.slides.duplicate(s)

def field_ids(data):
    z = zipfile.ZipFile(io.BytesIO(data))
    ids = []
    for name in sorted(z.namelist()):
        if name.endswith(".xml"):
            ids += [(name, m) for m in re.findall(r'<a:fld[^>]*\bid="([^"]+)"', z.read(name).decode("utf-8", "replace"))]
    return ids

def run(seed, steps):
    rng = random.Random(seed)
    data = base_bytes()
    seq = ops(rng, steps)
    a = rpptx.Presentation.from_bytes(data)
    b = rpptx.Presentation()
    for _ in range(4):
        sl = b.slides.add_slide(b.slide_layouts[1])
        sl.shapes.add_textbox(0, 0, 100, 100).text_frame.text = "a"
        sl.shapes.add_textbox(0, 0, 100, 100).text_frame.text = "b"
    c_bytes = data
    for k, op in enumerate(seq):
        errs = []
        for deck in (a, b):
            try:
                apply(deck, op); errs.append(None)
            except Exception as e:
                errs.append(f"{type(e).__name__}: {e}")
        c = rpptx.Presentation.from_bytes(c_bytes)
        try:
            apply(c, op); errs.append(None)
        except Exception as e:
            errs.append(f"{type(e).__name__}: {e}")
        c_bytes = c.to_bytes()
        ab, bb = a.to_bytes(), b.to_bytes()
        if len(set(errs)) != 1:
            return f"seed {seed} step {k} {op[0]}: errors differ {errs}"
        if ab != c_bytes or bb != c_bytes:
            fa, fb, fc = field_ids(ab), field_ids(bb), field_ids(c_bytes)
            return f"seed {seed} step {k} {op[0]}: bytes differ A==C {ab == c_bytes} B==C {bb == c_bytes} fieldsA==C {fa == fc} fieldsB==C {fb == fc}"
        ids = [i for _, i in field_ids(ab)]
    return None

if __name__ == "__main__":
    first, count, steps = int(sys.argv[1]), int(sys.argv[2]), int(sys.argv[3])
    bad = 0
    total_fields = 0
    for seed in range(first, first + count):
        r = run(seed, steps)
        if r:
            bad += 1
            print(r)
    print("done", count, "seeds,", bad, "mismatches")
