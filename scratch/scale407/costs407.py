"""N vs 4N costs of named calls of the skills scaling test, plus Run, Table and Cell .xml, with its bound."""
import sys, tempfile, pathlib
sys.path.insert(0, sys.argv[1])
import test_docx_scaling as t
from builders import edit_cost_docx

extra = {"Run.xml": t.on_made(t.run, lambda doc, r: r.xml),
         "Table.xml": t.on_made(t.table, lambda doc, x: x.xml),
         "Cell.xml": t.on_made(t.cell, lambda doc, c: c.xml)}
folder = pathlib.Path(tempfile.mkdtemp())
docs = (edit_cost_docx(folder / "n.docx", t.N, picture_side=100),
        edit_cost_docx(folder / "4n.docx", 4 * t.N, picture_side=100))
table = {**t.TIMED, **t.READS_TIMED, **extra}
for name in sys.argv[2:]:
    small, large = t.costs_ms(docs, table[name])
    bound = t.FIXED * small + t.FLOOR_MS
    print(f"{name:40s} N={small:8.2f} ms  4N={large:8.2f} ms  bound={bound:8.2f}  {'ok' if large < bound else 'GROWS'}")
