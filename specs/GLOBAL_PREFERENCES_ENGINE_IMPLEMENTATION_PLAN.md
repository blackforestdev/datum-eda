# Datum Global Preferences Engine Implementation and Production-Acceptance Plan

> **Status:** Governed planning contract; no execution is authorized.
>
> **Trackers:** `dat-shared-units-engine-build-915`,
> `dat-global-preferences-engine-build-vge`,
> `dat-shared-units-surface-parity-s9x`, and
> `dat-global-preferences-completion-f84`.
>
> **Authority:** Product Mechanics 037, the ratified V1 descriptor catalog,
> GP-C04 storage/recovery contract, GP-C05 interaction contract, Product
> Mechanics 039, and the shared Units requirement. This plan decomposes those
> contracts; it cannot change them.

## 1. Authorization law

This plan places four serial boundaries on the Frontier:

1. the cross-cutting shared Units exact core;
2. the Global Preferences foundation plus first real wgpu vertical slice;
3. shared Units surface parity through that real Preferences surface; and
4. complete Global Preferences stabilization and production acceptance.

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

## 3. Shared Units exact core

The Units engine is not a Preferences module. It is an engine-owned service used
by design storage, resolvers, checks, GUI, CLI, MCP, import/export, and later
Revision/Publish consumers. `dat-shared-units-engine-build-915` owns only the
exact core required before a real Preferences slice.

<!-- REQ:SHARED-UNITS-ENGINE:UNIT-I00 -->
<!-- OWNER:SHARED-UNITS-ENGINE:UNIT-I00:UNIT-I00 -->
### UNIT-I00 — authorize exact-value core only

Owner review may authorize only UNIT-I01. The review must confirm no-new-
dependency posture, module ownership outside Preferences, exact refusal cases,
and focused proof scope.

On 2026-08-31 the owner replied exactly
`UNITS-ENGINE-EXECUTION: approve UNIT-I01`. This authorizes only the exact
engine-owned quantity, parse, format, refusal-provenance, and focused proof
scope below. It authorizes no Preferences surface, descriptor, Project
mutation, Revision work, or new dependency.

<!-- EVIDENCE:SHARED-UNITS-ENGINE:UNIT-I00-OWNER-APPROVED -->

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

UNIT-I01 implements this boundary in `crates/engine/src/ir/units.rs`: typed
quantity/system/unit/precision/override identities, checked decimal-to-rational
parsing, exact signed nanometer conversion, typed refusal provenance, and
deterministic display-only formatting. Existing floating-point adapters remain
explicitly transitional until UNIT-I03 integration; no caller or persisted
schema is silently migrated in this slice.

<!-- EVIDENCE:SHARED-UNITS-ENGINE:UNIT-I01-EXACT-CORE -->

## 4. Global Preferences foundation and first real GUI slice

`dat-global-preferences-engine-build-vge` depends on UNIT-I01, not on Units
surface parity. Its first product result is one functional manual GUI slice,
not an engine/CLI/MCP-only system.

<!-- REQ:GLOBAL-PREFERENCES-ENGINE:GP-F00 -->
<!-- OWNER:GLOBAL-PREFERENCES-ENGINE:GP-F00:GP-F00 -->
### GP-F00 — authorize typed foundation only

Review UNIT-I01 evidence, module boundaries, active catalog digest, no-new-
dependency posture, and focused proof. Approval may authorize only GP-F01.

On 2026-08-31 the owner replied exactly
`PREFERENCES-FOUNDATION: approve GP-F01`. This authorizes only the pure
engine-owned typed descriptor and resolution foundation below. It authorizes no
repository persistence, GUI, Project mutation, Revision behavior, provider,
service, or dependency.

<!-- EVIDENCE:GLOBAL-PREFERENCES-ENGINE:GP-F00-OWNER-APPROVED -->

<!-- REQ:GLOBAL-PREFERENCES-ENGINE:GP-F01 -->
### GP-F01 — descriptors, sources, controls, and resolver

Implement stable `PreferenceKey`, active subsystem descriptors, typed classes
and sources, validation, AuthorityRelease, Recommend/Constrain/Pin/Lock, Q4
resolution, conflicts, retained values, and one pure explanation result.

