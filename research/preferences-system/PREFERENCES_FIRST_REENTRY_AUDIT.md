# Preferences-First Product Re-entry Audit

Status: RVR-A05 owner-review packet; no implementation authorized

Date: 2026-08-31

## Question and owner direction

This packet answers one product-sequencing question: what must exist before any
Revision UI can safely return?

The owner directed the audit with:

`REVISION-RECOVERY-NEXT: audit — reconcile the Preferences-first implementation sequence and specify the Global/Project settings doorway before Revision UI resumes`

The controlling product intent is:

- settings enter through the application menu bar, never through the Project
  Navigator;
- Global Preferences owns machine/user configuration;
- Project Preferences owns writable policy for the current Project;
- opening settings does not adopt Revision control;
- an unmanaged Project remains silent and fully capable; and
- Revision operations, if later authorized, are not settings and do not become
  permanent navigation groups.

This is a planning packet. It changes no runtime, dependency, public contract,
Project policy, descriptor, or protected prototype.

## Finding 1: the required doorway does not exist

The running application has no general Preferences window. The menu contract
lists `Edit > Preferences`, but `DATUM_GUI_MENU_BINDINGS.md` marks the command
`NOT-BUILT`. The only persisted GUI preference in code is the isolated Console
duration file; it is not the Global Preferences engine.

Project Preferences is less mature. The two protected studies say explicitly
that the surface is clay and not specified. There is no runtime window and no
scheduled implementation item for it.

The current product spec also conflicts with the clarified owner direction:
it places global `Preferences` under Edit and `project settings` under Project,
whereas the owner wants one menu-bar Preferences family that reaches both
scopes. That conflict must be reconciled before GUI implementation.

## Finding 2: the current build order repeats the sequencing mistake

The existing order is:

1. production-accept the complete shared Units program;
2. implement the Global Preferences resolver and repository;
3. implement Project seeding;
4. implement engine, CLI, and MCP surfaces; and only then
5. implement the wgpu Preferences window at GP-I09.

This leaves the manual product surface until near the end. It also creates a
cycle in practice: UNIT-I03 requires GUI/CLI/MCP parity for all six ratified
`datum.units.*` Preferences descriptors, but the Global Preferences build is
hard-blocked until UNIT-I03 completes. The project would again need a temporary
or standalone settings surface to prove a subsystem before the real settings
surface exists.

The exact Units core is a legitimate prerequisite. Full Units Preferences-GUI
parity is not. Those two obligations must be separated.

## Finding 3: Global and Project settings are separate authorities

One visual family does not mean one storage or mutation authority.

| Scope | Owns | Must not do |
| --- | --- | --- |
| Global Preferences | machine/user presentation, workflow defaults, capabilities, typed managed contributions, and eligible copy-once new-Project seeds | mutate an existing Project, adopt Revision control, create Revision records, or force Revision chrome into an unmanaged Project |
| Project Preferences | explicit queries and journaled mutations of current-Project policy and configuration | write the machine preference repository, live-follow later global changes, or hide policy mutation behind a local UI-only writer |
| Revision operations | contextual work on real Changes, baselines, comparisons, evidence, and releases after separate authorization | masquerade as configuration, appear merely because Preferences exists, or occupy permanent Project Navigator groups |

The Global window may show Project policy read-only and link to Project
Preferences. The Project window is the only writable settings surface for the
current Project. Both must identify their scope in words, not color alone.

## Recommended menu contract for owner disposition

Use one application-menu family with two explicit commands:

```text
Edit
└── Preferences
    ├── Global Preferences…
    └── Project Preferences…
```

`Project Preferences…` is disabled when no Project is open. Each command opens
the corresponding scope directly; there is no ambiguous scope toggle and no
Project-Navigator settings node. A Project-menu alias is unnecessary unless a
later usability study proves it materially improves discovery.

This recommendation follows the owner's stated entry point but still requires
RVR-O03 approval and protected-lane reconciliation before it becomes visual or
implementation authority.

## Recommended corrected implementation sequence

The sequence is vertical and manual-first. A surface is not declared stable
from a mock or fixture; every visible row uses real typed authority.

### P1 — govern the settings doorway

Reconcile the GUI product/menu contract and create a buildable Project
Preferences specification. The protected visual lane must render the two menu
commands, Global scope, Project scope, disabled-without-Project state,
read-only cross-link, narrow/keyboard/non-color behavior, and no Navigator
configuration presence.

