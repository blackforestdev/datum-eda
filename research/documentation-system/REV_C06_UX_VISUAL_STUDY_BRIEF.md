# REV-C06 Product Revision UX and Visual Study Brief

> **Status:** REV-C06 planning authority and owner-disposition ledger for a
> Claude-owned HTML study; not a ratified mechanism or implementation
> authorization.
> Claude owns edits under `docs/gui/prototypes/*.html`. Codex owns this brief,
> the authority audit, traceability registration, and later owner packets.

## 1. Purpose

The Product Revision Engine is broad, but its ordinary experience must be
quiet. Datum should protect engineering integrity without forcing a small-team
designer to operate an enterprise PLM dashboard. Regulated profiles must expose
the exact additional review, approval, effectivity, evidence, and audit facts
they require without creating a second engine.

The study must answer how users:

- understand working, baseline, successor-work, and issued context;
- enter and navigate revision/change work from the existing Datum shell;
- assess dependency impact and library uptake;
- compare baselines and account for controlled differences;
- regenerate only affected evidence;
- prepare, review, approve, and atomically issue a Release;
- inspect byte-reproducible evidence and failures; and
- do all of this by pointer, keyboard, and assistive technology.

It must render the decisions instead of reducing them to prose. It must also
show the normal case, not only warning dashboards.

## 2. Authority the study consumes

### 2.1 Owner-approved facts—do not reopen

1. Datum has one Product Revision Engine with profile-scaled ceremony.
2. Technical transaction history and human EngineeringRevision identity are
   separate clocks.
3. Sequential Alphanumeric (Legacy) is the factory profile; ISO 19650-oriented
   behavior is selectable and keeps REVISION and STATUS separate.
4. First divergence from a released baseline quietly creates identity-free
   successor work; no edit/save/timer automatically allocates revision identity.
5. V2-P is the approved provisional reservation treatment once explicit
   preparation begins.
6. EngineeringRevision belongs only to a revision-bearing CI. A baseline owns
   composition; a Release may atomically issue several affected CIs while
   reusing unchanged members' existing revisions.
7. PublishSet remains editable composition. ControlledDocument and immutable
   DocumentIssue carry controlled publication authority.
8. Typed standing facts use V10-C; issued records never mutate.
9. Complete local/offline authority is mandatory. Git is an optional adapter
   and never issues, approves, or releases.
10. Changed, affected, stale, orphaned, standing, and non-reproducible are
    separate facts.
11. Library bindings remain pinned until explicit uptake.
12. Release reproducibility means byte-identical specified outputs from exact
    captured inputs; equivalent-looking output is not sufficient.

### 2.2 Required source ranges

Claude must read these ranges before drawing:

| Authority | Required ranges | What it controls |
|---|---|---|
| `PRODUCT_REVISION_ENGINE_AUTHORITY_MODEL.md` | 12-76, 144-190, 224-387, 428-490, 497-577, 580-667, 729-1036 | Quiet/default doctrine, identities, candidates/releases, lifecycle, roles, operations, refusals, queries, and approved Q1-Q6 dispositions. |
| `REV_C04_OFFLINE_GIT_INTEGRATION_CONTRACT.md` | 1-55, 284-420, 469-566 | Offline authority, adapter-state presentation, external-change quarantine, and visual handoff constraints. |
| `REV_C05_IMPACT_STALENESS_REPRODUCIBILITY_CONTRACT.md` | 1-68, 111-535 | Exact vocabulary, impact, freshness, uptake, comparison, regeneration, reproduction, findings, invariants, and visual handoff. |
| `PRODUCT_REVISION_ENGINE_STANDARDS_MATRIX.md` | 89-131 | Profile-dependent change, impact, status, audit, document, approval, retention, and package requirements. |

Line ranges are tied to the files as committed at the REV-C06 handoff. If a
source moves, record the new exact range; do not silently work from memory.

