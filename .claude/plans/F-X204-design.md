# F-X204, Shared CLI one-shot editing commands

**Status**: draft
**Sprint**: S91
**Size**: L
**Depends on**: F-X202, F-X203, F-X205, F-X207
**Implementation**: not claimed, no source or Cargo changes

## Problem

Issue307, captured in S91-intake-issues.json on 2026-10-09, requests
scriptable one-shot editing and reports through the two CLIs. Existing Word
replace is single-pair at rdocx-cli/src/commands.rs:1948, even though native
try_replace_all_expected already supplies ordered transactional pairs at
rdocx/src/document.rs:27163 and Python replace_all exposes them at
rdocx-py/src/document.rs:3589. PowerPoint replace is likewise single-pair at
rpptx-cli/src/commands.rs:527 and has no native counted batch counterpart.

The other omissions are command exposure and some real shared capability
gaps. Native Word field operations are separate general and layout-backed
updates. ImageInfo inventory is body-only, and PowerPoint's existing
text_layout excludes table cells, master shapes and SmartArt. A CLI loop
cannot silently call those incomplete surfaces complete. The native providers
must own common public operations that Python and CLI both consume.

Full acceptance requires Word multi-pair counts, regex and JSON reports,
fields update, image extraction/inventory, core/custom metadata, tagged
content-control fill and counts. PowerPoint requires slide add/duplicate/
remove/move/hide/show, notes text/file updates, fit overflow reports, core
metadata and multi-pair replacement. Each command gets an integration test,
help/reference documentation, input-overwrite refusal, nonzero refusal exits
and all-or-nothing expected counts. Footer/theme CLI work belongs to F-X208.

## Spec reference

- docs/hld/10-bindings-spec.md, "CLIs", shared schema-1 reports, path saves,
  literal replacement scopes, read and mutation conventions.
- docs/hld/10-bindings-spec.md, Python API shape, counted replacements and
  field updates around lines390 through411, and presentation text layout.
- docs/hld/10-bindings-spec.md, "The invalidation problem, handled loudly".
- docs/hld/04-opc-and-packaging.md, owner relationships, preservation and
  package-class/path save contracts.
- docs/hld/06-presentationml-model.md, slide/layout/notes ownership.
- docs/hld/08-rendering-spec.md, deterministic layout and fitting facts.
- docs/hld/14-development-backlog.md, "F-X204, Complete shared CLI one-shot
  editing commands" and provider stories F-X202, F-X203, F-X205 and F-X207.

## Approach

### Layering and actual dependencies

Both CLI manifests already depend on their native family and
oxml-cli-support. Word also depends on existing XML/layout/PDF support and
serde_json. Neither depends on a Python binding. Keep that graph. A command
is a native API consumer, not a Python subprocess, embedded interpreter or
reverse Rust-to-Python edge. Python-visible means the same capability is
exposed by the installed Python API, not that Rust invokes Python.

Current backlog formal dependencies list X200, X201, X202, X203, X205, X206,
X207 and X208. The following is the proposed capability-based reduction:

| Command | Existing common API or real gap | Provider and prerequisite |
|---|---|---|
| Word literal replace map | try_replace_all_expected and Python replace_all exist | X204 consumes now, no formatting/section dependency |
| Word regex counts | Regex exists, expected per-pair report does not | X204 owns minimal shared native/Python addition |
| Word fields update | update_fields and update_layout_backed_fields exist in native/Python | X204 common orchestration, no blanket X201 dependency |
| Word image inventory/extract | images is body-only, image_data_for_story exists | X202 common all-story picture inventory checkpoint |
| Word core metadata | Native checked core updates and PyCoreProperties exist | X204 consumes now |
| Word custom metadata and counts | Native custom properties exist, typed Python/count exposure incomplete | X203 common capability checkpoint |
| Word tagged fill | Native by-tag/binding updates exist, Python exposure belongs to X203 | X203 content-control checkpoint |
| PowerPoint slide lifecycle | Native and Python operations already exist | X204 consumes now, no X206 dependency merely for same files |
| PowerPoint notes | Native and Python set_notes_text/notes_text exist | X204 consumes now |
| PowerPoint fit | text_layout exposed but incomplete inventory and needed-scale semantics | X207 common fit/report checkpoint |
| PowerPoint core metadata | Native core properties exist, Python editing exposure belongs to X205 | X205 metadata checkpoint |
| PowerPoint literal replace map | No counted batch common method | X204 owns minimal native/Python addition |
| Footer inspection/theme commands | Separate acceptance | X208 owns, no X204 dependency |

