# GP-C03 Project Seeding and Context Decision Packet

> **Status:** GP-C03-Q5, amended GP-C03-Q5A, GP-C03-Q11, and GP-C03-Q6
> approved; active GP-C03-Q7 owner packet. Q5B is the historical alias for Q11.
> Specification only; no implementation, dependency, Q7 disposition, or later-question disposition
> is authorized.
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

These inserted questions completed one-at-a-time owner review. Q5A remains the
recorded sub-letter exception. The Start-page question is canonical Q11; its
original Q5B packet identity remains a historical alias.

### 3.1 GP-C03-Q5A — initial baseline and setup surface

<!-- OWNER:GLOBAL-PREFERENCES-SPEC:GP-C03:GP-C03-Q5A -->

#### 3.1.1 Exact on-screen decision

**On first run, Datum may guide the user through four recommended checkpoints
inside the real Preferences window with a static accent ring, remaining-section
dots, and an explanatory coach bar that can be dismissed at any point, while an
optional assistant may only offer typed proposals for the user to accept,
review, or dismiss; Files & Projects exposes the machine-local completion state
and a Run setup again control that walks the same rows without changing them.**

Q5A decides whether this drawn guided-in-place path governs, whether any choice
is mandatory before Datum is fully usable, how the static guide remains
accessible, and what local state prevents it from returning. It does not decide
the Start page (Q11), preference persistence format (GP-C04), or the complete
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
- The owner-requested replay reconciliation landed in the Claude-owned real
  window at commit `b8f5a7a`: `preferences-window.html#files-projects`, source
  lines 228–231, draws **completed Aug 27**, **Run setup again**, the
  completed/dismissed/never-run state set, and the promise that replay walks the
  same rows and changes no setting by itself.

No further visual reconciliation is required for the recommended candidate. The
reduced-motion clause below extracts the already-static CSS outline and the
drawn “window never jumps” rule. The keyboard clauses make the drawn controls
operable under ratified accessibility law without adding another visible
surface. The replay amendment extracts the control now drawn in the real window;
it does not invent a second setup surface or setting mutation.

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
   state records only `never_run`, `active`, `completed`, or `dismissed` plus the
   last visited checkpoint while active.
8. `completed` and `dismissed` suppress automatic setup on later launches.
   Preferences → Files & Projects exposes the current machine-local state as
   completed, dismissed, or never run and provides **Run setup again**. Replay
   walks the same real Preferences rows and applies no setting by itself; Skip
   and Esc again set the persistent dismissed state. The control reads and
   changes onboarding state only, never Project or organization authority.
9. Candidate C is optional and can exist only on top of B. B remains complete
   when no assistant, model, account, network, or provider is present; Q5A adds
   no dependency or availability requirement.
10. Assistant output is a typed proposal set with per-key proposed value,
    reason, and provenance. The user may accept all, review and accept/reject
    individually, or dismiss; no value applies until the corresponding explicit
    acceptance enters the normal preference mutation path. The assistant never
    writes the store, seed profile, or Project directly.
11. Q5A does not choose Q11 Start-page behavior, persistence representation,
    migration, synchronization, organization enrollment, provider transport,
    implementation dependency, or any Q6-Q10 disposition.

#### 3.1.6 Recommendation and owner response

Approve **Q5A-B+C** with **zero mandatory choices** and the rendered replay
amendment. B is the governing
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

<!-- EVIDENCE:GLOBAL-PREFERENCES-SPEC:GP-C03-Q5A-APPROVED -->
**Owner review — Q5A-B+C approved with amended clause 8, 2026-08-27.** The owner
approved every Q5A clause as packeted except the original no-replay clause and
replaced it with the Claude-rendered `b8f5a7a` control: setup is replayable from
Preferences → Files & Projects; the control exposes completed, dismissed, and
never-run machine-local state; replay walks the same rows and applies nothing by
itself; and Skip/Esc remain persistent. B remains the sole shipped setup surface,
the mandatory choice set remains empty, and C remains optional proposal-only
assistance. This closes Q5A and advances only to the Start-page question.

### 3.2 GP-C03-Q11 — Start page

<!-- OWNER:GLOBAL-PREFERENCES-SPEC:GP-C03:GP-C03-Q11 -->
<!-- OWNER:GLOBAL-PREFERENCES-SPEC:GP-C03:GP-C03-Q5B -->

**Naming ledger.** `GP-C03-Q11` is the canonical forward identity. The owner
approved this boundary using its original `GP-C03-Q5B` name; Q5B is retained
only as a historical alias so committed references and markers continue to
resolve. Q5A remains unchanged as the dispositioned exception. Future inserted
questions use Q12, Q13, and so on; no further sub-lettered IDs are permitted.