## 3. Existing visual authority

The new study extends these sources; it does not redraw Datum from scratch:

| Existing source | Required treatment |
|---|---|
| `revision-identity-study.html` V1-V7 | Preserve approved scheme selection, V2-P, title-block projections, per-CI namespaces, baseline composition, ISO two-axis treatment, and prototype/production behavior. |
| `revision-authority-study.html` V8-V10 | Preserve per-CI authority, V9-A ControlledDocument chain, and V10-C typed standing presentation. |
| `publish-lifecycle-study.html#frame-g` and `#frame-g4` | Preserve G1 quiet working, G2 immutable baseline view, G3 successor-impact doorway, and G4 `ConfigurationRef`/Inspector seam. Correct only language that REV-C05 has made more precise. |
| `workspace-panes.html` | Preserve recursive tiling, focused-pane ownership, Project Navigator, Inspector binding, local open-beside behavior, and one cohesive native window. |
| `command-feedback-study.html` and Decision 033 | Console remains output-only, transient action feedback. It is not a revision dashboard or durable findings surface. |
| `DATUM_GUI_DESIGN_SPEC.md` | Preserve shell structure, pane focus, token discipline, status routing, and accessibility behavior. |

Required terminology correction: a source difference does not automatically
mean `Affected`, and `Affected` does not automatically mean `Stale`. G3 may say
that successor work requires impact analysis; the result must be the engine's
typed evaluation.

## 4. Interaction doctrine

1. **Quiet by default.** Ordinary valid working state uses restrained status and
   Inspector detail. No permanent red/amber dashboard occupies Design space.
2. **Local doorway, proper workspace.** A selected CI, document, evidence item,
   or finding exposes contextual actions locally. Complex review opens a typed
   adjacent pane through Datum's existing open-beside behavior.
3. **No surprise focus.** Background evaluation and status updates do not steal
   focus. An explicitly opened review pane receives predictable focus.
4. **Evidence before ceremony.** The UI explains what changed, why something is
   affected, and what proof is missing before presenting approval/release acts.
5. **One engine, scaled disclosure.** Lightweight and regulated profiles reveal
   different required fields/stages over the same typed records.
6. **Refusal is actionable.** Every blocked act names the subject, expected and
   actual facts, governing requirement, and next valid action.
7. **Issued truth is visually immutable.** Baseline/Release views never resemble
   editable Design or Publish state.
8. **No false roll-up.** Product/baseline labels never imply that independently
   revised child CIs share a revision.
9. **No Git-shaped UX.** Git receipts may be inspected as adapter evidence, but
   branches, commits, and tags never headline the release workflow.
10. **No rival histories.** Durable change/release facts come from revision
    authority; the Console may narrate an operation but does not store it.

## 5. Required study frames

Claude may split the study into cross-linked files before the 700-line ceiling.
Every frame requires an anchor, a requirement chip, and a coverage row.

### UX-V1 — Shell placement and entry points

Render the complete Datum shell with:

- Project Navigator revision-bearing CIs, Changes, Baselines, Releases,
  Controlled Documents, and evidence as restrained typed groups;
- selected-object Inspector facts and local actions;
- right-click entry from a selected Design/Publish subject or finding;
- double-click/Enter opening the selected authority beside the focused pane;
- revision/change panes participating in recursive tiling and full-pane focus;
- no detached native window and no replacement shell.

Render at least two credible doorway candidates for owner question Q1. Both must
begin locally and end in a proper adjacent pane; candidates may differ in
whether the Navigator has permanent revision groups or reveals them only after
the first controlled record exists.

### UX-V2 — Quiet successor work

Render these successive states at real shell/title-block size:

1. released baseline view—immutable and clearly identified;
2. first committed divergence—quiet `Successor work — revision identity not
   yet reserved` in Inspector/status, with no modal interruption;
