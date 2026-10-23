"""Split the samples under a root function of a macOS `sample` call graph into cost categories:
python split.py FILE ROOT_REGEX [--top N] [--children]

Each sample is classified by the first frame, from the root down, that matches a category marker:
  save   Document::to_bytes (whole package staging + ZIP write)
  inv    binding owner inventory: prepare_word_revision, word_owner_locations, indexes, splice, revision counter
  copy   clone_for_staging and dropping a whole Document
  rust   other whole-document native work (serialize, parse, scan, validate, canonicalize, layout)
  edit   everything else under the root
"""
import re
import sys
from collections import defaultdict

LINE = re.compile(r"^(?P<indent>[\s+!:|]*?)(?P<count>\d+) (?P<name>.+?)\s+\(in (?P<lib>[^)]+)\)")
CATEGORIES = [
    ("save", re.compile(r"8Document8to_bytes|8Document13to_bytes_for|Document::to_bytes")),
    ("inv", re.compile(r"prepare_word_revision|word_owner_locations|WordOwnerIndexes|commit_word_revision|"
                       r"relocated_property_owners|indexed_inventory_matches|word_owner_addresses|"
                       r"prepare_body_splice|commit_body_splice|RevisionCounter|15RevisionCounter|"
                       r"(drop_in_place|drop_glue).*WordOwner|PreparedWordRevision|adopt_property_edit")),
    ("copy", re.compile(r"clone_for_staging|(drop_in_place|drop_glue).*?5rdocx8document8Document")),
    ("reopen", re.compile(r"reopen_prepared_staged|from_package|8Document10from_bytes|from_bytes_with_limits")),
    ("stage", re.compile(r"prepare_staged_package|prepare_staged_output|flush_to_package|flush_document_to_package|"
                         r"canonicaliz|reconcile_comment|opaque_relationship_ids|refresh_related_story")),
    ("valid", re.compile(r"validate_numbering_graph|validate_internal_relationship|validated_content_boundary|"
                         r"document_numbering_references|validate_style_graph|prove_scoped_main_source")),
    ("addr", re.compile(r"story_item_source|story_source_and_owner|story_range_paragraph_source|8Document8pictures|"
                        r"modeled_main_cell_route|cell_paragraph_story_location|paragraph_story_location|"
                        r"comment_anchor|checked_movable_comment|fragment_story_source|Location6native|bind_prepared|"
                        r"collect_story_item_snapshots|index_elements|parent_can_hold")),
    ("rust", re.compile(r"to_xml|from_xml|story_sources|scan_story|validate|renumber|5rdocx_layout|"
                        r"flush_dirty|write_to|from_reader|modeled_owner_spans|"
                        r"story_item_snapshots|story_owner_snapshots|stories|5Clone5clone|clone_from")),
]


def pr_of(label, decider):
    """The PR of the plan that removes the whole-document work of one bucket entry."""
    if label == "inv":
        return "P1"
    if label == "addr" and re.search(r"bind_prepared|Location::native|_rdocx::", decider):
        return "P1"
    if label in ("save", "stage"):
        return "P2"
    if label == "valid":
        return "P4" if re.search(r"prove_scoped|validate_internal_relationship|validated_content_boundary", decider) else "P2"
    if label == "reopen":
        return "P3"
    if label == "addr":
        return "P4"
    if label == "rust":
        return "P2" if re.search(r"from_xml|to_xml|modeled_owner_spans|nested_modeled|replay_nested", decider) else "P4"
    if label == "copy":
        return "C"
    return "E"


def demangle(name):
    """Readable v0 path: the identifiers of a _R symbol, joined by ::."""
    name = re.sub(r"::h[0-9a-f]{16}$", "", name.strip())
    name = re.sub(r"\s+\+ \d+.*$", "", name)
    if not name.startswith("_R"):
        return name
    parts = []
    i = 0
    while i < len(name):
        m = re.match(r"(\d+)(_?)", name[i:])
        if m and int(m.group(1)) > 0 and (i == 0 or not name[i - 1].isdigit()):
            n = int(m.group(1))
            start = i + len(m.group(0))
            ident = name[start:start + n]
            if re.fullmatch(r"[A-Za-z_][A-Za-z0-9_]*", ident or "-"):
                parts.append(ident)
                i = start + n
                continue
        i += 1
    return "::".join(p for p in parts if not re.fullmatch(r"[a-z]{1,2}", p))[:160]


