# F-250, all, pass 1

**Reviewed**: complete uncommitted working-tree diff against `HEAD`, 3 files,
565 additions and 3 deletions
**Verdict**: 1 defect, 2 smells, 0 nitpicks

## Defects

### D1, removing a predecessor drops the surviving section's inherited story
`crates/rdocx/src/document.rs:12661`
`crates/rdocx/src/document.rs:12742`

Word resolves an omitted header or footer variant from the preceding section.
For example, section 0 can explicitly reference header A while section 1 omits
the default header and therefore inherits A. Removing section 0 clears its
boundary without materializing A on the surviving section 1. The reachability
pass then sees no active modeled reference and, when A is facade-owned, removes
its document relationship and part. A preserved producer part is retained, but
the surviving section still loses its effective header because it has no
reference after the predecessor disappears. Non-final removal must preserve the
following section's effective same-type header and footer variants before it
deletes the predecessor boundary and prunes the graph.

## Smells

### S1, the new section handles are forwarding-only wrappers
`crates/rdocx/src/document.rs:1776`
`crates/rdocx/src/document.rs:1793`

`Section` and `SectionRef` contain only a `CT_SectPr` borrow and expose only
`properties` forwarding methods. They add a second type to follow without
reducing any cases, which is the forwarding-only indirection prohibited by the
repository structural rules. The ordered accessors can return the concrete
immutable and mutable `CT_SectPr` borrows directly unless the handles gain
section behavior that justifies their existence in this feature.

### S2, the round-trip gate does not prove independent story references
`crates/rdocx/tests/integration_test.rs:491`

The planned round-trip test checks only the total number of header and footer
references on each reopened section. It does not assert each reference's kind,
relationship target, or story content, and only two of the three surviving
sections have a reference at all. A regression that swaps the surviving header
and footer roles or aliases references onto the wrong story can therefore keep
the asserted counts. The gate should resolve each surviving reference and
compare its expected variant, target, and content after reopen.

## Nitpicks

None.

## Not found

Ordinal lookup and insertion, final-owner promotion, staged failure atomicity,
schema child order, raw section XML retention, empty facade-boundary cleanup,
shared explicit-reference retention, producer-owned story retention, panic
safety, and arithmetic hazards produced no additional findings.
