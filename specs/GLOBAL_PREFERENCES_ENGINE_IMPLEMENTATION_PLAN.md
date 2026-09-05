# Datum Global Preferences Engine Implementation and Production-Acceptance Plan

> **Status:** Governed serial implementation contract. Execution authority is
> determined only by the synchronized Active Frontier; GP-CM01 is currently the
> bounded authorized step.
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
2. the Global Preferences foundation plus a production-grade first real wgpu
   vertical slice;
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

## 4. Global Preferences foundation and production-grade first GUI slice

`dat-global-preferences-engine-build-vge` depends on UNIT-I01, not on Units
surface parity. Its first product result is one functional manual GUI slice,
not an engine/CLI/MCP-only system. GP-F05 must be shippable for its exact row
inventory and application doorway; “vertical slice” limits breadth, never
quality, durability, accessibility, or honesty. It does not claim that the
remaining active catalog or the complete Global Preferences product is
production-accepted.

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

GP-F01 implemented this historical boundary in `crates/engine/src/preferences/`.
At that stage the pure engine module registered the 54 then-classified active
non-Revision V1 identities, refused
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

The corrective implementation declared all 54 then-classified active descriptors literally,
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
The corrected foundation landed in `12ff4c4`; GP-F02 is therefore a fresh owner
review boundary, not a continuation of the superseded initial review.

<!-- EVIDENCE:GLOBAL-PREFERENCES-ENGINE:GP-F02-OWNER-REVISION -->

<!-- EVIDENCE:GLOBAL-PREFERENCES-ENGINE:GP-F01-TYPED-FOUNDATION -->

<!-- REQ:GLOBAL-PREFERENCES-ENGINE:GP-F02 -->
<!-- OWNER:GLOBAL-PREFERENCES-ENGINE:GP-F02:GP-F02 -->
### GP-F02 — authorize minimal durable repository only

Review the corrected GP-F01 evidence before authorizing GP-F03. The owner's
2026-08-31 revision disposition returned GP-F01 to execution and did not
authorize storage.

On 2026-08-31 the owner replied exactly
`PREFERENCES-FOUNDATION: approve GP-F03` after reviewing the corrected GP-F01
evidence. This authorizes GP-F03 only. It does not authorize GP-F05, GUI or
prototype work, Project mutation, Revision behavior, provider/network services,
or a new dependency.

<!-- EVIDENCE:GLOBAL-PREFERENCES-ENGINE:GP-F02-OWNER-APPROVED -->

<!-- REQ:GLOBAL-PREFERENCES-ENGINE:GP-F03 -->
### GP-F03 — durable repository foundation

Implement expected-generation single-writer persistence, immutable generations,
exact unknown preservation, typed migration, backup, reversible restore, and
preserved-unreadable recovery sufficient for real visible rows. No provider or
network service is included.

GP-F03 landed in `fab5100`. The engine-owned `PreferenceRepository` uses one
exclusive local writer lease, exact expected-head comparison, complete immutable
canonical-JSON generations, and atomic head promotion. Installation and User
partitions remain distinct; defaults are never serialized merely by reading
them. Registered descriptors validate every ordinary write. Opaque unknown
envelopes retain exact bytes in content-addressed payloads through successor
generations, migration, backup, restore, and recovery.

Repository inspection and migration/restore planning are side-effect free.
Schema and alias migration requires a registered deterministic transform,
refuses collisions and unchosen substitutions, and creates a complete
pre-migration backup. Exact restore is previewed against an expected head,
backs up the displaced current generation first, preserves accumulated audit
receipts, and can itself be reversed. Missing, malformed, truncated, or
integrity-invalid head/generation/payload state is never repaired or reset in
place: exact suspect bytes and complete valid predecessor identities are exposed
for an explicit recovery operation.

Eight focused repository tests and all 970 engine library tests pass. Strict
all-target/all-feature workspace Clippy passes with warnings denied, as do
source-health, rustfmt, dependency-authority, Cargo-resource, evidence,
governance, parity, and Frontier gates. Full production fault-injection and
real-surface acceptance remain assigned to the later Global Preferences
completion boundary. GP-F03 adds no GUI/prototype work, Project mutation,
Revision behavior, provider/network service, synchronization, or dependency.

<!-- EVIDENCE:GLOBAL-PREFERENCES-ENGINE:GP-F03-DURABLE-REPOSITORY -->

<!-- REQ:GLOBAL-PREFERENCES-ENGINE:GP-F04 -->
<!-- OWNER:GLOBAL-PREFERENCES-ENGINE:GP-F04:GP-F04 -->
### GP-F04 — authorize first real wgpu vertical slice only

Review the resolver/repository evidence and then-current Claude target before
authorizing GP-F05.

On 2026-08-31 the owner rejected the underspecified GP-F05 boundary and replied
exactly:

> PREFERENCES-FOUNDATION: revise — before GP-F05 implementation, define the
> exact active descriptor inventory for the first functional slice and add
> typed presentation metadata for section placement, stable ordering, and
> control selection without key-prefix inference or GUI hard-coding; reconcile
> the protected visual target so deferred Revision, planned, Project-policy,
> Organization, setup, and unavailable management surfaces are excluded;
> specify the nested Edit > Preferences > Global Preferences doorway, the
> repository/resolver GUI lifecycle, and the keyboard, focus, accessibility,
> persistence, recovery, and zero-Project-mutation proof required for
> acceptance.

That disposition authorizes this contract correction only. GP-F04 remains the
owner boundary: GP-F05 implementation is not authorized until the corrected
contract and Claude-owned protected target are returned for a fresh exact
approval.

<!-- EVIDENCE:GLOBAL-PREFERENCES-ENGINE:GP-F04-OWNER-REVISION -->

On 2026-09-01, after the corrected contract and Claude-owned protected target
at `62e263b` were reviewed together, the owner replied exactly
`PREFERENCES-FOUNDATION: approve GP-F05`. This authorizes GP-F05 only, under the
complete GP-F05.1 through GP-F05.7 inventory, lifecycle, accessibility,
recovery, proof, exclusion, and no-new-dependency boundary below. It does not
authorize UNIT-I03, GP-CM01 or later completion work, Project Preferences,
Project mutation, Revision behavior, a provider/service, synchronization, or a
new dependency.