Remove X200, X201, X206 and X208 as blanket dependencies if this inventory is
confirmed. Their file overlap is a serialization concern, not a capability
requirement. X202/X203/X205/X207 are real requirements only for the named
capability checkpoints. The final canonical plan may depend on those stories
or their formally reviewed dependency-prefix checkpoints. Do not wait for
every unrelated acceptance item before designing the already-supported
commands. Do not mark X204 complete until the full acceptance has its common
native API and installed Python evidence.

X204 owns its two CLI source files, existing integration entrypoint and README
per family, and the narrow batch/orchestration operations described here if
the provider plans assign them to X204. Shared native and Python files must
be explicitly serialized with their owners. No new module, dependency or
feature flag is proposed. No CLI-only capability is introduced.

### Command grammar and counted replacement

Preserve the existing single-pair replace options. Add --map PATH and --json.
Word also gains --regex. Single pair and map are mutually exclusive. A map
is an ordered JSON array, not an object whose key semantics obscure order:

```json
[
  {"placeholder": "{{name}}", "value": "Ada", "expect": 1},
  {"placeholder": "{{state}}", "value": "ready", "expect": 2}
]
```

Reject unknown keys, wrong types, negative/nonintegral expected counts,
missing placeholder/value and conflicting command-line --expect in map mode.
An omitted or null expect means unrestricted count. Retain duplicate
placeholders as meaningful ordered pairs. Parse and validate the whole map
before mutable access. Empty map behaviour must be documented, with the
proposed result being an empty successful report and unchanged document.
Empty literal placeholders retain each family's checked native refusal.

Exact semantics match existing Word try_replace_all_expected:

1. Apply pairs in array order to one staged candidate.
2. Pair2 searches the result of pair1, not the original input. Therefore
   A-to-B followed by B-to-C may legitimately cascade.
3. Expected count is checked on that pair's current candidate scope.
4. On any mismatch, malformed pair, parser/serialization error or invalid
   replacement, discard the entire candidate. Original native state and
   binding revision remain unchanged. CLI creates no output.
5. On success, publish once and report counts in input order. Zero is valid
   when expect=0. Preserve existing family-specific single-pair unmatched
   behaviour rather than changing it implicitly in the map extension.

Use existing Word signature:

```rust
Document::try_replace_all_expected(
    &mut self,
    pairs: &[(&str, &str, Option<usize>)],
) -> Result<Result<Vec<usize>, ReplacementCountMismatch>>;
```

Add the same semantic operation to native PowerPoint in existing lib.rs,
using a concrete family-local mismatch report/error and one staged clone.
Python Presentation.replace_all exposes ordered tuples with optional expected
count, analogous to Word, and returns ordered counts. No dependency on
rdocx's mismatch type is allowed. Use concrete types in existing files, not
a new common generic replacement engine. Each family has distinct traversal,
escaping, field and notes contracts.

### Regex scope and reports

Regex is explicitly requested under Word. Do not add a PowerPoint regex
engine, dependency or flag by interpreting its many-pairs request broadly.
Word native replace_regex at document.rs:27377 and replace_all_regex:27388
already use the regex crate. Reuse that syntax, including $1/$2 capture
replacement. Python uses the same native regex dialect, not Python re.

Word regex searches the documented literal stories except chart labels:
body/nested tables, headers/footers, footnotes/endnotes, text boxes, content
controls and tracked insertions. It matches within supported paragraph/story
segments, not across unrelated paragraphs or XML nodes. Verify the existing
implementation and retained boundaries with actual tests before promising
cross-run matching or the handling of tracked deletions, note separators and
AlternateContent copies. Do not broaden that scope incidentally.

