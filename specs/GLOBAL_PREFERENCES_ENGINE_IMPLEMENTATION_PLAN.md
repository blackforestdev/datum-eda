# Datum Global Preferences Engine Implementation and Production-Acceptance Plan

> **Status:** Governed planning contract; no execution is authorized.
>
> **Trackers:** `dat-shared-units-engine-build-915` and
> `dat-global-preferences-engine-build-vge`.
>
> **Authority:** Product Mechanics 037, the ratified V1 descriptor catalog,
> GP-C04 storage/recovery contract, GP-C05 interaction contract, and the shared
> Units requirement. This plan decomposes those contracts; it cannot change
> them.

## 1. Authorization law

This plan places two programs on the Frontier:

1. the cross-cutting shared Units engine; then
2. the Global Preferences Engine that consumes it.

Every execution slice has its own immediately preceding owner-decision step.
Completing one slice grants no authority for the next one. Product Mechanics
029 continues to require a numbered decision for the exact dependency and its
license obligations before any new third-party code is added, fetched, linked,
or vendored. The initial posture is no new dependency.

Planning, issue creation, and Frontier placement authorize no Rust, migration,
GUI, provider, service, network, account, credential, or prototype work.

## 2. Proof, not restatement

Implementation evidence must exercise the ratified laws through public product
paths. A type declaration or prose restatement is not acceptance. Across the
applicable slices, proofs must demonstrate:

- byte-faithful unknown-data preservation through every preference mutation,
  import, export, migration, recovery, backup, restore, downgrade, and removal
  path;
- refusal of every portable and Project source for every Capability descriptor,
  before precedence is evaluated;
- one immutable resolved seed snapshot copied atomically through Project
  mutation authority, with a durable itemized receipt and no later live
  following;
- one resolver-owned provenance query whose typed semantic result is identical
  in GUI, CLI, and MCP, including absences, retained inert contributions,
  control disposition, winner, Q4 reason, descriptor facts, redaction, and
  remaining actions; and
- keyboard-only completion, reduced-motion equivalence, non-color cues, and
  screen-reader announcements for ordinary, managed, refused, recovery,
  onboarding, search, explanation, and narrow states.

Every Preferences-only proof must also demonstrate zero Project/design mutation
except the explicit Q5 new-Project seed transaction, and uninterrupted Design
authoring throughout recovery, refusal, setup, and management states.

## 3. Cross-cutting shared Units prerequisite

The Units engine is not a Preferences module. It is an engine-owned service used
by design storage, resolvers, checks, GUI, CLI, MCP, import/export, and later
revision/publish consumers. `dat-shared-units-engine-build-915` therefore has
its own Frontier item and must production-accept before the Preferences build
can begin.

<!-- REQ:SHARED-UNITS-ENGINE:UNIT-I00 -->
<!-- OWNER:SHARED-UNITS-ENGINE:UNIT-I00:UNIT-I00 -->
### UNIT-I00 — authorize exact-value core only

Owner review may authorize only UNIT-I01. The review must confirm no-new-
dependency posture, module ownership outside Preferences, exact refusal cases,
and focused proof scope.

<!-- REQ:SHARED-UNITS-ENGINE:UNIT-I01 -->
### UNIT-I01 — exact quantity, parse, and format authority

Implement one checked signed integer-nanometer authored-length authority and one
typed parse/format service. Display precision, rounding, suffixes, locale, and
measurement-system choice never rescale or rewrite stored design truth.
Per-quantity cross-system overrides remain explicit and flagged. Results carry
quantity, original token, explicit/contextual unit, resolved unit, exact
canonical value, precision, override state, and refusal provenance.

**Exit proof:** integer boundary/overflow/refusal tests; exact metric/imperial
round trips; precision changes with identical stored bytes; explicit valid
cross-system overrides; ambiguous/malformed token refusal; deterministic
locale/path-independent results; and source-health/dependency gates.

<!-- REQ:SHARED-UNITS-ENGINE:UNIT-I02 -->
<!-- OWNER:SHARED-UNITS-ENGINE:UNIT-I02:UNIT-I02 -->
### UNIT-I02 — authorize surface parity only

After UNIT-I01 evidence is committed, owner review may authorize only UNIT-I03.

<!-- REQ:SHARED-UNITS-ENGINE:UNIT-I03 -->
### UNIT-I03 — GUI, CLI, MCP parity and Units production acceptance

Route GUI controls, CLI values, and MCP typed fields through the same service;
delete or refuse rival conversion paths. Exercise all six ratified
`datum.units.*` descriptors without placing their ownership inside the Units
service.

**Exit proof:** generated surface inventory, byte-identical typed semantic
answers across GUI/CLI/MCP, real native Project formatting/parsing, import and
export boundary checks, accessibility for unit controls and refusals, guarded
workspace tests, independent source-health review, and explicit owner Units
production acceptance. This does not authorize Preferences execution.