<!-- EVIDENCE:GLOBAL-PREFERENCES-ENGINE:GP-F04-OWNER-APPROVED -->

<!-- REQ:GLOBAL-PREFERENCES-ENGINE:GP-F05 -->
### GP-F05 — production-grade Global Preferences doorway and wgpu slice

GP-F05 implements the conventional application-level Global Preferences
doorway and one complete, shippable Appearance slice. The slice uses only the
real descriptor, resolver, repository, and consumer paths. It must not render a
fixture catalog, infer presentation from a key prefix, or leave a visible
control disconnected from its declared consumer.

#### GP-F05.1 — exact visible inventory

The GP-F05 surface contains exactly one section, stable section identity
`appearance`, labeled **Appearance**, and exactly these three active V1 rows in
this order:

| Order | Stable key | Control | Required live result |
|---:|---|---|---|
| 10 | `datum.console.feedback_duration` | single-choice enum: 4 s, 6 s, 10 s, Never hide | The Console feedback timer immediately uses the resolved duration. The legacy `console_duration` value is migrated once through the registered alias path; the old GUI JSON writer is then removed as a rival persistent authority. |
| 20 | `datum.accessibility.reduced_motion` | boolean switch with explicit Off/On text | Non-essential GUI and terminal animation immediately follows the resolved value; state, focus, and non-color cues remain unchanged. |
| 30 | `datum.accessibility.high_contrast_noncolor` | boolean switch with explicit Off/On text | The application and terminal renderers immediately use the resolved high-contrast/non-color presentation; no state is communicated by color alone in either mode. |

No other descriptor, section, category, teaching row, placeholder, or read-only
policy projection is visible or searchable in GP-F05. In particular, the
surface contains no Revision, Units, Project Policy, Project Preferences,
Organization, Manage preferences, setup/replay, Start-page, planned, clay,
deferred, or empty category. Later work expands the same architecture; it does
not reserve empty navigation now.

Every row shows its registered accessible label and description, real control,
resolved effective value, explicit **Global · this device** scope, concise
provenance, and a setting-name action that opens the complete resolver-owned
explanation. Search covers the three current labels, descriptions, stable keys,
and the registered `console_duration` alias. Search results invoke the same
control and mutation path as the canonical row and cannot form a shadow store.

#### GP-F05.2 — typed presentation authority

The engine owns a typed `PreferenceSurfaceCatalog`; the GUI owns layout and
rendering only. Each exposed entry binds an existing `PreferenceKey` to:

- a stable `PreferenceSectionId`, localized section label, and unique section
  order;
- a unique row order within that section;
- a closed `PreferenceControlPresentation` variant compatible with the
  descriptor's `ValueSchema` (the GP-F05 variants are boolean switch and
  enumerated single choice);
- stable enum values mapped to visible labels without changing stored values;
  and
- the descriptor-owned accessible label, description, aliases, and provenance
  query identity.

Registry construction refuses a missing/duplicate key, duplicate order,
unknown section, schema/control mismatch, incomplete or duplicate enum mapping,
empty accessible copy, or a presentation entry for an inactive descriptor.
The GUI iterates this typed catalog. It may not switch on key strings, infer a
section from dot-separated names, duplicate defaults or enum choices, or carry
a second list of rows. A catalog-wide test proves every GP-F05 key resolves to
exactly one active descriptor and every visible value round-trips through that
descriptor's validator.

#### GP-F05.3 — application-menu and window contract

The menubar represents an actual nested submenu, not a flat label containing a
path and not a `not_built` refusal:

```text
Edit
└── Preferences
    └── Global Preferences…
```

Pointer click, keyboard menu traversal, and activation dispatch one stable
GUI-local command identity to the same open action. The submenu has correct
hover, focus, Escape/Left closure, Right/Enter opening, and edge-safe placement.
`Project Preferences…` is not advertised by this slice because its surface is
not implemented; PP-I01 later adds the second ratified PM-039 command and its
disabled-without-Project behavior. This staged visibility does not alter the
ratified two-command family.

The command opens one dedicated application-level native window titled
**Global Preferences — Datum**, owned by the main Datum window. The native
window frame fits directly around the Preferences interface; Datum must not
draw the interface as a centered card inside the Design workspace, retain a
full-workspace modal backdrop, or place an otherwise empty canvas outside the
Preferences boundary. It is not a document, tiled pane, Navigator item,
Inspector mode, or terminal surface. The window is input-modal to its owning
Datum window while open, resizable from a content-fitting default size, and
unique: repeated activation raises the existing instance rather than
duplicating it. Opening performs no write. Successful row changes commit
immediately; there is no generic Apply transaction and closing does not roll
back committed values. The interface states that changes save immediately and
does not draw Apply, OK, Cancel, or an in-content dialog Close action. Native
title-bar close and Escape return focus to the menu invoker (or the previously
focused editor if the menu invoker no longer exists).

This is the first consumer of Datum's shared application-owned settings-window
policy, not a Global Preferences exception. Global Preferences, future Project
Preferences, and every later input-modal settings window use the same class:
activating an owning Datum window raises and focuses its existing owned window;
the owner cannot remain stacked above it; switching to an unrelated application
does not raise it; and the owned window is never globally always-on-top. Backends
that expose native transient-parent metadata must use that compositor/window-
manager relationship; focus or attention requests alone are not acceptance
proof. A backend without native metadata may use application focus authority
only if direct platform proof demonstrates the same stacking contract.

At ordinary width the dialog follows the ratified two-column grammar: section
navigation on the left and pinned content-column search over the rows on the
right. At narrow width the section list becomes a labeled chooser, search stays
visible, and an open explanation stacks immediately below its row. No content
may clip, overlap, draw outside the dialog, or become reachable only by pointer.
Runtime chrome implements the protected target's component grammar directly:
the `.f5tbar` header and scope chips; neutral `.f5nav` column with one structural
divider; `.f5nav .on` row-only selection fill and two-pixel accent strip;
`.f5focus` information-blue keyboard outline on the actual focused control;
padded `.f5srch` band; divider-based `.f5row` and `.f5prov` sequence; compact
rounded `.f5sel`; and pill-and-knob `.f5tog` with a separate ON/OFF word cue.
Tinting or bounding the entire navigation column, replacing row dividers with
detached cards, or substituting generic text boxes for switches is a parity
failure even when the same settings remain reachable.
The focused search field renders a visible text caret at the insertion point;
the accent focus boundary alone is not an editable-text cursor. In ordinary,
focused, empty, populated, and narrow states, the search fill and its focus
boundary use the protected target's six-pixel rounded corners; a square runtime
field is a visual-parity failure.

