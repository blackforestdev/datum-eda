# GP-C03 Project Seeding and Context Decision Packet

> **Status:** GP-C03-Q5 approved; active GP-C03-Q5A owner packet. Q1–Q5
> remain approved and closed. Specification only; no implementation, dependency,
> Q5A disposition, or later-question disposition is authorized.
>
> **Tracker:** `dat-global-preferences-engine-qcv`
>
> **Continuation of:** `GP_C03_TYPED_AUTHORITY_DECISION_PACKET.md`

## 1. Preserved authority

This packet continues the one-question-at-a-time GP-C03 process without
reopening its approved foundation:

- Q1: one stable `PreferenceKey` and subsystem-owned descriptor;
- Q2: one resolver over typed sources while Project mutation remains governed;
- Q3: organization directives are gated by typed user-held `AuthorityRelease`;
- Q4: eligibility, controls, then ordinary value ranking; no last-writer wins.

The visual-anchor law assigns Q5 primarily to the real Preferences window and
secondarily to archival PX-V7. A materially different interaction must be
rendered by Claude in the window before an owner boundary opens.

## 2. GP-C03-Q5 — new-Project seeding and receipt

<!-- OWNER:GLOBAL-PREFERENCES-SPEC:GP-C03:GP-C03-Q5 -->

### 2.1 Exact on-screen decision

**In Preferences, every eligible new-Project seed row says that it copies once
at New Project with a receipt, and after creation the Project owns the copied
policy so existing Projects never follow later global changes.**

Q5 decides whether that drawn one-time, receipted copy is the authority contract
or whether seeding instead stays live or loses its durable receipt. It does not
let Preferences mutate an existing Project, decide unavailable-provider behavior
(Q7), or select storage/package transport (GP-C04).

### 2.2 Reviewed visual evidence

The primary visual authority is the Claude-owned real product surface,
`docs/gui/prototypes/preferences-window.html`, rendered and reviewed at
1400×1200:

- `#files-projects`, source lines 215–227: the visible **New-Project seed profile
  selection**, **New-project templates**, and **Default unit system for new
  projects** rows each state “copies once at New Project, with a receipt —
  existing projects never follow later changes.”
- `#revision`, source lines 236–244: the visible revision-profile, build-hierarchy,
  and prototype-to-production seed rows carry the same copy-once receipt law.
- `#publish-space`, source lines 150–163: title-block/template and sheet-format
  seeds copy into new Projects while Project-owned document conventions remain
  explicitly “shown here, edited there.”
- `#units`, source lines 112–126: personal display-unit controls remain machine
  preferences while issued-sheet units and the adopted drafting standard are
  Project policy that Preferences may seed but does not own.

The ratified presentation-class precedent is
`docs/gui/prototypes/canvas-background-decision.html`, source lines 42–44 and
88–119. It shows why classification matters: Candidate B is remembered in
Preferences because the schematic theme is machine Presentation state, while
plotted output remains Project/template authority. Product Mechanics 036 fixes
that boundary; Q5 cannot turn Presentation values into Project seeds.

The archival structural evidence is
`docs/gui/prototypes/preferences-ux-study.html#px-v7`, source lines 360–372.
PX-V7 expands the same window law into a receipt naming the source profile and
generation, copied values, descriptor-ineligible omissions, and the post-copy
Project-ownership rule. It is secondary evidence and does not override the real
window.

No visual reconciliation is required for Q5: the candidate below extracts the
behavior already drawn in all reviewed primary anchors and adds only bounded
engine/evidence clauses needed to make the visible receipt truthful.

### 2.3 Written evidence

- Approved Q2 makes `ProjectPolicy` read-only to Preferences and defines
  `ProjectPolicySeed` as the only setting class eligible for the explicit Q5
  boundary (`GP_C03_TYPED_AUTHORITY_DECISION_PACKET.md:300-329`).
- Approved Q4 resolves eligibility and controls before value ranking, preserves
  losing/refused provenance, and keeps Project authority separate
  (`GP_C03_TYPED_AUTHORITY_DECISION_PACKET.md:586-605,627-636`).
