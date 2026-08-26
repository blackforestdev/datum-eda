# GP-C03 Typed Authority Decision Packet

> **Status:** architecture decision packet in progress. Only the question named
> as the current owner boundary is open for disposition. No Preferences
> implementation, dependency, provider, storage format, synchronization service,
> or standards-conformance claim is authorized.
>
> **Tracker:** `dat-global-preferences-engine-qcv`
> **Frontier:** `GLOBAL-PREFERENCES-SPEC / GP-C03`
> **Approved factual baseline:** `GP-C01-BASELINE`
> **External evidence:** `GP_C02_EXTERNAL_AND_STANDARDS_RESEARCH.md`

## 1. Purpose and decision discipline

GP-C03 defines Datum's typed preference authority and effective-value resolver.
The decisions are sequenced because later questions depend on earlier terms.
Only one question is presented to the owner at a time. An approval applies only
to the exact contract under that question; comparative candidates remain
evidence and do not become alternate live modes.

This packet does not define persistence, migration mechanics, synchronization,
or final visual behavior. Those belong to GP-C04 and GP-C05 after the authority
model is stable.

## 2. Evidence foundation

### 2.1 Internal facts

The owner-approved GP-C01 audit establishes that Datum currently has no Global
Preferences engine. It has one narrow Console-owned file, independent launch
and environment inputs, session-only GUI state, and authoritative Project facts
that must not be silently overridden by machine-local preferences
(`GP_C01_INTERNAL_AUTHORITY_AUDIT.md:260-280,282-358,364-372`).

In particular:

- no typed preference registry or unified value schema exists;
- no common scope, precedence, provenance, constraint, or refusal model exists;
- opening the future Preferences UI must not rewrite unrelated or unknown data;
- Project rules, waivers, manufacturing plans, variants, and similar design
  facts are Project authority rather than user preferences;
- managed Revision visibility and first-Release onboarding state are required
  but absent.

### 2.2 External and standards evidence

The GP-C02 synthesis finds the following transferable mechanisms:

- stable setting identity is separate from localized labels and storage
  location (`GP_C02_EXTERNAL_AND_STANDARDS_RESEARCH.md:73-100`);
- a descriptor declares type, default/no-value state, constraints, allowed
  scopes, merge behavior, security/export classification, accessibility facts,
  and migration identity (`GP_C02_EXTERNAL_AND_STANDARDS_RESEARCH.md:83-97`);
- factory defaults, recommendations, explicit values, and managed values remain
  distinguishable (`GP_C02_EXTERNAL_AND_STANDARDS_RESEARCH.md:102-117`);
- validation and migration are different mechanisms
  (`GP_C02_EXTERNAL_AND_STANDARDS_RESEARCH.md:119-125`);
- value precedence and constraint authority are different axes
  (`GP_C02_EXTERNAL_AND_STANDARDS_RESEARCH.md:127-150`);
- provenance and losing/refused contributions must be queryable facts rather
  than GUI inference (`GP_C02_EXTERNAL_AND_STANDARDS_RESEARCH.md:152-168`);
- protected settings need declared scope eligibility
  (`GP_C02_EXTERNAL_AND_STANDARDS_RESEARCH.md:170-181`).

The sources support these mechanisms but do not prescribe Datum's exact key
syntax, scope lattice, provider transport, storage format, or dependency.

### 2.3 Standing owner boundaries

The governing plan requires:

1. machine/user, organization, Project, session, and contextual values to have
   typed scope, precedence, provenance, validation, and refusal;
2. new-Project seeding to copy policy into Project authority without a silent
   continuing global link;
3. presentation preferences never to hide audit facts, discard records, weaken
   release gates, or alter engineering meaning;
4. organization-managed policy to be capable of pinning Revision visibility on
   with disclosed provenance and reason;
5. first Release to be a specified accessible onboarding moment;
6. locally complete operation and no new dependency without Product Mechanics
   029 owner approval.

These are constraints on the questions below, not answers to them.

## 3. Terms used by the packet

The following names are provisional until their defining question is approved:

- **PreferenceKey:** stable machine identity for one setting concept.
- **PreferenceDescriptor:** the authoritative schema and behavior contract for
  one `PreferenceKey`.
- **PreferenceContribution:** a typed value offered for a key by one identified
  provider and scope.
- **PreferenceConstraint:** a typed limit or lock affecting which
  contributions may be stored or selected.
- **EffectivePreference:** the resolver's selected value plus its complete
  provenance and disposition facts.
- **Project policy:** governed Project authority, distinct from a machine/user
  preference even when initialized from a preference profile.
