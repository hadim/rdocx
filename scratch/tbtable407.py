"""insert_table and add_table in a main part with a text box, at 100 and 1000 paragraphs."""
import importlib.util, sys
spec = importlib.util.spec_from_file_location("siblings", sys.argv[0].replace("tbtable407.py", "siblings407.py"))
siblings = importlib.util.module_from_spec(spec)
exec(compile(open(spec.origin).read().split('print("rdocx"')[0], spec.origin, "exec"), siblings.__dict__)
TB = ('<w:p><w:r><w:drawing><wp:anchor distT="0" distB="0" distL="0" distR="0" simplePos="0" relativeHeight="1" behindDoc="0" locked="0" layoutInCell="1" allowOverlap="1">'
      '<wp:simplePos x="0" y="0"/><wp:positionH relativeFrom="column"><wp:posOffset>0</wp:posOffset></wp:positionH><wp:positionV relativeFrom="paragraph"><wp:posOffset>0</wp:posOffset></wp:positionV>'
      '<wp:extent cx="914400" cy="914400"/><wp:effectExtent l="0" t="0" r="0" b="0"/><wp:wrapNone/><wp:docPr id="7" name="TB"/><wp:cNvGraphicFramePr/>'
      '<a:graphic xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"><a:graphicData uri="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">'
      '<wps:wsp xmlns:wps="http://schemas.microsoft.com/office/word/2010/wordprocessingShape"><wps:cNvSpPr txBox="1"/><wps:spPr><a:xfrm><a:off x="0" y="0"/><a:ext cx="914400" cy="914400"/></a:xfrm><a:prstGeom prst="rect"><a:avLst/></a:prstGeom></wps:spPr>'
      '<wps:txbx><w:txbxContent><w:p><w:r><w:t>in box</w:t></w:r></w:p></w:txbxContent></wps:txbx><wps:bodyPr/></wps:wsp></a:graphicData></a:graphic></wp:anchor></w:drawing></w:r></w:p>')
siblings.TABLE = siblings.TABLE + TB
rows = {}
for count in (100, 1000):
    doc = siblings.build(count)
    rows[count] = (siblings.per_call(lambda: doc.insert_table(10, 2, 2)), siblings.per_call(lambda: doc.add_table(2, 2)))
for index, name in enumerate(("insert_table", "add_table")):
    print(f"{name} with a text box: {rows[100][index]:.2f} ms at 100, {rows[1000][index]:.2f} ms at 1000")
