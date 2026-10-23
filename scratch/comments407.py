"""remove_content and remove_content_range in a document with a comments part, at 100 and 1000 paragraphs."""
import importlib.util, sys
import rdocx
spec = importlib.util.spec_from_file_location("siblings", sys.argv[0].replace("comments407.py", "siblings407.py"))
siblings = importlib.util.module_from_spec(spec)
exec(compile(open(spec.origin).read().split('print("rdocx"')[0], spec.origin, "exec"), siblings.__dict__)
rows = {}
for count in (100, 1000):
    doc = siblings.build(count)
    doc.add_comment(rdocx.RunRange(start=rdocx.RunPosition(body_index=1, run_index=0), end=rdocx.RunPosition(body_index=1, run_index=1)), author="Ada", text="review")
    rows[count] = (siblings.per_call(lambda: doc.remove_content(10)), siblings.per_call(lambda: doc.remove_content_range(10, 12)))
for index, name in enumerate(("remove_content", "remove_content_range")):
    print(f"{name} with a comments part: {rows[100][index]:.2f} ms at 100, {rows[1000][index]:.2f} ms at 1000")