- Altium and Revit distinguish apply-once/template copying from continuing
  control, and make copied standards Project-owned
  (`GP_C02B_DOMAIN_PEER_PREFERENCES_RESEARCH.md:161-220`). The bounded synthesis
  calls for versioned seed material, source generation, an explicit receipt, and
  no silent Project rewrite (`GP_C02B_DOMAIN_PEER_PREFERENCES_RESEARCH.md:297-312`).
- Product Mechanics 035 permits an explicit receipted seed for the Project-owned
  `AdoptedDraftingStandard` and forbids live preference authority afterward
  (`docs/decisions/PRODUCT_MECHANICS_035_ADOPTED_DRAFTING_STANDARD_AUTHORITY.md:18-40`).
- The Product Revision Engine accepts one copied Project policy and forbids later
  global defaults from rewriting it
  (`specs/PRODUCT_REVISION_ENGINE_SPEC.md:111-121`).
- Product Mechanics 036 separately classifies the schematic drawing theme as
  machine Presentation state and keeps it outside Project and Publish authority
  (`docs/decisions/PRODUCT_MECHANICS_036_SCHEMATIC_DRAWING_THEMES.md:18-42,51-57`).

### 2.4 Genuine alternatives

#### Candidate Q5-A — explicit snapshot, atomic copy, durable receipt

Genesis resolves the chosen seed profile/package to one immutable snapshot,
copies only descriptor-eligible values through the Project mutation authority,
and records a durable receipt. Existing Projects never follow later source
changes; a newer generation may only be compared or proposed for deliberate
Project uptake.

This exactly matches the real window and archival PX-V7.

#### Candidate Q5-B — live-linked Project policy

The new Project remembers its source profile and automatically follows later
factory, user, or organization changes. This keeps centrally managed Projects
current with less ceremony, but makes an existing Project's engineering policy
change because external preference state changed.

This is a genuine centralized-management model, but it contradicts every
reviewed seed row's “copies once” and “never follow” text, PM-035's Project
ownership law, and the Product Revision Engine's copied-policy boundary. It
cannot be selected without a new owner decision and a Claude-rendered window
revision, so it does not survive for the present boundary.

#### Candidate Q5-C — one-time copy without a durable receipt

Genesis copies the selected values once and the Project owns them, but Datum
stores no durable itemized origin/omission receipt. This is simpler and still
avoids live rewrite, but “seeded from what, at which generation, with which
omissions?” becomes reconstruction or guesswork.

This contradicts the receipt explicitly drawn in every primary seed row and
PX-V7, and it weakens Q4's queryable provenance. It cannot be selected without a
Claude-rendered removal of the receipt promise, so it also does not survive for
the present boundary.

### 2.5 Exact bounded contract established by Q5-A approval

Approval establishes only these clauses:

1. A `PreferenceDescriptor` explicitly declares `ProjectPolicySeed`
   eligibility and the destination Project-policy identity/type. Presentation,
   Capability, WorkflowDefault, and other ineligible keys cannot cross Q5.
2. Project genesis resolves one explicit factory, user, or organization seed
   selection to an immutable source snapshot before any copy occurs.
3. The Project mutation authority atomically copies every eligible value into
   Project authority and writes one durable `ProjectSeedReceipt`; the
   Preferences store never writes Project facts directly.
4. The receipt records Project identity; source provider/profile/package and
   generation; actor and time; each copied source and destination key/value; and
   each omitted or refused key with its typed reason. It is evidence, not a live
   link.
5. After commit, copied values are Project-owned `ProjectPolicy`. Later factory,
   installation, user, organization, descriptor-default, package, provider, or
   preference changes cannot silently rewrite or invalidate them.
6. A later source generation may produce a typed comparison or proposal only.
   Uptake requires a deliberate governed Project mutation with its own
   provenance; “follow latest” is not a mode.
7. A user may explicitly select an organization seed package for one genesis
   transaction without granting continuing machine authority. Automatic
   organization-selected seeding requires sufficient active `AuthorityRelease`;
   an excess request remains visible and inert under approved Q3/Q4 law.
8. Factory and user-selected seeds use the same typed snapshot/copy/receipt
   mechanism. Profiles may require Project facts but cannot create a second
   Project-policy engine.
