"""Do the fz2 shapes take the splice? Count whole-body work around edits next to unrouted paragraphs (debug build)."""
import random
import fz2
import rdocx
from rdocx._rdocx import _word_whole_body_work as work


def slot_of(doc, text):
    for p in doc.paragraphs:
        if p.text.startswith(text):
            return doc.find_content_index(p)
    return None


for name in fz2.SHAPES:
    doc = fz2.base.build(name, random.Random(1))
    doc.paragraphs  # warm the inventory
    results = []
    count = doc.paragraphs and len(doc.paragraphs)
    # an insertion before every body slot, then removal of the inserted paragraph
    slots = len(doc.paragraphs) + len(doc.tables) + 2
    for slot in range(0, 40):
        try:
            before = work()
            doc.insert_paragraph(slot, "probe")
            mid = work()
            ok_remove = doc.remove_content(slot)
            after = work()
        except Exception as error:
            break
        results.append((slot, mid != before, after != mid))
    spliced_ins = sum(1 for _, i, _ in results if not i)
    spliced_rm = sum(1 for _, _, r in results if not r)
    fell = [s for s, i, r in results if i or r]
    print(f"{name:20s} slots={len(results)} insert spliced={spliced_ins} remove spliced={spliced_rm} fallback slots={fell}")
    print("   bytes ok:", len(doc.to_bytes()) > 0, "texts:", "|".join(p.text[:6] for p in doc.paragraphs)[:150])
