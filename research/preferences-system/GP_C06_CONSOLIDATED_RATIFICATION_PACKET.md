# GP-C06 Consolidated Global Preferences Ratification Packet

> **Status:** GP-C10 corrected owner-review packet; Claude reconciliation commits
> `3edd932`, `ed3df1b`, and `4928f26` are reviewed and the GP-C06 boundary is
> ready to reopen after the combined correction commit is recorded.
>
> **Mechanism record:** pending Product Mechanics 037 in
> `docs/decisions/PRODUCT_MECHANICS_037_GLOBAL_PREFERENCES_ENGINE.md`.
>
> **Boundary:** specification ratification only. Approval authorizes no
> implementation, dependency, migration execution, provider, account or
> synchronization service, Frontier execution, or prototype edit.

## 1. Findings-first adversarial review

### Finding 1 — the settled authority is coherent

Q1–Q11 plus the recorded Q5A exception define one stable subsystem-owned
descriptor identity, typed source eligibility, user-released management,
staged resolution, copy-once Project seeding, distinct Session/Context/state,
typed refusal, unknown preservation, one provenance query, Revision carry-
forwards, guided setup, and the Start page. There is no universal bypass.
Absence never masquerades as a contribution. Project mutation and operation
input remain outside the writable preference lattice.

Evidence: Q1–Q4 (`GP_C03_TYPED_AUTHORITY_DECISION_PACKET.md:210-274,385-458,500-648`);
Q5 (`GP_C03_PROJECT_SEED_AND_CONTEXT_DECISION_PACKET.md:145-169,195-200`);
Q5A (ibid. `:347-424`); Q11 (ibid. `:540-613`); Q6–Q10 (ibid.
`:755-839,973-1068,1264-1402,1584-1686,1918-2030`).

### Finding 2 — Project seeding has one registered class

`ProjectPolicySeed` is a registered descriptor class. All fifteen active
seed-bearing catalog rows use it; the sixteenth seed-bearing row, a future
`AdoptedDraftingStandard` seed, is also classified `ProjectPolicySeed` but
deferred because its schema is unspecified. Presentation, Capability,
WorkflowDefault, and every other class are ineligible to cross into Project
authority. Genesis resolves an immutable snapshot, the Project mutation path
copies it atomically, and a durable itemized receipt records every copy,
omission, refusal, and source generation. Existing Projects never follow later
profile changes.

The explicit optional `datum.projects.unit_policy_seed` aggregate wins a
`ProjectDisplayUnits` seed transaction when it has an eligible contribution.
When absent, the snapshot composes the six typed `datum.units.*` seed
descriptors. Absence never shadows those chosen typed values, and the receipt
records which path supplied each value.

Evidence: Q5 clauses 1–8
(`GP_C03_PROJECT_SEED_AND_CONTEXT_DECISION_PACKET.md:145-169`); corrected catalog
(`GLOBAL_PREFERENCES_V1_DESCRIPTOR_CATALOG.md:63-70,109-124,175-183`).

### Finding 3 — management remains consent-bounded

Recommend, Constrain, Pin, and Lock are inert beyond the user's typed,
revocable `AuthorityRelease`. Revocation re-resolves retained values; requests
above the release remain visible and inert. There is no universal bypass.
Personal accessibility carve-outs require a named descriptor and an explicit
justification; they are not blanket exceptions. Context only selects which
already-authoritative fact applies and cannot create, widen, constrain, pin, or
bypass Project or organization authority.

Evidence: Q3/Q4 (`GP_C03_TYPED_AUTHORITY_DECISION_PACKET.md:516-648`); Q6 clause
7 (`GP_C03_PROJECT_SEED_AND_CONTEXT_DECISION_PACKET.md:779-787`); real
`preferences-window.html#organization` and `#workspace`.

### Finding 4 — storage and exchange cannot manufacture authority