For --regex with map, compile every pattern and validate every replacement
before staging. Add the smallest native ordered expected-regex counterpart:

```rust
Document::try_replace_all_regex_expected(
    &mut self,
    pairs: &[(&str, &str, Option<usize>)],
) -> Result<Result<Vec<usize>, ReplacementCountMismatch>>;
```

It uses the existing regex replacement logic on one candidate, returns
per-pattern counts, and preserves the literal batch's ordered/atomic
contract. The existing mismatch placeholder field contains the pattern for
this entrypoint and must be labelled correctly in reports. Expose the same
capability in Python through an additive counted method or a compatible
extension to replace_all_regex, preserving its current total-count return
unless an explicit pre-1.0 migration is approved. Do not build an unsafe CLI
loop over committed native mutations.

All reports use current oxml-cli-support::json_envelope at lib.rs:355,
schema=1. Existing inspect keys stay compatible and gain additive fields.
Replacement success contains mode, ordered pair index/placeholder/count/
expected records, total count and output path. Keep explicit null expectations.
Human output has the same facts. Diagnostic/refusal information goes to
stderr and nonzero exit. Do not print progress or prose into JSON stdout.

### Fields, images, metadata, fill and counts

Add Word fields update next to existing toc rebuild. Native field.rs:1809
update_fields(context) evaluates ordinary typed caches, including supported
DATE/SEQ/REF, while :1840 update_layout_backed_fields uses deterministic
layout for PAGE/NUMPAGES/PAGEREF and section fields. Their existing contract
preserves dynamic header/footer PAGE/NUMPAGES caches and unsupported switches.
Do not change it merely to make every cache look updated.

The one command needs one atomic operation over ordinary and layout-backed
updates, not two visible native commits with a late partial failure. Proposed
minimal common native method in existing field.rs stages once, evaluates
ordinary fields, computes layout, applies supported layout-backed caches,
then commits once. Return an ordinary update count plus the existing
LayoutBackedFieldUpdateReport, using a concrete tuple or a justified concrete
report if named fields materially help both current wrappers. Expose it in
Python. Context uses existing FieldEvaluationContext:1355. Supply dates
explicitly or define --now semantics, and preserve diagnostics and unsupported
cache facts. --json reports what was updated, not a claim of complete support.

images extract DIR and inspect pictures consume X202's native owner-aware
inventory. Existing Document.images at document.rs:28939 reports only body
drawings with relationship ID/name/description/EMU size, so it cannot produce
the requested story list alone. image_data_for_story:17879 is the common byte
accessor. Report drawing occurrence identity and story separately from its
possibly shared image part. Extract exact embedded bytes, not reencoded
rendered images, preserve known media extension/content type and deterministic
safe filenames, and avoid network fetches for linked external images.
Precompute the entire output set, validate collisions/input identity, load
all required bytes, then stage and publish together using existing support.
Decide duplicate-part extraction and unsupported/missing-media reporting
explicitly. No partial directory contents after a rejected edit.

Word meta get/set uses PyCoreProperties/native core methods already present
at document.rs:25883 through :25897 and typed custom properties at :25950
through :26041. PowerPoint core methods are lib.rs:1927/:1932, with Python
exposure supplied by X205. Parse all assignments before mutation, use native
validation and preserve unspecified fields and opaque metadata. Custom
key=value type policy must match X203's typed Python contract, rather than
inventing CLI-only untyped conversions. Title/author/subject/keywords are
explicit supported core names. Repeatable assignments must reject duplicates
or document one chosen rule. Save through the package-class-aware path API.

fill --tag NAME=VALUE consumes a tag-only common operation. Native
set_content_control_value_by_tag at content_control.rs:95 exists, but
bind_content_controls:113 falls back to aliases when a tag is absent. Do not
use that fallback for a flag explicitly named --tag. Repeated tag pairs must
stage all updates once and preserve binding custom XML/display ownership.
X203 owns Python exposure. Validate known unsupported or ambiguous control
cases and no-match policy before publishing. Do not add control creation or
tracked-edit authoring, which the provider's backlog explicitly excludes.

