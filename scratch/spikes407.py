import sys, time, importlib.util
spec = importlib.util.spec_from_file_location("siblings", sys.argv[0].replace("spikes407.py", "siblings407.py"))
siblings = importlib.util.module_from_spec(spec)
src = open(spec.origin).read().split('print("rdocx"')[0]
exec(compile(src, spec.origin, "exec"), siblings.__dict__)
doc = siblings.build(1000)
times = []
for _ in range(60):
    start = time.perf_counter()
    doc.insert_paragraph(10, "x")
    times.append((time.perf_counter() - start) * 1000)
print(" ".join(f"{t:.1f}" for t in times))
