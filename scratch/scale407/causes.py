"""For each GROWING call of the skills scaling test, the binding's whole-body work in one call on the N document:
(inventory rebuilds, whole-source reads), from a debug build."""
import sys, tempfile, pathlib, json
sys.path.insert(0, sys.argv[1])
import test_docx_scaling as t
from builders import edit_cost_docx
from rdocx import _rdocx

work = _rdocx._word_whole_body_work
folder = pathlib.Path(tempfile.mkdtemp())
path = edit_cost_docx(folder / "n.docx", t.N, picture_side=100)
table = {**t.TIMED, **t.READS_TIMED}
out = {}
for name in sorted(t.GROWING):
    try:
        ready = table[name](t.rdocx.Document.open(str(path)))
        action = ready(0)
        before = work()
        action()
        after = work()
        out[name] = [after[0] - before[0], after[1] - before[1]]
    except BaseException as error:
        out[name] = ["err", f"{type(error).__name__}: {str(error)[:200]}"]
json.dump(out, sys.stdout, indent=0)
