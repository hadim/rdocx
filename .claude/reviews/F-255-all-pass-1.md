# F-255, all, pass 1

**Reviewed**: exact uncommitted implementation diff against claim base
`53c81086c95be671ea2f75bd14c31cab893409c0` on `work/f-255-codex`, excluding
this review artifact, 2 files, 783 insertions and 55 deletions
**Verdict**: 2 defects, 0 smells, 0 nitpicks

## Defects

### D1, footnote story insertion publishes stale typed footnotes
`crates/rdocx/src/document.rs:10335`

Picture and hyperlink insertion validate a reopened clone, discard it, and
publish the prepared candidate. For a footnote owner, the shared XML setter
updates only the package part and marks the old typed footnotes clean
(`crates/rdocx/src/document.rs:4826`). The published document therefore has
the new paragraph in package XML but not in `Document::footnotes`. A later
typed `add_footnote` mutation marks that stale model dirty
(`crates/rdocx/src/document.rs:10810`) and flushes it over the package part,
silently deleting the picture or hyperlink that this API reported as
successful. Commit the successfully reopened candidate, or synchronize the
typed footnote model before publishing. Add a regression that inserts into an
ordinary footnote, performs `add_footnote`, saves and reopens, then proves both
the inserted relationship-bearing paragraph and the new typed footnote remain.
Exercise both picture and hyperlink paths because they duplicate the discarded
reopen pattern at `crates/rdocx/src/document.rs:10365`.

### D2, retired drawing ids remain occupied after header replacement
`crates/rdocx/src/document.rs:8508`

Global drawing canonicalization now starts from every id ever reserved in the
facade. It removes ids only for drawings still present in the current typed
body or current fully authored story XML. Header replacement prunes the old
relationship and media part but never retires the removed drawing id
(`crates/rdocx/src/document.rs:12235`). Replacing an authored image header
therefore leaves its old id occupied, allocates a second provisional id, and
canonicalizes the sole final header drawing to `2`. Constructing the same final
header directly canonicalizes it to `1`. This makes output depend on mutation
history and leaks ids on every replacement. Seed the occupied set from
preserved ids plus drawings that will not be rewritten, rather than the full
historical registry. Add a sensitivity test comparing bytes from one direct
`set_header_image` call with two calls whose final image, filename and geometry
are identical, and assert the final `wp:docPr` id is `1` in both. The current
legacy integration performs only one header assignment
(`crates/rdocx/tests/regression_test.rs:464`).

## Smells

None.

## Nitpicks

None.

## Not found

No additional defects were found in custom story part targets, part-local
relationship allocation, identical local ids across independent owners, body
and text-box shared scope, cell ownership inheritance, stale and wrong-kind
story rejection, wrong relationship type or mode, missing targets, external
image or internal hyperlink rejection, relationship remapping, media target
ordering and pruning outside D2, schema insertion boundaries, fixed-prefix
serialization, aliased namespace reads, retained raw subtree preservation, or
failure atomicity. The staged package bytes used by the new insertion APIs are
the bytes validated by reopen. D1 is specifically the facade-state difference
caused by discarding that reopened value. The public Rust API additions are
additive for the pre-1.0 crate, and no new trait, generic, module, file, wrapper,
or panic surface was introduced.

The three named focused tests passed individually. The implementation diff
passes `git diff --check`. Progress records the full regression binary at 430
passed and 4 ignored, scoped check, clippy, formatting, hash harness, prose and
skill-sync gates as passing. The package dry run produced 36 files and 866826
bytes, below 10 MiB. Its verification build reached the known older published
sibling-crate mismatch described in the progress checkpoint.
