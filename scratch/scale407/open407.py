"""Document.open cost (best of 7) on the skills scaling documents, N, 4N and 1,000 paragraphs."""
import sys, tempfile, pathlib, time
sys.path.insert(0, sys.argv[1])
import rdocx
from builders import edit_cost_docx
folder = pathlib.Path(sys.argv[2])
folder.mkdir(exist_ok=True)
for n in (25, 100, 1000):
    path = folder / f"open{n}.docx"
    if not path.exists():
        edit_cost_docx(path, n, picture_side=100)
    best = min((lambda t0: (rdocx.Document.open(str(path)), time.perf_counter() - t0)[1])(time.perf_counter()) for _ in range(7))
    print(f"open {n:5d} paragraphs: {best * 1000:8.2f} ms")
