"""Targeted probes of the relaxed plain_local_edit guard: plain edits next to nested namespace owners.

usage: python guard407.py > log.json   (run under main and branch builds, compare with cmp407.py)
"""
import hashlib
import io
import json
import sys
import zipfile

import rdocx

W = "http://schemas.openxmlformats.org/wordprocessingml/2006/main"
W14 = "http://schemas.microsoft.com/office/word/2010/wordml"
W15 = "http://schemas.microsoft.com/office/word/2012/wordml"
MC = "http://schemas.openxmlformats.org/markup-compatibility/2006"
WORD_ROOT = (
    f' xmlns:w14="{W14}" xmlns:w15="{W15}"'
    ' xmlns:wp14="http://schemas.microsoft.com/office/word/2010/wordprocessingDrawing"'
    ' mc:Ignorable="w14 w15 wp14"'
)
WD = f'xmlns:w="{W}"'


def P(text):
    return f'<w:p><w:r><w:t>{text}</w:t></w:r></w:p>'


def around(owner, tag):
    """Plain paragraphs on both sides of `owner`."""
    return P(f"{tag}-b2") + P(f"{tag}-b1") + owner + P(f"{tag}-a1") + P(f"{tag}-a2")


SDT_W15 = (f'<w:sdt {WD} xmlns:w15="{W15}"><w:sdtPr><w:tag w:val="rows"/><w15:repeatingSection/></w:sdtPr><w:sdtContent>'
           '<w:sdt><w:sdtPr><w15:repeatingSectionItem/></w:sdtPr><w:sdtContent>' + P("row") +
           '</w:sdtContent></w:sdt></w:sdtContent></w:sdt>')
