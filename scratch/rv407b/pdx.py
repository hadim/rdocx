"""Real python-docx edit_cost_docx document: the scaling test's edits, then save; print hashes for main/branch."""
import hashlib, sys, os
sys.path.insert(0, "/private/tmp/claude-501/-Users-hadim-Code-libs-rdocx/08057ee4-afc5-4751-aca1-2668752380de/scratchpad/skills-main/tests")
import builders
import rdocx

path = os.path.join(os.path.dirname(os.path.abspath(__file__)), "pdx.docx")
if not os.path.exists(path):
    builders.edit_cost_docx(path, 40, probes=2)
data = open(path, "rb").read()


def h(b):
    return hashlib.sha1(b).hexdigest()[:12]


def run(label, fn):
    doc = rdocx.Document.from_bytes(data)
    try:
        r = fn(doc)
        out = doc.to_bytes()
        again = rdocx.Document.from_bytes(out)
        print(f"{label:28s} ok {str(r)[:20]:20s} save={h(out)} texts={h('|'.join(p.text for p in again.paragraphs).encode())}")
    except Exception as e:
        print(f"{label:28s} ERR {type(e).__name__}: {str(e)[:120]}")


def idx(doc, text):
    for p in doc.paragraphs:
        if p.text == text:
            return doc.find_content_index(p)
    raise LookupError(text)


run("remove spot 0", lambda d: d.remove_content(idx(d, "spot 0")))
run("remove after the rows", lambda d: d.remove_content(idx(d, "after the rows")))
run("remove end", lambda d: d.remove_content(idx(d, "end")))
run("insert before rows", lambda d: d.insert_paragraph(idx(d, "after the rows") - 1, "x").text)
run("insert before spot 1", lambda d: d.insert_paragraph(idx(d, "spot 1"), "x").text)
run("range spot1..", lambda d: d.remove_content_range(idx(d, "spot 1"), idx(d, "spot 1") + 1))
run("replace end", lambda d: d.paragraphs[-1].replace_xml(d.paragraphs[-1].xml.replace(">end<", ">fin<")))
run("set text spot 0", lambda d: setattr([p for p in d.paragraphs if p.text == "spot 0"][0], "text", "s0"))
run("insert table before rows", lambda d: len(d.insert_table(idx(d, "after the rows") - 1, 2, 2).rows))
run("remove the second section p", lambda d: d.remove_content(idx(d, "the second section")))