GP-F01 implements this boundary in `crates/engine/src/preferences/`. The pure
engine module registers the 54 active non-Revision V1 identities, refuses
duplicate/unknown identities and ineligible or invalid facts, gates typed
organization directives through explicit `AuthorityRelease`, preserves
displaced values, refuses arrival-order conflict resolution, and returns the
same complete side-effect-free explanation used by every future surface. It
owns no storage, GUI, Project mutation, provider transport, or Revision path.
The `ProjectPolicySeed` count discrepancy tracked as `dat-vuh` is resolved as a
count-only doctrine/catalog defect: the active post-PM-038 catalog contains
twelve seed rows. The implementation preserves every ratified row's explicit
class and does not restore a withdrawn Revision descriptor.

On 2026-08-31 the owner rejected advancement to GP-F03 and directed a bounded
GP-F01 correction before storage work. The correction replaces inferred
descriptor metadata with the exact catalog contract, resolves `dat-vuh`, makes
Context authority descriptor-specific, completes source/directive refusal and
control-conflict behavior, expands explanation provenance and authority state,
and adds catalog-wide semantic goldens. Existing GP-F01 evidence remains
historical evidence for the initial scaffold; it is not sufficient completion
evidence for this corrective pass. Persistence, GUI, Project mutation, Revision
behavior, providers/services, and new dependencies remain unauthorized.

The corrective implementation declares all 54 active descriptors literally,
including exact schemas, defaults, classes, eligible sources, directive-release
minimums, apply behavior, consumers, export class, accessible copy, and aliases.
Platform- or runtime-derived defaults use registered recipes instead of invented
literals. Context is applicability metadata only and never a universal ranking
source. Organization facts require authenticated, active release authority and
carry disclosure, provider generation state, complete provenance, remaining
freedom, appeal, and reactivation information into the one explanation model.
Typed control validation covers narrowing constraints, pins, locks, incompatible
directives, stale-effective providers, redaction, and deterministic refusal.
Catalog and explanation semantic goldens make meaning changes review-visible.

Proof for the corrected boundary is green: 28 focused preferences tests and 962
engine library tests pass; strict all-target/all-feature workspace Clippy passes
with warnings denied; source-health, rustfmt, dependency-authority, Cargo-resource,
evidence-traceability, spec-governance, spec-parity, and Frontier projection gates
all pass. No persistence code, GUI/prototype file, Project mutation, Revision
behavior, provider service or transport, or third-party dependency was added.

<!-- EVIDENCE:GLOBAL-PREFERENCES-ENGINE:GP-F02-OWNER-REVISION -->

<!-- EVIDENCE:GLOBAL-PREFERENCES-ENGINE:GP-F01-TYPED-FOUNDATION -->

<!-- REQ:GLOBAL-PREFERENCES-ENGINE:GP-F02 -->
<!-- OWNER:GLOBAL-PREFERENCES-ENGINE:GP-F02:GP-F02 -->
### GP-F02 — authorize minimal durable repository only

Review the corrected GP-F01 evidence before authorizing GP-F03. The owner's
2026-08-31 revision disposition returned GP-F01 to execution and did not
authorize storage.

<!-- REQ:GLOBAL-PREFERENCES-ENGINE:GP-F03 -->
### GP-F03 — durable repository foundation

Implement expected-generation single-writer persistence, immutable generations,
exact unknown preservation, typed migration, backup, reversible restore, and
preserved-unreadable recovery sufficient for real visible rows. No provider or
network service is included.

<!-- REQ:GLOBAL-PREFERENCES-ENGINE:GP-F04 -->
<!-- OWNER:GLOBAL-PREFERENCES-ENGINE:GP-F04:GP-F04 -->
### GP-F04 — authorize first real wgpu vertical slice only

Review the resolver/repository evidence and then-current Claude target before
authorizing GP-F05.

<!-- REQ:GLOBAL-PREFERENCES-ENGINE:GP-F05 -->
### GP-F05 — Global Preferences menu and functional wgpu slice

