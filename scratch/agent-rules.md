# Rules shared by the implementers of tensorbee/rdocx #403 and #407

S = /private/tmp/claude-501/-Users-hadim-Code-libs-rdocx/08057ee4-afc5-4751-aca1-2668752380de/scratchpad

## Repository

- Read `CLAUDE.md` and `AGENTS.md` of your worktree first. Their structural rules apply: smallest diff that fixes
  the cause, match the conventions of the file you edit, no new trait or generic without two users today.
- **No new file, module or crate.** If you think you need one, stop and say so in your report instead.
- Work only inside your worktree. Never touch `/Users/hadim/Code/libs/rdocx/rdocx` (the main checkout, on another
  branch) nor any other worktree under `/Users/hadim/Code/libs/rdocx/`.
- Never set `CARGO_TARGET_DIR`: the default `target/` inside your worktree is your own target dir. Never `cargo clean`.
- Do not push, do not open a PR, never rewrite history that is not yours. Commit on your branch only.
- Never use bare `git stash`; the stash stack is shared between worktrees. Use a WIP commit if you must set work aside.
- No Word or PowerPoint window ever. LibreOffice headless only, if at all.
- No em dash and no prose semicolon in tracked Markdown or commit messages. No `Co-Authored-By`, no attribution lines.

## Commits

Plain imperative subject, then a body that says what changed and why (see `git log fork/fix/337-nested-text-boxes`
in the main repo for the house style). The last commit of the branch is
`Re-record the <crate>, <crate> archive measurements`, on its own: every touched publishable crate listed in
`ARCHIVE_MEASUREMENTS` of `scripts/readme_doctests.py` must have its crates.io archive measurement re-recorded, or the
Docs CI job fails. Procedure: load `scripts/readme_doctests.py` as a module (register it in `sys.modules` before
`exec_module`, it uses dataclasses), call `build_package_archive(package)` and `archive_measurement(archive)` for each
crate, update its dict entry and the "Crates.io archive: <crate>" row of its README (root `README.md` for `rdocx`,
`crates/<crate>/README.md` otherwise), and repeat until stable since the README is an archive member. Run it without
`CARGO_TARGET_DIR` (the helpers look for `target/package` under the repo). Then run
`python3 scripts/readme_doctests.py` to confirm.

## Python builds

- One scratch venv per implementer, given in your brief. Build with
  `VIRTUAL_ENV=<venv> PATH=<venv>/bin:$PATH maturin develop --release --uv -m crates/<binding>/Cargo.toml` for timings,
  and without `--release` (debug, as CI does) for the binding pytest suite.
- At the end, delete the built `.so` from your worktree (`crates/<binding>/python/<pkg>/_<pkg>.abi3.so`) so that
  `git status` is clean.
- Baseline builds of main a57be365 (before any change) sit in `$S/baseline/{rdocx,rpptx}`; `$S/baseline-rdocx` and
  `$S/baseline-rpptx` hold one package each, for `PYTHONPATH` when a script needs the other library too.

## Timing protocol (two implementers share this Mac)

A timing is only valid when no compiler runs. Before any timed run (repro scripts, scaling test, profiles):
create the lock with `mkdir $S/timing.lock` (if it already exists, wait until it is gone), then check that
`pgrep -x rustc` prints nothing (wait otherwise), time, then `rmdir $S/timing.lock`. Before starting any cargo or
maturin build, check that `$S/timing.lock` does not exist, and wait while it does. Wait with the Monitor tool and an
until-loop, not with a foreground sleep. Keep the lock no longer than the timed run itself.

## Gates

- Tests of every crate you touch, with `RUST_MIN_STACK=8388608` (CI sets it; without it `regression_test` overflows).
  For rdocx, Poppler is not on PATH: run as `pixi exec -s poppler -- env RUST_MIN_STACK=8388608 cargo test -p rdocx
  --no-fail-fast`.
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`, `cargo fmt --all --check`,
  `python3 scripts/hash_harness.py --check`, `python3 scripts/prose_check.py`,
  `python3 scripts/sync_agent_skills.py --check`, `python3 scripts/readme_doctests.py`.
- The binding's pytest suite (`<venv>/bin/python -m pytest crates/<binding>/tests -q`) on a debug build.
- Failures that are this machine's environment only, not code (treat exactly these as environmental, anything else
  is real): rdocx lib `large_word_and_presentation_pdfs_preserve_logical_reading_order`,
  `svg::tests::fx195_variable_svg_paint_matches_independent_static_instances`, and
  `word_and_powerpoint_chart_pixels_are_identical` only when it panics at the rasterizer version assert (a panic at
  the `WORD_SHA256` assert is real); rdocx integration `odt_reader_matches_pinned_libreoffice_structure`,
  `public_authored_theme_and_fonts_match_pinned_word_resolution`,
  `sanitized_public_authoring_fixture_passes_every_conformance_stage`,
  `section_page_semantics_match_pinned_libreoffice_render`, `every_conditional_table_region_matches_word`; oxml-pdf
  `writer::tests::rotated_linear_gradient_renders_with_its_axis_rotated`; rpptx-layout
  `context::tests::all_corpus_preset_geometries_evaluate_or_fallback`, `all_corpus_slides_resolve_without_panics`,
  `corpus_slide_resolves_without_theme_references`; rpptx-cli `validate_rejects_corruption_and_accepts_the_pinned_corpus`
  (no local corpus); Python `test_python_docx_parity.py` if python-docx is missing, `test_rendering_threads.py`
  (pins pdfinfo 26.01.0). Compare against a run on the untouched base if in doubt.
- CI does not run the Rust `#[cfg(test)]` modules of `rdocx-py` and `rpptx-py` (they are excluded from the workspace
  test); it runs their pytest suites on a debug build. Put a guard where CI will run it.

## Report

Short and factual: the approach in a few lines, files touched, before/after numbers (with the timing protocol),
every gate with its result (environmental failures named), deviations from the brief, open points, and the commit
SHAs of the branch.