#### GP-F05.4 — repository, resolver, and consumer lifecycle

The application creates one preferences service from the platform-resolved
Datum configuration root, `active_v1_registry()`, the GP-F05 surface catalog,
and `PreferenceRepository`. GUI code never reads or writes preference files
directly.

1. On clean start, absence of a repository resolves compiled descriptor
   defaults without creating a file or serializing defaults.
2. Before the first ordinary load, the service inspects the legacy Console-only
   file. A valid `console_duration` contribution is previewed and migrated
   through the registered alias into a new immutable repository generation;
   legacy unknown bytes and an invalid/unreadable legacy file are preserved and
   reported, never silently discarded. The legacy file is retained as migration
   evidence until the new head is verified and backed up.
3. Opening the dialog reads one repository snapshot and resolves all three rows
   through the existing resolver. Render frames and input hit testing perform
   no filesystem access and never re-run migration.
4. Edit and Reset submit typed User-source mutations through one application
   coordinator with the displayed expected generation. Reset removes the User
   contribution and re-resolves; it never writes a guessed default.
5. A successful commit publishes the new snapshot, re-resolves all affected
   rows, applies the declared live consumers, updates provenance, and announces
   the result. For these three descriptors, consumer adapters must be total
   after descriptor validation; a visible row cannot report success while its
   live consumer retains an older value.
6. A stale expected generation, writer lease conflict, invalid value, or
   ineligible source is not retried or overwritten silently. The service
   preserves the user's draft, refreshes repository truth, leaves the last
   valid effective value active, and returns a typed refusal with the available
   retry/reset action.
7. Missing, malformed, truncated, or integrity-invalid repository state opens
   a non-blocking preserved-unreadable status. The three rows remain
   inspectable but not writable; the exact suspect data and valid predecessor
   identities remain preserved. GP-F05 provides no improvised repair, import,
   backup, restore, migration-management, or deletion UI. Design authoring and
   Project opening continue normally.
8. Restart must reproduce the same effective User values and provenance from
   the durable head. No row state is sourced from GUI-local cache, Project
   data, the Navigator, or the retired Console preference writer.

#### GP-F05.5 — keyboard and accessibility contract

The dialog exposes a programmatic dialog name and explicit Global scope. Tab
and Shift+Tab traverse, in visual order, section navigation, pinned search,
each setting-name explanation action, each real control, row Reset when
available, and explanation Close, with exactly one visible focus indicator and
no trap. Enter/Space operate actions and switches; enum choices support arrows
plus Home/End; Escape closes the innermost open choice or explanation before it
acts on the window. When the search field owns focus and contains a query,
Escape first clears the query, leaves focus and its caret in search, refreshes
the full result set, and announces the clear; a subsequent Escape closes the
window. An empty focused search closes on the first Escape.

Closing the native window discards its transient view state: search text and
filtering, open choice, open explanation, transient notice, and internal focus.
Reopening always returns to the ordinary Appearance state with the full three-
row result set and section navigation focused. Durable preference values and
their changed/reset provenance remain intact; window-view state is never stored
as a preference.

Every control exposes programmatic name, role, current value, availability,
scope, and concise provenance. Explanation content exposes the resolver's
effective contribution, absences, reason, descriptor facts, and available
actions in reading order. Search result count, successful value change, Reset,
and explanation opening use polite non-focus-stealing announcements; refusal
and repository-unreadable state are assertive once. Focus moves only after a
user action. Reduced-motion mode removes transitions and smooth scrolling but
no cue or information. Wide, narrow, default, changed, refused, and unreadable
states remain understandable with color removed.

#### GP-F05.6 — acceptance evidence

GP-F05 closes only when one reproducible proof packet contains all of the
following:

- unit/golden tests for the typed surface catalog, exact three-row inventory,
  ordering, schema/control compatibility, enum mapping, alias search, and
  rejection of GUI hard-coding or key-prefix inference;
- menu-model, pointer, and keyboard tests for the real nested submenu, single
  native Preferences-window instance, content-fitting default frame, absence
  of a workspace backdrop, Escape hierarchy, and focus restoration;
- focused service tests for clean start, first write, successful legacy Console
  migration, invalid/unreadable legacy preservation, restart persistence,
  Reset, stale generation, writer conflict, invalid draft, and corrupt-head
  preserved-unreadable behavior;
- consumer tests proving each of the three effective values changes its live
  declared behavior and that the retired Console writer cannot mutate
  persistent truth;
- semantic assertions that every visible provenance and explanation field is
  taken from the resolver result and that search controls use the canonical row
  mutation path;
- accessibility-tree assertions for dialog/section/search/name/control/value/
  state/provenance, keyboard-only completion, announcement priority, no focus
  trap, reduced-motion equivalence, and non-color state cues;
- Claude-owned ordinary and narrow running-target renders showing the exact
  three rows, an open explanation, changed provenance, and the preserved-
  unreadable banner, followed by running-app captures of the same states; and
- before/after hashes of every open Project shard and journal plus mutation-
  observer assertions proving that opening, searching, explaining, editing,
  resetting, refusing, closing, restarting, and corrupt-store handling perform
  zero Project/design mutation and add no Navigator settings presence.

Focused Rust tests, strict Clippy, source health, dependency authority, Cargo
resource policy, evidence traceability, spec governance/parity, menu-model,
project-state/render, private-writer, daemon-write-parity, and resolver-raw-load
gates must all pass. The implementation adds no third-party dependency.

#### GP-F05.7 — protected visual reconciliation required before authorization

