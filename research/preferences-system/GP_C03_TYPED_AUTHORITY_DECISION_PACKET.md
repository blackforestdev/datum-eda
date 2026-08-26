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
