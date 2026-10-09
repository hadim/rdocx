# F-X210, Extend metric-compatible fonts and bullet fallbacks

**Status**: draft
**Sprint**: S91
**Size**: L
**Depends on**: F-X195

## Problem

Issue313 repeats the caller-font and variable-instance work already owned by F-X195 for Issues296 and297. This story owns only the new coverage delta. Current mappings at crates/oxml-layout/src/font.rs:2117 substitute Liberation Serif for Georgia and Carlito for Segoe UI. Aptos has no explicit mapping or substitution message. The PowerPoint bullet mapper at crates/rpptx-render/src/text.rs:317 handles only F0B7 to U+2022. Word Symbol runs at crates/rdocx-layout/src/engine.rs:9647 still shape the original private-use character. Broader private-use mappings need font identity, not a universal codepoint substitution.

## Spec reference

- docs/hld/14-development-backlog.md, F-X210 and the existing F-X195 ownership.
- docs/hld/08-rendering-spec.md, Text in a shape, The PDF backend, The rasteriser and The SVG backend.
- docs/hld/10-bindings-spec.md, Python API shape, Native PowerPoint text layout and Native Word facade stability.
- docs/hld/15-build-and-toolchain.md, Deterministic rendering, Packaging and Dependency policy.
- docs/hld/12-testing-strategy.md, Test taxonomy, The mixed-script geometry golden gate and Binding tests.
- docs/hld/04-opc-and-packaging.md, The package, and docs/hld/06-presentationml-model.md, Preservation strategy.

## Approach

Wait for the reviewed F-X195 font-selection contract and reuse its owned instance identity, caller precedence, directory validation and renderer propagation. Do not implement a second resolver or repeat Issues296/297 acceptance under a second owner. Existing directory and explicit byte inputs remain supported.

Pin two genuine Gelasio variable faces, upright and italic, and suitable genuine Selawik regular and bold faces from their official upstream distributions. The inspected eight-file proposal and complete immutable hashes are recorded in /private/tmp/S91-F-X210-font-intake/eight-file-proposal.json. The total proposed tracked size is440844 bytes. Gelasio is pinned at commit9228e69b160d79f33950e026293f6e13ba9780d0. Selawik official release1.01 is pinned at tag commitb65ff9ce7c4673e75ec5c622924a6b6f61f2f876 and release archive SHA2563f62c51e05e3b5a1e6241cf92a371f0be2ea1183aa87b30718bbd40832a8d423. Record source format, face index, axes and available styles before copying any asset. Proposed core inventory is Gelasio-Regular.ttf and Gelasio-Italic.ttf, Selawik-Regular.ttf and Selawik-Bold.ttf, LICENSE-Gelasio, NOTICE-Gelasio, LICENSE-Selawik and NOTICE-Selawik under crates/oxml-layout/fonts. The user explicitly approved these exact eight files on2026-10-09. No further file approval is required for this inventory. Creation still waits for the F-X195 completion barrier and approved implementation design. Preserve original embedded family/style names and original license text. Do not imply that Selawik has a genuine italic face if the selected upstream distribution does not supply one. F-X195's verified synthesis policy handles unsupported styles.

