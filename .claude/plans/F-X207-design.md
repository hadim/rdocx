# F-X207, Persist PowerPoint autofit and implement fit_text

**Status**: draft
**Sprint**: S91
**Size**: L
**Depends on**: F-X194, F-X195

## Problem

Issue310 asks for stored fit results after template filling, not just a renderer that temporarily shrinks text. Native TextFrameMut::set_autofit_mode at crates/rpptx/src/lib.rs:9098 keeps or creates a bare NormalAutofit flag and ShapeAutofit changes no extent. Existing F-100 is done and correctly renders stored values and trusts stored shape extents. This story adds explicit authoring operations without silently mutating during rendering or ordinary saves.

ShapeTextLayout at crates/rpptx-render/src/lib.rs:214 already reports font_scale, usable bounds, text height and overflow from the shared renderer. It does not report computed line-space reduction. Bare autofit at crates/rpptx-render/src/text.rs:474 uses the current 2.5 percent ladder with no generated spacing reduction. Existing TextFrame and ShapeMut borrow only their text body or shape child, so they cannot independently resolve the owning deck's theme, inheritance and actual fonts. Python PyTextFrame at crates/rpptx-py/src/text.rs:133 retains presentation ownership and ContentPath, which permits delegation to a presentation-owned operation.

## Spec reference

- docs/hld/14-development-backlog.md, F-X207, existing F-100, F-X194, F-X195 and F-X204 ownership.
- docs/hld/08-rendering-spec.md, Text in a shape, Autofit and Slide text layout as data.
- docs/hld/06-presentationml-model.md, Public facade, The shape tree, Placeholders and Preservation strategy.
- docs/hld/07-inheritance-and-resolution.md, The chains: Position and size, Body properties, The nine-level list style and Fonts.
- docs/hld/10-bindings-spec.md, Python API shape and Native PowerPoint text layout.
- docs/hld/12-testing-strategy.md, Test taxonomy, Binding tests and the render fidelity gate.
- docs/hld/01-glossary.md, units, and docs/hld/15-build-and-toolchain.md, Deterministic rendering and Packaging.

## Approach

Use F-X195's exact caller-font preparation and F-X194's owner/handle conventions. Add presentation-owned concrete operations in existing rpptx/lib.rs rather than give a borrowed TextFrame a hidden global context. Proposed native signatures are:

    Presentation::refresh_autofit(&mut self, font_files: &[(&str, &[u8])])
        -> Result<AutofitRefreshResult>
    Presentation::fit_text(&mut self, slide_index: usize, shape_id: u32,
        font_family: Option<&str>, max_size: u16,
        font_files: &[(&str, &[u8])]) -> Result<AutofitUpdate>

AutofitRefreshResult contains concrete updates and owned diagnostics. Each AutofitUpdate records stable slide/shape identity, mode, stored scale/reduction and old/new bounds or the uniform written point size. These values have actual native, installed Python and F-X204 CLI consumers. No trait, builder, dynamic dispatch, new module or new dependency is needed. Keep the exact report fields aligned with existing LayoutResult diagnostics and ShapeTextLayout fields so inspection does not become a second layout engine.

Expose Python Presentation.refresh_autofit(*, fonts=None, font_dir=None) and TextFrame.fit_text(font_family=None, max_size=18, *, fonts=None, font_dir=None). Preserve existing auto_size setters. Explicit refresh is the chosen issue-supported save route, so ordinary save calls and read-only render/layout calls remain unchanged. Document calling refresh after template edits and before save. Do not add an implicit save hook or a second independently computed refresh flag unless explicitly requested during alignment.

At operation start validate selected owner identities, font inputs and numeric bounds. Resolve an immutable staged snapshot with the same caller faces used by PDF/PNG and fit reports. Prepare a concrete list of validated mutations on a clone. Commit all changed parts only after every target measurement, range check and serialization succeeds. Invalid directories, unsupported content, duplicate shape IDs, missing owners, nonpositive usable bounds, unrepresentable coordinates or an unfit floor must not leave a partly resized deck. Return the stable frame identity and reason on refusal. Caller font bytes are copied before detaching Python work, and mutation commits follow existing revision/handle policy.

