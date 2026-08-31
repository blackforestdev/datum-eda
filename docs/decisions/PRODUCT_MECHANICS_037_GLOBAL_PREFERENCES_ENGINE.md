# Product Mechanics 037: Global Preferences Engine

Status: ratified doctrine; Revision visibility/default clauses superseded in
part by Product Mechanics 038; delivery sequence amended by Product Mechanics
039

## Context

Datum needs one coherent machine-preference authority without converting
Project policy, design data, operation input, or restartable interaction state
into settings. GP-C03 Q1–Q11 and the recorded Q5A exception settled the typed
authority boundaries; GP-C04 settled persistence, recovery, migration, exchange,
and audit; GP-C05 settled the Preferences interaction; GP-C08 catalogued the
initial descriptor boundary; and the shared Units requirement preserved exact
engine truth. GP-C06 ratified their consolidated mechanism on 2026-08-28.

## Decision

Datum shall use subsystem-owned, versioned descriptors with stable
`PreferenceKey` identities. Typed eligible contributions resolve by Q4's
eligibility, control, and ordinary-value stages. There is no universal bypass.
Absence never masquerades as a contribution. Equal-authority values do not
resolve by arrival order.

`ProjectPolicySeed` is a registered descriptor class. Only that class may cross
the Q5 genesis seam into Project authority; Presentation, Capability, and
WorkflowDefault descriptors may not. A seed is an immutable resolved snapshot
copied atomically by Project mutation authority with a durable itemized receipt.
It never live-follows later profile changes.

The local PreferenceRepository uses expected-generation single-writer
transactions, immutable generations, exact unknown-data preservation, typed
migration, previewed portable exchange, and reversible previewed restore.
Unreadable stores are preserved rather than repaired in place. Downgrade and
platform-ineligible values remain preserved and inactive. Capability values and
managed authority refuse portable sources. Store operations never touch Project
policy or design data and never block Design authoring.

The Preferences window uses the GP-C05 refined Option A interaction at Claude
commit `a061fca`: two columns, search pinned over the settings pane, complete
rows, search-first discovery, and one resolver-owned explanation shared
semantically by GUI, CLI, and MCP. Search covers current labels/descriptions,
stable keys, retired or alternate names, planned rows, and read-only Project-
policy rows. Stable and retired names are searchable vocabulary, not merely
migration machinery.

Product Mechanics 039 controls delivery and entry. Global Preferences is
reached through `Edit > Preferences > Global Preferences…`; Project Preferences
is a separate command and authority. The first functional wgpu Global slice is
built with the minimum real resolver and repository foundation before full
surface parity and production acceptance. Exact Units core remains a
prerequisite, but Units Preferences-GUI parity is proved through the real
Global window and cannot hard-block creation of that window.

After Product Mechanics 038 defers the four Revision descriptors, the initial
catalog contains 54 active V1 descriptors. Eleven active seed-bearing rows are
`ProjectPolicySeed`; the Revision seed family and the additional
`AdoptedDraftingStandard` candidate remain deferred until separately recovered
or specified.
The agent-authority descriptors, including unattended authority, and three
prototype clay rows remain excluded pending dedicated review and visual
reconciliation. Library, Symbol Editor, Footprint Editor, and Organization have
zero active V1 descriptors; their visible queries or planned rows do not imply
registrations.

The six typed `datum.units.*` seed descriptors compose
`ProjectDisplayUnits` when `datum.projects.unit_policy_seed` is absent. An
eligible explicit aggregate contribution to `datum.projects.unit_policy_seed`
wins the seed transaction; its absence cannot shadow the typed contributions.
The receipt records which path supplied every copied value. The shared Units
engine continues to store authored lengths as checked signed integer nanometers;
display and parser preferences never rescale stored design truth.

## Product Mechanics preservation

Product Mechanics 034 remains the sole Product Revision Engine authority. Under
its controlling law, the Revision Engine “never blocks, prompts, or delays
Design authoring,” and “ignoring the revision engine entirely is a supported,
first-class workflow.” Preferences may seed policy at genesis and control
presentation or non-gating guidance only. It may not strengthen Revision into an
authoring gate, mint revision identity, alter records, weaken Release gates, or
create a competing lifecycle.

Product Mechanics 038 further limits this seam: Global Preferences cannot make
Revision chrome visible in an unmanaged Project, cannot pin teaching UI into an
unmanaged personal workflow, and cannot treat a factory Revision profile as
adopted Project policy. Revision visibility, default-profile, teaching, and
onboarding descriptors are pending recovery and are not implementation
authority from the current catalog.

Product Mechanics 035 remains the Project documentation authority for
`AdoptedDraftingStandard`. Preferences may provide a defined receipted seed and
read-only Context doorway, but cannot constrain or pin a Project standard through
a machine preference or live-update it.

Product Mechanics 036 remains the schematic drawing-theme authority. Theme is
machine Presentation state, not Project or drafting-standard authority, and it
does not alter Publish output.

## Accessibility and onboarding

Personal accessibility carve-outs require justification by a named descriptor;
they are not a blanket escape from management law. Guided setup is the real
Preferences window with zero required choices. Skip is available at every step,
with Esc as equivalent dismissal, and setup never blocks authoring. The static
outline follows the real tab order without window motion. Optional assistant
help only proposes typed values and applies nothing until accepted.

Machine-local onboarding records `completed`, `dismissed`, or `never run`.
Preferences → Files & Projects provides the drawn **Run setup again** control;
replay walks the same rows and applies no setting by itself.

## Review exclusions

This decision does not ratify the unspecified
`AdoptedDraftingStandard` seed schema, agent-authority/unattended descriptors,
the four zero-descriptor subsystems, or prototype rows still marked clay. Those
require their own evidence and, for visible behavior, Claude-owned rendering
before a later catalog amendment.

## Non-decisions

This decision authorizes no implementation, Frontier execution, dependency,
license exception, provider, account or synchronization service, network
transport, credential facility, GUI toolkit, cryptographic package, storage
library, or prototype edit. Product Mechanics 029 remains controlling for every
new third-party dependency.

## Owner evidence

<!-- OWNER:GLOBAL-PREFERENCES-SPEC:GP-C06:PM-037 -->

On 2026-08-28 the owner approved the corrected GP-C06 consolidated packet with
the exact response `GP-C06-RATIFICATION: approve`. The mechanism, preservation
law, review exclusions, non-decisions, and dependency boundary above are
therefore ratified without revision.

<!-- EVIDENCE:GLOBAL-PREFERENCES-SPEC:PM-037-OWNER-APPROVED -->

## Dependency and licensing impact

None. This decision grants no dependency or license exception under Product
Mechanics 029.

<!-- EVIDENCE:GLOBAL-PREFERENCES-SPEC:PM-037-PENDING -->
