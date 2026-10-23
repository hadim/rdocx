"""Mean ms of shape-bound calls in decks of N slides.  python timeit.py LIBDIR N [N ...]"""

import sys
import time

sys.path.insert(0, sys.argv[1])
import rpptx  # noqa: E402


def deck(slides):
    prs = rpptx.Presentation()
    for _ in range(slides):
        slide = prs.slides.add_slide(prs.slide_layouts[6])
        for index in range(25):
            box = slide.shapes.add_textbox(index, 0, 100, 100)
            box.text_frame.text = f"text {index}"
        for _ in range(12):
            slide.shapes.add_shape("rect", 0, 0, 100, 100)
        slide.shapes.add_table(4, 4, 0, 0, 900, 900)
    return prs


def bench(prs, label, setup, call, repeat=10):
    total = 0.0
    for index in range(repeat):
        target = setup(index)
        start = time.perf_counter()
        call(target, index)
        total += time.perf_counter() - start
    return label, 1000 * total / repeat


def main():
    results = {}
    for slides in map(int, sys.argv[2:]):
        prs = deck(slides)
        slide = prs.slides[0]
        new_slide = lambda i: prs.slides.add_slide(prs.slide_layouts[6])  # noqa: E731
        rows = [
            bench(prs, "add_shape", lambda i: slide, lambda s, i: s.shapes.add_shape("rect", 0, 0, 9, 9)),
            bench(prs, "add_run", lambda i: slide.shapes[i].text_frame.paragraphs[0], lambda p, i: p.add_run("x")),
            bench(prs, "text_frame.text", lambda i: slide.shapes[i].text_frame, lambda f, i: setattr(f, "text", f"n{i}")),
            bench(prs, "Shape.name", lambda i: slide.shapes[i], lambda s, i: setattr(s, "name", f"nm{i}")),
            bench(prs, "run hyperlink", lambda i: slide.shapes[i].text_frame.paragraphs[0].runs[0],
                  lambda r, i: setattr(r.hyperlink, "address", f"https://e.com/{i}")),
            bench(prs, "cell.text", lambda i: slide.shapes[37].table.cell(i % 4, i // 4 % 4), lambda c, i: setattr(c, "text", f"c{i}")),
            bench(prs, "shapes.move", lambda i: slide, lambda s, i: s.shapes.move(0, 5)),
            bench(prs, "add_slide", lambda i: None, lambda _, i: prs.slides.add_slide(prs.slide_layouts[6])),
            bench(prs, "slides.move", lambda i: None, lambda _, i: prs.slides.move(0, 1)),
            bench(prs, "replace_xml text", lambda i: slide.shapes[i + 10].element,
                  lambda e, i: e.replace_xml(e.xml.replace("text ", "TEXT ", 1))),
        ]
        results[slides] = rows
    labels = [label for label, _ in next(iter(results.values()))]
    print("call".ljust(20) + "".join(f"{n:>10}" for n in results))
    for at, label in enumerate(labels):
        print(label.ljust(20) + "".join(f"{results[n][at][1]:>10.2f}" for n in results))


main()
