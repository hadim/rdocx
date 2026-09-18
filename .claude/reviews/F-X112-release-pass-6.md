# F-X112, release, pass 6

**Reviewed**: `git diff aba94e0c..26cf1f7c`, 65 files, 1811 insertions and 233
deletions. Excluding review files, 60 files, 978 insertions and 233 deletions.
Remediation commit `26cf1f7c` changes only `CHANGELOG.md` (36 lines) and adds
the pass 5 review. The `ListLevel` claims were checked against HEAD and
`v0.13.1`, the `Blip` claim against HEAD and `rpptx-v0.11.0`, and the Python
stub against `py-rdocx-v0.13.2`.
**Verdict**: 0 defects, 0 smells, 1 nitpick

## Pass 5 findings

### D1, fixed

`CHANGELOG.md:247` to `CHANGELOG.md:249` now says `ListLevel` is no longer
`Copy` or `Eq` and has private fields. Verified at HEAD:

- The derive at `crates/rdocx/src/document.rs:22354` is `Debug, Clone` only,
  and there is no `impl Eq for ListLevel`. At `v0.13.1` the derive included
  `Copy, PartialEq, Eq`. Losing `Copy` and `Eq` is true.
- The manual `impl PartialEq for ListLevel` at
  `crates/rdocx/src/document.rs:22374` keeps `==` working. The false
  `PartialEq` claim is gone.
- Twelve private fields at `crates/rdocx/src/document.rs:22360` to
  `crates/rdocx/src/document.rs:22371` follow the public `format` and `start`,
  so the struct literal break is now stated.
- `ListLevel::new(format)` at `crates/rdocx/src/document.rs:22394` and the
  `start(self, u32)` builder at `crates/rdocx/src/document.rs:22424` both
  exist at HEAD and at `v0.13.1`. `ListLevel::new(format).start(start)` is a
  valid replacement when `start` is a `u32`. The shorthand literal on line 248
  implies an `Option<u32>` binding, which is recorded as a nitpick below.

### Pass 5 nitpick 1, fixed

`CHANGELOG.md:77` to `CHANGELOG.md:83` no longer names `Blip`. `Blip` has the
private fields `alpha_modulation_fix_raw_attributes` and `raw_children` at
HEAD (`crates/oxml-drawing/src/fill.rs:614` to
`crates/oxml-drawing/src/fill.rs:615`) and `raw_children` at
`rpptx-v0.11.0`, so omitting it is correct. The remaining listed items are
unchanged from pass 5 and still true.

### Pass 5 nitpick 2, fixed

`CHANGELOG.md:400` to `CHANGELOG.md:402` now says "The one incompatible
signature change". A diff of `crates/rdocx-py/python/rdocx/_rdocx.pyi`
against `py-rdocx-v0.13.2` shows three removed signature lines only: the
`TocRebuildReport` constructor with `diagnostic_count`, `add_comment`'s
`range: RunRange`, now widened to `RunRange | StoryRunRange` with an optional
`date` at `crates/rdocx-py/python/rdocx/_rdocx.pyi:466`, and `reply_to`, now
with an optional `date` at `crates/rdocx-py/python/rdocx/_rdocx.pyi:475`.
Only the first is incompatible. The sentence is exact.

## Defects

None.

## Smells

None.

## Nitpicks

- `CHANGELOG.md:248` to `CHANGELOG.md:249`: in the old shorthand literal
  `ListLevel { format, start }` the `start` binding is `Option<u32>`, because
  the field is `pub start: Option<u32>` at
  `crates/rdocx/src/document.rs:22359`. The builder takes `u32` at
  `crates/rdocx/src/document.rs:22424`, so the mechanical rewrite
  `ListLevel::new(format).start(start)` fails with a type mismatch. The
  compiler reports it at once and the direction is right, so no caller is
  misled into a silent change. `ListLevel { format, start: Some(n) }` becoming
  `ListLevel::new(format).start(n)` would be exact.

## Not found

- **Release-note truth and family scope.** Added, Fixed, and family inventory
  text is unchanged since pass 4. `release-notes --check` reports ok for
  `v0.14.0`, `rpptx-v0.12.0`, `py-rdocx-v0.14.0`, and `py-rpptx-v0.12.0`. The
  7-package stable and 15-package incubating sets in both Compatibility
  sections match `Cargo.toml:55` to `Cargo.toml:78` and the crate manifests.
- **Compatibility statements.** Beyond the nitpick, every named Rust item and
  the Python paragraph are true at HEAD. No additional unlisted headline facade
  break was found.
- **Contributor credit.** Unchanged since pass 5. No link or handle moved.
- **Test gate.** `s73_release_contract_requires_four_version_aligned_families`,
  `stable_release_family_is_prepared_at_0_14_0`, and
  `incubating_release_family_is_prepared_at_0_12_0` in
  `scripts/test_sprint_workflow.py` pass under `unittest`. The workflow
  references in `.github/workflows/publish.yml:122` and
  `.github/workflows/publish.yml:126` name tests that exist at
  `scripts/test_sprint_workflow.py:5154`,
  `scripts/test_sprint_workflow.py:5571`, and
  `scripts/test_sprint_workflow.py:6230`.
- **HLD and plan consistency.** HLD 03, 10, 12, 14, and 15 and the plan agree
  on 0.14.0 and 0.12.0, the four tags, incubating-first publication order,
  and the superseded 0.13.2 crates.io train. `26cf1f7c` touches no HLD or plan
  file.
- **Version carriers.** Workspace version 0.14.0, 16 incubating pins at
  0.12.0, 17 explicit incubating manifests, both `pyproject.toml` files,
  README and `readme_doctests.py` literals, CI WASM literals, and the lockfile
  (17 entries at 0.12.0, 10 at 0.14.0) are consistent.
- **Voice rules.** `prose_check` reports 0 violations for the tree, and the
  `26cf1f7c` commit message passes `--commit-msg`.
