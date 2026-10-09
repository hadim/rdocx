# F-X194, Complete Python table text and indentation APIs

**Status**: approved
**Sprint**: S91
**Size**: M
**Depends on**: none

## Problem

PowerPoint `PyCell` exposes text and cell formatting but no `text_frame`.
TextFrame, Paragraph, Run and Font resolution is restricted to shape text
bodies in `crates/rpptx-py/src/text.rs:159`, `:172`, `:454`, `:894` and
`:1148`. Adding only a getter would produce a handle whose operations fail
or address the wrong body. Native cell text frames already exist in
`crates/rpptx/src/lib.rs:8558` and `:8643`. This closes Issue295.

Word authored-indent inspection exists at
`crates/rdocx/src/table.rs:2168`, but the Python table has no indent property.
The checked native setter at `:798` calls the nonnegative checker at `:398`.
A signed checker already exists at `:411`. This closes Issue300, including
negative values and removal, without replacing Word's placement policy.

## Spec reference

- `docs/hld/10-bindings-spec.md`, "The chosen design", "The invalidation
  problem, handled loudly", and "Python API shape", specifically the table
  and PowerPoint TextFrame, Paragraph and Font contracts.
- `docs/hld/05-drawingml-model.md`, "Text body" and "Tables".
- `docs/hld/08-rendering-spec.md`, "Tables", specifically Word alignment,
  margins, compatibility behavior and floating-table placement.
- `docs/hld/01-glossary.md`, units.
- `docs/hld/14-development-backlog.md`, "F-X194, Complete Python table text
  and indentation APIs".

## Approach

Extend existing types and files. Add `Cell.text_frame -> TextFrame` to the
PowerPoint binding. Capture its existing cell path including Row and Cell.
Retrieving the handle validates ownership and revision without creating a text
body, mutating source or advancing revision.

Add two concrete resolution functions in existing `text.rs`:

```rust
fn text_frame_ref_at<'a>(
    presentation: &'a rpptx::Presentation,
    path: &ContentPath,
) -> Option<rpptx::TextFrameRef<'a>>;
fn text_frame_mut_at<'a>(
    presentation: &'a mut rpptx::Presentation,
    path: &ContentPath,
) -> Option<rpptx::TextFrame<'a>>;
```

They use existing cell resolvers for a complete Row/Cell address, otherwise
existing shape resolvers. Route every TextFrame, Paragraph, paragraph/run
collection, Run, Font and Hyperlink text-body resolution through them.
An explicit grid-cell path remains that cell's address, including a merge
continuation. Do not silently redirect it to a merge origin.

Make borrowed native handles lifetime-safe without retaining references to
temporary cell wrappers. Name the existing native cell implementation
lifetimes and expose these accessors alongside the existing mutable borrow:

```rust
impl<'a> TableCellRef<'a> {
    pub fn text_frame(&self) -> Option<TextFrameRef<'a>>;
}
impl<'a> TableCellMut<'a> {
    pub fn into_text_frame(self) -> TextFrame<'a>;
}
```

The consuming accessor follows `ShapeMut::into_text_frame` at
`crates/rpptx/src/lib.rs:8086`. Reuse all current public text-frame operations:
multiple paragraphs, runs, alignment, line spacing, spacing before/after,
font size/name/bold/colour, vertical_anchor and word_wrap. Anchor and wrapping
already exist at `crates/rpptx-py/src/text.rs:381` and `:415`, so a separate
cell anchor property is unnecessary. Retain existing hyperlink ownership
behavior while resolving its text properties through the cell body.

Structural changes advance the global revision once. Return new handles at
the new revision where the established API does so. Value-only formatting
retains live handles. Cell recovery paths already have the right shape at
`crates/rpptx-py/src/lib.rs:71`.

Use the existing signed twip checker for Word table indentation. Add:

```rust
pub fn set_indent_value(&mut self, length: Option<Length>) -> Result<()>;
```

`set_indent_checked` delegates to this checked optional setter. Validate
signed-i32 twip bounds before creating properties. Conversion truncates
toward zero. None removes direct tblInd only, and clearing an absent value
inserts no tblPr. Positive, zero and negative values remain explicit.

Add Python getter/setter in existing `PyTable`:

```rust
fn indent(&self, py: Python<'_>) -> PyResult<Option<Py<PyAny>>>;
fn set_indent(&self, py: Python<'_>, value: Option<i64>) -> PyResult<()>;
```

The public setter accepts Length, its established integer EMU equivalent,
or None. Read through TableRef::indent and write through PyTable.edit.
Invalid input fails before source mutation. Existing left/center/right,
margin, compatibility and floating-table placement behavior remains the
only rendering policy.

