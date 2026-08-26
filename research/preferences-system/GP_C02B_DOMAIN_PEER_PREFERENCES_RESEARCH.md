# GP-C02B Domain-Peer Preferences Research

> **Status:** bounded domain-peer addendum for GP-C03 Q3/Q4 and GP-C05.
> It supplies evidence and requirement candidates only. It does not adopt a
> peer product model, authorize an online authority, select a dependency, or
> establish a standards-conformance claim.
>
> **Tracker:** `dat-global-preferences-engine-qcv`
> **Frontier:** `GLOBAL-PREFERENCES-SPEC / GP-C02B`
> **Peers:** SOLIDWORKS, SOLIDWORKS PDM, Altium Designer/Workspace, Autodesk
> Revit, and KiCad 9.

## 1. Owner-directed questions

This addendum examines domain-peer behavior specifically for:

1. organization-managed distribution of settings;
2. recommendation, apply-once, constraint, pin, and lock semantics;
3. standards-profile selection as configuration rather than an accidental
   personal preference;
4. settings copy/export/import and portability;
5. offline and unavailable-manager behavior;
6. which peer mechanisms should inform GP-C03 Q3, Q4, and GP-C05;
7. why Datum needs a complete v1 descriptor catalog during specification.

The comparison is intentionally narrower than GP-C02. It asks how professional
engineering tools expose these mechanisms, not which tool Datum should clone.

## 2. Source-strength and limits

All behavioral claims below use vendor-owned documentation current or available
on 2026-08-25. Product documentation is authoritative for the documented peer
behavior but is not a standard and cannot, by itself, become Datum doctrine.

No proprietary file format, API, or implementation is incorporated. References
to registry, Workspace, PDM, deployment images, INI files, and KiCad
configuration are behavioral evidence only. Product Mechanics 029 still
controls dependencies, and Datum's ratified local-authority/subordinate-adapter
law still controls external integrations.

## 3. Source register

### 3.1 SOLIDWORKS and SOLIDWORKS PDM