Implement `Edit > Preferences > Global Preferences…` with real registered and
persisted rows, search, resolver-owned explanation, explicit Global scope,
keyboard/accessibility, ordinary/narrow states, and zero Project mutation. No
hard-coded catalog, empty Revision category, Project Preferences, setup flow,
or full-production claim is included.

## 5. Shared Units surface parity through real Preferences

<!-- REQ:SHARED-UNITS-SURFACE-PARITY:UNIT-I02 -->
<!-- OWNER:SHARED-UNITS-SURFACE-PARITY:UNIT-I02:UNIT-I02 -->
### UNIT-I02 — authorize surface parity only

After GP-F05 exists, review exact-core and real-surface evidence before
authorizing UNIT-I03.

<!-- REQ:SHARED-UNITS-SURFACE-PARITY:UNIT-I03 -->
### UNIT-I03 — GUI, CLI, MCP parity and Units production acceptance

Route the six ratified `datum.units.*` descriptors, GUI controls, CLI values,
MCP fields, import/export, and real-Project behavior through the exact service.
Delete or refuse rival conversion paths without moving descriptor ownership
into Units.

## 6. Global Preferences completion

<!-- REQ:GLOBAL-PREFERENCES-COMPLETION:GP-CM00 -->
<!-- OWNER:GLOBAL-PREFERENCES-COMPLETION:GP-CM00:GP-CM00 -->
### GP-CM00 — authorize Global completion only

Review GP-F05 and UNIT-I03 before authorizing GP-CM01.

<!-- REQ:GLOBAL-PREFERENCES-COMPLETION:GP-CM01 -->
### GP-CM01 — complete and stabilize the Global surface

Expand through the full active catalog, managed/refused states, unknown and
retired identities, recovery/migration/restore, setup/replay, Start page,
search/provenance, narrow layouts, and accessibility. Revision descriptors and
empty categories remain absent.

<!-- REQ:GLOBAL-PREFERENCES-COMPLETION:GP-CM02 -->
<!-- OWNER:GLOBAL-PREFERENCES-COMPLETION:GP-CM02:GP-CM02 -->
### GP-CM02 — authorize product-surface and seed parity only

Review GP-CM01 before authorizing GP-CM03.

<!-- REQ:GLOBAL-PREFERENCES-COMPLETION:GP-CM03 -->
### GP-CM03 — engine, CLI, MCP, and Project seed parity

Expose one typed operation/query/refusal/proposal family through engine, CLI,
and MCP with identical explanation semantics and no private writer. Implement
the existing ratified ProjectPolicySeed snapshot/receipt seam with no live
following; excluded or undefined seed schemas remain absent.

<!-- REQ:GLOBAL-PREFERENCES-COMPLETION:GP-CM04 -->
<!-- OWNER:GLOBAL-PREFERENCES-COMPLETION:GP-CM04:GP-CM04 -->
### GP-CM04 — authorize production acceptance only

Approve the exact durable real-Project corpus and resource budgets before
authorizing GP-CM05.

<!-- REQ:GLOBAL-PREFERENCES-COMPLETION:GP-CM05 -->
### GP-CM05 — Global Preferences production acceptance

Run clean-install, upgrade, downgrade, corrupt-store, portable collision,
backup/restore, Project genesis, existing-Project stability, managed/offline,
surface-parity, accessibility, performance/resource, negative, and independent
PM-037/PM-039 proofs on durable real native Projects.

## 7. Ratification exclusions remain blocked

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

## 8. Standing gates and boundaries

Every closure commit runs the affected focused tests plus dependency authority,
source health, cargo-resource policy, specification governance/parity, evidence
traceability, project-state/render, private-writer, daemon-write parity, and
resolver raw-load gates. Rust proof and Clippy run serially through
`scripts/run_cargo_guarded.py`; proof targets stay disk-backed.

No slice authorizes an account/synchronization provider, remote policy service,
credential store, cryptographic mechanism, distributed multi-writer merge,
Revision or Publish implementation, certification claim, or prototype edit.

<!-- EVIDENCE:GLOBAL-PREFERENCES-SPEC:GP-C07-IMPLEMENTATION-PLAN -->
