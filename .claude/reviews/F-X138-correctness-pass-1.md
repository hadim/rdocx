# F-X138, correctness, pass 1

**Reviewed**: `work/f-x138-codex` working diff against claim base `4a2e4ac2`, 31 files, 13,156 insertions and 6,377 deletions
**Verdict**: 1 defect, 1 smell, 0 nitpicks

## Defects

### D1, CLI replacement count has conflicting scope statements
`crates/rdocx-cli/README.md:92`

The older paragraph says the count covers text boxes without a location limit and that deleted text is never counted. The following paragraph states the implemented body, header and footer text-box reach and its deleted-run exception. A user cannot tell which `--expect` count to rely on. Keep one statement matching the implementation.

## Smells

### S1, DrawingML intent does not name its supported producer exception
`docs/hld/05-drawingml-model.md:41`

The text says unsupported producer `AlternateContent` remains opaque unless authored by the facade, then immediately describes reading a producer text-box Choice as a story. Clarify that text-box story projection is the supported exception while other producer branches remain opaque.

## Nitpicks

None.

## Not found

Correctness, run/body coordinates, XML preservation, panic paths, regression gates and structural rules produced no other cited finding. The source-built Issue 163 gate failed before implementation and passes now. The 49-case hash harness is unchanged.