For TEXT_TO_FIT_SHAPE recompute against current edited contents even when a previous stored value exists. Keep ordinary rendering's stored-value trust. Reuse the shared shaping, stacking and fitting code, including the same usable box, bullets, fields, paragraph spacing, wrap rules and 0.01-point fit tolerance. Store canonical integer percent values in existing NormalAutofit.font_scale and line_spacing_reduction at crates/oxml-drawing/src/text/body.rs:225. Candidate font scales use exact integer steps100000,97500,...,25000 rather than accidentally changing unit conversion policy.

The line-space candidate algorithm is not yet established by the existing implementation. Capture actual pinned PowerPoint frames before choosing it. Measure both stored fontScale and lnSpcReduction after native recomputation on controlled frames with explicit font bytes/version, insets, line spacing, multiple paragraphs and known height. Use those observations to determine whether the current ladder plus zero reduction is sufficient, or whether native PowerPoint searches coupled scale/reduction candidates. Do not invent lnSpcReduction as1-minus-scale, copy a layout result that has no reduction field, or claim native parity from hand calculations alone. Any required change to bare normal-autofit computation is explicitly reviewed and declared as rendering behavior. Add the selected reduction to the shared concrete measurement result and feed the same pair into rendering and persistence. A mismatching oracle leaves a noncompleted acceptance barrier until triaged and approved.

For SHAPE_TO_FIT_TEXT measure at the current usable width and effective run sizes, then update only the required frame extent and offset. Include top/bottom insets and paragraph leading/trailing rules. Preserve the physical anchor represented by existing top/center/bottom alignment. For unrotated vertical sizing, top holds old top, bottom holds old bottom, and center holds old center. With rotation, flips, vertical text or group transforms, preserve the corresponding anchor point in shape-local coordinates and transform the size/offset change through the existing group/shape affine model. Do not apply an axis-aligned shortcut that drifts rotated frames. Use existing EMU constructors and checked arithmetic, preserving truncation. Allow both growth and shrinkage. Do not resize the stored source just because a render happened.

fit_text chooses the largest supported integer point size not exceeding max_size that fits the fixed frame. Trials use the same actual fonts and layout path. Proposed font_family=None behavior preserves resolved per-run families and bold/italic styles, while an explicit family sets that family on affected text. This is a deliberate documented extension to python-pptx's defaultCalibri behavior and needs alignment. Apply one chosen size to direct text runs, field runs and relevant paragraph defaults/end properties without replacing text or losing hyperlinks, unknown properties or order. Set wrap on and explicit no-autofit after successful materialization, matching the pinned python-pptx operation's stored outcome. Empty text is a no-op. If no permitted size fits, refuse atomically and report overflow rather than store a falsely successful result. Validate the maximum against existing DrawingML font-size limits before trial shaping.

Refresh must enumerate recursively owned slide text frames with effective inherited flags, including populated placeholders and nested groups. Materialize inherited answers on the actual owning frame, not by modifying a shared master merely because a slide inherited its flag. Current text_layout_deterministic at rpptx/lib.rs:1199 omits master/layout shapes, SmartArt, table cells and empty frames. Do not reuse that filtered list and silently call it every flagged frame. The location boundary for note frames, directly owned master/layout text and table cells must be approved explicitly. If an unsupported flagged frame is encountered, return a named refusal or diagnostic and do not claim complete issue coverage. Shared owners instantiated in differing slide contexts require an explicit policy before writing one global extent or scale. F-X209 owns general master/layout authoring, so coordinate shared-owner mutation semantics instead of implementing that roadmap twice.

