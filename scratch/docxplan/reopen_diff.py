"""Do untouched body items keep their XML through edits that commit by staging and reopening?"""
import os, sys, tempfile
S = "/private/tmp/claude-501/-Users-hadim-Code-libs-rdocx/08057ee4-afc5-4751-aca1-2668752380de/scratchpad"
sys.path.insert(0, f"{S}/skills-main/tests")
import rdocx
import test_docx_scaling as t
from builders import edit_cost_docx

path = str(edit_cost_docx(os.path.join(tempfile.mkdtemp(), "d.docx"), 25, picture_side=100))


def body(doc):
    return [it.xml for it in doc.story_items if it.story.kind == "body" and len(it.index_path) == 1]


for name, act in [
    ("set_header", lambda d: d.set_header("x")),
    ("add_footnote", lambda d: d.add_footnote(t.para(d, 0), "note")),
    ("Cell.text", lambda d: setattr(t.cell(d, 0), "text", "x")),
    ("Table.add_row", lambda d: t.table(d, 0).add_row()),
    ("insert_section", lambda d: d.insert_section(len(d.sections))),
]:
    doc = rdocx.Document.open(path)
    before = body(doc)
    act(doc)
    after = body(doc)
    same = sum(1 for a, b in zip(before, after) if a == b)
    print(f"{name:16s} items before {len(before)} after {len(after)}; equal at same index {same}; "
          f"after in before: {sum(1 for x in after if x in set(before))}")
