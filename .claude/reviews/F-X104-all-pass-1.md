# F-X104, all aspects, pass 1

**Reviewed**: working-tree implementation, 9 files and 597 changed lines
**Verdict**: 1 defect, 0 smells, 0 nitpicks

## Defects

### D1, the named differential gate does not execute the native backend and round-trip assertions

`crates/rpptx/tests/integration.rs:14012`

The design contract assigns the LibreOffice comparison, round-trip XML, PDF
alpha, and repeated deterministic PNG assertions to
`picture_alpha_mod_fix_matches_presentation_renderers`. The named test only
runs the LibreOffice pixel checks. The native and round-trip assertions live in
a separate test at `crates/rpptx/tests/integration.rs:13970`, so running the
required gate by exact name does not prove the complete contract. Make the
named differential gate execute the regular backend gate before it invokes the
external oracle.

## Smells

None.

## Nitpicks

None.

## Not found

The typed amount validation, namespace shadowing, first-child ownership,
schema ordering, raw sibling preservation, background and preview propagation,
group opacity composition, public API scope, panic safety for untrusted input,
and structural indirection checks produced no other findings.