Fallback measurements retain requested and actual family information from F-X195/F-X210's shared selection facts. Reports must identify approximations and must not imply that a missing requested face was used. The same measurements and result schema are consumed by the F-X204 CLI fit owner. F-X207 does not add the CLI command or a competing CLI-only calculation.

## Rejected alternatives

- Mutate fitted values during read-only rendering or ordinary saves.
- Invent coupled line-spacing reduction without pinned native evidence.
- Duplicate CLI fitting instead of exposing the same Python-visible API.

## Test plan

**Test gate**: regression.

- Prove four public missing behaviors on the claimed Base through installed APIs: bare normal flags save without computed attributes, shape autofit saves without changed bounds, fit_text is absent, and refresh_autofit is absent. Existing render-time scaling is a positive control, not the missing stored-operation gate.
- Generate a small deterministic native PowerPoint oracle set and pin app/build, platform, source package SHA, actual font bytes/version, exact recomputation action and output package SHA. Extract native scale/reduction/extent numerically. Compare stored values exactly when quantization agrees, with explicitly reviewed unit/coordinate tolerances otherwise. python-pptx1.0.2 is a structural fit_text oracle, not an autofit-recomputation oracle.
- Retain the existing render tests for62500 scale, stored line-space reduction on percentages but not exact point spacing, no-autofit overflow, bare ladder and trusted spAutoFit extent. They must remain correct outside explicit authoring calls.
- Verify save/reopen retains computed normAutofit attributes, no duplicate autofit children, correct schema order and all genuine unknown XML bytes/namespaces. Namespace aliases and rebound/foreign children must not acquire typed semantics.
- Verify shape growth/shrinkage for top/center/bottom anchors, insets, wrapped and unwrapped text, explicit breaks, mixed run styles, bullets, fields, empty paragraphs, rotations/flips, vertical text and nested group transforms. Assert actual physical anchor coordinates and PDF/PNG containment.
- Verify fit_text stores the largest permitted size, candidate-plus-one overflows, max_size cap, explicit family selection, None-family policy, wrap-on/no-autofit result and preserved colors, bold/italic, links and text. Differential checks use pinned python-pptx1.0.2 where semantics intentionally coincide, with documented divergence elsewhere.
- Verify deck-wide refresh after repeated template fills, inherited flags, idempotence, stable identities, warm/fresh equality and original bytes on malformed/unsupported/atomic refusals. Include every approved location class and explicit unsupported-location reporting.
- Caller-only font controls must make refresh, text-layout reports and rendered output agree. Missing requested families produce the same approximation diagnostics. Use actual variable700 instances from F-X195, not regular outlines with bold labels.
- Add tests to existing rpptx integration.rs, existing rpptx-render text/lib test modules and rpptx-py test_documented_examples.py and typing_smoke.py. No new test binary or tracked binary fixture. Installed runtime, strict typing and stubtest cover both new Python methods and returned snapshots.
- Run scoped changed-crate fmt, lint, tests, policy and deterministic hash gates plus exact native oracles, both WASM target checks, verified patched affected-package dry runs, archive limits and actual updated measurements. Integrated full verification remains the sprint gate.

## HLD impact

- docs/hld/06-presentationml-model.md
- docs/hld/08-rendering-spec.md
- docs/hld/10-bindings-spec.md
- docs/hld/14-development-backlog.md

HLD08 must distinguish explicit authoring refresh from unchanged render-time trust, and state the approved scale/reduction algorithm, owner/location and anchor policies. HLD10 documents concrete native and installed Python surfaces. Do not weaken the existing stored-value rendering contract to implement refresh.

## Risk routing

