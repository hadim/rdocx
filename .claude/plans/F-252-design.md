# F-252, Rich per-section headers and footers

**Status**: approved
**Sprint**: S72
**Size**: L
**Depends on**: F-250, F-253

## Problem

Current header and footer methods at `crates/rdocx/src/document.rs:7496`
replace text or an image only for the final section. Part installation then
updates `section_properties_mut` at `crates/rdocx/src/document.rs:8102`, so it
cannot express independent default, first, and even variants for every section
or expose their rich content through the common story model.

## Spec reference

- `docs/hld/02-scope-and-non-goals.md`, "Modern DOCX capability matrix",
  DOCX-017.
- `docs/hld/03-architecture.md`, "Facade conventions".
- `docs/hld/04-opc-and-packaging.md`, header and footer inheritance and package
  integrity.
- `docs/hld/08-rendering-spec.md`, Word header and footer selection.
- `docs/hld/14-development-backlog.md`, "F-252, Rich per-section headers and footers".

## Approach

Use the F-253 story identity and content operations for every header and footer
part. Add per-section variant lookup plus staged create, link, unlink, replace,
and remove operations for default, first, and even stories. Omitted references
inherit only the same variant from the preceding section. Linking reuses an
existing internal exact-type story relationship. Unlinking clones the effective
story into a new collision-safe part before mutation. Removal prunes only an
unreferenced facade-owned graph. Existing final-section text and image helpers
remain compatibility conveniences over this path.

## Rejected alternatives

- One document-global header object cannot represent Word inheritance.
- Copying only story XML would omit part-local relationships and media.
- Treating a missing variant as blank would break same-type inheritance.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| differential | `section_header_footer_variants_match_word_width_and_inheritance` | Every variant renders at the correct width and follows pinned Word inheritance. |
| round-trip | `rich_section_stories_survive_reopen_replace_and_unlink` | Paragraphs, tables, fields, links, images, drawings, and nested supported content survive all lifecycle operations. |
| regression | `removing_one_variant_retains_shared_and_inherited_stories` | Shared graphs remain reachable and unrelated variants are byte-identical. |

The story test gate is differential: every section variant renders at the
correct width and survives save, reopen, replacement, and inheritance changes.

## HLD impact

- `docs/hld/02-scope-and-non-goals.md`
- `docs/hld/03-architecture.md`
- `docs/hld/04-opc-and-packaging.md`
- `docs/hld/08-rendering-spec.md`
- `docs/hld/10-bindings-spec.md`
- `docs/hld/12-testing-strategy.md`
- `docs/hld/13-risks-and-open-questions.md`
- `docs/hld/14-development-backlog.md`

## Risk routing

- Any parser or serialiser. Read `docs/hld/04-opc-and-packaging.md` and
  `docs/hld/06-presentationml-model.md`. Prove fixed write prefixes, schema
  order, prefix-tolerant reads, and byte-exact retained subtrees.
- Layout and pagination. Read `docs/hld/08-rendering-spec.md`. Run deterministic
  header and footer rendering with bundled fonts.
- Public API of a published crate. Read `docs/hld/10-bindings-spec.md` and the
  structural rules in `CLAUDE.md`. State additive pre-1.0 semver impact, run
  `cargo publish --dry-run -p rdocx`, and assert the packaged crate size.
- External oracle comparison. Read `.claude/skills/differential-testing.md`.
  Pin and record the exact Word oracle version.

## Hash harness

Expected to be unchanged. Existing final-section convenience calls must emit
the same package graph and bytes.

## Implementation checklist

- [ ] Resolve effective same-type variants per section.
- [ ] Add staged create, link, unlink, replace, and remove operations.
- [ ] Route rich edits through the F-253 story content model.
- [ ] Clone and prune complete part-local relationship closures safely.
- [ ] Add inheritance, rich-content, render, and reopen tests.

## Open questions

None. Inheritance removes the direct reference. Removal installs an explicit
empty story. Unlink clones the effective inherited story and its complete
internal relationship closure. First-page creation enables `titlePg`, while
even-story creation leaves the document-wide setting to an explicit operation.