Inspect word/character/page counts consume X203's public facts. Existing
word_count at document.rs:29025 counts whitespace tokens in the accepted
body including table/control paragraphs. It is not an all-story locale word
counter. State its scope. Character and deterministic page counts must use
the provider's specified common API, never a CLI-only heuristic or cached
docProps count. Keep existing inspect schema and keys additive. Layout
diagnostics and unsupported stories must be reported honestly.

### PowerPoint slides, notes and fit

Native add_slide:3253, duplicate_slide:2791, remove_slide:2572 and
move_slide:2585 already have Python consumers in slide.rs:575, :688, :670
and :711. Native/Python hidden mutation exists, as does notes_text setter
slide.rs:229. Add thin command parsing and reporting around these operations.
CLI slide numbers are one-based, native indices zero-based, and conversion
must happen exactly once. Proposed --at denotes a final one-based insertion
position including append. Resolve --layout by checked numeric index or an
exact unique name, with clear unknown/ambiguous diagnostics. Add then move
must be a single staged common native operation if --at is supported, also
exposed to Python. No half-added slide on a placement failure.

Slide duplicate retains native placement after the source unless the final
CLI contract specifies otherwise. Remove/move/hide/show use native checked
operations and package lifecycle. Notes set requires exactly one of --text
and --from-file, validates UTF-8 and reads the complete file before mutation.
Do not add undocumented trimming or newline conversion. All mutations use
an output path and refuse to overwrite input.

fit is a read-only report, not an autofit mutation. It exits1 when any
reported text frame overflows,0 otherwise, and uses the shared JSON envelope.
Existing Presentation.text_layout_deterministic at lib.rs:1199 returns
TextFrameLayout with owner slide/index/id/name and actual renderer font_scale
and overflow. Its documentation explicitly excludes table cells, master
shapes and SmartArt. Current font_scale is the renderer's existing scale,
not automatically the additional scale needed to make an overflowing frame
fit. X207 owns the required common all-frame fit analysis, meaningful needed
scale and unsupported diagnostics. CLI and Python consume it. No CLI-only
binary search or fabricated scale. Preserve ownership identities, units and
deterministic font policy in the report, including cell paths if supported.

### Publication and compatibility

Use existing ensure_output_paths_allowed in oxml-cli-support:143 to reject
input identity, aliases/symlinks/hardlinks and unsafe collisions. Keep current
stage_and_publish and AtomicOutputBatch mechanisms. Mutating commands default
to refusing an existing output like current replacements. A later --force
choice must use the existing shared contract and still never replace input.
Extension-based package class, VBA refusal and signatures remain the native
path-save contract. Do not publish success reports before files are published.

## Rejected alternatives

- Invoke Python from Rust. It adds a reverse dependency/runtime requirement
  and violates thin native wrappers.
- Put regex/counting, fit solving or content-control XML only in commands.rs.
  That creates CLI-only capabilities and inconsistent Python behaviour.
- Interpret replacement maps as unordered objects. It obscures ordered
  expected-count and duplicate-pair semantics.
- Apply each pair directly to live native state. A later mismatch leaves a
  partial edit even when the CLI does not save it.
- Reencode extracted images. Extraction must retain exact embedded bytes.
- Report current text layout as every frame and current font_scale as needed
  scale. Existing documented scope and meaning do not support those claims.
- Keep eight blanket dependencies because files overlap. Explicit provider
  checkpoints and serialized ownership express the actual constraints.

## Test plan

**Test gate**: regression. Reproduce every missing operation or reported
defect through the installed public API on the claimed Base before
implementation. Exercise all Issue307 acceptance criteria through save/reopen,
native rendering where applicable, malformed/atomic refusal controls and
preserved unmodelled XML. Python requires accurate stubs, strict typing and
stubtest. CLI requires help/reference and existing integration tests. Use
deterministic fonts and pinned native oracles for output claims. Google
Docs/Slides import is best effort. Reuse prerequisite evidence and test the
integrated behaviour. Require scoped verification and independent
zero-defect, zero-smell review.