## 4. Global Preferences ordered program

`dat-global-preferences-engine-build-vge` hard-depends on completed shared Units
production acceptance. Its slices are serial at the acceptance boundary.

<!-- REQ:GLOBAL-PREFERENCES-ENGINE:GP-I00 -->
<!-- OWNER:GLOBAL-PREFERENCES-ENGINE:GP-I00:GP-I00 -->
### GP-I00 — authorize typed core only

Review the completed Units evidence, intended module boundaries, descriptor
catalog digest, no-new-dependency posture, and focused proof plan. Approval may
authorize only GP-I01.

<!-- REQ:GLOBAL-PREFERENCES-ENGINE:GP-I01 -->
### GP-I01 — descriptors, sources, controls, and resolver

Implement stable `PreferenceKey`, subsystem-owned versioned descriptors, typed
classes/sources, validation, AuthorityRelease, Recommend/Constrain/Pin/Lock, Q4
eligibility/control/value stages, typed conflicts, retained displaced values,
and the pure resolver/explanation result. Defaults and absences are never
serialized as user contributions.

**Exit proof:** all 58 active descriptors register exactly once; class/source
eligibility matrices; Capability portable/Project refusal; absence and equal-
authority conflict tests; release/revocation and retained-value tests; complete
typed explanation goldens; and no Project/store/UI implementation.

<!-- REQ:GLOBAL-PREFERENCES-ENGINE:GP-I02 -->
<!-- OWNER:GLOBAL-PREFERENCES-ENGINE:GP-I02:GP-I02 -->
### GP-I02 — authorize repository and migration only

After GP-I01 evidence is committed, owner review may authorize only GP-I03.

<!-- REQ:GLOBAL-PREFERENCES-ENGINE:GP-I03 -->
### GP-I03 — persistence, exchange, migration, and recovery

Implement GP-C04's expected-generation single writer, immutable staged
generations, sole head promotion, current plus two validated predecessors,
exact unknown envelopes, downgrade/platform preservation, typed portable
planning/collision, migration receipts, exact backup, previewed reversible
restore, and preserved-unreadable recovery. No provider or network service is
included.

**Exit proof:** interruption at every durability boundary; stale writer and
stale preview refusal; corrupt/truncated store preserved without repair in
place; unknown bytes survive every write path; Capability and managed portable
refusal; no unchosen migration substitution; alias uniqueness; reversible
restore; audit/redaction completeness; offline operation; zero Project/design
mutation; and authoring continuity.

<!-- REQ:GLOBAL-PREFERENCES-ENGINE:GP-I04 -->
<!-- OWNER:GLOBAL-PREFERENCES-ENGINE:GP-I04:GP-I04 -->
### GP-I04 — authorize Project genesis seam only

After GP-I03 evidence is committed, owner review may authorize only GP-I05.

<!-- REQ:GLOBAL-PREFERENCES-ENGINE:GP-I05 -->
### GP-I05 — copy-once ProjectPolicySeed and durable receipt

Implement `CaptureProjectSeedSnapshot` and the one Project-authority genesis
operation. Only `ProjectPolicySeed` descriptors may cross. An eligible explicit
`datum.projects.unit_policy_seed` aggregate wins; otherwise the six typed Units
seeds compose `ProjectDisplayUnits`. Presentation, Capability, and
WorkflowDefault never cross.

**Exit proof:** atomic all-or-nothing copy, durable itemized receipt for values,
sources, omissions, refusals, generations, and aggregate/composed path;
concurrent preference change independence; failure-after-Project-commit
independence; existing Project non-following; and refusal of every non-seed
class. The unspecified adopted-standard and watermark seed schemas remain
absent.

<!-- REQ:GLOBAL-PREFERENCES-ENGINE:GP-I06 -->
<!-- OWNER:GLOBAL-PREFERENCES-ENGINE:GP-I06:GP-I06 -->
### GP-I06 — authorize product-surface parity only

After GP-I05 evidence is committed, owner review may authorize only GP-I07.

<!-- REQ:GLOBAL-PREFERENCES-ENGINE:GP-I07 -->
### GP-I07 — engine, CLI, MCP, and operation/query parity

Expose the approved typed operations, plans, queries, receipts, explanations,
and refusals through one engine service and the existing canonical verb/native-
write path. CLI and MCP remain projections and never parse stores or reconstruct
resolution independently.

**Exit proof:** generated public inventory/parity; identical explanation and
refusal payload semantics across engine/CLI/MCP; expected-generation fences;
proposal-only assistant path; search vocabulary for stable/retired identities;
offline behavior; and no private writer.

<!-- REQ:GLOBAL-PREFERENCES-ENGINE:GP-I08 -->
<!-- OWNER:GLOBAL-PREFERENCES-ENGINE:GP-I08:GP-I08 -->
### GP-I08 — authorize ratified wgpu surface only

