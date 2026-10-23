import sys, time, importlib.util
spec = importlib.util.spec_from_file_location("repro", sys.argv[0].replace("first407.py", "repro407.py"))
src = open(spec.origin).read().split("print(\"rdocx\"")[0]
g = {}
exec(compile(src, spec.origin, "exec"), g)
for trial in range(3):
    doc = g["build"](1000)
    p = doc.paragraphs[20]
    p.xml
    times = []
    for _ in range(6):
        start = time.perf_counter(); doc.insert_paragraph(10, "x"); times.append((time.perf_counter() - start) * 1000)
    rtimes = []
    for _ in range(4):
        start = time.perf_counter(); doc.remove_content(10); rtimes.append((time.perf_counter() - start) * 1000)
    print("insert", " ".join(f"{t:.2f}" for t in times), "| remove", " ".join(f"{t:.2f}" for t in rtimes))
