# Production Cleanup Manifest

Status: active cleanup manifest
Date: 2026-06-19

This manifest classifies the dirty worktree after product-mechanics spec
integration. It is intended to prevent accidental deletion of real production
assets while making the repo reviewable and commit-ready.

## Cleanup Verdict

Do not run `git clean` against this tree.

The untracked set is mostly active implementation, tests, visual baselines,
specification, and governance work. Production cleanup means:

- track source-of-truth work
- archive or retain historical audit evidence deliberately
- delete only ignored/generated scratch files
- verify code and spec gates before staging/commit

## Track As Production Source

These files/directories should be tracked as part of the current production
scope unless a later owner review finds a concrete defect.

### Product Mechanics And Specs

- `docs/DATUM_PRODUCT_MECHANICS.md`
- `docs/contracts/`
- `docs/decisions/`
- `docs/SPEC_INTEGRATION_CONDUCTOR_REPORT.md`
- `specs/ENGINE_SPEC.md`
- `specs/MCP_API_SPEC.md`
- `specs/CHECKING_ARCHITECTURE_SPEC.md`
- `specs/STANDARDS_COMPLIANCE_SPEC.md`
- `specs/PROGRESS.md`
- `specs/SPEC_PARITY.md`
- `specs/spec_parity_manifest.json`
- `scripts/check_spec_parity.py`

### Text, Visual, And GUI Assets

- `crates/engine/assets/fonts/`
- `crates/engine/src/text/`
- `crates/engine/testdata/golden/text/`
- `crates/gui-app/tests/visual_shell.rs`
- `crates/gui-render/src/bin/datum_visual_fixture.rs`
- `crates/gui-render/src/dim_policy.rs`
- `crates/gui-render/src/render_contract_tests.rs`
- `crates/gui-render/src/visual_capture.rs`
- `crates/gui-render/src/visual_diff.rs`
- `crates/gui-render/src/visual_manifest.rs`
- `crates/gui-render/src/visual_runner.rs`
- `crates/gui-render/tests/`
- `crates/gui-render/testdata/golden/board/`
- `docs/gui/DATUM_GUI_VISUAL_REGRESSION_HARNESS.md`
- `docs/gui/DATUM_TEXT_ENGINE_FIDELITY_FIXTURES.md`
- `docs/gui/DATUM_TEXT_ENGINE_PHASE_2_BRIEF.md`
- `docs/gui/DATUM_TEXT_ENGINE_PHASE_2_IMPLEMENTATION_PLAN.md`
- `docs/gui/DATUM_TEXT_ENGINE_OUTLINE_FILL_ATTAINMENT_NOTE.md`
- `docs/gui/DATUM_TEXT_ENGINE_OUTLINE_FILL_OWNERSHIP_NOTE.md`
- `docs/gui/DATUM_TEXT_ENGINE_OUTLINE_GEOMETRY_CONTRACT_NOTE.md`
- `docs/gui/M7_IMP_014_IMPLEMENTATION_PLAN.md`
- `docs/gui/M7_IMP_014_IMPORTED_TEXT_NORMALIZATION_BRIEF.md`
- `docs/gui/M7_INT_001_INTERACTION_STABILITY_BRIEF.md`

### Import, CLI, DRC, And Route Work

The modified tracked files under `crates/cli/`, `crates/engine/`,
`crates/engine-daemon/`, `crates/test-harness/`, and `scripts/` appear to be
active implementation work, not obsolete artifacts. They should be reviewed and
staged by logical implementation slice rather than deleted or reverted.

New untracked import helpers should be tracked with the KiCad/import slice:

- `crates/engine/src/import/kicad/board_objects.rs`
- `crates/engine/src/import/kicad/net_refs.rs`

## Retain Or Archive As Evidence

These documents are no longer controlling implementation specs after the
integration pass, but they preserve audit trail and rationale. Preferred
cleanup is to keep them tracked under an audit/history area, not delete them.