#### 3.2.1 Exact on-screen decision

**When Startup is set to Start page and no Project is open, Datum shows one
local, non-tabbed surface with New Project, Open, and Import actions, a Recent
list that states only engine-known Project truth, and a read-only rail previewing
the exact seed values the next New Project will copy; it contains no news,
marketing, alerts channel, telemetry, or startup network load.**

Q11 decides whether this page exists, whether it is the factory startup default,
which local facts it may show, and which content channels are forbidden. It does
not implement Project opening/genesis, define recent-list persistence format, or
change Q5's copy transaction.

#### 3.2.2 Reviewed visual evidence

The Claude-owned sources were rendered and reviewed without modification:

- `docs/gui/prototypes/start-page-study.html` at commit `15328f4`, source lines
  48–82, draws one **Datum** page with no Project open and no work running.
  Lines 53–65 show New Project, Open, Import a design, four Recent examples,
  Revision Engine truth, and an explicit missing-path/Locate state. Lines 67–78
  show machine context, a Preferences doorway, and the values the next Project
  will copy. Line 82 explicitly excludes news, marketing, telemetry, and network
  loading; preserves Last session/Empty; and forbids guessed recent state.
- The primary real-window anchor is
  `docs/gui/prototypes/preferences-window.html#files-projects`, source lines
  215–227. Its first row draws exactly **Start page | Last session | Empty** and
  marks Start page as the factory default. The same section draws the seed
  profile, templates, and unit-system values whose Q5 copy semantics the rail
  previews.
- `docs/gui/prototypes/canvas-background-decision.html`, source lines 42–44 and
  88–119, remains the PM-036 presentation-class precedent: a machine-facing
  choice may control a local view without changing Project or Publish truth.
  Q11 likewise cannot promote the Start page or its rail into Project authority.
- The archival `preferences-ux-study.html#px-v8`, source lines 374–385, keeps
  session/context/restartable state visibly distinct from persisted preference
  authority. It supplies boundary structure only; it does not override the new
  Start-page render.

No visual reconciliation is required for Candidate Q11-A. A tabbed content hub,
feed, alert stream, or network-backed card would materially differ from the
render and cannot become an owner candidate until Claude draws it first.

#### 3.2.3 Written evidence

- The GUI's canonical File surface names New Project, Open Project, and Import
  as product actions (`docs/gui/DATUM_GUI_PRODUCT_SPEC.md:115-124`). The Start
  page presents those same actions; it does not create alternative verbs.
- The approved Q1 descriptor contract distinguishes a factory default from an
  explicit user value (`GP_C03_TYPED_AUTHORITY_DECISION_PACKET.md:211-229`), so
  showing Start page by default does not serialize a user choice merely because
  the page was viewed.
- Q5-A establishes one immutable selected seed snapshot, atomic Project copy,
  and durable receipt; existing Projects never follow later values (this packet
  §2.5 and its recorded Q5 approval). A read-only preview can disclose those
  inputs but cannot perform genesis early.
- The Revision Engine consumes one resolved Project policy and forbids later
  global defaults from silently rewriting it
  (`specs/PRODUCT_REVISION_ENGINE_SPEC.md:111-121`). Revision chips on Recent
  rows must therefore be engine queries, never presentation guesses.
- Datum's Project/Revision authority remains fully functional without Git or a
  network (`specs/PRODUCT_REVISION_ENGINE_SPEC.md:301-315`). A startup surface
  cannot make a network content service a precondition for local work.
- Preferences accessibility requires keyboard operation, programmatic
  name/role/value, non-color state, and no undisclosed context change
  (`GP_C02_EXTERNAL_AND_STANDARDS_RESEARCH.md:374-393`).

#### 3.2.4 Genuine alternatives

##### Candidate Q11-A — the drawn local Start page

Datum ships the rendered Start page as the factory startup default while
retaining Last session and Empty. It prioritizes immediate local actions and
Recent Projects, exposes honest engine truth, and makes the next Q5 seed visible
before creation. It performs no work and contacts nothing until the user acts.

This is the only candidate that matches both registered visual sources and is
recommended.

##### Candidate Q11-B — no Start page

Datum offers only Last session and Empty. New/Open/Import remain in File and the
command palette, and recent Projects remain elsewhere or absent. This is simpler
and eliminates one surface, but contradicts the real Startup preference's named
Start page, removes the drawn pre-creation seed preview, and discards the
rendered honest Recent doorway.