Codex does not edit `docs/gui/prototypes/*.html`. Before GP-F04 may be approved,
the Claude-owned lane must reconcile
`docs/gui/prototypes/preferences-window.html` at the status banner, navigation,
`#appearance`, search-results, explanation, narrow-layout, and accessibility
regions so the protected GP-F05 target shows exactly the section and three rows
in GP-F05.1. It must also show the nested application-menu doorway and the
preserved-unreadable non-management state. It must preserve the ratified
two-column/pinned-search/row/explanation grammar and retain the full catalog as
future comparative evidence without letting it authorize GP-F05 content.

Expected proof is one explicitly labeled GP-F05 ordinary render, one narrow
render, a keyboard-focus map with return target, a screen-reader name/state
inventory, and a text/non-color unreadable-state render. No Revision, planned,
Project-policy, Project Preferences, Organization, setup, Manage preferences,
fixture, or empty-category content may appear in those GP-F05 states.

<!-- EVIDENCE:GLOBAL-PREFERENCES-ENGINE:GP-F05-REVISED-CONTRACT -->

On 2026-09-01 the owner inspected the running GP-F05 build and rejected the
in-shell modal backdrop: the native GUI window boundary must sit directly
against the Preferences interface. The owner directed Datum to reopen GP-F05
and correct the defect before UNIT-I02. This correction preserves the approved
three-row inventory, two-column grammar, input modality, engine ownership,
repository/resolver lifecycle, accessibility, recovery, and zero-Project-
mutation boundaries; it changes only native window hosting and its proof.

<!-- EVIDENCE:GLOBAL-PREFERENCES-ENGINE:GP-F05-NATIVE-WINDOW-CORRECTION -->

Owner QA then found that the main Datum window could be raised above the
input-modal Preferences window. The correction is incomplete until owner
activation redirects to the existing Preferences instance without imposing a
global always-on-top level. The reusable owned-settings-window policy established
here is also the default for future Project Preferences and comparable Datum
settings windows.

<!-- EVIDENCE:GLOBAL-PREFERENCES-ENGINE:GP-F05-OWNED-WINDOW-STACKING -->

A fresh Wayland/KWin owner QA run disproved the focus-request correction: the
main Datum window still covered Preferences. GP-F05 therefore requires the
native `xdg_toplevel` parent relationship on Wayland and direct compositor proof
that the child remains above its owner while unrelated applications remain
unaffected. The same owner review found that the in-content Close action became
redundant once Preferences gained a native title bar. Datum removes that action,
keeps immediate durable saving, adds a visible and programmatic “Changes save
immediately” statement, and does not add Apply, OK, or Cancel. This immediate-
save posture is the default for future Datum Preferences windows unless a later
owner decision deliberately introduces staged transactions.

<!-- EVIDENCE:GLOBAL-PREFERENCES-ENGINE:GP-F05-WAYLAND-NATIVE-OWNER-AND-IMMEDIATE-SAVE -->

Owner QA then found that the focused search showed only an accent boundary, not
an editable-text caret, and that Escape closed the entire native window while a
nonempty query remained. GP-F05 remains open until the runtime and protected
target both show a visible insertion caret and the standard two-stage Escape
sequence: clear focused search first, close the window second.

<!-- EVIDENCE:GLOBAL-PREFERENCES-ENGINE:GP-F05-SEARCH-CARET-AND-ESCAPE-UNWIND -->

The same owner QA comparison found that the protected search field's rounded
outline had been flattened into square runtime corners. GP-F05 therefore uses
the shared medium six-pixel radius for both the search fill and its boundary,
with geometry proof that the rendered shape excludes all four square corner
points while reaching each edge midpoint.

<!-- EVIDENCE:GLOBAL-PREFERENCES-ENGINE:GP-F05-SEARCH-CORNER-PARITY -->

Owner QA also found that closing and reopening Preferences retained the prior
search query and filtered result posture. GP-F05 treats all search, disclosure,
notice, and internal-focus state as window-local and resets it on close, while
leaving immediately saved preference values unchanged.

<!-- EVIDENCE:GLOBAL-PREFERENCES-ENGINE:GP-F05-REOPEN-ORDINARY-STATE -->

Fresh owner comparison against the protected HTML then found that the runtime
had reproduced the information hierarchy but not its component construction:
the whole navigation rail was tinted and focus-bounded, its structural divider
was absent, and generic card/button primitives replaced the target's row,
selector, and switch grammar. GP-F05 remains open until the runtime translates
the named `.f5*` rules above directly and a running-app capture confirms row-only
navigation selection and focus.

<!-- EVIDENCE:GLOBAL-PREFERENCES-ENGINE:GP-F05-HTML-COMPONENT-PARITY -->

## 5. Shared Units surface parity through real Preferences

<!-- REQ:SHARED-UNITS-SURFACE-PARITY:UNIT-I02 -->
<!-- OWNER:SHARED-UNITS-SURFACE-PARITY:UNIT-I02:UNIT-I02 -->
### UNIT-I02 — authorize surface parity only

After GP-F05 exists, review exact-core and real-surface evidence before
authorizing the first execution slice.

The owner instead revised this boundary on 2026-09-02: preserve the twelve
clauses and seven proof requirements in
`GP_SHARED_UNITS_ENGINE_REQUIREMENT.md`, but do not authorize execution until the
requirement, catalog, protected target, this plan, and Frontier describe one
exact system without gaps or conflicting ownership.

<!-- EVIDENCE:SHARED-UNITS-SURFACE-PARITY:UNIT-I02-OWNER-REVISION -->

<!-- REQ:SHARED-UNITS-SURFACE-PARITY:UNIT-I02R -->
### UNIT-I02R — reconcile the implementation and visual contract

The controlling requirement's revised reconciliation contract defines the typed
eight-descriptor profile, quantity-specific precision, lossless editing,
decimal-degree V1 input/display, exact scientific-notation grammar, future
expression seam, Global-default GUI, writable Project Working Units GUI,
separate Publish/document units, compatible adapters, deterministic legacy and
pre-feature-Project migration, rival-conversion removal, and clause-by-clause
proof. UNIT-I02R is documentation and protected-target reconciliation only.

The Claude-owned visual targets must be revised before this step completes. The
bounded brief is:

1. In `preferences-window.html`, preserve the native GP-F05 window and title the
   Global Units pane **Defaults for new Projects**. Render exactly eight active
   rows/selectors: Measurement system; Board Unit and Precision; Drill Unit and
   Precision; Schematic Unit and Precision; Angle precision. Pair each length
   quantity's controls visually without merging their stable identities, focus,
   provenance, or Reset. Every row states `Global · this device`, saves
   immediately, copies only at New Project, and leaves open/existing Projects
   unaffected. Do not show Project editing, Publish/document units, Revision,
   Organization, setup, DMS, radians, retired keys, or empty categories.
2. Attach the exact option inventories and defaults from the catalog to their
   controls. Every Unit selector shows its actual Follow-system resolution.
   Every Precision selector shows Automatic with its actual quantity/unit
   resolution, decimal places 0 through 6, and Exact nanometres. Cross-system
   overrides remain visible in row, provenance, resolver, non-color, and
   accessibility states. Angle exposes decimal-degree precision only.
3. In `project-preferences-category-study.html`, add a clearly separated
   Units-only owner-review target while retaining all non-Units content as clay.
   The terminal menu command is `Edit > Preferences > Project Preferences…`,
   disabled without an open Project; it opens the window directly, with no Units
   submenu. Show available and unavailable doorway examples separately—never two
   duplicate Project Preferences commands in one menu. Draw one Units category, scope
   `Project · <project name>`, the same eight values under Project Working Units
   authority, read-only seed/migration receipt provenance, immediate journaled
   and undoable commits, stale/refusal behavior, Reset-to-recorded-seed/migration
   behavior that never re-reads Global, and no Navigator entry. No
   Global, Publish/document, Revision, planned, or empty category is editable.
4. Both targets preserve the owned-window, close/reopen, Escape/search/caret,
   explanation, immediate-save, responsive, and non-color grammar. Provide
   ordinary, changed/reset, search, cross-system, refusal, unreadable, narrow,
   greyscale, keyboard, and screen-reader states. The Project target makes the
   lossless edit rule explicit: rounded view text is replaced on focus by an
   exact suffixed value; focus/blur and unchanged commit are no-ops; Escape
   cancels; actual edits parse and compare canonical values once.
5. Prove from HTML structure and renders that only the selected **Units** row is
   highlighted, category/content separation uses accepted rules, all controls
   are reachable/named, no excluded category is implied, and each file remains
   within its source-health limit.
6. Specify focus order as category rail, search, then each visible setting-name
   action and control in stable row order, with Unit before Precision inside each
   pair, each conditional Reset immediately after its control, and explanation
   Close only while open. Tab wraps in the modal window; GP-F05 Escape,
   close/reopen, return-focus, caret, and search-clearing behavior remains exact.
7. For each window and every Units selector, provide accessible name, role,
   stored and resolved value, provenance, scope, immediate-save consequence, and
   override/refusal state. Global must announce future-Project default/open-
   Project isolation. Project must announce Project scope, journal/Undo, and
   seed-or-migration provenance without implying live Global inheritance.

Two supporting Claude-owned studies must be reconciled in the same protected
lane so they cannot teach the superseded model:

- `project-preferences-category-study.html`: preserve its non-Units content as
  clay and keep Publish/document authority separate and Revision absent/deferred;
  the new Units-only target remains open until owner approval.
- `units-and-grid-model.html`, status and Rules 2/4 interpretation: mark authored
  expression persistence as unratified and the per-person live-display model as
  superseded by PM-040. Preserve explicit-suffix input, exact integer nanometer
  authority, display/grid separation, and all historical comparison, but state
  that an existing Project reads from Project Working Units. Identify Onshape as
  the primary modern professional-CAD authority reference and KiCad as
  low-weight comparison evidence only. Add the per-quantity precision and
  lossless-edit rules, scientific-notation seam, decimal-degree-only V1, and
  explicit deferral of DMS/radians and authored-expression persistence.

For both supporting files, prefer an explicit scoped amendment over silently
rewriting historical study content. Status wording must make clear what remains
clay, what PM-040 supersedes, and what survives.

Codex must not edit those files or supply a protected-lane marker. Both targets
remain owner-review evidence until the owner explicitly approves them.

<!-- EVIDENCE:SHARED-UNITS-SURFACE-PARITY:UNIT-I02R-RECONCILED-FOR-OWNER-REVIEW -->

<!-- REQ:SHARED-UNITS-SURFACE-PARITY:UNIT-I02V -->
<!-- OWNER:SHARED-UNITS-SURFACE-PARITY:UNIT-I02V:UNIT-I02V -->
### UNIT-I02V — owner review of reconciled contract and protected target

Owner review must compare the controlling requirement, all eight catalog rows,
both Claude-owned targets and their HTML construction, this plan, and the
Frontier. Approval authorizes only UNIT-I03A. A gap in profile semantics, Project
doorway/mutation behavior, lossless editing, angle behavior,
adapter compatibility, interchange independence, Global-default/Project-working/
Publish-document ownership, visual behavior, accessibility, or proof returns
the work to UNIT-I02R.

On 2026-09-03 the owner returned the work to UNIT-I02R before approval. The
industrial-readiness audit found six blocking gaps: no complete writable
Project Working Units doorway, one length-precision value shared across three
independently selectable unit families, no lossless focus/edit/commit contract,
ambiguous display-only angle behavior, no exact scientific-notation grammar or
future expression boundary, and runtime schema/catalog semantics that still
contradict the proposed contract. All six must be reconciled before UNIT-I02V
can be presented again.

Codex review of Claude commit `b89bb91` found that its own twenty-three
assertions did not test the controlling values or complete state inventory. It
remains a correction candidate, not completed visual evidence: Board and Drill
show `Automatic (0.01 mm)` instead of `0.001 mm`; the changed Board/mil example
shows `0.001 in` instead of `0.1 mil`; U-B retains the retired shared-precision
table; U-G and U-H retain the old length-precision/angle-notation order and
incomplete accessibility inventory; the scope caption still says six; the
Project doorway draws enabled and disabled commands simultaneously and implies
another submenu; its 700px layout is unusable; and it lacks actual changed,
search, cross-system, unreadable, greyscale, keyboard, and screen-reader states.
The next protected correction must assert exact resolved labels for every
automatic table cell and absence of every retired passage, not merely key and
option counts.

