# F-263 microscope review, all categories, pass 2

**Reviewed:** working tree against `123f12daf445b74e4213c327d49330f7798c8df1`, 36 files, 1,450 insertions and 80 deletions
**Verdict:** 2 defects, 0 smells, 0 nitpicks

## Defects

### D1. Multilingual PAGEREF can publish its placeholder as the resolved cache

`crates/rdocx-layout/src/engine.rs:5078` substitutes computed fields only for `PositionedElement::Text`. The wildcard arm at line 5112 leaves `MultilingualText` untouched, even though directional computed fields enter multilingual shaping, as exercised at line 9598. `crates/rdocx/src/field.rs:9024` then accepts `MultilingualText`, and line 9036 parses its unchanged logical text as the target-page value. A directional `PAGEREF` whose stored cache is `99` can therefore be reported and written back as page 99 instead of the bookmark page. Add multilingual field substitution or derive the value independently of rendered placeholder text, with an RTL `PAGEREF` regression.

### D2. The pure-Rust generator boundary permits runtime conversion through a child process

`scripts/docx_authoring_conformance.py:526` rejects named internal and HTML entry points, but it does not reject `std::process::Command`, converter executables, or runtime input reads. A generator can retain the required `Document::new()` and `Document::open(&output)` calls while using LibreOffice, Pandoc, or another child process to create `output`, and the checks at lines 549 through 552 still pass. That makes the new no-fallback gate insensitive to the conversion path it is required to exclude. Harden the boundary and add negative self-tests for process execution and runtime source reads.

## Smells

None.

## Nitpicks

None.

## Not found

No additional correctness, panic-boundary, OOXML ordering, preservation, Python ownership, or structural findings were identified in this pass.