## Rejected alternatives

- Separate cell paragraph/font wrappers duplicate existing public objects.
- A shape-backed frame path does not identify cell text ownership.
- Clamping negative indent violates Issue300's explicit contract.
- New table positioning rules duplicate existing layout behavior.

## Test plan

**Test gate**: regression, copied from the backlog.

| Category | Test | Asserts |
|---|---|---|
| regression | `test_table_cells_expose_the_same_editable_text_frame_surface_as_shapes` | Two paragraphs and runs, font size/name/bold/colour, alignment, spacing, anchor and wrapping survive save/reopen while the table remains editable. |
| regression | `test_cell_text_frame_edits_preserve_merge_origin_and_continuation_ownership` | Origin visible content, continuation addressing and unrelated cells preserve explicit grid-cell ownership. |
| regression | `test_cell_text_frame_structural_edits_stale_every_earlier_nested_handle` | Frame, collections, paragraph, run and font stale after successful structure changes, formatting alone retains validity. |
| regression | `test_cell_text_frame_edits_reach_pdf_and_raster_output` | Actual deterministic PDF and PNG show multi-paragraph cell edits and formatting. |
| regression | `test_table_indent_accepts_negative_zero_positive_and_none` | Length inspection, signed values, truncation and direct indent removal survive save/reopen. |
| regression | `test_invalid_table_indent_leaves_source_and_handles_unchanged` | Wrong types and signed twip overflow fail atomically without advancing revision. |
| integration | `checked_table_indentation_accepts_signed_twips_and_removes_direct_indent` | Native checked signed authoring, no-op clearing and source round trip. |
| regression | `signed_table_indentation_keeps_existing_alignment_margin_and_float_placement` | Actual layout/PDF placement controls for existing alignment, margins, compatibility modes and floating-table precedence. |

Use existing `crates/rpptx-py/tests/test_documented_examples.py` for the
PowerPoint tests, `crates/rdocx-py/tests/test_formatting_tables.py` for Python
Word tests, and existing `crates/rdocx/tests/integration_test.rs` and
`regression_test.rs` for native evidence. No new integration binary. Include
grouped-shape table ownership and missing optional cell-body read controls.
Update both existing typing smoke files and stubs. Reproduce both missing
APIs against the claimed base before implementation.

## HLD impact

- `docs/hld/10-bindings-spec.md`: Python cell-backed TextFrame ownership,
  complete reused surface, native consuming accessor and signed table indent
  authoring/removal. No HLD08 placement change is intended.

## Risk routing

- Unit conversion: read HLD01 units and CLAUDE's truncation contract. Check
  signed bounds and truncation toward zero. Do not round lengths.
- Public native API: the lifetime-safe consuming cell accessor and optional
  indent setter are additive pre-1.0 API changes. Run applicable
  `cargo publish --dry-run` and `.crate` size assertions for changed published
  facade packages. The change to checked negative-indent rejection is
  intentional and stated explicitly.
- PyO3: read HLD10. Run installed Python runtime, typing and stub checks.
  Run `cargo check --target wasm32-unknown-unknown -p rdocx-wasm -p rpptx-wasm`.
  Workspace test runs exclude rdocx-py and rpptx-py.
- Rendering controls: read HLD08 and use deterministic fonts for every
  PDF/raster baseline. No baseline recording against system fonts.
- No new file, module, crate, trait, generic parameter or feature flag.

## Hash harness

Expected unchanged. New authoring entrypoints do not change existing fixture
inputs or layout rules. An unexplained delta blocks completion.

## Implementation checklist

- [ ] Reproduce both missing APIs on the claimed base.
- [ ] Add native lifetime-safe accessors and cell-aware text resolvers.
- [ ] Route every nested text and font operation through those resolvers.
- [ ] Add Python cell text_frame and preserve revisions/ownership.
- [ ] Add signed optional checked Word table indent and Python property.
- [ ] Update existing stubs, examples and typing smoke.
- [ ] Run installed Python/native regressions and risk riders.
- [ ] Complete scoped verify and a zero-finding microscope.

Exclusive implementation paths are the existing native `rpptx/src/lib.rs`,
`rdocx/src/table.rs`, both families' binding table/text files, existing Python
stubs/tests and native Word integration/regression entrypoints. F-X197 must
follow this story for overlapping PowerPoint text/font paths.

## Open questions

None. Preserve existing explicit-grid-cell merge addressing and strict global
revision invalidation. Both decisions follow the established binding model.
