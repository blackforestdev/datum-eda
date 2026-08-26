# GP-C02 External and Standards Research

> **Status:** research synthesis for Global Preferences specification; no
> architecture, dependency, provider, standards-conformance claim, or
> implementation is authorized.
>
> **Tracker:** `dat-global-preferences-engine-qcv`
> **Frontier step:** `GLOBAL-PREFERENCES-SPEC / GP-C02`
> **Internal baseline:** `GP_C01_INTERNAL_AUTHORITY_AUDIT.md`, owner-approved as
> `GP-C01-BASELINE` on 2026-08-25.

## 1. Research questions

This report examines primary and authoritative sources for:

1. typed setting identity, schema, validation, and defaults;
2. scope, precedence, provenance, constraints, and managed policy;
3. separation of configuration, restartable UI state, Project authority, and
   per-operation input;
4. local/offline persistence, atomic replacement, durability, and recovery;
5. schema/value migration, downgrade behavior, and unknown-field preservation;
6. import/export, synchronization, concurrent updates, and conflict handling;
7. accessible discovery, editing, refusal, status, and onboarding;
8. regulated configuration-management expectations;
9. the exact limits of what these sources justify for Datum.

The goal is not to imitate one application. It is to extract mechanisms that
survive across desktop platforms and remain consistent with Datum's locally
complete, typed, Project-authoritative architecture.

## 2. Source-strength method

Sources are classified as:

- **standard / government authority:** normative or official public standard,
  RFC, W3C Recommendation, or NIST publication;
- **platform authority:** official operating-system or desktop-framework
  documentation;
- **product authority:** official documentation for the product whose behavior
  is being compared;
- **transferable mechanism:** a pattern supported across more than one source;
- **product convention:** useful evidence, but not a Datum requirement by
  itself.

No proprietary or licensed standard text was used. NIST controls are relevant
to managed software configuration, not evidence that all Datum preferences are
regulated configuration items. WCAG is used as interaction guidance for the
native GUI; this report does not claim web-content conformance for Datum.

## 3. Source register