B is genuine but cannot be selected without a Claude-rendered removal from the
real window.

##### Candidate Q11-C — tabbed welcome/content hub

Datum opens a multi-tab Welcome surface that separates Recents, Learn, News,
Alerts, and account/network content. This pattern can carry more onboarding and
commercial material, but it buries the primary Recent list, creates a startup
content and alert channel, invites telemetry/network work, and conflicts with
every explicit exclusion in the rendered study.

C is genuine comparative evidence but does not survive the render-first law.

#### 3.2.5 Exact bounded contract established by Q11-A approval

Approval establishes only these clauses:

1. Datum has one Start page. It is a local idle surface shown only when no
   Project is open and the machine's Startup preference resolves to Start page;
   it does no work until the user invokes an action.
2. `Startup` is one machine-local preference with exactly **Start page**, **Last
   session**, and **Empty**. Start page is the factory descriptor default;
   selecting Last session or Empty bypasses the page without deleting Recent
   history or changing Project authority.
3. The page exposes the same typed **New Project**, **Open**, and **Import a
   design** actions as Datum's canonical File surface. It creates no second
   mutation path and cannot silently open, import, or create anything.
4. Recent rows may show only locally known Project display name, last-known
   path, last-opened time, and engine-derived Revision standing. The drawn
   standing vocabulary includes issued revision, changes since last Release,
   and no Releases yet; absent or unresolved truth is stated as unknown rather
   than inferred.
5. A missing Project path remains visible with **not found at this path** and a
   Locate action. Selecting the row cannot fail silently, remove it silently,
   or pretend the Project was opened.
6. The right rail may show the effective machine seed profile, measurement
   system, organization context, and theme, plus the effective revision profile,
   drafting standard, and templates the next New Project would use. Every value
   is resolver/query output with provenance available through Preferences, not
   a duplicate editable store.
7. The rail is a read-only pre-creation preview. It neither freezes a snapshot
   nor copies Project facts; only the explicit New Project genesis transaction
   performs Q5's immutable resolution, atomic copy, and durable receipt. A
   **Preferences…** doorway lets the user change eligible machine values first.
8. The page contains one immediately visible Recent list; tabs cannot bury or
   partition Recents. It has no News, Learn/marketing, advertising, release-note,
   account-engagement, or alerts-channel surface.
9. The page emits no telemetry and performs no network request, update check,
   remote-content fetch, account lookup, or background synchronization at
   startup. Network-capable actions remain explicit operations outside Q11.
10. Revision standing and missing-path conditions use text in addition to color.
    All actions and Recent rows follow a predictable keyboard order, expose
    programmatic names and states, and do not trap or steal focus.
11. Clearing Recent history changes only machine-local history. It cannot delete
    a Project, Revision record, seed profile, receipt, or Project policy.
12. Q11 does not specify recent-history storage/migration, Project session
    restoration, import/genesis implementation, external providers,
    dependencies, or any Q6-Q10 disposition.

#### 3.2.6 Recommendation and owner response

Approve **Q11-A**. It is the only candidate consistent with the rendered Start
page, the real Startup preference, Q5's visible-before-copy seam, local engine
truth, and the explicit rejection of a tabbed network/content channel.

The boundary originally requested and received this historical response:

```text
GP-C03-Q5B: approve Q5B-A
```

or:

```text
GP-C03-Q5B: revise — <required existence, startup-preference, recent-truth, seed-preview, exclusion, or accessibility correction>
```

<!-- EVIDENCE:GLOBAL-PREFERENCES-SPEC:GP-C03-Q5B-PACKET -->

<!-- EVIDENCE:GLOBAL-PREFERENCES-SPEC:GP-C03-Q11-PACKET -->

<!-- EVIDENCE:GLOBAL-PREFERENCES-SPEC:GP-C03-Q11-APPROVED -->
**Owner review — Q11-A approved under historical alias Q5B-A, 2026-08-27.** The
owner approved the complete twelve-clause Start-page contract without revision,
then made Q11 its canonical forward identity. Q5B remains a historical alias and
its packet marker above is deliberately preserved. This closes Q11 and resumes
the pre-existing sequence at Q6 without authorizing implementation or Q6-Q10.

<!-- EVIDENCE:GLOBAL-PREFERENCES-SPEC:GP-C03-Q5B-APPROVED -->
Historical alias of the Q11 approval above; retained for committed references.

### 3.3 Effect on the current Q5 premise

