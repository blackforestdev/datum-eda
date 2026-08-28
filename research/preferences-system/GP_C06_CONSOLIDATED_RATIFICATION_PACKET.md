# GP-C06 Consolidated Global Preferences Ratification Packet

> **Status:** owner-review packet produced by GP-C09; no GP-C06 disposition is
> recorded yet.
>
> **Tracker:** `dat-global-preferences-engine-qcv`.
>
> **Boundary:** ratification of the consolidated specification only. Approval
> authorizes no implementation, dependency, migration execution, provider,
> account/synchronization service, or prototype edit.

## 1. Findings-first adversarial review

### Finding 1 — the architecture is internally coherent

No contradiction survives among the settled identity, scope, management,
precedence, seeding, state, validation, unknown-data, provenance, storage,
interaction, catalog, and units contracts. They form one direction:

1. a subsystem-owned descriptor defines one stable setting identity;
2. eligible typed sources contribute without redefining it;
3. user-released controls and ordinary values resolve in separate stages;
4. Project authority remains a different mutation system;
5. the repository preserves all unlike authority and unknown material without
   flattening it;
6. GUI, CLI, and MCP inspect the same resolver result; and
7. the V1 catalog declares every active descriptor or honestly classifies the
   apparent setting as deferred or not a preference.

Evidence: Q1–Q4 approved contracts
(`GP_C03_TYPED_AUTHORITY_DECISION_PACKET.md:210-274,385-458,500-648`), Q5–Q10
and Q11 approved contracts
(`GP_C03_PROJECT_SEED_AND_CONTEXT_DECISION_PACKET.md:139-201,360-424,540-613,748-839,973-1068,1264-1402,1584-1686,1918-2030`), GP-C04
(`GP_C04_STORAGE_MIGRATION_EXCHANGE_RECOVERY_CONTRACT.md:81-462`), GP-C05
(`GP_C05_INTERACTION_AND_VISUAL_CONTRACT.md:68-252`), and GP-C08
(`specs/GLOBAL_PREFERENCES_V1_DESCRIPTOR_CATALOG.md:1-271`).

### Finding 2 — Project authority is not weakened or duplicated

Preferences may read applicable Project Context, display read-only Project
policy, and copy an explicit immutable seed snapshot at Project genesis. It
cannot directly mutate Project policy, live-follow later profile changes, use a
machine reset/import/restore to change a Project, or create a rival Revision or
documentation lifecycle. The Project mutation authority owns the atomic seed
copy and durable itemized receipt; after commit the Project owns every copied
fact.

Evidence: approved Q2 clauses 5 and 9
(`GP_C03_TYPED_AUTHORITY_DECISION_PACKET.md:401-438`), Q5 clauses 1–8
(`GP_C03_PROJECT_SEED_AND_CONTEXT_DECISION_PACKET.md:139-174`), Q6 Context and
state law (ibid. `:764-817`), GP-C04 repository partitions and Project seam
(`GP_C04_STORAGE_MIGRATION_EXCHANGE_RECOVERY_CONTRACT.md:108-143,362-390`),
and GP-C05 clauses 6, 27, and 32
(`GP_C05_INTERACTION_AND_VISUAL_CONTRACT.md:81-98,151-200`).

### Finding 3 — management is consent-bounded, not enrollment authority

Organization Recommend, Constrain, Pin, and Lock directives are inert until the
user grants a typed, revocable `AuthorityRelease`. Requests above the released
level remain visible and inert; revocation lifts controls and re-resolves
retained user values. Personal accessibility descriptors are the only
organization-control carve-out class. Governed Project law remains independent
of that machine grant.

Evidence: Q3 approved clauses 1–11
(`GP_C03_TYPED_AUTHORITY_DECISION_PACKET.md:516-567`), Q4 staged resolution
(ibid. `:591-648`), Q7 provider-state law
(`GP_C03_PROJECT_SEED_AND_CONTEXT_DECISION_PACKET.md:973-1068`), and the drawn
Organization surface in `preferences-window.html#organization` at `a061fca`.