- **restartable state:** non-policy state needed to restore the user's working
  view, such as pane layout or open documents.
- **operation input:** a choice explicitly supplied to one invocation or
  transaction.

## 4. Planned owner-decision sequence

| Question | Decision | Depends on |
|---|---|---|
| `GP-C03-Q1` | Stable identity and descriptor ownership | GP-C01, GP-C02 |
| `GP-C03-Q2` | Scope identities and setting classes | Q1 |
| `GP-C03-Q3` | Recommendation, constraint, and lock model | Q1, Q2 |
| `GP-C03-Q4` | Effective-value precedence and conflict disposition | Q1-Q3 |
| `GP-C03-Q5` | New-Project policy seeding and receipt | Q1-Q4 |
| `GP-C03-Q6` | Session/context contributions versus operation input and restartable state | Q1-Q4 |
| `GP-C03-Q7` | Validation, refusal, protected scopes, and unavailable providers | Q1-Q4 |
| `GP-C03-Q8` | Unknown, retired, aliased, and unavailable setting identities | Q1-Q4 |
| `GP-C03-Q9` | Resolver query and provenance contract | Q1-Q8 |
| `GP-C03-Q10` | Revision carry-forward preference/state identities | Q1-Q9 |

This order is not approval of the question count or candidate answers. A
question may be split when owner review reveals more than one independent
choice.

## 5. GP-C03-Q1 — stable identity and descriptor ownership

<!-- OWNER:GLOBAL-PREFERENCES-SPEC:GP-C03:GP-C03-Q1 -->

### 5.1 The problem

Datum needs to identify a preference consistently across GUI labels, CLI
inspection, managed policy, export, migration, and future versions. Scope or
provider cannot be allowed to redefine what the setting means: a user value and
an organization recommendation for Revision visibility must still address the
same setting, have the same value type, and resolve through one contract.

The decision is whether identity and schema belong to one registered setting or
are recreated by each provider/storage layer.

### 5.2 Candidate A — one stable key and one authoritative descriptor

Each setting concept has exactly one stable `PreferenceKey` and one active
`PreferenceDescriptor` owned by its Datum subsystem. Factory defaults,
organization recommendations, explicit user values, contextual values, and
later Project-seed inputs are typed contributions to that identity. Scope,
provider, file path, GUI label, and current value are metadata or contributions;
none is the setting's identity.

Consequences:

- all providers must use the descriptor's type and validation law;
- one query can compare contributions and explain the effective value;
- localized labels and UI reorganization cannot break policy or migration;
- a provider cannot create a rival meaning for an existing key;
- aliases can redirect retired names during migration, but only one key remains
  live after migration;
- the registry becomes explicit infrastructure that must reject duplicate or
  incompatible registrations.

### 5.3 Candidate B — provider-owned schemas for nominally matching keys

Each provider may define its own key schema, and resolution attempts to match
keys by name or mapping.

Consequences:

- an organization provider can disagree with the application about type,
  allowed values, default, or semantics;
- provenance may identify the provider but cannot prove that compared values
  mean the same thing;
- migration becomes a many-provider coordination problem;
- unavailable providers can remove the only schema needed to interpret stored
  values;
- this model creates multiple authorities for one product behavior.

No surveyed source supplies evidence that this ambiguity is desirable.

### 5.4 Candidate C — storage/UI paths are the setting identity

The JSON path, Preferences-page location, or similar presentation path is the
identity and its current serialized value determines the effective type.

Consequences:

- moving a control or reorganizing a file becomes an identity migration;
- labels, storage, schema, and authority become entangled;
- defaults are likely to be serialized as if explicitly chosen;
- unknown values are difficult to preserve without accidentally activating
  them;
- managed policy and user values can appear comparable while carrying no common
  schema contract.

This resembles the flat-object failure mode rejected by GP-C02.

### 5.5 Recommendation

Adopt **Candidate A**.

It is the only candidate consistent with Datum's one-authority architecture and
the cross-source evidence. It also preserves a simple user experience: users
see one preference, while the engine retains the contributions and provenance
needed for individual, teaching, and enterprise environments.

### 5.6 Exact contract approved if Q1-A is accepted

Approval of Q1-A establishes only these clauses:

1. Every effective Datum preference resolves through one stable,
   non-localized `PreferenceKey`.
2. Every live key has exactly one authoritative active
   `PreferenceDescriptor`, owned by the subsystem that owns the behavior.