Neither Q5A nor Q11 changes Q5's premise. Q5 governs the invariant copy
semantics after a factory, user, or organization seed snapshot has been chosen:
one atomic copy into Project authority, one durable itemized receipt, and no
later live following. Q5A decides how a user profile may first be established;
Q11 decides whether and how the selected seed values are shown before creation.
The profile's origin or preview surface cannot change what genesis does with it.

## 4. GP-C03-Q6 — session, context, operation input, and state lifetimes

<!-- OWNER:GLOBAL-PREFERENCES-SPEC:GP-C03:GP-C03-Q6 -->

### 4.1 Exact on-screen decision

**Whenever Datum shows a session override, active context, or per-run choice, it
names that class and its lifetime on screen: a Session value says this run only,
Context is read-only and names what it applies to, and an operation still asks
for its input even when a remembered preference pre-fills the answer.**

Q6 decides which ephemeral resolution inputs exist, their precedence and exact
lifetimes, and the boundary separating them from restartable workspace state and
transient interaction state. It does not choose persistence technology (GP-C04),
settle individual clay descriptor classifications (GP-C05A), or decide refusal
and unavailable-provider behavior (Q7).

### 4.2 Reviewed visual evidence

The Claude-owned real Preferences window is the primary visual authority and was
reviewed without modification:

- `docs/gui/prototypes/preferences-window.html#workspace`, source lines 89–110,
  draws an active-Project adopted-standard constraint as visible context, labels
  Crosshair style as session-only today and still clay, and distinguishes
  dismissed-warning suppression state from the preference-facing restore action.
- `preferences-window.html#rules-checks`, source lines 165–168, draws a remembered
  Check profile pre-fill while saying the named `run_check` parameter is never
  hidden. `#output`, source lines 234–238, applies the same law to output-job and
  export-destination pre-fills, including **Ask each time**.
- `preferences-window.html#files-projects`, source lines 215–231, separates the
  persisted **Startup: Start page | Last session | Empty** preference from the
  machine-local completed/dismissed/never-run guided-setup state. The same row
  promises that replay changes no setting by itself.
- `preferences-window.html#project-policy-read-only`, source lines 268–272, draws
  the active Project policy and revision policy as read-only mirrors owned by
  the Project rather than machine-editable preference values.
- `docs/gui/prototypes/canvas-background-decision.html`, source lines 42–44 and
  88–119, is the ratified PM-036 presentation-class precedent. It proves that a
  machine presentation value may be remembered without changing Project or
  Publish truth; Q6 cannot use mere persistence or active context to erase that
  authority boundary.

The archival structural evidence is
`docs/gui/prototypes/preferences-ux-study.html#px-v8`, source lines 374–385.
PX-V8 renders **Reduced motion · SESSION · this run only**, **Active layer set ·
CONTEXT · applies while reviewing [one Project] · read-only contribution**, and
**Export destination · asked per run**, with the explicit rule that a
`WorkflowDefault` may pre-fill the operation but never remove the ask. Its
lifetime chip is archival support for the behavior already exposed by the real
window; it does not replace the window.

No visual reconciliation is required for Candidate Q6-A. Persisting a Session
override, making Context editable, or allowing a remembered pre-fill to remove
the operation's explicit ask would materially contradict both prototypes and
cannot become an owner candidate unless Claude first renders that different
behavior in the real window.

### 4.3 Written evidence

- Approved Q2 reserves `Session` as an explicit ephemeral override and `Context`
  as a read-only applicable fact, while keeping restartable workspace state,
  transient interaction state, and operation input outside writable Preferences
  (`GP_C03_TYPED_AUTHORITY_DECISION_PACKET.md:310-350,398-425`).
- Approved Q4 applies descriptor/source eligibility and active controls before
  ordinary value ranking, preserves displaced and refused facts, and explicitly
  leaves Session/Context precedence to Q6
  (`GP_C03_TYPED_AUTHORITY_DECISION_PACKET.md:570-615,639-646`).
- Datum's current in-memory GUI state includes crosshair mode, pane layout and
  focus, filters, dock geometry, terminal presentation, terminal sessions, and
  console history, with no current load/save path
  (`GP_C01_INTERNAL_AUTHORITY_AUDIT.md:133-157`). This is evidence for explicit
  classification, not authority to preserve every current accident.
- Route profile, import merge, check target/profile, artifact selection, and
  formatting mode are invocation-local typed choices today; remembering one
  later requires a separately governed default
  (`GP_C01_INTERNAL_AUTHORITY_AUDIT.md:273-280`).
- Configuration and restartable state have distinct authority, retention,
  exchange, privacy, and reset laws even if an implementation later shares a
  storage mechanism (`GP_C02_EXTERNAL_AND_STANDARDS_RESEARCH.md:195-212`). The
  bounded evidence separately requires preference, restartable state, transient
  state, Project policy, and operation input
  (`GP_C02_EXTERNAL_AND_STANDARDS_RESEARCH.md:423-433`).