### Finding 4 — storage failure and exchange cannot manufacture authority

The local repository uses immutable generations, expected-generation writes,
one commit point, current plus two validated predecessors, and exact opaque
unknown preservation. An unreadable store is preserved rather than repaired in
place; defaults may keep the session usable but are never written as if chosen.
Portable import is previewed and per-key, refuses Capability and managed
authority, and applies nothing until confirmation. Migration never substitutes
an unchosen value. Restore is previewed, reversible, and distinct from import.
No store operation touches Project policy or design data, requires a network, or
blocks authoring.

Evidence: GP-C04 §§3–12
(`GP_C04_STORAGE_MIGRATION_EXCHANGE_RECOVERY_CONTRACT.md:81-462`) and Claude
`preference-store-states-study.html` commit `586eb0d`, states at source lines
78–103.

### Finding 5 — interaction consumes engine truth and remains accessible

The ratified surface is refined Option A at Claude commit `a061fca`: two columns,
search pinned over the settings pane, row-complete label/description/control/
provenance, search-first discovery over labels, descriptions, stable keys, and
retired/alternate names, and one resolver-owned explanation beside the retained
list. The full-width search variant is rejected. Keyboard order, reduced motion,
narrow stacking, non-color proof, and announcements are controlled by `e2127fa`.
Starting/changing search closes stale explanation; Escape restores the specified
focus/navigation state. GUI, CLI, and MCP expose one typed semantic answer.

Evidence: GP-C05 clauses 1–39
(`GP_C05_INTERACTION_AND_VISUAL_CONTRACT.md:68-226`), conformance exclusions
(ibid. `:228-252`), `preferences-window.html` commit `a061fca`,
`preferences-accessibility-study.html` commit `e2127fa`, and comparative
`search-placement-study.html` commit `e1dd9c6`.

### Finding 6 — the V1 descriptor boundary is complete but pre-implementation

GP-C08 registers 63 active schema-version-1 descriptors, all using atomic
`Replace` merge in V1. Each row names stable key, owner, type/default,
persistent/resolution scopes, Q3/Q4 eligibility, seed destination where any,
apply behavior, consumers, portability, migration, and implementation
disposition. The same catalog explicitly defers unresolved candidates and
negatively classifies Project/document authority, operation input, Session/
restartable/transient/onboarding/provider/repository state, store operations,
unknown records, authority release, and read-only mirrors.

Evidence: `specs/GLOBAL_PREFERENCES_V1_DESCRIPTOR_CATALOG.md:1-271`, marker
`EVIDENCE:GLOBAL-PREFERENCES-SPEC:GP-C08-V1-DESCRIPTOR-CATALOG`, commit
`96579c5`. The earlier 60+59 intake remains historical clay, not authority
(`GP_DRAFT_SETTINGS_CATALOG_SEED.md:1-174`).

### Finding 7 — the units seam preserves canonical design truth

One engine-owned Units service serves GUI, CLI, and MCP edge adapters. Authored
length stays checked signed `i64` nanometers; system, unit, and precision are
display/parser projections only. Explicit suffix parsing uses checked integer/
rational arithmetic and refuses overflow, ambiguity, wrong quantity, and
non-integral-nanometer results. Bare numbers require explicit resolved field
context. Existing `_nm` schemas remain compatible until a governed adapter
migration is authorized.

Evidence: `GP_SHARED_UNITS_ENGINE_REQUIREMENT.md:7-82`, marker
`OWNER-REQUIREMENT:GLOBAL-PREFERENCES:SHARED-UNITS-ENGINE`; catalog unit
descriptors `GLOBAL_PREFERENCES_V1_DESCRIPTOR_CATALOG.md:91-104`; canonical IR
evidence cited by the units requirement at `docs/CANONICAL_IR.md:59-77` and
`crates/engine/src/ir/geometry.rs:3-15`.

### Finding 8 — one visual inconsistency remains bounded and non-authoritative

