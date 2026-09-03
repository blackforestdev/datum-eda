# Product Mechanics 040: Project Working Units Authority

Status: ratified doctrine

## Context

The UNIT-I02R review exposed a contradictory model. The six Global Units
descriptors were described both as live display/input preferences and as
copy-once Project seeds, even though Product Mechanics 039 already forbids a
Global preference from mutating or live-controlling an existing Project. That
ambiguity also encouraged the Global Preferences prototype to look like the
place where an open Project's units are operated.

Professional CAD precedent resolves the boundary consistently. Onshape account
unit changes apply to new documents, workspace units govern existing workspace
display and default numeric input, explicit unit suffixes override that context,
and drawing units are separate. SOLIDWORKS and Autodesk Inventor likewise keep
units at document/template authority, while Fusion distinguishes future-design
defaults from active-design and drawing settings.

Primary references:

- Onshape, [My Account — Preferences](https://cad.onshape.com/help/Content/Plans/my_account_preferences.htm)
  and [Setting Default Units per Workspace](https://cad.onshape.com/help/Content/Document/setting_default_units_per_workspace.htm)
- SOLIDWORKS, [Units and Dimension Standard](https://help.solidworks.com/2026/English/SolidWorks/sldworks/HIDD_UNITS_DIM_STD.htm)
  and [Document Properties — Units](https://help.solidworks.com/2021/english/SolidWorks/sldworks/hidd_options_units_new.htm)
- Autodesk Inventor, [About Units of Measure](https://help.autodesk.com/cloudhelp/2024/ENU/Inventor-Help/files/GUID-671A8679-AEC1-413A-A23B-428967648CE6.htm)
- Autodesk Fusion, [How to change units](https://help.autodesk.com/view/fusion360/ENU/?caas=caas%2Fsfdcarticles%2Fsfdcarticles%2FHow-to-Change-Units-in-Fusion-360.html)

Onshape is the primary modern professional-CAD reference for this decision.
The other professional systems are corroborating evidence. KiCad is only a
low-weight behavioral comparison and cannot establish Datum's architecture.

## Decision

Datum has one exact engine-owned Units service and three separate authorities:

1. **Global Units defaults** are machine/user preferences for future Projects.
   They use the six registered `datum.units.*` descriptors and save immediately,
   but never change an open or existing Project.
2. **Project Working Units** are Project-owned settings copied at Project
   creation and thereafter changed only through Project Preferences. They govern
   editor display, measurement readouts, and the contextual unit for bare
   numeric input in that Project. Their schema identity is
   `ProjectDisplayUnits`; their product-facing name is Project Working Units.
3. **Publish/document units** are separate Project/document policy governed by
   `AdoptedDraftingStandard` and Publish authority. They control dimensions and
   issued-document presentation and do not silently follow Project Working
   Units or Global defaults.

The six Global descriptors and `ProjectDisplayUnits` share one `UnitsProfile`
value shape so the exact parser, formatter, validation, and resolution semantics
are built once. Sharing a schema and service does not merge ownership.

At New Project, the resolver supplies one immutable Global-default snapshot.
The canonical Project mutation transaction copies it into
`ProjectDisplayUnits` and writes an itemized `ProjectSeedReceipt`. An eligible
explicit `datum.projects.unit_policy_seed` aggregate may replace the composed
six-descriptor snapshot under the existing seed precedence law. After creation,
there is no live link back to Global Preferences.

A Project Working Units change is a journaled Project settings mutation. It may
change display and the meaning of later bare numeric entry, but it never rescales,
rounds, or rewrites canonical geometry. Geometry shards remain byte-identical;
the Project settings and journal legitimately change.

Explicit supported suffixes remain context-independent in GUI, CLI, and MCP.
Headless bare expressions must either carry explicit quantity/unit context or
identify a Project and field from which the adapter resolves Project Working
Units. They must never consult machine Global defaults. Import and export remain
functions of file grammar, explicit operation options, and canonical model
truth, not Preferences display state.

## Interface consequences

Global Preferences labels the Units category as defaults for new Projects and
states that open Projects are unaffected. It does not expose or mirror the
active Project's working or Publish/document units.

Project Preferences is the only general settings doorway for Project Working
Units. The Units category is a required Project-owned payload, but its complete
interaction and protected visual truth remain governed by the pending Project
Preferences specification and are not authorized for implementation here.

Publish/document-unit editing remains on its separately governed Project or
document surface. It is not added to Global Preferences by this decision.

## Owner evidence

On 2026-09-02 the owner approved the researched Onshape/SOLIDWORKS-like model
with: `approved lets move forward with this model`, then directed Datum to
reshape the specification and visual prototypes to accommodate it.

<!-- EVIDENCE:SHARED-UNITS-SURFACE-PARITY:PM-040-OWNER-APPROVED -->

## Non-decisions

This decision does not authorize UNIT-I03 runtime work, Project Preferences
implementation, Publish implementation, a protected-prototype edit, a new
dependency, authored-expression persistence, or production acceptance.
Product Mechanics 029 remains controlling.