- Approved Q3 makes personal accessibility descriptors the only
  organization-control carve-out class and requires a named descriptor
  justification (`GP_C03_TYPED_AUTHORITY_DECISION_PACKET.md:548-557`). Q6's
  reduced-motion example therefore cannot silently create or remove management
  authority.

### 4.4 Genuine alternatives and the single surviving boundary

#### Candidate Q6-A — typed ephemeral sources and separate state classes

`Session` is an explicit, descriptor-eligible override for the current Datum
application run. `Context` is a named, read-only applicability fact that exists
only while its subject is active. Operation input remains an explicit choice for
one invocation even when a registered `WorkflowDefault` supplies the initial
answer. Restartable workspace state may restore a working view across launches,
and transient interaction state dies with its interaction, but neither becomes a
preference contribution merely because it is retained in memory or on disk.

This is the only candidate drawn by both reviewed prototypes and consistent with
approved Q2/Q4. It is recommended.

#### Rejected alternative Q6-B — persist every useful value as a preference

Session overrides, active layer/filter context, pane geometry, last focus,
dismissed-warning state, and export choices all become persistent preference
contributions in one precedence lattice. This can make restoration mechanically
uniform, but it turns interaction history and Project applicability into durable
policy-like inputs and makes “this run only” untrue.

Q6-B is a genuine storage simplification, but approved Q2 clauses 6 and 9 already
exclude this flattening, GP-C02 requires distinct authority/retention law, and
both visual sources label the separation. No Q6-B owner boundary survives without
reopening ratified Q2 and receiving a materially different Claude render.

#### Rejected alternative Q6-C — no Session or Context resolution sources

Datum allows only persisted preference contributions; every temporary need is an
operation parameter and every Project-dependent fact is read outside preference
resolution. This minimizes resolver inputs, but it cannot represent the drawn
this-run accessibility override or the read-only active-Project applicability
shown in the real window and PX-V8.

Q6-C is a genuine narrower resolver, but it contradicts Q2's approved reserved
sources and the reviewed visual truth. It cannot open as a selectable owner
candidate until those authorities are deliberately reconciled. Candidate Q6-A
is therefore a single-candidate boundary with cited ratified law explaining why
no alternative survives.

### 4.5 Exact bounded contract Candidate Q6-A would establish

Approval establishes only these clauses:

1. `Session`, `Context`, operation input, restartable workspace state, and
   transient interaction state are five distinct typed classes. Persistence,
   screen proximity, and shared implementation storage cannot reclassify one as
   another.
2. `Session` is a user-initiated value contribution admitted only where the
   `PreferenceDescriptor` explicitly allows it. It applies to the current Datum
   application run, is never written as a User/Installation/Organization
   preference, and expires when that run ends.
3. An eligible Session contribution is considered after Q4 eligibility and
   active control evaluation and, for the current run, outranks ordinary User,
   Organization Recommendation, Installation, and Descriptor Default values. It
   cannot bypass a descriptor constraint, eligible Pin or Lock, Project law, or
   the personal-accessibility carve-out established by Q3.
4. A Session override is explicit and reversible during the same run. Its UI and
   effective-value query name the value, `SESSION` source, actor, start, **this
   run only** lifetime, displaced value, and the value that will resume at exit.
   Reading or using the override never serializes it as an explicit user choice.
5. PX-V8's reduced-motion value is a Session-override example, not a decision
   that reduced motion lacks a persistent or system-derived baseline. Descriptor
   classification and allowed persistent sources remain GP-C05A catalog work.
6. `Context` is a read-only, typed fact supplied by the active Project, document,
   tool, review mode, or other descriptor-declared subject. It names that subject,
   applicability reason, source authority, and lifetime; neither Preferences nor
   the user can edit it as a contribution.
7. Context may select which already-authoritative rule or value is applicable;
   it does not receive a universal rank and cannot create, widen, or bypass
   Project policy, organization authority, descriptor eligibility, or Q4
   controls. When the named context ceases to apply, the contribution disappears
   and resolution is recomputed without retaining it as preference state.
8. Operation input is outside preference resolution and is explicitly requested
   for one invocation. A registered `WorkflowDefault` may pre-fill the control,
   but the control, current value, and opportunity to change it remain present;
   the accepted operation input governs only that invocation.