- Layout/shaping requires deterministic font bytes and measured shared-render containment controls. Read HLD08.
- Unit conversion requires checked range and frame-anchor controls while retaining existing truncating constructors. Read HLD01 and CLAUDE.md Things that are deliberately wrong.
- OOXML serialization requires prefix-tolerant read, canonical write, xsd:sequence and exact unmodelled XML round trips. Read HLD04 and HLD06.
- Published native API/report changes require stated additive or source-breaking semver impact, affected package verified dry-runs with the entire local patch graph, source membership and10MiB bounds. A public ShapeTextLayout field addition is pre-1.0 source-breaking for external literals and must be stated if selected.
- Python/WASM requires installed runtime, typing/stubtest and both wasm checks. Workspace Rust tests exclude both extension-module binding crates.
- External oracle riders require exact PowerPoint app/build and recorded recomputation, deterministic font inputs and pinned python-pptx1.0.2. No unobserved native pass claims.
- No new dependency, trait, generic, module, crate, feature flag or implementation file is proposed. A public concrete report has named current native/Python/CLI consumers and no builder.

## Hash harness

Ordinary rendering and save without explicit refresh should remain unchanged. Authored refresh/fit output intentionally differs from input when it writes scales, reductions, run sizes or bounds. If native evidence requires correcting the bare renderer's candidate algorithm, enumerate and independently review each actual deterministic harness delta before the exclusive baseline moves. Preserve attribution from F-X195 and other stories rather than blend their font changes into this story's delta.

## Implementation checklist

- [ ] Align explicit refresh route, location/owner policy and fit_text None-family semantics.
- [ ] Pin and capture native scale/reduction/anchor ground truth before choosing the missing algorithm.
- [ ] Prove all public Base failures and retain positive render controls.
- [ ] Reuse caller-font and owner infrastructure from prerequisites.
- [ ] Implement shared fitting results, atomic materialization and anchored resizing.
- [ ] Add installed Python methods and exact typing/runtime snapshot surfaces.
- [ ] Verify all approved location classes, refusal controls and save/reopen preservation.
- [ ] Complete scoped/oracle/packaging/WASM gates and independent zero-finding review.
- [ ] Declare actual harness behavior and update HLD impact files.

## Open questions

1. Approve explicit refresh_autofit before save as the issue-supported refresh route, with no implicit save mutation.
2. Confirm fit_text(None) preserves effective run families/styles and uses a uniform largest integer point size, or specify the desired python-pptx default-family behavior.
3. Which native PowerPoint build and immutable controlled frames establish the real coupled scale/line-space algorithm? No formula is approved before this observation.
4. Define every-frame scope for table cells, notes and shared layout/master owners. Current slide-layout report exclusions cannot silently satisfy that phrase.
5. For directly owned shared frames with different effective contexts, approve a conservative global result or explicit refusal/materialization policy before changing owner XML. General master/layout authoring remains F-X209-owned.

## Qualified autofit owner contract

## Inputs and concrete location

Reuse exactly X195 font preparation once per operation. Native input is font_files:&[(&str,&[u8])], and Python refresh_autofit and TextFrame.fit_text accept fonts=None, font_dir=None and font_dirs=None. Ordered directories, lexical file order, later same-rank precedence, explicit fonts last, embedded family/style identity, caller-only versus bundled fallback and invalid-directory failures remain the one X195 rule. Do not create a second fit loader or omit font_dirs now that313 is included. Copy caller byte ownership before detached work. All measurement trials use one prepared font selection, including genuine variable weight/italic instances and requested/actual family diagnostics.

Use a concrete frame location, not only slide_index+shape_id. Proposed existing-file native type `TextFrameLocation` carries owner_part:String, child_path:Vec<usize>, cell:Option<(usize,usize)> and checked owner fingerprint. Its constructor is private and inventory returns it. Context-specific reports also carry slide_index:Option<usize>, owner_kind and instantiated context identity. The source child ordinal path identifies group descendants, and cell coordinates identify a table text body. Shape ID is report metadata, not sole mutation identity. Duplicate IDs or failed owner correspondence are checked errors. No new generic path engine, source file or trait.

Proposed native signatures:

- Presentation::refresh_autofit(&mut self,font_files:&[(&str,&[u8])])->Result<AutofitRefreshResult>.
- Presentation::fit_text_at(&mut self,location:&TextFrameLocation,font_family:Option<&str>,max_size:u16,font_files:&[(&str,&[u8])])->Result<AutofitUpdate>.
- Presentation::fit_report(&self,font_files:&[(&str,&[u8])])->Result<FitReport>.

Python TextFrame.fit_text delegates its existing checked ContentPath to the concrete native location. Keep the initially proposed native slide-index/shape-ID method only as an optional convenience if it resolves exactly one same-owner frame and actually has another consumer. Do not add a forwarding wrapper merely to preserve a draft signature. Native location fields/checks can instead reuse a reviewed existing X206 owner locator if it is integrated by claim time. The draft X206 explicitly says no stable-ID resolver yet, plan:195, so that provider is not assumed implemented.

## Inventory and every-frame accounting

Inventory scans qualified typed owner parts in stable package ownership order, then shape-tree child/source order. Include ordinary shapes, populated placeholders, nested groups and table-cell text bodies. Inventory includes empty text and flag presence, not only visible characters. Existing text_layout_deterministic at rpptx/lib.rs:1199 omits master/layout shapes, SmartArt, table cells and empty frames, so it is not this inventory.

One physical owned text body is one mutation target. Slide-local inherited body flags are resolved through actual layout/master context, then their computed answer is materialized on that slide-local owner. Never change a shared master merely because a slide inherits a flag. Directly owned master/layout frames with flags are inventoried separately, along with every consuming slide context. Notes-slide and notes-master frame owners are inventoried separately using the existing notes resolution/theme/page geometry, not slide geometry. SmartArt/opaque unsupported text ownership is recorded with a named unsupported location, not ignored as though it had no flag.

For a directly shared master/layout/notes-master frame, evaluate every applicable context before mutation. Normal shrink may use only a common candidate pair that passes every context. Choose the least shrinking candidate in the approved candidate order satisfying their intersection. Shared shape-to-fit resize may be written only when required local bounds and anchor adjustment are identical in every context after canonical EMU quantization. If they differ, refuse the entire refresh with owner/context details, rather than select a global arbitrary size or duplicate shared XML into slide-local shapes. This is a conservative exact ownership rule, not an optimization based on similar rendered text. An unused shared owner without a resolvable context receives an explicit not-measured diagnostic and cannot count toward a complete every-frame acceptance until an owner-context policy is approved.

Table-cell normal shrink and uniform fit_text use the actual shared cell layout box and resolved paragraph style. Shape-to-fit flag on a table cell has no independently owned shape extent. Refuse with a named unsupported-cell-size policy until a reviewed row/table geometry transaction exists, rather than changing just text box height or falsely claiming one cell changed its enclosing row correctly. This is a real every-frame acceptance barrier if any required imported flagged cell uses that mode. Direct X209 master/layout authoring is not reimplemented here.

Empty flagged frames are accounted for with status='empty', no sizing/text mutation and no fabricated lines or zero scale. Their existing flag/raw properties remain. Empty fit_text is a no-op. Invalid retained XML, nonpositive usable box, duplicate IDs, unsupported flagged owner or unfit floor returns an atomic refresh error that names its physical location. A diagnostic-only report may enumerate unsupported owners, but successful refresh cannot claim it measured them.

## Fitting facts and thresholds

Use the existing shared shaping/line breaking/stacking/inheritance/theme and page-field value path, with width_factor fixed1.0 for authoring and fit report. Reuse 0.01-point acceptance tolerance from StackedText::fits at rpptx-render/text.rs:439 and ink-width checks at666. Both width and height must pass. Do not fit by height alone, estimate font width or count characters. Display rotations and text directions use their actual reading frame, ShapeTextLayout documentation at rpptx-render/lib.rs:214. Retain glyph ink handling, bullet extents, field values, indentation, insets and exact paragraph spacing.