3. multiple accumulated transactions under one Draft EngineeringChange;
4. contextual `Prepare revision…` entering the approved V2-P reservation;
5. a regulated profile refusing mutation until a Change is authorized.

Show what is announced non-visually and what remains silent. Do not put the
Draft Change lifecycle into the Datum Console.

### UX-V3 — Change record and lifecycle

Render an adjacent `Change` pane for the same EngineeringChange under:

- lightweight profile: summary, scope, changed subjects, impact status,
  verification, and release connection with minimal ceremony;
- regulated profile: lifecycle stage, roles, decisions, rationale,
  effectivity/applicability, departures, approvals, evidence, and audit links.

The pane must show Draft -> ImpactReview -> Authorized -> Implementing ->
Verification -> Closed plus Rejected/Deferred/Cancelled without turning the
states into freely editable strings. Authorization, verification, closure, and
release remain visibly distinct.

### UX-V4 — Impact analysis

Render a navigable impact pane with one concrete example spanning:

```text
library footprint change
  -> placed package binding
  -> board
  -> assembly drawing viewport
  -> manufacturing output job
  -> Gerber/drill/assembly evidence
```

The view must distinguish:

- `Changed` source observations;
- `Affected` consumers with witness paths;
- `Unaffected` consumers with a proof/reason;
- `Unknown` because an edge/evaluator is incomplete;
- required review/actions; and
- graph scope/completeness.

Render a summary-first view and an explain-path detail. A missing path must not
appear green or complete. Show filtering for affected, unaffected, unknown, and
required action without relying on color alone.

### UX-V5 — Library uptake and derived evidence

Render:

- a selected pinned library binding with a newer revision available;
- preview of geometry/pin-pad/unit/lifecycle/provenance differences;
- Compatible, Requires Remap, Breaking, and Unknown examples;
- explicit `Adopt revision…` under a Change;
- no silent rebind;
- Current, Stale, Orphaned, and Unknown evidence with exact differing or
  unresolved inputs; and
- zone fill as the familiar bounded precedent, plus a manufacturing artifact
  whose generator/policy—not board geometry—made it stale.

### UX-V6 — Baseline comparison and change accounting

Render two baselines aligned by stable CI/object identity. Show Added, Removed,
Modified, Retargeted, Unchanged, and a rename that does not become false
delete/add. Each controlled difference resolves to its governing Change,
departure, or administrative disposition.

Include one `Unaccounted controlled difference` blocker with its route to
resolution. Do not imply that comparison allocates a revision.

### UX-V7 — Regeneration plan

Render a topologically ordered plan that separates:

- evidence proven current and reused;
- stale/affected/missing evidence to regenerate;
- prerequisites and reason paths;
- running/progress state;
- a failed/missing producer; and
- newly created successor evidence linked to old immutable evidence.

No `Refresh in place` action may appear. Regenerating a candidate and
reproducing a historical Release must use visibly different verbs and context.

### UX-V8 — Release preparation and readiness

Render a ReleaseCandidate covering at least two independently revised CIs and
one unchanged reused CI. It must expose:

- exact proposed baseline members;
- proposed revision allocations/reservations by CI namespace;
- governing Changes and departures;
- effectivity;
- required evidence and readiness findings;
- proposed DocumentIssues/packages;
- applicable policy/profile; and
- attestations bound to the current candidate digest.

Render candidate states Preparing, Ready for Review, In Approval, Ready to
Release, Stale, Rejected, and Cancelled. A covered digest change must visibly
invalidate only the affected readiness/attestation facts and return the
candidate to preparation.

Render at least two credible workflow-composition candidates for Q2: one
cohesive release pane with progressively disclosed sections, and one staged
review flow over the same candidate. Neither may hide blockers or imply that a
wizard page is authority.

### UX-V9 — Approval and atomic release

Render:

- lightweight self-authorized release where the active profile permits it;
- regulated ordered/quorum/independence role requirements;
- approval target digest and stale-approval invalidation;
- final release-authority confirmation that summarizes irreversible results;
- least-destructive initial focus and an explicit cancel path;
- one atomic result creating the baseline, affected EngineeringRevisions,
  DocumentIssues/packages, and Release; and
- unchanged CI revisions reused without reminting.

The confirmation may be modal only for the final deliberate issuance boundary.
Background findings, progress, and ordinary lifecycle changes must not be modal.

### UX-V10 — Reproduction and audit evidence

Render a release-evidence pane showing:

- exact baseline/configuration;
- digest-qualified source subjects;
- producer build identity;
- invocation/settings;
- captured environment;
- specified outputs with size/digest;
- independent reproduction attempt; and
- `Byte identical`, `Output mismatch`, `Unavailable`, and `Execution failed`.

Render `Canonical equivalent` only as a non-satisfying diagnostic. Show that a
later failed reproduction does not alter the Release, issued bytes, approval,
or standing. Authenticity/signature verification must be a separate fact.

### UX-V11 — Adapter and offline posture

Render local release with no Git configured, then the same Release with optional
Git mapping/mirror evidence. Show pending/failed adapter mirroring as isolated
adapter status unless policy blocks distribution. Show quarantined external
change entering semantic review. Never use a Git tag/commit as the Release ID.

### UX-V12 — Responsive and accessibility matrix

For V1-V11 render representative:

- wide, tiled, narrow, maximized-pane, and full-screen-stage behavior;
- keyboard order, local context-menu route, tree navigation, expand/collapse,
  selection, and return-focus behavior;
- glyph + word + exact identity for state/severity;
- high-contrast and color-independent differentiation;
- zoom/text-scale and truncation with full Inspector/detail recovery;
- non-focus-stealing status announcements;
- assertive announcements only for truly time-sensitive blocking danger;
- long-running impact/regeneration/reproduction progress without chatty
  per-item announcements; and
- focus restoration after cancellation/completion.

## 6. Surface-routing law

| Fact/event | Primary visual owner | Console | Accessibility behavior |
|---|---|---|---|
| Quiet working/successor context | Pane status + Inspector | Optional one-time operation echo only | Polite concise status on transition, not every edit. |
| Durable impact/freshness finding | Impact/evidence pane + restrained Inspector/status count | Never durable owner | Announce new summary/count without moving focus; detail is navigable. |
| Release preparation/readiness | Release pane | May echo explicit verb outcome | Pane exposes semantic structure and updated readiness. |
| Long impact/regeneration/reproduction progress | Owning pane/status progress | No per-item log spam | Bounded meaningful progress announcements. |
| Typed refusal | Owning pane inline + Inspector/actionable detail | Brief refusal narration allowed | Alert only when immediate action failed; include remediation context. |
| Final atomic Release | Deliberate confirmation then immutable result pane | One success/refusal echo | Focus enters confirmation and returns logically; result is announced. |
| Audit/history | Revision authority pane | Never | Fully navigable structured records. |
| Git adapter lifecycle | Adapter evidence/detail | Never terminal/Console lifecycle spam | Polite status only when user-initiated or blocking. |

This preserves Decision 033: the Console explains verbs; it does not become a
second revision, findings, progress, terminal, or audit surface.

## 7. Accessibility evidence basis

The native GUI uses AT-SPI rather than HTML ARIA, but the behavioral obligations
remain useful:

- [WCAG 2.2 status-message guidance](https://www.w3.org/WAI/WCAG22/Understanding/status-messages.html)
  requires important content changes to be programmatically exposed without
  taking focus and warns against unnecessary chatty interruption.
- [WAI-ARIA modal-dialog pattern](https://www.w3.org/WAI/ARIA/apg/patterns/dialog-modal/)
  requires contained, predictable focus, Escape/cancel behavior, logical focus
  restoration, and recommends the least destructive initial focus for hard-to-
  reverse actions.
- [WAI-ARIA tree-view pattern](https://www.w3.org/WAI/ARIA/apg/patterns/treeview/)
  provides the behavioral reference for Navigator expansion, selection, and
  keyboard movement.
- [WCAG 2.2 focus-order guidance](https://www.w3.org/WAI/WCAG22/Understanding/focus-order.html)
  requires an operable sequence that preserves meaning and operation.

These are accessibility behavior references. The HTML prototype itself is not
proof that the future native implementation conforms.

## 8. Owner questions the study must make answerable

Do not ask these in prose before their candidates exist. After Codex audits the
study, present **one question per owner turn**, with exact authority and
prototype file/line ranges, plain-language consequences, recommendation, and
exact response syntax.

### REV-C06-Q1 — Navigator/doorway disclosure

Choose how revision authority becomes discoverable in the existing Project
Navigator while preserving local right-click entry and adjacent-pane opening.

**Owner disposition — approved 2026-08-25.** The reviewed authority is the
interaction doctrine in section 4, UX-V1 in section 5, and the rendered
`Q1-A-amended` candidate in
`docs/gui/prototypes/revision-ux-shell-study.html`, commit `02ee14a`.

Datum adopts `Q1-A-amended`. The Project Navigator contains a permanent
`Revision` group whose `Changes`, `Baselines`, `Releases`, `Controlled
Documents`, and `Evidence` groups are visible and expanded by default from
Project creation. Each empty group renders an informative row naming what
creates its first record. Datum does not collapse these groups by default and
does not wait for a first controlled record to reveal revision authority.

Future hiding or collapsing may exist only as a presentation preference under
the separately tracked Global Preferences specification
(`dat-global-preferences-engine-qcv`). Such a preference may change projection,
never engine authority or record existence. The existing local right-click
doorway, double-click/Enter open-beside behavior, recursive tiling, and
single-native-window law remain unchanged. The controlling rationale is
structural honesty: the Navigator exposes what the engine governs before first
use instead of creating hidden-authority debt for an unaware designer.

<!-- EVIDENCE:PRODUCT-REVISION-SPEC:REV-C06-Q1-APPROVED -->

### REV-C06-Q2 — Release workflow composition

Choose progressive disclosure in one Release pane or a staged review flow over
the same authoritative ReleaseCandidate.

**Owner disposition — approved 2026-08-25.** The reviewed authority is UX-V8
in section 5, the `ReleaseCandidate` structure and lifecycle in
`PRODUCT_REVISION_ENGINE_AUTHORITY_MODEL.md`, and the two-state
`Q2-A-amended` proof in
`docs/gui/prototypes/revision-ux-release-study.html`, commit `021798a5`.

Datum adopts `Q2-A-amended`: one cohesive Release pane projects the one
authoritative `ReleaseCandidate`. Its nine required semantic section headings,
typed verdict/status, and count remain visible in expanded and user-collapsed
states. A user may collapse only clean detail rows; the default state is fully
expanded. Blockers, `Unknown`, stale facts, invalidated attestations, missing
requirements, and unresolved departures refuse collapse and remain reachable
through the pinned blocker rail. Datum does not impose a staged wizard or treat
review screens as authority. Ordered review remains profile policy over the
same record.

<!-- EVIDENCE:PRODUCT-REVISION-SPEC:REV-C06-Q2-APPROVED -->

### REV-C06-Q3 — Impact-analysis information shape

Choose the primary summary/path relationship and how `Unknown` remains visible
without overwhelming the normal case.

**Owner disposition — approved 2026-08-25.** The reviewed authority is UX-V4
in section 5, the durable impact-result and witness requirements in
`REV_C05_IMPACT_STALENESS_REPRODUCIBILITY_CONTRACT.md`, and the scale,
open-beside, and zero-unknown `Q3-A-amended` proof in
`docs/gui/prototypes/revision-ux-impact-study.html`, commit `118113b`.

Datum adopts `Q3-A-amended`, not original Q3-A. The impact summary is the
primary view. Selecting a summary row opens the canonical Q3-B witness tree
beside the summary; it never replaces or hides the summary. The tree remains
the authoritative drill representation rather than becoming discarded
comparative UI. `ImpactUnknown` and graph-scope rows are permanent even when
they report zero and complete. A clean result is an explicit evaluated claim,
never inferred from an absent row.

<!-- EVIDENCE:PRODUCT-REVISION-SPEC:REV-C06-Q3-APPROVED -->

### REV-C06-Q4 — Lightweight versus regulated disclosure

Approve whether the same visual hierarchy scales cleanly from a one-person
Legacy-profile release to role/quorum/effectivity/audit-heavy profiles.

**Owner disposition — approved 2026-08-25.** The reviewed authority is UX-V3
in section 5, the scalable `EngineeringChange`, lifecycle, and capability model
in `PRODUCT_REVISION_ENGINE_AUTHORITY_MODEL.md`, and the complete paired
`Q4-revised` panes in
`docs/gui/prototypes/revision-ux-shell-study.html`, commit `bbdfa5b`.

Datum adopts `Q4-revised`. Lightweight and regulated profiles project one
stable `EngineeringChange` hierarchy with every section heading visible in the
same order. Lightweight policy labels non-required facts as optional or
self-authorized and supplies informative empty states; regulated policy marks
applicable facts required, exposes unsatisfied blockers, and may require scoped
role separation, order, quorum, or independence. Profiles change obligation and
detail, never record identity, field meaning, position, or discoverability.

<!-- EVIDENCE:PRODUCT-REVISION-SPEC:REV-C06-Q4-APPROVED -->

### REV-C06-Q5 — Final issuance boundary

Choose the exact final confirmation and immutable success treatment for the
atomic, hard-to-reverse Release operation.

**Owner disposition — approved 2026-08-25.** The reviewed authority is UX-V9
in section 5, the `ReleaseCandidate` / `Release` boundary, lifecycle, release
capability, typed operation, and invariants in
`PRODUCT_REVISION_ENGINE_AUTHORITY_MODEL.md`, and the two-state
`Q5-B-amended` proof in
`docs/gui/prototypes/revision-ux-release-study.html`, commit `323e2fb`.

Datum adopts `Q5-B-amended`, explicitly not original Q5-B. Final issuance stays
inside the already-approved cohesive Release pane and uses a pinned two-step
commit bar. The first press arms but does not issue; it exposes the exact typed
create, issue, and reuse consequences. Only the second deliberate confirmation
invokes the atomic `ReleaseConfiguration` operation. The bar is unavailable
until readiness is clean, `Esc`, explicit Disarm, or any interaction outside the
bar disarms it, and the underlying pane remains visible, scrollable, and
inspectable while armed. No modal or competing release authority is introduced.
Successful issuance creates the separate immutable `Release` and issued records;
it never mutates the candidate into release authority.

<!-- EVIDENCE:PRODUCT-REVISION-SPEC:REV-C06-Q5-APPROVED -->

### REV-C06-Q6 — Reproduction/evidence depth

Choose what remains visible in the release summary versus Inspector/detail,
while keeping byte identity and missing evidence unmistakable.

If the study reveals a real architecture contradiction, stop and return it to
Codex. Do not repair authority through an attractive visual.

## 9. Required Claude return

Return:

1. prototype path(s) and commit;
2. screenshot dimensions used for inspection;
3. UX-V1..V12 coverage map with anchors;
4. Q1..Q6 candidate-to-frame map;
5. explicit list of preserved owner dispositions;
6. any contradiction or missing authority discovered; and
7. source-health and link-check results.

The study must say `CANDIDATE — NOT APPROVED` wherever it introduces a choice.
No default may be inferred from visual polish, ordering, or a recommendation.
