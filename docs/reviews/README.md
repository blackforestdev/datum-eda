# Review records and temporary proof output

The owner's 2026-10-13 direction is controlling: screenshots used to demonstrate
work are temporary. Do not commit or push them, recordings, raw capture archives,
or compressed diagnostic bundles. The pre-commit temporary-artifact gate enforces
this rule, including forced additions. It also rejects compressed archives
anywhere in the repository so moving a bundle into testdata cannot evade the rule.

Write temporary proof to an ignored, task-owned directory under `target/` or to
an external temporary directory. Keep it only while needed for debugging or
review, then remove it. Before removing local output, check ownership and active
processes; never delete another session's live work or clean active Cargo targets.
Do not preserve duplicate build trees indefinitely as proof archives.

Commit concise text findings, relevant source revisions, reproducible commands,
review disposition and limitations. Hashes can identify artifacts that existed
during review; they do not promise permanent artifact availability. Screenshots
alone do not establish acceptance, and this storage policy does not waive required
runtime checks or independent review.

Automated image comparison inputs under `crates/gui-render/testdata/golden/` and
the two owner-approved design references under `docs/gui/reference/` remain test
and design inputs. Incidental `revision/running-app/` captures are excluded.
Authored product images belong under `crates/**/assets/`; authored diagrams are
also distinct from temporary work proof. Do not relocate screenshots into these
input directories to bypass the policy. Other raster images and recordings are
ignored and rejected by the gate even in newly named proof directories.

## Historical records

This cleanup removes accumulated proof images and compressed bundles from the
current tree. Older review records retain their original observations, paths and
checksums as historical records. Those paths no longer guarantee files in a fresh
checkout. Do not claim a historical bundle was replayed when it is unavailable;
regenerate only the evidence actually needed for a current decision.

The retired manual `scripts/workflow_delivery_pilot_capture.py` and
`scripts/workflow_delivery_pilot_package.py` require the removed historical
`research/process-quality/evidence/wdq-c01-current/native-fixture.tar.gz`.
They cannot replay that pilot from the cleaned checkout. Historical diagnosis
requires the pre-cleanup tree; this does not reinstall retired WDQ enforcement
or authorize a historical pilot replay as a prerequisite to current work.

The pre-cleanup tree at `cfd24461` contains the historical files. Git history has
not been rewritten, so historical objects still contribute to clone size. A
separately authorized history cleanup is required to remove those remote objects.
This change does not complete S5A or change its acceptance requirements.