For fit_text, max_size is a positive integer in the existing DrawingML size range, with explicit Python bool/nonintegral/overflow rejection. Validate against the actual native size setter before candidate trials rather than assuming u16 proves all schema bounds. Try each permitted integer point size descending from max_size through1. Choose the first that passes the same fixed-frame test. Exhaustive bounded descending search avoids an unproved monotonic binary-search assumption caused by line-wrap changes. Candidate-plus-one must fail unless the max cap selected the result.

font_family=None preserves effective per-run family and bold/italic while writing one chosen point size. Explicit family writes that family on affected native direct runs, fields and paragraph/default/end properties without replacing text. Preserve color, hyperlink, unknown attributes, ordered text choices and original XML owners. Set wrap on and explicit no-autofit only after success. The method is formatting and geometry, not text replacement. Unfit-at1 refuses with original bytes unchanged. Native python-pptx comparison only applies to deliberately matching explicit-family/default and stored-shape semantics, not automatic claim of identical None-family behavior.

Normal-autofit recomputation ignores previous stored scale/reduction for explicit refresh, but ordinary renderer continues to trust them. Current bare computation in text.rs:477 searches1.0 then0.975...0.25 with reductionNone. Canonical candidate scale lattice is integer100000,97500,...,25000, avoiding floating serialization. Its adequacy and coupled reduction order are NOT established as native PowerPoint steps by current source. That remains the exact native oracle question below.

Store a selected approved pair as decimal integer strings fontScale and lnSpcReduction in NormalAutofit, oxml-drawing/text/body.rs:225. Both values are present for a successfully recomputed nonempty frame, including100000/0 when unshrunk. Couple measurement and storage from the same concrete result. No formula lnSpcReduction=1-fontScale is assumed. Existing stored reduction applies to percentage/default spacing but not exact-point spacing, text.rs:930. Generated coupled candidates must preserve this existing stored-value interpretation. If floor still overflows, mutation refuses rather than storing a falsely successful25-percent pair.

For SHAPE_TO_FIT_TEXT, measure at current effective text sizes, usable width, wrap and paragraph settings, ignoring renderer-side shrink. New local height is measured content extent plus actual insets and supported leading/trailing rules. Allow growth and shrinkage. Preserve local top/bottom/center anchor exactly, transform through existing shape/group affine conversion and retain rotation/flip/vertical direction. Canonical EMU truncation stays unchanged. No axis-aligned world-space shortcut for rotated/grouped shapes. If inverse transform or coordinates cannot be represented, refuse before any mutation. Native oracle must verify anchor facts and group/rotation controls separately from the coupled shrink study.

## Reports and mutation generations

AutofitUpdate includes checked location, owner/context, shape ID/name metadata, mode, changed/status, old/new local bounds and canonical scale/reduction or written uniform point size, plus ordered requested/actual font diagnostics. AutofitRefreshResult contains updates in physical inventory order and diagnostics. Same-size/same-attributes refresh is idempotent and publishes no mutation. One genuinely changed deck-wide refresh is one staged package transaction, serialized/reopened before publication. A late failure changes no part or generation.

Use X194/X206 handle partition: formatting and extent changes do not alter indexed slide/shape/table/text topology, so no structural or text-topology revision bump is justified for successful refresh or fit_text alone. Existing Slide/Shape/TextFrame/Paragraph/Run handles remain live when their logical paths are unchanged. Invalidate native completed layout cache once after successful formatting/geometry publication. Do not bump text_revisions merely because font sizes were written. If implementation actually creates/removes logical paragraph/run owners to fit, reject that approach or explicitly revise its contract instead of silently applying blanket staleness. Immutable report snapshots remain historical values and are not mutation selectors.

