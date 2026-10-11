# F-X206, eight failure batch, pass 79

**Reviewed**: External R2 proposal receipt36afc5be1cc567777b87e5fc558f2c21de7aeba5b17a8d1773557ccbfe322d3c, complete diff70825c3530308c5462ff2dc43908c56f7bf71425b2355574b86fba90e81a38f4 against sourceb284fcad7528a8e86fec44714cea006aca8dc8bf52ac0bcabb756895ddcbf7ce. No source adoption or runtime occurred during this review.

**Verdict**: 1 defect, 0 smells, 0 nitpicks. Adoption is withheld.

## Defects

- D1, foreign relationship binding is still discarded before serialization. Candidate `/private/tmp/S91-F-X206-eight-failure-batch-r2-prepared/candidate/crates/rpptx-oxml/src/placeholder.rs:186` passes FIXED_SHAPE_TREE_PREFIXES, including r, into self_contained_attributes. Actual `crates/rpptx-oxml/src/namespace.rs:186` removes every fixed declaration regardless of its URI, and :204 skips its inherited binding. The new admission at candidate placeholder.rs:376 therefore permits foreign r but discards its declaration. The writer at :243 then inserts canonical R_NS, changing r:keep and r:payload expanded identities. The proposed standalone discriminator at :411 cannot pass its intended preservation contract. Preserve the admitted source binding through extraction and verify both standalone and inherited ownership. Review the analogous application-properties extraction at candidate :304 and :357 against its writer before widening admission there.

## Smells

None.

## Nitpicks

None.

## Not found

- Equality filters retain noncanonical declarations, typed values, other attributes and raw children. They change semantic equality only, not serialized bytes.
- Click cleanup stages the package and uses existing relationship reachability before publishing media state. Shared targets remain reachable.
- Field fixture corrections retain its GUID, text, child ownership, adjacency and opaque marker bytes.
- Both archive SHA expectations remain unchanged. The inverse recipe is explicitly unexecuted and does not establish historical attribution.

No feature completion or test execution credit is granted by this preparation review.