Codex source and rendered review of Claude correction `f8521cd` accepts the
eight-control construction, exact Automatic-resolution mappings, Project Reset
contract, lossless-editing contract, complete interaction/accessibility states,
and usable 1300px/700px layouts. It still rejects the Project prototype as
completed UNIT-I02R evidence because its authority boundaries contradict the
new target. The page-level PM-040 amendment still says that no Project
Preferences implementation is drawn; the target introduction says everything
below it remains unchanged clay; and the visible historical Units row after
the target still advertises six descriptors and an angle-format control. The
final protected correction is boundary-only: distinguish the open P-A through
P-J owner-review target from the original clay category study with explicit
status text and a divider, retract the obsolete no-drawing sentence without
claiming approval, and reconcile the historical Units row to eight settings
with decimal-degree angle precision. Preserve every accepted `f8521cd`
control, value, behavior, layout, exclusion, and non-approval boundary.

Final Codex source and 1300px/700px rendered review of Claude correction
`098c6da` accepts the protected visual reconciliation. P-A through P-J are now
unambiguously the open owner-review target; the original category study is
separately badged and divided as clay; the PM-040 note authorizes no runtime or
production acceptance; and its historical Units row now teaches the same eight
settings, decimal-degree-only V1 angle precision, Project-owned working values,
copy-once Global seed, and separate Publish/document authority as the
controlling contract. The full governance suite passes. UNIT-I02R is complete
as planning evidence and UNIT-I02V is now the required owner boundary. No
UNIT-I03A execution, dependency, or production acceptance is authorized.

<!-- EVIDENCE:SHARED-UNITS-SURFACE-PARITY:UNIT-I02R-FINAL-AUDIT -->

On 2026-09-03, after the corrected contract and protected targets passed the
final source, render, and governance audit, the owner replied exactly
`UNITS-SURFACE-PARITY: approve UNIT-I03A`. This completes UNIT-I02V and
authorizes only the exact engine, schema, migration, adapter-seam, and rival-
conversion-removal work in UNIT-I03A. It does not authorize UNIT-I03B GUI
integration, a dependency, running-app acceptance, or production acceptance.

<!-- EVIDENCE:SHARED-UNITS-SURFACE-PARITY:UNIT-I02V-OWNER-REVISION-20260903 -->

<!-- REQ:SHARED-UNITS-SURFACE-PARITY:UNIT-I03A -->
### UNIT-I03A — exact engine, schema, and migration execution

Only after UNIT-I02V approval, implement the eight-field profile and resolver,
per-quantity precision, lossless edit primitives, decimal-degree V1 service,
exact scientific notation, typed provenance/refusals, descriptor migrations,
Project profile/schema and pre-feature migration, compatible CLI/MCP seams, and
the complete rival-conversion inventory/removal. Correct catalog/runtime
lifecycles so Global Units are future-Project seeds only. No GUI production
acceptance is claimed, and UNIT-I03B does not start until these exact-core and
migration gates are green.

UNIT-I03A completed on 2026-09-04 in commits `7972fbd`, `add4e98`, `4403dae`,
and `4665c8c`. The engine now owns one exact eight-field Units profile, the
complete per-quantity Automatic precision tables, explicit same-system and
cross-system override state, lossless edit-session primitives, exact checked
decimal/scientific length parsing, and exact decimal-degree V1 angle
parse/format behavior. CLI, MCP, Eagle, KiCad, preference-schema, and Project
aggregate seams delegate to that service or refuse unsupported input; the
public rival conversion helpers are removed. Legacy descriptor migration is
planned before mutation, preserves/refuses invalid angle state, fans the retired
shared precision into only missing quantity fields, and records both retired
and live identities in its receipt. Pre-feature Project migration is sourced
from the versioned Datum factory profile rather than current machine defaults.

Proof passed without a new dependency: 1,004 engine tests, 894 CLI tests, 126
GUI-protocol tests, all doctests, strict guarded Clippy with warnings denied,
the Units conversion inventory, and the evidence, source-health, governance,
parity, progress, dependency, Cargo-resource, and alignment gates. This closes
the exact-core execution slice only. No Global or Project Units window, live
Project mutation, running-app acceptance, Publish behavior, Revision behavior,
or Units production acceptance is claimed, and UNIT-I03B remains separately
unauthorized.

<!-- EVIDENCE:SHARED-UNITS-SURFACE-PARITY:UNIT-I03A-EXACT-CORE -->

<!-- REQ:SHARED-UNITS-SURFACE-PARITY:UNIT-I03B -->
### UNIT-I03B — Global and Project Units surface integration

On 2026-09-04, after UNIT-I03A completed with all exact-core and governance
proofs green, the owner directed Datum to “please proceed” with the canonical
next step. This authorizes UNIT-I03B only: construction and proof of the Global
default and Units-only Project Preferences production candidate described
below. It does not authorize a new dependency, UNIT-I03V owner acceptance,
UNIT-I03C production closure, Publish behavior, Revision behavior, or any
non-Units Project Preferences category.

<!-- EVIDENCE:SHARED-UNITS-SURFACE-PARITY:UNIT-I03B-OWNER-AUTHORIZATION-20260904 -->

Route the eight Global default controls and the Units-only Project Preferences
window through UNIT-I03A. Integrate real New-Project seed/receipt, Project-owned
journal/Undo/stale/refusal behavior, GUI numeric fields, compatible CLI/MCP
edges, and preference-independent interchange. Existing Projects read only
`ProjectDisplayUnits`; Publish/document units remain separate. Produce all
protected-target, accessibility, narrow/non-color, real-Project, lossless-edit,
zero-geometry-mutation, and cross-surface matrix evidence. This creates a
production candidate; it does not self-accept the feature.