9. Q5 does not decide package signing or transport, provider unavailability,
   persistence, migration, detailed comparison UX, reset/import behavior,
   implementation dependencies, initial-setup or Start-page design, or any
   Q5A-Q10 disposition.

### 2.6 Recommendation and owner response

Approve **Q5-A**. It is the only candidate consistent with the real Preferences
window, the approved Q1-Q4 authority model, PM-035, PM-036's class boundary, and
the Product Revision Engine's copied-policy seam.

Reply exactly:

```text
GP-C03-Q5: approve Q5-A
```

or:

```text
GP-C03-Q5: revise — <required on-screen, seeding, receipt, or contract correction>
```

<!-- EVIDENCE:GLOBAL-PREFERENCES-SPEC:GP-C03-Q5-PACKET -->

<!-- EVIDENCE:GLOBAL-PREFERENCES-SPEC:GP-C03-Q5-APPROVED -->
**Owner review — Q5-A approved 2026-08-27.** The owner approved the immutable
snapshot, atomic Project-authority copy, durable itemized receipt, and
no-live-following contract exactly as bounded in §2.5. This closes Q5 without
deciding how a user seed profile is established, whether a Start page exists,
or any Q5A-Q10 behavior. Q5A is the next one-question-at-a-time boundary.

## 3. Drawn follow-on questions queued behind Q5

These questions are registered for one-at-a-time owner review. Q5A is now the
only prepared boundary; Q5B contains no advance contract extraction and cannot
open until Q5A is dispositioned.

### 3.1 GP-C03-Q5A — initial baseline and setup surface

<!-- OWNER:GLOBAL-PREFERENCES-SPEC:GP-C03:GP-C03-Q5A -->

#### 3.1.1 Exact on-screen decision

**On first run, Datum may guide the user through four recommended checkpoints
inside the real Preferences window with a static accent ring, remaining-section
dots, and an explanatory coach bar that can be dismissed at any point, while an
optional assistant may only offer typed proposals for the user to accept,
review, or dismiss.**

Q5A decides whether this drawn guided-in-place path governs, whether any choice
is mandatory before Datum is fully usable, how the static guide remains
accessible, and what local state prevents it from returning. It does not decide
the Start page (Q5B), preference persistence format (GP-C04), or the complete
interaction conformance matrix (GP-C05).

#### 3.1.2 Reviewed visual evidence

All visual sources are Claude-owned, registered route evidence, rendered at
1200 pixels wide, and reviewed without modification:

- Candidate A, `docs/gui/prototypes/first-run-study.html` at commit `849b7f1`,
  source lines 48–87, draws a separate four-step **Welcome to Datum** wizard.
  Its first step duplicates measurement-system and drafting-standard controls;
  Skip uses factory defaults; its summary includes measurement, drafting,
  theme, no organization, and Project files before saving the seed profile.
- Candidate B, the upper state of
  `docs/gui/prototypes/guided-setup-study.html` at commit `b5cac07`, source
  lines 56–74, draws the actual **Preferences — Datum** window. A static accent
  outline marks the current measurement control (CSS lines 36–38), dots mark
  remaining sections, the coach bar names the step and reason, and its controls
  provide Skip, Back, and Next. The caption states that the ring moves while the
  window never jumps, Esc or Skip ends setup forever, and nothing is blocked.
- Candidate C, the lower state of the same prototype, source lines 76–96,
  draws one sentence of user context followed by three typed preference
  proposals and **Accept all**, **Review one by one**, and **Dismiss**. The row
  explicitly says nothing is applied until acceptance; the caption makes C an
  optional accelerator riding on B and preserves B with no AI present.
- The real window anchors the fields B teaches:
  `preferences-window.html#units` at source lines 112–122 supplies factory-backed
  measurement choices, while `#files-projects` at lines 215–224 supplies a
  factory-backed Project location and seed selection. The rendered guide does
  not create a second copy of either surface.

No visual reconciliation is required for the recommended candidate. The
reduced-motion clause below extracts the already-static CSS outline and the
drawn “window never jumps” rule. The keyboard clauses make the drawn controls
operable under ratified accessibility law without adding another visible
surface. The prototype draws no replay/reset affordance, so this packet does
not invent one; adding one would first require a bounded Claude reconciliation.