SHAPES = {
    # python-docx review tail
    "pd_review": ("".join([
        P("x0"),
        f'<w:p><w:r><w:t xml:space="preserve">tracked </w:t></w:r><w:ins {WD} w:id="901" w:author="E" '
        'w:date="2026-01-01T00:00:00Z"><w:r><w:t>inserted</w:t></w:r></w:ins></w:p>',
        P("x1"),
        f'<w:sdt {WD}><w:sdtPr><w:tag w:val="probe0"/></w:sdtPr><w:sdtContent>' + P("control") + '</w:sdtContent></w:sdt>',
        P("s-b1"), SDT_W15, P("s-a1"), P("s-a2")]), ""),
    "pd_review_wordroot": (around(SDT_W15, "s"), WORD_ROOT),
    "sdt_q_alias": (around(f'<q:sdt xmlns:q="{W}"><q:sdtPr><q:tag q:val="t"/></q:sdtPr><q:sdtContent>'
                           '<q:p><q:r><q:t>in q</q:t></q:r></q:p></q:sdtContent></q:sdt>', "s"), ""),
    "sdt_w14_checkbox": (around(f'<w:sdt xmlns:w14="{W14}"><w:sdtPr><w14:checkbox><w14:checked w14:val="0"/>'
                                '</w14:checkbox></w:sdtPr><w:sdtContent><w:p w14:paraId="0000AAAA"><w:r><w:t>box</w:t>'
                                '</w:r></w:p></w:sdtContent></w:sdt>', "s"), ""),
    "sdt_foreign_attr": (around('<w:sdt xmlns:x="urn:x" x:foo="1"><w:sdtPr><w:tag w:val="t"/></w:sdtPr><w:sdtContent>'
                                + P("fa") + '</w:sdtContent></w:sdt>', "s"), ""),
    "sdt_foreign_child": (around('<w:sdt xmlns:x="urn:x"><w:sdtPr><w:tag w:val="t"/><x:meta x:v="1"/></w:sdtPr>'
                                 '<w:sdtContent>' + P("fc") + '</w:sdtContent></w:sdt>', "s"), ""),
    "sdt_shadow_w15": (around('<w:sdt xmlns:w15="urn:other"><w:sdtPr><w15:x/></w:sdtPr><w:sdtContent>' + P("sh")
                              + '</w:sdtContent></w:sdt>', "s"), ""),
    "sdt_mc_other": (around('<w:sdt xmlns:mc="urn:notmc"><w:sdtPr><mc:x/></w:sdtPr><w:sdtContent>' + P("mo")
                            + '</w:sdtContent></w:sdt>', "s"), ""),
    "twin_sdt_same": (around('<w:sdt xmlns:x="urn:x"><w:sdtPr><x:meta/></w:sdtPr><w:sdtContent>' + P("tw")
                             + '</w:sdtContent></w:sdt>', "s") + P("mid")
                      + '<w:sdt xmlns:x="urn:x"><w:sdtPr><x:meta/></w:sdtPr><w:sdtContent>' + P("tw")
                      + '</w:sdtContent></w:sdt>' + P("t-a1"), ""),
    "twin_sdt_alias": (around('<w:sdt xmlns:x="urn:x"><w:sdtPr><x:meta/></w:sdtPr><w:sdtContent>' + P("tw")
                              + '</w:sdtContent></w:sdt>', "s") + P("mid")
                       + '<w:sdt xmlns:y="urn:x"><w:sdtPr><y:meta/></w:sdtPr><w:sdtContent>' + P("tw")
                       + '</w:sdtContent></w:sdt>' + P("t-a1"), ""),
    "tbl_owner_w14": (around(f'<w:tbl xmlns:w14="{W14}"><w:tblGrid><w:gridCol w:w="2000"/></w:tblGrid>'
                             '<w:tr w14:paraId="00000001"><w:tc><w:p><w:r><w:t>cell</w:t></w:r></w:p></w:tc></w:tr></w:tbl>',
                             "s"), ""),
    "tc_owner": (around('<w:tbl><w:tblGrid><w:gridCol w:w="2000"/></w:tblGrid><w:tr>'
                        '<w:tc xmlns:x="urn:x"><w:tcPr><x:e/></w:tcPr><w:p><w:r><w:t>cell</w:t></w:r></w:p></w:tc>'
                        '</w:tr></w:tbl>', "s"), ""),
    "hyperlink_owner": (around('<w:p><w:hyperlink xmlns:x="urn:x" x:a="1" w:anchor="b"><w:r><w:t>link</w:t></w:r>'
                               '</w:hyperlink></w:p>', "s"), ""),
    "customxml_w": (around(f'<w:customXml {WD} w:element="blk">' + P("cx") + '</w:customXml>', "s"), ""),
    "customxml_x": (around('<w:customXml xmlns:x="urn:x" w:element="blk"><w:customXmlPr><w:attr w:name="a" w:val="1"/>'
                           '</w:customXmlPr>' + P("cx") + '</w:customXml>', "s"), ""),
    "p_owner_in_cell": (around('<w:tbl><w:tblGrid><w:gridCol w:w="2000"/></w:tblGrid><w:tr><w:tc>'
                               f'<w:p xmlns:w14="{W14}" w14:paraId="0000BBBB"><w:r><w:t>cellp</w:t></w:r></w:p>'
                               '</w:tc></w:tr></w:tbl>', "s"), ""),
    "p_owner_body_alias": (around(f'<q:p xmlns:q="{W}"><q:r><q:t>alias p</q:t></q:r></q:p>', "s"), ""),
    "nested_sdt_inner_decl": (around('<w:sdt><w:sdtPr><w:tag w:val="o"/></w:sdtPr><w:sdtContent>'
                                     '<w:sdt xmlns:x="urn:x"><w:sdtPr><x:m/></w:sdtPr><w:sdtContent>' + P("deep")
                                     + '</w:sdtContent></w:sdt>' + P("outer") + '</w:sdtContent></w:sdt>', "s"), ""),
    "two_repeating": (around(SDT_W15, "s") + P("mid") + SDT_W15 + P("t-a1"), ""),
    "sdt_w15_and_plain_decl_para": (around(SDT_W15, "s") + f'<w:p xmlns:x="urn:x"><w:r><w:t>unused decl</w:t></w:r></w:p>'
                                    + P("t-a1"), ""),
}


