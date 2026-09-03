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
authorizing UNIT-I03.

The owner instead revised this boundary on 2026-09-02: preserve the twelve
clauses and seven proof requirements in
`GP_SHARED_UNITS_ENGINE_REQUIREMENT.md`, but do not authorize UNIT-I03 until the
requirement, catalog, protected target, this plan, and Frontier describe one
exact system without gaps or conflicting ownership.

<!-- EVIDENCE:SHARED-UNITS-SURFACE-PARITY:UNIT-I02-OWNER-REVISION -->

<!-- REQ:SHARED-UNITS-SURFACE-PARITY:UNIT-I02R -->
### UNIT-I02R — reconcile the implementation and visual contract

The controlling requirement's **UNIT-I03 reconciliation contract**, as amended
by Product Mechanics 040, defines the typed unresolved/resolved profiles, full
follow-system table, precision-token mapping, bounded angle-format service,
Global-default GUI behavior, Project-owned Working Units, separate
Publish/document units, additive CLI/MCP compatibility,
preference-independent interchange, legacy conversion disposition, Project
snapshot/receipt seam, and clause-by-clause acceptance matrix. UNIT-I02R is
documentation and protected-target reconciliation only.

The Claude-owned `docs/gui/prototypes/preferences-window.html` must be revised
in its GP-F05 owner-review target before this step completes. The bounded brief
is:

1. Retitle the owner-review **Units** state in the real native Global
   Preferences window as **Defaults for new Projects**, using the accepted
   GP-F05 component grammar, not the non-buildable future catalog as
   implementation authority. Preserve exactly one Units rail entry; the
   category heading/consequence text carries the future-Project scope.
2. Render exactly the six catalog descriptor rows and seven focusable value
   selectors, their specified options/defaults,
   per-row `Global · this device` scope, immediate-save/reset/provenance, and the
   copy-once-at-New-Project receipt explanation. Every state must say that open
   and existing Projects are unaffected. Do not render or mirror Project Working
   Units, Project-policy editing, Publish/document units, Revision, setup,
   Organization, or empty categories.
3. Show every follow-system resolved unit, explicit cross-system override cue,
   five length-precision choices, and the notation-dependent angle precision
   choices. Attach each actual option inventory structurally to its owning
   selector in the HTML, rather than leaving the options only in explanatory
   tables. Invalid angle notation/precision pairs are never offered. Radians
   must be described as rounded display; no general angle-input promise may
   appear. The tagged Angle value is edited through notation and precision
   selectors. Selecting a notation atomically writes that notation with its
   defined default precision—`decimal_degrees`/`0.1`, `dms`/`1s`, or
   `radians`/`0.001`—and never persists an intermediate pair, carries forward a
   prior precision, or reinterprets the shared `0.001` token.
4. Preserve the accepted owned-window, close/reopen, Escape/search/caret,
   explanation, immediate-save, responsive, and non-color behavior. Add a Units
   unreadable-store state with all seven value selectors disabled/preserved, plus
   ordinary, changed/reset, search, cross-system, refusal/explanation, narrow,
   greyscale, and screen-reader evidence.
5. Prove from the HTML structure and renders that only the selected **Units**
   row is highlighted, category/content separation uses the accepted rules, all
   controls are reachable and named, no excluded category is implied, and the
   file remains within its source-health limit.
6. Specify the exact Units focus order: Appearance rail entry, Units rail entry,
   search, then each visible setting-name action and its control in stable row
   order, each conditional Reset immediately after its control, and explanation
   Close only while open. A dependent Angle format control exposes notation
   before its valid precision choices. A notation change keeps focus on the
   notation selector while replacing the precision inventory and atomically
   committing the complete tagged value. Tab wraps inside the input-modal window;
   Escape, close/reopen reset, return focus, caret, and search-clearing behavior
   remain exactly GP-F05.
