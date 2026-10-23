"""Differential fuzz of #407 body edits and reads: run under two rdocx builds, diff the JSON logs.

usage: python diff407.py SEED_START SEED_END > log.json
"""
import hashlib
import io
import json
import random
import sys
import zipfile

import rdocx

W14 = "http://schemas.microsoft.com/office/word/2010/wordml"
WORD_ROOT = (
    ' xmlns:wpc="http://schemas.microsoft.com/office/word/2010/wordprocessingCanvas"'
    ' xmlns:o="urn:schemas-microsoft-com:office:office"'
    ' xmlns:v="urn:schemas-microsoft-com:vml"'
    ' xmlns:w10="urn:schemas-microsoft-com:office:word"'
    f' xmlns:w14="{W14}"'
    ' xmlns:w15="http://schemas.microsoft.com/office/word/2012/wordml"'
    ' xmlns:wps="http://schemas.microsoft.com/office/word/2010/wordprocessingShape"'
    ' mc:Ignorable="w14 w15 wp14"'
)
_pid = [0]


def pid():
    _pid[0] += 1
    return f' w14:paraId="{_pid[0]:08X}" w14:textId="77777777" w:rsidR="00A1B2C3" w:rsidRDefault="00A1B2C3"'


def P(text, attrs="", ppr="", runs=1):
    rs = "".join(f'<w:r><w:rPr><w:i/></w:rPr><w:t xml:space="preserve">{text} r{k} </w:t></w:r>' for k in range(runs))
    return f"<w:p{attrs}>{ppr}{rs}</w:p>"


def TBL(prefix, rows=2, cols=2, nested=False):
    out = "<w:tbl><w:tblGrid>" + '<w:gridCol w:w="1500"/>' * cols + "</w:tblGrid>"
    for r in range(rows):
        out += "<w:tr>"
        for c in range(cols):
            inner = TBL(prefix + "n", 1, 1) + "<w:p/>" if nested and r == 0 and c == 0 else ""
            out += f"<w:tc><w:p><w:r><w:t>{prefix}{r}{c}</w:t></w:r></w:p>{inner}</w:tc>"
        out += "</w:tr>"
    return out + "</w:tbl>"


SECT = '<w:sectPr><w:pgSz w:w="12240" w:h="15840"/><w:pgMar w:top="1440" w:right="1440" w:bottom="1440" w:left="1440" w:header="720" w:footer="720" w:gutter="0"/></w:sectPr>'