GP-C04 defines immutable generations, an expected-generation single-writer
transaction, one head-promotion commit point, current plus two validated
predecessors, and byte-faithful unknown preservation. A platform unable to
provide required locking and atomic replacement refuses persistence rather than
writing unsafely. Downgrade/newer-version and platform-ineligible material stays
preserved and inactive. An unreadable store is never repaired in place.
Capability-class values and managed authority refuse portable sources.
Migration never substitutes a value the user did not choose. Restore is
previewed, confirmed, preserves the current store, and is itself reversible.
No store operation touches Project policy or design data, and none blocks
authoring.

Evidence: GP-C04 (`GP_C04_STORAGE_MIGRATION_EXCHANGE_RECOVERY_CONTRACT.md:114-207,209-360,362-438`);
Claude store states commit `586eb0d`.

### Finding 5 — interaction consumes resolver truth

GP-C05 refined Option A at Claude commit `a061fca` governs: two columns, search
pinned above the settings pane, complete rows, and a resolver-owned explanation
beside the retained list. Search-first discovery covers every row in every
section, including planned and read-only Project-policy rows. Current labels,
descriptions, stable keys, and retired/alternate names are searchable vocabulary;
the reason for an alias/key match is visible. GUI, CLI, and MCP expose the same
typed semantic answer. Accessibility/context states are governed by `e2127fa`,
and store states by `586eb0d`.

Evidence: GP-C05 clauses 1–39
(`GP_C05_INTERACTION_AND_VISUAL_CONTRACT.md:68-226`), especially search clause 7
at `:102-104`; visual commits `a061fca`, `e2127fa`, and `586eb0d`.

### Finding 6 — the corrected catalog is bounded, not falsely complete

The catalog registers 58 active V1 descriptors using `Replace` merge law. It
explicitly defers the three clay rows (grid-size presets, persistent crosshair
opening default, and opening-layer visibility), the two unreviewed agent-
authority descriptors including unattended authority, the unspecified
`AdoptedDraftingStandard` seed schema, and every other unproved candidate.
Library, Symbol Editor, Footprint Editor, and Organization have zero active V1
descriptors. Organization's authority/query rows are not preferences.

The historical intake was 60 current plus 59 planned. PM-036 moved Editor
appearance themes from planned to active, producing the later 61 current plus
58 planned comparison with the same 119 total. These are prototype intake
counts, not registry counts. Effective provisional-watermark state remains
Project/Revision/Publish authority; one future new-Project watermark
`ProjectPolicySeed` candidate is deferred, so no datum is double-classified.

Evidence: catalog active table and deferrals
(`GLOBAL_PREFERENCES_V1_DESCRIPTOR_CATALOG.md:72-183,217-246`); historical seed
(`GP_DRAFT_SETTINGS_CATALOG_SEED.md:1-174`); PM-036
(`PRODUCT_MECHANICS_036_SCHEMATIC_DRAWING_THEMES.md:18-57`).

### Finding 7 — units preserve exact design truth

One engine-owned Units service serves GUI, CLI, and MCP. Authored length remains
checked signed `i64` nanometers. Per-quantity overrides may deliberately choose
the other measurement system and must be returned/rendered as explicit
cross-system overrides, never normalized away. Parse/format results expose the
cross-system state plus quantity, token, explicit/contextual unit, resolved unit,
exact canonical value, precision, and refusal provenance.

Evidence: shared Units requirements 1–12
(`GP_SHARED_UNITS_ENGINE_REQUIREMENT.md:34-74`), especially requirements 4 and 9
at `:43-46,62-65`; canonical IR (`docs/CANONICAL_IR.md:59-77`).

### Finding 8 — visual evidence is authoritative only where settled