Read-only FitReport returns frames in instantiated slide/notes context order with their physical location, current measured scale/spacing, overflow, hypothetical needed_scale/reduction, bounds, status and diagnostics. Needed means a total hypothetical scale relative to original effective run sizes, not an additional ratio multiplied by current stored scale. It uses the same approved solver as refresh without persisting it. Unfit-at-approved-floor returns needed_scale=None with reason and remains overflow. Unsupported owners are explicit unmeasured records. F-X204 CLI consumes that common provider and must not turn unsupported into successful fit0 or run a second solver.

## Smallest coupled PowerPoint oracle study

Prepare one controlled slide with four otherwise identical fixed-size unrotated inline text boxes, explicit family/style/point size, no bullets, no theme-dependent fonts, same width/insets, and fresh bare normAutofit. Use a real installed native font whose exact original face bytes/version/hash are recorded and feed those same bytes to rdocx/rpptx measurement. No font install or UI capture is implied by this research task.

- A: several explicit text lines in one paragraph with100-percent/default line spacing, height chosen to overflow slightly at full size.
- B: identical text and bounds with exact point line spacing equal to the initial nominal spacing.
- C: identical text/bounds with150-percent spacing.
- D: identical text and default spacing but split into multiple paragraphs with explicit before/after spacing.

A/B discriminates whether native writes/uses a line-spacing reduction distinct from font shrink, because the current renderer intentionally does not reduce exact point spacing. C discriminates proportional reduction, absolute percent-point reduction or a scale-only answer. D discriminates reducing line spacing versus paragraph spacing. Fixed width and explicit breaks isolate vertical shrink. No wrap uncertainty should be introduced into this first study. Avoid a hand-calculated expected pair.

Capture actual pinned PowerPoint build, generated source SHA, app-selected fonts and exact native edit action that triggers recomputation, then save. Extract each fontScale/lnSpcReduction, actual extents/insets/spacing and complete owner XML from the saved package. Close/reopen/save without editing, capture another SHA and compare the exact values/owners. Merely opening/saving a bare flag is not evidence of recomputation. Do not manually author the expected attributes into the oracle output.

If A/B/C/D do not discriminate quantization, use a second two-height intervention on the same A text with all other inputs unchanged. The two heights should straddle an observed native step, determined from the first actual receipt. Only then choose a candidate lattice and coupling rule. Additional mixed-size/bullet/wrapped controls validate generalization after an actual rule has been identified. Finite observed matches do not prove arbitrary parity, so any remaining discrepancy stays an explicit acceptance barrier.

Separately, a three-box shape-autofit control shares text/size but differs top/center/bottom anchor. Record old/new local bounds and native anchor points. Add one rotated grouped frame only after unrotated anchor results are understood. This keeps coupling and coordinate diagnostics independent.

## Actual unresolved acceptance barriers

Native coupled scale/reduction algorithm is still unmeasured. Current F100 ladder with zero reduction cannot be asserted as the requested PowerPoint algorithm. Shared-context shape resize, unsupported cell/SmartArt/unused shared owners need actual admission and oracle evidence before every-frame completion. These are bounded provider design/evidence barriers, not user preference questions or authorization to approximate. Root can approve ownership/refusal policy, but no plan approval substitutes for required native ground truth.

Canonical read HEAD: `5caf36fd631fb32e2a00acb26b1625d0b99b92ca`. Source SHA receipts:

- `crates/rpptx/src/lib.rs`: `9e203e0e9e3fe71c8b0c9d070a1a0cffa6f757fbd849c88c46702ed910996b6b`.
- `crates/rpptx-render/src/text.rs`: `2a7570f8d9fcdbd1933625a4148ca4ae7d76f2f9773e1d25da821339043323d4`.
- `crates/rpptx-render/src/lib.rs`: `98f646dbb7e10806140b0079298165e79c71c3dcb571e03548fea23f223a641b`.
- `crates/oxml-drawing/src/text/body.rs`: `d9db6dd0a2dc2b61eaa7512c630ba7d33d27762eceb683f7e9a676f9541dc9cb`.
