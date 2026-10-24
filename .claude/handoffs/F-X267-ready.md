# F-X267 ready for integration

**F-ID**: F-X267
**Owner**: grok
**Branch**: work/f-x267-grok
**Worktree**: /Users/atulsharma/Documents/projects/tensorbee/rdocx-s954-f-x267-grok
**Base**: 4998e398107e204c7cd81b938af2d3610e58b88b
**Head**: 70573c91030be7073d29e3451383060eb207207a
**SourceHead**: 70573c91030be7073d29e3451383060eb207207a
**Design plan**: .claude/plans/F-X267-design.md
**Microscope**: .claude/reviews/F-X267-packaging-pass-5.md, 0 defects, 0 smells, and .claude/reviews/F-X267-all-pass-3.md, 0 defects, 0 smells
**Verify**: /verify --scoped F-X267, local checks pass, final sprint checks pending
**Hash harness**: deferred to final sprint gate, expected unchanged apart from a later measured M21 digest delta
**Test gate**: remove_part_returns_shared_bytes, reread_matches_write_and_read, reread_matches_shared_zip_name_limits, reread_raises_the_error_a_write_raises, zip_entries_and_write_to_share_pinned_zip_name_limit, hand_built_baseline_uses_fallback, untouched_slide_bytes_stay, fit_text_ignores_unrelated_broken_slide, fit_text_at_selects_size_valid_in_distinct_shared_contexts, footer_proof_is_part_local, parsed_baseline_tracks_real_edits, fx211b_native_opaque_root_edit_retains_invalidated_signature, embedded_replace_counts_commit_zip_write, unnormalized_opaque_part_counts_reread_zip_fallback, commit_counters_follow_real_work, pass
**Checkpoint**: sprint-final-acceptance
**Final checks**: A1 catalogue execution of 248 calls, A2 and A3 release and in-memory timings for 10, 40 and 80 including reopened signed and embedded fallback measurements, independent M21 digest measurement, Word 49, hash harness, full integrated verification, native oracles, installed typing and stub test, WASM and archive qualification, pending