The Viewport airwire-culling row's prose says factory **Off**, while its current
control visually displays **On**. GP-C08 deliberately records the descriptor
default as `false` and requires a bounded Claude reconciliation before
implementation. This mismatch does not authorize Codex to edit the prototype,
does not alter any other descriptor, and is excluded from approval as visual
proof of the current selected value. The row's written default and catalog law
remain the proposed schema fact for owner ratification.

Evidence: `preferences-window.html:166` at `a061fca` and
`GLOBAL_PREFERENCES_V1_DESCRIPTOR_CATALOG.md:114,123-127,264-267`.

### Finding 9 — external evidence is support, not imported authority

GP-C02 uses primary/authoritative XDG, Apple, Microsoft, GNOME, Android,
Kubernetes, Qt, SQLite, Protocol Buffers, IETF, W3C/WAI, NIST, and regulated
configuration sources to derive typed requirements for storage separation,
atomicity, expected generation, unknown preservation, conflicts, accessibility,
and audit. GP-C02B uses official SOLIDWORKS/PDM, Altium, Revit, and KiCad sources
as domain-peer evidence for selective management and portability. Peer behavior
is not Datum doctrine, and citations adopt no library, protocol, provider, or
dependency.

Evidence: `GP_C02_EXTERNAL_AND_STANDARDS_RESEARCH.md:67-105,111-403,408-438`;
`GP_C02B_DOMAIN_PEER_PREFERENCES_RESEARCH.md:64-170,265-430`; GP-C04's explicit
source-strength limit
(`GP_C04_STORAGE_MIGRATION_EXCHANGE_RECOVERY_CONTRACT.md:69-79`).

### Finding 10 — mechanism and dependency authority remain deliberately open

The specification does not select a GUI toolkit, cryptographic package format,
credential facility, organization transport, identity/account provider,
synchronization service, network protocol, or third-party persistence library.
GP-C04 specifies canonical local semantics but adopts no cited implementation.
Product Mechanics 029 still requires a separate numbered owner decision before
any new third-party code dependency is added, fetched, vendored, or linked.

Evidence: Q1/Q2 non-decisions
(`GP_C03_TYPED_AUTHORITY_DECISION_PACKET.md:244-250,433-438`), GP-C04 boundary
and synchronization exclusions
(`GP_C04_STORAGE_MIGRATION_EXCHANGE_RECOVERY_CONTRACT.md:12-22,331-360`),
GP-C05 boundary (`GP_C05_INTERACTION_AND_VISUAL_CONTRACT.md:12-25`), and
`PRODUCT_MECHANICS_029_DEPENDENCY_AUTHORITY.md:20-72`.

## 2. Complete GP-C03 disposition ledger

Canonical integer questions Q1–Q11 are eleven dispositions. Q5A is the sole
additional recorded sub-letter exception, so this ledger contains twelve rows.
No question is reopened by consolidation.