9. Canceling or completing an operation does not silently promote its input into
   a preference. Any remembered **last used** behavior requires a separately
   registered `WorkflowDefault` and the normal typed preference mutation path;
   Q6 does not authorize that mutation for any catalog row.
10. Restartable workspace state may survive an application restart solely to
    restore the user's working view, including eligible layout, open-document,
    focus, or history facts. It is not a `PreferenceScope`, has no place in Q4's
    preference precedence, and cannot carry Project or organization authority.
11. Transient interaction state, including hover, cursor, drag preview, open
    menu, and live terminal-byte state, expires with its owning interaction or
    live session and is neither restartable state nor a preference contribution.
12. Every visible Session or Context value uses text in addition to color, has a
    programmatic source and lifetime, and exposes its reason without stealing
    focus. Keyboard-only users can inspect and, where user-editable, end a
    Session override through the ordinary control order.
13. Q6 does not choose storage locations/formats, serialization, recovery,
    migration, synchronization, provider-offline behavior, detailed catalog
    classification, implementation dependencies, or any Q7-Q10 disposition.

### 4.6 Recommendation and owner response

Approve **Q6-A**. It is the sole candidate consistent with the real Preferences
window, archival PX-V8, approved Q2/Q4 source law, Q3's accessibility boundary,
and GP-C02's configuration/state separation.

Reply exactly:

```text
GP-C03-Q6: approve Q6-A
```

or:

```text
GP-C03-Q6: revise — <required session, context, operation-input, state-lifetime, precedence, or accessibility correction>
```

<!-- EVIDENCE:GLOBAL-PREFERENCES-SPEC:GP-C03-Q6-PACKET -->

<!-- EVIDENCE:GLOBAL-PREFERENCES-SPEC:GP-C03-Q6-APPROVED -->
**Owner review — Q6-A approved 2026-08-28.** The owner approved all thirteen
bounded clauses exactly as packeted: five distinct typed classes; explicit
this-run Session contributions after eligibility and control evaluation; named,
read-only, applicability-bounded Context; explicit per-invocation operation input
despite any `WorkflowDefault` pre-fill; separate restartable workspace state and
transient interaction state; visible lifetime/provenance and keyboard-accessible
inspection; and no persistence, catalog, implementation, or Q7-Q10 disposition.
This closes Q6 and advances only to Q7 packet preparation.

## 5. GP-C03-Q7 — validation, refusal, protected sources, and unavailable providers

<!-- OWNER:GLOBAL-PREFERENCES-SPEC:GP-C03:GP-C03-Q7 -->

### 5.1 Exact on-screen decision

**When a value is invalid, a source is ineligible, or a managing provider is
unreachable, Datum keeps the last valid effective value, leaves correctable input
visible with the exact refusal law, and shows the provider generation as stale
but still effective only for its declared offline-validity interval—never as a
silent policy removal.**

Q7 decides validation/refusal behavior, protected-source enforcement, and the
effect of provider availability, expiry, and revocation on resolution. It does
not decide unknown/retired identities (Q8), the complete provenance query (Q9),
or package transport, persistence, signing, and recovery mechanics (GP-C04).

### 5.2 Reviewed visual evidence

The Claude-owned real Preferences window is the primary visual authority and was
reviewed without modification:

- `docs/gui/prototypes/preferences-window.html#terminal`, source lines 169–191,
  draws shielded Terminal launch, scrollback, paste, clipboard, and link-opening
  rows. It says untrusted Projects can never set these Capability/security values,
  while the scrollback row exposes the 100,000-line/64-MiB constraint that an
  invalid draft must not silently evade.
- `preferences-window.html#agents`, source lines 250–256, applies the same
  protected-source treatment to agent authority and unattended-tool grants;
  Project specificity cannot become agent capability authority.
- `preferences-window.html#organization`, source lines 257–265, keeps excess
  organization requests visible but inert and draws one read-only managed-policy
  status covering provider, package, generation, managed settings, contact, and
  the distinct **effective / stale / expired / revoked** states. Reset/import are
  separately protected operations, not provider-failure fallbacks.
- `docs/gui/prototypes/canvas-background-decision.html`, source lines 42–44 and
  88–119, remains the PM-036 presentation-class precedent: resolver failure or
  provider loss cannot promote a machine presentation fallback into Project or
  Publish truth, or conversely grant a Project source Capability authority.

The archival structural evidence is
`docs/gui/prototypes/preferences-ux-study.html#px-v9`, source lines 387–398.
PX-V9 renders three exact failure states: **250,000 refused — above constraint
100,000**, with the draft kept for correction; **Project value ineligible**, with
the Capability class law named; and **provider unreachable 2h**, with last-known
generation 14 still in force and visibly stale. Its summary law is “no refusal
without its reason; no degradation without its banner.”