Claude commit `d414462` corrected the airwire-culling control to factory Off;
there is no remaining airwire mismatch or ratification exception. Commit
`ed3df1b` corrected all three adopted-standard Context banners to Q6 clause 7,
made search-result controls operate in place, and attached catalog keys broadly.
Commit `4928f26` completes exact 58-of-58 active catalog-key reachability,
including the rendered `datum.schematic.drawing_theme` identity, its searchable
retired draft name, and eight terminal descriptors. Commit `3edd932` marks the
Q5A, Q11, and Q10 studies with their recorded owner dispositions. Clay,
onboarding-state, and unreviewed-agent rows remain visibly/searchably inspectable
but are explicitly excluded in §5; a `data-key` search anchor does not register
a descriptor absent a catalog entry.

Evidence: `preferences-window.html:98,113-127,150,153,166-169,193-213,239-276`
after `d414462`, `ed3df1b`, and `4928f26`; disposition banners in
`guided-setup-study.html`, `start-page-study.html`, and
`revision-carryforward-study.html` at `3edd932`; and
Q6 clause 7 (`GP_C03_PROJECT_SEED_AND_CONTEXT_DECISION_PACKET.md:779-787`).

### Finding 9 — this must ratify through Product Mechanics

Research and packets preserve evidence, but they cannot ratify mechanism.
Pending Product Mechanics 037 contains the proposed mechanism and explicit
non-decisions. GP-C06 approval would promote that numbered record to ratified
doctrine; it would not ratify from `research/` alone. Product Mechanics 029
still requires a separate numbered decision for any new third-party dependency.

Evidence: `CLAUDE.md` specification-governance law; pending
`docs/decisions/PRODUCT_MECHANICS_037_GLOBAL_PREFERENCES_ENGINE.md`; PM-029
(`PRODUCT_MECHANICS_029_DEPENDENCY_AUTHORITY.md:20-72`).

## 2. Complete GP-C03 disposition ledger

Q1–Q11 are the eleven canonical integer dispositions. Q5A is the sole historical
sub-letter exception and remains as recorded; no new sub-letter identity is
created and no disposition is reopened.

| Boundary | Owner disposition and resulting law | Evidence |
|---|---|---|
| Q1 | Q1-A: stable key and subsystem-owned descriptor | `GP_C03_TYPED_AUTHORITY_DECISION_PACKET.md:210-274` |
| Q2 | Q2-A: typed sources/classes; Project mutation, state, and operation input excluded | ibid. `:385-458` |
| Q3 | user-held, typed, revocable AuthorityRelease; named accessibility justification | ibid. `:500-567` |
| Q4 | Q4-A: eligibility → controls → ordinary values; no arrival-order win | ibid. `:591-648` |
| Q5 | Q5-A: `ProjectPolicySeed`, immutable copy-once snapshot, receipt, no following | `GP_C03_PROJECT_SEED_AND_CONTEXT_DECISION_PACKET.md:145-169,195-200` |
| Q5A | Q5A-B+C: guided in place, zero required choices, replayable local state, proposal-only assistant | ibid. `:347-424` |
| Q6 | Q6-A: distinct Session, Context, operation input, restartable, and transient state | ibid. `:755-839` |
| Q7 | Q7-A: validation/refusal and explicit provider states without unrelated blocking | ibid. `:973-1068` |
| Q8 | Q8-A/P2: unknown preservation, explicit alias migration, non-deletion, Manage preferences home | ibid. `:1264-1402` |
| Q9 | Q9-A: one complete resolver-owned provenance query for GUI/CLI/MCP | ibid. `:1584-1686` |
| Q10 | Q10-A/S2/R1: Presentation-only visibility; machine-local ordinary onboarding; eligible non-gating per-Project teaching replay | ibid. `:1918-2030` |
| Q11 | Q11-A (historical Q5B-A): local Start page, engine-truth Recents/seed preview, explicit exclusions | ibid. `:540-613` |

## 3. Exact consolidated contract

Approval would establish only these clauses:

1. Pending Product Mechanics 037 becomes the numbered controlling mechanism;
   the cited GP-C03, GP-C04, GP-C05, GP-C08, and Units artifacts remain its
   evidence and bounded contracts, not independent ratification authority.
