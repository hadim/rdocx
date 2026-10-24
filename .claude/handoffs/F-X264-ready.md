# F-X264 ready for integration

**F-ID**: F-X264
**Owner**: grok
**Branch**: work/f-x264-grok
**Worktree**: /Users/atulsharma/Documents/projects/tensorbee/rdocx-s954-f-x264-grok
**Base**: 8084d2598786b2a41f1fcce798f5330bca49a679
**Head**: b51510031d0dbb7a04e96c4a5530f5d18f5a6677
**Design plan**: .claude/plans/F-X264-design.md
**Microscope**: .claude/reviews/F-X264-all-pass-2.md, 0 defects, 0 smells
**Checkpoint**: sprint-final-acceptance
**Verify**: /verify --scoped F-X264, local checks pass, final sprint checks pending
**Hash harness**: deferred to final sprint gate, expected unchanged except the declared endnote PAGE output and body table accepted text projection
**Test gate**: native regression 29 plus strengthened A8 (15 observations), OPC 2, Python extension 20, CLI 3, scoped mypy, fmt and clippy, pass
**Final checks**: /verify --full, hash harness, installed typing and stubtest, WASM, archive, native Word, and oracle riders, pending
**A10**: declared non-fix, not passed

## Evidence for the integrator

Root microscope pass 2 is `.claude/reviews/F-X264-all-pass-2.md`. Frozen working diff was 20 tracked files, 3899 insertions and 397 deletions. Binary diff SHA-256 `d78b2536f495250e5ae7ab6b8d31f61f48481aba77d38cecb11e67e28016bd5d`. Approved canonical revision `f778a49f96c4be056c81d7e688efb28666297493` is acknowledged. The local plan stays approved and in progress.

Head `b51510031d0dbb7a04e96c4a5530f5d18f5a6677` is the last source and docs commit. The handoff commit is metadata only and does not change `crates/`.

Inspected native A8 log `/tmp/fx264-a8-exact-s2.txt` SHA-256 `721ed3b1c88989d69166f002cc022c4b2926c7d252e46d552d37b82a66936686`. Artifact SHA-256 `e8be8e6add7f71dc280a9f416e502044489ee796a3d96fb46dce8a9873651bfa`. Regression source SHA-256 `280d847551cd31738ce02c3043232acb815a85296c298595f2604f75f81691ed`. Those hashes match the reviewed final tree. Preserved RED receipts are `/tmp/fx264-red-native.txt`, `/tmp/fx264-red-ref.txt`, and `/private/tmp/fx264-a8-exact.txt` SHA-256 `a603c97e80ffae39679b3fff1b85bb3f8e63430d6bc89d045f876676ca37f3bf`. The namespace diagnostic `/tmp/fx264-a8-snip.txt` is a harness failure, not product RED. Prior component receipts stay in `.claude/scratch/F-X264-progress.md`.

Working-tree SHA-256 of the frozen 20 files is `/tmp/fx264-frozen20.sha256` and matches after the source commits. The untracked extension `crates/rdocx-py/python/rdocx/_rdocx.abi3.so` is not part of the branch.
