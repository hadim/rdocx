# F-289, Modern Word form authoring

**Status**: approved
**Sprint**: S95.4
**Size**: L
**Depends on**: F-275, F-285, F-286, F-287, F-288
**Base**: a57be365 on sprint/s95.4
**Design date**: 2026-10-23

## Problem

Callers can create content controls, set bindings, repeat and remove controls, insert fields, add story permission ranges, and set document protection. They cannot validate, fill, reset, or protect a form as one atomic operation over those authoritative parts. A second document tree, form-builder object, or serialized shadow value would fork the model. Existing high-level control value updates fill main-body controls (`crates/rdocx/src/content_control.rs:208`). Creation is at line 1373, payloads at 1346, binding at 1454, the placeholder flag at 1531, repeat at 1592, and remove at 1625. `set_value_metadata(None)` (`crates/rdocx-oxml/src/content_control.rs:274`, `367`, `402`) does not clear `fullDate`, so a placeholder reset must clear that field explicitly. Permission ranges enter at `crates/rdocx/src/comments.rs:441`. Protection enters at `crates/rdocx/src/document.rs:33289`. Password derivation is an existing non-goal.

F-288 is pending and unclaimed. This plan drafts against its proposed contract, including the Q2 default split. Implementation waits for the Q4 checkpoint answered by the user on 2026-10-23: F-288 local RED then GREEN for all named focused regressions, scoped verification, a clean microscope, and an integrated source ancestor. Final Word differential, full verification, and riders remain pending. Word 16.114 (16.114.26101123) is installed. No new dense-form comparison or render differential has run. Older form evidence does not qualify this authoring. "Typed content-control native Word qualification" in `docs/hld/12-testing-strategy.md` still governs provenance.

## Spec reference

- `docs/hld/14-development-backlog.md`, "F-289, Modern Word form authoring" (lines 3140-3146). Test gate copied below.
- `docs/hld/02-scope-and-non-goals.md`, protection metadata and password non-goal.
- `docs/hld/03-architecture.md`, "What stays put" and "Facade conventions".
- `docs/hld/04-opc-and-packaging.md`, "The package".
- `docs/hld/10-bindings-spec.md`, "Python API shape" and "Live Word sections and rich related owners".
- `docs/hld/12-testing-strategy.md`, "Test taxonomy" and "Typed content-control native Word qualification".
- F-285 through F-287 backlog contracts at `docs/hld/14-development-backlog.md` lines 3096-3131.
- F-288 design `.claude/plans/F-288-design.md` for legacy defaults. Draft reliance only.

## Approach

No new module or file. Definitions live in existing `content_control.rs` and are exported from `lib.rs`. Carriers are transient operation inputs. They are not retained on the document. After reopen the caller supplies rules again.

```rust
pub enum FormFieldTarget {
    ContentControlTag(String),
    Legacy { source_part: String, ordinal: usize },
}
pub enum FormFieldValue {
    Text(String),
    Checked(bool),
    SelectedIndex(usize),
}
pub enum FormResetValue {
    Value(FormFieldValue),
    Placeholder { text: String },
}
pub struct FormFieldRule {
    pub target: FormFieldTarget,
    pub required: bool,
    pub reset: FormResetValue,
}
pub struct FormFieldAssignment {
    pub target: FormFieldTarget,
    pub value: FormFieldValue,
}
pub struct FormValidationIssue {
    pub target: FormFieldTarget,
    pub source_part: String,
    pub ordinal: usize,
    pub message: String,
}
impl Document {
    pub fn validate_form(&self, rules: &[FormFieldRule]) -> Result<Vec<FormValidationIssue>>;
    pub fn fill_form(&mut self, rules: &[FormFieldRule], assignments: &[FormFieldAssignment]) -> Result<Vec<usize>>;
    pub fn reset_form(&mut self, rules: &[FormFieldRule]) -> Result<Vec<usize>>;
    pub fn protect_form(&mut self, rules: &[FormFieldRule], protection: DocumentProtection) -> Result<()>;
}
```

One private physical inventory dedups by `(part, start, end)`. Source-part preorder of all controls, including structural controls, runs before the tag filter. The most specific valid owner wins. Overlapping targets refuse. One physical control produces one issue and one count. Fan-out applies only to distinct controls.

Imported current reads, proposed and not implemented:

- `showingPlcHdr` true means semantic empty regardless of the prompt. Ignore a stale retained full date as the value. A bound scalar should be empty, otherwise one issue.
- Rich and plain content exclude independent nested controls.
- A checkbox uses the primary checked child. A missing `checked` child is false, because `CT_SdtCheckbox.checked` defaults to unchecked ([MS-DOCX CT_SdtCheckbox](https://learn.microsoft.com/en-us/openspecs/office_standards/ms-docx/d4682d85-ae4d-48a7-8376-ec2448ae7e35), recorded as 2.6.3.28). A present `w14:checked` with no `val` is true, because `CT_OnOff.val` defaults to true ([MS-DOCX CT_OnOff](https://learn.microsoft.com/en-us/openspecs/office_standards/ms-docx/173a98b1-92a5-4dd8-a0be-0614b1cb6606), recorded as 2.6.3.17). Never infer a boolean from a glyph. A bound value expects a boolean.
- An active full date is canonical UTC midnight. The reader uses the ISO part normalizer, then compares `DateFormat` and `StoreMap`. Unsupported metadata is one issue and the bytes stay. No arbitrary normalization on a read-only path. Do not feed a `dd/MM` display into the ISO writer.
- A list uses the bound scalar when bound, otherwise the display, through the existing value and label ambiguity resolver. `lastValue` is an import hint and not sole authority. Ordinary read stays. A stale hint does not validate a wrong bound or display.
- A placeholder flag means semantic empty. A stale `fullDate` or `lastValue` is a hint, not a value, and a read-only path preserves it. A nonempty bound scalar is an issue. Do not reject an optional placeholder only because retained metadata is still present. Reset explicitly removes `fullDate` and list-selection hints.
- Missing or unresolvable bound-store ownership is `Err`. A valid rule with an invalid, unsupported, or inconsistent current state is one issue. Fill and reset of valid inputs repair a selected invalid current state atomically. Partial fill may leave an unassigned required field empty. Protect requires zero issues.

Discriminators: date `23/10/2026` with ISO metadata and mapping, a list label that differs from its bound value, duplicate labels, a stale `lastValue`, a false checkbox with nonempty display, bound disagreement becoming an issue and then a repair, a placeholder prompt, and a current control counted once.

`validate_form` returns one issue per physical target. `source_part` and `ordinal` are the control or legacy inventory coordinates. There is no forwarding report wrapper. A malformed rule is `Err`. A valid rule whose existing value is invalid returns an issue. Empty `rules` means no required validation. It is not a discovered schema.

`fill_form` checks the supplied values atomically and may leave other required fields empty, so partial fill is legal. `reset_form` is a real transaction. It restores typed defaults, bound custom XML, and placeholder display. It is not an alias of another setter. Reset checks structure, ownership, and type, and it may produce a required-empty document. `protect_form` requires `ProtectionMode::Forms`, requires `validate_form` to return no issues, then publishes protection metadata only. It does not derive a password. Empty-template protection is the caller passing `required: false` rules that currently validate. Native Word does not enforce these library required rules.

Selection:

- `ContentControlTag` is a case-sensitive exact tag match across every currently authorable internal Word story. Zero matches is `Err`. Several matches receive the same assignment in deterministic physical story order.
- If any matched control has an incompatible kind, the whole operation refuses.
- Duplicate, overlapping, or conflicting bound targets in one batch refuse before mutation.
- Alias is not treated as a tag.
- `Legacy` uses `source_part` plus ordinal, checked once against the pre-batch inventory. A structural edit in the same batch that would remove or reindex that legacy target refuses the batch.
- Counts in the returned `Vec<usize>` follow input assignment order for fill and input rule order for reset. They are not a global tally.
- Unknown assignments, or two assignments of the same target, refuse. Empty assignments with a valid rule schema are a no-op. A same typed value is not an early return. The no-op is the staged candidate bytes, using the same byte test as the F-288 controlled setter. A same checkbox or list value still repairs a wrong display. A same current assignment still repairs bound state and display. A byte-identical candidate preserves bytes and live handles. The private physical inventory stays one dedup on `(part, start, end)`. Legacy reset still uses the Q15 pre-batch pair and does not add a second default query.

Kind policy:

- Rich text, plain text, date, combo, and modern drop-down use `FormFieldValue::Text`.
- Checkbox uses `Checked`. `false` is a present value.
- Legacy drop-down uses `SelectedIndex`.
- Modern drop-down does not accept `SelectedIndex` on the selected path. Text is the declared value or label under the existing ambiguity policy. A wrong kind refuses. Admitting `SelectedIndex` for modern lists would be a plan revision.
- Picture, group, repeating section, and repeating item are structural. Scalar rules cannot target them. Existing create, repeat, remove, move, clone, and unwrap APIs remain their lifecycle.
- Required text is empty when stored XML whitespace makes it empty. Content is not trimmed before that test.
- Date display stays the existing Gregorian ISO path and only `yyyy-MM-dd` or `dd/MM/yyyy`.
- Picture placeholders stay on existing structural APIs.

Reset:

- `FormResetValue::Value` writes the typed value and the bound value, and clears `showingPlcHdr`.
- `FormResetValue::Placeholder { text }` is legal only for modern rich, plain, date, combo, and drop-down. It clears `fullDate` explicitly, clears the active list selection, sets the bound scalar to empty, and writes the prompt display with `showingPlcHdr`.
- Checkbox and legacy targets reject `Placeholder`.
- Legacy `Value` must equal the explicit default from the F-288 pre-batch pair, or the F-288 fallback (`Text("")`, `Checked(false)`, `SelectedIndex(0)`) when that pair's default is `None`. Any other rule value is `Err`, so the rule cannot become a second default authority. An absent legacy default therefore requires the rule to carry that same fallback if the caller wants a value reset. No second single-field default query is added.
- Placeholder text is display. Required validation uses semantic typed or bound values and placeholder state.
- Existing fill clears `showingPlcHdr`.
- Reset keeps repeating-section item count and resets matching descendants. Clone-first append and last-item refusal stay as F-286 Q24 recorded them.
- Reusable glossary placeholder parts stay on F-237 building blocks.

Publication clones, stages, and commits once. Existing markers, permissions, control kind and order, unknown properties, and source scopes stay in that commit. Cross-story matching is new work over the same owners. It is not a loop that only calls the main-body setter.

Q5 selected answer, user answer on 2026-10-23: the proposed modern-form contract. Transient supplied rules after reopen. All currently authorable internal stories. Partial fill. Typed, list, and required validation. Atomic fill, reset, and protect. Typed or placeholder reset. Repeat counts unchanged. Metadata-only protection and no password derivation. Modern scalar controls plus F-288 legacy fields. No regex, range, or custom callbacks. No persisted rule schema. No generated credentials. The richer historical alternatives stay rejected.

## Python API contract

This surface matches the current native four-field issue. Do not adopt a different location or reason carrier. There is no report wrapper, no reason enum, and no location getter. One issue per physical owner. `ordinal` is source-part scoped preorder over the applicable control inventory, or over the legacy inventory identified by the target, in deterministic physical story order.

All classes are frozen and public on `rdocx` and `rdocx._rdocx`, with `_rdocx.pyi`, `__all__`, and `__init__` imports aligned. Factory-private classes use the existing stub `__new__(cls, *, _private: _Never) -> Class`, which returns the class so static factories stay callable. Public input constructors are callable. Fields are readonly. Sequence inputs are copied and frozen. `str` and `bytes` are rejected for a sequence of rules, assignments, or choices. Bool is not an integer. Defaults are the actual `inspect.signature` values. Use the existing PyO3 `text_signature` where needed so `()` is not `Ellipsis`.

Shared error and lifetime contract. Strict wrong Python shapes and types raise `TypeError`. Negative or out-of-domain primitives raise `ValueError`. Nonnegative integers outside the `u32`, `u64`, or `usize` extraction range raise `OverflowError`. Bool is forbidden for numeric indices, counts, and sizes. Native semantic, XML, and package errors keep the existing `rdocx_to_pyerr` at `src/lib.rs:117`. Oxml maps to `XmlError`, package and io map to `PackageError`, and Other and Story map to `RdocxError`. Do not introduce another error hierarchy and do not convert every native error to `ValueError`. Wrong-kind and out-of-list-domain native validation uses that existing mapping. Factories validate in native code before return. All new method results, tuples, PyObjects, and revision tokens are prepared before publishing a candidate. Refusal and no-op preserve bytes, revisions, and live handles. Changed publication follows the existing physical owner policy and never revives dead owners. Owned old inventory snapshots remain old. Structural insertion uses the existing conservative publication. Fill and reset use `publish_form_candidate` at `document.rs:7084`, extended only for actual related-owner coverage. Protection metadata preserves content handles when the same owners are proved. Permission insertion is structural.

```python
class FormFieldTarget:  # frozen, factory private
    @staticmethod
    def content_control_tag(tag: str) -> FormFieldTarget: ...
    @staticmethod
    def legacy(source_part: str, ordinal: int) -> FormFieldTarget: ...

class FormFieldValue:  # frozen, factory private
    @staticmethod
    def text(value: str) -> FormFieldValue: ...
    @staticmethod
    def checked(value: bool) -> FormFieldValue: ...
    @staticmethod
    def selected_index(value: int) -> FormFieldValue: ...

class FormResetValue:  # frozen, factory private
    @staticmethod
    def value(value: FormFieldValue) -> FormResetValue: ...
    @staticmethod
    def placeholder(text: str) -> FormResetValue: ...

class FormFieldRule:  # frozen, public constructor
    def __init__(self, *, target: FormFieldTarget, reset: FormResetValue, required: bool = False) -> None: ...
    # readonly: target, reset, required

class FormFieldAssignment:  # frozen, public constructor
    def __init__(self, *, target: FormFieldTarget, value: FormFieldValue) -> None: ...
    # readonly: target, value

class FormValidationIssue:  # frozen, factory private
    # readonly: target: FormFieldTarget, source_part: str, ordinal: int, message: str

class DocumentProtection:  # frozen, public constructor
    def __init__(
        self,
        *,
        mode: Literal["readOnly", "comments", "trackedChanges", "forms"],
        enforcement: bool | None = None,
        formatting: bool | None = None,
        provider_type: Literal["rsaAES", "rsaFull", "custom"] | None = None,
        algorithm_class: Literal["hash", "custom"] | None = None,
        algorithm_type: Literal["typeAny", "custom"] | None = None,
        algorithm_sid: int | None = None,
        spin_count: int | None = None,
        hash: str | None = None,
        salt: str | None = None,
    ) -> None: ...
    # same 11 readonly properties. Integers are 0 through u32::MAX and exclude bool. No password API.

class Document:
    def validate_form(self, rules: Sequence[FormFieldRule]) -> tuple[FormValidationIssue, ...]: ...
    def fill_form(self, rules: Sequence[FormFieldRule], assignments: Sequence[FormFieldAssignment]) -> tuple[int, ...]: ...
    def reset_form(self, rules: Sequence[FormFieldRule]) -> tuple[int, ...]: ...
    def protect_form(self, rules: Sequence[FormFieldRule], protection: DocumentProtection) -> None: ...
    @property
    def document_protection(self) -> DocumentProtection | None: ...
    def set_document_protection(self, value: DocumentProtection) -> None: ...
    def remove_document_protection(self) -> DocumentProtection | None: ...
    def add_story_permission_range(self, range: StoryRunRange, *, editor: str | None = None, group: str | None = None) -> int: ...
```

`remove_document_protection` preallocates the old snapshot before the removal publish. `add_story_permission_range` requires exactly one nonempty editor or group. It reuses the existing range conversion used by `add_comment` at `document.rs:11038`.

Empty valid inputs are `()`. Malformed rules use the error mapping above. Rules are not globally discovered. Partial fill may leave other required fields empty. Reset may leave required fields empty. `protect_form` is forms mode only and requires a complete validation. Rule validation includes the reset payload and schema ownership, and it does not require whole-form required completeness until `validate_form` or `protect_form`. Tag all-story fan-out is atomic. Duplicate or overlapping bound targets refuse. Repeat counts are preserved.

Existing `insert_content_control`, type and payload factories, binding, repeat, moving, and the related control methods are already Python. Retain them and test composition. Do not add a new form tree. Existing body-only setters and listing keep their existing behavior. Only the new high-level form operations cover every story.

Missing composition providers that must expose existing native paths are `DocumentProtection`, `document_protection`, `set_document_protection`, `remove_document_protection`, and `add_story_permission_range`.

Python tests in existing `crates/rdocx-py/tests/test_core.py`: `test_form_rules_fill_reset_and_protect_public_api`, `test_form_batches_preserve_owners_and_refuse_invalid_targets`, and `test_form_protection_and_permission_composition`. Typed public usage of the factories uses existing `crates/rdocx-py/tests/typing_smoke.py`. No temporary string runner and no new tracked file. The final Word dense form source also has a Python-created counterpart to the Rust source, using the identical criterion. It is not a reuse of old F-286 evidence. Metrics stay unchanged.

Binding files are existing `src/document.rs` and `src/lib.rs` under the Python crate, plus `python/rdocx/__init__.py`, `python/rdocx/_rdocx.pyi`, and `tests/test_core.py`. F-289 does not add a `run.rs` binding. Native and HLD claims retain the files already listed in this plan.

The prior native planning basis remains 8 to 11 developer-days across the sprint. Q1 Python scope is an explicit planning estimate of 4 to 6 additional developer-days, for an expanded planning total of 12 to 17. That figure is an estimate, not a measurement, and it is not an undisclosed capacity promise.

## Consumer map

| Rule | Entry points and outputs | Shared provider and caller policy | Lifecycle stages compared |
|---|---|---|---|
| Build the form | Existing `insert_content_control`, `set_content_control_type`, `set_content_control_binding`, `repeat_content_control`, remove, move, clone, unwrap, `Document::insert_field` with F-288 `from_legacy_form`, Python `add_story_permission_range`, and `DocumentProtection` with `set_document_protection` / `remove_document_protection` / `document_protection` | Authoritative package parts. No shadow model. Existing Python control methods stay. No new form tree | Create, serialize |
| Validate | `validate_form` | Caller rules. Issues are per physical target | Read |
| Fill | `fill_form` | Supplied values only. Other required fields may stay empty | Update, serialize, reopen |
| Reset | `reset_form` | `FormResetValue` applied to controls and bound XML | Update, serialize, reopen |
| Protect | `protect_form` | `ProtectionMode::Forms` and zero issues. Metadata only | Update, serialize, reopen |
| Legacy default | F-288 `legacy_form_fields_with_defaults` pair | Rule value must match the explicit default or the fallback. Struct shapes stay unchanged | Update |
| Python, WASM, CLI | Q1 selected answer: new Rust and Python authoring APIs. The exact surface is `## Python API contract`. WASM and CLI gain no new authoring methods. Existing consumers retain preservation and current-value behavior | Frozen factories, four operations, protection getter and three methods, permission insertion, exports, and stubs. Codex root approved this contract on 2026-10-23. Implementation waits for the actual F-288 local prefix | Reopen |

## Behavior contracts and evidence

| Surface or placement | Exact behavior and boundary examples | Primary source identity and location | Local discriminator or pending evidence |
|---|---|---|---|
| Tag fan-out | Tag `A` on a body control and a header control both receive `Text("x")` when kinds match | New cross-story work. Body setter is only `content_control.rs:208` | `form_selection_and_updates_use_the_same_physical_story_owner` |
| Heterogeneous tag | Same tag on a checkbox and a plain text control | Whole batch `Err` | A2 negative |
| Legacy snapshot | Fill that also replaces the legacy field's parent | `Err` before publication | `form_batches_reject_invalid_values_without_partial_publication` |
| Value reset | Bound active date becomes the rule text, `fullDate` follows the existing date writer, `showingPlcHdr` clears | Binding writer `content_control.rs:1454` | A3 |
| Placeholder reset | Active date or list becomes empty bound scalar, prompt visible, `showingPlcHdr` set, `fullDate` cleared even though `set_value_metadata(None)` does not | `oxml` `content_control.rs:274`, `367`, `402` | A3 date and list arm |
| Required empty | Reset to empty text, then `protect_form` | Reset `Ok`. Protect `Err` | A4 |
| Incremental fill | One assignment while another required field is empty | `Ok` and one count entry | A2 |
| Repeat | Two items reset | Count stays 2. Append still clones the first item. Removing the last item still refuses | `form_reset_preserves_repeat_count_and_editable_ranges` |
| Protection | Existing permission ranges remain. Mode must be Forms | `document.rs:33289`, `comments.rs:441` | A4 |
| Render | Deterministic bundled fonts, 144 DPI, equal page count, exact text and checkbox semantics, SSIM at least 0.95 per page after documented margin alignment, matched fonts, and deterministic drawing | Q6 selected on 2026-10-23. Native independent Word comparison remains unrun | A7 pending |

## Dependency readiness

BACKLOG: F-275, F-285, F-286, and F-287 are done. F-288 and F-289 are pending and unclaimed. This draft assumes the F-288 contract in `.claude/plans/F-288-design.md`, especially `legacy_form_default_value`, the batch pair, and the fallback. Q4, user answer on 2026-10-23, starts F-289 only after the verified local F-288 checkpoint: local RED then GREEN for all named focused regressions, scoped verification, a clean microscope, and an integrated source ancestor. Final Word differential, full verification, and riders remain pending. The historical wait-for-final-differential alternative stays recorded and was not selected. Codex root approved this provider contract on 2026-10-23 at source `a57be365`. That approval does not mark F-288 done. Implementation of F-289 waits for the actual verified F-288 local prefix. Q15 does not add a default type to F-290.

## Acceptance criteria

| ID | Input or setup | Expected observation or refusal | Consumers | Local test | Final gate or pending evidence |
|---|---|---|---|---|---|
| A1 | Source-built dense form through existing create APIs and the four high-level methods, in Rust and in a Python-created counterpart that uses the identical criterion. No injected document XML. Not old F-286 reuse | One package model. Rules are not stored | Rust and the Python contract | `dense_authored_form_fill_reset_and_protect_share_one_model` and `test_form_rules_fill_reset_and_protect_public_api` | Differential pending. Metrics unchanged |
| A2 | Cross-story tags, type and list checks, required checks, atomic negatives, incremental fill, held handles, frozen issues, no-op, and refusal | Matching owners update together. One bad target publishes nothing. Partial fill of valid values is allowed | Rust and Python | `form_batches_reject_invalid_values_without_partial_publication` and `test_form_batches_preserve_owners_and_refuse_invalid_targets` | Word pending |
| A3 | Fill and reset of defaults, bounds, placeholders, and bound stores. Active date and list then placeholder reset | Typed value, empty bound scalar, prompt, and `showingPlcHdr` match the reset enum | Rust and Python | Covered by the dense test and `form_reopen_keeps_defaults_placeholders_and_bound_values` | Word qualification of prompt versus stored default pending |
| A4 | Protect and editable ranges. Required-empty then protect | Metadata published only when issues are empty. Ranges preserved. Password not derived. Content handles stay when the same owners are proved | Rust and Python | `form_reset_preserves_repeat_count_and_editable_ranges` and `test_form_protection_and_permission_composition` | Native protect behaviour pending |
| A5 | Two distinct repeated items | Count unchanged. Append clones the first item with a fresh identity. Last-item removal refuses | Existing repeat and remove | Same repeat test | F-286 Q24 remains |
| A6 | Body, table, header, footer, note, and text box. Unknown imported bytes. Related owners | Owner fidelity and verbatim unknowns through save and reopen | Rust and Python | `form_selection_and_updates_use_the_same_physical_story_owner` and the reopen test | Native pending |
| A7 | Native-created dense form, then fill, reset, protect, and reopen. Python-created counterpart uses the same criterion | Behaviour and deterministic render differential. Metrics unchanged | Word 16.114 | None until inputs exist | Pending. Q6 gate selected and unrun. Native Word only in the final union |
| A8 | Python export, stub, constructor, and factory types. Primitive errors. Protection getter plus three methods and permission. Related owners, held handles, frozen issues, no-op, and refusal | Public surface matches `## Python API contract`. Empty rules are `()`. Malformed rules use the shared error mapping | Python only, composing existing control APIs | `test_form_protection_and_permission_composition` and the typed public factory use | Strict mypy and stubtest only in the final union |

Backlog promise: a dense source-built form behaves and renders like the pinned Word form across fill, reset, protect, and reopen. The Q6 render gate is selected. Rendering acceptance stays missing until the native independent Word comparison runs. A miss against the threshold stays an explicit gap.

## Test matrix

| Case | Modes or state combined | Expected difference or agreement | Wrong implementation rejected |
|---|---|---|---|
| C1 | Dense form: scalar controls, one legacy field, one repeat, one permission range | Fill, reset, and protect share package parts | A shadow rule part fails reopen |
| C2 | Invalid assignment after a staged edit, and a legacy reindex in the same batch | No partial package | A commit-then-rollback leak fails |
| C3 | Same tag in header and body, same kind | Both physical owners change, story order stable | Body-only update fails |
| C4 | Same tag, checkbox plus text | Whole refusal | Partial fan-out fails |
| C5 | Placeholder reset after a bound date and a selected list | `fullDate` cleared, selection cleared, prompt shown, bound scalar empty | Relying on `set_value_metadata(None)` alone fails |
| C6 | Required-empty reset, then protect. Separate incremental fill | Reset ok, protect err, fill ok | Treating protect as fill, or rejecting partial fill, fails |
| C7 | Two repeat items | Count 2 after reset | Structural rebuild fails |

Also assert positioned text and render checks in the local dense test where the deterministic renderer already runs. XML-only success is not enough for A7. A7 itself stays pending.

Omitted consumers: a second form tree, a report wrapper, a reason enum, and a location getter. The historical Q1 recommendation of native-only scope stays in the question table as history. WASM and CLI gain no new authoring method. Existing consumers retain preservation and current-value behavior. Existing body-only setters and listing stay as they are. Structural picture and group scalars stay omitted. Regex, persisted schemas, structural reset, and a password API stay rejected under the Q5 answer.

## Rejected alternatives

- A persisted rule schema, regex or range checks, custom callbacks, structural reset, or a password API. Q5, user answer on 2026-10-23, keeps the proposed contract and rejects those alternatives.
- `FormFieldRule.default` plus `placeholder` as two live fields. Replaced by `FormResetValue` so value and prompt are mutually exclusive.
- A report wrapper around `Vec<FormValidationIssue>`.
- Modern drop-down `SelectedIndex` without an explicit revision.
- Looping only the main-body setter and calling that all-story support.
- Inheriting older native form captures for this dense authoring.

## Test plan

Backlog test gate, verbatim: differential. A dense source-built form behaves and renders like the pinned Word form across fill, reset, protect, and reopen operations.

| Timing | Category | Test | Asserts |
|---|---|---|---|
| local scoped | regression | `dense_authored_form_fill_reset_and_protect_share_one_model` | One model across the four operations, plus positioned text |
| local scoped | regression | `form_batches_reject_invalid_values_without_partial_publication` | Atomic refusal, incremental fill, late related-owner rollback |
| local scoped | regression | `form_selection_and_updates_use_the_same_physical_story_owner` | Cross-story fan-out and heterogeneous tag refusal |
| local scoped | regression | `form_reset_preserves_repeat_count_and_editable_ranges` | Count, ranges, required-empty protect refusal |
| local scoped | regression | `form_reopen_keeps_defaults_placeholders_and_bound_values` | Defaults, prompts, and bound values after reopen |
| local scoped | regression | `test_form_rules_fill_reset_and_protect_public_api` | All four operations on the public Python surface |
| local scoped | regression | `test_form_batches_preserve_owners_and_refuse_invalid_targets` | Related owners, held handles, frozen issues, no-op, and refusal |
| local scoped | regression | `test_form_protection_and_permission_composition` | Protection getter and three methods, permission, primitive errors |
| local scoped | typing | typed public usage of the factories | Existing `crates/rdocx-py/tests/typing_smoke.py`. Behaviour tests stay in `crates/rdocx-py/tests/test_core.py`. No temporary string runner and no new tracked file |
| final sprint | oracle | Word 16.114 dense form, independent PDF export, matched fonts. Python-created counterpart uses the identical criterion and is not old F-286 reuse | Q6 selected gate. Equal page count, exact text and checkbox semantics, per-page SSIM at least 0.95 at 144 DPI, matched fonts, deterministic drawing, and documented margin alignment. Native comparison remains unrun. Metrics unchanged |
| final sprint | risk rider | Integrated `/verify --full`, hash harness, strict mypy and stubtest, WASM, genuine publish dry-run with archive under 10 MiB, and the union of riders. Native Word only in that final union | Once per sprint. Scoped public API tests stay on this feature |
| before final freeze | metadata | `python3 -m unittest scripts.test_sprint_workflow.SprintWorkflowTests.test_modern_docx_capability_matrix_has_closed_classifications_and_evidence scripts.test_sprint_workflow.SprintWorkflowTests.test_every_incomplete_modern_docx_row_has_one_live_owner` | DOCX-056 cells, classification, evidence, and owner match the implemented and tested scope. `Y` means that bounded operation exists today. Native Word parity stays explicit debt. Do not mark the row complete only so the assertion passes |
| after completion metadata | metadata | `python3 -m unittest scripts.test_sprint_workflow.SprintWorkflowTests.test_every_incomplete_modern_docx_row_has_one_live_owner` | Backlog and current-sprint status agree with the DOCX-056 owner. A previous green run does not prove the new status. `capability_owner_problems` still rejects an incomplete owner that is done |

Tests land in the existing integration entrypoint. No new test binary.

## HLD impact

- `docs/hld/02-scope-and-non-goals.md`, row DOCX-056 for F-289 (transient rules, no password, no custom rule engine)
- `docs/hld/03-architecture.md`, "What stays put" and "Facade conventions"
- `docs/hld/04-opc-and-packaging.md`, "The package"
- `docs/hld/10-bindings-spec.md`, "Python API shape" and "Live Word sections and rich related owners"
- `docs/hld/12-testing-strategy.md`, "Test taxonomy" and "Typed content-control native Word qualification"

## Risk routing

- Parser or serialiser. Preserve unknown control, permission, and protection bytes. Clear `fullDate` explicitly.
- Public API. Semver note. Genuine publish dry-run with archive under 10 MiB, only in the final union. Rust and Python are both in scope under Q1. The exact surface is `## Python API contract`. The prior native planning basis remains 8 to 11 developer-days across the sprint. Q1 Python scope is an explicit planning estimate of 4 to 6 additional developer-days, for an expanded planning total of 12 to 17. That figure is an estimate, not a measurement, and it is not an undisclosed capacity promise. README measured package inventory may be recomputed by the existing `README.md` and `scripts/readme_doctests.py` only if that archive change requires it. Do not fabricate those values. No Cargo dependency change and no new module.
- External oracle and compatibility. Pin Word 16.114. Distinguish stored values from Word enforcement of library required rules.
- New projection over controls and legacy inventory. Absence, placeholder, and typed values specified above.
- Cached facts. Publishers are fill and reset. `LegacyFormFieldInfo` and each `FormValidationIssue` are owned, so a held pre-edit snapshot stays old. A fresh inventory, the native current value, and the reopened current value agree with the unchanged reset default. Existing Python live handles follow their survival and staleness policies. A refusal or a no-op preserves valid handles. This story does not revive a dead owner. Native page images are pending.
- Multiple parents. Every authorable internal story the existing control and field routes support, including move, clone, unwrap, and repeat.
- Shared consumers. Rust and Python high-level APIs. WASM and CLI gain no new authoring methods. Existing Python, CLI, WASM, and PDF readers retain preservation and current-value behavior.
- Grammar and empty collections. Empty rules, empty assignments, required-empty text, checkbox false, index 0.
- Layout and deterministic fonts for A7. No system-font baseline. No incidental re-record.
- PyO3 is in scope under the Q1 answer. Scoped affected public API tests and the small typed use run with this feature. Strict mypy, stubtest, the installed full suite, WASM, publish dry-run, full verify, and the hash harness run only in the final union. WASM gains no new authoring method. Workspace tests keep `--exclude rdocx-py --exclude rpptx-py`. Codex root semantic validation on 2026-10-23 approved this contract. Implementation waits for the actual F-288 local prefix, and the final union stays pending.

## Hash harness

Expected unchanged for existing fixtures. New tests use new inputs. No baseline re-recording. If a shared setter behaviour from F-288 moves an existing hash, that delta belongs to the F-288 behaviour commit, not to a silent F-289 fixture edit.

## Exclusive resources

Product source and test files already named in this plan stay the product lease. This amendment adds one existing file:

- `scripts/test_sprint_workflow.py`, only the expected live-owner inventory in `test_every_incomplete_modern_docx_row_has_one_live_owner`, and only when DOCX-056 genuinely leaves the incomplete set. Do not change the closed-classification rules in `test_modern_docx_capability_matrix_has_closed_classifications_and_evidence`. Do not add a done-owner exception. Do not edit this file while the row stays incomplete. The edit follows the F-288 lease of the same file and stays serial with F-290.

Before final source freeze, reconcile DOCX-056 honestly. An incomplete row cannot name a done owner. A resolved row cannot keep F-289 as owner. Satisfy that with cells and evidence from the tested scope. `Y` is a complete bounded operation today, not native Word parity. Leave that native debt explicit in the evidence boundary. The Q6 native comparison stays unrun until it actually runs.

## Implementation checklist

- [ ] Add the types and four methods in existing `content_control.rs` and export them. Native and HLD edits stay on files already named in this plan.
- [ ] Bind Python only in existing `crates/rdocx-py/src/document.rs`, `crates/rdocx-py/src/lib.rs`, `python/rdocx/__init__.py`, `python/rdocx/_rdocx.pyi`, and `tests/test_core.py`. No new tracked file, module, or test binary.
- [ ] Resolve tags across authorable stories in physical order.
- [ ] Snapshot legacy ordinals before the batch.
- [ ] Stage on a clone and publish once.
- [ ] Implement `FormResetValue` including an explicit `fullDate` clear.
- [ ] Enforce legacy reset equality with F-288 defaults.
- [ ] Keep repeat count and existing clone-first and last-item policy.
- [ ] Require `ProtectionMode::Forms` and zero issues before metadata publication.
- [ ] Add the five named native tests to the existing entrypoint and the three named Python tests to existing `tests/test_core.py`.
- [ ] Register every frozen class on public `rdocx` and `rdocx._rdocx`, with `_rdocx.pyi`, `__all__`, and `__init__` imports aligned.
- [ ] Leave A7 unclaimed until a real Word run exists. The Q6 gate is selected and unrun.
- [ ] Reconcile DOCX-056 before final source freeze. Update `scripts/test_sprint_workflow.py` only if that row genuinely leaves the incomplete owner set. Rerun the affected status assertion after completion metadata.

## Open questions

| ID | Affected F-IDs and A-IDs | Decision and recommendation | Status and answer/source |
|---|---|---|---|
| Q1 | F-289 A1 and A7. Shared with F-288 and F-290 | Historical recommendation: native Rust only. Python creation and form methods need exact signatures and binding tests before approval, followed by a plan revision. | Answered 2026-10-23 by user answer. Rust and Python new APIs. WASM and CLI gain no new authoring methods. Existing consumers retain preservation and current-value behavior. The exact Python surface is now recorded in `## Python API contract`. Codex root later approved this contract on 2026-10-23. This row keeps the original answer. |
| Q2 | F-289 A3 and A4. Shared with F-288 | Legacy reset authority is the F-288 explicit default or fallback. | Answered 2026-10-23 by source analysis. See F-288 Q2. This plan consumes that repair. Historical provenance is retained. |
| Q15 | F-289 A3 and A4. Shared with F-288 | Use the pre-batch explicit default. Do not add a struct field or a second query. | Answered 2026-10-23 by user answer, after the earlier assembly. See F-288 Q15. F-290 does not inherit this default type. |
| Q4 | F-289 dependency. F-288 A1, A2, and A4 | Historical recommendation: allow F-289 implementation after the named local F-288 prefix, with final Word evidence still open. The wait-for-final-differential alternative needs workflow reconciliation. | Answered 2026-10-23 by user answer. After a verified local provider checkpoint. F-288 local RED then GREEN for all named focused regressions, scoped verification, a clean microscope, and an integrated source ancestor are required before F-289. Final Word differential, full verification, and riders remain pending. |
| Q5 | F-289 A1 through A5 | Historical recommendation: the policy bundle in Approach, including all authorable internal stories, modern scalars plus F-288 legacy, required, type, and list validation, atomic invalid fill and protect, transient caller rules, `FormResetValue`, placeholder restoration, repeat count preserved, existing clone-first append and last-item refusal, metadata-only protection, no regex or custom engine, no persistent schema, and no generated credentials. Alternative persisted rules, richer validators, structural reset, or a password API requires a revised contract. | Answered 2026-10-23 by user answer. Proposed modern-form contract, including transient supplied rules after reopen, all currently authorable internal stories, partial fill, typed, list, and required validation, atomic fill, reset, and protect, typed or placeholder reset, repeat counts unchanged, and metadata-only protection with no password derivation. |
| Q6 | F-289 A7 | Historical question: is SSIM at least 0.95 per page acceptable, with equal page count and exact text and checkbox semantics, deterministic bundled fonts at 144 DPI, after documented margin alignment, against an independently exported Word PDF? The current renderer may miss that bar. A miss stays explicit missing acceptance. No silent downgrade. | Answered 2026-10-23 by user answer. Proposed render gate: equal page count, exact text and checkbox semantics, per-page SSIM at least 0.95 at 144 DPI, matched fonts, deterministic drawing, and documented margin alignment. Native independent Word comparison remains unrun. |

Q16 is answered on F-290 only. Provenance is the 2026-10-23 user choice "Author Word-compatible canonical slots (Recommended)". It is not a decision of this story. Original Q1 through Q15 on this plan stay unchanged.

## Contract closure

Result: user answers recorded. Codex root semantic validation on 2026-10-23 approved this contract at source `a57be365`. Q15 additive defaults, `Cow` readers, and candidate-byte no-op materialization stay the approved contract. Implementation waits for the actual F-288 local prefix. Native qualification and final gates stay pending. This approval does not mark the story completed.

Remaining decisions on this story: Q1, Q2, Q4, Q5, Q6, and Q15 are answered. Q2 keeps its source-analysis provenance. Q15 is the 2026-10-23 user answer shared with F-288. Q3 belongs only to F-288. Q7 and Q8 belong only to F-290. F-290 does not inherit the Q15 default type. No blocked product choice remains. Implementation and final evidence stay pending.

Blockers before approval:

- Q1 selects Rust and Python. The signatures in `## Python API contract` are the approved surface. Implementation waits for the actual F-288 local prefix.
- Q5 is selected. The historical richer alternatives stay rejected.
- Q6 is selected. A7 cannot be marked met by an internal render. The native independent Word comparison remains unrun.
- F-288 is not done. Legacy equality checks have no shipped reader until that field lands.
- Placeholder display versus stored typed value still needs a real Word qualification. If a prompt cannot be represented without a persisted schema flag, revise before approval.
- `set_value_metadata(None)` is cited as insufficient to clear `fullDate`. The clear is new behaviour and must be tested.
- All-story matching is substantive work. The body setter at `content_control.rs:208` is not that implementation.

Operations and proofs: create and lifecycle stay on the existing named APIs (A1, A5, A6). `validate_form` proves A2. `fill_form` proves A2 and A3. `reset_form` proves A3 and A5. `protect_form` proves A4. Each returns the signature in Approach.

Kind coverage: scalar rules cover rich, plain, date, checkbox, combo, modern drop-down, legacy text, legacy checkbox, and legacy drop-down. Picture, group, repeating section, and repeating item are excluded from scalar rules and remain on existing lifecycle APIs. Citation, equation, bibliography, and document-part kinds stay refused by F-286.

Physical inventory: one private dedup on `(part, start, end)`. All controls, including structural ones, are ordered by source-part preorder before the tag filter. The most specific valid owner wins. Overlapping targets refuse. One physical control is one issue and one count. Fan-out is only distinct controls.

Imported state: a placeholder-phase qualified `showingPlcHdr` is semantic empty. A retained `fullDate` or `lastValue` is not an active value. Both stay read-only until reset. A nonempty bound scalar is an issue. Optional placeholder metadata is not rejected only because it was retained. Reset clears those stale hints. A missing checkbox child is false. A present `w14:checked` without `val` is true. The live `CT_OnOff` page states that an omitted `val` defaults to true, and the live `CT_SdtCheckbox` page states that an omitted `checked` child defaults to unchecked. The recorded section labels are 2.6.3.17 and 2.6.3.28. A glyph is not authority. The no-op compares candidate bytes and does not return early on typed equality, so the same selected value can repair display and bound state. An active full date is canonical UTC midnight. The reader normalizes the ISO part, then compares `DateFormat` and `StoreMap`. Unsupported metadata is an issue and the bytes stay. A `dd/MM/yyyy` display such as `23/10/2026` is not fed to the ISO writer. List authority is the bound scalar when bound, otherwise display. Reset explicitly removes `fullDate` and list-selection hints. Missing store ownership is `Err`. Inconsistent current state on a valid rule is one issue and is repaired atomically by a valid fill or reset. Protect still requires zero issues. Codex root semantic validation on 2026-10-23 approved this contract. Implementation waits for the actual F-288 local prefix.

Cached facts: control typed value, `showingPlcHdr`, `fullDate`, list selection, data-binding store, and F-288 current versus default. Publishers are fill, reset, and the existing control setters when used outside these methods. A direct `set` that leaves the tag in place still changes the fresh read. `LegacyFormFieldInfo` and each `FormValidationIssue` are owned, so a held pre-edit snapshot stays old. A fresh inventory, the native current value, and the reopened current value agree with the unchanged reset default. Existing Python live handles follow their survival and staleness policies. A refusal or a no-op preserves valid handles. This story does not revive a dead owner. Native images are due.