2. There is no universal bypass. Eligibility, released control authority, and
   ordinary-value resolution remain distinct, and typed conflict never resolves
   by arrival order.
3. Absence never masquerades as a contribution. Defaults remain descriptor
   truth and are not serialized as user choices by reading, setup, recovery,
   import, migration, or first run.
4. `ProjectPolicySeed` is registered. Only that class may cross Q5; all fifteen
   active seed-bearing descriptors use it, while the sixteenth
   `AdoptedDraftingStandard` seed remains deferred pending schema.
5. An eligible explicit `datum.projects.unit_policy_seed` aggregate wins the
   `ProjectDisplayUnits` seed transaction; when absent, the six typed
   `datum.units.*` seeds compose it. The receipt discloses the path and sources.
6. Units requirements 4 and 9 are controlling: a valid cross-system
   per-quantity override remains explicit and parse/format results carry its
   complete typed provenance.
7. GP-C04's expected-generation exclusive-writer lease and sole head-promotion
   commit point are mandatory; no supported network or local multi-writer merge
   is implied.
8. A platform lacking required atomic replace/locking refuses persistent
   mutation. Downgrade/newer-version and platform-ineligible values remain exact,
   inactive preserved material; they are neither deleted nor guessed active.
9. No unreadable store is repaired in place; unknown data survives every path;
   portable Capability/managed sources refuse; migration substitutes no unchosen
   value; restore is previewed and reversible; Project policy/design data remain
   untouched; none blocks authoring.
10. GP-C05 refined Option A remains controlling. Search scope includes every
    current, planned, and read-only Project-policy row; stable keys and retired
    names are first-class searchable vocabulary.
11. Skip is available at every step, with Esc as equivalent persistent
    dismissal. The minimum setup choice set is zero, and setup never blocks,
    prompts outside the invoked guide, delays, or degrades Design authoring.
12. Preferences → Files & Projects exposes the drawn **Run setup again**
    control and `completed`/`dismissed`/`never run` machine-local state. Replay
    walks the same real rows and applies no setting by itself.
13. Guided setup uses a static outline with no window motion, follows the real
    keyboard tab order, and never substitutes accent color for focus. Optional
    assistant help proposes typed values and applies nothing until accepted.
14. Every personal accessibility management carve-out requires a named
    descriptor and explicit descriptor-level justification; no blanket
    accessibility class bypass exists.
15. PM-034's invariant is controlling verbatim: the Revision Engine “never
    blocks, prompts, or delays Design authoring,” and “ignoring the revision
    engine entirely is a supported, first-class workflow.” Preferences shall not
    strengthen Revision control, visibility, setup, or guidance into an
    authoring gate.
16. Preferences may seed revision policy once and project Revision truth or
    non-gating guidance; it cannot mint identities, mutate journals/records,
    weaken Release gates, or create a competing lifecycle.
17. PM-035 retains `AdoptedDraftingStandard` Project authority. Context may
    select an applicable already-authoritative value but cannot constrain, pin,
    or live-update that Project fact through Preferences.
18. PM-036 retains whole-system schematic theme as machine Presentation state;
    it cannot alter Project/drafting-standard authority, board theme, or
    Publish/print output.
19. The corrected catalog ratifies 58 active descriptors. The explicit §5
    limits remain excluded and require later governed amendment.
20. Approval authorizes no implementation, execution slice, dependency, license
    exception, provider, account/sync service, transport, credential facility,
    GUI toolkit, cryptographic package, storage library, or prototype edit.
21. Approval does not weaken, amend, supersede, or reopen PM-034, PM-035, or
    PM-036. Their doctrine prevails over any Preferences projection.

## 4. Product Mechanics preservation matrix