| Boundary | Owner disposition and resulting law | Controlling evidence | Visual authority |
|---|---|---|---|
| Q1 | Q1-A: one stable nonlocalized key and one subsystem-owned descriptor; providers contribute but cannot redefine it | `GP_C03_TYPED_AUTHORITY_DECISION_PACKET.md:210-274` | identity vocabulary later rendered in `preferences-window.html` `a061fca` |
| Q2 | Q2-A: typed source families/classes; Project mutation, restartable/transient state, and operation input stay outside writable Preferences | ibid. `:385-458` | `preferences-ux-study.html#px-v4` archival structure |
| Q3 | revised AuthorityRelease model: Recommend/Constrain/Pin/Lock only within a user-held five-level revocable release; accessibility carve-out | ibid. `:500-567` | `preferences-ux-study.html#px-v5` commit `06f5931`; real `#organization` at `a061fca` |
| Q4 | Q4-A/PX-V6: eligibility, then controls, then ordinary values; User outranks Recommendation/Installation/default; no arrival-order conflict resolution | ibid. `:591-648` | `preferences-ux-study.html#px-v6` commit `06f5931`; Q9 pane at `70bf2a2` |
| Q5 | Q5-A: one immutable selected seed snapshot, atomic Project-authority copy, durable itemized receipt, no live following | `GP_C03_PROJECT_SEED_AND_CONTEXT_DECISION_PACKET.md:139-201` | real `#files-projects`, `#revision`, `#publish-space`, `#units`; archival PX-V7 |
| Q5A | Q5A-B+C: guided-in-place is sole setup surface; zero required choices; replayable machine-local state; optional assistant proposals apply only after acceptance | ibid. `:360-424` | `guided-setup-study.html` commits `b5cac07` and `b8f5a7a`; real `#files-projects` |
| Q6 | Q6-A: distinct Session, Context, operation input, restartable state, and transient state with bounded lifetimes and accessibility | ibid. `:748-839` | real `#workspace/#rules-checks/#output/#files-projects`; archival PX-V8 |
| Q7 | Q7-A: descriptor validation, correctable refusal, protected-source exclusion, explicit stale/unavailable/expired/revoked provider states, no unrelated authoring block | ibid. `:973-1068` | real `#terminal/#agents/#organization`; archival PX-V9 |
| Q8 | Q8-A/P2: byte-faithful inactive unknowns, explicit one-live-identity alias migration, non-deletion, export-first removal, dedicated Manage preferences home | ibid. `:1264-1402` | `preferences-window.html` `54f2aba`; placement study `cd12baa`; adopted real P2 at `a061fca` |
| Q9 | Q9-A: one read-only resolver query returns effective value, all contributions/dispositions, Q4 reason, descriptor/Context facts, actions, and explicit redaction identically to GUI/CLI/MCP | ibid. `:1584-1686` | real explanation commit `70bf2a2`; archival PX-V11 |
| Q10 | Q10-A/S2/R1: Revision visibility is Presentation only; ordinary onboarding is machine-local; sufficiently released teaching policy may require dismissible non-gating replay once per Project | ibid. `:1918-2030` | `revision-carryforward-study.html` `7089ee8`; real `#revision`; unchanged Release arm bar |
| Q11 | Q11-A (historical Q5B-A): one local non-tabbed Start page with canonical actions, engine-truth Recents, read-only seed rail, and no news/marketing/alerts, telemetry, or startup network load | ibid. `:540-613` | `start-page-study.html` `15328f4`; real Startup row at `#files-projects` |

## 3. Consolidated normative contract

Approval of GP-C06 ratifies the following bounded clauses as one specification:

1. The approved GP-C03 Q1–Q11 dispositions plus Q5A remain controlling exactly
   as cited in §2; consolidation neither rewrites nor broadens them.
2. GP-C04 is the controlling persistence, migration, recovery, exchange,
   synchronization-conflict, audit, and Project-policy-seam contract. Its
   rendered clauses at `586eb0d` are part of the ratified visible behavior.
3. GP-C05 refined Option A is the controlling interaction/visual contract at
   `a061fca`, with accessibility/context states controlled by `e2127fa`, store
   states by `586eb0d`, and full-width search placement rejected.
4. GP-C08 is the complete initial V1 descriptor catalog: 63 active descriptors,
   all other candidates explicitly deferred or negatively classified. A later
   catalog amendment requires governed evidence and cannot occur as
   implementation-time improvisation.
5. The shared Units service requirement is controlling: one exact engine-owned
   conversion/parser/formatter seam, signed `i64` nanometer authored truth, typed
   per-quantity display projection, checked parsing, and GUI/CLI/MCP parity.
6. Descriptor defaults are schema facts, not stored user choices. Reading,
   setup, recovery, import, migration, or first run cannot manufacture an
   explicit selection.
7. Existing Projects own their policy and design data. Preferences may inspect
   them read-only or seed a new Project once through its mutation authority and
   receipt; no machine operation may live-update, reset, restore, migrate, or
   synchronize an existing Project.
8. Product Mechanics 034 remains the sole Product Revision Engine authority.
   Preferences may seed revision policy at genesis and control presentation/
   guidance only; it cannot mint revisions, mutate journals/records, weaken
   Release gates, or create a competing lifecycle.
9. Product Mechanics 035 remains the documentation-system authority for the
   Project's `AdoptedDraftingStandard`. Preferences may provide a receipted seed
   and read-only doorway only; it never owns or live-updates that object.