| Category | Test | Asserts |
|---|---|---|
| regression | word_replace_map_is_ordered_counted_and_atomic | Cascade, duplicate pairs, per-pair count, late mismatch, unchanged input/output |
| regression | word_regex_map_has_native_scope_and_capture_counts | Capture substitutions, cross-run controls, chart exclusion, invalid later regex and atomicity |
| regression | powerpoint_replace_map_is_ordered_and_includes_notes | Native/Python/CLI count parity, notes, late mismatch and opaque bytes |
| integration | fields_update_reports_general_and_layout_cache_results | DATE/SEQ/REF/PAGE/NUMPAGES, explicit time, retained diagnostics and late failure |
| integration | images_extract_preserves_bytes_and_reports_story_owners | Body/table/header/footer/note controls, shared media, missing/external image and safe filenames |
| integration | inspect_adds_pictures_and_public_counts_compatibly | Existing schema/keys retained, native/Python inventory and count parity |
| integration | word_meta_get_set_preserves_core_custom_types | Every requested core field, typed custom values, invalid atomic refusal |
| integration | fill_tag_updates_all_matching_controls_atomically | Tag-only selection, alias collision, bound custom XML and unsupported control refusal |
| integration | slide_add_resolves_layout_and_insertion_atomically | Name/index, first/middle/append, invalid/ambiguous layout and no half-edit |
| integration | slide_duplicate_preserves_native_slide_and_notes_ownership | Correct placement, IDs and related parts |
| integration | slide_remove_preserves_native_relationship_lifecycle | Reopen, removed/shared parts and invalid index |
| integration | slide_move_uses_one_based_cli_positions | Every boundary, no-op and rejected positions |
| integration | slide_hide_and_show_round_trip | Native/Python values, repeated no-op and opaque content |
| integration | notes_set_text_or_file_preserves_exact_text | UTF-8 multiline source, conflicting options and missing file refusal |
| integration | fit_reports_every_supported_overflow_and_needed_scale | Provider owner inventory, genuine scale, no write and exits0/1 |
| integration | powerpoint_meta_get_set_matches_python_core_properties | All requested fields and retained imported metadata |
| integration | every_new_command_has_help_and_reference_contract | Grammar, defaults, scope, output/JSON/expect and units |
| integration | new_mutations_reject_input_aliases_and_output_collisions | Same path/symlink/hardlink, existing output, late stage failure and no partial files |

Use only current CLI tests/integration.rs per family. Existing Word tests at
:287 inspect schema, :1402 replacement, :1509 namespace refusal, :1546
text-box counts, :1596 controls, :1710 notes/revisions and :3713 structured
reports are controls. Existing PowerPoint :1105 preservation, :1174 output
class, :1254 counted notes and :1523 additive inspect keys are controls.
Existing output refusal helpers :74/:105 Word and :63/:98 PowerPoint apply
to every new output command. Add no test binary.

Shared new native operations get tests in current native modules/entrypoints,
and installed Python tests go into current test_core.py and
test_documented_examples.py. Provider tests are reused with cited evidence.
Add current typing_smoke/stub changes for the new common capabilities. Verify
twice save/reopen and exact unknown XML/opaque package parts. Tests must prove
both native rollback and filesystem refusal, not just absence of output.

## HLD impact

- docs/hld/02-scope-and-non-goals.md, actual CLI and Python capabilities.
- docs/hld/03-architecture.md, only if common native operation ownership changes.
- docs/hld/04-opc-and-packaging.md, extraction/publication ownership contracts if expanded.
- docs/hld/08-rendering-spec.md, fit/report facts supplied by X207, no duplicated solver.
- docs/hld/10-bindings-spec.md, exact CLI grammar/reports, shared Python-visible operations and fields/count scopes.
- docs/hld/14-development-backlog.md, actual prerequisites and acceptance.

