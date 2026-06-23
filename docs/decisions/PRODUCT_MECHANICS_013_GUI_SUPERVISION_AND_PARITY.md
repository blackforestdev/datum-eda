# Product Mechanics Decision 013: GUI Supervision And Feature Parity

Status: draft for owner review; ratified-direction correction 2026-06-22. This
record encodes the owner-approved governance rule and corrects a conflation in
the prior sequencing, not the engine-first sequencing itself.
Date: 2026-06-22

Driven by:
- the supervision-gap finding: a large headless push (substrate, CLI, MCP)
  left the GUI visually unchanged, so the human supervisor cannot visually
  audit progress
- `docs/audits/CODE_VS_SPEC_CONFORMANCE_AUDIT.md`
- `PLAN.md` engine-first/GUI-last decision (dated 2026-03-24): "Engine-first,
  GUI-last — Headless is the differentiator; GUI is a consumer"
- `docs/decisions/PRODUCT_MECHANICS_000B_VIEW_COMPOSITION_AND_LIVE_PRODUCTION.md`
- `docs/decisions/PRODUCT_MECHANICS_002_MANUAL_EDITOR_BASELINE.md`
- `docs/decisions/PRODUCT_MECHANICS_012_APPLICATION_QUALITY_BAR.md`

## Decision Scope

Define the role of the GUI during a deliberately headless-first build, and
separate two distinct kinds of GUI/engine parity that were previously
conflated: read-only visual supervision of committed engine state, and
interactive authoring through GUI controls.

This decision governs sequencing and enforcement posture for those two parity
levels. It does not redefine the engine-first/GUI-last sequencing in `PLAN.md`;
it corrects an oversight in how that sequencing was interpreted.

## Problem

A large headless push across substrate, CLI, and MCP advanced engine
capability while the GUI stayed visually unchanged. The human supervisor
therefore cannot visually audit progress, because the only audit surfaces are
text, files, and machine interfaces.

"Engine-first, GUI-last" (`PLAN.md`, 2026-03-24 decision) is deliberate and
correct. It defers the GUI EDITOR until the substrate exists, and that
deferral stands. What it never required was that the GUI REFLECT engine state
for supervision.

Visual-reflection and interactive-editing were treated as one downstream
phase. That conflation is the oversight being corrected. Reflecting committed
state for a supervisor is not the same work as exposing interactive authoring,
and it should not have been deferred alongside it.

## The Two Parity Levels

This decision recognizes two distinct parity levels and treats them
separately.

### 1. Supervision-Reflection Parity (REQUIRED, near-term)

Supervision-reflection parity is the first GUI deliverable. The GUI must be
able to VISUALLY DISPLAY any committed engine or native state a human needs to
audit:
- native authored geometry
- operation and transaction results
- check findings
- manufacturing projections
- agent and session activity

This is read-only visibility — the supervisor's instrument panel. It does NOT
require interactive editing. It is the minimum that makes development
supervisable, and it is required near-term.

### 2. Interactive-Authoring Parity (TARGET, the GUI editor phase)

Interactive-authoring parity is the target for the GUI editor phase. Every
user-facing action has a GUI control emitting typed operations through the
single `commit()` path (Decision 002).

This level is sequenced AFTER the substrate and library foundation. It is the
GUI editor phase, not a near-term obligation.

## Sequencing Commitment

The GUI build-out is an EXPLICIT, NAMED downstream phase in the
post-correction sequence:

> substrate (in flight) -> library -> native authoring + GUI surface

Within that sequence:
- supervision-reflection parity comes early — it is the first GUI deliverable
  and the supervision unlock
- interactive-authoring parity is the GUI editor phase, after substrate and
  library

This is NOT an implied "M8 later". The GUI surface is a named phase in the
sequence above, and supervision-reflection is not held behind it.

## Build-Back Fuse + Parity Gate (Armed-When-GUI-Phase-Begins)

A build-back fuse and parity gate are DEFINED NOW and ARMED WHEN the GUI
editor phase begins. They are dormant until then.

When the interactive GUI editor phase starts, a touch-triggered rule applies:

> Any change that adds or alters a user-facing capability must add or update
> its GUI surface AND a VISUAL GOLDEN in the same change.

This is enforced by a golden-backed `check_gui_parity` wired into
`run_drift_gates`.

The fuse is defined now so it is ready when needed. It is explicitly NOT
enforced during the current substrate-focused (GUI-dormant) phase. Enforcing
it now would fight the deliberate sequencing and saddle work with GUI
obligations that have nothing to build on yet.

## Calibration / Non-Goals

Calibration of the parity rule:
- parity scope is USER-FACING capability only; internal plumbing is excluded
- "build back" must be MEANINGFUL — it renders real state and is proven by a
  visual golden; shallow stubs do not satisfy the rule
- owner-waived deferrals are allowed and tracked
- the retroactive existing-gap debt is a tracked burn-down backlog, NOT an
  immediate hard-fail; a gate that is red-forever forces stubs or stalls work,
  which is itself a failure mode

Enforcement layering. Prose rules alone do not govern an autonomous agent. The
enforcement layers, when armed, are:
- the agent's per-task definition-of-done
- the machine gate (`check_gui_parity` in `run_drift_gates`)

Doctrine is the weakest layer. This document is doctrine; it is necessary but
not sufficient, and it relies on the per-task definition-of-done and the
machine gate to actually govern behavior.

This decision does not require:
- interactive GUI editing during the substrate phase
- arming the parity gate before the GUI editor phase begins
- a red-forever hard-fail gate for the retroactive gap backlog
- GUI surfaces for internal plumbing that is not user-facing

## First Steps

The cheapest supervision unlock is to confirm whether the GUI can render
native or resolved state today:
- if YES, an author-via-CLI -> view-in-GUI audit loop already exists, and the
  first step is to confirm and document that loop as the supervision surface
- if NO, that audit loop is the first supervision-reflection deliverable, and
  building it is the first GUI work

From there:
- supervision-reflection parity is the first GUI deliverable, scoped to
  read-only display of committed native and resolved state and the audit
  surfaces named above
- the build-back fuse and `check_gui_parity` are authored as dormant
  definitions now, ready to arm when the GUI editor phase begins
- the retroactive existing-gap debt is captured as a tracked burn-down
  backlog, not converted into an immediate hard-fail

## Open Owner Questions

1. What is the exact arming trigger for the build-back fuse — the first commit
   of the GUI editor phase, a milestone marker, or an explicit owner switch?
2. Should supervision-reflection parity get its own lighter gate now (for
   example, a goldens-backed check that the supervision surfaces render real
   committed state), or is its own per-task definition-of-done sufficient
   until the GUI editor phase arms the full `check_gui_parity`?
3. Which committed-state surfaces are mandatory for the first
   supervision-reflection deliverable, and which may be owner-waived for a
   later increment?
4. What is the minimum visual-golden coverage that counts as a MEANINGFUL
   build-back, so the gate cannot be satisfied by shallow stubs?
5. How is the retroactive existing-gap backlog tracked and burned down, and
   what review cadence keeps it from silently becoming a hard-fail?