#### 3.1.3 Written evidence

- Datum is manual-first: every core workflow must work without AI, while AI may
  assist and propose but never becomes hidden authority (`CLAUDE.md:24-31`).
- AI/tool-generated GUI edits must become proposals, and direct manual edits
  remain visible typed operations (`docs/gui/DATUM_GUI_PRODUCT_SPEC.md:168-188`).
  A separate assistant/editor state surface or mutation control that bypasses
  proposal review is forbidden (`docs/gui/DATUM_GUI_PRODUCT_SPEC.md:228-237`).
- Every descriptor supplies a factory-default or no-value state, accessible
  metadata, and apply behavior; merely reading a factory default never writes
  an explicit user choice
  (`GP_C03_TYPED_AUTHORITY_DECISION_PACKET.md:211-229`).
- The approved Q5 contract copies whichever eligible immutable seed snapshot is
  selected and does not depend on how its user contribution was established
  (this packet §2.5 and its recorded Q5 owner-approval evidence).
- Under PM-034, profiles that have not adopted earlier control never block,
  prompt, or delay Design authoring; ignoring Revision authoring is first-class
  (`docs/decisions/PRODUCT_MECHANICS_034_PRODUCT_REVISION_ENGINE.md:173-177`).
  Q5A carries the same non-blocking product law into initial setup.
- The Preferences accessibility research requires keyboard operation without a
  trap, programmatic name/role/value, non-color state, announced status without
  focus theft, and no undisclosed context change
  (`GP_C02_EXTERNAL_AND_STANDARDS_RESEARCH.md:374-396`).

#### 3.1.4 Genuine alternatives

##### Candidate Q5A-A — separate first-run wizard

Datum opens the four-step Welcome wizard drawn in `first-run-study.html`, then
saves its summary as the user's seed profile. It is short, explicit, and
skippable, but duplicates settings already owned by Preferences. The duplicate
surface must track descriptor labels, choices, provenance, accessibility, and
future window changes or visibly drift.

A is genuine and fully drawn, but it conflicts with the later Claude comparison
that identifies this duplication as the reason to prefer B. Selecting A would
require maintaining two settings surfaces and would reject the drawn B premise;
it is not recommended.

##### Candidate Q5A-B — guided in place, manual path only

Datum opens the real Preferences window and guides the user through the four
drawn checkpoints. The current real control receives the accent outline, the
coach bar explains why it matters, and remaining-section dots provide progress.
The user may change a value, keep its factory default, or end setup at any time.

B eliminates the duplicate surface and is a complete manual-first path. It is
the governing candidate.

##### Candidate Q5A-B+C — B governs; C is an optional proposal accelerator

B remains the complete setup path. If the user explicitly invokes assistant
help and an assistant is available, one sentence of context may yield typed
preference proposals. Nothing changes until the user accepts all or accepts an
item during individual review; Dismiss applies nothing.

This is not a rival setup authority. It is the drawn optional extension of B
and the only candidate that preserves both the preferred in-place surface and
Datum's normal proposal law. It is recommended as the complete Q5A disposition.

##### Mandatory-choice alternatives

- **Zero mandatory choices:** factory descriptor defaults make Datum fully
  usable immediately; the four guide checkpoints are recommendations only.
- **Require the two “How you work” choices:** measurement and drafting standard
  must be explicitly chosen before authoring. This offers stronger intent but
  contradicts both drawn Skip paths and would serialize defaults as choices.
- **Require all four checkpoints:** setup must finish before authoring. This
  provides maximum confirmation but directly contradicts “nothing is blocked”
  and PM-034's non-blocking law.

Only zero mandatory choices survives the drawn factory-default Skip behavior,
approved descriptor law, and PM-034. The guide may recommend the four drawn
checkpoints—measurement system, drafting standard for new Projects, Appearance,
and Files & Projects—but no field becomes required, and organization remains
`none` unless the user deliberately changes it.

#### 3.1.5 Exact bounded contract established by Q5A-B+C approval

Approval establishes only these clauses:

1. Candidate B is Datum's initial-setup surface. It guides inside the canonical
   Preferences window; Datum does not ship Candidate A's separate settings
   wizard or maintain a duplicate initial-setup value surface.