Help and CLI references update current main.rs clap descriptions,
crates/rdocx-cli/README.md, crates/rpptx-cli/README.md and the existing root
README CLI examples where affected. No new reference document is proposed.
Source additions earn actual archive record maintenance in the existing
README/script carriers. Root regenerates integrated measurements, not stale
worker tuples.

## Risk routing

- Published native API. Read HLD10 and structural rules, state additive
  pre-1.0 signatures and run actual archive dry-run/size checks for affected
  native/CLI crates, including required measured records.
- PyO3 exposure. Read HLD10, installed source-built parity, strict mypy and
  stubtest, documented workspace binding exclusions and WASM check.
- Parser/serializer and regex/field edits. Read HLD04/HLD06, test actual
  namespaces, schema order, unsupported switches and byte-preserved raw nodes.
- Rendering/layout for field pages, counts and fit. Read HLD08, use
  deterministic fonts and preserve provider diagnostics, with pinned oracle
  evidence for claims that affect output.
- Dependency edges. Read HLD03 and retain native-to-support edges only,
  with no Python runtime reverse dependency or new regex dependency in CLI.
- Unit conversion and index arithmetic. Read HLD01/CLAUDE, preserve
  truncation and checked one-based-to-zero-based conversion.
- External oracle. Read canonical differential-testing, pin versions and
  retain provenance. CLI parsing is not a PowerPoint open-without-repair check.

These add to normal scoped verification and independent microscope, never
replace them. No new file/module/trait/generic/feature flag is needed.

## Hash harness

Expected unchanged for new commands and additive shared operations. They do
not alter existing default output, traversal or rendering. Explicit command
edits naturally change newly generated documents, with counts and expected
bytes asserted in their own source-built regressions. Provider behaviour
changes keep their own labelled reviewed deltas. Do not attribute an
unexplained hash delta to CLI work or move the exclusive baseline.

## Implementation checklist

- [ ] Review and reduce actual capability dependencies and assign native/Python provider ownership.
- [ ] Capture installed/native Base failures and existing supported controls.
- [ ] Finalize map grammar, exact ordered rollback semantics and JSON schema.
- [ ] Add common counted batch/regex/field orchestration operations with Python exposure.
- [ ] Add thin commands consuming provider image/count/control/meta/fit surfaces.
- [ ] Add every command's existing-entrypoint integration test and filesystem rollback controls.
- [ ] Update help, existing reference text, stubs and HLD contracts.
- [ ] Run focused evidence, applicable riders, unchanged hash and scoped gate.
- [ ] Obtain independent microscope over final diff including measured records.
- [ ] Prepare only after the full contract is satisfied, with no worker ledger edits.

## Open questions

1. Confirm which provider checkpoints are independently reviewable and can
   replace blanket story dependencies. Canonical backlog changes belong to
   root, not this scratch writer.
2. Choose exact map keys and Python counted regex signature. The proposed
   keys mirror existing CLI placeholder/value names and preserve current
   Python total-count compatibility.
3. Confirm Word regex physical AlternateContent, tracked deletion, note
   separator and field-result boundaries from source-built controls. Document
   actual scope rather than expanding it accidentally.
4. Choose common combined field-update return shape and explicit time policy,
   preserving existing saved-cache semantics and atomicity.
5. Set typed custom-property input syntax and clear semantics consistently
   with X203, including ambiguous numeric/string values and duplicate keys.
6. Choose image occurrence versus unique-part extraction filenames/reports,
   linked-image diagnostics and full publication rollback semantics.
7. Finalize one-based --at and move semantics and whether numeric layout
   indexes are likewise one-based in CLI. They must be explicit in help.
8. Confirm X207's every-frame inventory, meaningful needed-scale contract and
   unsupported-frame reporting before declaring fit complete.
9. Clarify whether --json refusals need structured stderr diagnostics in
   addition to existing nonzero exit, without corrupting success stdout.

All unresolved questions remain in draft. No approval, source, baseline or
Cargo action is implied by this ignored proposal. F-X193's scoped gate
remains incomplete and its reviewed source is untouched.
