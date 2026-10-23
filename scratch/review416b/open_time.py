"""Median time of Presentation.from_bytes on a saved deck, and of one
slide-level footer edit, for the build on sys.path."""
import statistics, sys, time
import rpptx

data = open(sys.argv[1], "rb").read()
runs = int(sys.argv[2])
opens = []
for _ in range(runs):
    t = time.perf_counter()
    rpptx.Presentation.from_bytes(data)
    opens.append(time.perf_counter() - t)
edits = []
for _ in range(max(3, runs // 2)):
    p = rpptx.Presentation.from_bytes(data)
    t = time.perf_counter()
    p.slides[0].header_footer.slide_number = True
    edits.append(time.perf_counter() - t)
print(f"{sys.argv[3]}: open median {statistics.median(opens)*1e3:.1f} ms (min {min(opens)*1e3:.1f}), slide_number edit median {statistics.median(edits)*1e3:.2f} ms")