Google Fonts metadata currently identifies Gelasio[wght].ttf and Gelasio-Italic[wght].ttf, range400 to700, sourced from SorkinType/Gelasio commit9228e69b160d79f33950e026293f6e13ba9780d0. Recheck the immutable binary source and license during implementation rather than rely on a mutable main URL. Upstream describes regular, bold, italic and bold-italic Georgia compatibility, while documenting repertoire and kerning limitations. Primary references are [Google Fonts metadata](https://github.com/google/fonts/blob/main/ofl/gelasio/METADATA.pb) and [Gelasio upstream](https://github.com/sorkintype/Gelasio). This is upstream intent, not our measured acceptance evidence.

Microsoft describes Selawik as a Segoe UI replacement and explicitly lists missing matching kerning and hinting in the [Selawik README](https://github.com/microsoft/Selawik). [Upstream issue 6](https://github.com/microsoft/Selawik/issues/6) also discusses unresolved letterform and metric differences. Therefore state measured compatibility for a pinned reference version and tested repertoire, not universal Segoe UI equivalence. Character advances, line metrics and shaped strings are separate measurements. A broad tolerance cannot hide pair-kerning or layout discrepancies. Record real divergences and a reviewed bounded acceptance rule before declaring completion. If available evidence cannot meet the approved criterion, retain a noncompleted barrier instead of calling the face exact.

Update the existing bundled inventory and map Georgia to Gelasio and Segoe UI to Selawik. Caller-supplied authentic faces still win. Keep every existing unrelated family, default CJK coordinate and coverage fallback unchanged. Aptos remains a documented approximate fallback, initially the existing Carlito generic choice. Emit a diagnostic that names Aptos, the selected actual face and the approximation whenever no caller or embedded Aptos face was selected. A supplied Aptos face must suppress that warning. Do not claim that Carlito matches Aptos metrics.

Use existing output Diagnostic.message at crates/oxml-layout/src/output.rs:402 and existing owned layout diagnostics. Record substitution facts within the manager's current layout lifecycle, deduplicated in stable first-use order. The selected output bundle owns the result. Do not add a global diagnostic registry. Word native owned layout and PowerPoint render_deterministic at crates/rpptx/src/lib.rs:1108 already expose LayoutResult. Preserve existing byte-only PDF and PNG return types. Coordinate any missing installed Python diagnostic projection with the existing binding/export owners rather than silently invent a new return shape. F-X207 consumes the same substitution facts in fit/refresh reports.

For common Symbol and Wingdings bullet characters, prefer a supplied genuine font that covers the original codepoint. Otherwise apply an explicit font-family-qualified PUA mapping to supported Unicode and shape through an existing bundled face with verified coverage. Put the common mapping in an existing neutral font source file and call it from existing Word and PowerPoint lowering. Preserve source XML and stored char codes. Unknown PUA remains unchanged with an honest missing-glyph diagnostic. Do not remap private-use content from unrelated fonts, arbitrary regular text, or Wingdings2/3 without their own verified table. Include bullet, small square, arrow and check-mark controls only after the exact original glyph is independently verified. The issue's ambiguous section-square description needs an exact family/codepoint before inclusion. The existing F-099 F0B7 control stays green.

Optional Google font pack decision: the refreshed Issue313 explicitly sets a font pack aside and directs callers to the existing caller-font path for ordinary licensed local directories now. Recommend a separate explicitly designed, hash-pinned archive distribution if a curated optional pack is approved. Do not add a new crate, network downloader, automatic download, runtime fetch, default font pack or dependency here. The current issue asks for a decision, not speculative package machinery. A requested distributed pack needs an exact named consumer, selected family/style inventory, license manifest, archive limits, transport/offline behavior and release ownership before implementation. Documentation must state which decision was made and which behavior actually exists.

Issue313 explicitly requests font_dirs= on the listed render and measurement methods. Amend F-X195's API/input-order contract once and add ordered plural support under that owner. Preserve font_dir compatibility and define directory-to-directory precedence and explicit-byte override before writing bindings. Do not create a duplicate loader under F-X210.

## Rejected alternatives

- Claim Selawik metric equality from upstream intent alone.
- Add a speculative font-pack crate or automatic network loader.
- Rewrite arbitrary private-use text without a verified family-specific mapping.

## Test plan

**Test gate**: regression.

- Add existing-entrypoint red tests proving Georgia and Segoe UI select current inaccurate substitutes, Aptos emits no requested approximation diagnostic, and additional verified PUA bullets remain missing before the patch.
- Compare Gelasio four requested styles with hash-pinned licensed Georgia references. Four Georgia files are available in this host's System/Library/Fonts/Supplemental directory, but references remain ignored local oracle inputs. Pin font versions, bytes, UPEM, HarfBuzz14.4.0 and exact shaping options. Do not redistribute Microsoft font files.
- Obtain a valid pinned licensed Segoe UI reference or authoritative numeric metric evidence before approving Selawik claims. A genuine private Segoe UI5.31 Regular WOFF was located in the installed PowerPoint OfflineFiles resources. It remains in place and is not redistributed. Its SHA256 is d79ad533adf61e76cd74ab32d3d2f53ae11f50360f2f7c95613e4d23787502a0. Selawik1.01 differs in hhea metrics while OS/2 typo and Windows metrics match this reference. Actual shaping and metric-selection controls remain required. Test raw advances, kerning-sensitive AV/To pairs, ligatures, punctuation, numerals, ascender/descender/line pitch and long wrapping strings. Record actual tolerances separately for each measurement.
- Verify PDF selected font identity, actual advances and PNG/SVG visible output in both document families, including caller override, genuine italic, variable bold and permitted synthesis.
- Verify Aptos warning presence, actual selected family, stable deduplication, warm/cold equality, caller suppression and fallback propagation into text-layout/fit reports. Diagnostic-only changes must not change rendered bytes by themselves.
- Test verified Symbol/Wingdings mappings in Word bullets, explicit Word symbol runs and PowerPoint bullets. Assert a nonzero Unicode glyph, visible rendering, actual caller-font preference, unknown mappings and unrelated PUA preservation, and unchanged original OOXML after layout and save/reopen.
- Reuse F-X195 acceptance for caller inputs and variable geometry. The existing six licensed ignored weight controls and receipt are at /private/tmp/S91-F-X195-variable-oracles. Do not copy them as tracked binary fixtures.
- Run scoped affected-crate checks, full changed-crate tests, exact installed binding runtime and typing coverage if projected diagnostics change, no-default-features, WASM, verified patched packaging, package source/license inventory, actual locally generated archive rows and deterministic hash comparison. Integrated full verification remains the sprint gate.

## HLD impact

- docs/hld/08-rendering-spec.md
- docs/hld/10-bindings-spec.md
- docs/hld/14-development-backlog.md
- docs/hld/15-build-and-toolchain.md

Replace current mapping, diagnostic and packaged-font inventory prose with actual reviewed behavior. Do not present upstream intent as a verified universal metric claim.

## Risk routing

- Layout/shaping and bundled fonts require deterministic controls, exact license/provenance records, archive source membership and unchanged existing font hashes. Read HLD08 and HLD15.
- Parser/serializer exposure through symbol handling requires unchanged original source and byte-preserved unmodelled XML, alias/rebound controls and schema ordering. Read HLD04 and HLD06.
- Published shared API changes require explicit semver impact, verified affected-package dry runs with the full local patch graph and archive bounds under10MiB. Ordinary diagnostic output is a behavior change. An added public metadata field needs its pre-1.0 source impact stated.
- Bindings/WASM require both wasm target checks and installed runtime, strict typing and stubtest when their public diagnostics are changed. Do not link PyO3 extension test binaries.
- External font/native oracles are pinned and remain test infrastructure. A metric divergence is triaged, documented and asserted. Never use production output as the expected answer.
- New asset files need explicit approval. No new trait, generic, module, crate, feature flag, dependency or binary test fixture is proposed.

## Hash harness

Existing cases that request Georgia, Segoe UI, Aptos or verified symbol fallbacks can intentionally change. Enumerate actual deterministic entries, measured glyph/advance and diagnostic changes, review them independently and move only the designated exclusive baseline. Unrelated Latin and CJK entries must remain unchanged. Do not guess all49 entries stay unchanged before measurement.

## Implementation checklist

- [ ] Settle plural font_dirs scope and optional-pack decision with the canonical owner.
- [ ] Approve the exact asset file list, immutable sources, licenses and availability of metric references.
- [ ] Prove focused public failures on claimed Base.
- [ ] Reuse F-X195 selection and implement approved bundled mappings and lifecycle diagnostics.
- [ ] Verify font-specific PUA controls without source mutation.
- [ ] Record measured Gelasio/Selawik compatibility and actual limitations.
- [ ] Run scoped, oracle, packaging, license, binding and WASM riders.
- [ ] Review actual harness changes and update declared HLD sections.

## Open questions

1. Resolved: Issue313 explicitly requests ordered font_dirs. F-X195 owns it and preserves singular font_dir compatibility.
2. Resolved: user approved the exact eight-file core inventory. Genuine regular/bold Selawik and F-X195 synthesis cover this approved inventory.
3. What pinned Segoe UI reference and truthful measured tolerance satisfy acceptance given the upstream kerning limitation? Do not declare compatibility without reference evidence.
4. Resolved: the refreshed issue explicitly defers a distributed font pack. Caller-provided fonts are the supported route.
5. Which exact family/codepoint denotes the issue's section-square bullet, and what installed diagnostic surface consumes fallback reports after byte-only rendering?

## Performed bounded shaping experiment

Pinned HarfBuzz14.4.0 shaped five exact Latin controls in each of four
Gelasio styles at wght400 or700, compared with the installed licensed Georgia
faces read in place. Controls cover kerning pairs, ligatures, numerals,
punctuation and longer strings, with explicit LTR/Latn/en, font-size=upem
and kern0/kern1. All40 pairs have genuine glyph coverage. Maximum absolute
string-advance deltas are0.28857421875em regular,0.48486328125em bold,
0.32470703125em italic and0.38427734375em bold-italic across these controls.
These are observed differences, not acceptable universal tolerances or a
claim that every line wraps identically. Compare the existing substitute,
line metrics and actual layout selection before final compatibility claims.

The exact commands, font versions, hashes, glyph results and normalized
advances are retained in S91-F-X210-font-intake/independent-shaping-controls.json,
SHA256ecb815b27f26ecb27b6a1690ce0d595efdd4bf5dff0895e97f82d4d154a11a6e.
HarfBuzz failed to load the licensed Segoe WOFF reference directly in all10
controls. Those are explicit failed controls, with no numerical shaping
claim and no font conversion or redistribution. Selawik shaping parity
remains unproved by this experiment. No production or baseline changed.