- [SOLIDWORKS Settings Wizard](https://help.solidworks.com/2024/English/SolidWorks/sldworks/c_solidworks_settings_wizard.htm)
  documents category-selective save, restore, and propagation to a current
  user, network computers, or roaming profiles.
- [Settings Administrator Tool](https://help.solidworks.com/2025/English/SolidWorks/install_guide/c_settings_administrator_tool.htm),
  [Applying and Locking Options](https://help.solidworks.com/2026/English/SolidWorks/install_guide/settings_options_apply_and_lock.htm),
  and [Finish Setting Options](https://help.solidworks.com/2026/English/Installation/install_guide/t_settings_options_finish.htm)
  document applied versus locked options, first-start versus every-start
  application, offline lock choice, API-override choice, administrator identity,
  and a deployable settings artifact.
- [Copy Settings When Options Are Locked](https://help.solidworks.com/2022/English/WhatsNew/c_wn2022_admin_copy_settings.htm)
  states that copied user settings merge with administrator settings and that
  administrator settings win conflicts.
- [PDM User Settings](https://help.solidworks.com/2024/English/EnterprisePDM/admin/c_settings_db.htm)
  documents vault-specific user settings, administrative editing, group-wide
  application, client caching, and user visibility.
- [Document Properties — Drafting Standard](https://help.solidworks.com/2024/english/Solidworks/sldworks/HIDD_OPTIONS_DRAFTING_STD.htm)
  separates an overall document drafting standard, external custom-standard
  files, and other document properties retained in the drawing template.

### 3.2 Altium Designer and Altium Workspace

- [Accessing, Defining & Managing System Preferences](https://www.altium.com/documentation/altium-designer/accessing-defining-managing-system-preferences?space=nexus)
  documents one editor-spanning Preferences surface, local preference files,
  revisioned Workspace preference items, per-page release selection, and
  `Apply and Lock`, `Apply First Time`, and `Do Not Apply` modes.
- [Managing Environment Configurations](https://www.altium.com/documentation/altium-365/managing-environment-configurations?version=1)
  documents administrator-selected configurations that constrain designers to
  company-ratified preference revisions, templates, output jobs, layer stacks,
  and other configuration items.
- [Data Management Preferences](https://www.altium.com/documentation/altium-designer/preferences/data-management)
  documents local and Workspace templates, selected defaults, and environment
  configurations that may restrict available template revisions.
- [Creating a Project Template](https://www.altium.com/documentation/altium-designer/creating-project-template)
  documents project templates containing project/document options, design
  rules, parameters, units, title blocks, release options, and output jobs.

### 3.3 Autodesk Revit

- [About the Revit.ini File](https://help.autodesk.com/cloudhelp/2025/ENU/Revit-Customize/files/GUID-ECD1B2C6-7612-431E-ACDE-92A557862020.htm)
  documents installation defaults copied into a user's profile on first run,
  later use of the user copy, and restart-required changes.
- [Customize Application Settings for a Revit Deployment](https://help.autodesk.com/cloudhelp/2017/ENU/Revit-Installation/files/GUID-9C4D7807-1745-4E8A-8DB8-BB533BBFC561.htm)
  documents custom application settings and project templates distributed in a
  deployment.
- [Install Settings in Revit.ini](https://help.autodesk.com/cloudhelp/2024/ENU/Revit-Customize/files/GUID-7CBDEA96-ADE7-414B-B16C-D5E6F4E1D291.htm)
  documents an explicit update list for selected user settings across an
  organization.
- [Best Practices: Project Templates](https://help.autodesk.com/cloudhelp/2026/ENU/Revit-Customize/files/GUID-7F7D88BC-69E5-4EBB-8B6A-3AFECEC80BDD.htm)
  and [Project Template Settings](https://help.autodesk.com/cloudhelp/2014/ENU/Revit/files/GUID-C63C8425-A2A6-4318-BF84-EE3E67029EC3.htm)
  document templates as the starting environment for project standards,
  parameters, families, views, sheets, print settings, and related facts.
- [Transfer Project Standards](https://help.autodesk.com/cloudhelp/2023/ENU/Revit-Customize/files/GUID-2A49FE80-1B07-4C95-A0FE-42CE2001F3C2.htm)
  documents explicit selective copying with overwrite/new-only/cancel conflict
  choices.
- [Shared Parameters](https://help.autodesk.com/cloudhelp/2024/ENU/Revit-Model/files/GUID-E7D12B71-C50D-46D8-886B-8E0C2B285988.htm)
  documents portable parameter definitions while explicitly stating that their
  information is not automatically applied to another project or family.

### 3.4 KiCad 9 — selected EDA peer

- [KiCad 9 application documentation](https://docs.kicad.org/9.0/en/kicad/kicad.pdf)
  documents a shared Preferences dialog containing common and tool-specific
  categories, per-category reset, session choices, project-backup choices,
  path variables, and hotkey-file transfer between computers.
- [KiCad 9 PCB Editor documentation](https://docs.kicad.org/9.0/en/pcbnew/pcbnew.html)
  documents global and Project-specific library tables, Project-table
  precedence for duplicate nicknames, relocatable paths, and import of board
  settings from a template board.

KiCad is included as the EDA peer because its local, file-oriented behavior is
closer to Datum's offline and Git-friendly constraints than an exclusively
cloud-managed system. Its documentation does not establish an organization
lock/provider mechanism comparable to SOLIDWORKS or Altium.

## 4. Peer findings

### 4.1 SOLIDWORKS: portability and administration are separate mechanisms

The Settings Wizard treats portability as an explicit user/admin operation. A
package can contain selected categories such as system options, toolbars,
shortcuts, gestures, menus, and saved views, then target one user, machines, or
roaming profiles. It does not make the package a continuously authoritative
policy source.

The Settings Administrator Tool adds a distinct administrative mechanism:

- select which settings to apply;
- separately select which applied settings to lock;
- apply only at first start or on every start;
- decide whether locks remain effective offline;
- identify an administrator whose name/contact appears with the lock;
- optionally define an override path;
- distribute a named settings artifact.

The 2022 merge rule is explicit: user-restored settings may coexist with admin
settings, but administrator values win conflicts. This is useful Q4 evidence
for retaining a losing user contribution rather than deleting it.

PDM adds a different administrative domain. Settings are vault-specific and
stored on each user's database record; applying group settings updates current
members rather than defining a simple permanent group-inheritance law. Users
can inspect settings without necessarily having permission to modify them.

The drafting-standard surface is the important standards-profile precedent.
ANSI/ISO/DIN/JIS/BSI/GOST/GB and custom standards are document properties,
while the drawing template retains additional document facts. SOLIDWORKS does
not treat the selected engineering standard as merely a roaming UI preference.

**Transferable mechanisms:** explicit export scope, apply versus lock, first-use
versus continuous application, offline lock disposition, attributed management,
inspect-without-edit, and document/template ownership of engineering standards.

**Do not copy:** registry-shaped storage, password override as the universal
governance model, or PDM's current-member copy behavior as hidden dynamic group
inheritance.

### 4.2 Altium: revisioned preference packages and environment configurations

Altium presents global and editor-specific preferences in one categorized
surface. It supports two portability paths:

1. a local preferences file; and
2. a revisioned Designer Preferences item in a Workspace.

When releasing a Workspace preference revision, the administrator selects
content by preference page and assigns one of three modes:

- `Apply and Lock` — loaded read-only while Workspace control applies;
- `Apply First Time` — initial value that the user may later edit;
- `Do Not Apply` — leave the existing setting alone.

Environment Configurations compose one selected preference revision with
company-ratified templates, output jobs, layer stacks, and other design
elements. This demonstrates that a professional configuration profile is not
just one preference blob: it is a manifest selecting revisions of several
typed configuration assets.

Altium's project templates further separate environment preference from Project
authority. They can seed design rules, parameters, units, title blocks, release
options, and output jobs into a new Project. That parallels Datum's ratified
copy-then-Project-owns seam more closely than a live global preference link.

**Transferable mechanisms:** revisioned configuration packages, inclusion by
typed category, apply-once versus lock, profile manifests over heterogeneous
configuration assets, and explicit template-based Project seeding.

**Do not copy:** a connected Workspace as release or preference authority.
Datum must remain locally complete; any future organization service is a
subordinate provider whose packages are validated, quarantined, cached, and
resolved by Datum authority.

### 4.3 Revit: deployment defaults, user copies, and Project-owned standards

Revit exposes a useful split even though its mechanisms are less unified:

- a deployment/UserDataCache `Revit.ini` supplies initial application defaults;
- first run copies those defaults into the user's profile;
- later edits use the user copy;
- an organization may explicitly list selected settings to update;
- project templates establish standards for new projects;
- Transfer Project Standards performs a deliberate copy between existing
  projects with explicit conflict choices.

The difference between initial copy and selected later update is material.
“Deployed by the organization” does not identify whether a value is a seed, a
continuing recommendation, or a current lock. Datum must make that lifecycle a
typed fact rather than infer it from file location.

Revit templates and transfer operations also reinforce the Project boundary.
Standards such as units, object styles, view templates, parameters, annotation,
print settings, and sheets become Project facts. Shared parameter definitions
can be portable without silently applying their values everywhere.

**Transferable mechanisms:** first-run copy, explicit selected update, Project
templates as standards seeds, deliberate standards transfer, dependency-aware
copy, and conflict choices.

**Do not copy:** an INI section/path as semantic identity or deployment layout
as implicit provenance. Datum needs Q1's registered keys and receipts.

### 4.4 KiCad: local portability and clear global/Project separation

KiCad's application Preferences are shared across running tools but organized
into common and editor-specific categories. Hotkeys can be exported/imported
through files, and path variables make libraries relocatable across machines.

KiCad also distinguishes global and Project-specific library tables. Project
tables can take precedence for duplicate nicknames, but the tables remain
separate visible artifacts. Board settings can be imported deliberately from a
template board rather than becoming a live preference feed.

This is strong evidence for local portability and contextual Project data, but
weak evidence for managed policy. The official material reviewed here does not
describe organization recommendations, attributed constraints, or locks.

**Transferable mechanisms:** one discoverable surface across tools, explicit
global versus Project artifacts, relocatable path variables, category-specific
reset, file portability, and deliberate template import.

**Do not copy:** assuming that Project-over-global precedence for library
nicknames is a universal preference precedence rule. That rule belongs to the
library-table domain.

## 5. Cross-peer synthesis

### 5.1 No peer has only one undifferentiated “settings” concept

Every peer separates at least three concerns:

1. personal/application preferences;
2. organization deployment or managed configuration;
3. document/Project templates and engineering standards.

Portability is a fourth concern: saved settings packages, hotkey files,
external standards, project templates, or explicit standards transfer.

This confirms Q2-A. A unified Datum resolver is valuable, but a writable
Global Preferences store must not own Project standards, operation inputs, or
restartable state.

### 5.2 Organization control requires typed intent, not merely priority

Across SOLIDWORKS and Altium, materially different modes recur:

- **seed/apply once:** provide an initial value and then permit user ownership;
- **apply/recommend:** supply an organization value that remains overridable;
- **constrain:** limit the allowed value domain;
- **pin/lock:** make a selected value effective and refuse ordinary override;
- **omit/do not control:** make no contribution for that setting.

The exact vocabulary differs, but collapsing these modes into one high-priority
organization value loses essential intent. Q3 should preserve the PX-V5 ladder
while defining `pin` as a value-specific form of lock rather than a separate
authority engine.

### 5.3 Precedence is conditional on the control mode and setting class

SOLIDWORKS documents administrator settings winning direct conflicts. KiCad
documents Project library nicknames taking precedence over global ones. These
are valid domain rules, not proof of one universal order.

The peers instead support a two-stage Q4 model:

1. establish eligibility and apply constraints/locks;
2. rank remaining eligible value contributions using the descriptor's declared
   precedence law.

Losing values should remain inspectable. SOLIDWORKS merges user and admin
settings; Altium distinguishes applied/locked from first-time values. Neither
requires erasing a user's stored preference merely because a managed value is
currently effective.

### 5.4 Standards-profile selection is configuration composition

SOLIDWORKS drafting standards, Revit project templates/standards, and Altium
Environment Configurations all show that engineering standards are broader than
an application preference. They combine document rules, templates, libraries,
output definitions, and sometimes user-environment defaults.

Datum should therefore treat a standards/profile selection as a typed
configuration manifest that may:

- recommend or lock eligible Preferences;
- select versioned Project-policy seed material;
- select templates and presentation defaults;
- record source identity and generation;
- produce an explicit Project seed receipt;
- never silently rewrite an existing Project.

This is compatible with the approved Revision scheme/profile direction while
preventing “ISO mode” from becoming one magic boolean.

### 5.5 Portability needs selection, preview, and provenance

The peer evidence supports explicit export/import packages with category or
page selection. Datum should improve on these precedents by requiring:

- descriptor-key identity rather than UI/registry paths;
- a manifest of included and intentionally omitted keys;
- source product/schema version and generation;
- portability/export eligibility from each descriptor;
- preview of changes and managed conflicts before apply;
- preservation of unknown identities;
- no import-time authority escalation;
- a durable receipt or audit result.

Exact package format, signing, migration, and conflict choreography remain
GP-C04 decisions.

### 5.6 Local/offline behavior must be explicit

SOLIDWORKS exposes whether locks persist offline. Altium distinguishes values
applied once from values controlled while connected. Revit and KiCad rely on
local copies/files. The portable lesson is not one timeout value; it is that
unavailable-manager behavior must be declared per managed package or policy and
shown to the user.

Datum's local-complete posture requires an authenticated/validated local policy
generation with explicit effective, stale, expired, revoked, or unavailable
state. GP-C03 Q7 and GP-C04 must decide retention and refusal details.

## 6. Requirements fed into GP-C03 Q3 and Q4

### 6.1 Q3 — recommendations, constraints, pins, and locks

Q3 must cite and reconcile PX-V5 with these peer-backed requirements:

1. organization intent is typed as no-control, seed/apply-once,
   recommendation, constraint, or lock;
2. a pin is a lock to a specific value, not a separate resolver;
3. a constraint may narrow range/enum/structure without supplying the winning
   value;
4. managed control identifies provider, policy/package, generation, author or
   accountable role, reason, and remaining user freedom;
5. inspectability does not require edit authority;
6. offline continuation/expiry is an explicit policy fact, not inferred from
   current connectivity;
7. override/deviation, if a descriptor/profile permits it, is explicit,
   authorized, bounded, and auditable—not a universal password bypass.

### 6.2 Q4 — precedence and conflict

Q4 must cite and reconcile PX-V6 with these peer-backed requirements:

1. source family alone does not define universal precedence;
2. constraints and locks are evaluated separately from ordinary value ranking;
3. the descriptor declares the ranking/merge law for eligible contributions;
4. an active lock may make a lower-ranked managed value effective without
   deleting a user's stored contribution;
5. same-authority incompatible managed contributions become a visible conflict
   unless the descriptor defines a deterministic join;
6. losing, inert, refused, and conflicting contributions remain queryable;
7. lifting a lock re-resolves retained contributions rather than inventing a
   new value.

## 7. Requirements fed into GP-C05

GP-C05 must use the domain-peer evidence without reproducing peer limitations:

- retain one categorized/searchable Preferences surface spanning subsystems;
- show managed state with word, glyph, value, source, reason, and remaining
  freedom—never color or disabled styling alone;
- expose accountable administrator/role and an appeal/deviation path where one
  exists;
- support inspect-without-edit;
- distinguish `recommended`, `constrained`, `pinned`, and `locked` on the row;
- provide explicit save/export and load/import surfaces with category selection,
  preview, omissions, compatibility, and provenance;
- distinguish profile composition from a single-setting change;
- show offline/stale/expired provider state without blocking unrelated Design
  work;
- preserve PX-V5 through PX-V12 as the reviewed visual source of truth for
  GP-C03 Q3-Q10; materially different candidates must be rendered before their
  owner boundary opens.

## 8. V1 descriptor catalog consequence

The peer products reveal a recurring governance weakness: administrative tools
control a documented subset of settings, and different categories/assets use
different packaging rules. Datum cannot postpone that coverage question until
implementation.

The Global Preferences specification must therefore deliver a governed **V1
Preference Descriptor Catalog** covering every Datum subsystem. For every
planned v1 key, the catalog must name at least:

- owning subsystem and stable `PreferenceKey`;
- user-facing purpose and consequence;
- value type, default/no-value state, constraints, and merge category;
- `PreferenceClass` and eligible resolution sources;
- organization recommendation/constraint/lock eligibility;
- Project-policy seed eligibility and destination authority;
- apply-live/restart behavior and affected consumers;
- capability/security/export sensitivity;
- portability/import/export eligibility;
- accessible label, description, state wording, and reset semantics;
- schema version, legacy source/key, migration identity, and retirement aliases;
- initial implementation disposition: implement, migrate, intentionally defer,
  or explicitly not-a-preference.

The catalog must include negative classifications for values that look like
preferences but are restartable state, transient state, Project authority, or
operation input. That prevents later implementers from recreating the scope
confusion Q2-A removed.

## 9. Findings and limits

The domain-peer survey confirms the architecture direction without making it
automatic doctrine:

- professional tools separate personal settings, managed deployment, Project
  standards/templates, and portability;
- apply-once, recommendation, constraint, and lock are materially distinct;
- standards selection is a configuration/profile composition problem;
- portability is explicit and selective;
- offline managed behavior must be declared;
- no surveyed peer provides Datum's complete typed identity, provenance,
  Project-seed receipt, unknown-preservation, and local-authority contract as one
  coherent mechanism.

Q3 and Q4 must decide Datum's mechanism against both this addendum and the PX
visual anchors. GP-C05 must turn the decided facts into accessible interaction.
No peer's server, file format, registry model, password override, or precedence
order is adopted by this research.

<!-- EVIDENCE:GLOBAL-PREFERENCES-SPEC:GP-C02B-DOMAIN-PEER-RESEARCH -->