2. The minimum required choice set is empty. Datum is fully usable from factory
   descriptor defaults before, during, and after setup; viewing or skipping the
   guide does not serialize those defaults as explicit user choices.
3. The guide recommends exactly the four drawn checkpoints: measurement system,
   drafting standard for new Projects, Appearance, and Files & Projects. A user
   may change a value, retain its displayed factory default, move Back/Next, or
   end setup without completing any checkpoint.
4. Skip is available at every step, and Esc performs the same persistent
   dismissal. Setup never blocks, prompts outside the invoked guide, delays, or
   degrades Design authoring; it creates no Release or Project gate.
5. The accent treatment is a static outline on the current real control. Step
   changes relocate it only after an explicit Back/Next action; under reduced
   motion it appears at the new target without travel, scrolling, pulsing, or
   animation, and the Preferences window never jumps.
6. Keyboard-only operation follows the real Preferences tab order. The current
   control, explanatory coach text, Skip, Back, and Next expose programmatic
   name/role/state; visible keyboard focus is not replaced by accent color; no
   focus trap exists; Enter/Space activates the focused control and Esc dismisses
   setup. Dots are supplementary progress, not the sole state signal.
7. Completion and dismissal are machine-local restartable onboarding state,
   not a preference contribution, user seed value, Project policy, organization
   directive, or evidence that a factory default was explicitly chosen. The
   state records only `not_seen`, `active`, `completed`, or `dismissed` plus the
   last visited checkpoint while active.
8. `completed` and `dismissed` suppress automatic setup on later launches. The
   drawn Q5A surface provides no replay/reset affordance; Q5A therefore approves
   none. Adding replay or reset requires a Claude-rendered control and a later
   owner disposition before GP-C05 may specify it.
9. Candidate C is optional and can exist only on top of B. B remains complete
   when no assistant, model, account, network, or provider is present; Q5A adds
   no dependency or availability requirement.
10. Assistant output is a typed proposal set with per-key proposed value,
    reason, and provenance. The user may accept all, review and accept/reject
    individually, or dismiss; no value applies until the corresponding explicit
    acceptance enters the normal preference mutation path. The assistant never
    writes the store, seed profile, or Project directly.
11. Q5A does not choose Q5B Start-page behavior, persistence representation,
    migration, synchronization, organization enrollment, provider transport,
    implementation dependency, or any Q6-Q10 disposition.

#### 3.1.6 Recommendation and owner response

Approve **Q5A-B+C** with **zero mandatory choices**. B is the governing
manual-first surface; C is optional proposal-only assistance. This exactly
preserves the registered visual states, PM-034, Q1 factory-default semantics,
Q5's profile-origin independence, accessibility law, and the canonical proposal
path without inventing replay/reset UI.

Reply exactly:

```text
GP-C03-Q5A: approve Q5A-B+C
```

or:

```text
GP-C03-Q5A: revise — <required candidate, minimum-set, skip, accessibility, state, replay/reset, or assistant-proposal correction>
```

<!-- EVIDENCE:GLOBAL-PREFERENCES-SPEC:GP-C03-Q5A-PACKET -->

### 3.2 GP-C03-Q5B — Start page

Q5B asks whether the optional Start page exists, what it may show, and what it
must exclude. `start-page-study.html:48-82` draws New/Open/Import actions,
recent Projects with Revision Engine truth and explicit missing paths, a
pre-creation view of the values New Project will copy, and the explicit absence
of news, marketing, telemetry, alerts-channel behavior, and network loading.
It also preserves `Last session` and `Empty` as ways to skip the page.

None of that clay behavior is approved merely by registration. A later packet
must compare genuine alternatives and extract only owner-approved clauses before
GP-C05 reconciles the settled Start-page interaction with Preferences.

### 3.3 Effect on the current Q5 premise

Neither Q5A nor Q5B changes Q5's premise. Q5 governs the invariant copy
semantics after a factory, user, or organization seed snapshot has been chosen:
one atomic copy into Project authority, one durable itemized receipt, and no
later live following. Q5A decides how a user profile may first be established;
Q5B decides whether and how the selected seed values are shown before creation.
The profile's origin or preview surface cannot change what genesis does with it.