def parse(path):
    text = open(path, errors="replace").read()
    graph = text.split("Call graph:", 1)[1].split("Total number in stack", 1)[0]
    nodes = []  # (depth, name, count, parent)
    stack = []
    for line in graph.splitlines():
        m = LINE.match(line)
        if not m:
            continue
        depth = len(m.group("indent"))
        while stack and nodes[stack[-1]][0] >= depth:
            stack.pop()
        parent = stack[-1] if stack else None
        nodes.append((depth, m.group("name").strip(), int(m.group("count")), parent))
        stack.append(len(nodes) - 1)
    return nodes


def main():
    path, root = sys.argv[1], re.compile(sys.argv[2])
    top = int(sys.argv[sys.argv.index("--top") + 1]) if "--top" in sys.argv else 8
    nodes = parse(path)
    children = defaultdict(list)
    for i, (_, _, _, parent) in enumerate(nodes):
        if parent is not None:
            children[parent].append(i)
    roots = []
    for i, (_, name, _, parent) in enumerate(nodes):
        if root.search(name):
            p, nested = parent, False
            while p is not None:
                if root.search(nodes[p][1]):
                    nested = True
                    break
                p = nodes[p][3]
            if not nested:
                roots.append(i)
    total = sum(nodes[r][2] for r in roots)
    split = defaultdict(int)
    self_frames = defaultdict(lambda: defaultdict(int))
    first_children = defaultdict(int)
    deciders = defaultdict(lambda: defaultdict(int))

    def walk(i, category, decider):
        own = nodes[i][2] - sum(nodes[c][2] for c in children[i])
        if own > 0:
            split[category] += own
            self_frames[category][demangle(nodes[i][1])] += own
            deciders[category][decider] += own
        for c in children[i]:
            cat, dec = category, decider
            if category == "edit":
                for label, pattern in CATEGORIES:
                    if pattern.search(nodes[c][1]):
                        cat, dec = label, demangle(nodes[c][1])
                        break
            walk(c, cat, dec)

    for r in roots:
        for c in children[r]:
            first_children[demangle(nodes[c][1])] += nodes[c][2]
        walk(r, "edit", "-")
    print(f"root samples {total} in {len(roots)} root frames")
    if "--pr" in sys.argv:
        groups = defaultdict(int)
        for label, by in deciders.items():
            for decider, count in by.items():
                groups[pr_of(label, decider)] += count
        print("PR", sys.argv[sys.argv.index("--pr") + 1], total,
              *[f"{100 * groups[g] / max(total, 1):.1f}" for g in ("P1", "P2", "P3", "P4", "C", "E")])
        return
    if "--row" in sys.argv:
        print("ROW", sys.argv[sys.argv.index("--row") + 1], total, *[f"{100 * split[l] / max(total, 1):.1f}" for l in ("copy", "inv", "save", "stage", "reopen", "valid", "addr", "rust", "edit")])
        return
    print("  " + "  ".join(f"{label} {100 * split[label] / max(total, 1):4.1f}%"
                           for label in ("copy", "inv", "save", "stage", "reopen", "valid", "addr", "rust", "edit")))
    if "--children" in sys.argv:
        print("children of the root:")
        for name, count in sorted(first_children.items(), key=lambda x: -x[1])[:top]:
            print(f"   {100 * count / total:5.1f}%  {name}")
    for label in ("copy", "inv", "save", "stage", "reopen", "valid", "addr", "rust", "edit"):
        if not split[label]:
            continue
        print(f"[{label}] entered through:")
        for name, count in sorted(deciders[label].items(), key=lambda x: -x[1])[:top]:
            print(f"   {100 * count / total:5.1f}%  {name}")


main()


def inclusive_under_root():
    """--inclusive N: the N functions with the largest inclusive share under the root."""
    path, root = sys.argv[1], re.compile(sys.argv[2])
    count = int(sys.argv[sys.argv.index("--inclusive") + 1])
    nodes = parse(path)
    children = defaultdict(list)
    for i, (_, _, _, parent) in enumerate(nodes):
        if parent is not None:
            children[parent].append(i)
    inclusive = defaultdict(int)
    total = 0

    def walk(i, seen):
        name = demangle(nodes[i][1])
        if name not in seen:
            inclusive[name] += nodes[i][2]
        for c in children[i]:
            walk(c, seen | {name})

    for i, (_, name, cnt, parent) in enumerate(nodes):
        if root.search(name):
            p, nested = parent, False
            while p is not None:
                if root.search(nodes[p][1]):
                    nested = True
                    break
                p = nodes[p][3]
            if not nested:
                total += cnt
                walk(i, frozenset())
    print(f"inclusive under root ({total} samples)")
    for name, c in sorted(inclusive.items(), key=lambda x: -x[1])[:count]:
        print(f"   {100 * c / max(total, 1):5.1f}%  {name}")


if "--inclusive" in sys.argv:
    inclusive_under_root()