3. The descriptor declares at minimum the value type/canonical form,
   factory-default or no-value state, validation constraints, allowed scope
   classes, merge category, apply/restart behavior, affected consumers,
   security/export classification, accessible presentation metadata, schema
   version, and retirement/migration identity.
4. Factory defaults remain descriptor facts and are never stored as explicit
   user choices merely because a UI or query reads them.
5. Providers and scopes contribute values or constraints to a registered key;
   they do not redefine that key's meaning or value type.
6. Storage path, provider identity, scope, GUI location, localized label, and
   current value are not `PreferenceKey` identity.
7. Duplicate or incompatible active descriptor registration is refused.
8. A retired identifier may be an explicit migration alias, but it cannot
   remain a second live preference identity after successful migration.
9. An unregistered/unknown stored identity is never made effective by guessing
   its type. Its preservation and later reactivation rules remain GP-C03-Q8 and
   GP-C04 decisions.
10. This decision does not select the exact key encoding, scope lattice,
    precedence, policy-provider transport, persistence format, dependency, or
    user interface.

### 5.7 Owner response

Approve only if the clauses in §5.6 establish the correct identity/schema
foundation without deciding the later scope, policy, storage, or UX questions.

Reply exactly:

```text
GP-C03-Q1: approve Q1-A
```

or:

```text
GP-C03-Q1: revise — <missing, incorrect, or overreaching clause>
```

<!-- EVIDENCE:GLOBAL-PREFERENCES-SPEC:GP-C03-Q1-PACKET -->

### 5.8 Owner disposition

<!-- EVIDENCE:GLOBAL-PREFERENCES-SPEC:GP-C03-Q1-APPROVED -->

**Approved 2026-08-25 — Q1-A.** The owner approved the exact §5.6 contract.
Datum preferences therefore use one stable `PreferenceKey` and one
authoritative subsystem-owned descriptor; providers and scopes may contribute
typed values or constraints but cannot redefine the setting. The approval does
not select the scope taxonomy, precedence, managed-policy transport,
persistence, dependency, or UX.

## 6. GP-C03-Q2 — scope identities and setting classes

<!-- OWNER:GLOBAL-PREFERENCES-SPEC:GP-C03:GP-C03-Q2 -->

### 6.1 The problem

The word “scope” is currently carrying too many meanings. A factory default, a
user's theme selection, an organization lock, a Project's released-document
policy, a restored pane layout, and an export destination do not have the same
authority or lifecycle merely because each can affect what the user sees or
does.

Datum needs one resolver capable of explaining all inputs that affect a
registered key without turning every input into a writable Global Preference.
This question selects the source taxonomy and the minimum setting classes. It
does **not** select source precedence or managed-lock choreography.

### 6.2 Evidence

The owner-approved internal audit establishes three hard separations:

- authored Project rules and release/manufacturing facts are governed Project
  authority, not machine-local preferences
  (`GP_C01_INTERNAL_AUTHORITY_AUDIT.md:260-280`);
- pane layout, open panes, focus, camera, and similar values are currently
  session state rather than preferences
  (`GP_C01_INTERNAL_AUTHORITY_AUDIT.md:282-358`);
- no common typed scope model exists today
  (`GP_C01_INTERNAL_AUTHORITY_AUDIT.md:342-372`).

GP-C02 adds that configuration, restartable state, transient state, Project
policy, and operation input require different authority and retention laws
(`GP_C02_EXTERNAL_AND_STANDARDS_RESEARCH.md:195-231`). Protected settings also
need descriptor-declared source eligibility so an untrusted Project cannot
choose executable paths or trust roots
(`GP_C02_EXTERNAL_AND_STANDARDS_RESEARCH.md:170-181`).

### 6.3 Candidate A — one resolver, typed source families, strict exclusions

The effective-value resolver accepts typed `ResolutionSource` facts, but only a
subset are writable `PreferenceScope` contributions:

| Source family | Meaning | Writable as a Global Preference? |
|---|---|---|
| `DescriptorDefault` | factory fallback declared by Q1's descriptor | no |
| `Installation` | local installation/machine contribution | yes, when the descriptor permits |
| `Organization` | identified organization recommendation, constraint, or managed contribution | through its managed provider, not ordinary user editing |
| `User` | explicit choice for one user | yes, when permitted |
| `ProjectPolicy` | governed fact copied/authored into the Project | no; mutated only through Project authority |
| `Session` | explicit ephemeral override for the running session | only where Q6 permits |
| `Context` | read-only applicable context supplied during resolution | no; Q6 defines its exact law |

The descriptor also classifies each key by what it controls:

1. `Presentation` — visual/interaction presentation without engineering meaning;
2. `WorkflowDefault` — a remembered starting choice that never conceals or
   replaces an explicit operation input;
3. `Capability` — executable, environment, resource, agent, endpoint, or trust
   configuration requiring protected-source eligibility;
4. `ProjectPolicySeed` — a value eligible to initialize governed Project policy
   through the explicit Q5 copy/receipt boundary.

Restartable workspace state, transient interaction state, and explicit
operation input remain separate data classes outside the preference store.
They may be inspected alongside resolution where useful, but are never silently
promoted into preferences. Remembering an operation's last choice requires a
separate registered `WorkflowDefault` key.

Consequences:

- one resolver can explain a Project policy value without granting the
  Preferences UI authority to edit it;
- a descriptor can permit User/Organization sources for a theme while refusing
  Project control of executable paths;
- persistence technology may be shared later, but authority and reset behavior
  remain distinct;
- source identity says *who/where*, setting class says *what kind of effect*;
  neither silently establishes precedence;
- new setting classes or source kinds require governed schema evolution rather
  than ad hoc strings.

### 6.4 Candidate B — one flat ordered scope stack

Factory, installation, organization, user, Project, session, context, and
operation values are all writable layers in one last-winner stack.

Consequences:

- explicit operation input can be mistaken for persistent policy;
- Project facts become editable through a machine-local Preferences surface;
- a specific context may override a protected capability solely because it is
  “closer”;
- restartable layout state and engineering policy inherit the same reset,
  export, and synchronization law;
- the stack answers precedence before Datum has defined constraints, trust, or
  applicability.

This is simple to draw but false to Datum's authority boundaries.

### 6.5 Candidate C — separate preference engines per subsystem

GUI, Terminal, Revision, export, and other subsystems retain independent scope
models and expose their own effective-value queries.

Consequences:

- each subsystem can choose locally convenient rules;
- organization management must integrate with several incompatible models;
- provenance and refusal wording drift;
- cross-cutting settings such as reduced motion, Revision visibility, and
  terminal capability cannot be inspected through one authority;
- it preserves the fragmentation proved by GP-C01.

### 6.6 Recommendation

Adopt **Candidate A**.

It gives Datum one explainable resolver without pretending that everything the
resolver considers is a Global Preference. This is the smallest model that
preserves user simplicity, Project authority, protected capability, managed
environments, and explicit operation control at the same time.

### 6.7 Exact contract approved if Q2-A is accepted

Approval of Q2-A establishes only these clauses:

1. `ResolutionSource` is the umbrella for facts considered by effective-value
   resolution; being a resolution source does not grant write authority.
2. The initial source families are `DescriptorDefault`, `Installation`,
   `Organization`, `User`, `ProjectPolicy`, `Session`, and `Context`.
3. `DescriptorDefault` is immutable descriptor data, not a persisted preference
   scope.
4. `Installation`, `Organization`, and `User` are persistent preference-source
   scopes; their exact provider, precedence, recommendation, constraint, and
   lock laws remain Q3, Q4, and GP-C04 decisions.
5. `ProjectPolicy` participates only for descriptors that explicitly admit
   Project authority. Its values are read through the resolver but mutated only
   through the Project's governed mutation path, never by the Global Preferences
   store or UI.
6. `Session` and `Context` are reserved typed sources whose applicability,
   mutability, lifetime, and precedence remain Q6 decisions.
7. Every descriptor declares allowed source families and one setting class.
   Ineligible contributions are refused rather than silently considered.
8. The initial setting classes are `Presentation`, `WorkflowDefault`,
   `Capability`, and `ProjectPolicySeed`. Additional classes require governed
   schema evolution, not provider-defined strings.
9. Restartable workspace state, transient interaction state, and explicit
   operation input are not preference scopes. Remembering an operation choice
   requires a separately registered `WorkflowDefault` preference.
10. Source family and setting class do not imply precedence. Q2 approval does
    not decide which eligible contribution wins, how managed conflicts resolve,
    or whether a value can be locked.
11. This decision does not select storage locations/formats, synchronization,
    provider transport, dependencies, or final UI.

### 6.8 Owner response

Approve only if Candidate A gives Datum one coherent resolution vocabulary
while keeping Project mutation, restartable state, transient state, and explicit
operation input outside the writable preference-scope lattice.

Reply exactly:

```text
GP-C03-Q2: approve Q2-A
```

or:

```text
GP-C03-Q2: revise — <missing, incorrect, or overreaching clause>
```

<!-- EVIDENCE:GLOBAL-PREFERENCES-SPEC:GP-C03-Q2-PACKET -->
