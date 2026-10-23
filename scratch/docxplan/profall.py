"""Short profile of every call listed in a file, one document built once:
python profall.py CALLS_FILE PARAGRAPHS SECONDS OUTDIR

Takes the timing lock around each sampled call only (and waits while rustc runs)."""
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

LOCK = f"{S}/timing.lock"


def rounds(path, setup):
    doc = rdocx.Document.open(path)
    ready = setup(doc)
    for k in range(t.CALLS):
        ready(k)()
    return t.CALLS


def locked():
    while True:
        try:
            os.mkdir(LOCK)
            break
        except FileExistsError:
            time.sleep(1)
    while subprocess.run(["pgrep", "-x", "rustc"], capture_output=True).returncode == 0:
        time.sleep(1)


def main():
    names = [line.split()[0] for line in open(sys.argv[1]) if line.strip()]
    count, seconds, outdir = int(sys.argv[2]), int(sys.argv[3]), sys.argv[4]
    os.makedirs(outdir, exist_ok=True)
    path = str(edit_cost_docx(os.path.join(tempfile.mkdtemp(), "d.docx"), count, picture_side=100))
    table = {**t.TIMED, **t.READS_TIMED}
    for name in names:
        out = os.path.join(outdir, f"{name}.{count}.txt")
        if os.path.exists(out):
            continue
        setup = table[name]
        rounds(path, setup)  # warm, untimed
        locked()
        try:
            sampler = subprocess.Popen(["sample", str(os.getpid()), str(seconds), "1", "-mayDie", "-file", out],
                                       stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
            n = 0
            while sampler.poll() is None:
                n += rounds(path, setup)
        finally:
            os.rmdir(LOCK)
        print(f"{name}: {n} calls", flush=True)


main()
