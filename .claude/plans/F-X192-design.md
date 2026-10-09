# F-X192, Complete the remaining Word bibliography catalogue

**Status**: approved
**Sprint**: S91
**Size**: L
**Depends on**: F-282

## Problem

The measured native foundation refuses standard non-APA sparse or non-ASCII
operands at `crates/rdocx/src/bibliography.rs:12169`. Localized APA sparse,
repeated and corporate branches refuse at `bibliography.rs:12005`. Sorting
rejects non-ASCII source operands at `bibliography.rs:7781`. Standard selected
and empty bibliography output refuse at `bibliography.rs:2701` and `:2845`.
Citation modifier, multiple-source and script branches also remain unfinished.
These cannot become unsupported-producer cache retention.

## Spec reference

- `docs/hld/02-scope-and-non-goals.md`, citation/bibliography boundary DOCX-049.
- `docs/hld/03-architecture.md`, What stays put, native bibliography ownership and physical field caches.
- `docs/hld/04-opc-and-packaging.md`, The package, bibliography source graph and atomic publication.
- `docs/hld/08-rendering-spec.md`, Word field caches and deterministic script rendering.
- `docs/hld/10-bindings-spec.md`, native bibliography and Python/WASM preservation boundaries.
- `docs/hld/12-testing-strategy.md`, Word corpus, measured bibliography gate.
- `docs/hld/14-development-backlog.md`, F-X192 complete catalogue contract.

## Approach

Retain the existing public API, concrete formatter ownership and staged
transaction. Extend existing bibliography.rs, field.rs and existing regression
entrypoint. No new dependency is selected. No new module, file, trait, generic
or feature flag is needed for known formatting consumers.

Complete the thirteen remaining dense APA selectors from each own completed
native result:1054,1105,1107,1108,1109,1111,1112,1113,1115,1121,2117,2128,2145.
Do not infer neighboring grammars. Complete all seventeen source kinds and223
ordinary selectors across the eleven other styles, retaining pinned editions,
rich properties and IEEE table geometry. Independently vary sparse/repeated
operands, contributors, corporate/plural names, missing dates and mixed scripts.
Complete grouped citations, suppression, locators, prefixes/suffixes, numeric
assignment, disambiguation, multiple-source/empty selection and all locale
precedence/application-default/filter branches.

Derive actual sorting strings and comparator discriminators before extending
bibliography_sort_key. The installed Harvard XSL assembles Last+initials for
up to four people, Year/no-date and Title without delimiters. APA uses its
formatted contributor representation and source-kind-specific title keys.
IEEE uses numeric RefOrder. Existing tuple Last/Title probes do not establish
those string keys. Retained ICU probes tested partial operands and tuples,
not complete Harvard or APA SortingString values. Their failure does not prove
complete-key ICU failure. No complete-key comparator receipt is authenticated.
Default ICU, Core Foundation and ASCII fallback remain prohibited.
The approved implementation route is source-kind-specific native sort-key
assembly plus an independently demonstrated portable comparator and stable
source/citation tie policy. No comparator backend is presumed approved.

Before comparator production edits, run the bounded native key/host study:
fixed first/middle/year/title with varied last names, then reverse title and
vary year independently. Distinct Tag/Guid with equal semantic keys and
independently reversed source/citation order distinguish ties. Concatenation
collision pairs distinguish one string from tuples. Harvard and APA controls
must record exact identities, actual selected style, completed F9 and normal
reopen. Retain numeric styles' separate encounter-order authority.
If a new dependency is demonstrated necessary, revise this plan with exact
versions, data and all dependency riders before adding it. Missing exact
comparator evidence blocks completion and dependent work, not independent
sprint repairs. It is a technical implementation barrier, not a request for
the user to guess a library or waive parity.

Authenticate the existing census rather than repeating broad captures:
`/private/tmp/S90-F282-full-catalogue-oracle-readiness-census-20261009.json`
records223 selectors,17 kinds,3791 records per style,1475 file bindings and
zero census errors. It does not prove arbitrary-input, sparse/context or
comparator parity. The paired Thai/Tibetan Book capture completed F9 and
normal reopen with equal output hashes. Its finite four-source/eight-field
control can be independently audited. own1054/own1105 dense compact context
working files still equal prepared inputs and need actual completed controls.

Keep source identities, raw XML, namespaces, physical begin/separate/end
owners, rich paragraphs and IEEE cells/grids. Stage the entire update through
update_bibliography_transaction at `bibliography.rs:16186`. One unfinished
recognized standard owner aborts the whole candidate. Preserve locked and
producer extension contracts. Prove public source mutation, repeat update and
library save/reopen against completed controls.

## Rejected alternatives

- Alias adjacent locale grammars or substitute modern style editions.
- Lookup expected native caches in production or execute producer XSL.
- Treat incomplete standard branches as unsupported extensions.
- Assume a Unicode-capable comparator is Word-equivalent.
- Repeat the full dense capture matrix despite existing indexed authority.

## Test plan

**Test gate**: differential, full_bibliography_catalogue_matches_pinned_word.