After GP-I07 evidence is committed, owner review may authorize only GP-I09 and
must confirm that the Claude-owned visual commits remain the target.

<!-- REQ:GLOBAL-PREFERENCES-ENGINE:GP-I09 -->
### GP-I09 — wgpu Preferences, setup, Start page, and accessibility

Implement the Preferences window exactly against the ratified
`preferences-window.html` state at `a061fca`, its later catalog-key parity, the
accessibility/context render at `e2127fa`, the store states at `586eb0d`, and the
settled guided-setup, Start-page, and Revision carry-forward studies. Search is
pinned over the settings pane; the real row and one resolver answer remain the
authority.

**Exit proof:** structured human screenshot parity between the running wgpu app
and the Claude-owned render at the same viewport/state matrix, followed by
standing wgpu-to-wgpu goldens. Cross-engine pixel subtraction is not a machine
oracle. The review covers ordinary/wide/narrow, search/no-match/alias,
explanation, managed/refused, recovery/migration/restore, setup/replay, Start
page, revision carry-forward, keyboard-only, reduced motion, color-removed, and
screen-reader states. Any material target change first requires a Claude-owned
render and evidence-route reconciliation.

<!-- REQ:GLOBAL-PREFERENCES-ENGINE:GP-I10 -->
<!-- OWNER:GLOBAL-PREFERENCES-ENGINE:GP-I10:GP-I10 -->
### GP-I10 — authorize real-project production acceptance only

After GP-I09 evidence is committed, owner review may authorize only GP-I11 and
must approve the exact real-project fixture set and resource budgets.

<!-- REQ:GLOBAL-PREFERENCES-ENGINE:GP-I11 -->
### GP-I11 — migrations, real Projects, and production acceptance

Run clean-install, upgrade, downgrade, corrupt-store, portable collision,
backup/restore, Project genesis, existing-Project stability, managed/offline,
surface-parity, accessibility, and performance/resource proofs on durable real
native Projects. Independently audit every PM-037 clause and GP-C06 exclusion.

**Exit proof:** all prior slice evidence addressable; guarded locked/offline
workspace tests and strict all-target Clippy; all dependency, source-health,
mutation/resolver, parity, evidence, and project-state gates; deterministic
restore/migration rollback; screenshot review in the running app; negative and
fault-injection corpus; zero accidental Project/design writes; owner production
acceptance; and no implicit successor selection.

## 5. Ratification exclusions remain blocked

These are not implementation backlog hidden inside a slice:

| Excluded set | Blocking reason | Exact unblock requirement |
|---|---|---|
| Agent-authority descriptors, including unattended authority | Claude commits `b6eedfa`, `0293d25`, `60b0d4b`, `d8dd663`, and `15ac6b7` now carry drawn security review and owner dispositions, but PM-037 deliberately excluded these descriptors and no numbered mechanism, typed schema, or catalog amendment has ratified them. | A dedicated Codex contract must extract the disposed capability, grant, refusal, alert, threat-limit, identity-binding, and organization-restriction law from those renders; owner ratification through a numbered decision, catalog amendment, evidence reconciliation, and a separately authorized implementation slice must follow. |
| Three clay rows: grid-size presets, persistent crosshair opening default, opening-layer visibility | The prototype exposes candidates but PM-037 did not register them. | Owner disposition, subsystem descriptor evidence, catalog amendment, and Claude reconciliation removing clay status. |
| `AdoptedDraftingStandard` seed | PM-035 owns Project truth; seed schema is unspecified. | `dat-adopted-drafting-standard-object-er9` lands a ratified schema and seed boundary, followed by catalog/evidence updates and any required Claude render. |
| Library, Symbol Editor, Footprint Editor, and Organization descriptor sets | Each has zero active V1 descriptors. Organization rows are authority/query state, not settings. | Per-subsystem inventory and owner-ratified catalog amendment; any new visible row must be Claude-rendered first. |
| Provisional-watermark seed schema | Effective watermark remains Revision/Publish/Project authority and the future seed has no schema. | Revision/Publish authority defines the seed and export semantics, owner ratifies it, and Claude renders the visible behavior before catalog amendment. |

An anchor, searchable row, or implementation convenience cannot satisfy an
unblock requirement.

## 6. Standing gates and boundaries

Every closure commit runs the affected focused tests plus dependency authority,
source health, cargo-resource policy, specification governance/parity, evidence
traceability, project-state/render, private-writer, daemon-write parity, and
resolver raw-load gates. Rust proof and Clippy run serially through
`scripts/run_cargo_guarded.py`; proof targets stay disk-backed.

No slice authorizes an account/synchronization provider, remote policy service,
credential store, cryptographic mechanism, distributed multi-writer merge,
Revision or Publish implementation, certification claim, or prototype edit.

<!-- EVIDENCE:GLOBAL-PREFERENCES-SPEC:GP-C07-IMPLEMENTATION-PLAN -->
