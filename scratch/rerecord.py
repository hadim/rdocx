"""Re-record crates.io archive measurements for the given crates until stable."""
import importlib.util, json, re, subprocess, sys
from pathlib import Path

ROOT = Path("/Users/hadim/Code/libs/rdocx/worktrees/403-inventory")
DATE = "2026-10-23"
crates = sys.argv[1:]
spec = importlib.util.spec_from_file_location("readme_doctests", ROOT / "scripts/readme_doctests.py")

def load():
    module = importlib.util.module_from_spec(spec)
    sys.modules["readme_doctests"] = module
    spec.loader.exec_module(module)
    return module

def readme(name):
    return ROOT / "README.md" if name == "rdocx" else ROOT / f"crates/{name}/README.md"

for round in range(5):
    module = load()
    metadata = json.loads(subprocess.run(["cargo", "metadata", "--format-version", "1", "--no-deps"], cwd=ROOT,
                                         capture_output=True, text=True, check=True).stdout)
    packages = {p["name"]: p for p in metadata["packages"]}
    changed = False
    script = (ROOT / "scripts/readme_doctests.py").read_text()
    for name in crates:
        archive = module.build_package_archive(packages[name])
        measured = module.archive_measurement(archive)
        old = module.ARCHIVE_MEASUREMENTS[name]
        old_date = module.ARCHIVE_REMEASUREMENT_DATES[name]
        print(round, name, old, "->", measured)
        if tuple(measured) == tuple(old) and old_date == DATE:
            continue
        changed = True
        fmt = lambda t: f"({t[0]:_}, {t[1]:_}, {t[2]})"
        script = script.replace(f'    "{name}": {fmt(old)},', f'    "{name}": {fmt(measured)},', 1)
        script = re.sub(rf'(ARCHIVE_REMEASUREMENT_DATES = \{{.*?\n    "{re.escape(name)}": )"[^"]+"', rf'\1"{DATE}"', script, count=1, flags=re.S)
        path = readme(name)
        text = path.read_text()
        old_row = f"| Crates.io archive: {name} | {old[0]:,} compressed bytes, {old[1]:,} member bytes, {old[2]} members |"
        new_row = f"| Crates.io archive: {name} | {measured[0]:,} compressed bytes, {measured[1]:,} member bytes, {measured[2]} members |"
        assert old_row in text, (name, old_row)
        text = text.replace(old_row, new_row, 1)
        # the row ends with the measurement date
        lines = text.split("\n")
        for i, line in enumerate(lines):
            if line.startswith(new_row):
                lines[i] = re.sub(r"\| [0-9-]+ \|$", f"| {DATE} |", line)
        path.write_text("\n".join(lines))
    (ROOT / "scripts/readme_doctests.py").write_text(script)
    if not changed:
        print("stable")
        break