| Category | Test | Asserts |
|---|---|---|
| differential | full_bibliography_catalogue_matches_pinned_word | Every style/kind/selector row has exact identity, instructions, selection, order, text, rich properties and reopen semantics, without skips/assumed aliases/cache fallback |
| differential | bibliography_sparse_and_repeated_operands_match_word | Missing/repeated properties, roles, names, dates and scripts match independent controls |
| differential | citation_groups_and_modifiers_match_word | Grouping, suppression, locators, prefix/suffix, disambiguation and numeric interaction |
| differential | bibliography_sort_discriminators_match_word | Exact keys, locales, normalization, independent source/citation order and stable ties |
| differential | bibliography_locale_precedence_matches_word | Source, field, omitted/default-zero, explicit default and filtering remain distinct |
| regression | one_unfinished_bibliography_owner_preserves_every_owner | Mixed-owner failure retains package, caches and revision |
| round-trip | bibliography_mutation_and_repeat_updates_reopen_consistently | Public CRUD, repeat update and ordinary save/reopen retain published identity and cache |
| round-trip | bibliography_namespace_aliases_preserve_opaque_xml | Equivalent prefixes produce equal output with retained opaque bytes |
| integration | ieee_bibliography_preserves_table_and_story_geometry | Rich labels/grid/cells and first/interior/end paragraph context |
| golden | bibliography_render_uses_matched_content_and_deterministic_fonts | Declared finite script controls compare matched content and actual rendering |

Pin Microsoft Word for Mac16.113.2 build16.113.26092012. Bind source/native
output hashes, actual update/reopen provenance and normalization classification.
Use existing test files and source-built inputs. No cache implies completion.

## HLD impact

- `docs/hld/02-scope-and-non-goals.md`
- `docs/hld/03-architecture.md`
- `docs/hld/04-opc-and-packaging.md`
- `docs/hld/08-rendering-spec.md`
- `docs/hld/10-bindings-spec.md`
- `docs/hld/12-testing-strategy.md`
- `docs/hld/14-development-backlog.md`

## Risk routing

- Parser/serializer: HLD04/HLD06, schema order, qualified owners and exact
  opaque subtree retention.
- Layout/shaping: HLD08, deterministic fonts, actual finite script controls
  and declared independently reviewed rendering deltas.
- External oracle: exact Word pin and completed-operation source-bound evidence.
- Public API: existing additive pre-1.0 foundation, package dry-run and10MiB
  archive checks. No new public API is planned.
- Crate graph: existing facade dependencies only. No reversed format edge.
- Any earned collation dependency requires design revision with exact MSRV1.93,
  WASM, no-default, graph, data/licence/provenance, supply-chain and packaging
  riders before implementation. No dependency is selected by this approval.

## Hash harness

Expected unchanged for the existing49 entries. Diagnose unexplained deltas.
An intentional change needs separately labelled, declared and reviewed evidence.

## Exclusive file claims

- `crates/rdocx/src/bibliography.rs`
- `crates/rdocx/src/field.rs`
- `crates/rdocx/tests/regression_test.rs`
- Exact listed HLD sections, oracle capture and all Cargo/test execution

## Implementation checklist

- [ ] Authenticate census and completed locale/script owners.
- [ ] Finish remaining own dense APA grammar consumers.
- [ ] Complete eleven style families and sparse/contributor branches.
- [ ] Complete citation, selection and application context.
- [ ] Derive exact style keys, comparator and independent tie authority.
- [ ] Revise before any newly justified dependency.
- [ ] Pass full catalogue plus atomicity, mutation, reopen and rendering gate.
- [ ] Run scoped verification and independent zero-finding microscope.

## Open questions

No unresolved product decision. The user authorized the full deferred contract.
The comparator and own-locale native evidence are explicit technical execution
barriers. This approval does not permit approximation, a new dependency or
completion before those barriers are resolved.

## Bounded native key and tie evidence

Word16.113.2 build16.113.26092012 authenticated ten existing name/title/year
controls and six new four-source Harvard Anglia and APA Sixth controls.
The new controls expose Publisher identity markers outside both installed
sort-key builders. All six explicitly completed F9 and preserve exact bytes
through normal reopen without F9. Their nine field instructions and four
source trees survive, apart from native RefOrder. Immutable input hashes,
complete artifact hashes and installed style hashes are recorded in
/private/tmp/S91-F192-key-tie-study/capture-receipt.json and independently
checked before recording this plan evidence. The read-only study appendix is
.claude/scratch/F-X192-native-key-comparator-study.md.

For both styles, baseline identity order is19,20,17,18. Source reversal yields
20,19,18,17. Citation reversal retains19,20,17,18 despite reversed RefOrder.
These controlled equal-key pairs therefore retain source collection order.
Do not use Tag, Guid or citation encounter order to break their ties. This
result is bounded to the tested Book operands and contexts.

Installed Harvard SortingString at HarvardAnglia2008OfficeOnline.xsl:6608
concatenates contributor representation, Year and Title without delimiters.
The two distinct operand combinations Boundary+A+2020+Collision and
BoundaryA+2+020+Collision produce BoundaryA2020Collision. Their native tie
reverses with source order. This disproves the current independent operand
tuple as a sufficient Harvard key. APA uses its formatted contributor and
source-kind-specific title/year assembly at APASixthEditionOfficeOnline.xsl:5826.
Its tested personal/corporate pair is source-stable, but exact intermediate
formatted-key byte equality was not independently extracted.

The existing17-name controls show style-specific mixed-script order and
suffix-sensitive name-equivalent pairs. They do not license a shared raw
Last-name comparator. Portable comparator parity, nonempty/repeated initials,
all source kinds/locales and remaining dense grammar branches are still
unproved. No dependency or production change is authorized by this finite
study. Preserve the existing exact-evidence implementation barrier.
