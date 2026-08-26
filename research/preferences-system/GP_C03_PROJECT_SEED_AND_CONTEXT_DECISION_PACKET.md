# GP-C03 Project Seeding and Context Decision Packet

> **Status:** Q5 owner-decision packet; Q1–Q4 are approved. Specification only.
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

The visual-anchor law assigns Q5 to PX-V7. A materially different interaction
must be rendered before an owner boundary opens.

## 2. GP-C03-Q5 — new-Project seeding and receipt

<!-- OWNER:GLOBAL-PREFERENCES-SPEC:GP-C03:GP-C03-Q5 -->

### 2.1 Decision being made

Q5 decides how eligible defaults cross from preference/configuration sources
into governed Project authority at Project genesis, whether they remain linked,
and what durable evidence records the transfer. It does not let the Preferences
store mutate an existing Project, decide later provider availability (Q7), or
specify persistence/package transport (GP-C04).

### 2.2 Internal and external evidence

The approved Q2 contract defines `ProjectPolicySeed` as a setting class eligible
to initialize governed Project policy, while `ProjectPolicy` can be mutated only
through the Project authority (`GP_C03_TYPED_AUTHORITY_DECISION_PACKET.md`,
§6.3 and §6.7). Q3 makes organization directives inert beyond a user grant but
keeps Governed-Project law independent after it exists (§8.4). Q4 resolves only
eligible sources and preserves provenance (§9.3 and §9.5).

The professional-tool addendum shows Altium templates seeding design rules,
title blocks, release options, and output jobs, and Revit templates/Transfer
Project Standards deliberately copying standards into Project-owned state
(`GP_C02B_DOMAIN_PEER_PREFERENCES_RESEARCH.md:174-215`). Its bounded synthesis
requires versioned Project-policy seed material, explicit receipts, and no
silent rewrite of existing Projects (lines 295–313). These are mechanisms, not
authority to copy either peer's connected-service model.

### 2.3 Reviewed visual evidence — PX-V7

**Visual authority:**
`docs/gui/prototypes/preferences-ux-study.html#px-v7`, source lines 360–372 at
commit `7c41f65`. PX-V7 renders one genesis receipt naming the source profile and
generation, copied values, descriptor-ineligible omissions, and the post-copy
law: Project policy is Project truth; later global changes propose and never
rewrite. The durable receipt keeps origin answerable.

### 2.4 Candidate Q5-A/PX-V7 — explicit snapshot, atomic copy, Project ownership

Approval establishes only these clauses:

1. A descriptor explicitly declares `ProjectPolicySeed` eligibility and the
   destination Project-policy identity/type. Presentation, Capability, and other
   ineligible keys cannot cross this boundary.
2. Project genesis resolves one explicit seed selection to an immutable source
   snapshot. The Project mutation path atomically copies eligible values into
   Project authority and writes a durable `ProjectSeedReceipt`; the Preferences
   store never writes Project facts directly.
3. The receipt records Project identity, source provider/profile/package and
   generation, actor/time, each copied source and destination key/value, and
   every omitted/refused key with its reason. It is evidence, not a live link.
4. After commit, copied values are `ProjectPolicy` owned by the Project. Later
   user, installation, organization, descriptor-default, package, or preference
   changes cannot silently rewrite them.
5. A later source generation may produce a typed comparison/proposal. Uptake is
   a deliberate governed Project mutation with its own provenance; “follow
   latest” is not a mode.
6. An organization may offer seed packages regardless of machine-management
   level. A user may explicitly select one for this genesis transaction without
   granting ongoing machine authority. Automatic organization-selected seeding
   requires sufficient `AuthorityRelease`; an excess request stays visible and
   inert.
7. Revoking `AuthorityRelease`, disconnecting a provider, or organization
   withdrawal never rewrites or invalidates already-copied Project policy. The
   Project's own later governance decides whether change is required.
8. Factory and user-selected seeds use the same typed copy/receipt mechanism;
   profiles may require particular Project facts, but they do not create a
   second Project-policy engine.
9. Q5 does not decide package signing/transport, unavailable-provider behavior,
   migration, detailed comparison UX, or implementation dependencies.

### 2.5 Recommendation and owner response

Approve **Q5-A/PX-V7**. It matches the rendered receipt, preserves the Project
as authority, makes every omission inspectable, and lets an explicit one-time
seed choice remain distinct from ongoing organization management.

Reply exactly:

```text
GP-C03-Q5: approve Q5-A/PX-V7
```

or:

```text
GP-C03-Q5: revise — <required seeding or receipt correction>
```

<!-- EVIDENCE:GLOBAL-PREFERENCES-SPEC:GP-C03-Q5-PACKET -->