| Doctrine | Preserved invariant | Preferences may do | Preferences may not do | Evidence |
|---|---|---|---|---|
| PM-034 | Under non-earlier-control profiles, Revision “never blocks, prompts, or delays Design authoring”; ignoring it is supported first-class workflow | seed explicit new-Project revision policy; show/hide projection; non-gating guidance/query | strengthen Revision into authoring gate; mint identity; alter records; weaken Release gates | `PRODUCT_MECHANICS_034_PRODUCT_REVISION_ENGINE.md:173-183`; Q5/Q10 |
| PM-035 | Project documentation owns `AdoptedDraftingStandard` | copy a defined receipted seed; show read-only Context | constrain/pin via machine Preferences; live-follow; reset/import/restore Project standard | `PRODUCT_MECHANICS_035_ADOPTED_DRAFTING_STANDARD_AUTHORITY.md:13-58`; Q5/Q6 |
| PM-036 | schematic theme is whole-system machine Presentation state; print independent | persist/reset/query the governed Dark/Light choice | free-edit palette; alter board, Publish, Project, or drafting-standard authority | `PRODUCT_MECHANICS_036_SCHEMATIC_DRAWING_THEMES.md:13-70` |

## 5. Review limits and completed Claude reconciliation

The following are explicitly **not ratified**:

- agent-authority descriptors, including owner-enabled unattended authority and
  the unattended-tool allowlist, because no dedicated security/authority owner
  review has occurred;
- the `AdoptedDraftingStandard` `ProjectPolicySeed` schema, because its exact
  typed value and registry identity are unspecified;
- Library, Symbol Editor, Footprint Editor, and Organization descriptor sets,
  because each has zero active V1 descriptors;
- the clay grid-size-preset, persistent-crosshair-opening-default, and
  opening-layer-visibility rows; and
- any future provisional-watermark seed schema, provider, implementation,
  dependency, or execution plan.

The completed visual reconciliation establishes these bounded facts:

1. `ed3df1b` preserves Q6 clause 7 at `#workspace`, `#schematic`, and
   `#pcb-board`: Context selects applicability and never creates organization-
   style constrain/pin authority over Project policy.
2. The three unsettled rows remain explicitly marked **classification unsure —
   clay**. Their `data-key` attributes make the candidate rows searchable; they
   do not override this packet/catalog's deferral or register V1 descriptors.
3. Agent authority and unattended-tool rows remain protected candidate evidence.
   Their searchable anchors do not overcome the explicit exclusion above or
   substitute for a dedicated authority/security owner review.
4. Guided setup's searchable state anchor remains Q5A machine-local onboarding
   state, not a preference descriptor or Project/organization authority.
5. `4928f26` proves all 58 active catalog identities occur exactly once in the
   real window. The rendered identities are
   `datum.schematic.drawing_theme`, `datum.projects.startup_view`,
   `datum.projects.template_set`, and `datum.publish.publish_set_naming`; former
   GP-C08 spellings remain searchable vocabulary only.
6. Manage preferences retains reset/import/export as protected store operations,
   not descriptor or classification authority.
7. `3edd932` reconciles the guided-setup, Start-page, and Revision carry-forward
   studies to Q5A-B+C, Q11-A, and Q10-A/S2/R1 without changing their behavior.

No prototype was edited or marked by Codex.

## 6. Owner response

<!-- OWNER:GLOBAL-PREFERENCES-SPEC:GP-C06:GP-C06-CONSOLIDATED-PACKET -->

Approve only if this corrected packet and pending Product Mechanics 037
faithfully consolidate the cited settled authority, retain every §5 exclusion,
and authorize no implementation or dependency.

Reply exactly:

```text
GP-C06-RATIFICATION: approve
```

or:

```text
GP-C06-RATIFICATION: revise — <specific contradiction, omission, or boundary correction>
```

<!-- EVIDENCE:GLOBAL-PREFERENCES-SPEC:GP-C10-CORRECTED-RATIFICATION-PACKET -->
<!-- EVIDENCE:GLOBAL-PREFERENCES-SPEC:GP-C09-CONSOLIDATED-RATIFICATION-PACKET -->