def shape(name):
    a = ""
    if name == "plain":
        return "".join(P(f"p{i}", runs=2) for i in range(8)), ""
    if name == "word":
        body = [P(f"w{i}", pid(), runs=2) for i in range(4)]
        body.insert(1, f'<w:p{pid()}><w:bookmarkStart w:id="5" w:name="bm"/><w:r><w:t>bm start</w:t></w:r></w:p>')
        body.insert(3, f'<w:p{pid()}><w:r><w:t>bm end</w:t></w:r><w:bookmarkEnd w:id="5"/></w:p>')
        body.insert(4, P("section break", pid(), f"<w:pPr>{SECT}</w:pPr>"))
        body.append(f'<w:p{pid()}><w:ins w:id="9" w:author="a" w:date="2026-01-01T00:00:00Z"><w:r><w:t>ins</w:t></w:r></w:ins></w:p>')
        body.append(f'<w:p{pid()}><w:hyperlink w:anchor="bm"><w:r><w:t>link</w:t></w:r></w:hyperlink></w:p>')
        body.append(TBL("wt"))
        body += [P(f"t{i}", pid()) for i in range(3)]
        return "".join(body), WORD_ROOT
    if name == "tables":
        body = [TBL("a", nested=True), P("x0"), P("x1"), TBL("b", 3, 2), P("x2"), TBL("c", 1, 3, nested=True), P("x3"),
                '<w:tbl><w:tblGrid><w:gridCol w:w="1000"/><w:gridCol w:w="1000"/></w:tblGrid><w:tr><w:tc><w:tcPr><w:gridSpan w:val="2"/></w:tcPr><w:p/></w:tc></w:tr>'
                '<w:tr><w:tc><w:tcPr><w:vMerge w:val="restart"/></w:tcPr><w:p/></w:tc><w:tc><w:p/></w:tc></w:tr><w:tr><w:tc><w:tcPr><w:vMerge/></w:tcPr><w:p/></w:tc><w:tc><w:p/></w:tc></w:tr></w:tbl>',
                P("x4")]
        return "".join(body), ""
    if name == "sdt":
        body = [P("s0"), '<w:sdt><w:sdtPr><w:alias w:val="blk"/></w:sdtPr><w:sdtContent>' + P("in0") + TBL("sd") + P("in1") + "</w:sdtContent></w:sdt>",
                P("s1"), TBL("after"), P("s2"), '<w:sdt><w:sdtPr/><w:sdtContent>' + P("in2") + "</w:sdtContent></w:sdt>", P("s3")]
        return "".join(body), ""
    if name == "w14inside":
        body = [P("a"), f'<w:p xmlns:w14="{W14}" w14:paraId="1234ABCD"><w:r><w:t>has id</w:t></w:r></w:p>', P("b"), TBL("q"), P("c"),
                P("lit w14:thing"), P("d")]
        return "".join(body), ""
    if name == "customxml":
        body = [P("c0"), '<w:customXml w:element="blk">' + TBL("cx") + "</w:customXml>", P("c1"), TBL("real"), P("c2"), P("c3")]
        return "".join(body), ""
    if name == "altchunk":
        body = [P("k0"), '<mc:AlternateContent><mc:Choice Requires="w14">' + TBL("ch") + '</mc:Choice><mc:Fallback>' + TBL("fb") + '</mc:Fallback></mc:AlternateContent>', P("k1"), TBL("k"), P("k2")]
        return "".join(body), ""
    if name == "textbox":
        tb = ('<w:p><w:r><mc:AlternateContent><mc:Choice Requires="wps"><w:drawing><wp:anchor distT="0" distB="0" distL="0" distR="0" simplePos="0" relativeHeight="1" behindDoc="0" locked="0" layoutInCell="1" allowOverlap="1">'
              '<wp:simplePos x="0" y="0"/><wp:positionH relativeFrom="column"><wp:posOffset>0</wp:posOffset></wp:positionH><wp:positionV relativeFrom="paragraph"><wp:posOffset>0</wp:posOffset></wp:positionV>'
              '<wp:extent cx="914400" cy="914400"/><wp:effectExtent l="0" t="0" r="0" b="0"/><wp:wrapNone/><wp:docPr id="7" name="TB"/><wp:cNvGraphicFramePr/>'
              '<a:graphic xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"><a:graphicData uri="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">'
              '<wps:wsp xmlns:wps="http://schemas.microsoft.com/office/word/2010/wordprocessingShape"><wps:cNvSpPr txBox="1"/><wps:spPr><a:xfrm><a:off x="0" y="0"/><a:ext cx="914400" cy="914400"/></a:xfrm><a:prstGeom prst="rect"><a:avLst/></a:prstGeom></wps:spPr>'
              '<wps:txbx><w:txbxContent><w:p><w:r><w:t>in box</w:t></w:r></w:p>' + TBL("tb") + '<w:p/></w:txbxContent></wps:txbx><wps:bodyPr/></wps:wsp></a:graphicData></a:graphic></wp:anchor></w:drawing></mc:Choice>'
              '<mc:Fallback/></mc:AlternateContent></w:r></w:p>')
        body = [P("b0"), TBL("pre"), P("b1"), tb, P("b2"), TBL("post"), P("b3")]
        return "".join(body), ""
    if name == "empty":
        return "", ""
    if name == "review":
        # python-docx review documents: snippets that declare `w` again, a repeating section declaring `w15`.
        w = 'xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"'
        w15 = 'xmlns:w15="http://schemas.microsoft.com/office/word/2012/wordml"'
        body = []
        for i in range(12):
            body.append(P(f"rv{i}", runs=3))
            if i % 4 == 3:
                body.append(TBL(f"rt{i}"))
        body.insert(6, P("section break", "", f"<w:pPr>{SECT}</w:pPr>"))
        body.append(f'<w:p><w:r><w:t xml:space="preserve">tracked </w:t></w:r><w:ins {w} w:id="901" w:author="Editor" '
                    'w:date="2026-01-01T00:00:00Z"><w:r><w:t>inserted</w:t></w:r></w:ins></w:p>')
        body.append(f'<w:sdt {w}><w:sdtPr><w:tag w:val="probe0"/></w:sdtPr><w:sdtContent>{P("control")}'
                    '</w:sdtContent></w:sdt>')
        body.append(P("spot"))
        body.append(f'<w:sdt {w} {w15}><w:sdtPr><w:tag w:val="rows"/><w15:repeatingSection/></w:sdtPr><w:sdtContent>'
                    f'<w:sdt><w:sdtPr><w15:repeatingSectionItem/></w:sdtPr><w:sdtContent>{P("row")}</w:sdtContent>'
                    '</w:sdt></w:sdtContent></w:sdt>')
        body.append(P("after rows"))
        return "".join(body), ""
    if name == "lastsect":
        body = [P("l0"), P("l1"), TBL("ls"), P("l2", "", f"<w:pPr>{SECT}</w:pPr>")]
        return "".join(body), ""
    raise ValueError(name)