7. For the window and every Units value selector, provide accessible name, role, value,
   resolved unit where relevant, factory-or-changed provenance, `Global · this
   device`, `changes save immediately`, `default for new Projects`, and `open
   Projects unaffected`. Screen-reader evidence must not imply that Global
   values operate the current Project.

Two supporting Claude-owned studies must be reconciled in the same protected
lane so they cannot teach the superseded model:

- `project-preferences-category-study.html`, Units row and category rationale:
  identify `ProjectDisplayUnits` as **Project Working Units**, cite PM-040, state
  that it owns existing-Project editor display/readout/bare input, and keep
  Publish/document units under the separate Drafting Standard/Publish authority.
  Preserve the artifact's clay status; do not draw or authorize a Project
  Preferences implementation.
- `units-and-grid-model.html`, status and Rules 2/4 interpretation: mark authored
  expression persistence as unratified and the per-person live-display model as
  superseded by PM-040. Preserve explicit-suffix input, exact integer nanometer
  authority, display/grid separation, and all historical comparison, but state
  that an existing Project reads from Project Working Units. Identify Onshape as
  the primary modern professional-CAD authority reference and KiCad as
  low-weight comparison evidence only.

For both supporting files, prefer an explicit scoped amendment over silently
rewriting historical study content. Status wording must make clear what remains
clay, what PM-040 supersedes, and what survives.

Codex must not edit that file or supply its protected-lane marker. The target
remains owner-review evidence until the owner explicitly approves it.

<!-- EVIDENCE:SHARED-UNITS-SURFACE-PARITY:UNIT-I02R-RECONCILED-FOR-OWNER-REVIEW -->

<!-- REQ:SHARED-UNITS-SURFACE-PARITY:UNIT-I02V -->
<!-- OWNER:SHARED-UNITS-SURFACE-PARITY:UNIT-I02V:UNIT-I02V -->
### UNIT-I02V — owner review of reconciled contract and protected target

Owner review must compare the controlling requirement, all six catalog rows,
the Claude-owned target and its HTML construction, this plan, and the Frontier.
Approval authorizes only UNIT-I03. A gap in profile semantics, angle behavior,
adapter compatibility, interchange independence, Global-default/Project-working/
Publish-document ownership, visual behavior, accessibility, or proof returns
the work to UNIT-I02R.

<!-- REQ:SHARED-UNITS-SURFACE-PARITY:UNIT-I03 -->
### UNIT-I03 — GUI, CLI, MCP parity and Units production acceptance

Only after UNIT-I02V approval, implement the reconciliation contract and route
the six ratified `datum.units.*` descriptors, GUI controls, compatible CLI/MCP
edges, deterministic interchange adapters, and real New-Project Units seed
through the one exact service. The Project mutation authority remains the sole
writer of `ProjectDisplayUnits` and its receipt. Existing Projects resolve
working display and contextual bare input from that Project-owned value, never
from Global Preferences; Publish/document units remain separate. Delete,
deprecate, or refuse
every inventoried rival conversion path according to the contract without
moving descriptor ownership into Units. Production acceptance requires every
matrix row and protected-target proof; partial parity cannot close UNIT-I03.

## 6. Global Preferences completion

<!-- REQ:GLOBAL-PREFERENCES-COMPLETION:GP-CM00 -->
<!-- OWNER:GLOBAL-PREFERENCES-COMPLETION:GP-CM00:GP-CM00 -->
### GP-CM00 — authorize Global completion only

Review GP-F05 and UNIT-I03 before authorizing GP-CM01.

<!-- REQ:GLOBAL-PREFERENCES-COMPLETION:GP-CM01 -->
### GP-CM01 — complete and stabilize the Global surface

Expand the accepted GP-F05 architecture through the full active catalog and
the remaining managed/refused, unknown/retired-identity,
recovery/migration/restore, setup/replay, Start-page, search/provenance,
responsive, and accessibility states. Do not replace GP-F05's repository,
resolver, presentation-catalog, menu, mutation, focus, or accessibility paths
with a second implementation. Revision descriptors and empty categories remain
absent.

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