UNIT-I03B completed its production-candidate construction on 2026-09-04 in
commits `e87932c`, `24541d2`, `bc7bd17`, `e149e8d`, `92caa68`, and `2ae0eb8`.
Global Preferences now exposes the eight exact future-Project defaults beside
the accepted Appearance slice. Edit > Preferences > Project Preferences opens
one native owned, input-modal Units-only window when a Project exists and is
disabled without one; it adds no Navigator category. Both windows reuse the
descriptor catalog, immediate-save behavior, search/focus/Escape contract,
accessible value/provenance projection, narrow scrolling, and preserved-
unreadable refusal state.

The native Project aggregate owns `ProjectDisplayUnits` and one immutable,
itemized seed or migration receipt. New Projects copy the resolved Global
defaults once; pre-feature Projects deterministically receive the versioned
factory profile; existing Projects never live-follow Global. Project changes
use guarded journal operations with inverse/Undo and stale-write refusal, while
board and schematic geometry remain byte-identical. Reset restores the receipt
value without consulting the machine. The GUI exact-edit adapter consumes an
immutable Project-resolved profile and typed quantity/field context; no broad
geometry numeric editor exists yet to expose a second interpretation path.
CLI and MCP provide the same exact resolver while retaining `_nm`
compatibility, and interchange remains preference-independent.

Focused proof passes 41 engine Units tests, 2 CLI adapter tests, 22 MCP Units
and protocol tests, the real daemon Project mutation proof, 292 GUI tests with
8 intentional skips, 411 MCP self-tests with 45 intentional skips, strict
workspace Clippy, source health, parity, governance, evidence, dependency,
alignment, accessibility, and cross-surface boundary checks. The broad
workspace test run passed its Units coverage and reproduced one pre-existing
terminal signal-normalization timing failure that passed in isolation. The
full drift route reaches an unrelated stale Console visual golden: its stored
image still contains the permanent Revision Navigator groups withdrawn by
PM-038, while current runtime correctly omits them. That visual-authority debt
is tracked separately as `dat-console-golden-revision-recovery-mtv` and was not
laundered through the Units lane.

This evidence completes UNIT-I03B as an owner-QA candidate only. It does not
claim running-app approval, Units production acceptance, Publish behavior,
Revision behavior, a new dependency, or any non-Units Project Preferences
category.

<!-- EVIDENCE:SHARED-UNITS-SURFACE-PARITY:UNIT-I03B-PRODUCTION-CANDIDATE -->

<!-- REQ:SHARED-UNITS-SURFACE-PARITY:UNIT-I03V -->
<!-- OWNER:SHARED-UNITS-SURFACE-PARITY:UNIT-I03V:UNIT-I03V -->
### UNIT-I03V — owner running-app acceptance

The owner reviews both native windows and real-Project behavior, including
Global isolation, Project changes/Undo, precision resolution, cross-system
overrides, exact focus/edit behavior, explicit and bare input, migration,
accessibility, and failure states. Rejection returns to the responsible
execution slice; approval authorizes final proof closure only.

On 2026-09-04 the owner replied exactly
`UNITS-PRODUCTION-CANDIDATE: approve UNIT-I03C` after direct running-app review.
This completes UNIT-I03V and authorizes only the final governed proof closure
defined by UNIT-I03C. It does not itself record production acceptance, authorize
a dependency, expand Project Preferences, or change Publish or Revision.

<!-- EVIDENCE:SHARED-UNITS-SURFACE-PARITY:UNIT-I03V-OWNER-APPROVAL-20260904 -->

<!-- REQ:SHARED-UNITS-SURFACE-PARITY:UNIT-I03C -->
### UNIT-I03C — governed production-acceptance closure

Re-run every acceptance-matrix row, governance gate, guarded Rust proof, runtime
capture, compatibility fixture, and source-health/dependency check against the
owner-approved candidate. Only this step may record Units production acceptance
and close the tracker. Partial parity or planning/prototype evidence cannot.

UNIT-I03C completed the governed production-acceptance closure on 2026-09-04.
The final proof run found and corrected three previously unreachable inventory
and recovery defects: commit `662dcdb` adds the exact Units resolver terminal
handoff to its production catalog proof; commit `57baf17` preserves displayed
values while disabling both controls and Reset in the unreadable Preferences
state; and commit `7331467` reconciles the complete native-write and eleven-row
default-service inventories. Each correction passed its focused proof before the
complete matrix was restarted.

The guarded `cargo test --workspace --all-targets` run is green from the final
committed state, including 893 CLI tests, 300 GUI-app tests with eight intentional
environment-bound skips, 151 GUI-render tests, 1,009 engine tests, daemon and
test-harness suites, and the long M3 acceptance gate. Strict guarded workspace
Clippy passes with warnings denied. The MCP self-test passes 411 tests with 45
intentional skips, alignment passes, and the source-health, dependency,
conversion-inventory, spec-parity, spec-governance, evidence-traceability,
progress, project-state, MCP-taxonomy, daemon-write, menu-model, and Cargo-
resource gates all pass.

Fresh compositor captures from a disposable native Project were rendered and
inspected at `/tmp/unit-i03c-global-preferences-native.png` and
`/tmp/unit-i03c-project-preferences-native.png`. They prove the independent
owned Global window with only Appearance and Units, immediate-save scope and no
duplicate transaction buttons, plus the Units-only Project window with exact
resolved Follow-system/Automatic labels, immutable seed-or-migration provenance,
scrolling, and no Navigator presence. The disposable process was stopped after
capture and no user Project was opened or changed.

The umbrella drift route passes every preceding gate and then stops only at the
separately tracked Console visual golden mismatch: 3,778 pixels (0.384318%) in
`routine-focused.golden.png`. That stored image still depicts the permanent
Revision Navigator groups withdrawn by PM-038 while current runtime correctly
omits them. This is unrelated visual-authority debt tracked by
`dat-console-golden-revision-recovery-mtv`; it is a related, non-blocking issue,
was not refreshed or blessed from the Units lane, and does not weaken any Units
matrix row.

The eight-descriptor exact Units service, Global defaults for future Projects,
Project Working Units authority for existing Projects, lossless editing,
deterministic seed/migration receipts, exact CLI/MCP compatibility, accessibility,
and preference-independent interchange boundary are therefore production-
accepted. This closes UNIT-I03C and `dat-shared-units-surface-parity-s9x`. It
does not authorize a dependency, Publish or Revision behavior, any non-Units
Project Preferences category, or Global Preferences completion; GP-CM00 remains
its own owner boundary.