| Source | Strength | What it supports | What it does not decide |
|---|---|---|---|
| [Git `git-config`](https://git-scm.com/docs/git-config) | product authority | named scopes, ordered precedence, typed canonicalization, origin/scope inspection, protected scopes | Datum's exact scope order or managed-policy semantics |
| [XDG Base Directory Specification 0.8](https://specifications.freedesktop.org/basedir/latest/) | platform specification | separate config/state/cache/runtime locations, absolute-path handling, precedence and error duties | Windows/macOS locations or Datum file format |
| [GSettings](https://docs.gtk.org/gio/class.Settings.html) | platform authority | typed schemas, defaults, ranges/enums, explicit user action, atomic grouped apply, writability | adopting GLib/dconf as a dependency |
| [GNOME dconf profiles](https://help.gnome.org/system-admin-guide/dconf-profiles.html) and [locks](https://help.gnome.org/system-admin-guide/dconf-lockdown.html) | platform authority | layered databases, a write target, reversed lock precedence, user writability | Datum's provider or exact layer names |
| [Apple Managed Preferences](https://developer.apple.com/documentation/devicemanagement/managedpreferences) | platform authority | app preference domains, forced and set-once managed values, device/user channels | cross-platform transport or Datum policy identity |
| [Chromium preference/policy guidance](https://www.chromium.org/administrators/configuring-other-preferences/) | product authority | mandatory policy > user preference > recommended policy; recommended values remain editable | a universal precedence law |
| [Firefox enterprise policy templates](https://mozilla.github.io/policy-templates/) | product authority | explicit default/locked/user/clear states, typed values, persistence after policy removal | current authoritative Firefox syntax for every policy; page warns some documentation moved |
| [JSON Schema 2020-12 Core](https://json-schema.org/draft/2020-12/json-schema-core) and [Validation](https://json-schema.org/draft/2020-12/json-schema-validation) | specification | type, enum, range, object/unknown-property validation vocabulary | storage, migration, or runtime precedence |
| [Protocol Buffers proto3 guide](https://protobuf.dev/programming-guides/proto3/) | product-format authority | compatible evolution, reserved identities, unknown-field preservation and loss hazards | adoption of Protocol Buffers |
| [Qt `QSaveFile`](https://doc.qt.io/qt-6/qsavefile.html) | framework authority | same-directory temporary write, commit, full-disk detection, refusing unsafe fallback for config | complete power-loss durability or adopting Qt |
| [SQLite atomic commit](https://www.sqlite.org/atomiccommit.html) and [corruption guidance](https://www.sqlite.org/howtocorrupt.html) | product authority | flush/order/rollback concepts; filesystem lock and network-filesystem hazards | adoption of SQLite |
| [RFC 6902 JSON Patch](https://www.rfc-editor.org/rfc/rfc6902.html) | standards track | ordered typed patch operations, test preconditions, all-or-error processing | Datum's exchange format |
| [RFC 7396 JSON Merge Patch](https://www.rfc-editor.org/rfc/rfc7396.html) | standards track | simple object merge and explicit deletion | array merge, explicit-null domains, concurrency |
| [RFC 9110 conditional requests](https://www.rfc-editor.org/rfc/rfc9110.html#name-if-match) | Internet Standard | validators/If-Match preventing lost updates | requiring HTTP or a server |
| [RFC 8785 JSON Canonicalization Scheme](https://www.rfc-editor.org/rfc/rfc8785.html) | informational RFC | deterministic JSON for reproducible hashing/signing | cryptographic algorithm, trust roots, or adoption |
| [WCAG 2.2](https://www.w3.org/TR/WCAG22/) | W3C Recommendation | keyboard access, name/role/value, non-color semantics, status messages, predictable input behavior | native-platform conformance certification |
| [NIST SP 800-128](https://csrc.nist.gov/pubs/sp/800/128/upd1/final) | government guidance | controlled baselines, monitored configuration, risk-based configuration management | making every presentation choice a controlled baseline |
| [NIST SP 800-53 Rev. 5.1 CM-6](https://csrc.nist.gov/CSRC/media/Projects/risk-management/800-53%20Downloads/800-53r5/SP_800-53_v5_1-derived-OSCAL.pdf) | government control catalog | document settings, approve deviations, monitor/control changes, automated management/verification | a mandatory profile for every Datum user |

## 4. Typed registry and value model

### 4.1 Stable identity belongs to the setting, not its current label

GSettings identifies schemas and keys independently of localized summaries and
descriptions. It declares types, defaults, ranges, choices, enums, and flags.
Git similarly gives each configuration value a stable dotted key and can
canonicalize declared primitive types. JSON Schema separates validation
vocabulary from instance data.

The transferable mechanism is a typed descriptor registry. A complete Datum
descriptor needs at least:

- stable `PreferenceKey` identity and owning subsystem;
- value type and canonical encoding;
- factory default or explicit “no value” state;
- enum/range/structural constraints;
- allowed scopes and whether the value is seedable into a Project;
- whether organization policy may recommend, constrain, or lock it;
- merge behavior (scalar replace, set union, ordered list, keyed map, or
  indivisible object);
- restart/live-apply behavior and affected consumers;
- sensitivity/security classification and export eligibility;
- accessible label, description, consequences, and reset semantics;
- schema version, migration identity, and retirement aliases.

This is an evidence-backed requirement candidate for GP-C03, not an approved
Datum schema.

### 4.2 Defaults must remain distinguishable from stored values

GSettings stores defaults in the schema and exposes user values separately.
Git's `--default` supplies a fallback without pretending it was configured.
Firefox distinguishes default, user, locked, and clear states. These systems
support a crucial distinction:

```text
factory default ≠ recommended value ≠ explicit user value ≠ managed value
```

Datum therefore should never write factory defaults into a user file merely
because a Preferences UI was opened. GSettings explicitly warns applications
to modify settings only in response to user action and not while initializing
widgets. This directly addresses GP-C01's risk of accidental rewrite and makes
“reset” a removal of an explicit contribution, not another magic value.

### 4.3 Validation does not equal migration

JSON Schema and GSettings prove that type/range validation can be declarative.
Neither automatically answers how an old key becomes a new key, how a unit
changes, or how a removed enum value is handled. Datum must define migrations
as explicit, versioned transformations with evidence and failure behavior;
merely accepting a new schema version is insufficient.

## 5. Scope, precedence, provenance, and managed policy

### 5.1 Value precedence and constraint authority are different axes

Git reads system, global, local, worktree, and command scopes in order and can
report both `--show-origin` and `--show-scope`. GNOME dconf normally lets user
values override system defaults, but reverses precedence for locks: a system
lock prevents the user layer from writing. Chromium similarly separates
recommended policy (editable) from mandatory policy (not editable).

The cross-source conclusion is decisive:

> A managed lock is not merely another high-precedence value. It is a typed
> constraint on which contributions may exist or become effective.

A Datum resolver therefore needs to consider, per key:

1. available value contributions;
2. recommendation/default contributions;
3. constraints and locks;
4. provider trust/availability;
5. context applicability;
6. validation and migration state;
7. the selected effective value and why other values lost or were refused.

### 5.2 Provenance must be a first-class query

Git's origin/scope reporting is the strongest directly transferable precedent.
For every effective Datum value, inspection should be capable of returning:

- effective value;
- contributing scope and provider identity;
- whether it is factory, recommended, explicit, seeded, contextual, or managed;
- constraint/lock and managing reason;
- source revision/generation and evaluation time;
- ignored, invalid, unavailable, superseded, or conflicting contributions;
- whether the current user can change, reset, or override it.

The UI should consume that query rather than independently reconstructing
precedence. This is especially important for the approved Managed Revision
Visibility requirement: “visible because organization policy pins it on” must
be a real resolver fact, not greyed-out widget folklore.

### 5.3 Protected settings need scope eligibility

Git ignores certain security-sensitive options unless they come from a
protected scope. This establishes another transferable mechanism: some keys
must declare which scopes are eligible to control them. A Project controlled by
an untrusted checkout must not be able to select executable paths, unattended
agent authority, managed-provider endpoints, or trust roots merely because a
Project-local value would otherwise have high contextual specificity.

This supports classifying the GP-C01 terminal profile mix before migration:
theme and font are presentation; executable/environment are launch capability;
agent authority is security policy. They cannot all share the same scope law.

### 5.4 Managed-policy conflicts must be visible, not guessed away

Microsoft Intune's official guidance exposes policy conflict instead of always
inventing an implicit winner; GNOME makes lock order explicit. The evidence does
not support a universal “most restrictive” algorithm for arbitrary values.
“Most restrictive” is meaningful only when a descriptor defines an ordering.

Datum should therefore treat same-authority incompatible managed contributions
as a typed conflict unless that setting's descriptor provides a deterministic
join rule. Silent last-writer-wins would make administration order an
undocumented authority.

## 6. Configuration, state, Project policy, and operation input

### 6.1 Configuration and restartable state have different homes

The XDG specification explicitly distinguishes user configuration from
restartable state such as view, layout, open files, and history. This maps
cleanly onto GP-C01:

- Console duration, theme, crosshair default, and terminal appearance are
  preference candidates;
- pane layout, open panes, focus, camera, dock geometry, and last-opened context
  are restartable workspace state;
- hover, cursor position, drag previews, active menus, and live terminal bytes
  are transient session state.

Persistence alone does not make all three “Global Preferences.” A unified
storage implementation may serve multiple classes, but their authority,
retention, exchange, privacy, and reset rules remain typed and distinct.

### 6.2 Project policy is copied authority, not a live global override

Git's repository-local configuration demonstrates contextual scope, but Datum's
ratified Project doctrine is stricter for engineering policy: a new Project may
be seeded from current preferences/organization policy, after which the copied
Project policy is authored Project truth. It must not change silently when a
machine preference or organization recommendation later changes.

The research supports recording a seed receipt with source profile identity,
generation, copied values, time, and omissions. It does not support a live link
unless an independently governed Project policy explicitly adopts one.

### 6.3 Per-operation arguments remain explicit

Command-line and dialog inputs such as export destination, route endpoints, or
one check target are not automatically preferences. Remembering a last-used
value is a separate presentation convenience with its own key and scope. It
must not convert an explicit operation parameter into hidden policy.

## 7. Local persistence, atomicity, and recovery

### 7.1 Platform-correct locations

XDG assigns configuration to `$XDG_CONFIG_HOME` and restartable state to
`$XDG_STATE_HOME`; relative environment paths are invalid, and inaccessible
locations require defined handling. It also specifies ordered system
configuration directories. Datum's current Console path mostly follows the
config fallback but does not validate absolute paths and has no state split.

The transferable requirement is a platform location adapter with typed store
class, not scattered direct reads of `$HOME`. Windows and macOS backends need
their own platform-authoritative mapping during implementation planning.

### 7.2 Rename is necessary but not a complete recovery design

Qt's QSaveFile documents the familiar safe-save sequence: write a temporary
file in the target directory, detect write/full-disk errors, and commit by
moving it into place. It deliberately refuses unsafe direct fallback for
internal configuration unless the caller opts out of atomicity. SQLite's atomic
commit documentation adds the missing durability lesson: correct flush and
ordering matter, and storage controllers/filesystems can violate assumptions.

A robust Datum file-store contract therefore needs explicit decisions for:

- same-directory temporary creation;
- complete write and validation before promotion;
- file sync and parent-directory sync where the platform requires them;
- atomic replacement capability probing/refusal;
- retained last-known-good generation or rollback record;
- startup selection among current, pending, and backup generations;
- corrupt/truncated/unsupported-version quarantine;
- bounded cleanup of abandoned temporaries;
- single-writer locking and generation compare-and-swap;
- clear user-visible degraded/session-only behavior when persistence fails.

This is a proof obligation, not an instruction to add Qt or SQLite. Product
Mechanics 029 still governs any dependency.

### 7.3 Network filesystems and multiple writers cannot be assumed safe

SQLite's corruption guidance warns that broken filesystem locking, especially
on network filesystems, can corrupt shared databases. Datum must not describe a
local preference store as a collaborative database merely because it is placed
in a synchronized or network-mounted directory. Synchronization belongs above
the local durable store, with explicit generations and conflicts.

## 8. Evolution, migration, downgrade, and unknown data

### 8.1 Unknown preservation is an architectural property

Protocol Buffers demonstrates two important principles without implying that
Datum should adopt protobuf:

- stable field identities must not be reused after removal;
- old readers can preserve unknown fields and re-emit them, but field-by-field
  reconstruction or JSON conversion can discard them.

Datum's current Console preference writer performs exactly the lossy pattern:
it parses the known value and rewrites a freshly constructed known-only object.
A future store must retain an opaque unknown-field/provider envelope alongside
typed known values or mutate the original document without dropping unknowns.

### 8.2 Migration needs three explicit outcomes

Every schema/value migration should end as one of:

1. **migrated:** a deterministic transform produced a valid current value and
   retained a receipt;
2. **preserved unavailable:** unknown key/provider/version is round-tripped but
   excluded from effective resolution;
3. **quarantined/refused:** malformed or unsafe input cannot participate and is
   retained for diagnosis/recovery without overwriting the last-known-good
   store.

Downgrade must not rewrite newer data through an older known-only schema. A
safe older binary may operate read-only for affected keys, preserve the whole
unknown envelope, or use an explicitly compatible projection. Which behavior
applies must be declared by schema/provider, not guessed globally.

### 8.3 Defaults must not be migration side effects

When a new key appears, its schema default supplies behavior without requiring
a write. A migration should write only when old explicit state must be
transformed. This keeps “upgrade Datum” from turning every default into a user
override that forever masks later factory improvements.

## 9. Import, export, synchronization, and conflict

### 9.1 Full snapshots and patches serve different jobs

RFC 7396 provides a compact object-shaped merge patch but assigns `null` to
deletion and replaces arrays wholesale. RFC 6902 provides ordered add/remove/
replace/move/copy/test operations and terminates on failure. Neither is a
complete preference synchronization protocol.

Datum needs to distinguish:

- human-portable export of selected non-sensitive explicit values;
- exact backup/restore of one store generation;
- organization-policy package exchange;
- synchronization deltas between a user's devices;
- Project-policy seed packages;
- diagnostic evidence export.

Each requires a typed envelope containing schema/provider identities, source
scope, generation, exclusions/redactions, and integrity metadata.

### 9.2 Compare-and-swap prevents lost updates

RFC 9110's `If-Match` uses a previously observed validator to prevent one
writer from accidentally overwriting another. The mechanism is transport-
independent: a Datum mutation can require an expected store generation/digest
even when no HTTP server exists. RFC 6902's `test` operation provides a similar
per-value precondition.

This supports:

- local single-writer transactions with an expected generation;
- sync intake that refuses stale-base overwrite;
- typed conflict records containing base, local, and incoming values;
- explicit resolution rather than timestamp-only last-writer-wins.

### 9.3 Merge behavior belongs to the descriptor

Scalars normally replace. Sets may union only if removal semantics are also
defined. Ordered lists need identity and ordering rules. Structured preferences
may be indivisible or field-mergeable. Security/managed values may refuse any
user-sync merge. Unknown values cannot be semantically merged and should remain
opaque/conflicted.

Therefore there is no safe universal JSON merge law for all preferences.

### 9.4 Canonicalization can support evidence, not trust by itself

RFC 8785 defines deterministic JSON suitable for hashing/signing. It does not
choose algorithms, credentials, trust roots, expiry, revocation, or replay
protection. Datum may use canonical bytes/digests for receipts and exchange
evidence, but GP-C03/GP-C04 must separately define provider trust and Product
Mechanics 029 must approve any dependency.

## 10. Accessibility and interaction

WCAG 2.2 supplies transferable interaction obligations:

- all functionality is keyboard operable and has no keyboard trap;
- controls expose programmatic name, role, and value;
- state and refusal do not rely on color alone;
- status messages are announced without unnecessary focus theft;
- changing a setting does not cause an undisclosed context change;
- zoom/reflow and text spacing do not hide controls or provenance.

Applied to Datum Preferences, this means:

- search, categories, editing, reset, import/export, and provenance inspection
  require keyboard paths;
- a managed value remains focusable/readable even when not writable;
- the control exposes effective value, managing scope, reason, and refusal;
- save/apply/failure/conflict/migration results use the established output-only
  Console/notification semantics and accessibility announcements;
- live preview must identify which changes are immediate and which need restart;
- first-Release onboarding preserves context, explains consequence before the
  arm-then-confirm boundary, can be revisited/reset according to policy, and is
  not a surprise modal refusal.

These are design inputs for GP-C05, not a claim that an HTML technique is the
correct native implementation.

## 11. Regulated and enterprise configuration

NIST SP 800-128 and SP 800-53 CM-6 support organization-defined configuration
settings, documented baselines, approved deviations, monitored changes, and
automated management/application/verification where required. The transferable
posture is:

- organization-managed policy has stable identity and version;
- authority to issue it is explicit;
- effective managed values and deviations are inspectable;
- changes and failed/unauthorized attempts produce evidence;
- verification can compare intended and effective configuration;
- profiles tailor which settings are controlled.

This does **not** mean every cursor or color preference enters the Product
Revision journal. The descriptor must classify audit sensitivity. Machine-local
presentation history can remain lightweight, while managed security/release
policy requires stronger receipts and retention. Project engineering policy
continues to use the Project/Revision authorities ratified elsewhere.

## 12. Cross-source requirements carried to GP-C03 and GP-C04

The evidence supports the following questions/requirements; none is yet an
owner-approved Datum mechanism:

1. one typed descriptor registry with stable identities and explicit ownership;
2. separate value contributions, recommendations, constraints, and locks;
3. per-key allowed scopes and protected-scope eligibility;
4. one resolver returning effective value plus complete provenance/refusals;
5. explicit distinction among preference, restartable workspace state,
   transient session state, Project policy, and operation input;
6. copy-on-create Project policy seeding with a receipt and no silent live link;
7. platform-correct config/state locations and strict invalid-path handling;
8. durable generation-based local writes, recovery, and single-writer control;
9. versioned deterministic migrations with last-known-good preservation;
10. opaque preservation of unknown keys/providers/versions;
11. typed import/export envelopes with redaction and integrity metadata;
12. expected-generation mutation and explicit three-way conflict records;
13. descriptor-owned merge laws instead of generic last-writer-wins;
14. managed policy that can recommend, constrain, or lock with visible reason;
15. accessible editing and provenance across writable, managed, invalid,
    unavailable, migrating, conflicting, and recovery states;
16. audit strength proportional to setting sensitivity and governing profile;
17. complete local/offline resolution from cached/installed authorities, with
    provider absence never fabricating a policy removal;
18. no external provider becomes engineering or release authority over Datum.

## 13. Open owner decisions for GP-C03

The research intentionally leaves these choices open:

1. the exact Datum scope lattice and whether session/context contributions can
   override an explicit user value for each setting class;
2. organization recommendation versus hard constraint/lock semantics;
3. whether managed policy is installed-only in v1 or also supports a signed
   subordinate adapter package;
4. how long last-known managed policy remains effective while its provider is
   unavailable, and how expiry/revocation is represented;
5. which preferences may seed Project policy and what receipt is stored;
6. the canonical local store shape and recovery generation count;
7. import/export and sync inclusion defaults, especially terminal launch and
   security-sensitive values;
8. conflict presentation and whether any setting classes have safe automatic
   joins;
9. first-Release onboarding completion scope: user, installation, Project,
   organization, or a typed combination;
10. exact preference/profile controls for the Revision carry-forwards.

Each decision requires consequences and a recommendation before owner
disposition. Visual questions belong in GP-C05 and the Claude-owned prototype
workflow, not in this research report.

## 14. Research conclusion

No surveyed authority supports treating preferences as a flat JSON object with
“last file wins.” Mature systems separate typed schema, value layers,
provenance, writability/locks, and durable storage. Standards for exchange and
concurrent updates further show that validation, patching, canonicalization,
and conflict prevention are distinct mechanisms.

The most evidence-aligned Datum direction is therefore a locally complete,
typed resolver over distinct contributions and constraints, with explicit
provenance, Project-seed isolation, durable generational persistence, unknown
preservation, expected-generation mutation, accessible managed-state
presentation, and subordinate external adapters. GP-C03 must convert that
direction into one-question-at-a-time owner decisions before it becomes
architecture.

<!-- EVIDENCE:GLOBAL-PREFERENCES-SPEC:GP-C02-EXTERNAL-RESEARCH -->
