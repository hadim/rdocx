"""diff407 random edits on shapes with unrouted paragraphs (nested content controls) at several places.

usage: python fz2.py SEED_START SEED_END STEPS > log.json
"""
import json
import random
import sys

sys.path.insert(0, "/private/tmp/claude-501/-Users-hadim-Code-libs-rdocx/08057ee4-afc5-4751-aca1-2668752380de/scratchpad/rv407")
import diff407 as base  # noqa: E402

P, TBL = base.P, base.TBL
ORIGINAL_SHAPE = base.shape
W = 'xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"'
W15 = 'xmlns:w15="http://schemas.microsoft.com/office/word/2012/wordml"'


def rows(tag, n=1, extra=""):
    items = "".join(f'<w:sdt><w:sdtPr><w15:repeatingSectionItem/></w:sdtPr><w:sdtContent>{P(f"{tag}row{i}")}'
                    '</w:sdtContent></w:sdt>' for i in range(n))
    return (f'<w:sdt {W} {W15}><w:sdtPr><w:tag w:val="{tag}"/><w15:repeatingSection/></w:sdtPr><w:sdtContent>'
            f'{extra}{items}</w:sdtContent></w:sdt>')


def nested_plain(tag):
    """A block control holding a routed paragraph, a nested control's paragraph and another routed paragraph."""
    return (f'<w:sdt><w:sdtPr><w:tag w:val="{tag}"/></w:sdtPr><w:sdtContent>{P(tag + "a")}'
            f'<w:sdt><w:sdtPr><w:tag w:val="{tag}in"/></w:sdtPr><w:sdtContent>{P(tag + "deep")}</w:sdtContent></w:sdt>'
            f'{P(tag + "b")}</w:sdtContent></w:sdt>')


def shape(name):
    if name == "rows_first":
        return rows("r0", 2) + P("a0") + P("a1") + TBL("t") + P("a2") + rows("r1") + P("a3"), ""
    if name == "rows_consecutive":
        return P("a0") + rows("r0") + rows("r1", 2) + TBL("t") + rows("r2") + P("a1") + P("a2"), ""
    if name == "rows_before_table":
        return P("a0") + P("a1") + rows("r0") + TBL("t") + P("a2") + rows("r1", 1, P("lead")) + P("a3"), ""
    if name == "rows_last":
        return P("a0") + TBL("t") + P("a1") + P("a2") + rows("r0", 3), ""
    if name == "nested_mixed":
        return P("a0") + nested_plain("n0") + P("a1") + TBL("t") + nested_plain("n1") + P("a2") + rows("r0") + P("a3"), ""
    if name == "review":
        return ORIGINAL_SHAPE("review")
    raise ValueError(name)


SHAPES = ["rows_first", "rows_consecutive", "rows_before_table", "rows_last", "nested_mixed", "review"]
base.shape = shape
base.SHAPES = SHAPES

if __name__ == "__main__":
    base.main()
