# Incremental delivery obstruction: repository findings

Observed 2026-09-16 at main `71a5e695`; scope is the installed workflow, not
Preferences implementation or a new product architecture.

- `check_workflow_delivery.py` validates every enrolled delivery. The completed
  pilot remains enrolled and its contract input roots include all of `crates`.
- `workflow_delivery_proof.py` compares that proof's retained input manifest to
  the current candidate. A legitimate later Rust change therefore invalidates
  old acceptance, even when source ownership has already been promoted.
- `workflow_delivery_checkpoints.py` also escalates structure/readiness to
  activation/verification whenever source inputs differ from the trusted base.
  This conflates developing an implementation with completing its delivery.
- The committed `docs/reviews/preferences/gp-cm05-implementation/daemon-routing.md`
  records a tested routing patch blocked by these historical-input checks.
- The product agent reported GUI-launch candidate
  `0eb46520fec663d6cd532a658bdad500ec8ded25`, with a cold-cache DOA2526 launch and
  focused tests, blocked by historical freshness plus three missing source paths.
  This lane inspected its five-file diff, but did not independently repeat its
  GUI execution or accept the product result.
- The installed runner is pinned under `.git/datum-wdq/trusted/a467ccc4...`.
  Merely changing a working-tree script cannot repair the installed gate.

The owner's instruction is a subtractive, bounded correction: permit ordinary
development while preserving honest completion checks. The owner explicitly
approved a one-time correction-commit hook bypass after direct checks, not a
global hook disable or private trust replacement. Keep the product agent's work
and the stopped seven-gap candidate separate.

Historical acceptance should remain attached to its immutable tested revision.
Current implementation should undergo relevant regression checks; advancing
verification/review/acceptance must still require current evidence. The accompanying
PM044 candidate defines that boundary and its limited GUI permission correction.
