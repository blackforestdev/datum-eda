# Product Mechanics 039: Preferences-First Settings Entry

Status: ratified doctrine

## Context

The Revision recovery proved that Datum had implemented and exposed subsystem
UI before the settings surfaces intended to contain its configuration existed.
The application menu still marked Preferences `NOT-BUILT`, Project Preferences
was unratified clay, and the Global Preferences plan placed its wgpu surface
near the end. The shared Units plan also required GUI parity for Preferences
descriptors while hard-blocking the Preferences build.

The owner approved the RVR-A05 Preferences-first packet and directed Datum to
follow conventional application settings-entry behavior.

## Decision

Datum has one conventional application-menu settings family:

```text
Edit
└── Preferences
    ├── Global Preferences…
    └── Project Preferences…
```

`Project Preferences…` is disabled when no Project is open. Each command opens
its named scope directly. Scope is stated in words and is never inferred from
color, active document, or the location of a row.

Configuration never appears as a Project Navigator group, row, badge,
placeholder, or empty category. A visible setting is backed by real typed
authority; a mock record or hard-coded catalog cannot stand in for a settings
service.

Global Preferences owns machine/user values, typed management contributions,
and eligible copy-once new-Project seeds. It cannot mutate an existing Project,
adopt Revision control, or make Revision visible in an unmanaged Project.

Project Preferences reads Project-owned authority and writes only through
canonical journaled Project mutations. It does not write the machine preference
repository or live-follow later Global changes. Opening the window changes
nothing.

Revision configuration may appear only in Project Preferences after that
surface is separately specified, built, stabilized, and the Revision category
is separately ratified. Revision operations are not preferences; any later
operational work surface remains contextual and separately authorized.

## Delivery order

1. Govern the menu doorway and Project Preferences contract.
2. Implement the engine-owned exact Units core.
3. Build one real Global Preferences vertical slice spanning typed resolver,
   minimal durable repository, menu command, and wgpu rows.
4. Complete Units GUI/CLI/MCP parity through that real surface.
5. Complete and production-accept Global Preferences.
6. Complete and ratify the Project Preferences specification and protected
   visual truth.
7. Build and stabilize Project Preferences.
8. Only then consider Project Revision configuration and later contextual
   Revision operations through separate owner decisions.

The exact Units core precedes Preferences. Full Units surface parity does not
block creation of the real Preferences surface. Manual GUI capability arrives
before full CLI/MCP and production-acceptance closure, while semantic parity
remains mandatory before production acceptance.

## Conventional does not mean implicit

“Conventional” is bounded to the menu, scope, disabled-state, authority, and
mutation rules above. It does not import another product's apply/cancel model,
window modality, category list, default values, migration behavior, Revision
workflow, or settings storage. Those remain governed by Datum's own specs and
owner-reviewed visual truth.

## Owner evidence

On 2026-08-31 the owner approved the complete RVR-A05 packet with
`PREFERENCES-FIRST-REENTRY: approve` and explicitly directed the conventional
settings-entry and operation rule captured here.

<!-- EVIDENCE:PREFERENCES-FIRST-SETTINGS:PM-039-OWNER-APPROVED -->

## Non-decisions

This decision authorizes no runtime implementation, dependency, provider,
service, descriptor, Project policy mutation, Revision UI, operational Revision
surface, or protected-prototype edit. Product Mechanics 029 remains controlling.
