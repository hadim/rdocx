"""First and second insert_paragraph / remove_content after open, on the skills documents of 25, 100 and 1,000 paragraphs."""
import sys, time, pathlib
import rdocx
folder = pathlib.Path(sys.argv[1])
for n in (25, 100, 1000):
    path = str(folder / f"open{n}.docx")
    rows = []
    for name, edit in (("insert_paragraph", lambda d: d.insert_paragraph(10, "x")), ("remove_content", lambda d: d.remove_content(10))):
        firsts, seconds = [], []
        for _ in range(3):
            doc = rdocx.Document.open(path)
            t = time.perf_counter(); edit(doc); firsts.append(time.perf_counter() - t)
            t = time.perf_counter(); edit(doc); seconds.append(time.perf_counter() - t)
        rows.append(f"{name} first {min(firsts)*1000:7.2f} ms, second {min(seconds)*1000:6.2f} ms")
    print(f"{n:5d} paragraphs: " + " | ".join(rows))
