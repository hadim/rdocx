# F-X230, Group comparison changes for readable revisions

**Status**: approved
**Sprint**: S92
**Size**: M
**Depends on**: F-X229

## Problem

Granular alignment preserves whitespace anchors between replacements and emits
each changed word pair independently (`crates/rdocx/src/comparison.rs:4151`,
`crates/rdocx/src/comparison.rs:4216`). Coalescing only joins adjacent identical
actions with identical run ownership (`crates/rdocx/src/comparison.rs:4457`).
Inline controls use the same pattern (`crates/rdocx/src/comparison.rs:6378`).
Issue372's rewritten sentence produces18 word revisions instead of readable
changed stretches. Run comparison already yields two revisions and must retain
that behavior. This draft does not authorize starting before F-X229 completes.

## Spec reference

- `docs/hld/14-development-backlog.md`, "F-X230, Group comparison changes for readable revisions".
- `docs/hld/03-architecture.md`, "What stays put", deterministic comparison and preserved owner boundaries.
- `docs/hld/10-bindings-spec.md`, "Native Word facade stability", comparison granularity, ignored content and package reconstruction paragraphs.
- `docs/hld/12-testing-strategy.md`, "Test taxonomy", document-comparison policy matrix and exact normalized revision records.

## Approach

After F-X229 completes, adapt only grouping commit
`7ebb5120aceee40d765a408faf19e1fa4bcc1724` from
[PR374](https://github.com/tensorbee/rdocx/pull/374), by `hadim`, assessed at head
`3d7a3c9b703042b5ad53e586b94fe01eb843daf0`. Attribute this reuse independently
from the correctness subset. Use existing comparison and regression files only.

Add a private linear `grouped_text_alignment` pass over attributed unit alignment
before existing coalescing and emission. Use the existing concrete alignment and
unit types, with no new public surface, trait, module, dependency or production
file. Avoid a new generic helper unless two current concrete instantiations
justify it. A concrete text-word helper is sufficient.

1. Partition alignment at existing shell cuts for hyperlinks, controls, bookmarks
   and comment ranges. Never group across these boundaries.
2. Identify changed plain-text stretches within each segment. Fold whitespace
   anchors between changes. Fold other unchanged anchors only when their word
   count is strictly smaller than the changed sides required by the donor rule.
   A one-word anchor between one-word replacements stays unchanged.
3. Emit all deletions before all insertions within a grouped stretch. Retain
   original and edited formatting independently. Keep owner-limited coalescing,
   so different run owners can remain consecutive distinct wrappers.
4. End stretches at fields, drawings, raw children, ignored units, formatting-only
   differences and formatting-ignored replacements. Preserve each boundary's
   physical and semantic owner.
5. Apply the same policy inside supported inline controls. Leave run granularity
   byte-identical and retain package-wide acceptance and rejection postconditions.
6. Use a forward scan with linear temporary storage, without another LCS or
   repeated growing-suffix scans. Keep deterministic revision allocation.

The exact Issue372 sentence should have11 word revisions and the expected ending
shown below. Character granularity uses the same grouping policy, while the
existing run and default comparison contract remains unchanged.

## Rejected alternatives

- Replace whole runs, hiding light edits.
- Add a paragraph similarity threshold, introducing an unspecified new policy.
- Use character-length cleanup, folding retained words such as "and" and
  "inspection" in the reported example.
- Move fields or markers to allow grouping, changing their ownership.

## Test plan

The **test gate** is **regression**, as declared in the backlog.
Use the existing `crates/rdocx/tests/regression_test.rs` binary.

| Category | Test | Asserts |
|---|---|---|
| regression | `word_comparison_groups_a_rewritten_stretch_into_one_deletion_and_one_insertion` | Exact Issue372 OLD and NEW sentence yields11 word revisions and the required markup, two unchanged run revisions and successful character reconstruction. |
| regression | `word_grouping_keeps_a_one_word_edit_and_long_anchors` | One-word change remains minimal, strictly shorter anchor folds, long and equal anchors remain, anchor beside pure deletion remains. |
| regression | `word_grouping_never_folds_a_field_a_hyperlink_or_a_formatting_change` | Simple and complex fields, hyperlink shells and formatting boundaries stop grouping. |
| regression | `word_grouping_preserves_bookmark_comment_and_raw_boundaries` | Bookmark, comment range, foreign raw child and PI retain exact ordering and ownership through reopen. |
| regression | `word_grouping_respects_ignored_units_and_non_text_content` | Field, formatting and whitespace ignores, drawings and hard breaks retain left-biased policy and owner boundaries. |
| regression | `word_grouping_applies_inside_an_inline_control` | Mixed rewrite inside an inline control groups identically while preserving its shell once. |
| regression | existing comparison policy matrix | Only deliberately changed word and character records change. Run/default equality and unchanged controls remain exact. |
| regression | F-X229 repair matrix | All hyperlink, table, empty and trailing paragraph cases still reconstruct exactly. |
| round-trip | existing granular unmodelled XML preservation tests | Opaque XML and relationship ownership survive tracked, accepted, rejected and reopened paths. |
| unit | grouping scan control | Multiple alternating text stretches retain deterministic output with one forward scan and linear temporary storage. |

Use these exact Issue372 inputs:

```text
OLD: On receipt, the Sponsor inspects the material and notifies the site of any specimen that does not pass inspection, with the reason.
NEW: After receipt, the Sponsor confirms the delivery and reports to the laboratory any specimen that fails inspection.
```

The word markup must end with:

```text
and [-notifies the site of-]{+reports to the laboratory+} any specimen that [-does not pass-]{+fails+} inspection[-, with the reason-].
```

Assert independent expected markup, revision kinds, counts and order. Do not
generate expected results by calling grouping again. Save and reopen each
redline, independently resolve both copies, then save and reopen the outcomes.
Inspect properties and relationships as well as text. Cover mixed edits and
unchanged documents, default options, independent ignores and shell boundaries.
Do not use timing thresholds to assert complexity.

Run `cargo check -p rdocx --all-targets` while iterating. Execute focused tests
with `RUST_MIN_STACK=8388608 cargo test -p rdocx --test regression_test <filter>`,
using `word_`, `comparison_policy_`, `granular_` and the F-X229 repair filters.
Run the native regression binary before completion. Pass
`/microscope F-X230 --working` and `/verify --scoped F-X230`.

## HLD impact

- `docs/hld/03-architecture.md`
- `docs/hld/10-bindings-spec.md`
- `docs/hld/12-testing-strategy.md`

Document grouping and strict anchor policy, formatting and owner boundaries,
run compatibility and intentional normalized revision output changes.

## Risk routing

- **Any parser or serialiser**: read `docs/hld/04-opc-and-packaging.md` and
  `docs/hld/06-presentationml-model.md`. Verify schema order, revision nesting,
  fixed-prefix output and prefix-tolerant input. Require exact opaque-subtree
  round trips and physical ownership checks across every listed boundary.
- No public API, crate edge, binding, feature flag, production file or layout
  behavior changes. Those extra riders do not apply.
- No new external oracle is planned. Issue372 markup is acceptance evidence,
  not execution of Word. Retain existing applicable pinned oracle checks.

## Hash harness

Expected unchanged, all49 entries. Samples do not invoke comparison. Intentional
word and character wrapper, count and revision-ID changes receive separate review
and the F-X230 commit. Run/default output remains exact. No baseline rerecording
is planned.

## Implementation checklist

- [ ] Confirm F-X229 completed with dependency-prefix evidence.
- [ ] Establish current Issue372 output.
- [ ] Adapt only grouping commit and relevant tests.
- [ ] Add bookmark, comment, opaque-child and ignored-unit boundary controls.
- [ ] Prove both property and relationship outcomes through reopen.
- [ ] Review intentional normalized output changes separately.
- [ ] Pass independent microscope and scoped verification.

## Open questions

No blocking product question. For consolidated approval, assume strict word-count
anchor folding, whitespace folding within owner boundaries and11 revisions for
the exact reported sentence. Run granularity stays unchanged. F-X229 remains an
implementation dependency even when this plan is approved during sprint design.
