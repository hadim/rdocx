"""Re-record the archive measurement of the given crates until stable: python rerecord407.py REPO CRATE..."""
import importlib.util
import re
import sys
from pathlib import Path

repo = Path(sys.argv[1])
crates = sys.argv[2:]
DATE = "2026-10-23"
spec = importlib.util.spec_from_file_location("readme_doctests", repo / "scripts/readme_doctests.py")
module = importlib.util.module_from_spec(spec)
sys.modules["readme_doctests"] = module
spec.loader.exec_module(module)
packages = {p["name"]: p for p in module.cargo_metadata()["packages"]}
script = repo / "scripts/readme_doctests.py"
for attempt in range(4):
    changed = False
    for crate in crates:
        archive = module.build_package_archive(packages[crate])
        compressed, members, count = module.archive_measurement(archive)
        text = script.read_text()
        entry = re.compile(rf'(    "{re.escape(crate)}": )\([0-9_]+, [0-9_]+, [0-9]+\),')
        new = f'\\g<1>({compressed:_}, {members:_}, {count}),'
        updated = entry.sub(new, text, count=1)
        updated = re.sub(rf'(    "{re.escape(crate)}": )"[0-9-]+",', rf'\g<1>"{DATE}",', updated, count=1)
        if updated != text:
            script.write_text(updated)
            changed = True
        readme = repo / ("README.md" if crate == "rdocx" else f"crates/{crate}/README.md")
        text = readme.read_text()
        row = re.compile(rf'(\| Crates\.io archive: {re.escape(crate)} \| )[0-9,]+ compressed bytes, [0-9,]+ member bytes, [0-9]+ members(.*\| )[0-9-]+( \|)')
        updated = row.sub(rf'\g<1>{compressed:,} compressed bytes, {members:,} member bytes, {count} members\g<2>{DATE}\g<3>', text, count=1)
        if updated != text:
            readme.write_text(updated)
            changed = True
        print(attempt, crate, compressed, members, count)
    if not changed:
        break