SHAPES = ["plain", "word", "tables", "sdt", "w14inside", "customxml", "altchunk", "textbox", "empty", "lastsect",
          "review"]


def png():
    """A one-pixel PNG."""
    import struct, zlib

    def chunk(kind, data):
        return struct.pack(">I", len(data)) + kind + data + struct.pack(">I", zlib.crc32(kind + data) & 0xFFFFFFFF)
    return (b"\x89PNG\r\n\x1a\n" + chunk(b"IHDR", struct.pack(">IIBBBBB", 1, 1, 8, 2, 0, 0, 0))
            + chunk(b"IDAT", zlib.compress(b"\x00\xff\xff\xff")) + chunk(b"IEND", b""))


def build(name, rng):
    body, root = shape(name)
    doc = rdocx.Document()
    if name != "empty":
        doc.add_paragraph("first")
    extras = rng.random()
    if name == "review":
        doc.add_paragraph("pictured").add_run("").add_picture(png(), "pixel.png")
        doc.add_comment_on_text("first", author="a", text="c")
    if extras < 0.5 or name == "review":
        try:
            doc.section(0).header.add_paragraph("header text")
            doc.section(0).header.add_table(1, 2)
        except Exception:
            pass
    if 0.3 < extras < 0.6 and name != "empty":
        try:
            doc.add_paragraph("noted")
            doc.add_footnote(doc.paragraphs[-1], "foot text")
        except Exception as error:
            print("footnote failed", error, file=sys.stderr)
    if extras > 0.85 and name != "empty":
        try:
            doc.add_comment_on_text("first", author="a", text="c")
        except Exception as error:
            print("comment failed", error, file=sys.stderr)
    z = zipfile.ZipFile(io.BytesIO(doc.to_bytes()))
    out = io.BytesIO()
    with zipfile.ZipFile(out, "w") as archive:
        for info in z.infolist():
            data = z.read(info.filename)
            if info.filename == "word/document.xml":
                text = data.decode()
                at = text.index("<w:sectPr")
                text = text[:at] + body + text[at:]
                if root:
                    text = text.replace("<w:document ", "<w:document" + root + " ", 1)
                    if 'xmlns:mc=' not in text.split(">", 2)[1]:
                        pass
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
    except BaseException as error:  # PanicException derives from BaseException
        if isinstance(error, KeyboardInterrupt):
            raise
        return ["err", type(error).__name__, str(error)[:400]]


def reads(doc):
    log = {}
    try:
        paragraphs = list(doc.paragraphs)
    except BaseException as error:
        return {"paragraphs": ["err", type(error).__name__, str(error)]}
    log["texts"] = [outcome(lambda p=p: p.text) for p in paragraphs]
    log["pxml"] = [outcome(lambda p=p: p.xml) for p in paragraphs]
    runs = []
    for p in paragraphs:
        try:
            rs = list(p.runs)
        except BaseException as error:
            runs.append(["err", type(error).__name__, str(error)])
            continue
        runs.append([outcome(lambda r=r: r.xml) for r in rs])
    log["rxml"] = runs
    tables = []
    try:
        ts = list(doc.tables)
    except BaseException as error:
        ts = []
        tables.append(["err", type(error).__name__, str(error)])
    for t in ts:
        entry = {"xml": outcome(lambda t=t: t.xml), "cells": []}
        try:
            for row in t.rows:
                for cell in row.cells:
                    entry["cells"].append(outcome(lambda c=cell: c.xml))
                    entry["cells"].append(outcome(lambda c=cell: [q.text for q in c.paragraphs] and "|".join(q.text for q in c.paragraphs)))
        except BaseException as error:
            entry["cells"].append(["err", type(error).__name__, str(error)])
        tables.append(entry)
    log["tables"] = tables
    log["bytes"] = outcome(doc.to_bytes)
    log["count"] = outcome(lambda: len(doc.paragraphs))
    return log


