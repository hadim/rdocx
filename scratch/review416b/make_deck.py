import sys
import rpptx
p = rpptx.Presentation()
for i in range(int(sys.argv[2])):
    s = p.slides.add_slide(p.slide_layouts[1])
    for _ in range(4):
        s.shapes.add_textbox(0, 0, 100, 100).text_frame.text = "text " * 20
    s.shapes.add_table(3, 3, 0, 0, 900, 900)
    s.notes_slide.notes_text_frame.text = "notes"
open(sys.argv[1], "wb").write(p.to_bytes())