No visual reconciliation is required for Candidate Q7-A. Silently clamping the
invalid draft, accepting the Project executable, treating disconnect as policy
withdrawal, or hiding stale/expired/revoked status would materially contradict
the reviewed surfaces and cannot open an owner boundary until Claude renders
that different behavior in the real window.

### 5.3 Written evidence

- Approved Q1 makes validation constraints and canonical form descriptor-owned,
  refuses duplicate/incompatible registration, and forbids providers from
  redefining a key's meaning (`GP_C03_TYPED_AUTHORITY_DECISION_PACKET.md:214-241`).
- Approved Q2 requires every descriptor to declare allowed sources and refuses
  ineligible contributions rather than silently considering them; Capability
  settings explicitly require protected-source eligibility
  (`GP_C03_TYPED_AUTHORITY_DECISION_PACKET.md:310-350,395-425`).
- Approved Q4 evaluates descriptor/source eligibility before controls and value
  ranking, forbids last-writer-wins, and keeps every losing, inert, refused, and
  conflicting contribution queryable with a reason
  (`GP_C03_TYPED_AUTHORITY_DECISION_PACKET.md:596-615,639-646`).
- Primary external evidence requires provider trust/availability and validation
  state to participate in resolution, and requires inspection to expose invalid
  and unavailable contributions rather than reconstructing them in the UI
  (`GP_C02_EXTERNAL_AND_STANDARDS_RESEARCH.md:127-168`). Protected keys cannot
  accept Project-local executable, agent, provider-endpoint, or trust-root values
  (`GP_C02_EXTERNAL_AND_STANDARDS_RESEARCH.md:170-181`).
- Provider absence must never fabricate policy removal, while complete local and
  offline resolution uses cached/installed authorities
  (`GP_C02_EXTERNAL_AND_STANDARDS_RESEARCH.md:438-446`). The exact duration and
  representation of last-known managed authority was deliberately left to Q7
  (`GP_C02_EXTERNAL_AND_STANDARDS_RESEARCH.md:448-458`).
- Domain-peer evidence rejects one inferred disconnect rule: each managed package
  declares offline continuation/expiry, and Datum shows authenticated local
  generations as effective, stale, expired, revoked, or unavailable
  (`GP_C02B_DOMAIN_PEER_PREFERENCES_RESEARCH.md:334-344,360-373`). Unrelated
  Design authoring remains non-blocking while provider status is visible
  (`GP_C02B_DOMAIN_PEER_PREFERENCES_RESEARCH.md:390-405`).
- Accessibility evidence requires refusal/state to use more than color, announces
  status without stealing focus, and keeps managed values inspectable even when
  unwritable (`GP_C02_EXTERNAL_AND_STANDARDS_RESEARCH.md:374-396`).

### 5.4 Genuine alternatives and the single surviving boundary

#### Candidate Q7-A — fail honestly, retain last valid truth, honor declared offline validity

Datum validates before mutation or resolution. Invalid drafts remain in their
controls for correction while the last valid effective value continues; an
ineligible source produces a typed refusal before precedence. A disconnected
provider does not vanish: its last authenticated generation remains visibly
stale and effective only through the offline-validity law already carried by that
generation. Explicit expiry or revocation ends its participation and triggers Q4
re-resolution from retained eligible sources, while the inactive generation and
reason remain inspectable.

This is the only candidate consistent with the real window, archival PX-V9, and
approved Q1–Q4. It is recommended.

#### Rejected alternative Q7-B — immediate fail-closed on every disconnect

Any provider connection loss immediately blocks managed settings or all Datum
work until the provider returns. This can maximize central freshness, but it
conflates temporary unreachability with revocation, discards the package's
declared offline law, and makes unrelated local Design depend on a network.

Q7-B is a genuine centrally controlled posture, but it contradicts the rendered
stale-generation-in-force state, the real window's distinct stale/expired/revoked
states, Datum's local-complete evidence, and PM-034 non-blocking authoring. It
cannot survive without a materially different Claude render and new owner law.

#### Rejected alternative Q7-C — silent fallback and forgiving coercion

Invalid values are clamped or replaced automatically, ineligible source values
are ignored without a refusal record, and provider loss silently removes managed
authority so lower-ranked values take effect. This maximizes apparent continuity
and minimizes warnings, but the user's entered value, attempted authority, and
reason for the effective-value change become unknowable.