10. Product Mechanics 036 remains the schematic drawing-theme authority. The
    theme is one persisted machine Presentation selection over complete
    governed Dark/Light systems, remains outside Project/drafting-standard law,
    and cannot alter Publish/print output or imply a Light board theme.
11. Every visible Preferences behavior remains non-blocking to Design authoring;
    invalid input, unavailable providers, corrupt stores, setup, onboarding,
    import conflicts, and inspection cannot become an unrelated authoring gate.
12. Search, explanation, validation, refusal, management, storage states, setup,
    Start page, and Revision guidance retain the exact accessibility law in
    GP-C05 and their controlling Claude renders. Material visual change remains
    render-first and Claude-owned.
13. The airwire-culling selected-control mismatch in Finding 8 remains a bounded
    visual reconciliation debt. Approval ratifies factory `false` from the row's
    written law and catalog, not the prototype's contradictory displayed `On`.
14. Approval authorizes **no implementation** and does not place or execute any
    implementation slice. Only GP-C07 may later propose bounded Frontier work,
    and execution still requires explicit owner authorization.
15. Approval authorizes **no dependency** or licensing exception. Product
    Mechanics 029 remains controlling for every new third-party dependency.
16. Approval selects no provider, account/identity system, network transport,
    synchronization service, credential facility, GUI toolkit, cryptographic
    package, or implementation ABI.
17. Approval does not weaken, amend, supersede, or reopen Product Mechanics 034,
    035, or 036; where a Preferences projection touches their subject, those
    doctrine boundaries prevail.

## 4. Product Mechanics preservation matrix

| Doctrine | Authority preserved | Preferences may do | Preferences may not do | Evidence |
|---|---|---|---|---|
| PM-034 | configuration items, Changes, baselines, engineering revisions, Release, document issues, standing, reproducibility, audit | seed explicit new-Project revision policy; show/hide projections; provide non-gating guidance; query resolved policy | mint identity, alter journal/records, bypass gates, turn visibility/onboarding into Revision authority | `PRODUCT_MECHANICS_034_PRODUCT_REVISION_ENGINE.md:13-97`; Q5/Q10; GP-C04 §10 |
| PM-035 | Project documentation authority owns `AdoptedDraftingStandard`; standards registry owns cited basis | copy one receipted seed; show read-only doorway/context | live-follow, reset/import/restore Project standard, redefine revision/status law | `PRODUCT_MECHANICS_035_ADOPTED_DRAFTING_STANDARD_AUTHORITY.md:13-58`; Q5; catalog publish seeds |
| PM-036 | schematic theme is whole-system machine Presentation state; Dark default; Light warm paper; print independent | persist/reset/query theme and provenance | free-edit palette, alter board theme, Publish/print, Project policy, drafting standard | `PRODUCT_MECHANICS_036_SCHEMATIC_DRAWING_THEMES.md:13-70`; catalog `datum.schematic.theme` |

## 5. Review limits and successor boundary

This packet is complete for specification ratification but intentionally not an
implementation plan. GP-C07 remains pending and may only place bounded future
work after owner approval. It must retain separate authorization for execution,
the PM-029 dependency gate, Claude ownership of visual truth, conformance proof
for every contract above, and the ordinary Frontier selector/claim discipline.

No completion evidence is recorded for GP-C06 by preparing this packet. Only an
explicit owner response below may disposition the boundary.

## 6. Owner response

<!-- OWNER:GLOBAL-PREFERENCES-SPEC:GP-C06:GP-C06-CONSOLIDATED-PACKET -->

Approve only if this packet faithfully consolidates the cited settled authority,
states the unresolved limits honestly, and preserves every explicit exclusion.

Reply exactly:

```text
GP-C06-RATIFICATION: approve
```

or:

```text
GP-C06-RATIFICATION: revise — <specific contradiction, omission, or boundary correction>
```

<!-- EVIDENCE:GLOBAL-PREFERENCES-SPEC:GP-C09-CONSOLIDATED-RATIFICATION-PACKET -->