def build(body, root):
    doc = rdocx.Document()
    doc.add_paragraph("first")
    z = zipfile.ZipFile(io.BytesIO(doc.to_bytes()))
    out = io.BytesIO()
    with zipfile.ZipFile(out, "w") as archive:
        for info in z.infolist():
            data = z.read(info.filename)
            if info.filename == "word/document.xml":
                text = data.decode()
                at = text.index("<w:sectPr")
                text = text[:at] + body + P("last") + text[at:]
                if root:
                    if "xmlns:mc=" not in text.split(">", 2)[1]:
                        root = f' xmlns:mc="{MC}"' + root
                    text = text.replace("<w:document ", "<w:document" + root + " ", 1)
                data = text.encode()
            archive.writestr(info, data)
    return rdocx.Document.from_bytes(out.getvalue())


def outcome(fn):
    try:
        value = fn()
        if isinstance(value, (bytes, bytearray)):
            return ["ok", hashlib.sha1(value).hexdigest()]
        if isinstance(value, (str, int, bool, type(None))):
            return ["ok", value]
        return ["ok", type(value).__name__]
    except BaseException as error:
        if isinstance(error, KeyboardInterrupt):
            raise
        return ["err", type(error).__name__, str(error)[:300]]


def state(doc):
    log = {}
    log["texts"] = outcome(lambda: "|".join(p.text for p in doc.paragraphs))
    log["pxml"] = [outcome(lambda p=p: p.xml) for p in doc.paragraphs] if outcome(lambda: len(doc.paragraphs))[0] == "ok" else []
    log["txml"] = [outcome(lambda t=t: t.xml) for t in doc.tables]
    data = outcome(doc.to_bytes)
    log["bytes"] = data
    if data[0] == "ok":
        def reopen():
            again = rdocx.Document.from_bytes(doc.to_bytes())
            return "|".join(p.text for p in again.paragraphs) + "#" + again.part_xml("/word/document.xml")
        log["reopen"] = outcome(reopen)
    return log


def index_of(doc, text):
    for i, p in enumerate(doc.paragraphs):
        if p.text == text:
            return doc.find_content_index(p)
    raise LookupError(text)


def para(doc, text):
    for p in doc.paragraphs:
        if p.text == text:
            return p
    raise LookupError(text)


def replace(doc, text, new):
    p = para(doc, text)
    p.replace_xml(p.xml.replace(f">{text}<", f">{new}<"))
    return p.text


OPS = [
    ("rm_before", lambda d: d.remove_content(index_of(d, "s-b1"))),
    ("rm_after", lambda d: d.remove_content(index_of(d, "s-a1"))),
    ("ins_before_owner", lambda d: d.insert_paragraph(index_of(d, "s-b2") + 1, "ib").text),
    ("ins_after_owner", lambda d: d.insert_paragraph(index_of(d, "s-a2"), "ia").text),
    ("replace_adjacent", lambda d: replace(d, "s-a2", "s-a2x")),
    ("range_before", lambda d: d.remove_content_range(index_of(d, "s-b2"), index_of(d, "s-b2") + 2)),
    ("before_after_owner", lambda d: para(d, "s-a2x").insert_paragraph_before("pb").text),
    ("set_text", lambda d: setattr(para(d, "last"), "text", "last2")),
    ("rm_last_tail", lambda d: d.remove_content(index_of(d, "t-a1"))),
    ("add", lambda d: d.add_paragraph("added").text),
]


def main():
    result = {}
    for name, (body, root) in SHAPES.items():
        for presave in (False, True):
            key = f"{name}{'+presave' if presave else ''}"
            try:
                doc = build(body, root)
            except BaseException as error:
                result[key] = {"build": [type(error).__name__, str(error)[:300]]}
                continue
            if presave:
                outcome(doc.to_bytes)
            log = {"initial": state(doc), "steps": []}
            for op_name, fn in OPS:
                log["steps"].append([op_name, outcome(lambda: fn(doc)), state(doc)])
            result[key] = log
    json.dump(result, sys.stdout, indent=0)


if __name__ == "__main__":
    main()