Q7-C is a genuine convenience model, but it directly violates PX-V9, the real
managed-status row, approved Q2/Q4 refusal/provenance law, and the external rule
that provider absence cannot fabricate policy removal. Candidate Q7-A is
therefore a single-candidate boundary with cited ratified law explaining why no
alternative survives.

### 5.5 Exact bounded contract Candidate Q7-A would establish

Approval establishes only these clauses:

1. Every candidate contribution and control directive is validated against the
   authoritative `PreferenceDescriptor` identity, type/canonical form,
   constraints, allowed source families, and setting class before storage or
   resolution. Active Q3 controls are then evaluated before Q4 ranking; only a
   control that explicitly refuses mutation from that source blocks its storage.
2. Descriptor-declared canonicalization may normalize equivalent representation
   without changing meaning. A value outside the declared type, enum, range,
   structure, constraint, or security law is refused; Datum never silently clamps,
   truncates, substitutes, or serializes a different meaningful value.
3. A refused interactive draft remains visible and editable in its field with
   the attempted value, the violated descriptor/constraint law, and a correction
   path. It is not a stored contribution and does not replace the last valid
   effective value merely because the user pressed Apply.
4. Non-interactive invalid input is returned as a typed refusal to its caller.
   Any diagnostic retention or redaction follows descriptor security law and
   cannot overwrite the last-known-good preference generation; exact durable
   recovery representation remains GP-C04.
5. Source eligibility is evaluated before precedence. A contribution from an
   ineligible source is refused with the key, source family/identity, setting
   class, and descriptor law that refused it; specificity, recency, provider
   order, and Context cannot make it eligible.
6. Capability and protected security descriptors reject Project sources for
   executable/argv/environment choice, agent authority and unattended tools,
   managed-provider endpoints, trust roots, clipboard/paste/link gates, and every
   other source family the descriptor excludes. The attempt cannot mutate either
   Preferences or governed Project facts.
7. Refused and ineligible contributions never participate in Q4 value ranking,
   controls, or merge. They remain queryable as inactive facts with their reason,
   subject to descriptor-declared secret redaction; Q9 defines the complete query
   shape and UI projection.
8. A managed package/generation declares its authenticated identity, effective
   interval, and explicit offline-validity law before provider loss. Q7 selects
   no universal timeout and connectivity alone cannot invent or extend one.
9. When a provider becomes unreachable, the last locally validated generation
   remains effective only while its declared offline-validity law permits. Datum
   immediately marks it **stale/unreachable**, retains provider/package/generation,
   last successful contact, elapsed staleness, affected keys, reason, and contact,
   and does not describe the provider as revoked or removed.
10. When no validated local generation exists, the provider is **unavailable**
    and contributes no value or control. Resolution uses retained eligible sources
    under Q4 and visibly states that no managed generation is active; it does not
    synthesize an organization default, constraint, Pin, or Lock.
11. When declared offline validity ends, the generation becomes **expired** and
    ceases to participate. Once an authenticated revocation event is locally
    available, the generation becomes **revoked** and ceases participation
    regardless of later connectivity. Both events trigger Q4 re-resolution from
    retained eligible sources and remain inspectable; neither deletes displaced
    user values or rewrites Project policy.
12. Provider restoration validates the received generation before it can replace
    stale/expired state. Arrival order or reconnection alone grants no authority;
    package transport, signing, expected-generation mutation, persistence, and
    rollback mechanics remain GP-C04.
13. Validation/refusal and provider-state presentation uses text plus a non-color
    status cue, exposes programmatic name/state/reason, remains keyboard
    inspectable, and announces changes without focus theft. A refused field keeps
    its user's editing context.
14. Invalid preference input or provider degradation cannot block unrelated
    Design authoring. Governed Project policy continues under its own authority;
    Q7 neither weakens Project gates nor turns organization connectivity into
    engineering or Release authority.
15. Q7 does not decide unknown/retired/aliased identities, migration, storage
    format, cryptographic package design, recovery generations, synchronization,
    complete query schema, implementation dependencies, or any Q8-Q10 disposition.

### 5.6 Recommendation and owner response

Approve **Q7-A**. It is the sole candidate consistent with the protected rows and
managed-status states in the real Preferences window, archival PX-V9, approved
Q1–Q4, explicit offline policy, local completeness, and accessible failure
honesty.

Reply exactly:

```text
GP-C03-Q7: approve Q7-A
```

or:

```text
GP-C03-Q7: revise — <required validation, refusal, protected-source, offline-validity, expiry/revocation, fallback, or accessibility correction>
```

<!-- EVIDENCE:GLOBAL-PREFERENCES-SPEC:GP-C03-Q7-PACKET -->