<!-- EVIDENCE:SHARED-UNITS-SURFACE-PARITY:UNIT-I03C-PRODUCTION-ACCEPTED -->

## 6. Global Preferences completion

<!-- REQ:GLOBAL-PREFERENCES-COMPLETION:GP-CM00 -->
<!-- OWNER:GLOBAL-PREFERENCES-COMPLETION:GP-CM00:GP-CM00 -->
### GP-CM00 — authorize Global completion only

Review GP-F05 and completed UNIT-I03C before authorizing GP-CM01.

On 2026-09-04 the owner replied exactly
`GLOBAL-PREFERENCES-COMPLETION: approve GP-CM01`. This completes GP-CM00 and
authorizes only the bounded Global Preferences surface and repository
stabilization in GP-CM01. Revision descriptors, empty categories, product-
surface API parity, ProjectPolicySeed expansion, and production acceptance
remain outside this authorization.

<!-- EVIDENCE:GLOBAL-PREFERENCES-COMPLETION:GP-CM00-OWNER-APPROVED-20260904 -->

On 2026-09-05 the owner corrected GP-CM01 after the full-catalog readiness
audit found 45 registered descriptors without real consumers and several
incomplete presentation contracts. The owner directed consumer-ready
activation: reserved keys do not enter the product surface until control,
consumer, effect timing, accessibility, and proof are complete; Unwired rows
and empty sections remain absent. Manage Preferences appears only as an
implemented operations section. The aggregate Units seed is not an ordinary
GUI control pending separate justification. Guided setup remains deferred until
all four approved checkpoints exist, and the Start page and PM-040 must agree
with the real eight-descriptor Units authority before GP-CM01 resumes surface
expansion.

<!-- EVIDENCE:GLOBAL-PREFERENCES-COMPLETION:GP-CM01-OWNER-REVISION-20260905 -->

<!-- REQ:GLOBAL-PREFERENCES-COMPLETION:GP-CM01 -->
### GP-CM01 — complete and stabilize the Global surface

Preserve the accepted 11-row Appearance-and-Units surface as the exact initial
production-active inventory. The remaining 45 registered identities are
reserved candidates, not active settings. Activate a candidate only after its
typed control and defaults, declared subsystem consumer, effect timing,
repository behavior, accessibility, and focused proof are complete. Until then
it is unknown-and-preserved to the production repository and absent from GUI,
search, setup, APIs, and Project seeding. Sections appear only when they contain
an active row.

Complete managed/refused, unknown/retired-identity, recovery/migration/restore,
search/provenance, responsive, and accessibility behavior through the existing
GP-F05 repository, resolver, presentation-catalog, menu, mutation, focus, and
accessibility paths. Add Manage Preferences only when its previewed operations
are implemented; it is a nonempty operations section, not a descriptor or an
empty placeholder. Keep guided setup deferred until all four approved
checkpoints are available. Reconcile the separate Start-page truth before it is
enabled. `datum.projects.unit_policy_seed` remains a reserved internal
aggregate and is absent from the ordinary GUI pending separate justification.
Revision descriptors and empty categories remain absent.

<!-- REQ:GLOBAL-PREFERENCES-COMPLETION:GP-CM02 -->
<!-- OWNER:GLOBAL-PREFERENCES-COMPLETION:GP-CM02:GP-CM02 -->
### GP-CM02 — authorize product-surface and seed parity only

Review GP-CM01 before authorizing GP-CM03.

On 2026-09-05 the owner declined direct GP-CM03 authorization and directed
exactly:

> GLOBAL-PREFERENCES-COMPLETION: revise — before GP-CM03 implementation,
> replace the placeholder product-surface paragraph with an exact build
> contract. Define the complete typed operation, query, refusal, Reset,
> explanation, and proposal inventories; stable request/response schemas;
> actor, provenance, authorization, expected-generation, idempotency, and audit
> semantics; exact CLI and MCP commands/tools and error mappings; one
> engine-owned service path with no private parser or writer; and the
> configuration-root, daemon, and multi-process ownership model. Limit Project
> genesis to the eight production-active Units seeds, keep the other six
> reserved ProjectPolicySeed identities and datum.projects.unit_policy_seed
> aggregate inactive, and specify identical GUI/CLI/MCP Project-creation modes.
> Define pinned-snapshot atomicity, concurrent-write and crash behavior,
> immutable receipt contents, retry/replay rules,
> unreadable/migration/stale/refusal outcomes, and explicit factory-mode
> behavior. Supersede PM-037’s stale planned/read-only search clause and add a
> clause-by-clause cross-surface acceptance matrix proving no live Project
> following, no reserved activation, no private mutation path, and deterministic
> real-Project behavior. Do not authorize GP-CM03 until these contracts agree
> across doctrine, implementation plan, public registries, CLI, MCP, daemon, and
> Project genesis.

This disposition completes GP-CM02 as a revision decision and authorizes only
GP-CM02R specification reconciliation. It does not authorize runtime, CLI, MCP,
daemon, GUI, Project, prototype, dependency, Publish, Revision, or GP-CM03
implementation work.

<!-- EVIDENCE:GLOBAL-PREFERENCES-COMPLETION:GP-CM02-OWNER-REVISION-20260905 -->

<!-- REQ:GLOBAL-PREFERENCES-COMPLETION:GP-CM02R -->
### GP-CM02R — reconcile the GP-CM03 production-build contract

Specify and cross-prove all nine owner-required readiness corrections across
the controlling doctrine, engine/API boundary, CLI, MCP, daemon, Project
genesis, security, concurrency, failure, and acceptance contracts. Preserve the
11-active/45-reserved boundary and make no implementation change.

<!-- REQ:GLOBAL-PREFERENCES-COMPLETION:GP-CM02V -->
<!-- OWNER:GLOBAL-PREFERENCES-COMPLETION:GP-CM02V:GP-CM02V -->
### GP-CM02V — fresh owner authorization after reconciliation

Review the committed GP-CM02R contract and evidence before authorizing GP-CM03.

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