No runtime work begins from P1 alone.

### P2 — implement the exact Units core only

Retain UNIT-I01's engine-owned checked integer-nanometer parse/format authority
as an internal prerequisite. Do not require the full UNIT-I03 Preferences-GUI
parity gate before the Preferences application exists.

### P3 — build one real Global Preferences vertical slice

Implement the descriptor registry, typed resolver/explanation, minimal durable
repository path, `Edit > Preferences > Global Preferences…`, and the wgpu
window together. The first acceptance slice must use real registered values and
persisted state; it may not be a hard-coded catalog or visual fixture.

The existing Console duration preference is a migration witness, not a rival
repository. The slice must keep Project mutation at zero.

### P4 — complete and stabilize Global Preferences

Expand through the ratified active descriptor catalog, managed/refused states,
unknown preservation, migration/recovery, search, provenance, accessibility,
setup/replay, narrow layouts, and real-project continuity. Complete Units GUI,
CLI, and MCP parity through this real surface, then production-accept Units.

Revision descriptors remain deferred and no empty Revision category is shown.
Engine/CLI/MCP preference parity remains required before Global Preferences
production acceptance, but it does not precede the first manual GUI slice.

### P5 — build and stabilize Project Preferences

Implement `Edit > Preferences > Project Preferences…` against a ratified
Project-settings inventory and protected visual contract. Reads come from
Project authority; writes use the canonical journaled Project mutation path.
The window must prove scope labeling, dirty/conflict/refusal behavior,
read-only global provenance where relevant, keyboard/accessibility, and no
machine-repository mutation.

Project-policy seeding remains copy-once with a receipt. It does not live-link
Global and Project settings.

### P6 — integrate Project Revision configuration

Only after P5 is accepted may a separately ratified Revision category appear in
Project Preferences. It must ship unmanaged, do nothing merely by being viewed,
and make adoption an explicit Project mutation. The policy choices and their
effects require their own owner decision; the existing clay dropdown is not
authority.

The four deferred global `datum.revision.*` keys remain absent unless a later
review proves a genuinely machine/user-scoped need. Project adoption is never a
Global preference.

### P7 — consider contextual Revision operations

Only after Project Revision configuration is real and stable may a later owner
decision authorize contextual operational entry points. Controlled-project
record browsing remains unresolved. No outcome of P1-P6 creates permanent
Navigator groups or restores the rejected REV-I10 surfaces.

## Required plan and Frontier repairs after approval

If RVR-O03 approves this packet, a later governance transaction—not this
audit—must:

1. split the shared Units hard blocker into exact-core and later surface-parity
   obligations;
2. reorder the Global Preferences plan so a real wgpu vertical slice follows
   the minimum resolver/repository core rather than GP-I07;
3. add a governed Project Preferences specification and its own bounded
   implementation item;
4. bind the application-menu commands and remove the conflicting menu wording;
5. request exact protected-prototype changes through the Claude-owned lane;
6. keep every Revision descriptor and runtime surface deferred; and
7. return with explicit execution authorization per bounded slice.

The transaction must preserve the existing typed preference, persistence,
unknown-data, provenance, accessibility, one-mutation-path, and no-new-
dependency laws. Reordering delivery does not weaken proof.

## Acceptance questions for RVR-O03

The owner is being asked to approve or revise only these five statements:

1. The menu entry is `Edit > Preferences > Global Preferences…` and
   `Edit > Preferences > Project Preferences…`.
2. Exact Units core precedes the first Preferences slice, while Units GUI
   parity is completed through the real Preferences window rather than blocking
   its creation.
3. A functional Global Preferences vertical slice is built early, then the full
   Global surface is stabilized before Project Preferences begins.
4. Project Preferences is separately specified and stabilized before any
   Project Revision configuration is integrated.
5. Revision configuration never appears in the Project Navigator, and
   operational Revision surfaces remain deferred beyond this sequence.

Approval authorizes the later governance repair only. It does not authorize P1
implementation, a dependency, a prototype edit, or any Revision behavior.

<!-- EVIDENCE:REVISION-RECOVERY:RVR-A05-PREFERENCES-FIRST-PACKET -->

## Dependency and licensing impact

None. The packet adds no dependency and grants no exception to Product
Mechanics 029.