def held(doc, rng):
    hs = []
    try:
        ps = list(doc.paragraphs)
        for p in ps[:1] + ps[len(ps) // 2:len(ps) // 2 + 1] + ps[-1:]:
            hs.append(("p", p))
            rs = list(p.runs)
            if rs:
                hs.append(("r", rs[0]))
        ts = list(doc.tables)
        for t in ts[:1] + ts[-1:]:
            hs.append(("t", t))
            hs.append(("c", t.cell(0, 0)))
            cps = list(t.cell(0, 0).paragraphs)
            if cps:
                hs.append(("cp", cps[0]))
    except BaseException as error:
        hs.append(("err", str(error)))
    return hs


def probe(handles):
    out = []
    for kind, h in handles:
        if kind == "err":
            out.append(["held-err", h])
        elif kind in ("p", "cp"):
            out.append([kind, outcome(lambda: h.text), outcome(lambda: h.xml)])
        elif kind == "r":
            out.append([kind, outcome(lambda: h.text), outcome(lambda: h.xml)])
        else:
            out.append([kind, outcome(lambda: h.xml)])
    return out


def op(doc, rng):
    count = len(doc.paragraphs) + len(doc.tables) + 1
    k = rng.choice(["ins", "ins", "ins_style", "before", "add", "add_pb", "add_heading", "add_style", "rm", "rm", "rmr", "instbl", "addtbl", "addtbl_style",
                    "addrun", "rxml", "celltext", "settext"])
    i = rng.randint(0, count)
    if k == "ins":
        return k, i, outcome(lambda: doc.insert_paragraph(i, f"ins{i}").text)
    if k == "ins_style":
        return k, i, outcome(lambda: doc.insert_paragraph(i, f"sty{i}", "Heading1").text)
    if k == "before":
        ps = doc.paragraphs
        j = rng.randrange(len(ps)) if len(ps) else 0
        return k, j, outcome(lambda: ps[j].insert_paragraph_before("before").text)
    if k == "add":
        return k, 0, outcome(lambda: doc.add_paragraph("added").text)
    if k == "add_pb":
        return k, 0, outcome(lambda: doc.add_page_break().text)
    if k == "add_heading":
        return k, 0, outcome(lambda: doc.add_heading("hd", 2).text)
    if k == "add_style":
        return k, 0, outcome(lambda: doc.add_paragraph("as", "Heading1").text)
    if k == "rm":
        return k, i, outcome(lambda: doc.remove_content(i))
    if k == "rmr":
        j = rng.randint(i, min(count, i + 3))
        return k, (i, j), outcome(lambda: doc.remove_content_range(i, j))
    if k == "instbl":
        return k, i, outcome(lambda: len(doc.insert_table(i, 2, 2).rows))
    if k == "addtbl":
        return k, 0, outcome(lambda: len(doc.add_table(1, 3).rows))
    if k == "addtbl_style":
        return k, 0, outcome(lambda: len(doc.add_table(1, 2, "Table Grid").rows))
    if k == "addrun":
        ps = doc.paragraphs
        j = rng.randrange(len(ps)) if len(ps) else 0
        return k, j, outcome(lambda: ps[j].add_run("zz").text)
    if k == "rxml":
        ps = doc.paragraphs
        j = rng.randrange(len(ps)) if len(ps) else 0

        def go():
            x = ps[j].xml
            at = x.rindex("</w:p>") if "</w:p>" in x else None
            if at is None:
                x = x.replace("/>", "><w:r><w:t>rx</w:t></w:r></w:p>", 1) if x.endswith("/>") else x
            else:
                x = x[:at] + "<w:r><w:t>rx</w:t></w:r>" + x[at:]
            ps[j].replace_xml(x)
            return ps[j].text
        return k, j, outcome(go)
    if k == "celltext":
        ts = doc.tables

        def go():
            t = ts[rng.randrange(len(ts))]
            t.cell(0, 0).text = "ct"
            return t.cell(0, 0).text
        return k, 0, outcome(go)
    if k == "settext":
        ps = doc.paragraphs
        j = rng.randrange(len(ps)) if len(ps) else 0

        def go():
            ps[j].text = "st"
            return ps[j].text
        return k, j, outcome(go)


def main():
    start, end = int(sys.argv[1]), int(sys.argv[2])
    steps = int(sys.argv[3]) if len(sys.argv) > 3 else 10
    result = {}
    for seed in range(start, end):
        rng = random.Random(seed)
        name = SHAPES[seed % len(SHAPES)]
        _pid[0] = 0
        log = {"shape": name}
        try:
            doc = build(name, rng)
        except BaseException as error:
            result[seed] = {"shape": name, "build": [type(error).__name__, str(error)]}
            continue
        log["initial"] = reads(doc)
        log["steps"] = []
        for _ in range(steps):
            hs = held(doc, rng)
            step = list(op(doc, rng))
            step.append(probe(hs))
            step.append(reads(doc))
            log["steps"].append(step)
        result[seed] = log
    json.dump(result, sys.stdout)


if __name__ == "__main__":
    main()
