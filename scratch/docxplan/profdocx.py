"""Profile one call of the rdocx-skills scaling test in a loop:
python profdocx.py NAME PARAGRAPHS SECONDS OUT  (OUT '-' times only)

Builds edit_cost_docx(PARAGRAPHS, picture_side=100) as the test does, then repeats: open the document, make the
test's targets k = 0, 1, 2 and run the timed action on each, until the sampler exits. Prints the best time of the
timed action over the first opens."""
import os
import subprocess
import sys
import tempfile
import time

S = "/private/tmp/claude-501/-Users-hadim-Code-libs-rdocx/08057ee4-afc5-4751-aca1-2668752380de/scratchpad"
sys.path.insert(0, f"{S}/skills-main/tests")
import rdocx  # noqa: E402
import test_docx_scaling as t  # noqa: E402
from builders import edit_cost_docx  # noqa: E402


def rounds(path, setup):
    doc = rdocx.Document.open(path)
    ready = setup(doc)
    times = []
    for k in range(t.CALLS):
        action = ready(k)
        start = time.perf_counter()
        action()
        times.append(time.perf_counter() - start)
    return times


def main():
    name, count, seconds, out = sys.argv[1], int(sys.argv[2]), float(sys.argv[3]), sys.argv[4]
    setup = {**t.TIMED, **t.READS_TIMED}[name]
    folder = tempfile.mkdtemp()
    path = str(edit_cost_docx(os.path.join(folder, "d.docx"), count, picture_side=100))
    best = min(min(rounds(path, setup)) for _ in range(2))
    print(f"{name} @ {count}: best {best * 1000:.1f} ms, docx {os.path.getsize(path) / 1e3:.0f} kB", flush=True)
    if out == "-":
        return
    sampler = subprocess.Popen(["sample", str(os.getpid()), str(int(seconds)), "1", "-mayDie", "-file", out],
                               stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    n = 0
    while sampler.poll() is None:
        n += len(rounds(path, setup))
    print(f"{n} calls sampled into {out}", flush=True)


main()