- `docs/audits/scope-integration/NORTH_STAR_PROJECT_AUDIT.md`
- `docs/audits/scope-integration/DATUM_PRODUCT_MECHANICS_RESEARCH_SYNTHESIS.md`
- `docs/audits/scope-integration/DATUM_PRODUCT_MECHANICS_IMPLEMENTATION_READINESS_AUDIT.md`
- `docs/audits/scope-integration/DATUM_SCOPE_INTEGRATION_READINESS_AUDIT.md`
- `docs/audits/scope-integration/DOC_CODE_PARITY_DELTA_REPORT.md`
- `docs/audits/scope-integration/DATUM_PRODUCT_MECHANICS_DOCUMENTATION_GOALS.md`
- `docs/audits/scope-integration/DATUM_PRODUCT_MECHANICS_REVIEW_AGENDA.md`
- `docs/audits/scope-integration/STANDARDS_DOMAINS_3_4_INTEGRATION_GUIDANCE.md`

If these are moved later, references from controlling specs and reports must be
updated in the same change.

## Delete Candidates

No untracked source, doc, fixture, or script file is currently classified as a
safe delete candidate.

Ignored/generated local cache may be removed outside the production commit:

- `scripts/__pycache__/`
- other ignored Python bytecode caches

Do not delete committed or untracked `.golden.png` files in this tree unless
the visual regression owner removes the corresponding tests and fixture
manifests in the same change.

## Cleanup Actions Taken

2026-06-19:

- Removed stale CLI imports and visibility mismatches that produced warnings.
- Removed obsolete legacy route explain modules that were no longer referenced
  after the current generic route explain wrapper became the active path.
- Verified no stale references remain for the removed modules.
- Re-ran `cargo check --workspace`; it passed with no warnings.
- Moved historical scope-integration audit/research artifacts under
  `docs/audits/scope-integration/` and added an archive index.
- Updated references to those archive paths; no stale top-level audit-document
  references remain.

## Review Groups

Use these groups for staging, review, and commit hygiene:

1. Spec/product doctrine: `specs/`, `docs/contracts/`, `docs/decisions/`,
   product-mechanics summary docs, and conductor report.
2. Text engine and fixtures: `crates/engine/src/text/`,
   `crates/engine/assets/fonts/`, and `crates/engine/testdata/golden/text/`.
3. GUI visual harness: `crates/gui-*` visual/text files and
   `crates/gui-render/testdata/golden/board/`.
4. Import/CLI/DRC implementation: modified `crates/cli/`, `crates/engine/`,
   `crates/engine-daemon/`, and `crates/test-harness/` files.
5. Parity/gates: `scripts/check_spec_parity.py`,
   `specs/SPEC_PARITY.md`, `specs/spec_parity_manifest.json`, and modified
   drift-gate scripts.
6. Audit evidence: historical audit and research synthesis docs listed above.

## Production-Readiness Gates

Before calling the tree production-clean:

- `git diff --check`
- `cargo check --workspace`
- `cargo test --workspace`
- `scripts/run_drift_gates.sh`
- visual golden tests for the GUI render harness
- targeted CLI/golden tests for text/import/gerber changes

Current cleanup evidence:

- `git diff --check` passed during cleanup.
- `cargo check -p datum-eda-cli --bin datum-eda` passed with no warnings after
  CLI cleanup.
- `cargo check --workspace` passed with no warnings after CLI cleanup.
- `cargo test --workspace` passed after restoring the test-visible manifest
  loader helper.
- `scripts/run_drift_gates.sh` passed after restoring MCP current-method
  compatibility anchors and reducing the GUI protocol inline-test tail to its
  frozen budget.
- GUI renderer visual feature tests passed:
  `visual_manifest`, `visual_diff`, `visual_capture::tests::align_to_preserves_aligned_rows`,
  and `visual_runner`.
- Layer A and Layer B visual checks passed:
  `cargo test -p datum-gui-render --features visual --test visual_goldens -- --ignored --nocapture`
  and
  `cargo test -p datum-gui-app --features visual --test visual_shell -- --ignored --nocapture`.
- Targeted CLI text/import/gerber tests are covered by the passing
  `cargo test --workspace` run; the test inventory includes board text,
  project text, imported query/check goldens, and gerber text/mechanical/
  silkscreen cases.

Remaining production sign-off work is deliberate staging by the review groups
above and final review of tracked versus archived audit evidence.
